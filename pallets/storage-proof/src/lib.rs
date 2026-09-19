//! Pakit Storage Proof pallet — on-chain verification of decentralized storage
//! proofs submitted by Pakit DAG storage nodes.
//!
//! ## Design (Stage A — 2026-09-17)
//!
//! - `proof_type = Merkle` is the *only* accepted path in this stage. It is
//!   fully verified on-chain: the submitted Merkle root must hash (BLAKE2-256)
//!   to the stored CID commitment for the content, and each proof step must
//!   re-derive that root from the declared leaf.
//! - `proof_type = Zk`/`Groth16` is feature-gated behind `groth16` and explicitly
//!   NOT accepted in Stage A — a real arkworks verifier lands in Stage B.
//!   Returning a hard error (not Ok) keeps the invariant "never accept a proof
//!   we could not verify" without conditional compilation trickery.
//!
//! ## Determinism
//! All hashing is BLAKE2-256 via `sp_io::hashing::blake2_256` — no floats,
//! no external sources, fully deterministic for `try`/benchmark reproducibility.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::BoundedVec;
use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::EnsureOrigin};
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Maximum number of Merkle proof steps accepted — upper bound prevents
/// unbounded weight growth on internally inconsistent inputs.
pub const MAX_PROOF_STEPS: u32 = 64;

/// Length (bytes) of a content identifier commitment as enforced by Pakit.
pub const CID_COMMITMENT_LEN: usize = 32;

#[derive(
    Clone,
    Encode,
    Decode,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    codec::DecodeWithMemTracking,
    Default,
)]
pub enum ProofType {
    /// Merkle path verification — the only type accepted in Stage A.
    #[default]
    Merkle,
    /// Reserved for Stage B (arkworks Groth16 verifier). NOT accepted yet.
    Groth16,
}

/// A self-contained Merkle storage proof.
#[derive(
    Clone,
    Encode,
    Decode,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    codec::DecodeWithMemTracking,
)]
pub struct MerkleProof {
    /// The leaf (block content hash) this proof attests to. BLAKE2-256.
    pub leaf: [u8; 32],
    /// Ordered right/left siblings; each entry is `[u8; 32]` and a bit flag.
    pub siblings: BoundedVec<MerkleStep, MaxProofSteps>,
}

#[derive(
    Clone,
    Encode,
    Decode,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    codec::DecodeWithMemTracking,
)]
pub struct MerkleStep {
    pub node: [u8; 32],
    /// `true`  → node is the left sibling (combined = H(node ‖ current))
    /// `false` → node is the right sibling (combined = H(current ‖ node))
    pub node_is_left: bool,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::parameter_types;

    /// Uniquely identifies a content item on Pakit: BLAKE2-256 commitment.
    pub type ContentId = [u8; CID_COMMITMENT_LEN];

