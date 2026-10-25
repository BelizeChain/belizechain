//! Unit tests for pallet-storage-proof.

use crate::mock::{
    advance, commit_and_advance, commit_only, new_test_ext, RuntimeOrigin, Test as Runtime,
    TestDuration, TestInterval, TEST_ESCROW, TEST_PROOFS, TEST_TRANCHE,
};
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

/// Deposit reserved by `register_content`, in mock balance units.
fn content_deposit() -> u128 {
    <<Runtime as crate::Config>::StorageDeposit as Get<u128>>::get()
}

/// Shorthand for the bounded CID parameter the extrinsics expect.
fn cid(bytes: &[u8]) -> BoundedVec<u8, crate::pallet::MaxIpfsCidLen> {
    BoundedVec::truncate_from(bytes.to_vec())
}

/// Shorthand for the bounded metadata parameter the extrinsics expect.
fn meta(bytes: &[u8]) -> BoundedVec<u8, crate::pallet::MaxContentMetadataLen> {
    BoundedVec::truncate_from(bytes.to_vec())
}

#[test]
fn register_content_stores_record_and_reserves_deposit() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];

        assert_ok!(crate::pallet::Pallet::<Runtime>::register_content(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"bafybeigdyrzt"),
            meta(br#"{"size":1024,"mime":"text/plain"}"#),
        ));

        let record =
            crate::ContentMetadata::<Runtime>::get(content_id).expect("record must exist");
        assert_eq!(record.owner, 1u64);
        assert_eq!(record.ipfs_cid.as_slice(), b"bafybeigdyrzt");
        assert_eq!(
            record.metadata.as_slice(),
            br#"{"size":1024,"mime":"text/plain"}"#
        );
        assert_eq!(record.updated_at, record.registered_at);
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(1),
            content_deposit()
        );
    });
}

/// Registering metadata must not be an upsert: overwriting another account's
/// row would let anyone rewrite data they neither own nor paid for.
#[test]
fn register_content_rejects_an_existing_record() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];
        assert_ok!(crate::pallet::Pallet::<Runtime>::register_content(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"cid-one"),
            meta(b"one"),
        ));

        assert_err!(
            crate::pallet::Pallet::<Runtime>::register_content(
                RuntimeOrigin::signed(2),
                content_id,
                cid(b"cid-two"),
                meta(b"two"),
            ),
            Error::<Runtime>::ContentAlreadyRegistered
        );

        // The original owner's row is untouched by the rejected attempt.
        let record =
            crate::ContentMetadata::<Runtime>::get(content_id).expect("record must exist");
        assert_eq!(record.owner, 1u64);
        assert_eq!(record.ipfs_cid.as_slice(), b"cid-one");
    });
}

#[test]
fn update_content_metadata_is_owner_only() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];
        assert_ok!(crate::pallet::Pallet::<Runtime>::register_content(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"cid-one"),
            meta(b"one"),
        ));

        assert_err!(
            crate::pallet::Pallet::<Runtime>::update_content_metadata(
                RuntimeOrigin::signed(2),
                content_id,
                cid(b"cid-hijack"),
                meta(b"hijack"),
            ),
            Error::<Runtime>::NotContentOwner
        );

        let record =
            crate::ContentMetadata::<Runtime>::get(content_id).expect("record must exist");
        assert_eq!(record.ipfs_cid.as_slice(), b"cid-one");

        advance(5);
        assert_ok!(crate::pallet::Pallet::<Runtime>::update_content_metadata(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"cid-updated"),
            meta(b"updated"),
        ));

        let record =
            crate::ContentMetadata::<Runtime>::get(content_id).expect("record must exist");
        assert_eq!(record.ipfs_cid.as_slice(), b"cid-updated");
        assert_eq!(record.metadata.as_slice(), b"updated");
        assert!(record.updated_at > record.registered_at);
    });
}

