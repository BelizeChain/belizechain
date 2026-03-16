#![allow(clippy::double_parens)]
#![allow(unused_parens)]
#![allow(dead_code)]
use crate as pallet_belize_interoperability;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, Everything},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Interoperability: pallet_belize_interoperability,
        Timestamp: pallet_timestamp,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
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

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
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
    type MinimumPeriod = ConstU64<3>;
    type WeightInfo = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
    pub const MaxBridgeValidators: u32 = 100;
    pub const MinBridgeAmount: u128 = 10_000; // 10K minimum
    pub const BridgeFeeRate: u32 = 50; // 0.5% fee (50 basis points)
    pub const PQSignatureThreshold: u32 = 5; // 5 signatures required
    pub const TreasuryAccount: u64 = 999; // Treasury account
    pub const InteropPalletId: frame_support::PalletId = frame_support::PalletId(*b"bz/intop");
}

// Mock Identity provider for cross-chain KYC verification
pub struct MockIdentity;
impl pallet_belize_interoperability::InteroperabilityIdentityProvider<u64> for MockIdentity {
    fn get_kyc_level(account: &u64) -> Option<u8> {
        match account {
            // ALICE, BOB, CHARLIE: Level 3 (Full KYC - bridge operators)
            1..=3 => Some(3),
            // Test accounts 10-50: Level 2 (Enhanced KYC - can use bridges)
            10..=50 => Some(2),
            // Account 100: Level 1 (Basic KYC - insufficient for bridges)
            100 => Some(1),
            // Account 666: Sanctioned but has Level 2 (to test sanction check)
            666 => Some(2),
            // Others: No KYC
            _ => None,
        }
    }

    fn verify_bridge_operator(account: &u64) -> bool {
        // Only accounts 1-3 (ALICE, BOB, CHARLIE) are verified bridge operators (Level 3)
        matches!(account, 1..=3)
    }

    fn is_sanctioned(account: &u64) -> bool {
        // Account 666 is sanctioned
        *account == 666
    }
}

impl pallet_belize_interoperability::Config for Test {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type Time = Timestamp;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
    type Treasury = TreasuryAccount;
    type MaxBridgeValidators = MaxBridgeValidators;
    type MinBridgeAmount = MinBridgeAmount;
    type BridgeFeeRate = BridgeFeeRate;
    type PQSignatureThreshold = PQSignatureThreshold;
    type WeightInfo = ();
    type Identity = MockIdentity;
    type ChallengePeriod = ConstU64<100>;
    type MaxBridgePerBlock = ConstU32<5>;
    type PQVerifier = pallet_belize_interoperability::PassthroughPQVerifier;
    type PalletId = InteropPalletId;
}

// Test account constants
pub const ALICE: u64 = 1; // Bridge operator with Level 3 KYC
pub const BOB: u64 = 2; // Bridge operator with Level 3 KYC
pub const CHARLIE: u64 = 3; // Bridge operator with Level 3 KYC
pub const EVE: u64 = 10; // Regular user with Level 2 KYC
pub const FERDIE: u64 = 11; // Regular user with Level 2 KYC
pub const NO_KYC: u64 = 200; // User without KYC
pub const LOW_KYC: u64 = 100; // User with Level 1 KYC (insufficient for bridges)
pub const SANCTIONED: u64 = 666; // Sanctioned account
pub const TREASURY: u64 = 999; // Treasury account

// Helper function to create test external address
pub fn test_eth_address() -> Vec<u8> {
    vec![0xAB; 20] // Mock Ethereum address
}

// Helper function to create test transaction hash
pub fn test_tx_hash() -> Vec<u8> {
    vec![0xCD; 32] // Mock transaction hash
}

// Helper function to create test PQ signature
pub fn test_pq_signature() -> Vec<u8> {
    vec![0xEF; 64] // Mock post-quantum signature (minimum 64 bytes)
}

// Helper function to create test message payload
pub fn test_message_payload() -> Vec<u8> {
    b"test cross-chain message".to_vec()
}

// Build test externalities
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000_000),
            (BOB, 1_000_000_000),
            (CHARLIE, 1_000_000_000),
            (EVE, 500_000_000),
            (FERDIE, 500_000_000),
            (NO_KYC, 500_000_000),
            (LOW_KYC, 500_000_000),
            (SANCTIONED, 500_000_000),
            (TREASURY, 1_000), // Must be above existential deposit
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

// Helper to run to a specific block
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
        Timestamp::set_timestamp(((System::block_number() * 6000)));
    }
}
