//! Weight information for pallet-belize-oracle

use frame_support::weights::constants::RocksDbWeight;
use frame_support::weights::Weight;

/// Weight functions needed for the Oracle pallet
pub trait WeightInfo {
    fn add_operator() -> Weight;
    fn remove_operator() -> Weight;
    fn submit_price() -> Weight;
    fn verify_merchant() -> Weight;
    fn add_sanction() -> Weight;
    fn remove_sanction() -> Weight;
    fn verify_identity() -> Weight;
    fn register_land() -> Weight;
    fn update_exchange_rate() -> Weight;
    fn register_iot_device() -> Weight;
    fn submit_iot_data() -> Weight;
    fn verify_iot_device() -> Weight;
    fn claim_oracle_rewards() -> Weight;
    fn resolve_kyc_dispute() -> Weight;
    fn submit_behavior_flag() -> Weight;
    fn clear_behavior_flag() -> Weight;
}

/// Default weight implementation for SubstrateWeight
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_operator() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn remove_operator() -> Weight {
        Weight::from_parts(20_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn submit_price() -> Weight {
        Weight::from_parts(35_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(3)) // Check operator, read submissions, read existing feed
            .saturating_add(RocksDbWeight::get().writes(2)) // Write submission, write aggregated feed
    }

    fn verify_merchant() -> Weight {
        Weight::from_parts(30_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(1)) // Check operator
            .saturating_add(RocksDbWeight::get().writes(1)) // Write merchant info
    }

    fn add_sanction() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(1)) // Check operator
            .saturating_add(RocksDbWeight::get().writes(1)) // Write sanction
    }

    fn remove_sanction() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2)) // Check operator, check sanction exists
            .saturating_add(RocksDbWeight::get().writes(1)) // Remove sanction
    }

    fn verify_identity() -> Weight {
        Weight::from_parts(40_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(1)) // Check operator
            .saturating_add(RocksDbWeight::get().writes(1)) // Write identity info
    }

    fn register_land() -> Weight {
        Weight::from_parts(35_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1)) // Check operator
            .saturating_add(RocksDbWeight::get().writes(1)) // Write land registry
    }

    fn update_exchange_rate() -> Weight {
        Weight::from_parts(20_000_000, 512).saturating_add(RocksDbWeight::get().writes(1))
        // Write manual exchange rate
    }

    fn register_iot_device() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(1)) // Check not exists
            .saturating_add(RocksDbWeight::get().writes(1)) // Insert device
    }

    fn submit_iot_data() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(2)) // Read device, read operator stats
            .saturating_add(RocksDbWeight::get().writes(2)) // Update device, update operator stats
    }

    fn verify_iot_device() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2)) // Check operator, read device
            .saturating_add(RocksDbWeight::get().writes(1)) // Update device verified flag
    }

    fn claim_oracle_rewards() -> Weight {
        Weight::from_parts(25_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1)) // Read operator stats
            .saturating_add(RocksDbWeight::get().writes(1)) // Remove stats after claim
    }

    fn resolve_kyc_dispute() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(1)) // Read dispute flag
            .saturating_add(RocksDbWeight::get().writes(3)) // Clear flag, leading vote, pending prefix
    }

    fn submit_behavior_flag() -> Weight {
        Weight::from_parts(15_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(3)) // operators, pending votes, existing flag
            .saturating_add(RocksDbWeight::get().writes(2)) // pending flag vote, flag/cooldown on consensus
    }

    fn clear_behavior_flag() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1)) // existing flag
            .saturating_add(RocksDbWeight::get().writes(2)) // flag, cooldown
    }
}
