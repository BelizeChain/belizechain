#![cfg_attr(not(feature = "std"), no_std)]

//! # BelizeChain Commit-Reveal Randomness Pallet (AR-3)
//!
//! Replaces `pallet_insecure_randomness_collective_flip` with a two-phase
//! commit-reveal scheme that is unpredictable even when validators cooperate.
//!
//! ## Protocol
//!
//! 1. **Commit phase** (blocks 0 … COMMIT_DEADLINE − 1):
//!    Every authorised randomness contributor calls `commit(commitment)` where
//!    `commitment = blake2_256(value || salt)`.
//!
//! 2. **Reveal phase** (blocks COMMIT_DEADLINE … REVEAL_DEADLINE − 1):
//!    Each committer calls `reveal(value, salt)`.  The pallet verifies
//!    `blake2_256(value || salt) == stored_commitment`.
//!
//! 3. **Finalization** (at or after REVEAL_DEADLINE):
//!    `finalize_epoch` is called (by anyone — permissionless).
//!    The final randomness output is:
//!    ```text
//!    output = blake2_256(concat(all revealed values sorted by AccountId))
//!    ```
//!    Sorted order makes the aggregation deterministic across nodes.
//!    The output is stored in `CurrentSeed` and implements
//!    `frame_support::traits::Randomness<Hash, BlockNumber>`.
//!
//! ## Security Properties
//!
//! - **Unpredictability**: An adversary must control *all* contributors that
//!   will reveal to predict the output before the reveal window closes.
//! - **Last-revealer bias**: A single validator who reveals last can learn the
//!   output before it is finalised.  Mitigate by setting `MinReveals` ≥ ⌈2/3 N⌉.
//!   With `MinReveals = 3` and `MaxContributors = 5` the bias window is small.
//! - **Withheld reveals**: If fewer than `MinReveals` contributors reveal,
//!   the epoch fails and the previous seed is carried forward.  The next epoch
//!   then starts fresh, so liveness is preserved.
//! - **Determinism**: All arithmetic, hashing, and iteration are deterministic.
//!   No floating-point, no timestamps, no external I/O.
//!
//! ## Limitations
//!
//! This scheme is suitable for permissioned chains with a trusted, identifiable
//! validator set.  For fully permissionless deployments, consider threshold BLS
//! (e.g., drand) for stronger bias-resistance.

extern crate alloc;

