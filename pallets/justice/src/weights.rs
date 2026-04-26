//! Weight functions for the justice pallet

use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for pallet_belize_justice.
pub trait WeightInfo {
    fn open_dispute() -> Weight;
    fn mediator_ruling() -> Weight;
    fn appeal_ruling() -> Weight;
    fn complete_rehabilitation() -> Weight;
    fn add_mediator() -> Weight;
    fn remove_mediator() -> Weight;
}

/// Conservative weight estimates — replace with benchmarks before mainnet.
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn open_dispute() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn mediator_ruling() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn appeal_ruling() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn complete_rehabilitation() -> Weight {
        Weight::from_parts(35_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn add_mediator() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn remove_mediator() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}

impl WeightInfo for () {
    fn open_dispute() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
    fn mediator_ruling() -> Weight {
        Weight::from_parts(30_000_000, 512)
    }
    fn appeal_ruling() -> Weight {
        Weight::from_parts(25_000_000, 512)
    }
    fn complete_rehabilitation() -> Weight {
        Weight::from_parts(35_000_000, 512)
    }
    fn add_mediator() -> Weight {
        Weight::from_parts(20_000_000, 512)
    }
    fn remove_mediator() -> Weight {
        Weight::from_parts(20_000_000, 512)
    }
}
