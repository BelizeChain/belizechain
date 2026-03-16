//! Hand-estimated weights for pallet-belize-staking
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_staking using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: Validators (r:1 w:1), ValidatorCount (r:1 w:1),
    ///          Currency::reserve (r:1 w:1)
    fn join_validators() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: Validators (r:1 w:1), UnbondingQueue (r:0 w:1)
    fn leave_validators() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: UnbondingQueue (r:1 w:1), Currency::unreserve (r:1 w:1)
    fn withdraw_unbonded() -> Weight {
        Weight::from_parts(25_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Validators (r:1 w:0), FLTasks (r:1 w:0),
    ///          ModelDeltas (r:1 w:1)
    fn submit_model_delta() -> Weight {
        Weight::from_parts(35_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: FLTasks (r:0 w:1), NextTaskId (r:1 w:1)
    fn assign_fl_task() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Validators iteration (r:N w:0), Currency::deposit (r:N w:N),
    ///          RewardRecords (r:0 w:N), MaxSupply (r:1), TotalIssuance (r:N)
    /// DOS-002 FIX: O(N) — parameterized by MaxValidators to account for
    /// per-validator reads (Validators iter + ModelSubmissions get + total_issuance)
    /// and writes (deposit_creating + ModelSubmissions clear + QuantumStats clear).
    fn distribute_rewards() -> Weight {
        // Base cost: EpochRewards write, CurrentEpoch read+write, MaxSupply read,
        //            ModelSubmissions::clear, QuantumStats::clear, EpochQuantumJobs::kill
        let base = Weight::from_parts(80_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(5));
        // Per-validator cost: Validators read, ModelSubmissions read, total_issuance read,
        //                     deposit_creating write
        let per_validator = T::DbWeight::get().reads(3)
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(5_000_000, 256));
        // MaxValidators = 100 (runtime constant)
        base.saturating_add(per_validator.saturating_mul(100))
    }

    /// Storage: Validators (r:1 w:0), QuantumContributions (r:1 w:1),
    ///          ContributionStats (r:1 w:1)
    fn record_quantum_contribution() -> Weight {
        Weight::from_parts(30_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: OperatorStats (r:1 w:1), EpochStats (r:1 w:1)
    fn record_domain_contribution() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Validators (r:1 w:0), DomainStats (r:1 w:0),
    ///          Currency::deposit (r:1 w:1)
    fn claim_pouw_with_domain_bonus() -> Weight {
        Weight::from_parts(45_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Validators (r:1 w:1), SlashingSpans (r:1 w:1),
    ///          Currency::slash (r:1 w:1)
    fn report_validator_offense() -> Weight {
        Weight::from_parts(50_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