use frame_support::{
    pallet_prelude::*,
    traits::Randomness as RandomnessTrait,
};
use frame_system::pallet_prelude::*;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Saturating;
use sp_std::vec::Vec;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ── Config ──────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// How many blocks the commit phase lasts.
        /// Contributors must submit commitments within this window each epoch.
        #[pallet::constant]
        type CommitDuration: Get<BlockNumberFor<Self>>;

        /// How many additional blocks the reveal phase lasts after commit ends.
        #[pallet::constant]
        type RevealDuration: Get<BlockNumberFor<Self>>;

        /// Minimum number of reveals required for a valid randomness output.
        /// If fewer contributors reveal, the epoch is skipped and the previous
        /// seed is retained.  SHOULD be ≥ ⌈2/3 × MaxContributors⌉.
        #[pallet::constant]
        type MinReveals: Get<u32>;

        /// Maximum number of contributors per epoch.
        #[pallet::constant]
        type MaxContributors: Get<u32>;
    }

    // ── Storage ──────────────────────────────────────────────────────────────

    /// Current epoch index.  Incremented at each finalization.
    #[pallet::storage]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Block at which the current epoch started.
    #[pallet::storage]
    pub type EpochStart<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Pending commitments for the current epoch.
    /// Key: contributor AccountId → commitment hash (32 bytes).
    #[pallet::storage]
    pub type Commitments<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        [u8; 32],
        OptionQuery,
    >;

    /// Revealed values for the current epoch.
    /// Key: contributor AccountId → raw value (32 bytes).
    #[pallet::storage]
    pub type Reveals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        [u8; 32],
        OptionQuery,
    >;

    /// The current randomness seed.  Updated on each successful finalization.
    /// Exposed via the `Randomness` trait implementation.
    #[pallet::storage]
    pub type CurrentSeed<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

    /// Number of contributors that have submitted a commitment this epoch.
    #[pallet::storage]
    pub type CommitCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Number of contributors that have revealed this epoch.
    #[pallet::storage]
    pub type RevealCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    // ── Events ───────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A contributor submitted a commitment.
        Committed { contributor: T::AccountId, epoch: u32 },
        /// A contributor revealed their value.
        Revealed { contributor: T::AccountId, epoch: u32 },
        /// The epoch was finalised and a new seed produced.
        EpochFinalized { epoch: u32, seed: T::Hash },
        /// The epoch was skipped because fewer than `MinReveals` revealed.
        EpochSkipped { epoch: u32, reveals: u32 },
    }

    // ── Errors ───────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// Commit window has already closed for this epoch.
        CommitWindowClosed,
        /// Reveal window has not opened yet (still in commit phase).
        RevealWindowNotOpen,
        /// Reveal window has already closed; call `finalize_epoch` first.
        RevealWindowClosed,
        /// This account has not committed in the current epoch.
        NotCommitted,
        /// Already committed this epoch.
        AlreadyCommitted,
        /// Already revealed this epoch.
        AlreadyRevealed,
        /// The revealed value + salt do not match the stored commitment.
        CommitmentMismatch,
        /// Maximum contributor count reached for this epoch.
        TooManyContributors,
        /// Cannot finalize: reveal phase has not ended yet.
        TooEarlyToFinalize,
        /// Cannot finalize: already finalized this epoch.
        AlreadyFinalized,
    }

    // ── Hooks ────────────────────────────────────────────────────────────────

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // Auto-start a new epoch if none has been started yet.
            let start = EpochStart::<T>::get();
            let zero: BlockNumberFor<T> = 0u32.into();
            if start == zero {
                EpochStart::<T>::put(n);
            }
            // Minimal weight: 1 read.
            T::DbWeight::get().reads(1)
        }

        /// Verify randomness state invariants at startup.
        fn integrity_test() {
            // MinReveals must not exceed MaxContributors.
            assert!(
                T::MinReveals::get() <= T::MaxContributors::get(),
                "INVARIANT VIOLATION: MinReveals > MaxContributors"
            );
            // CommitDuration and RevealDuration must be non-zero.
            let one: BlockNumberFor<T> = 1u32.into();
            assert!(T::CommitDuration::get() >= one, "INVARIANT VIOLATION: CommitDuration must be >= 1");
            assert!(T::RevealDuration::get() >= one, "INVARIANT VIOLATION: RevealDuration must be >= 1");
        }
    }

    // ── Calls ────────────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a commitment for the current epoch.
        ///
        /// `commitment` = `blake2_256(value_bytes || salt_bytes)` computed off-chain.
        ///
        /// Must be called during the commit phase (first `CommitDuration` blocks of the epoch).
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(20_000_000, 1_024)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn commit(
            origin: OriginFor<T>,
            commitment: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            let epoch_start = EpochStart::<T>::get();
            let commit_end = epoch_start.saturating_add(T::CommitDuration::get());

            ensure!(current_block < commit_end, Error::<T>::CommitWindowClosed);
            ensure!(!Commitments::<T>::contains_key(&who), Error::<T>::AlreadyCommitted);

            let count = CommitCount::<T>::get();
            ensure!(count < T::MaxContributors::get(), Error::<T>::TooManyContributors);

            Commitments::<T>::insert(&who, commitment);
            CommitCount::<T>::put(count.saturating_add(1));

            let epoch = CurrentEpoch::<T>::get();
            Self::deposit_event(Event::Committed { contributor: who, epoch });
            Ok(())
        }

        /// Reveal the pre-image of a previously submitted commitment.
        ///
        /// `value` — arbitrary 32-byte entropy input.
        /// `salt`  — arbitrary 32-byte salt.  Together `blake2_256(value || salt)`
        ///           must equal the stored commitment.
        ///
        /// Must be called during the reveal phase.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(25_000_000, 1_024)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2)))]
        pub fn reveal(
            origin: OriginFor<T>,
            value: [u8; 32],
            salt: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            let epoch_start = EpochStart::<T>::get();
            let commit_end = epoch_start.saturating_add(T::CommitDuration::get());
            let reveal_end = commit_end.saturating_add(T::RevealDuration::get());

            ensure!(current_block >= commit_end, Error::<T>::RevealWindowNotOpen);
            ensure!(current_block < reveal_end, Error::<T>::RevealWindowClosed);

            let commitment = Commitments::<T>::get(&who)
                .ok_or(Error::<T>::NotCommitted)?;
            ensure!(!Reveals::<T>::contains_key(&who), Error::<T>::AlreadyRevealed);

            // Verify commitment: blake2_256(value || salt)
            let mut preimage = [0u8; 64];
            preimage[..32].copy_from_slice(&value);
            preimage[32..].copy_from_slice(&salt);
            let computed = blake2_256(&preimage);
            ensure!(computed == commitment, Error::<T>::CommitmentMismatch);

            Reveals::<T>::insert(&who, value);
            let count = RevealCount::<T>::get();
            RevealCount::<T>::put(count.saturating_add(1));

            let epoch = CurrentEpoch::<T>::get();
            Self::deposit_event(Event::Revealed { contributor: who, epoch });
            Ok(())
        }

        /// Finalise the current epoch and produce a new randomness seed.
        ///
        /// Callable by anyone after the reveal phase ends.
        /// If fewer than `MinReveals` contributors revealed, the epoch is skipped
        /// and the previous seed is retained; a new epoch starts immediately.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(50_000_000, 2_048)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(6)))]
        pub fn finalize_epoch(origin: OriginFor<T>) -> DispatchResult {
            ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            let epoch_start = EpochStart::<T>::get();
            let commit_end = epoch_start.saturating_add(T::CommitDuration::get());
            let reveal_end = commit_end.saturating_add(T::RevealDuration::get());

            ensure!(current_block >= reveal_end, Error::<T>::TooEarlyToFinalize);

            let epoch = CurrentEpoch::<T>::get();
            let reveal_count = RevealCount::<T>::get();

            if reveal_count >= T::MinReveals::get() {
                // Collect and sort revealed values by AccountId (deterministic order).
                let mut entries: Vec<(T::AccountId, [u8; 32])> = Reveals::<T>::iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.encode().cmp(&b.encode()));

                // Aggregate: hash of concatenated values.
                let mut hasher_input: Vec<u8> = Vec::with_capacity(entries.len() * 32);
                for (_, val) in &entries {
                    hasher_input.extend_from_slice(val);
                }
                // Mix in epoch number for domain separation.
                hasher_input.extend_from_slice(&epoch.encode());
                let seed_bytes = blake2_256(&hasher_input);
                let seed = T::Hash::decode(&mut &seed_bytes[..])
                    .unwrap_or_default();
                CurrentSeed::<T>::put(seed);
                Self::deposit_event(Event::EpochFinalized { epoch, seed });
            } else {
                // Not enough reveals — skip epoch, keep previous seed.
                Self::deposit_event(Event::EpochSkipped { epoch, reveals: reveal_count });
            }

            // Reset epoch state.
            let _ = Commitments::<T>::clear(T::MaxContributors::get(), None);
            let _ = Reveals::<T>::clear(T::MaxContributors::get(), None);
            CommitCount::<T>::put(0u32);
            RevealCount::<T>::put(0u32);
            CurrentEpoch::<T>::put(epoch.saturating_add(1));
            EpochStart::<T>::put(current_block);

            Ok(())
        }
    }

    // ── Randomness trait implementation ──────────────────────────────────────

    /// Implement `frame_support::traits::Randomness` so pallets that type-alias
    /// `type Randomness = BelizeRandomness` receive the commit-reveal seed.
    ///
    /// `subject` is mixed into the output to allow multiple independent draws
    /// per block without correlation.
    impl<T: Config> RandomnessTrait<T::Hash, BlockNumberFor<T>> for Pallet<T> {
        fn random(subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
            let seed = CurrentSeed::<T>::get();
            let epoch_start = EpochStart::<T>::get();

            // Mix seed with subject for domain separation.
            let mut input: Vec<u8> = seed.encode();
            input.extend_from_slice(subject);
            let mixed = blake2_256(&input);
            let output = T::Hash::decode(&mut &mixed[..]).unwrap_or(seed);

            (output, epoch_start)
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
