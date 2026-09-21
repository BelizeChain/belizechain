//! Benchmarking for pallet-storage-proof.

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, EnsureOrigin, Get};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

/// Builds a valid `levels`-deep Merkle proof using the same BLAKE2-256
/// derivation the pallet verifies, so the benchmark exercises real verification
/// work rather than a mocked shortcut.
fn build_valid_proof(levels: u32) -> (ContentId, MerkleProof) {
    let leaf = sp_io::hashing::blake2_256(b"belizechain-bench-leaf");
    let mut current = leaf;
    let mut siblings = Vec::with_capacity(levels as usize);

    for i in 0..levels {
        let node = sp_io::hashing::blake2_256(&i.to_le_bytes());
        let mut buf = Vec::with_capacity(64);
        buf.extend_from_slice(&current);
        buf.extend_from_slice(&node);
        current = sp_io::hashing::blake2_256(&buf);
        siblings.push(MerkleStep {
            node,
            node_is_left: false,
        });
    }

    (
        current,
        MerkleProof {
            leaf,
            siblings: BoundedVec::truncate_from(siblings),
        },
    )
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// `s` is the number of Merkle siblings; the trait exposes it as a weight
    /// component because verification cost is linear in the proof depth.
    #[benchmark]
    fn submit_storage_proof(s: Linear<1, MAX_PROOF_STEPS>) {
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::StorageDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());

        let (root, proof) = build_valid_proof(s);

        #[extrinsic_call]
        submit_storage_proof(RawOrigin::Signed(caller), root, proof, ProofType::Merkle);
    }

    #[benchmark]
    fn revoke_proof() {
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::StorageDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());

        let (root, proof) = build_valid_proof(MAX_PROOF_STEPS);
        let _ = Pallet::<T>::submit_storage_proof(
            RawOrigin::Signed(caller).into(),
            root,
            proof,
            ProofType::Merkle,
        );

        // Resolve the privileged origin the same way both the mock and the real
        // runtime do, so this benchmark works with either origin configuration.
        let origin =
            T::RevocationOrigin::try_successful_origin().expect("revocation origin is configured");

        #[extrinsic_call]
        revoke_proof(origin, root);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
