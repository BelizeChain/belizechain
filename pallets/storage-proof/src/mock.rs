//! Test mock runtime for pallet-storage-proof.
#![allow(dead_code)]

use super::*;
use crate as pallet_storage_proof;
use frame_support::{parameter_types, traits::Everything};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

pub type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
        StorageProof: pallet_storage_proof::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 1981;
}

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
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = ();
}

parameter_types! {
    pub const RevocationAuthority: u64 = 42;
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

/// Authorizes account 42 as the revocation authority.
pub struct RevocationOrigin;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for RevocationOrigin {
    type Success = u64;
    fn try_origin(o: RuntimeOrigin) -> Result<u64, RuntimeOrigin> {
        Into::<Result<frame_system::RawOrigin<u64>, RuntimeOrigin>>::into(o).and_then(|o| match o {
            frame_system::RawOrigin::Signed(who) if who == 42 => Ok(who),
            r => Err(RuntimeOrigin::from(r)),
        })
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::from(frame_system::RawOrigin::Signed(42)))
    }
}

/// Term used by test commitments, in blocks — `TEST_PROOFS` × `TEST_INTERVAL_RAW`.
const TEST_DURATION_RAW: u64 = 100;
/// Proof interval used by test commitments, in blocks.
const TEST_INTERVAL_RAW: u64 = 10;

/// Escrow used by test commitments — comfortably above `MinEscrow`.
pub const TEST_ESCROW: u128 = 100_000_000_000;
/// Proofs a test commitment pays: `TEST_DURATION_RAW / TEST_INTERVAL_RAW`.
pub const TEST_PROOFS: u128 = 10;
/// Per-proof payout for a test commitment.
pub const TEST_TRANCHE: u128 = TEST_ESCROW / TEST_PROOFS;

parameter_types! {
    /// Minimum proof interval accepted by the mock.
    pub const MinStorageProofInterval: BlockNumberFor<Test> = 5;
    /// Longest term accepted by the mock.
    pub const MaxStorageCommitmentDuration: BlockNumberFor<Test> = 1_000_000;
    /// Challenge window used by the mock — 20 blocks.
    pub const StorageChallengeWindow: BlockNumberFor<Test> = 20;
    /// Interval for test commitments.
    pub const TestInterval: BlockNumberFor<Test> = TEST_INTERVAL_RAW as BlockNumberFor<Test>;
    /// Term for test commitments.
    pub const TestDuration: BlockNumberFor<Test> = TEST_DURATION_RAW as BlockNumberFor<Test>;
}

impl pallet_storage_proof::Config for Test {
    type RevocationOrigin = RevocationOrigin;
    type Currency = Balances;
    /// Refundable deposit used in tests — 1 DALLA at 12 decimals.
    type StorageDeposit = frame_support::traits::ConstU128<1_000_000_000>;
    /// Minimum escrow for tests — 10 DALLA at 12 decimals.
    type MinEscrow = frame_support::traits::ConstU128<10_000_000_000>;
    type MinProofInterval = MinStorageProofInterval;
    type MaxCommitmentDuration = MaxStorageCommitmentDuration;
    type MaxOpenCommitments = ConstU32<4>;
    type ChallengeWindow = StorageChallengeWindow;
    type WeightInfo = ();
}

/// Advances the mock chain by `blocks`.
pub fn advance(blocks: BlockNumberFor<Test>) {
    let now = frame_system::Pallet::<Test>::block_number();
    frame_system::Pallet::<Test>::set_block_number(now + blocks);
}

/// Opens a commitment for `content_id` funded by `client`, leaving the block
/// number untouched.
pub fn commit_only(client: u64, content_id: [u8; 32]) {
    frame_support::assert_ok!(crate::pallet::Pallet::<Test>::commit_storage(
        RuntimeOrigin::signed(client),
        content_id,
        TEST_ESCROW,
        TestInterval::get(),
        TestDuration::get(),
    ));
}

/// `commit_only` plus one interval, so the first proof is due.
///
/// Every successful submission needs a funded commitment — without one the
/// pallet's anchor check rejects the proof outright.
pub fn commit_and_advance(client: u64, content_id: [u8; 32]) {
    commit_only(client, content_id);
    advance(TestInterval::get());
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1u64, 1_000_000_000_000),
            (2u64, 1_000_000_000_000),
            (42u64, 1_000_000_000_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
