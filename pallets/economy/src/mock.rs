use crate as pallet_economy;
use frame_support::{
    parameter_types,
    traits::{ConstU128, ConstU32, ConstU64},
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
    // Mock max supply: 501_000_000_000_000_000 base units (= 501,000 DALLA at 12 decimals)
    pub const MaxSupply: u64 = 501_000_000_000_000_000u64;
    pub const PublicGoodsTreasuryAccount: u64 = 101;
    pub const PublicGoodsPercent: u8 = 10;
    pub const WellbeingTreasuryAccount: u64 = 102;
    pub const WellbeingPercent: u8 = 5;
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
    fn meets_kyc_requirement(account: &u64, required_level: u8) -> bool {
        // An account with no override passes every requirement, which is what the
        // rest of this suite relies on. `set_kyc_level` lets one test drive a
        // specific account below a requirement — without it the `transfer_bbzd`
        // KYC gates were unexercisable here, because a provider that always
        // returns `true` cannot fail.
        match KYC_LEVELS.with(|levels| levels.borrow().get(account).copied()) {
            Some(level) => level >= required_level,
            None => true,
        }
    }
    fn is_sanctioned(account: &u64) -> bool {
        SANCTIONED.with(|set| set.borrow().contains(account))
    }
}

#[cfg(test)]
thread_local! {
    /// Per-account KYC overrides. See `MockOracleProvider::meets_kyc_requirement`.
    static KYC_LEVELS: std::cell::RefCell<std::collections::BTreeMap<u64, u8>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

#[cfg(test)]
thread_local! {
    /// Accounts the mock treats as sanctioned.
    static SANCTIONED: std::cell::RefCell<std::collections::BTreeSet<u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

/// Drive `account`'s KYC level to exactly `level` for the current test.
pub fn set_kyc_level(account: u64, level: u8) {
    KYC_LEVELS.with(|levels| {
        levels.borrow_mut().insert(account, level);
    });
}

/// Mark `account` as sanctioned for the current test.
pub fn sanction(account: u64) {
    SANCTIONED.with(|set| {
        set.borrow_mut().insert(account);
    });
}

/// Clear every oracle override. Called by [`new_test_ext`] so one test cannot leak
/// into the next.
fn reset_oracle_overrides() {
    KYC_LEVELS.with(|levels| levels.borrow_mut().clear());
    SANCTIONED.with(|set| set.borrow_mut().clear());
}

impl pallet_economy::Config for Test {
    type Currency = Balances;
    type Treasury = TreasuryAccount;
    type UnixTime = Timestamp;
    type WeightInfo = crate::SubstrateWeight<Test>;
    type MaxSupply = MaxSupply;
    type GovernanceOrigin = EnsureRootOrHalfCouncil;
    type Oracle = MockOracleProvider;
    type MaxMintPerBlock = ConstU32<5>;
    type PublicGoodsTreasury = PublicGoodsTreasuryAccount;
    type PublicGoodsRoutingPercent = PublicGoodsPercent;
    type WellbeingTreasury = WellbeingTreasuryAccount;
    type WellbeingFundPercent = WellbeingPercent;
    type TravelRuleThreshold = ConstU128<100_000_000_000>; // 100 DALLA
    type ComplianceReporter = (); // No-op in tests
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    reset_oracle_overrides();

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
        // Sync TotalSupply to actual pallet_balances total issuance so it stays
        // consistent with the H-20 fix that syncs on every burn/mint operation.
        crate::TotalSupply::<Test>::put(pallet_balances::Pallet::<Test>::total_issuance());
    });
    ext
}
