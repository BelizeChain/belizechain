/// Common weights for BelizeChain pallets
/// This provides simple weight implementations for all pallets

use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions needed for all BelizeChain pallets.
pub trait WeightInfo {
    fn create_transaction() -> Weight { Weight::from_parts(50_000_000, 512) }
    fn update_config() -> Weight { Weight::from_parts(25_000_000, 512) }
    fn emergency_action() -> Weight { Weight::from_parts(100_000_000, 512) }
    fn simple_action() -> Weight { Weight::from_parts(10_000_000, 512) }
    fn complex_action() -> Weight { Weight::from_parts(75_000_000, 1024) }
}

/// An implementation of `WeightInfo` with default weights.
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_transaction() -> Weight {
        Weight::from_parts(50_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    
    fn update_config() -> Weight {
        Weight::from_parts(25_000_000, 3072)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    
    fn emergency_action() -> Weight {
        Weight::from_parts(100_000_000, 3072)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    
    fn simple_action() -> Weight {
        Weight::from_parts(10_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(1))
    }
    
    fn complex_action() -> Weight {
        Weight::from_parts(75_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}