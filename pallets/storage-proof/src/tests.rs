//! Unit tests for pallet-storage-proof.

use crate::mock::{new_test_ext, RuntimeOrigin, Test as Runtime};
use crate::{BoundedVec, Error, MerkleProof, MerkleStep, ProofType};
use frame_support::traits::Get;
use frame_support::{assert_err, assert_ok};

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
            MerkleStep {
                node: s1,
                node_is_left: false,
            },
            MerkleStep {
                node: s2,
                node_is_left: false,
            },
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
        assert_ok!(crate::pallet::Pallet::<Runtime>::revoke_proof(
            RuntimeOrigin::signed(42),
            root
        ));
    });
}

#[test]
fn submit_reserves_deposit_and_revoke_refunds_it() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        let deposit = <Runtime as crate::Config>::StorageDeposit::get();
        let free_before = pallet_balances::Pallet::<Runtime>::free_balance(1);

        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        // The deposit is locked while the proof is stored.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(1),
            deposit
        );
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            free_before - deposit
        );

        assert_ok!(crate::pallet::Pallet::<Runtime>::revoke_proof(
            RuntimeOrigin::signed(42),
            root
        ));

        // Revocation refunds the deposit and clears the record.
        assert_eq!(pallet_balances::Pallet::<Runtime>::reserved_balance(1), 0);
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            free_before
        );
        assert!(!crate::StorageProofs::<Runtime>::contains_key(root));
    });
}

#[test]
fn submit_without_funds_for_deposit_fails() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        // Account 3 has no genesis balance, so it cannot cover the deposit.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(3),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::InsufficientDeposit
        );
        assert!(!crate::StorageProofs::<Runtime>::contains_key(root));
    });
}

#[test]
fn revoke_missing_proof_reports_not_found() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        assert_err!(
            crate::pallet::Pallet::<Runtime>::revoke_proof(RuntimeOrigin::signed(42), root),
            Error::<Runtime>::ProofNotFound
        );
    });
}
