use crate::{self as pallet_belize_identity, *};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything},
    PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

// Mock Oracle provider for testing
pub struct MockOracle;
impl IdentityOracleProvider<u64> for MockOracle {
    fn get_kyc_level(_account: &u64) -> Option<u8> {
        // Return None to rely on on-chain attestations
        None
    }
    
    fn meets_kyc_requirement(account: &u64, required_level: u8) -> bool {
        // Account 100 is "oracle-verified" for testing
        if *account == 100 && required_level <= 2 {
            return true;
        }
        false
    }
    
    fn is_sanctioned(account: &u64) -> bool {
        // Account 666 is sanctioned for testing
        *account == 666
    }
}

type Block = frame_system::mocking::MockBlock<Test>;

// Configure mock runtime to test the pallet
construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        BelizeIdentity: pallet_belize_identity,
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
    pub const IdentityPalletId: PalletId = PalletId(*b"id/vault");
    pub const TreasuryAccount: u64 = 1000;
    pub const MaxAccountsPerIdentity: u32 = 5;
    pub const MaxNameLen: u32 = 64;
    pub const MaxAnchorLen: u32 = 128;
    pub const MaxIssuerCount: u32 = 10;
    pub const KycValidityBlocks: u64 = 100; // 100 blocks (~annual in production)
    pub const KycGraceBlocks: u64 = 20; // 20 blocks grace period
    pub const MaxHistoryLen: u32 = 50;
}

impl pallet_belize_identity::Config for Test {
    type Currency = Balances;
    type PalletId = IdentityPalletId;
    type Treasury = TreasuryAccount;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type RevokeOrigin = EnsureRoot<Self::AccountId>;
    type Oracle = MockOracle;
    type MaxAccountsPerIdentity = MaxAccountsPerIdentity;
    type MaxNameLen = MaxNameLen;
    type MaxAnchorLen = MaxAnchorLen;
    type MaxIssuerCount = MaxIssuerCount;
    type KycValidityBlocks = KycValidityBlocks;
    type KycGraceBlocks = KycGraceBlocks;
    type MaxHistoryLen = MaxHistoryLen;
    type WeightInfo = ();
}

// Test accounts
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
pub const SSN_ISSUER: u64 = 10;
pub const PASSPORT_ISSUER: u64 = 11;
pub const BIO_ISSUER: u64 = 12;
pub const ORACLE_VERIFIED: u64 = 100;
pub const SANCTIONED: u64 = 666;
pub const TREASURY: u64 = 1000;

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 1_000_000),
            (BOB, 1_000_000),
            (CHARLIE, 1_000_000),
            (DAVE, 1_000_000),
            (EVE, 1_000_000),
            (SSN_ISSUER, 10_000_000),
            (PASSPORT_ISSUER, 10_000_000),
            (BIO_ISSUER, 10_000_000),
            (ORACLE_VERIFIED, 1_000_000),
            (SANCTIONED, 1_000_000),
            (TREASURY, 100_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_belize_identity::GenesisConfig::<Test> {
        operation_fee: Some(100), // 100 units per operation
        ssn_standard_version: 1,
        passport_standard_version: 1,
        initial_ssn_issuers: vec![SSN_ISSUER],
        initial_passport_issuers: vec![PASSPORT_ISSUER],
        initial_biometric_issuers: vec![BIO_ISSUER],
        paused: false,
        start_identity_id: 1000,
        issuer_bond_amount: Some(1_000_000), // 1M bond requirement
        rate_window_blocks: Some(50), // 50 block window
        rate_limit_ssn: Some(10), // 10 per window
        rate_limit_passport: Some(5), // 5 per window
        rate_limit_biometrics: Some(3), // 3 per window
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Helper to advance block number
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

// Helper to create test hash
pub fn test_hash(n: u8) -> H256 {
    H256::from([n; 32])
}

// Helper to create test anchor
pub fn test_anchor(n: u8) -> BoundedVec<u8, MaxAnchorLen> {
    let mut data = b"Qm".to_vec(); // IPFS CID prefix
    data.extend_from_slice(&[n; 40]);
    BoundedVec::try_from(data).unwrap()
}

// Helper to create test name
pub fn test_name(s: &str) -> BoundedVec<u8, MaxNameLen> {
    BoundedVec::try_from(s.as_bytes().to_vec()).unwrap()
}