#[test]
fn deregister_content_returns_the_deposit() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];
        assert_ok!(crate::pallet::Pallet::<Runtime>::register_content(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"cid-one"),
            meta(b"one"),
        ));
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(1),
            content_deposit()
        );

        assert_err!(
            crate::pallet::Pallet::<Runtime>::deregister_content(
                RuntimeOrigin::signed(2),
                content_id
            ),
            Error::<Runtime>::NotContentOwner
        );

        assert_ok!(crate::pallet::Pallet::<Runtime>::deregister_content(
            RuntimeOrigin::signed(1),
            content_id
        ));
        assert!(!crate::ContentMetadata::<Runtime>::contains_key(content_id));
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(1),
            0
        );
    });
}

#[test]
fn content_metadata_calls_reject_unknown_content() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];
        assert_err!(
            crate::pallet::Pallet::<Runtime>::update_content_metadata(
                RuntimeOrigin::signed(1),
                content_id,
                cid(b"cid"),
                meta(b"m"),
            ),
            Error::<Runtime>::ContentNotFound
        );
        assert_err!(
            crate::pallet::Pallet::<Runtime>::deregister_content(
                RuntimeOrigin::signed(1),
                content_id
            ),
            Error::<Runtime>::ContentNotFound
        );
    });
}

/// An account with no free balance cannot pay the deposit, so it cannot register.
#[test]
fn register_content_requires_the_deposit() {
    new_test_ext().execute_with(|| {
        assert_err!(
            crate::pallet::Pallet::<Runtime>::register_content(
                RuntimeOrigin::signed(3),
                [9u8; 32],
                cid(b"cid"),
                meta(b"m"),
            ),
            Error::<Runtime>::InsufficientDeposit
        );
    });
}

/// The registry describes content; it must not imply the bytes were proven or
/// that any escrow exists. Otherwise a free `register_content` would be a
/// payout claim, which is exactly what `submit_storage_proof` refuses.
#[test]
fn content_registration_creates_no_commitment() {
    new_test_ext().execute_with(|| {
        let content_id = [9u8; 32];
        assert_ok!(crate::pallet::Pallet::<Runtime>::register_content(
            RuntimeOrigin::signed(1),
            content_id,
            cid(b"cid-one"),
            meta(b"one"),
        ));

        assert!(!crate::Commitments::<Runtime>::contains_key(content_id));
        assert!(!crate::StorageProofs::<Runtime>::contains_key(content_id));

        // The anchor rule is unchanged: a registered record still cannot be paid.
        let (root, proof) = build_valid_proof();
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::NoCommitment
        );
    });
}

#[test]
fn accept_valid_merkle_proof() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        // Account 2 funds the deal; account 1 will prove it.
        commit_and_advance(2, root);
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

/// The commitment is the pallet's anchor. Without one, a self-declared root
/// proves nothing and must not be rewarded.
#[test]
fn reject_without_commitment() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::NoCommitment
        );
        assert!(!crate::StorageProofs::<Runtime>::contains_key(root));
    });
}

/// A proof arriving before the commitment's interval has elapsed is rejected —
/// this spacing rule, not a duplicate guard, is what bounds payment.
#[test]
fn reject_proof_too_soon() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_only(2, root);
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::ProofTooSoon
        );
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

/// Repeat proofs of the same content are how tranches get paid, so a second
/// proof *after* the interval is accepted rather than rejected as a duplicate.
#[test]
fn repeat_proof_after_interval_pays_another_tranche() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_only(2, root);
        let client_reserved = pallet_balances::Pallet::<Runtime>::reserved_balance(2);

        advance(TestInterval::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof.clone(),
            ProofType::Merkle,
        ));

        // Same block, second attempt — too soon.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof.clone(),
                ProofType::Merkle,
            ),
            Error::<Runtime>::ProofTooSoon
        );

        advance(TestInterval::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(2),
            client_reserved - TEST_TRANCHE * 2
        );
    });
}

