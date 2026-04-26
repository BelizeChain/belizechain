use crate as pallet_belize_compliance;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Compliance: pallet_belize_compliance,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeTask = ();
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
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
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

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

use crate::VerificationLevel;

parameter_types! {
    pub const MinValidatorVerification: VerificationLevel = VerificationLevel::Enhanced;
    pub const MinGovernanceVerification: VerificationLevel = VerificationLevel::Standard;
    pub const MinTreasuryVerification: VerificationLevel = VerificationLevel::Enhanced;
    pub const TravelRuleThreshold: u128 = 10_000_000_000; // 10K DALLA
    pub const VerificationValidityPeriod: u64 = 31_536_000; // 1 year in seconds
    pub const MaxAuditRecords: u32 = 100;
    pub const MaxSuspiciousActivityReports: u32 = 50;
    pub const StructuringWindowBlocks: u64 = 14_400; // ~1 day at 6s blocks
    pub const MaxTransactionWindowEntries: u32 = 50;
}

impl pallet_belize_compliance::Config for Test {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type ComplianceOrigin = frame_system::EnsureRoot<u64>;
    type SanctionsOrigin = frame_system::EnsureRoot<u64>;
    type MinValidatorVerification = MinValidatorVerification;
    type MinGovernanceVerification = MinGovernanceVerification;
    type MinTreasuryVerification = MinTreasuryVerification;
    type TravelRuleThreshold = TravelRuleThreshold;
    type VerificationValidityPeriod = VerificationValidityPeriod;
    type MaxAuditRecords = MaxAuditRecords;
    type MaxSuspiciousActivityReports = MaxSuspiciousActivityReports;
    type StructuringWindowBlocks = StructuringWindowBlocks;
    type MaxTransactionWindowEntries = MaxTransactionWindowEntries;
    type WeightInfo = ();
    type AccountSanctionsChecker = (); // No-op in tests
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100_000), (2, 100_000), (3, 100_000)],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1_000_000);
    });
    ext
}

// Re-export for test convenience
// Note: Compliance is already defined by construct_runtime! macro
// RuntimeOrigin, RuntimeEvent, System are automatically available from construct_runtime!
