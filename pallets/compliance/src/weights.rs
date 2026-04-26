//! Hand-estimated weights for pallet-belize-compliance
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_compliance using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: ComplianceStatus (r:1 w:1), VerificationRecords (r:1 w:1),
    ///          Identity check (r:1 w:0)
    fn verify_account() -> Weight {
        Weight::from_parts(50_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: ComplianceStatus (r:1 w:1), RiskScores (r:1 w:1)
    fn update_risk_level() -> Weight {
        Weight::from_parts(35_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: WhitelistedAccounts (r:1 w:1)
    fn whitelist_account() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ComplianceStatus (r:1 w:1), RestrictedAccounts (r:1 w:1)
    fn restrict_account() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: ComplianceStatus (r:1 w:0), SuspiciousFlags (r:1 w:1)
    fn flag_suspicious_activity() -> Weight {
        Weight::from_parts(60_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: SanctionsList (r:1 w:1)
    fn add_sanctions_entry() -> Weight {
        Weight::from_parts(45_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: SanctionsList (r:1 w:1)
    fn remove_sanctions_entry() -> Weight {
        Weight::from_parts(35_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ComplianceStatus (r:1 w:0), SanctionsList (r:1 w:0),
    ///          WhitelistedAccounts (r:1 w:0)
    fn check_compliance() -> Weight {
        Weight::from_parts(25_000_000, 2560).saturating_add(T::DbWeight::get().reads(3))
    }

    /// Storage: ComplianceStatus (r:1 w:1), VerificationRecords (r:1 w:0)
    fn update_verification_level() -> Weight {
        Weight::from_parts(40_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