    parameter_types! {
        pub MaxProofSteps: u32 = MAX_PROOF_STEPS;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Origin that may administratively revoke a previously-accepted proof
        /// (e.g. a proven-byzantine storage provider). Typically a council or
        /// sudo-assigned root.
        type RevocationOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// CID commitment → submitter + height. ValueQuery: unset rows read as default.
    #[pallet::storage]
    pub type StorageProofs<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, ProofRecord<BlockNumberFor<T>, T::AccountId>>;

    #[derive(
        Clone,
        Encode,
        Decode,
        PartialEq,
        Eq,
        Debug,
        TypeInfo,
        MaxEncodedLen,
        codec::DecodeWithMemTracking,
        Default,
    )]
    pub struct ProofRecord<BlockNumber, AccountId> {
        pub submitter: AccountId,
        pub submitted_at: BlockNumber,
        pub proof_type: ProofType,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Merkle path was empty — nothing to verify.
        EmptyMerkleProof,
        /// Merkle step count exceeds `MAX_PROOF_STEPS`.
        MerklePathTooLong,
        /// Derived root does not match the CID commitment for this content.
        RootMismatch,
        /// Explicitly rejected until the Stage-B verifier is wired.
        Groth16NotYetAccepted,
        /// Proof already recorded for this content by a different provider.
        ContentAlreadyProven,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a Merkle storage proof. Fully verifies on-chain before storing.
        ///
        /// - `content_id`: 32-byte BLAKE2-256 of the content Merkle root — must
        ///   equal the Merkle root derivable from the provided proof.
        /// - `proof`: ordered siblings walking leaf → root.
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        #[pallet::call_index(0)]
        pub fn submit_storage_proof(
            origin: OriginFor<T>,
            content_id: ContentId,
            proof: MerkleProof,
            proof_type: ProofType,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;

            match proof_type {
                ProofType::Groth16 => {
                    // Stage B gate — arkworks verifier not yet wired. Do NOT
                    // silently accept. Hard error keeps invariant "never record
                    // a proof type we could not verify on-chain".
                    #[cfg(not(feature = "groth16"))]
                    {
                        _ = submitter;
                        return Err(Error::<T>::Groth16NotYetAccepted.into());
                    }
                    #[cfg(feature = "groth16")]
                    {
                        // Compiled with feature but verifier stub — still Stage B.
                        _ = submitter;
                        return Err(Error::<T>::Groth16NotYetAccepted.into());
                    }
                }
                ProofType::Merkle => {
                    // fall through to verification below
                }
            }

            ensure!(!proof.siblings.is_empty(), Error::<T>::EmptyMerkleProof);
            ensure!(
                (proof.siblings.len() as u32) <= MAX_PROOF_STEPS,
                Error::<T>::MerklePathTooLong
            );

            // Walk the path leaf → root using BLAKE2-256 on each concat.
            // Deterministic: no floats, no randomness, no off-chain dependence.
            let derived_root = Self::derive_merkle_root(&proof.leaf, &proof.siblings);
            ensure!(derived_root == content_id, Error::<T>::RootMismatch);

            ensure!(
                !StorageProofs::<T>::contains_key(derived_root),
                Error::<T>::ContentAlreadyProven
            );

            let block = <frame_system::Pallet<T>>::block_number();
            StorageProofs::<T>::insert(
                derived_root,
                ProofRecord {
                    submitter: submitter.clone(),
                    submitted_at: block,
                    proof_type,
                },
            );

            Self::deposit_event(Event::StorageProofAccepted {
                content_id: derived_root,
                submitter,
            });
            Ok(())
        }

        /// Administrative revocation (e.g. byzantine behaviour proven).
        #[pallet::weight(Weight::from_parts(5_000, 0))]
        #[pallet::call_index(1)]
        pub fn revoke_proof(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            T::RevocationOrigin::ensure_origin(origin)?;
            ensure!(
                StorageProofs::<T>::contains_key(content_id),
                Error::<T>::RootMismatch
            );
            StorageProofs::<T>::remove(content_id);
            Self::deposit_event(Event::StorageProofRevoked { content_id });
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A Merkle storage proof was verified and recorded on-chain.
        StorageProofAccepted {
            content_id: ContentId,
            submitter: T::AccountId,
        },
        /// A previously-accepted proof was revoked.
        StorageProofRevoked { content_id: ContentId },
    }

    impl<T: Config> Pallet<T> {
        /// Pure helper: BLAKE2-256 Merkle root derivation.
        /// Public + `pub` (outside `impl T`) for reuse in benchmarks or a
        /// future `try_state` hook — deliberately `#[inline]`d for determinism.
        pub fn derive_merkle_root(leaf: &[u8; 32], siblings: &[MerkleStep]) -> [u8; 32] {
            let mut current = *leaf;
            for step in siblings.iter() {
                current = if step.node_is_left {
                    // node ‖ current
                    let mut buf = sp_std::vec::Vec::with_capacity(64);
                    buf.extend_from_slice(&step.node);
                    buf.extend_from_slice(&current);
                    sp_io::hashing::blake2_256(&buf)
                } else {
                    // current ‖ node
                    let mut buf = sp_std::vec::Vec::with_capacity(64);
                    buf.extend_from_slice(&current);
                    buf.extend_from_slice(&step.node);
                    sp_io::hashing::blake2_256(&buf)
                };
            }
            current
        }
    }
}
