//! Privacy-preserving registry attestations.
//!
//! # What this does
//!
//! Proves that a record is committed in a published registry root **and**
//! satisfies a predicate, without revealing the record. A ministry publishes a
//! root over its registry; a citizen proves "my record is in there and the value
//! is at least N" and discloses neither.
//!
//! Merkle proofs cannot do this. A Merkle proof requires handing over the leaf,
//! so proving inclusion reveals the record by construction.
//!
//! # Why Poseidon here and BLAKE2 elsewhere
//!
//! `ContentId` on this chain is `blake2_256`, and it must stay that way: pakit
//! derives it with `hashlib.blake2b(digest_size=32)`, which is byte-identical to
//! Substrate's `blake2_256`, and every stored record and IPFS relationship is
//! keyed on it.
//!
//! Inside a circuit, though, BLAKE2 costs roughly 2^15 constraints per hash. A
//! depth-32 path is ~1M constraints — minutes of proving and gigabytes of
//! memory. Poseidon costs a few hundred constraints per hash, so the same path
//! is ~15K. The two hashes therefore do different jobs:
//!
//! | Purpose | Hash | Why |
//! |---|---|---|
//! | Content addressing (`ContentId`) | BLAKE2-256 | Off-circuit; changing it breaks pakit and every record |
//! | ZK commitment tree | Poseidon | Cheap in-circuit |
//!
//! # Security properties, stated plainly
//!
//! **This layer is not post-quantum.** Groth16 over BLS12-381 relies on discrete
//! log. It sits *beside* the chain's ML-DSA-87 machinery, not inside it. A
//! quantum adversary could forge an attestation; it could not forge an identity
//! or a bridge delivery. Deployments that must resist that are limited to
//! Merkle proofs, which are symmetric and therefore PQ.
//!
//! **The prover must use this crate.** Poseidon parameters are
//! implementation-specific: Arkworks' sponge construction is not the same as
//! circom's `Poseidon`, even over the same field. A circom prover will produce
//! proofs this verifier rejects, and nothing about the failure will say why.
//!
//! # Layout
//!
//! - [`encoding`] — canonical field-element bytes, used by both sides
//! - [`verify`] — the Groth16 pairing check; this is all the chain runs
//! - `tree` / `circuit` — prover-side only, behind the `prover` feature

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

extern crate alloc;

pub mod encoding;
pub mod verify;

#[cfg(feature = "prover")]
pub mod circuit;
#[cfg(feature = "prover")]
pub mod tree;

pub use encoding::canonical_field;
pub use verify::{
    verify_attestation, verify_attestation_inputs, AttestationInputs, VerifyError, FIELD_BYTES,
    PUBLIC_INPUT_BYTES, PUBLIC_INPUT_COUNT,
};

#[cfg(feature = "prover")]
pub use circuit::{
    difference_bits, field_to_public_bytes, prove, public_bytes_to_field, public_inputs_of,
    setup_for_testing, u64_bits, u64_field, AttestationCircuit, AttestationMaterial, ProverError,
    Setup, VALUE_BITS,
};
#[cfg(feature = "prover")]
pub use tree::{leaf_hash, poseidon_config, record_hash, root_from_path, SparseTree, TREE_DEPTH};

/// The scalar field, so callers need not depend on `ark-bls12-381` directly and
/// cannot pick a mismatched version.
#[cfg(feature = "prover")]
pub use ark_bls12_381::Fr;