/// Once a deal has a provider, no other account can take its escrow — even
/// after authority revocation clears the proof record.
#[test]
fn reject_not_deal_provider() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_and_advance(2, root);
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof.clone(),
            ProofType::Merkle,
        ));

        assert_ok!(crate::pallet::Pallet::<Runtime>::revoke_proof(
            RuntimeOrigin::signed(42),
            root
        ));

        advance(TestInterval::get());
        assert_err!(
            crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(2),
                root,
                proof,
                ProofType::Merkle,
            ),
            Error::<Runtime>::NotDealProvider
        );
    });
}

#[test]
fn revoke_only_by_authority() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_and_advance(2, root);
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
        // Client 2 funds the deal; account 1 proves it, so account 1's reserved
        // balance is only ever the deposit.
        commit_and_advance(2, root);
        let deposit = <Runtime as crate::Config>::StorageDeposit::get();
        let free_before = pallet_balances::Pallet::<Runtime>::free_balance(1);

        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        // The deposit is locked while the proof is stored, and the provider was
        // paid one tranche out of the client's escrow.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::reserved_balance(1),
            deposit
        );
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            free_before - deposit + TEST_TRANCHE
        );

        assert_ok!(crate::pallet::Pallet::<Runtime>::revoke_proof(
            RuntimeOrigin::signed(42),
            root
        ));

        // Revocation refunds the deposit and clears the record. The tranche is
        // earned work, so it is deliberately not clawed back.
        assert_eq!(pallet_balances::Pallet::<Runtime>::reserved_balance(1), 0);
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            free_before + TEST_TRANCHE
        );
        assert!(!crate::StorageProofs::<Runtime>::contains_key(root));
    });
}

/// The provider's tranche is paid *before* the deposit is reserved, so a
/// provider holding almost nothing of its own can still submit.
///
/// This replaces the old `submit_without_funds_for_deposit_fails`: with escrow
/// in play the tranche covers the deposit, which makes
/// `Error::InsufficientDeposit` a defensive guard rather than a reachable path.
#[test]
fn first_tranche_covers_the_deposit_for_a_barely_funded_provider() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_and_advance(2, root);
        let deposit: u128 = <Runtime as crate::Config>::StorageDeposit::get();

        // Account 3 has no genesis balance. Give it the barest existence so it is
        // a real account before the payout arrives.
        <pallet_balances::Pallet<Runtime> as frame_support::traits::Currency<u64>>::make_free_balance_be(
            &3, 1,
        );

        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(3),
            root,
            proof,
            ProofType::Merkle,
        ));

        assert_eq!(pallet_balances::Pallet::<Runtime>::reserved_balance(3), deposit);
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(3),
            1 + TEST_TRANCHE - deposit
        );
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

// ---------------------------------------------------------------------------
// Commitment validation
// ---------------------------------------------------------------------------

#[test]
fn commit_rejects_escrow_below_minimum() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        // Bind first: `get()` needs a concrete type before the subtraction.
        let min_escrow: u128 = <Runtime as crate::Config>::MinEscrow::get();
        let below = min_escrow - 1;
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(2),
                root,
                below,
                TestInterval::get(),
                TestDuration::get(),
            ),
            Error::<Runtime>::EscrowBelowMinimum
        );
    });
}

#[test]
fn commit_rejects_interval_below_minimum() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        let below = <Runtime as crate::Config>::MinProofInterval::get() - 1;
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(2),
                root,
                TEST_ESCROW,
                below,
                TestDuration::get(),
            ),
            Error::<Runtime>::IntervalBelowMinimum
        );
    });
}

/// A term shorter than one interval could never be proved, so it is rejected
/// outright rather than creating a commitment that is born expired.
#[test]
fn commit_rejects_term_shorter_than_interval() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(2),
                root,
                TEST_ESCROW,
                TestDuration::get(),
                TestInterval::get(),
            ),
            Error::<Runtime>::InvalidDuration
        );
    });
}

