use crate as pallet_quantum;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, Hooks},
    PalletId,
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Quantum: pallet_quantum,
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
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
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
    type MinimumPeriod = ConstU64<3>;
    type WeightInfo = ();
}

parameter_types! {
    pub const QuantumPalletId: PalletId = PalletId(*b"bzquantm");
    pub const MaxActiveJobs: u32 = 100;
    pub const DallaPerQubit: u128 = 1_000_000; // 0.000001 DALLA per qubit
    pub const DallaPerShot: u128 = 100_000; // 0.0000001 DALLA per shot
    pub const NFTMintingFee: u128 = 500_000_000_000; // 0.5 DALLA
    pub const QuantumTreasuryAccount: u64 = 100; // Mock treasury account
}

impl pallet_quantum::Config for Test {
    type Currency = Balances;
    type MaxActiveJobs = MaxActiveJobs;
    type DallaPerQubit = DallaPerQubit;
    type DallaPerShot = DallaPerShot;
    type NFTMintingFee = NFTMintingFee;
    type Treasury = QuantumTreasuryAccount;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000), // Alice - 1M DALLA
            (2, 1_000_000_000_000_000), // Bob - 1M DALLA
            (3, 1_000_000_000_000_000), // Charlie - 1M DALLA
            (4, 500_000_000_000_000),   // Dave - 500K DALLA
            (5, 500_000_000_000_000),   // Eve - 500K DALLA
            (100, 10_000_000_000_000_000), // Treasury - 10M DALLA
        ],
        dev_accounts: None,
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

// Helper functions for tests
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            <Quantum as Hooks<u64>>::on_finalize(System::block_number());
            <System as Hooks<u64>>::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        <System as Hooks<u64>>::on_initialize(System::block_number());
        <Quantum as Hooks<u64>>::on_initialize(System::block_number());
        Timestamp::set_timestamp(System::block_number() * 6000);
    }
}

pub fn last_event() -> RuntimeEvent {
    System::events().pop().expect("Event expected").event
}

pub fn expect_event<E: Into<RuntimeEvent>>(e: E) {
    assert_eq!(last_event(), e.into());
}
