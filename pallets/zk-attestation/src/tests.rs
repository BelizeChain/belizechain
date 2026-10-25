//! Tests for `pallet-zk-attestation`.
//!
//! The proof fixture is real — a genuine Groth16 proof over a genuine Poseidon
//! tree, generated once and shared. A fixture built from placeholder bytes would
//! pass every "the happy path works" test while proving nothing, which is the
//! exact failure mode this pallet exists to prevent.

use crate::{mock::*, Error, Event, RegistryRoots};
use belizechain_zk_attestation::{
    difference_bits, field_to_public_bytes, leaf_hash, poseidon_config, prove, record_hash,
    root_from_path, setup_for_testing, tree::hash_two, u64_bits, u64_field, AttestationCircuit,
    AttestationInputs, SparseTree, FIELD_BYTES, PUBLIC_INPUT_COUNT, TREE_DEPTH,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use std::sync::OnceLock;

const SET_ID: u32 = 1;
const CIRCUIT_VERSION: u16 = 1;
const RECORD_VALUE: u64 = 5_000;
const THRESHOLD: u64 = 1_000;
const LEAF_INDEX: u64 = 3;

/// A verifying key, a matching proof, and the public inputs it commits to.
struct Fixture {
    verifying_key: Vec<u8>,
    proof: Vec<u8>,
    inputs: AttestationInputs,
}

/// Generating a proof costs seconds, so build it once for the whole test binary.
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(build_fixture)
}

fn build_fixture() -> Fixture {
    let params = poseidon_config();
    let tag = u64_field(1);
    let salt = u64_field(99);
    let subject = u64_field(7);

    let mine = record_hash(&params, subject, u64_field(RECORD_VALUE), tag);
    let my_leaf = leaf_hash(&params, mine, salt);

    // A decoy, so the Merkle path is not trivially all-empty nodes.
    let decoy = leaf_hash(
        &params,
        record_hash(&params, u64_field(8), u64_field(42), tag),
        u64_field(11),
    );
    let tree = SparseTree::new(&params, [(LEAF_INDEX, my_leaf), (0, decoy)]).expect("valid leaves");
    let root = tree.root(&params);
    let path = tree.path(&params, LEAF_INDEX).expect("index in range");

    let index_bits: Vec<bool> = (0..TREE_DEPTH)
        .map(|level| (LEAF_INDEX >> level) & 1 == 1)
        .collect();

    let circuit = AttestationCircuit {
        params: params.clone(),
        set_root: Some(root),
        threshold: Some(u64_field(THRESHOLD)),
        nullifier: Some(hash_two(&params, salt, mine)),
        subject: Some(subject),
        tag: Some(tag),
        salt: Some(salt),
        value_bits: Some(u64_bits(RECORD_VALUE)),
        diff_bits: difference_bits(RECORD_VALUE, THRESHOLD),
        path: Some(path),
        index_bits: Some(index_bits),
    };

    let setup = setup_for_testing(&circuit).expect("setup");
    let material = prove(&circuit, &setup).expect("proof");

    // Sanity: the fixture must actually verify, or every test below is theatre.
    belizechain_zk_attestation::verify_attestation_inputs(
        &setup.verifying_key,
        &material.proof,
        &material.inputs,
    )
    .expect("the shared fixture must be a valid proof");

    Fixture {
        verifying_key: setup.verifying_key,
        proof: material.proof,
        inputs: material.inputs,
    }
}

/// Publish a key and a root, leaving the pallet ready to accept a proof.
fn publish_registry() {
    let fixture = fixture();
    assert_ok!(ZkAttestation::set_verifying_key(
        RuntimeOrigin::root(),
        CIRCUIT_VERSION,
        fixture.verifying_key.clone(),
    ));
    assert_ok!(ZkAttestation::set_registry_root(
        RuntimeOrigin::root(),
        SET_ID,
        fixture.inputs.set_root,
        CIRCUIT_VERSION,
        2,
    ));
}

fn submit(who: u64, proof: Vec<u8>, inputs: &[[u8; FIELD_BYTES]]) -> Result<(), DispatchError> {
    ZkAttestation::submit_attestation(RuntimeOrigin::signed(who), SET_ID, proof, inputs.to_vec())
}

// --- governance -----------------------------------------------------------

