use crate as pallet_belize_landledger;
use crate::LandLedgerOracleProvider;
use frame_support::{
    parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, Everything},
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
        LandLedger: pallet_belize_landledger,
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

parameter_types! {
    pub const RegistrationDeposit: u128 = 10_000; // 10K deposit to register property
    pub const TransferTaxRate: u32 = 250; // 2.5% transfer tax (250 basis points)
    pub const MaxDescriptionLength: u32 = 256;
}

// Mock Oracle provider for land ownership verification
pub struct MockOracle;
impl pallet_belize_landledger::LandLedgerOracleProvider<u64> for MockOracle {
    fn verify_land_owner(property_id: u32, account: &u64) -> bool {
        // Properties 1-100: Verified for accounts 1-3 (ALICE, BOB, CHARLIE)
        // Property 101: Verified for account 666 (SANCTIONED - to test oracle verification)
        // Others: Not verified
        match property_id {
            1..=100 => matches!(account, 1..=3),
            101 => *account == 666,
            _ => false,
        }
    }

    fn get_kyc_level(account: &u64) -> Option<u8> {
        match account {
            // ALICE, BOB, CHARLIE: Level 3 (Full KYC)
            1..=3 => Some(3),
            // Test accounts 10-50: Level 2 (Enhanced KYC - can buy property)
            10..=50 => Some(2),
            // Account 100: Level 1 (Basic KYC - insufficient for property)
            100 => Some(1),
            // Account 666: Sanctioned but has Level 2 (to test sanction check)
            666 => Some(2),
            // Others: No KYC
            _ => None,
        }
    }

    fn is_sanctioned(account: &u64) -> bool {
        // Account 666 is sanctioned
        *account == 666
    }
}

/// Mock KYC provider — delegates to MockOracle's get_kyc_level
pub struct MockKyc;
impl pallet_belize_identity::BelizeKyc<u64, u64> for MockKyc {
    fn is_kyc_verified(who: &u64, level: pallet_belize_identity::KycLevel, _now: u64) -> bool {
        let required = match level {
            pallet_belize_identity::KycLevel::L0 => 0,
            pallet_belize_identity::KycLevel::L1 => 1,
            pallet_belize_identity::KycLevel::L2 => 2,
            pallet_belize_identity::KycLevel::L3 => 3,
        };
        MockOracle::get_kyc_level(who).is_some_and(|l| l >= required)
    }
}

impl pallet_belize_landledger::Config for Test {
    type Currency = Balances;
    type BelizeKyc = MockKyc;
    type GovernmentOrigin = frame_system::EnsureRoot<u64>;
    type SurveyorOrigin = frame_system::EnsureRoot<u64>;
    type EnvironmentalOrigin = frame_system::EnsureRoot<u64>;
    type Oracle = MockOracle;
    type RegistrationDeposit = RegistrationDeposit;
    type TransferTaxRate = TransferTaxRate;
    type MaxDescriptionLength = MaxDescriptionLength;
    type WeightInfo = ();
    type MaxPropertyPrice = ConstU128<100_000_000_000_000_000_000>; // 100M * 10^12
}

// Test account constants
pub const ALICE: u64 = 1; // Level 3 KYC
pub const BOB: u64 = 2; // Level 3 KYC
pub const CHARLIE: u64 = 3; // Level 3 KYC
pub const EVE: u64 = 10; // Level 2 KYC
pub const FERDIE: u64 = 11; // Level 2 KYC
pub const NO_KYC: u64 = 200; // No KYC
pub const LOW_KYC: u64 = 100; // Level 1 KYC (insufficient)
pub const SANCTIONED: u64 = 666; // Sanctioned account
pub const GOVERNMENT: u64 = 1000; // Government account for administrative functions

// Helper function to create test title number
pub fn test_title(n: u32) -> Vec<u8> {
    format!("BZ-TITLE-{:06}", n).into_bytes()
}

// Helper function to create test description
pub fn test_description(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

// Helper function to create valid Belize coordinates
pub fn test_coordinates(lat_offset: i64, lon_offset: i64) -> (i64, i64) {
    // Belize latitude: ~15.5° to 18.5° N (15_500_000 to 18_500_000 micro-degrees)
    // Belize longitude: ~87.5° to 89.2° W (-89_200_000 to -87_500_000 micro-degrees)
    (17_000_000 + lat_offset, -88_000_000 + lon_offset)
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
            (GOVERNMENT, 1_000_000_000), // Government account
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

// Helper to run to a specific block
#[allow(dead_code)]
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
