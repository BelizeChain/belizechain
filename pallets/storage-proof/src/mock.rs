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
    type AccountData = ();
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

impl pallet_storage_proof::Config for Test {
    type RevocationOrigin = RevocationOrigin;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}
