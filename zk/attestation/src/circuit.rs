//! The Groth16 circuit behind an attestation proof.
//!
//! # What it proves
//!
//! *"Some record committed in registry `set_root` satisfies `value >= threshold`,
//! and I have not attested before."* Nothing else about the record is revealed —
//! not the subject, not the value, not the tag, not even which leaf it sits in.
//!
//! Public: `[set_root, threshold, nullifier]`.
//! Witness: `subject`, `value_bits`, `diff_bits`, `tag`, `salt`, `path`, `index_bits`.
//!
//! # Why this is stronger than checking a Merkle path natively
//!
//! A plain Merkle proof proves "I hold a path to some leaf", and the prover
//! chooses which leaf. That discloses the record and proves nothing about it. A
//! predicate over private data can only be proven by a succinct argument, which
//! is what this circuit is for.
//!
//! # Ordering of public inputs
//!
//! `set_root`, then `threshold`, then `nullifier` — the order the constraint
//! system allocates them, and therefore the order the verifier must supply. The
//! pallet and the prover CLI both build the array the same way; `PUBLIC_INPUT_COUNT`
//! is the shared constant.
//!
//! # Nullifier choice
//!
//! `nullifier = Poseidon(salt, record_hash)`, deliberately *not* keyed on
//! `set_root`: governance re-publishing a root after a registry update would
//! otherwise mint a fresh nullifier for everybody and let one record attest
//! twice. `salt` is fixed when the tree is built — the leaf commits to it — so a
//! prover cannot pick a new one and dodge the replay check.

use alloc::vec::Vec;

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::constraints::CryptographicSpongeVar;
use ark_crypto_primitives::sponge::poseidon::constraints::PoseidonSpongeVar;
use ark_crypto_primitives::sponge::poseidon::PoseidonConfig;
use ark_ff::{BigInt, PrimeField};
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::select::CondSelectGadget;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use crate::tree::TREE_DEPTH;
use crate::verify::AttestationInputs;

/// Width of the value field, and of the comparison difference.
///
/// Values above `2^64` are unsupported rather than handled: a 64-bit range is
/// far beyond any DALLA or bBZD amount (12 decimals, so ~1.8e7 DALLA would
/// already be 1e19), and fixing the width keeps the circuit shape constant.
pub const VALUE_BITS: usize = 64;

/// Witness and public inputs for one attestation.
#[derive(Clone)]
pub struct AttestationCircuit {
    /// Poseidon parameters. Must be the same instance used to build the tree —
    /// prover and verifier disagreeing here produces proofs that never verify and
    /// no compile error.
    pub params: PoseidonConfig<Fr>,

    /// Registry root the record must be committed under.
    pub set_root: Option<Fr>,
    /// Minimum value the record must meet, inclusive.
    pub threshold: Option<Fr>,
    /// Anti-replay tag, revealed.
    pub nullifier: Option<Fr>,

    /// Record subject, e.g. a hashed BelizeID.
    pub subject: Option<Fr>,
    /// Record tag, e.g. the schema or claim type.
    pub tag: Option<Fr>,
    /// Per-record nonce, fixed when the leaf was created.
    pub salt: Option<Fr>,

    /// Little-endian bits of the record value.
    pub value_bits: Option<Vec<bool>>,
    /// Little-endian bits of `value - threshold`. Proving this exists and fits in
    /// `VALUE_BITS` is what enforces `value >= threshold` without revealing either.
    pub diff_bits: Option<Vec<bool>>,

    /// Sibling hashes, lowest level first.
    pub path: Option<Vec<Fr>>,
    /// Position bits, lowest level first. Private: revealing it would identify
    /// the record.
    pub index_bits: Option<Vec<bool>>,
}

impl AttestationCircuit {
    /// An empty instance with the canonical Poseidon parameters.
    pub fn empty() -> Self {
        Self {
            params: crate::tree::poseidon_config(),
            set_root: None,
            threshold: None,
            nullifier: None,
            subject: None,
            tag: None,
            salt: None,
            value_bits: None,
            diff_bits: None,
            path: None,
            index_bits: None,
        }
    }
}

/// Poseidon over the given variables, absorbed in order.
fn poseidon_var(
    cs: ConstraintSystemRef<Fr>,
    params: &PoseidonConfig<Fr>,
    inputs: &[FpVar<Fr>],
) -> Result<FpVar<Fr>, SynthesisError> {
    let mut sponge = PoseidonSpongeVar::<Fr>::new(cs, params);
    for input in inputs {
        sponge.absorb(input)?;
    }
    let mut out = sponge.squeeze_field_elements(1)?;
    out.pop().ok_or(SynthesisError::Unsatisfiable)
}