#[test]
fn commit_rejects_duplicate_content_id() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        commit_only(2, root);
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(2),
                root,
                TEST_ESCROW,
                TestInterval::get(),
                TestDuration::get(),
            ),
            Error::<Runtime>::CommitmentExists
        );
    });
}

#[test]
fn commit_rejects_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        // Account 3 has no genesis balance.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(3),
                root,
                TEST_ESCROW,
                TestInterval::get(),
                TestDuration::get(),
            ),
            Error::<Runtime>::InsufficientEscrow
        );
        assert!(!crate::Commitments::<Runtime>::contains_key(root));
    });
}

// ---------------------------------------------------------------------------
// Escrow lifecycle
// ---------------------------------------------------------------------------

#[test]
fn refund_expired_returns_only_the_unspent_remainder() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        let client_free = pallet_balances::Pallet::<Runtime>::free_balance(2);
        commit_only(2, root);

        advance(TestInterval::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        // Term is still running, so nothing may be reclaimed yet.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::refund_expired(RuntimeOrigin::signed(1), root),
            Error::<Runtime>::CommitmentNotExpired
        );

        advance(TestDuration::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::refund_expired(
            RuntimeOrigin::signed(1),
            root
        ));

        // Only the unpaid escrow returns; the earned tranche stays with the
        // provider, and the content id is freed for a future deal.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(2),
            client_free - TEST_TRANCHE
        );
        assert!(!crate::Commitments::<Runtime>::contains_key(root));
        assert_eq!(
            crate::ClientCommitmentCount::<Runtime>::get(2u64),
            0,
            "the per-client slot must be freed"
        );
    });
}

/// Delivering every expected proof pays out the whole escrow. The commitment
/// then **persists** until its term ends rather than closing early, so the
/// content id stays claimed and cannot be re-funded in the meantime.
#[test]
fn commitment_persists_after_the_escrow_is_paid_out() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        let client_free = pallet_balances::Pallet::<Runtime>::free_balance(2);
        let provider_free = pallet_balances::Pallet::<Runtime>::free_balance(1);
        let deposit: u128 = <Runtime as crate::Config>::StorageDeposit::get();
        commit_only(2, root);

        for _ in 0..TEST_PROOFS {
            advance(TestInterval::get());
            assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
                RuntimeOrigin::signed(1),
                root,
                proof.clone(),
                ProofType::Merkle,
            ));
        }

        let commitment = crate::Commitments::<Runtime>::get(root).expect("persists");
        assert_eq!(commitment.proofs_paid, TEST_PROOFS as u32);
        assert_eq!(commitment.released, TEST_ESCROW);

        // The client paid the whole escrow, and no more.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(2),
            client_free - TEST_ESCROW
        );
        // The provider received it all, less the deposit it locked once.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            provider_free + TEST_ESCROW - deposit
        );

        // The term has not ended, so the content id is still claimed and cannot
        // be funded again yet — this is what "persist" buys.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::commit_storage(
                RuntimeOrigin::signed(2),
                root,
                TEST_ESCROW,
                TestInterval::get(),
                TestDuration::get(),
            ),
            Error::<Runtime>::CommitmentExists
        );

        // At term end the id is released, with nothing left to refund, and it
        // becomes fundable again.
        advance(TestDuration::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::refund_expired(
            RuntimeOrigin::signed(1),
            root
        ));
        assert!(!crate::Commitments::<Runtime>::contains_key(root));
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(2),
            client_free - TEST_ESCROW
        );
        assert_ok!(crate::pallet::Pallet::<Runtime>::commit_storage(
            RuntimeOrigin::signed(2),
            root,
            TEST_ESCROW,
            TestInterval::get(),
            TestDuration::get(),
        ));
    });
}

