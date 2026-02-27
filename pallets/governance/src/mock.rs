use crate::{self as pallet_belize_governance, *};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything, Randomness},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

// Simple test randomness implementation
pub struct TestRandomness;
impl Randomness<H256, u64> for TestRandomness {
    fn random(_subject: &[u8]) -> (H256, u64) {
        (H256::zero(), 0)
    }
}

// Mock compliance provider - allows all accounts except 999 (restricted test account)
pub struct MockCompliance;
impl ComplianceCheck<u64> for MockCompliance {
    fn can_participate_in_governance(account: &u64) -> bool {
        // Account 999 is restricted for testing
        *account != 999
    }
}

// Mock community participation provider (Phase 6 integration)
pub struct MockCommunityParticipation;
impl pallet_belize_community::GovernanceParticipation<u64> for MockCommunityParticipation {
    fn record_proposal_submission(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }
    
    fn record_vote_cast(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }
    
    fn record_proposal_approval(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }
    
    fn record_council_activity(_account: &u64) -> Result<(), &'static str> {
        Ok(()) // No-op in tests
    }
}

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        BelizeGovernance: pallet_belize_governance,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
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
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const GovernancePalletId: PalletId = PalletId(*b"bz/govnc");
    pub const MinimumDeposit: u128 = 1_000_000_000; // 1000 DALLA
    pub const VotingPeriod: u64 = 100_800; // 7 days in blocks
    pub const LaunchPeriod: u64 = 28_800; // 2 days in blocks
}

impl pallet_belize_governance::Config for Test {
    type Currency = Balances;
    type Randomness = TestRandomness;
    type CouncilOrigin = EnsureRoot<u64>;
    type CommunityOrigin = EnsureRoot<u64>;
    type ComplianceProvider = MockCompliance;
    type CommunityParticipation = MockCommunityParticipation;
    type MinimumDeposit = MinimumDeposit;
    type VotingPeriod = VotingPeriod;
    type LaunchPeriod = LaunchPeriod;
    type WeightInfo = ();
    type MaxCandidatesPerElection = ConstU32<50>;
}

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000_000_000), // Alice: 100k DALLA
            (2, 100_000_000_000), // Bob: 100k DALLA
            (3, 100_000_000_000), // Charlie: 100k DALLA
            (4, 50_000_000_000),  // Dave: 50k DALLA
            (5, 50_000_000_000),  // Eve: 50k DALLA
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Initialize governance with council members
    pallet_belize_governance::GenesisConfig::<Test> {
        council_members: vec![
            (1, 100, 200), // Alice: community_rank=100, pouw=200, weight=300
            (2, 80, 150),  // Bob: community_rank=80, pouw=150, weight=230
            (3, 60, 100),  // Charlie: community_rank=60, pouw=100, weight=160
        ],
        democracy_launch_period: 28_800,
        democracy_voting_period: 100_800,
        democracy_minimum_deposit: 1_000_000_000,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

// Test helper functions
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

pub fn last_event() -> RuntimeEvent {
    System::events().pop().expect("Event expected").event
}

