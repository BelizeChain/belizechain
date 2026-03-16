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

    /// Storage: CurrentRound (r:1 w:1), ConsensusRounds (r:1 w:1),
    ///          total_issuance (r:1 w:0), GlobalAIMetrics (r:1 w:1)
    ///          Per-validator: ConsensusValidators get+mutate (r:2 w:1),
    ///          Currency::deposit_creating (r:1 w:1)
    /// DOS-014 FIX: Parameterized by MaxValidators (100) for O(N) reward loop.
    fn finalize_consensus_round() -> Weight {
        // Base cost: 4 reads + 3 writes
        let base = Weight::from_parts(80_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3));
        // Per-validator cost: 3 reads + 2 writes + 5M ref_time + 256 proof_size
        let per_validator = Weight::from_parts(5_000_000, 256)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2));
        // MaxValidators = 100
        base.saturating_add(per_validator.saturating_mul(100))
    }
}
