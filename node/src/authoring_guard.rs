//! Authoring guards for validator nodes.
//!
//! ## Why this exists
//!
//! When a Substrate validator cannot author, it fails *silently*. The BABE slot
//! worker emits `Attempting to claim slot N` at **debug** level (invisible at
//! the default `info` level), the claim returns `None`, and the slot worker
//! returns without a single further log line. The process stays up, the HTTP
//! healthcheck passes, and Docker reports the container as healthy — while the
//! chain stops producing blocks.
//!
//! That is exactly what happened to the Ceiba testnet on 2026-09-21: a restored
//! chain snapshot could not author a single block, and nothing in the logs said
//! so. Hours were lost proving it was even stalled.
//!
//! This module makes that class of failure impossible to miss:
//!
//! 1. [`log_authoring_preflight`] — at startup, cross-check the on-chain BABE
//!    and GRANDPA authority sets against the keys actually present in this
//!    node's keystore and print one banner. A validator without a matching key
//!    gets an ERROR that names the problem and the fix.
//! 2. [`authoring_watchdog`] — at runtime, escalate to ERROR (with a
//!    diagnostic snapshot) if this validator holds matching keys yet no block
//!    has been imported for [`STALL_ALERT_SLOTS`] slots.
//!
//! Neither guard changes consensus behaviour: they only observe and log.

use crate::service::FullClient;
use log::{error, info, warn};
use sc_client_api::HeaderBackend;
use sc_keystore::Keystore;
use sc_service::Role;
use sp_api::ProvideRuntimeApi;
use sp_consensus_babe::BabeApi;
use sp_consensus_grandpa::GrandpaApi;
use sp_core::ByteArray;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Log target shared by every guard message, so operators can filter on it.
const LOG_TARGET: &str = "authoring-guard";

/// Warn once a validator has gone this many slots without importing a block.
/// At a 6 s slot this is 10 minutes — roughly the point where a missed block
/// window stops looking like ordinary jitter.
pub const STALL_ALERT_SLOTS: u64 = 100;

/// While the stall continues, repeat the diagnostic at most this often.
const STALL_ALERT_INTERVAL: Duration = Duration::from_secs(300);

/// Minimum interval between watchdog samples.
const WATCHDOG_SAMPLE_INTERVAL: Duration = Duration::from_secs(30);

/// On-chain authority sets versus the keys this node actually holds.
#[derive(Debug, Clone, Copy)]
pub struct KeyStatus {
    /// BABE authorities at the head's epoch.
    pub babe_on_chain: usize,
    /// GRANDPA authorities at the head.
    pub grandpa_on_chain: usize,
    /// Local BABE keys that match an on-chain BABE authority.
    pub babe_matched: usize,
    /// Local GRANDPA keys that match an on-chain GRANDPA authority.
    pub grandpa_matched: usize,
}

impl KeyStatus {
    /// The node can author only with a matching BABE key; GRANDPA is reported
    /// separately because a missing GRANDPA key stalls finality, not block
    /// production.
    pub fn can_author(&self) -> bool {
        self.babe_matched > 0
    }
}

/// Cross-check the head's on-chain authority sets against the local keystore.
pub fn key_status(client: &Arc<FullClient>, keystore: &Arc<dyn Keystore>) -> KeyStatus {
    let head = client.info().best_hash;
    let api = client.runtime_api();

    let babe_on_chain = api
        .current_epoch(head)
        .map(|epoch| epoch.authorities)
        .unwrap_or_default();
    let grandpa_on_chain = api.grandpa_authorities(head).unwrap_or_default();

    let local_babe = keystore.sr25519_public_keys(sp_consensus_babe::KEY_TYPE);
    let local_grandpa = keystore.ed25519_public_keys(sp_consensus_grandpa::KEY_TYPE);

    let babe_matched = babe_on_chain
        .iter()
        .filter(|(id, _)| local_babe.iter().any(|key| key.as_slice() == id.as_slice()))
        .count();
    let grandpa_matched = grandpa_on_chain
        .iter()
        .filter(|(id, _)| {
            local_grandpa
                .iter()
                .any(|key| key.as_slice() == id.as_slice())
        })
        .count();

    KeyStatus {
        babe_on_chain: babe_on_chain.len(),
        grandpa_on_chain: grandpa_on_chain.len(),
        babe_matched,
        grandpa_matched,
    }
}

