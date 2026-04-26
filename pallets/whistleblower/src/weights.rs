//! Weight functions for the whistleblower pallet

use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for pallet_belize_whistleblower.
pub trait WeightInfo {
    fn submit_report() -> Weight;
    fn review_report() -> Weight;
    fn claim_reward() -> Weight;
    fn fund_whistleblower_pool() -> Weight;
}

pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_report() -> Weight {
        Weight::from_parts(35_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn review_report() -> Weight {
        Weight::from_parts(30_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn claim_reward() -> Weight {
        Weight::from_parts(40_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn fund_whistleblower_pool() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}

impl WeightInfo for () {
    fn submit_report() -> Weight {
        Weight::from_parts(35_000_000, 512)
    }
    fn review_report() -> Weight {
        Weight::from_parts(30_000_000, 512)
    }
    fn claim_reward() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
    fn fund_whistleblower_pool() -> Weight {
        Weight::from_parts(20_000_000, 512)
    }
}
