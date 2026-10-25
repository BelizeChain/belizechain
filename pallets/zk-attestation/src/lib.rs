//! ZK Attestation pallet — verify privacy-preserving registry proofs on chain.
//!
//! # The problem this solves
//!
//! A ministry holds a registry: incomes, land titles, licences, whatever. A
//! citizen needs to prove *"my record is in that registry and meets this
//! condition"* — to claim a benefit, to satisfy a threshold, to qualify — without
//! handing over the record itself.
//!
//! Merkle proofs cannot do this. Verifying a Merkle path requires the leaf, so
//! the act of proving discloses the data. The chain also cannot be trusted with
//! the leaf: publishing it would put every citizen's income on a public ledger.
//!
//! So the proof is a succinct argument over a private witness. Off chain, a
//! ministry builds a Poseidon commitment tree over its registry and publishes the
//! root through governance. A citizen generates a Groth16 proof that their leaf is
//! under that root and that the committed value clears a threshold. The chain
//! runs a pairing check — constant cost, no registry, no witness, no leaf.
//!
//! # What the chain does and does not touch
//!
//! | Thing | Where it lives | Chain sees it |
//! |---|---|---|
//! | Registry contents | Ministry, off chain | never |
//! | Commitment tree | Ministry, off chain | never |
//! | Poseidon parameters | Prover crate | never — the chain only does pairings |
//! | Registry root | `RegistryRoots` | yes, published by governance |
//! | Verifying key | `VerifyingKeys` | yes, published by governance |
//! | Proof | Extrinsic argument | yes, but reveals nothing |
//! | Nullifier | `UsedNullifiers` | yes, an opaque anti-replay tag |
//!
//! That split is why this pallet links only `belizechain-zk-attestation`'s
//! verifier. There is no Poseidon, no constraint system and no prover code
//! anywhere in the runtime wasm.
//!
//! # What this does NOT hide
//!
//! Submitting is a signed extrinsic, so an observer can see *which account*
//! submitted a proof against *which registry*. What stays hidden is the record:
//! subject, value, tag, and which leaf. Anyone promising more than that is
//! promising something this design does not deliver. Attesting through a fresh
//! account narrows the linkage; it does not eliminate it.
//!
//! # Not post-quantum
//!
//! Groth16 over BLS12-381 rests on discrete log. A quantum adversary could forge
//! an attestation. It could not forge an identity or a bridge delivery — those
//! use ML-DSA-87. Deployments that must survive that adversary have to stay with
//! Merkle proofs, which are symmetric. Pretending otherwise would be the more
//! dangerous choice, so the limitation is stated in the type system's terms:
//! [`belizechain_zk_attestation::verify_attestation_inputs`] is the only
//! verification entry point and its docs say the same.
//!
//! # Nullifier scope
//!
//! Nullifiers are keyed by `(set_id, nullifier)`, not globally. The same person
//! legitimately attesting to two different registries must not be blocked by the
//! first, and a registry cannot poison another's set.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use belizechain_zk_attestation::{
    verify_attestation_inputs, AttestationInputs, FIELD_BYTES,
};

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Encoded length of a Groth16 proof over BLS12-381 with compressed points.
///
/// Measured at 192 bytes. The bound is generous because the real limit is
/// enforced by the decoder — a longer blob simply fails to deserialize — and a
/// tight bound would need a storage migration if the curve or encoding ever
/// changed.
pub const MAX_PROOF_LEN: u32 = 256;

/// Encoded length of a verifying key.
///
/// Measured at 536 bytes for a three-input circuit. Same reasoning as
/// [`MAX_PROOF_LEN`].
pub const MAX_VERIFYING_KEY_LEN: u32 = 1024;

/// A published registry commitment.
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
pub struct RegistryRoot<BlockNumber> {
    /// Poseidon root, as a canonical little-endian scalar.
    pub root: [u8; FIELD_BYTES],
    /// Which verifying key governs proofs against this root. Bumping it is how a
    /// registry migrates to a new circuit without invalidating old proofs
    /// retroactively.
    pub circuit_version: u16,
    /// Number of records committed, for operator sanity checks only. The chain
    /// cannot verify it and never trusts it.
    pub leaf_count: u32,
    /// Block the root was published in.
    pub published_at: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Storage layout version.
    ///
    /// Stays at 0 — the same value `pallet-storage-proof` carried when it was
    /// introduced. Version 0 means "no migration has ever been applied", which is
    /// exactly true here: the pallet is new, every map starts empty on the chain
    /// that gains it, and there is no pre-existing encoding to convert. Declaring
    /// 1 without registering a migration would break the runtime's
    /// `every_belize_pallet_declares_matching_storage_version` invariant, and it
    /// would be a lie about a migration that never happened.
    ///
    /// Bump this only when a real migration lands and is registered in
    /// `runtime/src/migrations.rs`.
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Origin allowed to publish registry roots and verifying keys — a
        /// ministry, a council, or a collective. Never `EnsureSigned`, because a
        /// root is a statement of fact about a whole registry.
        type RegistryAuthority: EnsureOrigin<Self::RuntimeOrigin>;

