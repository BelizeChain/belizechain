//! The Groth16 verifier — the only part of this crate the chain executes.
//!
//! Verification is a pairing check, so cost is constant in the circuit's size.
//! That property is the whole reason the predicate can be elaborate: a heavier
//! statement costs the prover, not the block.

use alloc::vec::Vec;

use ark_bls12_381::Bls12_381;
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, Compress, Validate};

use crate::encoding::canonical_field;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerifyError {
    /// The verifying key could not be decoded, or is not a valid curve element.
    BadVerifyingKey,
    /// The proof could not be decoded, or is not a valid curve element.
    BadProof,
    /// A public input was not the canonical encoding of a scalar.
    NonCanonicalInput,
    /// Public input count does not match the key — the caller and the circuit
    /// disagree about the statement, which is a bug, not a forgery.
    InputCountMismatch,
    /// The threshold did not fit in 64 bits. See [`verify_attestation_inputs`].
    UnsupportedThreshold,
    /// The pairing check failed: the proof is well-formed but false.
    ProofRejected,
}

/// Bytes in one canonically encoded scalar.
pub const FIELD_BYTES: usize = 32;

/// Public inputs an attestation proof carries, in circuit order.
pub const PUBLIC_INPUT_COUNT: usize = 3;

/// Encoded width of the whole public-input array.
pub const PUBLIC_INPUT_BYTES: usize = PUBLIC_INPUT_COUNT * FIELD_BYTES;

/// The three values an attestation proof publishes, named so a caller cannot
/// silently transpose them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestationInputs {
    /// Registry root the record must be committed under.
    pub set_root: [u8; FIELD_BYTES],
    /// Inclusive minimum value, as a canonical scalar below `2^64`.
    pub threshold: [u8; FIELD_BYTES],
    /// Anti-replay tag.
    pub nullifier: [u8; FIELD_BYTES],
}

impl AttestationInputs {
    /// The array form, in circuit order.
    pub fn to_array(&self) -> [[u8; FIELD_BYTES]; PUBLIC_INPUT_COUNT] {
        [self.set_root, self.threshold, self.nullifier]
    }

    /// Read the array form, in circuit order.
    pub fn from_slice(inputs: &[[u8; FIELD_BYTES]]) -> Option<Self> {
        if inputs.len() != PUBLIC_INPUT_COUNT {
            return None;
        }
        Some(Self {
            set_root: inputs[0],
            threshold: inputs[1],
            nullifier: inputs[2],
        })
    }

    /// True when the threshold is a canonical scalar that fits in 64 bits.
    pub fn threshold_supported(&self) -> bool {
        canonical_field(&self.threshold).is_some() && self.threshold[8..].iter().all(|b| *b == 0)
    }
}

/// Verify an attestation proof against named inputs, enforcing the threshold
/// range check.
///
/// # Why the range check is not optional
///
/// The circuit proves `value - threshold ≡ diff (mod r)` with `diff` decomposed
/// into 64 bits, and `value` likewise. It does **not** range-check `threshold`,
/// because that is a public input and checking it costs nothing outside the
/// circuit.
///
/// Skipping the check breaks soundness. Take `threshold = r - 1`. Then
/// `value - threshold ≡ value + 1 (mod r)`, which is a small positive number, so
/// a prover with `value = 5` can satisfy the circuit for a threshold near the
/// modulus. Requiring `threshold < 2^64` removes that: with all three terms below
/// `2^64`, the field equation can only hold when the integer equation does.
///
/// Use this function rather than [`verify_attestation`] unless you have your own
/// equivalent check.
pub fn verify_attestation_inputs(
    vk_bytes: &[u8],
    proof_bytes: &[u8],
    inputs: &AttestationInputs,
) -> Result<(), VerifyError> {
    if !inputs.threshold_supported() {
        return Err(VerifyError::UnsupportedThreshold);
    }
    verify_attestation(vk_bytes, proof_bytes, &inputs.to_array())
}

