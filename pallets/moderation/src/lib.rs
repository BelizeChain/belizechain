#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Moderation Pallet — Phase 5C
//!
//! Community-driven content moderation with Nawal AI-assisted auto-queuing.
//!
//! # Overview
//!
//! This pallet allows verified community members to flag on-chain content
//! (identified by a 32-byte Blake2b hash).  Flagged items are reviewed by a
//! curated set of human moderators who issue one of three rulings:
//! `Cleared`, `Removed`, or `Escalated`.
//!
//! Nawal AI may submit a risk score (0–100) for any piece of content via a
//! privileged oracle origin.  If the score exceeds `NawalAutoQueueScore` the
//! item is automatically added to the moderation queue without needing to
//! await the community flag threshold.
//!
//! # Thresholds
//!
//! - `FlagThreshold` (default 5) — number of unique community flags that
//!   trigger automatic queuing.
//! - `NawalAutoQueueScore` (default 50) — Nawal risk score that triggers
//!   automatic queuing.
//!
//! # Rulings
//!
//! | Index | Ruling | Effect |
//! |-------|--------|--------|
//! | 0     | Cleared | Item removed from queue; no further action |
//! | 1     | Removed | Item marked as removed; consumers must act |
//! | 2     | Escalated | Item forwarded to governance for constitutional review |
//!
//! # Security Notes
//!
//! - Each account may flag a given content item at most once.
//! - Only accounts in `ModeratorSet` may call `review_content`.
//! - `submit_nawal_assessment` is gated by `NawalOracleOrigin`.
//! - Adding/removing moderators requires `ModeratorAdminOrigin`.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

pub use weights::WeightInfo;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

/// 32-byte Blake2b content hash uniquely identifying a piece of on-chain content.
pub type ContentHash = [u8; 32];

/// Reasons a community member may flag content.
#[derive(
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum FlagReason {
    /// Hate speech or harassment (0)
    HateSpeech,
    /// Misinformation or demonstrably false claims (1)
    Misinformation,
    /// Spam or unsolicited advertising (2)
    Spam,
    /// Illegal content (3)
    IllegalContent,
    /// Compulsive/addictive pattern content (4)
    AddictivePattern,
}

impl FlagReason {
    fn from_index(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::HateSpeech),
            1 => Some(Self::Misinformation),
            2 => Some(Self::Spam),
            3 => Some(Self::IllegalContent),
            4 => Some(Self::AddictivePattern),
            _ => None,
        }
    }
}

/// Ruling issued by a human moderator after content review.
#[derive(
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ModerationRuling {
    /// Content is acceptable; close the queue item (0)
    Cleared,
    /// Content violates community standards; must be removed (1)
    Removed,
    /// Requires constitutional/governance escalation (2)
    Escalated,
}

impl ModerationRuling {
    fn from_index(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Cleared),
            1 => Some(Self::Removed),
            2 => Some(Self::Escalated),
            _ => None,
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Origin that may add/remove moderators (e.g. governance council).
        type ModeratorAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin that may submit Nawal AI assessments (protected oracle key).
        type NawalOracleOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Number of unique community flags required to auto-queue content.
        #[pallet::constant]
        type FlagThreshold: Get<u32>;

        /// Nawal risk score above which content is auto-queued (0–100).
        #[pallet::constant]
        type NawalAutoQueueScore: Get<u8>;

        /// Maximum number of moderators in the set.
        #[pallet::constant]
        type MaxModerators: Get<u32>;

        /// Maximum number of unique flags per content item (bounds clear_prefix).
        #[pallet::constant]
        type MaxFlagsPerContent: Get<u32>;

