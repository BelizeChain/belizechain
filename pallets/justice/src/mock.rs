//! Mock runtime for Justice pallet tests
#![cfg(test)]

use crate as pallet_belize_justice;
use frame_support::{construct_runtime, parameter_types, traits::{ConstU16, ConstU32, Everything, Hooks}};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        Justice: pallet_belize_justice,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const ExistentialDeposit: u128 = 1;
    pub const OpenDisputeBond: u128 = 500;
    pub const CoolingOffPeriod: u64 = 100;
    pub const MaxMediators: u32 = 10;
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
    type AccountData = pallet_balances::AccountData<u128>;
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

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type DoneSlashHandler = ();
}

impl pallet_belize_justice::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
    type MediatorOrigin = frame_system::EnsureSigned<u64>;
    type OpenDisputeBond = OpenDisputeBond;
    type CoolingOffPeriod = CoolingOffPeriod;
    type MaxMediators = MaxMediators;
    type WeightInfo = ();
}

// Test accounts
pub const ALICE: u64 = 1;    // Disputant
pub const BOB: u64 = 2;      // Target / accused
pub const MEDIATOR: u64 = 3; // Mediator account
pub const DAVE: u64 = 4;     // Additional account
pub const GOVERNANCE: u64 = 99;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000),
            (BOB, 1_000_000),
            (MEDIATOR, 1_000_000),
            (DAVE, 1_000_000),
            (GOVERNANCE, 1_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Advance blocks
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