#[test]
fn only_the_registry_authority_may_publish() {
    new_test_ext().execute_with(|| {
        let fixture = fixture();
        assert_noop!(
            ZkAttestation::set_registry_root(
                RuntimeOrigin::signed(1),
                SET_ID,
                fixture.inputs.set_root,
                CIRCUIT_VERSION,
                2,
            ),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            ZkAttestation::set_verifying_key(
                RuntimeOrigin::signed(1),
                CIRCUIT_VERSION,
                fixture.verifying_key.clone(),
            ),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn a_root_may_not_name_a_missing_verifying_key() {
    new_test_ext().execute_with(|| {
        let fixture = fixture();
        assert_noop!(
            ZkAttestation::set_registry_root(
                RuntimeOrigin::root(),
                SET_ID,
                fixture.inputs.set_root,
                7,
                2,
            ),
            Error::<Test>::UnknownCircuitVersion,
        );
    });
}

#[test]
fn a_non_canonical_root_is_refused() {
    new_test_ext().execute_with(|| {
        let fixture = fixture();
        assert_ok!(ZkAttestation::set_verifying_key(
            RuntimeOrigin::root(),
            CIRCUIT_VERSION,
            fixture.verifying_key.clone(),
        ));

        // All-ones is far above the BLS12-381 scalar modulus.
        assert_noop!(
            ZkAttestation::set_registry_root(
                RuntimeOrigin::root(),
                SET_ID,
                [0xff; FIELD_BYTES],
                CIRCUIT_VERSION,
                2,
            ),
            Error::<Test>::NonCanonicalRoot,
        );
    });
}

#[test]
fn publishing_a_root_records_it_and_emits() {
    new_test_ext().execute_with(|| {
        publish_registry();

        assert_eq!(
            ZkAttestation::current_root(SET_ID),
            Some(fixture().inputs.set_root)
        );
        assert_eq!(RegistryRoots::<Test>::get(SET_ID).expect("stored").leaf_count, 2);
        assert!(System::events().iter().any(|record| matches!(
            record.event,
            RuntimeEvent::ZkAttestation(Event::RegistryRootSet { set_id: SET_ID, .. })
        )));
    });
}

#[test]
fn an_over_long_verifying_key_is_refused() {
    new_test_ext().execute_with(|| {
        let oversized = vec![0u8; (crate::MAX_VERIFYING_KEY_LEN + 1) as usize];
        assert_noop!(
            ZkAttestation::set_verifying_key(RuntimeOrigin::root(), CIRCUIT_VERSION, oversized),
            Error::<Test>::VerifyingKeyTooLong,
        );
    });
}

// --- accepting a proof ----------------------------------------------------

#[test]
fn an_honest_proof_is_accepted_and_consumes_its_nullifier() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        assert_ok!(submit(1, fixture.proof.clone(), &fixture.inputs.to_array()));

        assert!(ZkAttestation::is_nullifier_used(SET_ID, &fixture.inputs.nullifier));
        assert_eq!(crate::AttestationCount::<Test>::get(SET_ID), 1);
        assert!(System::events().iter().any(|record| matches!(
            record.event,
            RuntimeEvent::ZkAttestation(Event::AttestationAccepted { set_id: SET_ID, whose: 1, .. })
        )));
    });
}

#[test]
fn the_same_proof_cannot_be_submitted_twice() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        assert_ok!(submit(1, fixture.proof.clone(), &fixture.inputs.to_array()));
        // Same proof from a different account is refused too: the nullifier is
        // the record's, not the submitter's.
        assert_noop!(
            submit(2, fixture.proof.clone(), &fixture.inputs.to_array()),
            Error::<Test>::AlreadyAttested,
        );
    });
}

#[test]
fn a_proof_for_another_registry_is_refused() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        // A second set whose published root is a different value.
        let other_root = {
            let params = poseidon_config();
            let tree = SparseTree::new(&params, [(0u64, u64_field(4242))]).expect("valid");
            field_to_public_bytes(tree.root(&params))
        };
        assert_ok!(ZkAttestation::set_registry_root(
            RuntimeOrigin::root(),
            2,
            other_root,
            CIRCUIT_VERSION,
            1,
        ));

        let mut inputs = fixture.inputs.to_array();
        inputs[0] = other_root;
        assert_noop!(
            ZkAttestation::submit_attestation(
                RuntimeOrigin::signed(1),
                2,
                fixture.proof.clone(),
                inputs.to_vec(),
            ),
            Error::<Test>::ProofRejected,
        );
    });
}

#[test]
fn an_unknown_registry_is_refused() {
    new_test_ext().execute_with(|| {
        let fixture = fixture();
        assert_noop!(
            ZkAttestation::submit_attestation(
                RuntimeOrigin::signed(1),
                99,
                fixture.proof.clone(),
                fixture.inputs.to_array().to_vec(),
            ),
            Error::<Test>::UnknownRegistry,
        );
    });
}

