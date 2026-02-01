//! Mock runtime for oracle pallet testing

use crate as pallet_belize_oracle;
use frame_support::{
    parameter_types,
    traits::ConstU32,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Oracle: pallet_belize_oracle,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
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
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

parameter_types! {
    pub const MaxOperators: u32 = 10;
    pub const MaxDataStaleness: u64 = 100; // 100 blocks
    pub const MinConsensusOperators: u32 = 1; // Minimum 1 operator for testing (production should be 2+)
}

impl pallet_belize_oracle::Config for Test {
    type OracleAdminOrigin = frame_system::EnsureRoot<u64>;
    type MaxOperators = MaxOperators;
    type MaxDataStaleness = MaxDataStaleness;
    type MinConsensusOperators = MinConsensusOperators;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;
}

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_belize_oracle::GenesisConfig::<Test> {
        operators: vec![1, 2], // Alice and Bob as initial operators
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