/// Verify a Groth16 proof of a registry attestation.
///
/// `public_inputs` are canonical 32-byte little-endian scalars, in the order the
/// circuit declares them: `[set_root, threshold, nullifier]`.
///
/// Both halves are decoded with `Validate::Yes`, which rejects off-curve points
/// and non-canonical field elements. `*_unchecked` would skip those checks and is
/// precisely how a forged proof gets accepted; there is no reason to use it here
/// and this crate never does.
pub fn verify_attestation(
    vk_bytes: &[u8],
    proof_bytes: &[u8],
    public_inputs: &[[u8; 32]],
) -> Result<(), VerifyError> {
    let vk =
        VerifyingKey::<Bls12_381>::deserialize_with_mode(vk_bytes, Compress::Yes, Validate::Yes)
            .map_err(|_| VerifyError::BadVerifyingKey)?;
    let proof =
        Proof::<Bls12_381>::deserialize_with_mode(proof_bytes, Compress::Yes, Validate::Yes)
            .map_err(|_| VerifyError::BadProof)?;

    // `gamma_abc_g1` has one entry per public input plus the constant term.
    let expected = vk.gamma_abc_g1.len().saturating_sub(1);
    if public_inputs.len() != expected {
        return Err(VerifyError::InputCountMismatch);
    }

    let mut inputs: Vec<_> = Vec::with_capacity(public_inputs.len());
    for raw in public_inputs {
        inputs.push(canonical_field(raw).ok_or(VerifyError::NonCanonicalInput)?);
    }

    match Groth16::<Bls12_381>::verify_proof(
        &ark_groth16::prepare_verifying_key(&vk),
        &proof,
        &inputs,
    ) {
        Ok(true) => Ok(()),
        // `false` is a well-formed but false statement. An `Err` here means the
        // arithmetic could not even be evaluated; from a caller's point of view
        // both are "not verified", and collapsing them avoids leaking which.
        Ok(false) | Err(_) => Err(VerifyError::ProofRejected),
    }
}

/// Number of public inputs a verifying key expects.
///
/// Lets the pallet reject an arity mismatch before paying for a decode of the
/// proof, and lets the prover CLI check its own output.
pub fn expected_input_count(vk_bytes: &[u8]) -> Option<usize> {
    let vk =
        VerifyingKey::<Bls12_381>::deserialize_with_mode(vk_bytes, Compress::Yes, Validate::Yes)
            .ok()?;
    Some(vk.gamma_abc_g1.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_truncated_verifying_key_is_refused() {
        assert_eq!(
            verify_attestation(&[0u8; 4], &[0u8; 192], &[]),
            Err(VerifyError::BadVerifyingKey)
        );
    }

    #[test]
    fn an_empty_verifying_key_is_refused() {
        assert_eq!(
            verify_attestation(&[], &[], &[]),
            Err(VerifyError::BadVerifyingKey)
        );
    }

    #[test]
    fn an_unknown_key_has_no_arity() {
        assert_eq!(expected_input_count(&[0u8; 4]), None);
        assert_eq!(expected_input_count(&[]), None);
    }

    #[test]
    fn a_threshold_near_the_modulus_is_refused() {
        // `r - 1` is a perfectly canonical field element, and accepting it as a
        // threshold is exactly the soundness hole the range check exists to
        // close. Refuse before the pairing check, not after.
        let mut near_modulus = [0u8; FIELD_BYTES];
        near_modulus.copy_from_slice(&crate::encoding::FR_MODULUS_LE);
        near_modulus[0] -= 1;

        let inputs = AttestationInputs {
            set_root: [1u8; FIELD_BYTES],
            threshold: near_modulus,
            nullifier: [2u8; FIELD_BYTES],
        };
        assert!(!inputs.threshold_supported());
        assert_eq!(
            verify_attestation_inputs(&[], &[], &inputs),
            Err(VerifyError::UnsupportedThreshold),
        );
    }

    #[test]
    fn a_sixty_four_bit_threshold_is_accepted() {
        let mut threshold = [0u8; FIELD_BYTES];
        threshold[..8].copy_from_slice(&5_000u64.to_le_bytes());
        let inputs = AttestationInputs {
            set_root: [1u8; FIELD_BYTES],
            threshold,
            nullifier: [2u8; FIELD_BYTES],
        };
        assert!(inputs.threshold_supported());

        // The threshold is fine, so the failure is the (absent) key instead.
        assert_eq!(
            verify_attestation_inputs(&[], &[], &inputs),
            Err(VerifyError::BadVerifyingKey),
        );
    }

    #[test]
    fn inputs_round_trip_through_the_array_form() {
        let inputs = AttestationInputs {
            set_root: [3u8; FIELD_BYTES],
            threshold: [0u8; FIELD_BYTES],
            nullifier: [9u8; FIELD_BYTES],
        };
        assert_eq!(
            AttestationInputs::from_slice(&inputs.to_array()),
            Some(inputs)
        );
        assert_eq!(AttestationInputs::from_slice(&[[0u8; 32]; 2]), None);
    }
}
