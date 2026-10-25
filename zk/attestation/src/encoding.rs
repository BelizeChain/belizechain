//! Canonical field-element encoding.
//!
//! Every value that crosses the trust boundary — roots, thresholds, nullifiers —
//! travels as 32 bytes and must be decoded **canonically**.
//!
//! This is not pedantry. `Fr::from_le_bytes_mod_order` silently reduces: `r + 1`
//! and `1` decode to the same element. If a nullifier were keyed on the raw
//! bytes, one witness could be re-submitted under a different byte encoding and
//! pass the replay check while proving the same statement. Requiring the bytes
//! to be the unique canonical encoding of the element removes the freedom.

use ark_bls12_381::Fr;
use ark_ff::{BigInteger, PrimeField};

/// `r` for BLS12-381's scalar field, little-endian.
///
/// `0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001`, reversed.
pub const FR_MODULUS_LE: [u8; 32] = [
    0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x5b, 0xfe, 0xff, 0x02, 0xa4, 0xbd, 0x53,
    0x05, 0xd8, 0xa1, 0x09, 0x08, 0xd8, 0x39, 0x33, 0x48, 0x7d, 0x9d, 0x29, 0x53, 0xa7, 0xed, 0x73,
];

/// True when `bytes` is the canonical little-endian encoding of a scalar.
///
/// Compares from the most significant byte down; the first difference decides.
pub fn is_canonical(bytes: &[u8; 32]) -> bool {
    for index in (0..32).rev() {
        if bytes[index] < FR_MODULUS_LE[index] {
            return true;
        }
        if bytes[index] > FR_MODULUS_LE[index] {
            return false;
        }
    }
    // Exactly equal to the modulus is not a valid element: it reduces to zero,
    // and zero has its own distinct encoding.
    false
}

/// Decode 32 little-endian bytes into a scalar, rejecting non-canonical input.
pub fn canonical_field(bytes: &[u8; 32]) -> Option<Fr> {
    if !is_canonical(bytes) {
        return None;
    }
    Some(Fr::from_le_bytes_mod_order(bytes))
}

/// Encode a scalar to its canonical 32-byte little-endian form.
///
/// The output is always accepted by [`canonical_field`]: this is the only
/// encoding either side may produce.
pub fn field_to_bytes(value: Fr) -> [u8; 32] {
    let mut out = [0u8; 32];
    let raw = value.into_bigint().to_bytes_le();
    out[..raw.len()].copy_from_slice(&raw);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::One;

    #[test]
    fn zero_is_canonical_and_round_trips() {
        let zero = [0u8; 32];
        assert!(is_canonical(&zero));
        assert_eq!(canonical_field(&zero), Some(Fr::from(0u64)));
        assert_eq!(field_to_bytes(Fr::from(0u64)), zero);
    }

    #[test]
    fn one_round_trips() {
        let one = field_to_bytes(Fr::one());
        assert_eq!(one[0], 1);
        assert_eq!(canonical_field(&one), Some(Fr::one()));
    }

    #[test]
    fn the_modulus_itself_is_rejected() {
        // Encodes the same element as zero, so accepting it would give one value
        // two encodings — exactly the malleability this module exists to stop.
        assert!(!is_canonical(&FR_MODULUS_LE));
        assert_eq!(canonical_field(&FR_MODULUS_LE), None);
    }

    #[test]
    fn modulus_minus_one_is_canonical() {
        let mut minus_one = FR_MODULUS_LE;
        minus_one[0] -= 1;
        assert!(is_canonical(&minus_one));
        let value = canonical_field(&minus_one).expect("valid");
        assert_eq!(field_to_bytes(value), minus_one);
    }

    #[test]
    fn a_value_above_the_modulus_is_rejected() {
        let mut above = [0u8; 32];
        above[31] = 0x74; // greater than the top byte of r
        assert!(!is_canonical(&above));
        assert_eq!(canonical_field(&above), None);
    }

    #[test]
    fn all_ones_is_rejected() {
        // 2^256 - 1 is far above r; reducing it silently would be a bypass.
        assert!(!is_canonical(&[0xff; 32]));
    }

    #[test]
    fn arbitrary_elements_round_trip() {
        for seed in 0u64..64 {
            let value = Fr::from(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).max(1));
            let bytes = field_to_bytes(value);
            assert!(is_canonical(&bytes), "encoded output must always be canonical");
            assert_eq!(canonical_field(&bytes), Some(value));
        }
    }
}
