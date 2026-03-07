//! Hand-estimated weights for pallet-belize-identity
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_identity using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: NextIdentityId (r:1 w:1), Identities (r:0 w:1), IdentityOf (r:0 w:1)
    fn register_identity() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: IdentityOf (r:1 w:0), LinkedAccounts (r:1 w:1)
    fn link_account() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: IdentityOf (r:1 w:1)
    fn update_did() -> Weight {
        Weight::from_parts(15_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: IdentityOf (r:1 w:1)
    fn admin_simple() -> Weight {
        Weight::from_parts(10_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: IdentityOf (r:1 w:0), Attestations (r:1 w:1), AttesterInfo (r:1 w:0)
    fn issue_attestation() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: Attestations (r:1 w:1)
    fn revoke() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