fn alloc_bits(
    cs: &ConstraintSystemRef<Fr>,
    bits: &Option<Vec<bool>>,
    len: usize,
) -> Result<Vec<Boolean<Fr>>, SynthesisError> {
    let values = match bits {
        Some(bits) if bits.len() == len => bits.as_slice(),
        Some(_) => return Err(SynthesisError::Unsatisfiable),
        None => {
            return (0..len)
                .map(|_| {
                    Boolean::new_witness(cs.clone(), || -> Result<bool, SynthesisError> {
                        Err(SynthesisError::AssignmentMissing)
                    })
                })
                .collect()
        }
    };
    values
        .iter()
        .map(|bit| Boolean::new_witness(cs.clone(), || Ok(*bit)))
        .collect()
}

fn alloc_fields(
    cs: &ConstraintSystemRef<Fr>,
    values: &Option<Vec<Fr>>,
    len: usize,
) -> Result<Vec<FpVar<Fr>>, SynthesisError> {
    let values = match values {
        Some(values) if values.len() == len => values.as_slice(),
        Some(_) => return Err(SynthesisError::Unsatisfiable),
        None => {
            return (0..len)
                .map(|_| {
                    FpVar::new_witness(cs.clone(), || -> Result<Fr, SynthesisError> {
                        Err(SynthesisError::AssignmentMissing)
                    })
                })
                .collect()
        }
    };
    values
        .iter()
        .map(|value| FpVar::new_witness(cs.clone(), || Ok(*value)))
        .collect()
}

fn alloc_public(cs: &ConstraintSystemRef<Fr>, value: Option<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
    FpVar::new_input(cs.clone(), || value.ok_or(SynthesisError::AssignmentMissing))
}

fn alloc_witness(
    cs: &ConstraintSystemRef<Fr>,
    value: Option<Fr>,
) -> Result<FpVar<Fr>, SynthesisError> {
    FpVar::new_witness(cs.clone(), || value.ok_or(SynthesisError::AssignmentMissing))
}

fn generate_constraints_impl(
    witness: AttestationCircuit,
    cs: ConstraintSystemRef<Fr>,
) -> Result<(), SynthesisError> {
    let params = witness.params;

    // Public inputs, in the order the verifier supplies them.
    let set_root = alloc_public(&cs, witness.set_root)?;
    let threshold = alloc_public(&cs, witness.threshold)?;
    let nullifier = alloc_public(&cs, witness.nullifier)?;

    // Private witness.
    let subject = alloc_witness(&cs, witness.subject)?;
    let tag = alloc_witness(&cs, witness.tag)?;
    let salt = alloc_witness(&cs, witness.salt)?;
    let value_bits = alloc_bits(&cs, &witness.value_bits, VALUE_BITS)?;
    let diff_bits = alloc_bits(&cs, &witness.diff_bits, VALUE_BITS)?;
    let path = alloc_fields(&cs, &witness.path, TREE_DEPTH)?;
    let index_bits = alloc_bits(&cs, &witness.index_bits, TREE_DEPTH)?;

    // 1. Recover the value from its bits. `le_bits_to_fp` also enforces that every
    //    bit is a bit, which is what stops a prover handing over a field element
    //    as a "bit" to make the comparison lie.
    let value = Boolean::le_bits_to_fp(&value_bits)?;
    let diff = Boolean::le_bits_to_fp(&diff_bits)?;

    // 2. `value - threshold == diff` over the field. All three are below 2^65, so
    //    the field equation holds exactly when the integer one does — there is no
    //    room to wrap around the modulus.
    (value.clone() - threshold).enforce_equal(&diff)?;

    // 3. The leaf must commit to this record.
    let record = poseidon_var(cs.clone(), &params, &[subject, value, tag])?;
    let leaf = poseidon_var(cs.clone(), &params, &[record.clone(), salt.clone()])?;

    // 4. Walk the path to the root. Bit `i` decides which side the running hash
    //    sits on, matching `tree::root_from_path` exactly.
    let mut current = leaf;
    for (level, sibling) in path.iter().enumerate() {
        let bit = &index_bits[level];
        let left = FpVar::conditionally_select(bit, sibling, &current)?;
        let right = FpVar::conditionally_select(bit, &current, sibling)?;
        current = poseidon_var(cs.clone(), &params, &[left, right])?;
    }
    current.enforce_equal(&set_root)?;

    // 5. One attestation per record, no matter how many roots governance publishes.
    let expected_nullifier = poseidon_var(cs.clone(), &params, &[salt, record])?;
    expected_nullifier.enforce_equal(&nullifier)?;

    Ok(())
}

