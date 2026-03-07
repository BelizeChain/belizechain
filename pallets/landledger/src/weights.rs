//! Hand-estimated weights for pallet-belize-landledger
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_landledger using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: Properties (r:1 w:1), NextPropertyId (r:1 w:1),
    ///          Oracle verification (r:1 w:0), Currency::transfer (r:2 w:2)
    /// Event: 1 write
    fn register_property() -> Weight {
        Weight::from_parts(55_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: Properties (r:1 w:1), TransferRecords (r:0 w:1),
    ///          Currency::transfer for tax (r:2 w:2),
    ///          Oracle price check (r:1 w:0)
    /// Event: 1 write
    fn transfer_property() -> Weight {
        Weight::from_parts(70_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: Properties (r:1 w:1)
    /// Event: 1 write
    fn verify_property() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: Properties (r:1 w:1), RegisteredSurveyors (r:1 w:0)
    /// Event: 1 write
    fn survey_property() -> Weight {
        Weight::from_parts(35_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: RegisteredSurveyors (r:1 w:1)
    /// Event: 1 write
    fn register_surveyor() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
