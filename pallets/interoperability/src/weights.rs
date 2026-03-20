//! Hand-estimated weights for pallet-belize-interoperability
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_interoperability using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: BridgeTransfers (r:1 w:1), NextTransferId (r:1 w:1),
    ///          Currency::reserve (r:1 w:1)
    fn initiate_bridge() -> Weight {
        Weight::from_parts(45_000_000, 3072)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: BridgeTransfers (r:1 w:1), Guardians (r:1 w:0),
    ///          Signatures (r:1 w:1)
    fn provide_signature() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: LiquidityPools (r:1 w:1), NextPoolId (r:1 w:1),
    ///          Currency::transfer (r:2 w:2)
    fn create_liquidity_pool() -> Weight {
        Weight::from_parts(50_000_000, 3584)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: BridgeTransfers (r:1 w:1), Currency::transfer (r:2 w:2)
    fn process_unlock() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: MessageOutbox (r:0 w:1), NextMessageId (r:1 w:1)
    fn send_message() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: BridgeConfig (r:0 w:1)
    fn update_config() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(0))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// P0-1: Storage: BridgeTransactions (r:1 w:0), BurnConfirmations (r:1 w:1),
    ///               BurnConfirmationCount (r:1 w:1)
    fn confirm_burn_proof() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