impl ark_relations::r1cs::ConstraintSynthesizer<Fr> for AttestationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        generate_constraints_impl(self, cs)
    }
}

/// Encode a scalar as the 32-byte little-endian canonical form the verifier reads.
pub fn field_to_public_bytes(value: Fr) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let limbs = value.into_bigint().0;
    for (i, limb) in limbs.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
    }
    bytes
}

/// Rebuild a scalar from its 32-byte little-endian form, rejecting non-canonical
/// encodings — a value at or above the modulus has more than one accepted
/// spelling, which is how malleable proofs get made.
pub fn public_bytes_to_field(bytes: &[u8; 32]) -> Option<Fr> {
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().ok()?);
    }
    Fr::from_bigint(BigInt(limbs))
}

/// Decompose `value - threshold` into little-endian bits, or `None` when
/// `value < threshold` (no valid proof exists in that case).
pub fn difference_bits(value: u64, threshold: u64) -> Option<Vec<bool>> {
    let difference = value.checked_sub(threshold)?;
    Some((0..VALUE_BITS).map(|i| (difference >> i) & 1 == 1).collect())
}

/// Little-endian bits of a `u64`, padded to `VALUE_BITS`.
pub fn u64_bits(value: u64) -> Vec<bool> {
    (0..VALUE_BITS).map(|i| (value >> i) & 1 == 1).collect()
}

/// Convenience for tests and the CLI: the scalar a `u64` becomes.
pub fn u64_field(value: u64) -> Fr {
    Fr::from(value)
}

// --- proving -------------------------------------------------------------

/// A Groth16 common reference string: the keys produced by a setup.
#[derive(Clone)]
pub struct Setup {
    /// Needed to produce proofs. Never published — a leaked proving key is not
    /// itself fatal, but it is not public material either.
    pub proving_key: Vec<u8>,
    /// The only half the chain sees.
    pub verifying_key: Vec<u8>,
}

/// What a prover hands to a verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestationMaterial {
    /// Encoded proof.
    pub proof: Vec<u8>,
    /// The three scalars the proof commits to.
    pub inputs: AttestationInputs,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProverError {
    /// A public input or witness field was `None`.
    MissingWitness,
    /// Parameter generation failed.
    SetupFailed,
    /// Proof construction failed — usually an unsatisfied constraint, which means
    /// the witness is lying about something.
    ProveFailed,
    /// A key or proof could not be encoded.
    SerializationFailed,
}

impl core::fmt::Display for ProverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::MissingWitness => "the witness is incomplete",
            Self::SetupFailed => "parameter generation failed",
            Self::ProveFailed => "proof construction failed; the witness does not satisfy the circuit",
            Self::SerializationFailed => "a key or proof could not be encoded",
        };
        f.write_str(message)
    }
}

/// Read the three public values off a populated circuit.
pub fn public_inputs_of(circuit: &AttestationCircuit) -> Option<AttestationInputs> {
    Some(AttestationInputs {
        set_root: field_to_public_bytes(circuit.set_root?),
        threshold: field_to_public_bytes(circuit.threshold?),
        nullifier: field_to_public_bytes(circuit.nullifier?),
    })
}

/// Generate a common reference string for this circuit shape.
///
/// # This is not a trusted setup, and the name says so
///
/// Groth16 needs a per-circuit ceremony. Whoever holds the toxic waste from that
/// ceremony can forge proofs — silently, for every verifier, forever. Running
/// this function locally produces a CRS whose toxic waste is in this process's
/// memory, which is fine for tests and catastrophic for production.
///
/// A real deployment must derive the CRS from a multi-party ceremony with at
/// least one honest participant, and publish only the verifying key. Until that
/// has happened, an attestation proves the prover knows *a* witness, not that the
/// witness is true.
pub fn setup_for_testing(circuit: &AttestationCircuit) -> Result<Setup, ProverError> {
    use ark_bls12_381::Bls12_381;
    use ark_groth16::Groth16;
    use ark_serialize::{CanonicalSerialize, Compress};
    use ark_snark::SNARK;
    use ark_std::rand::{rngs::StdRng, SeedableRng};

    let mut rng = StdRng::from_rng(ark_std::rand::thread_rng())
        .map_err(|_| ProverError::SetupFailed)?;
    let (proving_key, verifying_key) =
        Groth16::<Bls12_381>::circuit_specific_setup(circuit.clone(), &mut rng)
            .map_err(|_| ProverError::SetupFailed)?;

    let mut proving_key_bytes = Vec::new();
    proving_key
        .serialize_with_mode(&mut proving_key_bytes, Compress::Yes)
        .map_err(|_| ProverError::SerializationFailed)?;
    let mut verifying_key_bytes = Vec::new();
    verifying_key
        .serialize_with_mode(&mut verifying_key_bytes, Compress::Yes)
        .map_err(|_| ProverError::SerializationFailed)?;

    Ok(Setup {
        proving_key: proving_key_bytes,
        verifying_key: verifying_key_bytes,
    })
}

