//! Test runtime for `pallet-zk-attestation`.
//!
//! Deliberately minimal: the pallet has no currency, no identity coupling and no
//! asset dependency, so nothing else needs standing up to exercise it.

use crate::{self as pallet_zk_attestation, MAX_PROOF_LEN, MAX_VERIFYING_KEY_LEN};
use frame_support::{derive_impl, parameter_types};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        ZkAttestation: pallet_zk_attestation,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

parameter_types! {
    pub const MockMaxProofLen: u32 = MAX_PROOF_LEN;
    pub const MockMaxVerifyingKeyLen: u32 = MAX_VERIFYING_KEY_LEN;
}

impl pallet_zk_attestation::Config for Test {
    type RegistryAuthority = frame_system::EnsureRoot<u64>;
    type MaxProofLen = MockMaxProofLen;
    type MaxVerifyingKeyLen = MockMaxVerifyingKeyLen;
    type WeightInfo = ();
}

/// Fresh externalities with one block already produced.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test genesis is well-formed");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
