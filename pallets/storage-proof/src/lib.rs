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
//!
//! ## Storage commitments (2026-10-12)
//!
//! `content_id` and the Merkle `proof` both arrive from the caller, so on their
//! own the verification above establishes only that the caller built a
//! self-consistent tree. It was therefore never safe to *pay* for a proof:
//! minting against a caller-declared root is free money.
//!
//! Payment is instead funded by the party that wants the data stored. A client
//! opens a [`StorageCommitment`] with their own escrow via `commit_storage`; the
//! first accepted proof claims the deal's provider slot, and each accepted proof
//! releases one tranche from that escrow. A commitment's presence is what anchors
//! a proof, and no value is ever created:
//!
//! | Step   | Movement                        | Primitive             |
//! |--------|---------------------------------|-----------------------|
//! | commit | client free → client reserved   | `reserve`             |
//! | prove  | client reserved → provider free | `repatriate_reserved` |
//! | refund | client reserved → client free   | `unreserve`           |
//!
//! `total_issuance` is invariant across all three, so self-dealing moves an
//! actor's own money between its own accounts for a net loss of fees.
//!
//! ## What this does NOT prove
//!
//! A Merkle path proves the provider *could* produce a valid path over the
//! committed content at proof time — not that it still holds the bytes. The
//! client can `challenge_provider` and reclaim the remaining escrow if the
//! provider fails to answer inside `ChallengeWindow`, which makes non-retention
//! detectable and costly rather than prevented. Proving retention needs
//! chunk-indexed challenges or erasure coding; that is not implemented.
//!
//! See `docs/technical-reference/concensus/STORAGE_COMMITMENT_REGISTRY_DESIGN.md`.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::*,
    traits::{BalanceStatus, Currency, EnsureOrigin, ReservableCurrency},
    weights::constants::RocksDbWeight,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::SubstrateWeight;

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

/// A funded storage deal.
///
/// Keyed by content id in [`pallet::Commitments`], so the content id is not
/// duplicated as a field. `total - released` is the escrow still owed.
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
pub struct StorageCommitment<AccountId, Balance, BlockNumber> {
    /// Who funded the escrow. The only account that can be refunded, and the
    /// only account allowed to challenge or reclaim.
    pub client: AccountId,
    /// Escrow originally reserved.
    pub total: Balance,
    /// Already paid out to the provider.
    pub released: Balance,
    /// Paid per accepted proof: `total / expected_proofs`. Stored because an
    /// exact `u32 → Balance` conversion is not available generically.
    pub tranche: Balance,
    /// Proofs the term expects: `duration / proof_interval`.
    pub expected_proofs: u32,
    /// Proofs paid so far. The commitment closes once this reaches
    /// `expected_proofs`.
    pub proofs_paid: u32,
    /// Set by the first accepted proof and fixed thereafter.
    pub provider: Option<AccountId>,
    /// Minimum spacing between paid proofs.
    pub proof_interval: BlockNumber,
    /// Block of the last paid proof (the commit block before the first).
    pub last_proof_at: BlockNumber,
    /// Term end. Surviving escrow returns to the client at or after this block.
    pub expires_at: BlockNumber,
    /// Set while a client challenge is open; cleared by an accepted proof.
    pub challenged_at: Option<BlockNumber>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::parameter_types;

    /// Uniquely identifies a content item on Pakit: BLAKE2-256 commitment.
    pub type ContentId = [u8; CID_COMMITMENT_LEN];

    /// Balance type of the configured currency.
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    parameter_types! {
        pub MaxProofSteps: u32 = MAX_PROOF_STEPS;
    }

    /// On-chain storage version for this pallet.
    ///
    /// Bump this and register a migration in the runtime's `Migrations` tuple
    /// whenever this pallet's storage layout changes.
    pub const STORAGE_VERSION: frame_support::traits::StorageVersion =
        frame_support::traits::StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Origin that may administratively revoke a previously-accepted proof
        /// (e.g. a proven-byzantine storage provider). Typically a council or
        /// sudo-assigned root.
        type RevocationOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Currency used for the refundable proof deposit.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Refundable deposit reserved from the submitter for every stored
        /// proof. Without it, storing a proof would be permanent free state
        /// growth that any funded account could drive (H-1).
        #[pallet::constant]
        type StorageDeposit: Get<BalanceOf<Self>>;

