use crate as pallet_belize_consensus;
use frame_support::{
    parameter_types,
    traits::ConstU32,
    PalletId,
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
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Consensus: pallet_belize_consensus,
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
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
    pub const MaxValidators: u32 = 100;
    pub const MinConsensusStake: u128 = 1_000_000;
    pub const MinModelQualityScore: u32 = 50; // 50% (not basis points, just 0-100 scale)
    pub const ConsensusReward: u128 = 100_000;
    pub const ConsensusPalletId: PalletId = PalletId(*b"bz/consn");
}

// Mock staking provider for consensus tests
pub struct MockStakingProvider;
impl pallet_belize_consensus::ConsensusStakingProvider<u64, u128> for MockStakingProvider {
    fn get_validator_reputation(account: &u64) -> u8 {
        match *account {
            1..=3 => 90, // High reputation validators
            10 | 11 => 70,   // Medium reputation
            100 => 40,       // Low reputation
            _ => 50,         // Default
        }
    }

    fn get_model_quality_score(account: &u64) -> u8 {
        match *account {
            1..=3 => 85, // High quality models
            10 | 11 => 65,   // Medium quality
            100 => 35,       // Low quality
            _ => 50,         // Default
        }
    }

    fn get_validator_stake(account: &u64) -> u128 {
        match *account {
            1..=3 => 10_000_000,
            10 | 11 => 5_000_000,
            100 => 1_000_000,
            _ => 0,
        }
    }

    fn update_reputation(account: &u64, _quality_score: u8) -> bool {
        // In mock, all updates succeed for accounts 1-100
        *account <= 100
    }
}

impl pallet_belize_consensus::Config for Test {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type UnixTime = Timestamp;
    type AIAuthorityOrigin = frame_system::EnsureRoot<u64>;
    type MaxValidators = MaxValidators;
    type MinConsensusStake = MinConsensusStake;
    type MinModelQualityScore = MinModelQualityScore;
    type ConsensusReward = ConsensusReward;
    type WeightInfo = ();
    type Staking = MockStakingProvider;
    type MaxSubmitPerBlock = ConstU32<5>;
}

// Test account constants
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const EVE: u64 = 10;
pub const FERDIE: u64 = 11;
pub const LOW_QUALITY: u64 = 100;
pub const NO_FUNDS: u64 = 200;

// Helper functions for test data
pub fn test_parameters_hash(n: u8) -> [u8; 32] {
    let mut hash = [0u8; 32];
    hash[0] = n;
    hash
}

pub fn test_pq_signature(length: usize) -> Vec<u8> {
    vec![0x42; length]
}

// Build test externalities
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 100_000_000),
            (BOB, 100_000_000),
            (CHARLIE, 100_000_000),
            (EVE, 50_000_000),
            (FERDIE, 50_000_000),
            (LOW_QUALITY, 10_000_000),
            (NO_FUNDS, 100_000), // Less than MinConsensusStake
        ],
        ..Default::default()
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

#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        Timestamp::set_timestamp(Timestamp::get() + 1000);
    }
}
