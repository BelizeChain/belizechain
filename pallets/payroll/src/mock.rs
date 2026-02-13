use crate as pallet_payroll;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

pub type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Payroll: pallet_payroll,
    }
);

parameter_types! {
    pub const PayrollPalletId: PalletId = PalletId(*b"bz/payrl");
    pub const MaxEmployees: u32 = 1000;
    pub const MaxDeductions: u32 = 10;
    pub const MaxDepartments: u32 = 50;
    pub const MinimumPayment: u64 = 1_000_000; // 1 DALLA (10^6 with 6 decimals)
}

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
    type AccountData = pallet_balances::AccountData<u64>;
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

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

// Mock Oracle provider for testing
pub struct MockPayrollOracleProvider;
impl pallet_payroll::PayrollOracleProvider<u64> for MockPayrollOracleProvider {
    fn get_kyc_level(_account: &u64) -> Option<u8> {
        Some(1)
    }
    
    fn meets_kyc_requirement(_account: &u64, required_level: u8) -> bool {
        required_level <= 1
    }
}

impl pallet_payroll::Config for Test {
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type PalletId = PayrollPalletId;
    type MaxEmployees = MaxEmployees;
    type MaxDeductions = MaxDeductions;
    type MaxDepartments = MaxDepartments;
    type MinimumPayment = MinimumPayment;
    type VerifierOrigin = frame_system::EnsureRoot<u64>;
    type Oracle = MockPayrollOracleProvider;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000),  // Employer 1 (1 million DALLA with 6 decimals)
            (2, 1_000_000_000_000_000),  // Employer 2
            (3, 10_000_000_000),         // Employee 1 (10K DALLA)
            (4, 10_000_000_000),         // Employee 2
            (5, 10_000_000_000),         // Employee 3
            (6, 10_000_000_000),         // Employee 4
            (100, 500_000_000_000_000),  // Admin account
        ],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1000);
    });
    ext
}