        /// Minimum escrow per commitment. Also guarantees a non-zero tranche.
        #[pallet::constant]
        type MinEscrow: Get<BalanceOf<Self>>;

        /// Floor on the caller-supplied proof interval, so nobody can demand an
        /// unrealistic proof cadence.
        #[pallet::constant]
        type MinProofInterval: Get<BlockNumberFor<Self>>;

        /// Longest commitment term accepted.
        #[pallet::constant]
        type MaxCommitmentDuration: Get<BlockNumberFor<Self>>;

        /// Maximum simultaneously-open commitments per client, bounding
        /// per-account state growth.
        #[pallet::constant]
        type MaxOpenCommitments: Get<u32>;

        /// Blocks a provider has to answer a client challenge before the client
        /// may reclaim the remaining escrow.
        #[pallet::constant]
        type ChallengeWindow: Get<BlockNumberFor<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// CID commitment → submitter + height. ValueQuery: unset rows read as default.
    #[pallet::storage]
    pub type StorageProofs<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, ProofRecord<BlockNumberFor<T>, T::AccountId>>;

    /// Content id → funded storage deal.
    ///
    /// The presence of a row is the pallet's anchor: `submit_storage_proof`
    /// refuses content that nobody has funded, so a caller cannot be paid for a
    /// root it merely declared.
    #[pallet::storage]
    pub type Commitments<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ContentId,
        StorageCommitment<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    >;

    /// Client → number of open commitments.
    #[pallet::storage]
    pub type ClientCommitmentCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

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
        /// Submitter cannot afford the refundable storage deposit.
        InsufficientDeposit,
        /// No stored proof exists for this content id.
        ProofNotFound,

