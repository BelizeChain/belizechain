//! Mock runtime for BelizeChain commit-reveal randomness pallet tests.

use crate as pallet_belize_randomness;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        BelizeRandomness: pallet_belize_randomness,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

parameter_types! {
    // Commit window: 5 blocks from epoch start.
    pub const CommitDuration: u64 = 5;
    // Reveal window: 5 blocks after commit window closes.
    pub const RevealDuration: u64 = 5;
}

impl pallet_belize_randomness::Config for Test {
    type CommitDuration = CommitDuration;
    type RevealDuration = RevealDuration;
    /// At least 2 reveals required for a valid epoch output.
    type MinReveals = ConstU32<2>;
    /// Maximum 5 contributors per epoch.
    type MaxContributors = ConstU32<5>;
}

/// Build genesis + advance to block 1 so `on_initialize` fires once,
/// placing `EpochStart = 1`.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        // Simulate on_initialize at block 1 to initialise EpochStart.
        use frame_support::traits::OnInitialize;
        System::set_block_number(1);
        BelizeRandomness::on_initialize(1u64);
    });
    ext
}

/// Advance the mock chain to block `n`, firing hooks each step.
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        use frame_support::traits::OnInitialize;
        BelizeRandomness::on_initialize(System::block_number());
    }
}