        /// Weight information for pallet extrinsics.
        type WeightInfo: WeightInfo;
    }

    // =========================================================================
    // Storage
    // =========================================================================

    /// Tracks which accounts have flagged which content items.
    /// DoubleMap: content_hash × flagger_account → FlagReason.
    /// Prevents the same account from flagging the same item twice.
    #[pallet::storage]
    #[pallet::getter(fn content_flag)]
    pub type ContentFlags<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ContentHash,
        Blake2_128Concat,
        T::AccountId,
        FlagReason,
        OptionQuery,
    >;

    /// Cumulative flag count per content item.
    #[pallet::storage]
    #[pallet::getter(fn flag_count)]
    pub type FlagCounts<T: Config> = StorageMap<_, Blake2_128Concat, ContentHash, u32, ValueQuery>;

    /// Set of content hashes awaiting moderator review.
    #[pallet::storage]
    #[pallet::getter(fn is_queued)]
    pub type ModerationQueue<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentHash, bool, ValueQuery>;

    /// Final ruling issued for a content item, if any.
    #[pallet::storage]
    #[pallet::getter(fn ruling)]
    pub type RuledContent<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentHash, ModerationRuling, OptionQuery>;

    /// Nawal AI risk scores for content items (0 = clean, 100 = extreme risk).
    #[pallet::storage]
    #[pallet::getter(fn nawal_score)]
    pub type NawalAssessments<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentHash, u8, OptionQuery>;

    /// Set of accounts authorised to review queued content.
    /// Stored as a bounded vec of account IDs; duplicates prevented on add.
    #[pallet::storage]
    #[pallet::getter(fn moderator_set)]
    pub type ModeratorSet<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxModerators>, ValueQuery>;

    // =========================================================================
    // Events
    // =========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A community member flagged content.
        ContentFlagged {
            content_hash: ContentHash,
            flagger: T::AccountId,
            reason: u8,
            total_flags: u32,
        },
        /// Content was automatically added to the moderation queue.
        ContentAutoQueued {
            content_hash: ContentHash,
            trigger: AutoQueueTrigger,
        },
        /// A moderator issued a ruling on queued content.
        ContentRuled {
            content_hash: ContentHash,
            moderator: T::AccountId,
            ruling: u8,
        },
        /// Nawal AI submitted a risk assessment.
        NawalAssessmentSubmitted {
            content_hash: ContentHash,
            score: u8,
        },
        /// A moderator was added to the moderation set.
        ModeratorAdded { account: T::AccountId },
        /// A moderator was removed from the moderation set.
        ModeratorRemoved { account: T::AccountId },
    }

    /// What triggered an auto-queue action.
    #[derive(
        Encode,
        Decode,
        codec::DecodeWithMemTracking,
        Clone,
        PartialEq,
        Eq,
        Debug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum AutoQueueTrigger {
        /// Community flag count reached `FlagThreshold`.
        FlagThreshold,
        /// Nawal score exceeded `NawalAutoQueueScore`.
        NawalScore,
    }

    // =========================================================================
    // Errors
    // =========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// Account has already flagged this content item.
        AlreadyFlagged,
        /// The provided flag reason index is not valid (must be 0–4).
        InvalidFlagReason,
        /// The provided ruling index is not valid (must be 0–2).
        InvalidRuling,
        /// Content is not in the moderation queue.
        NotInQueue,
        /// Caller is not a registered moderator.
        NotModerator,
        /// Account is already in the moderator set.
        AlreadyModerator,
        /// Account is not in the moderator set.
        ModeratorNotFound,
        /// The moderator set has reached `MaxModerators` capacity.
        ModeratorSetFull,
        /// Content has already received a final ruling; re-flagging not permitted.
        AlreadyRuled,
        /// Nawal risk score must be in the range 0–100.
        ScoreOutOfRange,
        /// Maximum flags per content item reached.
        FlagLimitReached,
    }

    // =========================================================================
    // Extrinsics
    // =========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Flag a piece of content for moderator review.
        ///
        /// Any signed account may flag content at most once per item.  When the
        /// total unique flag count reaches `FlagThreshold` the item is added to
        /// the moderation queue automatically.
        ///
        /// ## Parameters
        /// - `content_hash`: 32-byte Blake2b hash of the content.
        /// - `reason_index`: Category code (0=HateSpeech, 1=Misinformation,
        ///   2=Spam, 3=IllegalContent, 4=AddictivePattern).
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::flag_content())]
        pub fn flag_content(
            origin: OriginFor<T>,
            content_hash: ContentHash,
            reason_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Reject if already given a final ruling
            ensure!(
                RuledContent::<T>::get(content_hash).is_none(),
                Error::<T>::AlreadyRuled
            );

            // Validate reason
            let reason =
                FlagReason::from_index(reason_index).ok_or(Error::<T>::InvalidFlagReason)?;

            // Prevent double-flagging by the same account
            ensure!(
                ContentFlags::<T>::get(content_hash, &who).is_none(),
                Error::<T>::AlreadyFlagged
            );

            // Record the flag
            ContentFlags::<T>::insert(content_hash, &who, reason);
            let new_count = FlagCounts::<T>::get(content_hash).saturating_add(1);
            // MOD-01 FIX: Enforce MaxFlagsPerContent to bound storage and clear_prefix
            ensure!(
                new_count <= T::MaxFlagsPerContent::get(),
                Error::<T>::FlagLimitReached
            );
            FlagCounts::<T>::insert(content_hash, new_count);

            Self::deposit_event(Event::ContentFlagged {
                content_hash,
                flagger: who,
                reason: reason_index,
                total_flags: new_count,
            });

            // Auto-queue if flag threshold reached
            if new_count >= T::FlagThreshold::get() && !ModerationQueue::<T>::get(content_hash) {
                ModerationQueue::<T>::insert(content_hash, true);
                Self::deposit_event(Event::ContentAutoQueued {
                    content_hash,
                    trigger: AutoQueueTrigger::FlagThreshold,
                });
            }

            Ok(())
        }

        /// Issue a moderation ruling on a queued content item.
        ///
        /// Caller must be a registered moderator.  The item must be present in
        /// the moderation queue.  After ruling the item is removed from the
        /// queue and the ruling is permanently recorded.
        ///
        /// ## Parameters
        /// - `content_hash`: 32-byte Blake2b hash of the queued content.
        /// - `ruling_index`: Ruling code (0=Cleared, 1=Removed, 2=Escalated).
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::review_content())]
        pub fn review_content(
            origin: OriginFor<T>,
            content_hash: ContentHash,
            ruling_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Caller must be a registered moderator
            let mods = ModeratorSet::<T>::get();
            ensure!(mods.contains(&who), Error::<T>::NotModerator);

            // Item must be queued
            ensure!(
                ModerationQueue::<T>::get(content_hash),
                Error::<T>::NotInQueue
            );

            // Validate ruling
            let ruling =
                ModerationRuling::from_index(ruling_index).ok_or(Error::<T>::InvalidRuling)?;

            // Dequeue and record ruling
            ModerationQueue::<T>::remove(content_hash);
            RuledContent::<T>::insert(content_hash, ruling);

            // MOD-01 FIX: Bounded clear_prefix using MaxFlagsPerContent instead of u32::MAX
            let _ =
                ContentFlags::<T>::clear_prefix(content_hash, T::MaxFlagsPerContent::get(), None);
            FlagCounts::<T>::remove(content_hash);
            NawalAssessments::<T>::remove(content_hash);

            Self::deposit_event(Event::ContentRuled {
                content_hash,
                moderator: who,
                ruling: ruling_index,
            });

            Ok(())
        }

        /// Register an account as a content moderator.
        ///
        /// Requires `ModeratorAdminOrigin` (e.g. governance council multisig).
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::add_moderator())]
        pub fn add_moderator(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::ModeratorAdminOrigin::ensure_origin(origin)?;

            ModeratorSet::<T>::try_mutate(|mods| -> DispatchResult {
                ensure!(!mods.contains(&account), Error::<T>::AlreadyModerator);
                mods.try_push(account.clone())
                    .map_err(|_| Error::<T>::ModeratorSetFull)?;
                Ok(())
            })?;

            Self::deposit_event(Event::ModeratorAdded { account });
            Ok(())
        }

        /// Remove an account from the moderator set.
        ///
        /// Requires `ModeratorAdminOrigin`.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::remove_moderator())]
        pub fn remove_moderator(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::ModeratorAdminOrigin::ensure_origin(origin)?;

            ModeratorSet::<T>::try_mutate(|mods| -> DispatchResult {
                let pos = mods
                    .iter()
                    .position(|a| a == &account)
                    .ok_or(Error::<T>::ModeratorNotFound)?;
                mods.remove(pos);
                Ok(())
            })?;

            Self::deposit_event(Event::ModeratorRemoved { account });
            Ok(())
        }

        /// Submit a Nawal AI risk assessment for a content item.
        ///
        /// Requires `NawalOracleOrigin`.  Overwrites any previous score.  If
        /// the new score exceeds `NawalAutoQueueScore` the item is queued (if
        /// not already queued or ruled).
        ///
        /// ## Parameters
        /// - `content_hash`: 32-byte Blake2b hash of the content.
        /// - `score`: Risk score in the range `[0, 100]`.  Higher = riskier.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::submit_nawal_assessment())]
        pub fn submit_nawal_assessment(
            origin: OriginFor<T>,
            content_hash: ContentHash,
            score: u8,
        ) -> DispatchResult {
            T::NawalOracleOrigin::ensure_origin(origin)?;

            // M-4 FIX: Validate score range (0–100)
            ensure!(score <= 100, Error::<T>::ScoreOutOfRange);

            NawalAssessments::<T>::insert(content_hash, score);

            Self::deposit_event(Event::NawalAssessmentSubmitted {
                content_hash,
                score,
            });

            // Auto-queue if score exceeds threshold and item not already queued/ruled
            if score > T::NawalAutoQueueScore::get()
                && !ModerationQueue::<T>::get(content_hash)
                && RuledContent::<T>::get(content_hash).is_none()
            {
                ModerationQueue::<T>::insert(content_hash, true);
                Self::deposit_event(Event::ContentAutoQueued {
                    content_hash,
                    trigger: AutoQueueTrigger::NawalScore,
                });
            }

            Ok(())
        }
    }

    // =========================================================================
    // Public API (callable from runtime and other pallets)
    // =========================================================================

    impl<T: Config> Pallet<T> {
        /// Returns `true` if the content item has been ruled `Removed`.
        pub fn is_removed(content_hash: &ContentHash) -> bool {
            matches!(
                RuledContent::<T>::get(content_hash),
                Some(ModerationRuling::Removed)
            )
        }

        /// Returns `true` if the content item is currently queued for review.
        pub fn is_queued_for_review(content_hash: &ContentHash) -> bool {
            ModerationQueue::<T>::get(content_hash)
        }

        /// Returns the Nawal AI risk score for a content item, if any.
        pub fn nawal_risk_score(content_hash: &ContentHash) -> Option<u8> {
            NawalAssessments::<T>::get(content_hash)
        }
    }
}