/// Build a proof for `circuit` using `setup`'s proving key.
///
/// The returned inputs are read from the circuit itself rather than passed
/// alongside, so a caller cannot publish a proof next to inputs it does not
/// match.
pub fn prove(circuit: &AttestationCircuit, setup: &Setup) -> Result<AttestationMaterial, ProverError> {
    use ark_bls12_381::Bls12_381;
    use ark_groth16::{Groth16, ProvingKey};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
    use ark_snark::SNARK;
    use ark_std::rand::{rngs::StdRng, SeedableRng};

    let inputs = public_inputs_of(circuit).ok_or(ProverError::MissingWitness)?;

    let proving_key = ProvingKey::<Bls12_381>::deserialize_with_mode(
        &setup.proving_key[..],
        Compress::Yes,
        Validate::Yes,
    )
    .map_err(|_| ProverError::SetupFailed)?;

    let mut rng = StdRng::from_rng(ark_std::rand::thread_rng())
        .map_err(|_| ProverError::ProveFailed)?;
    let proof = Groth16::<Bls12_381>::prove(&proving_key, circuit.clone(), &mut rng)
        .map_err(|_| ProverError::ProveFailed)?;

    let mut proof_bytes = Vec::new();
    proof
        .serialize_with_mode(&mut proof_bytes, Compress::Yes)
        .map_err(|_| ProverError::SerializationFailed)?;

    Ok(AttestationMaterial {
        proof: proof_bytes,
        inputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{hash_two, leaf_hash, poseidon_config, record_hash, root_from_path, SparseTree};
    use crate::verify::verify_attestation;
    use crate::verify::{PUBLIC_INPUT_BYTES, PUBLIC_INPUT_COUNT};
    use ark_bls12_381::Bls12_381;
    use ark_groth16::{Groth16, Proof, VerifyingKey};
    use ark_serialize::{CanonicalDeserialize, Compress, Validate};
    use ark_snark::SNARK;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};

    const HONEST_VALUE: u64 = 5_000;
    const HONEST_THRESHOLD: u64 = 1_000;

    struct Fixture {
        params: PoseidonConfig<Fr>,
        root: Fr,
        circuit: AttestationCircuit,
    }

    /// One registry with three records, one of which we hold.
    fn fixture() -> Fixture {
        let params = poseidon_config();
        let tag = Fr::from(1u64);

        let mine = record_hash(&params, Fr::from(7u64), u64_field(HONEST_VALUE), tag);
        let salt = Fr::from(99u64);
        let my_leaf = leaf_hash(&params, mine, salt);

        // Two decoys, so the path has real siblings rather than only empty nodes.
        let decoy_a = leaf_hash(
            &params,
            record_hash(&params, Fr::from(8u64), u64_field(42), tag),
            Fr::from(11u64),
        );
        let decoy_b = leaf_hash(
            &params,
            record_hash(&params, Fr::from(9u64), u64_field(7), tag),
            Fr::from(12u64),
        );

        let index = 3u64;
        let tree = SparseTree::new(&params, [(index, my_leaf), (0, decoy_a), (9, decoy_b)]).unwrap();
        let root = tree.root(&params);
        let path = tree.path(&params, index).unwrap();

        // The native walk must agree with the tree, or every circuit proof fails.
        assert_eq!(root_from_path(&params, my_leaf, index, &path).unwrap(), root);

        let index_bits = (0..TREE_DEPTH).map(|i| (index >> i) & 1 == 1).collect();
        let nullifier = hash_two(&params, salt, mine);

        Fixture {
            params: params.clone(),
            root,
            circuit: AttestationCircuit {
                params,
                set_root: Some(root),
                threshold: Some(u64_field(HONEST_THRESHOLD)),
                nullifier: Some(nullifier),
                subject: Some(Fr::from(7u64)),
                tag: Some(tag),
                salt: Some(salt),
                value_bits: Some(u64_bits(HONEST_VALUE)),
                diff_bits: difference_bits(HONEST_VALUE, HONEST_THRESHOLD),
                path: Some(path),
                index_bits: Some(index_bits),
            },
        }
    }

    /// Public inputs in allocation order, as scalars.
    fn public_scalars(circuit: &AttestationCircuit) -> Vec<Fr> {
        vec![
            circuit.set_root.unwrap(),
            circuit.threshold.unwrap(),
            circuit.nullifier.unwrap(),
        ]
    }

    /// The same three, in the byte form the chain reads.
    fn public_inputs(circuit: &AttestationCircuit) -> Vec<[u8; 32]> {
        public_scalars(circuit)
            .into_iter()
            .map(field_to_public_bytes)
            .collect()
    }

    fn prove_fixture(fixture: &Fixture) -> (Vec<u8>, Vec<u8>) {
        // Exercise the public prover API rather than a private reimplementation —
        // if `setup_for_testing`/`prove` drift from the circuit, these tests must
        // be the ones that notice.
        let setup = setup_for_testing(&fixture.circuit).expect("setup");
        let material = prove(&fixture.circuit, &setup).expect("proof");

        let vk = VerifyingKey::<Bls12_381>::deserialize_with_mode(
            &setup.verifying_key[..],
            Compress::Yes,
            Validate::Yes,
        )
        .unwrap();
        let proof =
            Proof::<Bls12_381>::deserialize_with_mode(&material.proof[..], Compress::Yes, Validate::Yes)
                .unwrap();
        assert!(Groth16::<Bls12_381>::verify(&vk, &public_scalars(&fixture.circuit), &proof).unwrap());

        assert_eq!(
            material.inputs,
            public_inputs_of(&fixture.circuit).expect("populated"),
            "the prover must publish the inputs the circuit actually commits to",
        );

        (setup.verifying_key, material.proof)
    }

    #[test]
    fn honest_attestation_verifies() {
        let fixture = fixture();
        let (vk_bytes, proof_bytes) = prove_fixture(&fixture);

        // Confirm the serialized round trip is faithful before trusting it.
        let proof =
            Proof::<Bls12_381>::deserialize_with_mode(&proof_bytes[..], Compress::Yes, Validate::Yes)
                .unwrap();
        let vk = VerifyingKey::<Bls12_381>::deserialize_with_mode(
            &vk_bytes[..],
            Compress::Yes,
            Validate::Yes,
        )
        .unwrap();
        assert!(Groth16::<Bls12_381>::verify(&vk, &public_scalars(&fixture.circuit), &proof).unwrap());

        // And through the exact path the chain uses.
        verify_attestation(&vk_bytes, &proof_bytes, &public_inputs(&fixture.circuit))
            .expect("the on-chain verifier must accept an honest proof");

        let named = crate::verify::AttestationInputs::from_slice(&public_inputs(&fixture.circuit))
            .expect("the public-input array has the expected arity");
        crate::verify::verify_attestation_inputs(&vk_bytes, &proof_bytes, &named)
            .expect("the checked verifier must accept an honest proof");
    }

    #[test]
    fn the_circuit_allocates_exactly_the_documented_arity() {
        // If the circuit ever grows a public input, the verifier's constant and
        // every caller's array must grow with it. Catch that here rather than at
        // the first live proof.
        let fixture = fixture();
        let cs = ConstraintSystem::<Fr>::new_ref();
        fixture
            .circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();
        assert_eq!(cs.num_instance_variables(), PUBLIC_INPUT_COUNT + 1);
        assert_eq!(public_inputs(&fixture.circuit).len(), PUBLIC_INPUT_COUNT);
        assert_eq!(PUBLIC_INPUT_BYTES, PUBLIC_INPUT_COUNT * 32);
    }

    #[test]
    fn raised_threshold_is_rejected() {
        let fixture = fixture();
        let (vk_bytes, proof_bytes) = prove_fixture(&fixture);
        let mut inputs = public_inputs(&fixture.circuit);
        inputs[1] = field_to_public_bytes(u64_field(HONEST_VALUE + 1));

        assert!(verify_attestation(&vk_bytes, &proof_bytes, &inputs).is_err());
    }

    #[test]
    fn foreign_root_is_rejected() {
        let fixture = fixture();
        let (vk_bytes, proof_bytes) = prove_fixture(&fixture);
        let mut inputs = public_inputs(&fixture.circuit);
        inputs[0] = field_to_public_bytes(fixture.root + Fr::from(1u64));

        assert!(verify_attestation(&vk_bytes, &proof_bytes, &inputs).is_err());
    }

    #[test]
    fn forged_nullifier_is_rejected() {
        let fixture = fixture();
        let (vk_bytes, proof_bytes) = prove_fixture(&fixture);
        let mut inputs = public_inputs(&fixture.circuit);
        inputs[2] = field_to_public_bytes(Fr::from(12345u64));

        assert!(verify_attestation(&vk_bytes, &proof_bytes, &inputs).is_err());
    }

    #[test]
    fn a_proof_cannot_be_replayed_against_a_different_registry() {
        let fixture = fixture();
        let (vk_bytes, proof_bytes) = prove_fixture(&fixture);

        // Same record, same value, different registry root.
        let other = SparseTree::new(&fixture.params, [(0u64, Fr::from(555u64))]).unwrap();
        let mut inputs = public_inputs(&fixture.circuit);
        inputs[0] = field_to_public_bytes(other.root(&fixture.params));

        assert!(verify_attestation(&vk_bytes, &proof_bytes, &inputs).is_err());
    }

    #[test]
    fn value_below_threshold_has_no_valid_witness() {
        // The honest prover cannot even assemble a witness: there is no
        // non-negative difference to decompose.
        assert!(difference_bits(999, 1_000).is_none());
        assert!(difference_bits(1_000, 1_000).is_some(), "the threshold is inclusive");
    }

    #[test]
    fn a_lying_difference_fails_the_constraint_system() {
        // Claim `value >= threshold` while supplying a difference that belongs to
        // some other (value, threshold) pair.
        let fixture = fixture();
        let mut lying = fixture.circuit.clone();
        lying.diff_bits = Some(u64_bits(1));

        let cs = ConstraintSystem::<Fr>::new_ref();
        let unsatisfied = lying.generate_constraints(cs.clone());
        assert!(
            unsatisfied.is_err() || !cs.is_satisfied().unwrap(),
            "a wrong difference must not produce a satisfied circuit"
        );
    }

    #[test]
    fn claiming_another_persons_record_fails() {
        // Using the real path and salt but somebody else's subject.
        let fixture = fixture();
        let mut forged = fixture.circuit.clone();
        forged.subject = Some(Fr::from(8u64));

        let cs = ConstraintSystem::<Fr>::new_ref();
        let unsatisfied = forged.generate_constraints(cs.clone());
        assert!(
            unsatisfied.is_err() || !cs.is_satisfied().unwrap(),
            "a mismatched subject must not produce a satisfied circuit"
        );
    }

    #[test]
    fn public_bytes_round_trip_and_reject_non_canonical() {
        for value in [0u64, 1, 2, u32::MAX as u64, 1u64 << 63] {
            let scalar = u64_field(value);
            let bytes = field_to_public_bytes(scalar);
            assert_eq!(public_bytes_to_field(&bytes), Some(scalar));
        }

        // An all-ones encoding is far above the modulus and must be refused.
        assert_eq!(public_bytes_to_field(&[0xff; 32]), None);
    }

    #[test]
    fn bit_helpers_agree_with_u64() {
        for value in [0u64, 1, 255, 256, u64::MAX] {
            let bits = u64_bits(value);
            assert_eq!(bits.len(), VALUE_BITS);
            let rebuilt: u64 = bits
                .iter()
                .enumerate()
                .filter(|(_, bit)| **bit)
                .fold(0u64, |acc, (i, _)| acc | (1u64 << i));
            assert_eq!(rebuilt, value);
        }
    }

    #[test]
    fn circuit_stays_affordable() {
        // Proving cost scales with constraints, and the whole point of choosing
        // Poseidon over BLAKE2 was to keep this in the tens of thousands. A
        // regression to six figures would make proving impractical on citizen
        // hardware, so fail loudly rather than quietly.
        let fixture = fixture();
        let cs = ConstraintSystem::<Fr>::new_ref();
        fixture
            .circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();

        let constraints = cs.num_constraints();
        println!("attestation circuit: {constraints} constraints");
        assert!(
            constraints < 40_000,
            "circuit grew to {constraints} constraints; proving is no longer cheap"
        );
    }
}
