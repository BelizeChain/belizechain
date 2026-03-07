//! Hand-estimated weights for pallet-belize-consensus
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_consensus using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: AIModels (r:1 w:1), NextModelId (r:1 w:1)
    fn register_ai_model() -> Weight {
        Weight::from_parts(30_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Validators (r:1 w:1), Currency::reserve (r:1 w:1)
    fn join_validator() -> Weight {
        Weight::from_parts(35_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: ValidatorByAccount (r:1 w:1), ConsensusValidators (r:1 w:1), Currency::remove_lock (r:0 w:1)
    fn leave_validator() -> Weight {
        Weight::from_parts(35_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: AIModels (r:1 w:1), Validators (r:1 w:0)
    fn validate_model() -> Weight {
        Weight::from_parts(20_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: CurrentRound (r:1 w:1), Validators (r:1 w:0), RoundInfo (r:0 w:1)
    fn start_consensus_round() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: CurrentRound (r:1 w:0), Validators (r:1 w:0), AIWorkSubmissions (r:0 w:1)
    fn submit_ai_work() -> Weight {
        Weight::from_parts(45_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: CurrentRound (r:1 w:1), RoundInfo (r:1 w:1),
    ///          AIWorkSubmissions (r:N w:0), Validators (r:N w:N)
    /// O(N) — iterates validators
    fn finalize_consensus_round() -> Weight {
        Weight::from_parts(80_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}
