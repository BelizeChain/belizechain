use crate as pallet_economy;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
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
        Economy: pallet_economy,
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
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU64<1>;
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
    pub const TreasuryAccount: u64 = 100;
    pub const EconomyPalletId: PalletId = PalletId(*b"bz/trsry");
    // Max supply: 501B DALLA with 6 decimals = 501_000_000_000_000_000
    pub const MaxSupply: u64 = 501_000_000_000_000_000u64;
}

pub struct EnsureRootOrHalfCouncil;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureRootOrHalfCouncil {
    type Success = u64;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        Into::<Result<frame_system::RawOrigin<u64>, RuntimeOrigin>>::into(o).and_then(|o| match o {
            frame_system::RawOrigin::Root => Ok(0),
            frame_system::RawOrigin::Signed(who) if who == 1 => Ok(who), // Account 1 is governance
            r => Err(RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::from(frame_system::RawOrigin::Root))
    }
}

// Mock Oracle provider for testing
pub struct MockOracleProvider;
impl pallet_economy::OracleProvider<u64> for MockOracleProvider {
    fn is_merchant_verified(_merchant: &u64, _category: u8) -> bool {
        // For testing, always return true
        true
    }
        fn meets_kyc_requirement(_account: &u64, _required_level: u8) -> bool {
            // In mock environment, default to true to simplify tests
            true
        }
        fn is_sanctioned(_account: &u64) -> bool {
            // In mock environment, no sanctioned accounts
            false
        }
}

impl pallet_economy::Config for Test {
    type Currency = Balances;
    type Treasury = TreasuryAccount;
    type UnixTime = Timestamp;
    type WeightInfo = crate::SubstrateWeight<Test>;
    type MaxSupply = MaxSupply;
    type GovernanceOrigin = EnsureRootOrHalfCouncil;
    type Oracle = MockOracleProvider;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000), // Governance account
            (2, 1_000_000_000_000),     // User 1
            (3, 1_000_000_000_000),     // User 2
            (100, 10_000_000_000_000),  // Treasury
        ],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // Set initial total supply
        crate::TotalSupply::<Test>::put(1_000_000_000_000_000u64);
    });
    ext
}