/// Startup banner: can this validator author, and with which keys?
///
/// Non-authority roles are ignored (they are not expected to hold keys).
pub fn log_authoring_preflight(client: &Arc<FullClient>, keystore: &Arc<dyn Keystore>, role: Role) {
    if !role.is_authority() {
        return;
    }

    let status = key_status(client, keystore);
    let head = client.info();

    if status.can_author() {
        info!(
            target: LOG_TARGET,
            "authoring preflight: validator at #{} — BABE authorities on-chain {} / matched local keys {}; \
             GRANDPA authorities on-chain {} / matched local keys {}",
            head.best_number,
            status.babe_on_chain,
            status.babe_matched,
            status.grandpa_on_chain,
            status.grandpa_matched,
        );
    } else {
        error!(
            target: LOG_TARGET,
            "AUTHORING PREFLIGHT FAILED: this node runs as a validator but holds NO keystore key that \
             matches an on-chain BABE authority (chain has {} BABE authorit{}, local matches {}). \
             Substrate will claim slots, find no usable key, and return without logging anything — \
             the chain will NOT produce blocks and this container will still report healthy. \
             Checked: BABE {} | GRANDPA {} at #{}.",
            status.babe_on_chain,
            if status.babe_on_chain == 1 { "y" } else { "ies" },
            status.babe_matched,
            status.babe_matched,
            status.grandpa_matched,
            head.best_number,
        );
    }

    if status.grandpa_on_chain > 0 && status.grandpa_matched == 0 {
        warn!(
            target: LOG_TARGET,
            "authoring preflight: no local GRANDPA key matches the on-chain GRANDPA set \
             ({} authorit{} on-chain). Block production is unaffected, but finality will not advance.",
            status.grandpa_on_chain,
            if status.grandpa_on_chain == 1 { "y" } else { "ies" },
        );
    }
}

/// Watch block production and escalate loudly if this validator stalls.
///
/// Spawn with [`sc_service::TaskManager::spawn_handle`]. Returns immediately on
/// non-authority roles.
pub async fn authoring_watchdog(
    client: Arc<FullClient>,
    keystore: Arc<dyn Keystore>,
    role: Role,
    slot_duration: Duration,
) {
    if !role.is_authority() {
        return;
    }

    let sample_interval = WATCHDOG_SAMPLE_INTERVAL.max(slot_duration);
    let slot_millis = slot_duration.as_millis().max(1) as u64;

    let mut last_best = client.info().best_number;
    let mut stalled_since: Option<Instant> = None;
    let mut last_alert: Option<Instant> = None;

    loop {
        tokio::time::sleep(sample_interval).await;

        let info = client.info();
        if info.best_number != last_best {
            if let Some(since) = stalled_since.take() {
                info!(
                    target: LOG_TARGET,
                    "block production resumed at #{} after a {:?} stall",
                    info.best_number,
                    since.elapsed(),
                );
            }
            last_best = info.best_number;
            continue;
        }

        let since = *stalled_since.get_or_insert_with(Instant::now);
        let stalled_slots = since.elapsed().as_millis() as u64 / slot_millis;
        if stalled_slots < STALL_ALERT_SLOTS {
            continue;
        }

        let due = last_alert
            .map(|last| last.elapsed() >= STALL_ALERT_INTERVAL)
            .unwrap_or(true);
        if !due {
            continue;
        }
        last_alert = Some(Instant::now());

        let status = key_status(&client, &keystore);
        let can_author = status.can_author();

        error!(
            target: LOG_TARGET,
            "STALL DETECTED: no block imported for {} slots (~{:?}). best #{}, finalized #{}. \
             Local authoring capability: {} (on-chain BABE authorities {}, matched local keys {}; \
             GRANDPA on-chain {}, matched {}). \
             {}",
            stalled_slots,
            since.elapsed(),
            info.best_number,
            info.finalized_number,
            if can_author { "keys present" } else { "NO MATCHING BABE KEY" },
            status.babe_on_chain,
            status.babe_matched,
            status.grandpa_on_chain,
            status.grandpa_matched,
            if can_author {
                "Block production has stopped even though this node holds the keys — collect the node \
                 logs and check the BABE slot-claim path (epoch/slot state) before restarting."
            } else {
                "This node cannot author at all: restore the keystore or re-authorise this account."
            },
        );
    }
}
