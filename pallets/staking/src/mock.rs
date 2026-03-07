use crate::{self as pallet_belize_staking, *};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything, Randomness},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

// Mock randomness for validator selection
pub struct TestRandomness;
impl Randomness<H256, u64> for TestRandomness {
    fn random(_subject: &[u8]) -> (H256, u64) {
        (H256::from([1u8; 32]), 0)
    }
}

// Mock Identity provider for KYC checks
pub struct MockIdentity;
impl StakingIdentityProvider<u64> for MockIdentity {
    fn get_kyc_level(account: &u64) -> Option<u8> {
        // Validators must have L2 KYC (SSN + Passport)
        if (*account >= 10 && *account < 200) || *account == 666 {
            Some(2) // Validators range and SANCTIONED have L2
        } else if *account == 999 {
            None // Restricted account
        } else {
            Some(1) // Regular accounts have L1
        }
    }
    
    fn meets_validator_kyc(account: &u64) -> bool {
        Self::get_kyc_level(account).unwrap_or(0) >= 2
    }
    
    fn is_sanctioned(account: &u64) -> bool {
        *account == 666 // Test sanctioned account
    }
}

// Mock OracleVerifier for oracle verification in tests
impl OracleVerifier<u64> for MockIdentity {
    fn is_authorized_operator(_who: &u64) -> bool {
        true // All accounts are authorized operators in tests
    }
}

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        BelizeStaking: pallet_belize_staking,
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
    pub const MaxValidators: u32 = 100;
    pub const MinValidatorStake: u128 = 10_000_000_000; // 10K DALLA
    pub const BaseReward: u128 = 100_000_000; // 100 DALLA per epoch
    pub const EpochDuration: u64 = 100; // 100 blocks per epoch
    pub const UnbondingPeriod: u64 = 100; // 100 blocks unbonding
    pub const TestMaxSupply: u128 = 501_000_000_000_000_000; // 501B DALLA
}

// Mock justice provider — always report pending review so slashes
// bypass the escrow path and apply directly (testing the direct slash flow).
pub struct MockJustice;
impl pallet_belize_staking::JusticeProvider<u64, u128> for MockJustice {
    fn try_escrow_slash(_account: &u64, _amount: u128) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn has_pending_review(_account: &u64) -> bool {
        true
    }
}

impl pallet_belize_staking::Config for Test {
    type Currency = Balances;
    type OracleVerifier = MockIdentity; // Reuse MockIdentity for testing
    type Randomness = TestRandomness;
    type Identity = MockIdentity;
    type MaxValidators = MaxValidators;
    type MinValidatorStake = MinValidatorStake;
    type BaseReward = BaseReward;
    type EpochDuration = EpochDuration;
    type UnbondingPeriod = UnbondingPeriod;
    type MaxSupply = TestMaxSupply;
    type WeightInfo = ();
    type JusticeProvider = MockJustice;
}

// Test accounts
pub const ALICE: u64 = 10; // Validator with L2 KYC
pub const BOB: u64 = 11; // Validator with L2 KYC
pub const CHARLIE: u64 = 12; // Validator with L2 KYC
pub const DAVE: u64 = 13; // Validator with L2 KYC
pub const EVE: u64 = 1; // Regular account L1
pub const SANCTIONED: u64 = 666; // Sanctioned account
pub const NO_KYC: u64 = 999; // No KYC account

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000_000_000), // 1M DALLA
            (BOB, 1_000_000_000_000),
            (CHARLIE, 1_000_000_000_000),
            (DAVE, 1_000_000_000_000),
            (EVE, 100_000_000_000), // 100K DALLA
            (SANCTIONED, 100_000_000_000),
            (NO_KYC, 100_000_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Helper to advance blocks
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

// Helper to create bounded location
pub fn test_location(s: &str) -> BoundedVec<u8, ConstU32<64>> {
    BoundedVec::try_from(s.as_bytes().to_vec()).unwrap()
}

// Helper to fund staking rewards (deposit directly to validators)
#[allow(dead_code)]
pub fn fund_staking_rewards(validator: u64, amount: u128) {
    Balances::make_free_balance_be(&validator, Balances::free_balance(validator) + amount);
}
