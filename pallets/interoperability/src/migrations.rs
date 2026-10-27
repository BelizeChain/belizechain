//! Storage migrations for `pallet-interoperability`.
//!
//! The bridge registry has two seed paths and they must agree:
//!
//! - **A chain launched at genesis** gets `ChainConfigurations` from this
//!   module's helpers via the pallet's `#[pallet::genesis_config]`. Without that
//!   path a launched chain starts empty, and nothing could fill it:
//!   `update_bridge_config` deliberately rejects a chain it does not already
//!   hold (`Error::UnsupportedChain`) and no `add_bridge_chain` call exists.
//! - **A chain upgraded from an earlier spec** gets them from
//!   [`SeedChainConfigurations`], the V1 → V2 step of the runtime's
//!   `CoordinatedUpgrade`.
//!
//! `on_runtime_upgrade` never runs on a fresh genesis, which is why both paths
//! exist. They derive from the same functions here, so a launched chain and an
//! upgraded chain cannot end up with different registries.
//!
//! Governance can adjust any seeded value afterwards through
//! `update_bridge_config`, and can disable a chain outright by setting
//! `enabled = false`.

use crate::{BridgeChain, ChainConfig, ChainConfigurations, Config};
use frame_support::{
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
    BoundedVec,
};
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};

/// Chains the bridge offers in the wallet.
///
/// Kept in step with `BRIDGE_CHAIN_CATALOGUE` in the UI
/// (`ui/maya-wallet/src/services/pallets/interoperability.ts`) so the picker and
/// the chain agree on what exists.
pub const SEEDED_CHAINS: &[BridgeChain] = &[
    BridgeChain::Bitcoin,
    BridgeChain::Ethereum,
    BridgeChain::BinanceSmartChain,
    BridgeChain::Solana,
    BridgeChain::Tron,
    BridgeChain::Polygon,
    BridgeChain::Polkadot,
    BridgeChain::Avalanche,
    BridgeChain::Near,
    BridgeChain::Sui,
    BridgeChain::Base,
    BridgeChain::ArbitrumOne,
    BridgeChain::Optimism,
];

/// Confirmation depth applied to every seeded chain.
///
/// One conservative value is used deliberately: inventing a different depth per
/// chain would be asserting a finality property we have not verified for each of
/// them. Governance can tune it per chain afterwards.
pub const SEEDED_MIN_CONFIRMATIONS: u32 = 12;

/// Per-transaction ceiling, in the 12-decimal base unit: 1,000,000 DALLA/bBZD.
pub const SEEDED_MAX_AMOUNT: u128 = 1_000_000 * 1_000_000_000_000;

/// The configuration written for every seeded chain.
///
/// `rpc_endpoint` and `contract_address` are left empty on purpose: no relayer
/// endpoint is registered for any chain, and no contract is deployed on the far
/// side, so recording a placeholder would be a claim that cannot be backed.
///
/// Lives outside `impl<T: Config>` so the chain specs can call it through
/// [`seeded_chain_configurations`] without naming the runtime.
pub fn seeded_chain_config(fee_rate: u32, pq_signatures_required: u32) -> ChainConfig {
    ChainConfig {
        enabled: true,
        min_confirmations: SEEDED_MIN_CONFIRMATIONS,
        max_amount: SEEDED_MAX_AMOUNT,
        fee_rate,
        pq_signatures_required,
        rpc_endpoint: BoundedVec::default(),
        contract_address: None,
    }
}

/// Every [`SEEDED_CHAINS`] entry paired with its configuration.
///
/// Both the genesis config and the seeding migration use this function, so a
/// chain launched at genesis and a chain upgraded from an earlier spec end up
/// with identical configurations.
pub fn seeded_chain_configurations(
    fee_rate: u32,
    pq_signatures_required: u32,
) -> Vec<(BridgeChain, ChainConfig)> {
    let config = seeded_chain_config(fee_rate, pq_signatures_required);
    SEEDED_CHAINS
        .iter()
        .map(|chain| (chain.clone(), config.clone()))
        .collect()
}

/// [`seeded_chain_configurations`] flattened into the primitive tuple shape the
/// genesis config takes, since `ChainConfig` is not serde-capable under `no_std`.
///
/// Tuple layout: `(chain_index, enabled, min_confirmations, max_amount,
/// fee_rate, pq_signatures_required)`.
pub fn seeded_chain_configuration_entries(
    fee_rate: u32,
    pq_signatures_required: u32,
) -> Vec<(u8, bool, u32, u128, u32, u32)> {
    let config = seeded_chain_config(fee_rate, pq_signatures_required);
    SEEDED_CHAINS
        .iter()
        .map(|chain| {
            (
                crate::chain_index(chain),
                config.enabled,
                config.min_confirmations,
                config.max_amount,
                config.fee_rate,
                config.pq_signatures_required,
            )
        })
        .collect()
}

/// Seeds `ChainConfigurations` for [`SEEDED_CHAINS`].
pub struct SeedChainConfigurations<T>(PhantomData<T>);

impl<T: Config> SeedChainConfigurations<T> {
    fn chain_config() -> ChainConfig {
        seeded_chain_config(T::BridgeFeeRate::get(), T::PQSignatureThreshold::get())
    }
}

impl<T: Config> OnRuntimeUpgrade for SeedChainConfigurations<T> {
    fn on_runtime_upgrade() -> Weight {
        let config = Self::chain_config();
        let mut written = 0u64;

        for chain in SEEDED_CHAINS {
            // Never overwrite an existing configuration: a chain already tuned
            // by governance must survive a re-run of this migration.
            if ChainConfigurations::<T>::contains_key(chain) {
                continue;
            }
            ChainConfigurations::<T>::insert(chain, config.clone());
            written += 1;
        }

        log::info!(
            "🌉 interoperability: seeded {written} of {} bridge chain configurations",
            SEEDED_CHAINS.len()
        );

        <T as frame_system::Config>::DbWeight::get()
            .reads_writes(SEEDED_CHAINS.len() as u64, written)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        Ok((ChainConfigurations::<T>::iter_keys().count() as u32).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let before = u32::decode(&mut &state[..]).map_err(|_| "bad pre-upgrade state")?;
        let after = ChainConfigurations::<T>::iter_keys().count() as u32;
        let expected = SEEDED_CHAINS.len() as u32;

        assert!(
            after >= expected,
            "bridge chain configurations were not seeded"
        );
        assert!(
            after >= before,
            "seeding must never remove an existing chain configuration"
        );
        Ok(())
    }
}
