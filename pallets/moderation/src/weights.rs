use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions needed for pallet_belize_moderation.
pub trait WeightInfo {
    fn flag_content() -> Weight;
    fn review_content() -> Weight;
    fn add_moderator() -> Weight;
    fn remove_moderator() -> Weight;
    fn submit_nawal_assessment() -> Weight;
}

/// Default conservative weight estimates for pallet_belize_moderation.
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn flag_content() -> Weight {
        Weight::from_parts(30_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn review_content() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn add_moderator() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn remove_moderator() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn submit_nawal_assessment() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
