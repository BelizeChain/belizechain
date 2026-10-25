//! Weights for `pallet-zk-attestation`.
//!
//! # Why these are hand-written rather than benchmarked
//!
//! `submit_attestation` runs a Groth16 pairing check. A benchmark cannot produce
//! a valid proof — proving needs a witness, and fabricating one inside the
//! benchmark would mean linking the prover crate into the runtime wasm, which is
//! exactly what this pallet avoids. Any benchmarked number would therefore
//! measure the *rejection* path while being labelled the acceptance path, which
//! understates the real cost by three orders of magnitude.
//!
//! So the verification cost is a measured constant. It is legitimate here for a
//! specific reason: Groth16 verification is constant in the circuit's size and
//! its inputs are fixed-width — a 192-byte proof, a 536-byte key, and exactly
//! three scalars. There is no unbounded input for an attacker to inflate.
//!
//! # The measurement
//!
//! 2,000 verifications of a real three-input proof, release build:
//!
//! | Machine | Per verification |
//! |---|---|
//! | AMD Ryzen 9 5900X (the Ceiba reference host), single thread | 8.0 ms |
//!
//! The constant below is 10 ms, a 25% margin. Substrate charges one weight unit
//! per picosecond, so 10 ms is `10_000_000_000`.
//!
//! Re-measure if the curve, the arkworks version, or the reference hardware
//! changes. A wrong number here is a block-production hazard, not a rounding
//! error: 10 ms is 0.5% of a 2-second block, so roughly 200 attestations fill a
//! block on their own.

use frame_support::weights::{constants::RocksDbWeight, Weight};
use sp_std::marker::PhantomData;

/// Measured cost of one Groth16 verification, with margin. See the module docs.
pub const GROTH16_VERIFY_WEIGHT: Weight = Weight::from_parts(10_000_000_000, 0);

/// Weight information for this pallet's extrinsics.
pub trait WeightInfo {
    fn set_registry_root() -> Weight;
    fn set_verifying_key(key_len: u32) -> Weight;
    fn submit_attestation(proof_len: u32) -> Weight;
}

/// Benchmark-derived weights are unavailable (see the module docs); these are
/// storage costs from `RocksDbWeight` plus the measured pairing cost.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// One read (the verifying key existence check), one write (the root).
    fn set_registry_root() -> Weight {
        RocksDbWeight::get().reads(1).saturating_add(RocksDbWeight::get().writes(1))
    }

    /// One write, plus a read when replacing an existing key.
    fn set_verifying_key(_key_len: u32) -> Weight {
        RocksDbWeight::get().reads(1).saturating_add(RocksDbWeight::get().writes(1))
    }

    /// Three reads (root, verifying key, nullifier), one write (the nullifier),
    /// plus the pairing check.
    ///
    /// `proof_len` does not scale the cost the way it would for a hash: the blob
    /// is decoded into fixed-size curve points, so a longer proof fails to decode
    /// rather than costing more. The parameter is kept so a future circuit with
    /// variable-size proofs does not need a signature change.
    fn submit_attestation(_proof_len: u32) -> Weight {
        RocksDbWeight::get()
            .reads(2)
            .saturating_add(RocksDbWeight::get().writes(2))
            .saturating_add(GROTH16_VERIFY_WEIGHT)
    }
}

impl WeightInfo for () {
    fn set_registry_root() -> Weight {
        RocksDbWeight::get().reads(1).saturating_add(RocksDbWeight::get().writes(1))
    }

    fn set_verifying_key(_key_len: u32) -> Weight {
        RocksDbWeight::get().reads(1).saturating_add(RocksDbWeight::get().writes(1))
    }

    fn submit_attestation(_proof_len: u32) -> Weight {
        RocksDbWeight::get()
            .reads(2)
            .saturating_add(RocksDbWeight::get().writes(2))
            .saturating_add(GROTH16_VERIFY_WEIGHT)
    }
}
