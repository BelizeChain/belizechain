//! Mock runtime for BelizeX pallet tests
#![allow(dead_code)]

use super::*;
use crate as pallet_belize_belizex;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU32, Everything},
    PalletId,
};
use frame_system as system;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

pub type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

// Minimal KYC impl for tests
pub struct MockKyc;
impl KycCheck<AccountId> for MockKyc {
    fn is_kyc_ok(_who: &AccountId) -> bool {
        true
    }
    fn is_sanctioned(_who: &AccountId) -> bool {
        false
    }
}

// A dummy KYC that rejects to test gating
#[allow(dead_code)]
pub struct DenyKyc;
impl KycCheck<AccountId> for DenyKyc {
    fn is_kyc_ok(_who: &AccountId) -> bool {
        false
    }
    fn is_sanctioned(_who: &AccountId) -> bool {
        true
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const ExistentialDeposit: u128 = 1;
    pub const TradingFeeRate: u32 = 30; // 0.3%
    pub const TourismDiscountRate: u32 = 10; // 0.1%
    pub const ProtocolFeeToTreasuryBps: u32 = 5; // 0.05%
    pub const MinLiquidityAmount: u128 = 1;
    pub const MaxOracleDeviationBpsConst: u32 = 500; // 5%
    pub const DexPalletId: PalletId = PalletId(*b"belizexd");
    pub const OrderEscrowPalletId: PalletId = PalletId(*b"belizexo");
}

// Runtime for tests
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        BelizeX: pallet_belize_belizex,
    }
);

// System config
impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Block = Block;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
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

// Balances config to provide Currency for the DEX
impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = [u8; 8];
    type MaxFreezes = ConstU32<50>;
    type DoneSlashHandler = ();
}

// Randomness: use system Hash for simplicity
pub struct DummyRandom;
impl Randomness<sp_core::H256, u64> for DummyRandom {
    fn random(_subject: &[u8]) -> (sp_core::H256, u64) {
        (sp_core::H256::repeat_byte(7), 0)
    }
}

// Origin types
#[allow(dead_code)]
pub struct RootOrigin;

// Mock Oracle for tests
pub struct MockOracle;

thread_local! {
    /// Rate the mock oracle reports for every pair. `None` models a missing or
    /// stale rate, which the pallet's deviation guard must treat as fail-closed.
    static ORACLE_RATE: core::cell::RefCell<Option<u128>> =
        const { core::cell::RefCell::new(Some(1_000_000)) };
}

impl MockOracle {
    /// Set the rate every pair reports (1e6 precision).
    pub fn set_rate(rate: Option<u128>) {
        ORACLE_RATE.with(|r| *r.borrow_mut() = rate);
    }
    pub fn reset() {
        Self::set_rate(Some(1_000_000));
    }
}

impl BelizeXOracleProvider<AccountId> for MockOracle {
    fn get_crypto_exchange_rate(base_asset: u8, quote_asset: u8) -> Option<u128> {
        // Same orientation logic the runtime uses, but with no peg fallback: the
        // peg is the *runtime's* policy, and tests need to be able to model a
        // provider with no rate at all to exercise the fail-closed branch.
        let observed = ORACLE_RATE.with(|r| *r.borrow());
        crate::reference_rate(base_asset, quote_asset, observed, None)
    }
    fn is_tourism_merchant(_account: &AccountId) -> bool {
        false
    }
    fn get_trading_volume_tier(_account: &AccountId) -> u8 {
        0
    }
}

// Runtime for normal tests with KYC allowed
impl Config for Test {
    type Currency = Balances;
    // Test ledger: every asset maps to the mock `Balances`, which preserves the
    // pre-existing single-currency behaviour these tests were written against.
    // `MockAssetLedger` lives in `tests.rs` so it can use the test externality.
    type Assets = crate::tests::MockAssetLedger;
    type Randomness = DummyRandom;
    type TourismOrigin = system::EnsureRoot<AccountId>;
    type PairListingOrigin = system::EnsureRoot<AccountId>;
    type Treasury = TreasuryAccount;
    type TradingFeeRate = TradingFeeRate;
    type TourismDiscountRate = TourismDiscountRate;
    type MinLiquidityAmount = MinLiquidityAmount;
    type WeightInfo = ();
    type Kyc = MockKyc;
    type ProtocolFeeToTreasuryBps = ProtocolFeeToTreasuryBps;
    type Oracle = MockOracle;
    type MaxOracleDeviationBps = MaxOracleDeviationBpsConst;
    type DexPalletId = DexPalletId;
    type OrderEscrowPalletId = OrderEscrowPalletId;
    type MaxOrdersPerAccount = ConstU32<50>;
    type MaxOrdersPerPair = ConstU32<20>;
}

parameter_types! {
    pub static TreasuryAccount: AccountId = 999;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Thread-locals outlive a single test when the harness reuses threads, so
    // every test starts from the default oracle rate.
    MockOracle::reset();

    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000u128),
            (2, 1_000_000_000_000u128),
            (999, 1_000_000_000_000u128),
        ],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Assimilate BelizeX genesis into storage
    GenesisConfig::<Test> {
        enable_default_pairs: true,
        initial_providers: vec![],
        dev_seed_default_liquidity: true,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    sp_io::TestExternalities::new(t)
}

// An alternate runtime variant to test KYC rejection could be added if needed.
