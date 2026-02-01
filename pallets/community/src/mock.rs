use crate::{self as pallet_belize_community};
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use pallet_belize_identity::{BelizeKyc, KycLevel};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Community: pallet_belize_community,
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
    type AccountData = pallet_balances::AccountData<u64>;
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
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type DoneSlashHandler = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
}

// Mock KYC implementation for testing
pub struct MockKyc;
impl BelizeKyc<u64, u64> for MockKyc {
    fn is_kyc_verified(account: &u64, _level: KycLevel, _now: u64) -> bool {
        // Accounts 1-10 are verified for testing purposes
        matches!(account, 1..=10)
    }
}

// Community pallet configuration
parameter_types! {
    pub const ProposalDepositPercentage: u32 = 10;
    pub const FeeExemptionMonthlyLimit: u32 = 100_000; // 100K dBZD for testing
    pub const EducationRewardAmount: u64 = 100;
    pub const ReferralRewardAmount: u64 = 100;
    pub const CommunityVotingPeriod: u64 = 7 * 24 * 60 * 10; // 7 days in blocks (~10s blocks)
    pub const CommunityTreasuryAccount: u64 = 999; // Mock treasury account
    pub const MaxTitleLength: u32 = 128;
    pub const MaxDescriptionLength: u32 = 1024;
    pub const MaxParticipationHistory: u32 = 1000;
}

impl pallet_belize_community::Config for Test {
    type Currency = Balances;
    type BelizeKyc = MockKyc;
    type ProposalDepositPercentage = ProposalDepositPercentage;
    type FeeExemptionMonthlyLimit = FeeExemptionMonthlyLimit;
    type EducationRewardAmount = EducationRewardAmount;
    type ReferralRewardAmount = ReferralRewardAmount;
    type CommunityVotingPeriod = CommunityVotingPeriod;
    type CommunityTreasuryAccount = CommunityTreasuryAccount;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
    type MaxTitleLength = MaxTitleLength;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxParticipationHistory = MaxParticipationHistory;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),
            (2, 10000),
            (3, 10000),
            (999, 1000000), // Treasury
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Helper function to advance blocks
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
