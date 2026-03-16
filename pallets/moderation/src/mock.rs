//! Mock runtime for Moderation pallet tests
#![cfg(test)]

use crate as pallet_belize_moderation;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU16, ConstU32, Everything},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub struct Test {
        System: frame_system,
        Moderation: pallet_belize_moderation,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const FlagThreshold: u32 = 3;
    pub const NawalAutoQueueScore: u8 = 50;
    pub const MaxModerators: u32 = 10;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeTask = ();
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

impl pallet_belize_moderation::Config for Test {
    type ModeratorAdminOrigin = frame_system::EnsureRoot<u64>;
    type NawalOracleOrigin = frame_system::EnsureRoot<u64>;
    type FlagThreshold = FlagThreshold;
    type NawalAutoQueueScore = NawalAutoQueueScore;
    type MaxModerators = MaxModerators;
    type MaxFlagsPerContent = frame_support::traits::ConstU32<100>;
    type WeightInfo = ();
}

// Provide a trivial WeightInfo impl for tests
impl crate::weights::WeightInfo for () {
    fn flag_content() -> frame_support::weights::Weight { frame_support::weights::Weight::zero() }
    fn review_content() -> frame_support::weights::Weight { frame_support::weights::Weight::zero() }
    fn add_moderator() -> frame_support::weights::Weight { frame_support::weights::Weight::zero() }
    fn remove_moderator() -> frame_support::weights::Weight { frame_support::weights::Weight::zero() }
    fn submit_nawal_assessment() -> frame_support::weights::Weight { frame_support::weights::Weight::zero() }
}

// Test accounts
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const MODERATOR_1: u64 = 10;
pub const MODERATOR_2: u64 = 11;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Helper: content hash fixtures
pub fn content_a() -> [u8; 32] { [0xAA; 32] }
pub fn content_b() -> [u8; 32] { [0xBB; 32] }
