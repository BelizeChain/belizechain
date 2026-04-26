//! Mock runtime for testing the Meshtastic mesh pallet

use crate as pallet_mesh;
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64},
    PalletId,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Mesh: pallet_mesh,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
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
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
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
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

// Mock identity provider
pub struct MockMeshIdentityProvider;
impl pallet_mesh::MeshIdentityProvider<u64> for MockMeshIdentityProvider {
    fn get_kyc_level(account: &u64) -> u8 {
        match account {
            // Account 1: Full KYC (level 3)
            1 => 3,
            // Account 2: Verified KYC (level 2)
            2 => 2,
            // Account 3: Basic KYC (level 1)
            3 => 1,
            // Account 4: No KYC
            4 => 0,
            // Account 5: Emergency authority with full KYC
            5 => 3,
            // Account 10: Validator with full KYC
            10 => 3,
            // Default: basic KYC
            _ => 1,
        }
    }

    fn is_emergency_authority(account: &u64) -> bool {
        // Account 5 is NEMO (National Emergency Management Organization)
        *account == 5
    }

    fn is_validator(account: &u64) -> bool {
        // Account 10 is a registered validator
        *account == 10
    }
}

parameter_types! {
    pub const MeshPalletId: PalletId = PalletId(*b"bz/mesht");
}

impl pallet_mesh::Config for Test {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type Identity = MockMeshIdentityProvider;
    type GovernanceOrigin = frame_system::EnsureRoot<u64>;
    type EmergencyOrigin = frame_system::EnsureRoot<u64>;
    type MaxMeshNodes = ConstU32<1000>;
    type MaxPendingMeshTx = ConstU32<500>;
    type MaxActiveAlerts = ConstU32<50>;
    type MaxRelayProofsPerClaim = ConstU32<100>;
    type RelayRewardPerTransaction = ConstU128<1_000_000_000>; // 1 DALLA
    type RelayRewardPerBlockHeader = ConstU128<500_000_000>; // 0.5 DALLA
    type RelayRewardPerEmergencyAlert = ConstU128<2_000_000_000>; // 2 DALLA
    type NodeRegistrationDeposit = ConstU128<10_000_000_000>; // 10 DALLA
    type HeartbeatTimeout = ConstU32<100>; // 100 blocks
    type MaxMeshTxPerBlock = ConstU32<50>;
    type WeightInfo = ();
}

/// Helper to create test externalities
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000),  // 1000 DALLA
            (2, 500_000_000_000),    // 500 DALLA
            (3, 100_000_000_000),    // 100 DALLA
            (4, 50_000_000_000),     // 50 DALLA (no KYC)
            (5, 500_000_000_000),    // 500 DALLA (emergency authority)
            (10, 1_000_000_000_000), // 1000 DALLA (validator)
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Advance to the given block number
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