#[test]
fn the_wrong_number_of_public_inputs_is_refused() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        assert_noop!(
            submit(1, fixture.proof.clone(), &fixture.inputs.to_array()[..2]),
            Error::<Test>::WrongPublicInputCount,
        );
    });
}

#[test]
fn a_threshold_near_the_modulus_is_refused() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        // A canonical field element, so nothing else would object — this is the
        // soundness hole the range check closes.
        let mut near_modulus = [0u8; FIELD_BYTES];
        near_modulus.copy_from_slice(&belizechain_zk_attestation::encoding::FR_MODULUS_LE);
        near_modulus[0] -= 1;

        let mut inputs = fixture.inputs.to_array();
        inputs[1] = near_modulus;

        assert_noop!(
            submit(1, fixture.proof.clone(), &inputs),
            Error::<Test>::UnsupportedThreshold,
        );
    });
}

#[test]
fn garbage_proof_bytes_are_refused_without_panicking() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        assert_noop!(
            submit(1, vec![0u8; 192], &fixture.inputs.to_array()),
            Error::<Test>::ProofRejected,
        );
        assert_noop!(
            submit(1, Vec::new(), &fixture.inputs.to_array()),
            Error::<Test>::ProofRejected,
        );
    });
}

#[test]
fn an_oversized_proof_is_refused_before_it_is_decoded() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        // Weight is charged per extrinsic, not per byte, so the cap is what stops
        // a caller from buying unbounded decoding work.
        let oversized = vec![0xab; (crate::MAX_PROOF_LEN + 1) as usize];
        assert_noop!(
            submit(1, oversized, &fixture.inputs.to_array()),
            Error::<Test>::ProofTooLong,
        );
    });
}

#[test]
fn a_tampered_public_input_is_refused() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        // Claim a higher threshold than the one actually proven.
        let mut inputs = fixture.inputs.to_array();
        inputs[1] = field_to_public_bytes(u64_field(RECORD_VALUE + 1));
        assert_noop!(
            submit(1, fixture.proof.clone(), &inputs),
            Error::<Test>::ProofRejected,
        );

        // Forge the nullifier to dodge the replay check.
        let mut inputs = fixture.inputs.to_array();
        inputs[2] = field_to_public_bytes(u64_field(31337));
        assert_noop!(
            submit(1, fixture.proof.clone(), &inputs),
            Error::<Test>::ProofRejected,
        );
    });
}

#[test]
fn a_rejected_proof_leaves_no_trace() {
    new_test_ext().execute_with(|| {
        publish_registry();
        let fixture = fixture();

        let mut inputs = fixture.inputs.to_array();
        inputs[1] = field_to_public_bytes(u64_field(RECORD_VALUE + 1));
        assert_noop!(
            submit(1, fixture.proof.clone(), &inputs),
            Error::<Test>::ProofRejected,
        );

        // The honest nullifier is still free, and the counter did not move.
        assert!(!ZkAttestation::is_nullifier_used(SET_ID, &fixture.inputs.nullifier));
        assert_eq!(crate::AttestationCount::<Test>::get(SET_ID), 0);
    });
}

#[test]
fn the_arity_constant_matches_the_fixture() {
    // Guards the contract between the circuit, the verifier and this pallet.
    assert_eq!(fixture().inputs.to_array().len(), PUBLIC_INPUT_COUNT);
    assert_eq!(PUBLIC_INPUT_COUNT, 3);
}

#[test]
fn the_merkle_path_in_the_fixture_is_genuine() {
    // A placeholder path would make the honest test meaningless, so check the
    // path independently of the proof.
    let params = poseidon_config();
    let tag = u64_field(1);
    let salt = u64_field(99);
    let record = record_hash(&params, u64_field(7), u64_field(RECORD_VALUE), tag);
    let leaf = leaf_hash(&params, record, salt);

    assert_eq!(
        field_to_public_bytes(root_from_path(&params, leaf, LEAF_INDEX, &rebuilt_path()).unwrap()),
        fixture().inputs.set_root,
    );
}

/// Rebuild the fixture's path from scratch, so the Merkle assertion does not
/// depend on the fixture holding it.
fn rebuilt_path() -> Vec<belizechain_zk_attestation::Fr> {
    let params = poseidon_config();
    let tag = u64_field(1);
    let decoy = leaf_hash(
        &params,
        record_hash(&params, u64_field(8), u64_field(42), tag),
        u64_field(11),
    );
    let leaf = leaf_hash(
        &params,
        record_hash(&params, u64_field(7), u64_field(RECORD_VALUE), tag),
        u64_field(99),
    );
    let tree = SparseTree::new(&params, [(LEAF_INDEX, leaf), (0, decoy)]).unwrap();
    tree.path(&params, LEAF_INDEX).unwrap()
}
