//! Storage migrations for `pallet-belize-bns`.
//!
//! ## v0 → v1: hosting expiry index (spec 127)
//!
//! `HostingInfo.expires_at` existed from the first release and was read by
//! nobody — the pallet gated only its own mutating calls, so a lapsed
//! subscription kept being served by the read path indefinitely. Spec 127 adds
//! `HostingExpiries`, a `block → [domain]` index that lets `on_initialize` find
//! lapsed subscriptions with bounded work.
//!
//! The index is derived state: it can be rebuilt from `HostedWebsites` at any
//! time, so there is nothing to preserve and nothing to roll back. This
//! migration walks the existing records once and indexes the ones that carry a
//! recurring charge.
//!
//! **Free subscriptions are deliberately not indexed.** Their monthly fee is 0,
//! so there is no payment to miss and expiring them would revoke service for
//! non-payment of nothing. That also means a chain whose hosted domains are all
//! free — which is the state of Jade on 2026-11-17 — ends this migration with an
//! empty index, exactly as intended.

use crate::{Config, HostedWebsites, HostingExpiries, Pallet};
use frame_support::{
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use {
    codec::{Decode, Encode},
    sp_runtime::TryRuntimeError,
    sp_std::vec::Vec,
};

/// Target storage version after this migration.
pub const TARGET_VERSION: u16 = 1;

/// Backfill `HostingExpiries` from the existing `HostedWebsites` records.
pub struct MigrateV0ToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = StorageVersion::get::<Pallet<T>>();
        if on_chain >= StorageVersion::new(TARGET_VERSION) {
            log::info!(
                "bns: already at storage v{:?}, skipping hosting expiry index migration",
                on_chain
            );
            return T::DbWeight::get().reads(1);
        }

        let mut scanned: u64 = 0;
        let mut indexed: u64 = 0;

        for (domain, info) in HostedWebsites::<T>::iter() {
            scanned = scanned.saturating_add(1);
            if info.monthly_fee == 0 {
                continue;
            }

            // Shaped the same way as `index_expiry`, minus the extrinsic error:
            // a migration cannot fail the block, so an over-full bucket is
            // reported and the remaining domains are still visited.
            let mut bucket = HostingExpiries::<T>::get(info.expires_at);
            match bucket.try_push(domain) {
                Ok(()) => {
                    HostingExpiries::<T>::insert(info.expires_at, bucket);
                    indexed = indexed.saturating_add(1);
                }
                Err(_) => log::warn!(
                    "bns: expiry bucket full at {:?}; a paid subscription was left unindexed",
                    info.expires_at
                ),
            }
        }

        StorageVersion::new(TARGET_VERSION).put::<Pallet<T>>();

        log::info!(
            "bns: indexed {indexed} of {scanned} hosted domains for expiry sweep \
             (free subscriptions are never indexed)"
        );

        T::DbWeight::get().reads_writes(scanned.saturating_add(2), indexed.saturating_add(1))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        let on_chain = StorageVersion::get::<Pallet<T>>();
        let paid = HostedWebsites::<T>::iter()
            .filter(|(_, info)| info.monthly_fee > 0)
            .count();
        log::info!("bns pre_upgrade: storage v{on_chain:?}, {paid} paid subscriptions to index");
        Ok((on_chain, paid as u32).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let (_pre_version, pre_paid): (StorageVersion, u32) =
            Decode::decode(&mut &state[..]).map_err(|_| "bns: undecodable state")?;

        let on_chain = StorageVersion::get::<Pallet<T>>();
        if on_chain != StorageVersion::new(TARGET_VERSION) {
            return Err("bns: storage version not advanced to v1".into());
        }

        // Every record that can lapse must be reachable by the sweep; this is
        // the invariant the migration exists to establish.
        let indexed: u32 = HostingExpiries::<T>::iter_values()
            .map(|bucket| bucket.len() as u32)
            .sum();
        if indexed != pre_paid {
            return Err("bns: indexed count does not match paid subscriptions".into());
        }

        // A free record must never be reachable, or the sweep would revoke a
        // service nobody was charged for.
        for (domain, info) in HostedWebsites::<T>::iter() {
            if info.monthly_fee == 0
                && HostingExpiries::<T>::get(info.expires_at)
                    .map(|bucket| bucket.contains(&domain))
                    .unwrap_or(false)
            {
                return Err("bns: a free subscription was indexed for expiry".into());
            }
        }

        log::info!("bns post_upgrade: {indexed} subscriptions indexable for expiry");
        Ok(())
    }
}