        /// Maximum proof length accepted, in bytes.
        #[pallet::constant]
        type MaxProofLen: Get<u32>;

        /// Maximum verifying-key length accepted, in bytes.
        #[pallet::constant]
        type MaxVerifyingKeyLen: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Registry roots, keyed by set id.
    ///
    /// Absence means "no such registry". An entry is never removed, only replaced
    /// — a citizen proving against a superseded root must not have their proof
    /// silently become valid or invalid based on timing, and old roots are cheap
    /// to keep.
    #[pallet::storage]
    pub type RegistryRoots<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, RegistryRoot<BlockNumberFor<T>>, OptionQuery>;

    /// Verifying keys, keyed by circuit version.
    #[pallet::storage]
    pub type VerifyingKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, u16, BoundedVec<u8, T::MaxVerifyingKeyLen>, OptionQuery>;

    /// Nullifiers already consumed, per registry set.
    ///
    /// This is the whole anti-replay mechanism: a nullifier is deterministic in
    /// the record's salt, which the leaf commits to, so a prover cannot mint a
    /// fresh one. The pallet does not need to know what the record is to know it
    /// has been used.
    #[pallet::storage]
    pub type UsedNullifiers<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, [u8; FIELD_BYTES], (), OptionQuery>;

    /// Total attestations accepted per registry set. Observability only — never
    /// consulted for authorisation.
    #[pallet::storage]
    pub type AttestationCount<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A registry root was published or replaced.
        RegistryRootSet {
            set_id: u32,
            root: [u8; FIELD_BYTES],
            circuit_version: u16,
            leaf_count: u32,
        },
        /// A verifying key was published or replaced.
        VerifyingKeySet { circuit_version: u16 },
        /// A proof was accepted.
        ///
        /// `nullifier` is the only durable handle a downstream pallet needs; the
        /// proof itself reveals nothing and is not retained. Note that this event
        /// does link the submitting account to the registry — see the module docs.
        AttestationAccepted {
            set_id: u32,
            nullifier: [u8; FIELD_BYTES],
            whose: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No root has been published for this set id.
        UnknownRegistry,
        /// No verifying key has been published for the version this root names.
        UnknownCircuitVersion,
        /// The proof bytes exceeded `MaxProofLen`.
        ProofTooLong,
        /// The verifying key exceeded `MaxVerifyingKeyLen`.
        VerifyingKeyTooLong,
        /// The public inputs were not exactly `[set_root, threshold, nullifier]`.
        WrongPublicInputCount,
        /// The threshold was not a canonical scalar below `2^64`.
        ///
        /// Accepting a larger one would let a prover satisfy the comparison with
        /// a threshold near the field modulus. See
        /// [`belizechain_zk_attestation::verify_attestation_inputs`].
        UnsupportedThreshold,
        /// The proof did not verify.
        ProofRejected,
        /// This nullifier has already been consumed for this set.
        AlreadyAttested,
        /// The registry root exceeded the field modulus, or was otherwise not a
        /// canonical scalar.
        NonCanonicalRoot,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Publish or replace the commitment root for a registry set.
        ///
        /// The pallet cannot check that the root is the root of anything — it has
        /// no registry and no tree. What it can check is that the root is a
        /// canonical scalar and that a verifying key exists for the circuit
        /// version, so a root is never published pointing at a key that does not
        /// exist.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_registry_root())]
        pub fn set_registry_root(
            origin: OriginFor<T>,
            set_id: u32,
            root: [u8; FIELD_BYTES],
            circuit_version: u16,
            leaf_count: u32,
        ) -> DispatchResult {
            T::RegistryAuthority::ensure_origin(origin)?;

            ensure!(
                belizechain_zk_attestation::canonical_field(&root).is_some(),
                Error::<T>::NonCanonicalRoot
            );
            ensure!(
                VerifyingKeys::<T>::contains_key(circuit_version),
                Error::<T>::UnknownCircuitVersion
            );

            RegistryRoots::<T>::insert(
                set_id,
                RegistryRoot {
                    root,
                    circuit_version,
                    leaf_count,
                    published_at: frame_system::Pallet::<T>::block_number(),
                },
            );

            Self::deposit_event(Event::RegistryRootSet {
                set_id,
                root,
                circuit_version,
                leaf_count,
            });
            Ok(())
        }

        /// Publish or replace the verifying key for a circuit version.
        ///
        /// Uploading a wrong key is not a safety problem — every proof against it
        /// fails, which is loud. It is an availability problem, so a version
        /// should be exercised on a testnet before the root that names it goes
        /// live.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_verifying_key(key.len() as u32))]
        pub fn set_verifying_key(
            origin: OriginFor<T>,
            circuit_version: u16,
            key: Vec<u8>,
        ) -> DispatchResult {
            T::RegistryAuthority::ensure_origin(origin)?;

            let key: BoundedVec<u8, T::MaxVerifyingKeyLen> =
                key.try_into().map_err(|_| Error::<T>::VerifyingKeyTooLong)?;

            VerifyingKeys::<T>::insert(circuit_version, key);
            Self::deposit_event(Event::VerifyingKeySet { circuit_version });
            Ok(())
        }

        /// Submit a proof that a record in `set_id` exists and clears a threshold.
        ///
        /// The public inputs are supplied by the caller but are bound to the
        /// proof: altering any of them makes the pairing check fail. `set_root`
        /// and `threshold` are additionally checked against chain state and the
        /// supported range, because a proof over the *right* record with a
        /// *wrong* threshold would otherwise be a valid proof of the wrong
        /// statement.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::submit_attestation(proof.len() as u32))]
        pub fn submit_attestation(
            origin: OriginFor<T>,
            set_id: u32,
            proof: Vec<u8>,
            public_inputs: Vec<[u8; FIELD_BYTES]>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Cap the blob before anything touches it. The verifier would reject an
            // oversized proof anyway, but only after paying to decode it, and weight
            // is charged per call rather than per byte — so without this an attacker
            // gets megabytes of decoding for the price of one extrinsic.
            ensure!(
                proof.len() <= T::MaxProofLen::get() as usize,
                Error::<T>::ProofTooLong
            );

            let registry = RegistryRoots::<T>::get(set_id).ok_or(Error::<T>::UnknownRegistry)?;
            let verifying_key = VerifyingKeys::<T>::get(registry.circuit_version)
                .ok_or(Error::<T>::UnknownCircuitVersion)?;

            let inputs = AttestationInputs::from_slice(&public_inputs)
                .ok_or(Error::<T>::WrongPublicInputCount)?;

            // The prover must be proving against the registry this pallet holds,
            // not against one of their own choosing.
            ensure!(
                inputs.set_root == registry.root,
                Error::<T>::ProofRejected
            );
            ensure!(
                inputs.threshold_supported(),
                Error::<T>::UnsupportedThreshold
            );

            // Cheap replay check before the pairing.
            ensure!(
                !UsedNullifiers::<T>::contains_key(set_id, inputs.nullifier),
                Error::<T>::AlreadyAttested
            );

            // The order of the two checks above is deliberate: `contains_key` is a
            // read, the pairing is milliseconds of CPU. Rejecting replays first
            // means a replayed proof costs the chain almost nothing.
            verify_attestation_inputs(&verifying_key, &proof, &inputs)
                .map_err(|_| Error::<T>::ProofRejected)?;

            UsedNullifiers::<T>::insert(set_id, inputs.nullifier, ());
            AttestationCount::<T>::mutate(set_id, |count| *count = count.saturating_add(1));

            Self::deposit_event(Event::AttestationAccepted {
                set_id,
                nullifier: inputs.nullifier,
                whose: who,
            });
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Whether `nullifier` has been consumed for `set_id`.
    ///
    /// The read a downstream pallet uses to authorise a claim. Reading this is
    /// free of privacy implications: the caller learns only what is already
    /// public.
    pub fn is_nullifier_used(set_id: u32, nullifier: &[u8; FIELD_BYTES]) -> bool {
        UsedNullifiers::<T>::contains_key(set_id, nullifier)
    }

    /// The root currently published for `set_id`.
    pub fn current_root(set_id: u32) -> Option<[u8; FIELD_BYTES]> {
        RegistryRoots::<T>::get(set_id).map(|registry| registry.root)
    }
}