/// The client may end a deal early and recover the unspent escrow; the provider
/// keeps what it already earned.
#[test]
fn release_provider_returns_the_unspent_escrow() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        let client_free = pallet_balances::Pallet::<Runtime>::free_balance(2);
        let provider_free = pallet_balances::Pallet::<Runtime>::free_balance(1);
        let deposit: u128 = <Runtime as crate::Config>::StorageDeposit::get();
        commit_and_advance(2, root);
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        assert_ok!(crate::pallet::Pallet::<Runtime>::release_provider(
            RuntimeOrigin::signed(2),
            root
        ));

        // Client gets back everything except the one earned tranche.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(2),
            client_free - TEST_TRANCHE
        );
        // Provider keeps that tranche.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(1),
            provider_free + TEST_TRANCHE - deposit
        );
        assert!(!crate::Commitments::<Runtime>::contains_key(root));
        assert_eq!(crate::ClientCommitmentCount::<Runtime>::get(2u64), 0);
    });
}

#[test]
fn release_provider_only_by_the_client() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        commit_only(2, root);
        assert_err!(
            crate::pallet::Pallet::<Runtime>::release_provider(RuntimeOrigin::signed(3), root),
            Error::<Runtime>::NotCommitmentClient
        );
    });
}

// ---------------------------------------------------------------------------
// Challenges — making non-retention detectable by the funded party
// ---------------------------------------------------------------------------

#[test]
fn challenge_only_by_the_client() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_and_advance(2, root);
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        // The provider cannot challenge itself.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::challenge_provider(RuntimeOrigin::signed(1), root),
            Error::<Runtime>::NotCommitmentClient
        );
    });
}

#[test]
fn challenge_needs_a_claimed_provider() {
    new_test_ext().execute_with(|| {
        let (root, _) = build_valid_proof();
        commit_only(2, root);
        assert_err!(
            crate::pallet::Pallet::<Runtime>::challenge_provider(RuntimeOrigin::signed(2), root),
            Error::<Runtime>::NoProvider
        );
    });
}

#[test]
fn answered_challenge_clears_the_challenge() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        commit_and_advance(2, root);
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof.clone(),
            ProofType::Merkle,
        ));

        assert_ok!(crate::pallet::Pallet::<Runtime>::challenge_provider(
            RuntimeOrigin::signed(2),
            root
        ));
        assert!(crate::Commitments::<Runtime>::get(root)
            .expect("commitment")
            .challenged_at
            .is_some());

        advance(TestInterval::get());
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        assert!(crate::Commitments::<Runtime>::get(root)
            .expect("commitment")
            .challenged_at
            .is_none());
    });
}

/// An unanswered challenge is the client's exit: the chain itself verifies the
/// window elapsed with no proof, so the exit is earned rather than discretionary.
#[test]
fn unanswered_challenge_lets_the_client_reclaim_escrow() {
    new_test_ext().execute_with(|| {
        let (root, proof) = build_valid_proof();
        let client_free = pallet_balances::Pallet::<Runtime>::free_balance(2);
        let window = <Runtime as crate::Config>::ChallengeWindow::get();
        commit_and_advance(2, root);
        assert_ok!(crate::pallet::Pallet::<Runtime>::submit_storage_proof(
            RuntimeOrigin::signed(1),
            root,
            proof,
            ProofType::Merkle,
        ));

        assert_ok!(crate::pallet::Pallet::<Runtime>::challenge_provider(
            RuntimeOrigin::signed(2),
            root
        ));

        // Window still open — the provider must not be cut off early.
        assert_err!(
            crate::pallet::Pallet::<Runtime>::claim_challenge_failed(
                RuntimeOrigin::signed(2),
                root
            ),
            Error::<Runtime>::ChallengeStillOpen
        );

        advance(window);
        assert_ok!(crate::pallet::Pallet::<Runtime>::claim_challenge_failed(
            RuntimeOrigin::signed(2),
            root
        ));

        // Unspent escrow returns to the client; the earned tranche does not.
        assert_eq!(
            pallet_balances::Pallet::<Runtime>::free_balance(2),
            client_free - TEST_TRANCHE
        );
        assert!(!crate::Commitments::<Runtime>::contains_key(root));
    });
}
