//! Storage migrations for `pallet-interoperability`.
//!
//! `ChainConfigurations` has no genesis config and no extrinsic that can create
//! an entry: `update_bridge_config` deliberately rejects a chain it does not
//! already hold (`Error::UnsupportedChain`), and no `add_bridge_chain` call
//! exists. A chain launched without the interoperability genesis patch — which
//! is the case on Ceiba — therefore has a permanently empty bridge: the picker
//! has nothing to offer and `initiate_bridge` fails with `UnsupportedChain` for
//! every chain index.
//!
//! This module seeds the configurations once, on the runtime upgrade that ships
//! it. Governance can adjust any seeded value afterwards through
//! `update_bridge_config`, and can disable a chain outright by setting
//! `enabled = false`.

use crate::{BridgeChain, ChainConfig, ChainConfigurations, Config};
use frame_support::{
    traits::{Get, OnRuntimeUpgrade},
    weights::Weight,
    BoundedVec,
};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

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

/// Seeds `ChainConfigurations` for [`SEEDED_CHAINS`].
pub struct SeedChainConfigurations<T>(PhantomData<T>);

impl<T: Config> SeedChainConfigurations<T> {
    /// The configuration written for every seeded chain.
    ///
    /// `rpc_endpoint` and `contract_address` are left empty on purpose: no
    /// relayer endpoint is registered for any chain, and no contract is deployed
    /// on the far side, so recording a placeholder would be a claim that cannot
    /// be backed.
    fn chain_config() -> ChainConfig {
        ChainConfig {
            enabled: true,
            min_confirmations: SEEDED_MIN_CONFIRMATIONS,
            max_amount: SEEDED_MAX_AMOUNT,
            fee_rate: T::BridgeFeeRate::get(),
            pq_signatures_required: T::PQSignatureThreshold::get(),
            rpc_endpoint: BoundedVec::default(),
            contract_address: None,
        }
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