        // --- storage commitments ---
        /// No storage commitment exists for this content id, so nothing funds a
        /// payout for it.
        NoCommitment,
        /// The commitment's term has ended.
        CommitmentExpired,
        /// The commitment's term has not ended yet.
        CommitmentNotExpired,
        /// A commitment already exists for this content id.
        CommitmentExists,
        /// A proof arrived before the commitment's interval elapsed.
        ProofTooSoon,
        /// The submitter is not the provider that claimed this deal.
        NotDealProvider,
        /// Escrow is below the configured minimum, or the tranche rounds to zero.
        EscrowBelowMinimum,
        /// The commitment term is zero, longer than allowed, or shorter than one
        /// interval.
        InvalidDuration,
        /// The proof interval is below the configured minimum.
        IntervalBelowMinimum,
        /// This account already has the maximum number of open commitments.
        TooManyCommitments,
        /// This account is not the commitment's client.
        NotCommitmentClient,
        /// No provider has claimed this deal yet.
        NoProvider,
        /// A challenge is already open on this commitment.
        ChallengeAlreadyOpen,
        /// No challenge is open on this commitment.
        NoOpenChallenge,
        /// The challenge window has not elapsed yet.
        ChallengeStillOpen,
        /// Escrow cannot cover the next payout.
        EscrowExhausted,
        /// The client's reserved escrow could not be released.
        EscrowUnavailable,
        /// The client cannot cover the requested escrow.
        InsufficientEscrow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a Merkle storage proof against a funded storage commitment.
        ///
        /// The commitment is the pallet's anchor: `content_id` and `proof` both
        /// arrive from the caller, so consistency between them proves nothing on
        /// its own. Requiring a commitment that a *client* funded with their own
        /// escrow is what makes the check meaningful, because every payout moves
        /// the client's reserved balance instead of creating new issuance.
        ///
        /// On success the client's escrow releases one tranche to the provider,
        /// any open challenge is answered, and the proof is recorded.
        ///
        /// - `content_id`: 32-byte BLAKE2-256 of the content Merkle root — must
        ///   equal the Merkle root derivable from the provided proof, and must
        ///   have a live funded commitment.
        /// - `proof`: ordered siblings walking leaf → root.
        #[pallet::weight(
            T::WeightInfo::submit_storage_proof(proof.siblings.len() as u32)
                // Commitment read + proof-record existence check. Estimate, not
                // benchmarked: `weights.rs` predates the commitment registry.
                .saturating_add(RocksDbWeight::get().reads(2))
                // Commitment write or removal, proof record, client counter.
                .saturating_add(RocksDbWeight::get().reads_writes(1, 3))
        )]
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

            let now = <frame_system::Pallet<T>>::block_number();

            // A commitment is the pallet's anchor: both `content_id` and `proof`
            // come from the caller, so requiring content that somebody *paid* to
            // have stored is what stops a self-declared root being rewarded.
            let mut commitment =
                Commitments::<T>::get(derived_root).ok_or(Error::<T>::NoCommitment)?;
            // Term is inclusive of its final block: `expected_proofs` x
            // `proof_interval` can land exactly on `expires_at`, and the refunder
            // only acts strictly after it, so the two never overlap.
            ensure!(now <= commitment.expires_at, Error::<T>::CommitmentExpired);
            // Repeated proofs over the term are the point — that is what the
            // tranches pay for — so the spacing rule, not a duplicate guard, is
            // what bounds payment. Once every expected proof has been paid the
            // commitment closes, after which `NoCommitment` applies.
            ensure!(
                now >= commitment
                    .last_proof_at
                    .saturating_add(commitment.proof_interval),
                Error::<T>::ProofTooSoon
            );

            // The first accepted proof claims the deal; afterwards only that
            // provider is paid, so a latecomer cannot take over the escrow.
            match commitment.provider.clone() {
                None => {
                    commitment.provider = Some(submitter.clone());
                    Self::deposit_event(Event::ProviderClaimed {
                        content_id: derived_root,
                        provider: submitter.clone(),
                    });
                }
                Some(provider) => ensure!(provider == submitter, Error::<T>::NotDealProvider),
            }

            // Pay one tranche out of the client's reserved escrow.
            // `repatriate_reserved` moves reserved → free without touching total
            // issuance, which is why this pallet needs no mint and no supply cap.
            //
            // Once every expected proof has been paid the escrow is spent, but the
            // commitment deliberately persists until its term ends rather than
            // closing early: that keeps the content id claimed for the full term
            // instead of leaving it free to be funded again immediately.
            //
            // Both guards below are unreachable by construction, kept as
            // invariants rather than as live paths. With `expected_proofs =
            // duration / proof_interval`, the next proof after the last paid one
            // could only fall at `commit + (expected + 1) * proof_interval`, which
            // is strictly past `expires_at` — so `CommitmentExpired` always fires
            // first. A commitment therefore can never be asked to pay a tranche it
            // does not hold.
            ensure!(
                commitment.proofs_paid < commitment.expected_proofs,
                Error::<T>::EscrowExhausted
            );
            let remaining = commitment.total.saturating_sub(commitment.released);
            let proofs_paid = commitment.proofs_paid.saturating_add(1);
            // The final proof sweeps the remainder, so integer division cannot
            // strand escrow in the client's reserved balance.
            let payout = if proofs_paid >= commitment.expected_proofs {
                remaining
            } else {
                commitment.tranche
            };
            ensure!(!payout.is_zero(), Error::<T>::EscrowExhausted);

            let leftover = T::Currency::repatriate_reserved(
                &commitment.client,
                &submitter,
                payout,
                BalanceStatus::Free,
            )
            .map_err(|_| Error::<T>::EscrowUnavailable)?;
            ensure!(leftover.is_zero(), Error::<T>::EscrowUnavailable);

            // An accepted proof inside the window *is* the answer to a challenge.
            let answered_challenge = commitment.challenged_at.take().is_some();

            commitment.released = commitment.released.saturating_add(payout);
            commitment.proofs_paid = proofs_paid;
            commitment.last_proof_at = now;

            // Reserve the refundable storage deposit. The proof record is an
            // upsert, so the deposit is taken only for the first proof of this
            // content — otherwise every tranche would lock another deposit
            // against a single record, and revocation could return only one.
            if !StorageProofs::<T>::contains_key(derived_root) {
                T::Currency::reserve(&submitter, T::StorageDeposit::get())
                    .map_err(|_| Error::<T>::InsufficientDeposit)?;
            }

            StorageProofs::<T>::insert(
                derived_root,
                ProofRecord {
                    submitter: submitter.clone(),
                    submitted_at: now,
                    proof_type,
                },
            );

            if answered_challenge {
                Self::deposit_event(Event::ChallengeAnswered {
                    content_id: derived_root,
                    provider: submitter.clone(),
                });
            }

            Self::deposit_event(Event::TrancheReleased {
                content_id: derived_root,
                provider: submitter.clone(),
                amount: payout,
                released_total: commitment.released,
            });

            let released_total = commitment.released;
            let paid_in_full = commitment.proofs_paid >= commitment.expected_proofs;
            Commitments::<T>::insert(derived_root, commitment);

            if paid_in_full {
                Self::deposit_event(Event::CommitmentPaidOut {
                    content_id: derived_root,
                    provider: submitter.clone(),
                    released: released_total,
                });
            }

            Self::deposit_event(Event::StorageProofAccepted {
                content_id: derived_root,
                submitter,
            });
            Ok(())
        }

        /// Administrative revocation (e.g. byzantine behaviour proven).
        ///
        /// Returns the storage deposit to the original submitter and removes
        /// the record so the content can be proven again by a good provider.
        #[pallet::weight(T::WeightInfo::revoke_proof())]
        #[pallet::call_index(1)]
        pub fn revoke_proof(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            T::RevocationOrigin::ensure_origin(origin)?;
            let record = StorageProofs::<T>::get(content_id).ok_or(Error::<T>::ProofNotFound)?;

            // Refund the deposit to the account that reserved it.
            let _ = T::Currency::unreserve(&record.submitter, T::StorageDeposit::get());

            StorageProofs::<T>::remove(content_id);
            Self::deposit_event(Event::StorageProofRevoked { content_id });
            Ok(())
        }

        /// Fund storage of `content_id` and open a commitment for it.
        ///
        /// This is the only way an anchor comes into existence, and it is what
        /// makes paying for a proof safe: the escrow is the client's own money,
        /// so no value is created and a self-declared root buys nothing.
        ///
        /// - `escrow`: reserved from the client for the whole term.
        /// - `proof_interval`: minimum blocks between paid proofs.
        /// - `duration`: term length; `duration / proof_interval` proofs are paid.
        #[pallet::weight(T::WeightInfo::commit_storage())]
        #[pallet::call_index(2)]
        pub fn commit_storage(
            origin: OriginFor<T>,
            content_id: ContentId,
            escrow: BalanceOf<T>,
            proof_interval: BlockNumberFor<T>,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let client = ensure_signed(origin)?;
            let now = <frame_system::Pallet<T>>::block_number();

            ensure!(
                escrow >= T::MinEscrow::get(),
                Error::<T>::EscrowBelowMinimum
            );
            ensure!(
                proof_interval >= T::MinProofInterval::get(),
                Error::<T>::IntervalBelowMinimum
            );
            // Strictly less, so at least one full proof window fits before
            // expiry — otherwise the commitment could never be proved at all.
            ensure!(
                proof_interval < duration && duration <= T::MaxCommitmentDuration::get(),
                Error::<T>::InvalidDuration
            );

            let expected_proofs = (duration / proof_interval).saturated_into::<u32>();
            ensure!(expected_proofs > 0, Error::<T>::InvalidDuration);

            let tranche = escrow / BalanceOf::<T>::from(expected_proofs);
            ensure!(!tranche.is_zero(), Error::<T>::EscrowBelowMinimum);

            ensure!(
                !Commitments::<T>::contains_key(content_id),
                Error::<T>::CommitmentExists
            );
            ensure!(
                ClientCommitmentCount::<T>::get(&client) < T::MaxOpenCommitments::get(),
                Error::<T>::TooManyCommitments
            );

            T::Currency::reserve(&client, escrow).map_err(|_| Error::<T>::InsufficientEscrow)?;

            let expires_at = now.saturating_add(duration);
            Commitments::<T>::insert(
                content_id,
                StorageCommitment {
                    client: client.clone(),
                    total: escrow,
                    released: Zero::zero(),
                    tranche,
                    expected_proofs,
                    proofs_paid: 0,
                    provider: None,
                    proof_interval,
                    last_proof_at: now,
                    expires_at,
                    challenged_at: None,
                },
            );
            ClientCommitmentCount::<T>::mutate(&client, |open| *open = open.saturating_add(1));

            Self::deposit_event(Event::StorageCommitted {
                content_id,
                client,
                escrow,
                expires_at,
            });
            Ok(())
        }

        /// Reclaim whatever escrow is left after the term ends.
        ///
        /// Permissionless: the destination is fixed to the commitment's client,
        /// so there is no griefing vector in letting anyone settle it.
        #[pallet::weight(T::WeightInfo::refund_expired())]
        #[pallet::call_index(3)]
        pub fn refund_expired(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            ensure_signed(origin)?;

            let commitment = Commitments::<T>::get(content_id).ok_or(Error::<T>::NoCommitment)?;
            let now = <frame_system::Pallet<T>>::block_number();
            // Strictly after, so it cannot race a proof submitted on the final
            // block of the term (which `submit_storage_proof` accepts).
            ensure!(
                now > commitment.expires_at,
                Error::<T>::CommitmentNotExpired
            );

            let remainder = commitment.total.saturating_sub(commitment.released);
            if !remainder.is_zero() {
                let _ = T::Currency::unreserve(&commitment.client, remainder);
            }

            Self::close_commitment(content_id, &commitment);
            Self::deposit_event(Event::CommitmentRefunded {
                content_id,
                client: commitment.client,
                amount: remainder,
            });
            Ok(())
        }

        /// Challenge the provider to prove it still holds the content.
        ///
        /// The client is the only account with the incentive to police a deal,
        /// so it is the only origin allowed to challenge. If the provider does
        /// not answer with an accepted proof inside `ChallengeWindow`, the client
        /// can reclaim the remaining escrow via `claim_challenge_failed`.
        ///
        /// This does not *prove* retention; it makes non-retention detectable by
        /// the party whose money is at stake.
        #[pallet::weight(T::WeightInfo::challenge_provider())]
        #[pallet::call_index(4)]
        pub fn challenge_provider(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut commitment =
                Commitments::<T>::get(content_id).ok_or(Error::<T>::NoCommitment)?;
            ensure!(who == commitment.client, Error::<T>::NotCommitmentClient);
            ensure!(commitment.provider.is_some(), Error::<T>::NoProvider);
            ensure!(
                commitment.challenged_at.is_none(),
                Error::<T>::ChallengeAlreadyOpen
            );

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now <= commitment.expires_at, Error::<T>::CommitmentExpired);

            commitment.challenged_at = Some(now);
            let deadline = now.saturating_add(T::ChallengeWindow::get());
            Commitments::<T>::insert(content_id, commitment);

            Self::deposit_event(Event::ProviderChallenged {
                content_id,
                deadline,
            });
            Ok(())
        }

        /// Reclaim the remaining escrow after an unanswered challenge.
        ///
        /// The exit is earned rather than discretionary: the chain itself
        /// verifies that the challenge window elapsed with no accepted proof, so
        /// a client cannot rug a provider that is still answering challenges.
        #[pallet::weight(T::WeightInfo::claim_challenge_failed())]
        #[pallet::call_index(5)]
        pub fn claim_challenge_failed(
            origin: OriginFor<T>,
            content_id: ContentId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let commitment = Commitments::<T>::get(content_id).ok_or(Error::<T>::NoCommitment)?;
            ensure!(who == commitment.client, Error::<T>::NotCommitmentClient);

            let challenged_at = commitment
                .challenged_at
                .ok_or(Error::<T>::NoOpenChallenge)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                now >= challenged_at.saturating_add(T::ChallengeWindow::get()),
                Error::<T>::ChallengeStillOpen
            );

            let remainder = commitment.total.saturating_sub(commitment.released);
            if !remainder.is_zero() {
                let _ = T::Currency::unreserve(&commitment.client, remainder);
            }

            let provider = commitment.provider.clone();
            Self::close_commitment(content_id, &commitment);
            Self::deposit_event(Event::ChallengeDefaulted {
                content_id,
                provider,
                refunded: remainder,
            });
            Ok(())
        }

        /// End a deal early and return the unspent escrow to the client.
        ///
        /// The client is the payer, so it may stop funding storage it no longer
        /// needs, whether or not a provider has claimed the deal.
        ///
        /// **Note the asymmetry this creates:** the provider keeps everything
        /// already paid for accepted proofs, but has no claim on the remaining
        /// tranches and no recourse if the client walks away. That is deliberate
        /// — the client cannot be forced to keep paying — but it does mean a
        /// provider's expected future income is not guaranteed.
        #[pallet::weight(T::WeightInfo::release_provider())]
        #[pallet::call_index(6)]
        pub fn release_provider(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let commitment = Commitments::<T>::get(content_id).ok_or(Error::<T>::NoCommitment)?;
            ensure!(who == commitment.client, Error::<T>::NotCommitmentClient);

            let remainder = commitment.total.saturating_sub(commitment.released);
            if !remainder.is_zero() {
                let _ = T::Currency::unreserve(&commitment.client, remainder);
            }

            let provider = commitment.provider.clone();
            Self::close_commitment(content_id, &commitment);
            Self::deposit_event(Event::CommitmentReleased {
                content_id,
                client: commitment.client,
                provider,
                refunded: remainder,
            });
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
        /// A storage commitment was created and escrow reserved.
        StorageCommitted {
            content_id: ContentId,
            client: T::AccountId,
            escrow: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// A commitment's unspent escrow was returned after expiry.
        CommitmentRefunded {
            content_id: ContentId,
            client: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// The first accepted proof claimed the deal's provider slot.
        ProviderClaimed {
            content_id: ContentId,
            provider: T::AccountId,
        },
        /// One tranche of a client's escrow was paid to the provider.
        TrancheReleased {
            content_id: ContentId,
            provider: T::AccountId,
            amount: BalanceOf<T>,
            released_total: BalanceOf<T>,
        },
        /// Every expected proof was delivered, so the escrow is fully spent.
        ///
        /// The commitment is **not** removed — it persists until `expires_at`,
        /// holding the content id for the rest of the term.
        CommitmentPaidOut {
            content_id: ContentId,
            provider: T::AccountId,
            released: BalanceOf<T>,
        },
        /// The client ended a deal early and took back the unspent escrow.
        CommitmentReleased {
            content_id: ContentId,
            client: T::AccountId,
            provider: Option<T::AccountId>,
            refunded: BalanceOf<T>,
        },
        /// The client challenged the provider to prove retention again.
        ProviderChallenged {
            content_id: ContentId,
            deadline: BlockNumberFor<T>,
        },
        /// The provider answered a challenge with an accepted proof in time.
        ChallengeAnswered {
            content_id: ContentId,
            provider: T::AccountId,
        },
        /// A challenge went unanswered past its deadline, so the client took the
        /// remaining escrow back and the deal terminated.
        ChallengeDefaulted {
            content_id: ContentId,
            provider: Option<T::AccountId>,
            refunded: BalanceOf<T>,
        },
    }

    impl<T: Config> Pallet<T> {
        /// Removes a commitment and frees its per-client slot.
        ///
        /// Escrow accounting is the caller's responsibility: every call site
        /// either pays it out in full or unreserves the remainder first.
        fn close_commitment(
            content_id: ContentId,
            commitment: &StorageCommitment<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        ) {
            Commitments::<T>::remove(content_id);
            ClientCommitmentCount::<T>::mutate(&commitment.client, |open| {
                *open = open.saturating_sub(1)
            });
        }

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

/// Weight information for pallet extrinsics.
///
/// `SubstrateWeight<T>` (in `weights.rs`) carries the benchmarked numbers; the
/// `()` implementation below is the constant fallback used in tests and by any
/// runtime that has not wired generated weights yet.
pub trait WeightInfo {
    /// `s` is the number of Merkle siblings in the submitted proof.
    fn submit_storage_proof(s: u32) -> Weight;
    fn revoke_proof() -> Weight;
    fn commit_storage() -> Weight;
    fn refund_expired() -> Weight;
    fn challenge_provider() -> Weight;
    fn claim_challenge_failed() -> Weight;
    fn release_provider() -> Weight;
}

impl WeightInfo for () {
    fn submit_storage_proof(s: u32) -> Weight {
        Weight::from_parts(20_000_000, 3_500)
            .saturating_add(Weight::from_parts(2_500_000, 300).saturating_mul(s as u64))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn revoke_proof() -> Weight {
        Weight::from_parts(15_000_000, 2_500)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Placeholder — `weights.rs` is regenerated, not benchmarked here.
    fn commit_storage() -> Weight {
        Weight::from_parts(25_000_000, 4_000)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    /// Placeholder — see `commit_storage`.
    fn refund_expired() -> Weight {
        Weight::from_parts(20_000_000, 3_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    /// Placeholder — see `commit_storage`.
    fn challenge_provider() -> Weight {
        Weight::from_parts(20_000_000, 3_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Placeholder — see `commit_storage`.
    fn claim_challenge_failed() -> Weight {
        Weight::from_parts(22_000_000, 3_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    /// Placeholder — see `commit_storage`.
    fn release_provider() -> Weight {
        Weight::from_parts(22_000_000, 3_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
}
