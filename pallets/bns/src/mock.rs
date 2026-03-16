//! BNS Pallet Mock Runtime for Testing

use super::*;
use crate as pallet_belize_bns;

use frame_support::{
    parameter_types,
    traits::{ConstU32, Everything},
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Bns: pallet_belize_bns,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
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
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
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
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const TreasuryAccount: u64 = 999;
    pub const MaxDomainsPerAccount: u32 = 100;
    pub const MaxDomainLength: u32 = 64;
    pub const MaxTextRecords: u32 = 20;
    pub const MinDomainLength: u32 = 3;
}

// Mock Identity Provider
pub struct MockIdentityProvider;
impl BnsIdentityProvider<u64> for MockIdentityProvider {
    fn can_register_domain(_account: &u64) -> bool {
        // Allow all accounts for testing
        true
    }

    fn can_register_verified(_account: &u64) -> bool {
        // Require account ID >= 100 for verified domains
        _account >= &100
    }

    fn is_sanctioned(_account: &u64) -> bool {
        // Mark account 666 as sanctioned for testing
        _account == &666
    }
}

impl pallet_belize_bns::Config for Test {
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type Treasury = TreasuryAccount;
    type MaxDomainsPerAccount = MaxDomainsPerAccount;
    type MaxDomainLength = MaxDomainLength;
    type MaxTextRecords = MaxTextRecords;
    type MinDomainLength = MinDomainLength;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;
    type Identity = MockIdentityProvider;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
    type MaxContentVersions = frame_support::traits::ConstU32<50>;
}

// Build genesis storage
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10_000_000_000_000_000),  // 10000 DALLA
            (2, 10_000_000_000_000_000),  // 10000 DALLA
            (3, 5_000_000_000_000_000),   // 5000 DALLA
            (100, 20_000_000_000_000_000), // 20000 DALLA (verified account)
            (666, 10_000_000_000_000_000), // 10000 DALLA (sanctioned)
            (999, 10_000_000_000_000),     // Treasury with 10 DALLA initial balance
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
