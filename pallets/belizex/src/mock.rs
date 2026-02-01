//! Mock runtime for BelizeX pallet tests
#![cfg(test)]
#![allow(clippy::duplicated_attributes)]
#![allow(dead_code)]

use crate as pallet_belize_belizex;
use super::*;
use frame_support::{parameter_types, traits::{ConstU16, ConstU32, Everything}};
use frame_system as system;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, BuildStorage};

pub type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

// Minimal KYC impl for tests
pub struct MockKyc;
impl KycCheck<AccountId> for MockKyc {
    fn is_kyc_ok(_who: &AccountId) -> bool { true }
}

// A dummy KYC that rejects to test gating
#[allow(dead_code)]
pub struct DenyKyc;
impl KycCheck<AccountId> for DenyKyc {
    fn is_kyc_ok(_who: &AccountId) -> bool { false }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const ExistentialDeposit: u128 = 1;
    pub const TradingFeeRate: u32 = 30; // 0.3%
    pub const TourismDiscountRate: u32 = 10; // 0.1%
    pub const ProtocolFeeToTreasuryBps: u32 = 5; // 0.05%
    pub const MinLiquidityAmount: u128 = 1;
    pub const MaxOracleDeviationBpsConst: u32 = 500; // 5%
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
impl BelizeXOracleProvider<AccountId> for MockOracle {
    fn get_crypto_exchange_rate(_base_asset: u8, _quote_asset: u8) -> Option<u128> {
        Some(1_000_000) // 1:1 rate
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
}

parameter_types! {
    pub static TreasuryAccount: AccountId = 999;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1_000_000_000_000u128), (2, 1_000_000_000_000u128), (999, 1_000_000_000_000u128)],
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
