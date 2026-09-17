//! Unit tests for pallet-storage-proof.

use crate::{
    mock::new_test_ext, mock::Test as Runtime, mock::*, Error, MerkleProof, MerkleStep,
    Pallet as StorageProof, ProofType, StorageProofs,
};
use frame_support::{assert_err, assert_ok, BoundedVec};
use sp_std::vec;

/// Builds a correct 2-level Merkle proof using BLAKE2-256.
/// leaf = blake2_256(b"block-data")
/// root = blake2_256( blake2_256(leaf \u{2029} s1) \u{2029} s2 )
fn build_valid_proof() -> ([u8; 32], MerkleProof) {
    let leaf = sp_io::hashing::blake2_256(b"block-data");
    let s1 = sp_io::hashing::blake2_256(b"sibling-1");
    let s2 = sp_io::hashing::blake2_256(b"sibling-2");

    let level1 = {
        let mut buf = sp_std::vec::Vec::with_capacity(64);
        buf.extend_from_slice(&leaf);
        buf.extend_from_slice(&s1);
        sp_io::hashing::blake2_256(&buf)
    };
    let root = {
        let mut buf = sp_std::vec::Vec::with_capacity(64);
        buf.extend_from_slice(&level1);
        buf.extend_from_slice(&s2);
        sp_io::hashing::blake2_256(&buf)
    };

    let proof = MerkleProof {
        leaf,
        siblings: BoundedVec::truncate_from(vec![
            MerkleStep { node: s1, node_is_left: false },
            MerkleStep { node: s2, node_is_left: false },
        ]),
    };

    (root, proof)
}

#[test]
fn accept_valid_merkle_proof() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));
        let record = crate::StorageProofs::<Runtime>::get(root).expect("should exist");
        assert_eq!(record.submitter, 1u64);
    });
}

#[test]
fn reject_root_mismatch() {
    new_test_ext().execute_with(|| {
        let (_, proof) = build_valid_proof();
        let wrong_root = sp_io::hashing::blake2_256(b"not-the-root");
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                wrong_root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::RootMismatch
        );
    });
}

#[test]
fn reject_empty_siblings() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        let empty = MerkleProof {
            leaf: [0u8; 32],
            siblings: BoundedVec::default(),
        };
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                empty,
                ProofType::Merkle,
            ),
            Error::<Runtime>::EmptyMerkleProof
        );
    });
}

#[test]
fn reject_groth16_always() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        // Stage-B not wired — even with a valid-looking proof we must refuse.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof,
                ProofType::Groth16,
            ),
            Error::<Runtime>::Groth16NotYetAccepted
        );
    });
}

#[test]
fn reject_duplicate_content() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof.clone(),
            ProofType::Merkle,
        ));
        // A second submission — even from a different account — must reject.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(2),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::ContentAlreadyProven
        );
    });
}

#[test]
fn revoke_only_by_authority() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));
        // Non-authority cannot revoke.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::revoke_proof(RuntimeOrigin::signed(1), root),
            sp_runtime::DispatchError::BadOrigin
        );
        // Authority (account 42) can.
        assert_ok!(crate::pallet::Pallet::<Runtime>::revoke_proof(RuntimeOrigin::signed(42), root));
    });
}
