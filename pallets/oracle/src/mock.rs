//! Mock runtime for oracle pallet testing

use crate as pallet_belize_oracle;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
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
        Oracle: pallet_belize_oracle,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub TreasuryAccount: u64 = 999;
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
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
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

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MaxOperators: u32 = 10;
    pub const MaxDataStaleness: u64 = 100; // 100 blocks
    pub const MinConsensusOperators: u32 = 1; // Minimum 1 operator for testing (production should be 2+)
    pub const MinOracleAgreement: u32 = 1;
    pub const BehaviorCooldownBlocks: u64 = 50;
    pub const ExpiryDurationBlocks: u64 = 200;
    pub const MaxPriceDeviation: u32 = 500;
}

impl pallet_belize_oracle::Config for Test {
    type OracleAdminOrigin = frame_system::EnsureRoot<u64>;
    type MaxOperators = MaxOperators;
    type MaxDataStaleness = MaxDataStaleness;
    type MinConsensusOperators = MinConsensusOperators;
    type WeightInfo = crate::weights::SubstrateWeight<Test>;
    type Currency = Balances;
    type TreasuryAccount = TreasuryAccount;
    type MinOracleAgreement = MinOracleAgreement;
    type BehaviorCooldownBlocks = BehaviorCooldownBlocks;
    type ExpiryDurationBlocks = ExpiryDurationBlocks;
    type MaxPriceDeviation = MaxPriceDeviation;
}

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_belize_oracle::GenesisConfig::<Test> {
        operators: vec![1, 2], // Alice and Bob as initial operators
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Helper: submit identity verification from both ALICE (1) and BOB (2) so that
/// quorum is reached and the verification is finalized in storage.
pub fn verify_identity_with_quorum(
    account: u64,
    kyc_level: u8,
    id_hash: [u8; 32],
    provider: sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<64>>,
    biometric_verified: bool,
    address_verified: bool,
) {
    use frame_support::assert_ok;
    // First oracle vote (stages but does NOT finalize)
    assert_ok!(crate::Pallet::<Test>::verify_identity(
        RuntimeOrigin::signed(1),
        account,
        kyc_level,
        id_hash,
        provider.clone(),
        biometric_verified,
        address_verified,
    ));
    // Second oracle vote — reaches quorum and finalizes
    assert_ok!(crate::Pallet::<Test>::verify_identity(
        RuntimeOrigin::signed(2),
        account,
        kyc_level,
        id_hash,
        provider,
        biometric_verified,
        address_verified,
    ));
}
