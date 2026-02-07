//! # Community Pallet
//!
//! BelizeChain's community pallet implementing Social Responsibility Score (SRS),
//! zero-fee protocol, community fund management, and citizen engagement programs.
//!
//! ## Overview
//!
//! This pallet provides:
//! - Social Responsibility Score (SRS) calculation and tracking
//! - Zero-fee transaction protocol for verified citizens
//! - Community treasury proposal system
//! - Learn-to-earn education programs
//! - Green project support and sustainability tracking
//! - Peer endorsement system
//! - Referral rewards
//! - Ethics filter for proposal screening
//!
//! ## Features
//!
//! ### Social Responsibility Score (SRS)
//! 
//! Every verified citizen receives an SRS (0-10,000 scale) based on:
//! - Governance participation (25%)
//! - Education completion (15%)
//! - Sustainability contributions (15%)
//! - General participation (25%)
//! - Peer endorsements (10%)
//! - Honesty rating (10%)
//!
//! SRS determines:
//! - Community voting weight
//! - Fee exemption tiers
//! - Proposal submission eligibility
//! - Access to premium features
//!
//! ### Zero-Fee Protocol
//!
//! Verified citizens with sufficient SRS can transact without fees:
//! - Bronze tier: 100 dBZD/month (governance adjustable)
//! - Silver tier: 100 dBZD/month (governance adjustable)
//! - Gold tier: 100 dBZD/month (governance adjustable)
//! - Platinum+ tier: Unlimited
//!
//! Treasury reimburses validators for exempted fees.
//!
//! ### Community Fund Management
//!
//! 10% of treasury allocated to community proposals:
//! - Local projects
//! - Education modules
//! - Green initiatives
//! - Cultural preservation
//! - Disaster relief
//! - Community bounties
//!
//! Voting is SRS-weighted, no council required.
//!
//! ## Integration
//!
//! Exports traits for other pallets:
//! - `CommunityRank` - Provides voting weights to governance
//! - `FeeCalculator` - Provides fee discounts to economy
//! - `ParticipationTracker` - Receives updates from staking/governance
//!
//! Consumes traits:
//! - `BelizeKyc` - Verifies citizen identity from identity pallet
//! - `Currency` - Handles token transfers and reservations
//!
//! ## Implementation Status
//!
//! **Phase 1**: SRS System (In Progress)
//! - [ ] Data structures and storage
//! - [ ] Score calculation algorithms
//! - [ ] Participation tracking
//! - [ ] Privacy controls
//!
//! **Phase 2**: Zero-Fee Protocol
//! **Phase 3**: Community Fund Management
//! **Phase 4**: Ethics Filter
//! **Phase 5**: Incentive Programs
//! **Phase 6**: Cross-Pallet Integration

#![cfg_attr(not(feature = "std"), no_std)]

// For use in trait implementations outside pallet module
extern crate alloc;
use alloc::vec::Vec;

pub use pallet::*;
pub use types::{
    SRSTier, ActivityType, ParticipationRecord, ProposalStatus, CommunityProposalType, 
    EndorsementType, SRSData, ProposalStats, FeeExemptionData, CommunityProposal, 
    Vote, EthicsConfig, SanctionStatus, EducationModule, CompletionData, GreenProject
}; // Export for traits

// Additional imports for genesis config
use frame_support::BoundedVec;
use sp_core::ConstU32;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Get},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{SaturatedConversion, CheckedAdd};
    
    use crate::weights::WeightInfo;
    use pallet_belize_identity::BelizeKyc as BelizeKycTrait;

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Currency type for handling balances
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// KYC verification from identity pallet
        type BelizeKyc: pallet_belize_identity::BelizeKyc<Self::AccountId, BlockNumberFor<Self>>;

        /// Proposal deposit percentage (default: 10%)
        #[pallet::constant]
        type ProposalDepositPercentage: Get<u32>;

        /// Fee exemption monthly limit in dBZD (default: 100 dBZD)
        #[pallet::constant]
        type FeeExemptionMonthlyLimit: Get<u32>;

        /// Education module reward amount (governance-adjustable for inflation)
        #[pallet::constant]
        type EducationRewardAmount: Get<BalanceOf<Self>>;

        /// Referral reward amount (default: 100 DALLA)
        #[pallet::constant]
        type ReferralRewardAmount: Get<BalanceOf<Self>>;

        /// Community voting period (7 days, matching governance)
        #[pallet::constant]
        type CommunityVotingPeriod: Get<BlockNumberFor<Self>>;

        /// Community treasury account
        #[pallet::constant]
        type CommunityTreasuryAccount: Get<Self::AccountId>;

        /// Governance origin for sanctioning accounts
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Maximum length of proposal title
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;

        /// Maximum length of proposal description
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// Maximum participation history per account
        #[pallet::constant]
        type MaxParticipationHistory: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ================================
    // Storage Items - Phase 1: SRS System
    // ================================

    /// Social Responsibility Scores for all accounts
    /// Maps AccountId -> SRSData
    #[pallet::storage]
    #[pallet::getter(fn srs_scores)]
    pub type SocialResponsibilityScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        SRSData<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Participation history for SRS calculation
    /// Maps AccountId -> BoundedVec<ParticipationRecord>
    #[pallet::storage]
    #[pallet::getter(fn participation_history)]
    pub type ParticipationHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<ParticipationRecord<BlockNumberFor<T>>, T::MaxParticipationHistory>,
        ValueQuery,
    >;

    /// Peer endorsements count
    /// Maps AccountId -> u32
    #[pallet::storage]
    #[pallet::getter(fn peer_endorsements)]
    pub type PeerEndorsements<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Last endorsement timestamp between two accounts
    /// Maps (Endorser, Endorsee) -> BlockNumber
    #[pallet::storage]
    #[pallet::getter(fn last_endorsement)]
    pub type LastEndorsement<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (T::AccountId, T::AccountId),
        BlockNumberFor<T>,
        ValueQuery,
    >;

    /// User proposal statistics for honesty score
    /// Maps AccountId -> ProposalStats
    #[pallet::storage]
    #[pallet::getter(fn user_proposals)]
    pub type UserProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ProposalStats,
        ValueQuery,
    >;

    // ================================
    // Storage Items - Phase 2: Zero-Fee Protocol
    // ================================

    /// Fee exemption usage tracking (monthly limit)
    /// Maps AccountId -> FeeExemptionData
    #[pallet::storage]
    #[pallet::getter(fn fee_exemption_usage)]
    pub type FeeExemptionUsage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        FeeExemptionData<BlockNumberFor<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    // ================================
    // Storage Items - Phase 3: Community Fund
    // ================================

    /// Community proposal counter for unique IDs
    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub type ProposalCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Community proposals
    /// Maps ProposalId -> CommunityProposal
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type CommunityProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        CommunityProposal<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Votes on community proposals
    /// Maps (ProposalId, AccountId) -> Vote
    #[pallet::storage]
    #[pallet::getter(fn proposal_votes)]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32,
        Blake2_128Concat,
        T::AccountId,
        Vote,
        OptionQuery,
    >;

    // ================================
    // Storage Items - Phase 4: Ethics Filter
    // ================================

    /// Ethics filter configuration
    #[pallet::storage]
    #[pallet::getter(fn ethics_filter_config)]
    pub type EthicsFilterConfig<T: Config> = StorageValue<
        _,
        EthicsConfig<T::AccountId>,
        ValueQuery,
    >;

    /// Sanctioned accounts (cannot receive community funds)
    #[pallet::storage]
    #[pallet::getter(fn is_sanctioned)]
    pub type SanctionedAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        SanctionStatus,
        OptionQuery,
    >;

    /// Ethics council votes on proposals
    #[pallet::storage]
    #[pallet::getter(fn ethics_council_votes)]
    pub type EthicsCouncilVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // Proposal ID
        Blake2_128Concat,
        T::AccountId, // Council member
        bool, // Approve/reject
        OptionQuery,
    >;

    // ==================== Phase 5: Incentive Programs ====================

    /// Education modules for learn-to-earn
    #[pallet::storage]
    #[pallet::getter(fn education_modules)]
    pub type EducationModules<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Module ID
        EducationModule,
        OptionQuery,
    >;

    /// Track completed education modules per account
    #[pallet::storage]
    #[pallet::getter(fn completed_education)]
    pub type CompletedEducation<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32, // Module ID
        CompletionData,
        OptionQuery,
    >;

    /// Green/sustainability projects
    #[pallet::storage]
    #[pallet::getter(fn green_projects)]
    pub type GreenProjects<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Project ID
        GreenProject,
        OptionQuery,
    >;

    /// Track green project contributions per account
    #[pallet::storage]
    #[pallet::getter(fn green_contributions)]
    pub type GreenContributions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32, // Project ID
        u64, // Total amount contributed
        ValueQuery,
    >;

    /// Referral tracking data per account
    #[pallet::storage]
    #[pallet::getter(fn referral_data)]
    pub type ReferralData<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        crate::types::ReferralData,
        ValueQuery,
    >;

    /// Track if referral has been claimed (referee → referrer mapping)
    #[pallet::storage]
    #[pallet::getter(fn referral_claimed)]
    pub type ReferralClaimed<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Referee (new user)
        Blake2_128Concat,
        T::AccountId, // Referrer (existing user)
        bool,
        ValueQuery,
    >;

    // ================================
    // Events
    // ================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// SRS updated for an account
        SRSUpdated {
            account: T::AccountId,
            old_score: u32,
            new_score: u32,
            old_tier: u8,  // 0=Bronze, 1=Silver, 2=Gold, 3=Platinum, 4=Diamond
            new_tier: u8,
        },
        /// Participation activity recorded
        ParticipationRecorded {
            account: T::AccountId,
            activity_type: u8,  // Activity type code
            block_number: BlockNumberFor<T>,
        },
        /// Peer endorsed another account
        PeerEndorsed {
            endorser: T::AccountId,
            endorsee: T::AccountId,
            endorsement_type: u8,  // Endorsement type code
        },
        /// SRS privacy settings updated
        SRSPrivacyUpdated {
            account: T::AccountId,
            public_display: bool,
        },
        /// Fee exemption applied
        FeeExemptionApplied {
            account: T::AccountId,
            original_fee: BalanceOf<T>,
            discounted_fee: BalanceOf<T>,
            tier: u8,  // SRS tier code
        },
        /// Monthly fee limit reached
        FeeExemptionLimitReached {
            account: T::AccountId,
            used_amount: BalanceOf<T>,
            limit: BalanceOf<T>,
        },
        /// Fee exemption usage reset (new month)
        FeeExemptionReset {
            account: T::AccountId,
        },

        // === Phase 3: Community Fund Events ===
        
        /// Community proposal submitted
        ProposalSubmitted {
            proposal_id: u32,
            proposer: T::AccountId,
            proposal_type: u8,  // Proposal type code
            amount: BalanceOf<T>,
            deposit: BalanceOf<T>,
        },

        /// Vote cast on community proposal
        ProposalVoted {
            proposal_id: u32,
            voter: T::AccountId,
            approve: bool,
            weight: u32,
        },

        /// Community proposal finalized
        ProposalFinalized {
            proposal_id: u32,
            approved: bool,
            votes_for: u32,
            votes_against: u32,
        },

        /// Community proposal executed (funds disbursed)
        ProposalExecuted {
            proposal_id: u32,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// Community proposal rejected
        ProposalRejected {
            proposal_id: u32,
            reason: BoundedVec<u8, ConstU32<64>>,
        },

        /// Proposal deposit returned
        DepositReturned {
            proposal_id: u32,
            proposer: T::AccountId,
            amount: BalanceOf<T>,
        },

        // === Phase 4: Ethics Filter Events ===
        
        /// Proposal flagged for ethics review
        ProposalFlaggedForReview {
            proposal_id: u32,
            reason: BoundedVec<u8, ConstU32<128>>,
        },

        /// Ethics council voted on proposal
        EthicsCouncilVoted {
            proposal_id: u32,
            council_member: T::AccountId,
            approve: bool,
        },

        /// Ethics council decision finalized
        EthicsDecisionFinalized {
            proposal_id: u32,
            approved: bool,
            votes_for: u32,
            votes_against: u32,
        },

        /// Account sanctioned
        AccountSanctioned {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        },

        /// Account sanction lifted
        SanctionLifted {
            account: T::AccountId,
        },

        // === Phase 5: Incentive Programs Events ===

        /// Education module completed
        EducationModuleCompleted {
            account: T::AccountId,
            module_id: u32,
            reward_amount: u64,
        },

        /// Education reward claimed
        EducationRewardClaimed {
            account: T::AccountId,
            module_id: u32,
            amount: u64,
        },

        /// Contribution made to green project
        GreenContributionMade {
            account: T::AccountId,
            project_id: u32,
            amount: u64,
            total_contributed: u64,
        },

        /// Green project milestone reached
        GreenMilestoneReached {
            project_id: u32,
            milestone_amount: u64,
            total_contributors: u32,
        },

        /// Referral claimed
        ReferralClaimed {
            referee: T::AccountId,
            referrer: T::AccountId,
        },

        /// Referral reward paid
        ReferralRewardPaid {
            referrer: T::AccountId,
            amount: u64,
            total_referrals: u32,
        },
    }

    // ================================
    // Errors
    // ================================

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not verified via BelizeID
        NotVerified,
        /// SRS score is insufficient for this action
        InsufficientSRS,
        /// Cannot endorse self
        CannotEndorseSelf,
        /// Endorsement too frequent (must wait 1 month)
        EndorsementTooFrequent,
        /// Participation history is full
        ParticipationHistoryFull,
        /// Account has no SRS record
        NoSRSRecord,
        /// Monthly fee exemption limit exceeded
        FeeExemptionLimitExceeded,
        /// Insufficient funds to pay discounted fee
        InsufficientFundsForFee,
        /// Invalid activity type code provided
        InvalidActivityType,
        /// Invalid endorsement type code provided
        InvalidEndorsementType,
        /// Invalid proposal type code provided
        InvalidProposalType,

        // === Phase 3: Community Fund Errors ===
        
        /// Proposal not found
        ProposalNotFound,
        /// Already voted on this proposal
        AlreadyVoted,
        /// Voting period has ended
        VotingPeriodEnded,
        /// Insufficient deposit for proposal
        InsufficientDeposit,
        /// Proposal is not in correct status
        InvalidProposalStatus,
        /// Title exceeds maximum length
        TitleTooLong,
        /// Description exceeds maximum length
        DescriptionTooLong,

        // === Phase 4: Ethics Filter Errors ===
        
        /// Account is sanctioned
        AccountSanctioned,
        /// Beneficiary not verified
        BeneficiaryNotVerified,
        /// Proposer honesty rating too low
        LowHonestyRating,
        /// Ethics review required
        EthicsReviewRequired,
        /// Not an ethics council member
        NotEthicsCouncilMember,
        /// Already voted in ethics council
        AlreadyVotedInCouncil,

        // === Phase 5: Incentive Programs Errors ===

        /// Education module not found
        ModuleNotFound,
        /// Education module is inactive
        ModuleInactive,
        /// Education module already completed
        AlreadyCompleted,
        /// Education module has reached capacity
        ModuleCapReached,
        /// Green project not found
        ProjectNotFound,
        /// Green project is inactive
        ProjectInactive,
        /// Invalid referral (self-referral or non-existent referee)
        InvalidReferral,
        /// Referral already claimed
        ReferralAlreadyClaimed,
    }

    // ================================
    // Extrinsics - Phase 1: SRS System
    // ================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Record participation activity (called by other pallets or root)
        /// 
        /// Updates the participation history for an account and triggers SRS recalculation.
        /// 
        /// # Arguments
        /// * `origin` - Root or the account itself
        /// * `account` - The account to record participation for
        /// * `activity` - Type of activity being recorded
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::record_participation())]
        pub fn record_participation(
            origin: OriginFor<T>,
            account: T::AccountId,
            activity_code: u8,
        ) -> DispatchResult {
            // Allow both root (for system-triggered activities) and self-reporting
            // NOTE: In production, cross-pallet trait calls should be used for automatic recording
            // (e.g., governance pallet calls this when a vote is cast)
            let _ = ensure_signed(origin.clone()).or_else(|_| -> Result<T::AccountId, DispatchError> {
                ensure_root(origin)?;
                Ok(account.clone())
            })?;

            // Convert u8 code to ActivityType enum
            let activity = ActivityType::from_u8(activity_code)
                .ok_or(Error::<T>::InvalidActivityType)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // Verify account is BelizeID verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&account, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );
            
            let record = ParticipationRecord {
                activity_type: activity.clone(),
                block_number: current_block,
                value: 0, // Value for weighted activities (0 for simple boolean activities)
            };

            // Add to participation history
            ParticipationHistory::<T>::try_mutate(&account, |history| {
                history.try_push(record)
                    .map_err(|_| Error::<T>::ParticipationHistoryFull)?;
                Ok::<(), DispatchError>(())
            })?;

            // Update proposal stats if applicable
            match activity {
                ActivityType::ProposalSubmission => {
                    UserProposals::<T>::mutate(&account, |stats| {
                        stats.total = stats.total.saturating_add(1);
                    });
                },
                ActivityType::ProposalApproved => {
                    UserProposals::<T>::mutate(&account, |stats| {
                        stats.approved = stats.approved.saturating_add(1);
                    });
                },
                _ => {},
            }

            // Trigger SRS update
            Self::update_srs_internal(&account)?;

            Self::deposit_event(Event::ParticipationRecorded {
                account,
                activity_type: activity.as_u8(),
                block_number: current_block,
            });

            Ok(())
        }

        /// Update SRS for an account
        /// 
        /// Recalculates the complete SRS based on all participation factors.
        /// Can be called by anyone (permissionless) as calculation is deterministic.
        /// 
        /// # Arguments
        /// * `origin` - Any signed account
        /// * `account` - The account to update SRS for
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_srs())]
        pub fn update_srs(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Verify account is BelizeID verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&account, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );
            
            Self::update_srs_internal(&account)?;
            
            Ok(())
        }

        /// Endorse a peer for positive contributions
        /// 
        /// Allows verified citizens with Silver+ tier to endorse others.
        /// Limited to once per month per endorser-endorsee pair.
        /// 
        /// # Arguments
        /// * `origin` - Signed account (endorser)
        /// * `endorsee` - Account being endorsed
        /// * `endorsement_code` - Type of endorsement (u8 code)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::endorse_peer())]
        pub fn endorse_peer(
            origin: OriginFor<T>,
            endorsee: T::AccountId,
            endorsement_code: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Convert u8 code to EndorsementType enum
            let endorsement_type = EndorsementType::from_u8(endorsement_code)
                .ok_or(Error::<T>::InvalidEndorsementType)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();

            // Both must be verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&who, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );
            ensure!(
                T::BelizeKyc::is_kyc_verified(&endorsee, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );

            // Cannot endorse self
            ensure!(who != endorsee, Error::<T>::CannotEndorseSelf);

            // Check endorser's SRS (must be Silver+)
            let endorser_srs = Self::get_srs(&who).ok_or(Error::<T>::NoSRSRecord)?;
            ensure!(
                endorser_srs.tier >= SRSTier::Silver,
                Error::<T>::InsufficientSRS
            );

            // One endorsement per pair per month
            let key = (who.clone(), endorsee.clone());
            let last_endorsement = LastEndorsement::<T>::get(&key);
            let blocks_per_month: u32 = 6 * 24 * 30 * 10; // ~10s blocks = ~1 month
            
            // If last_endorsement is 0 (never endorsed), allow it
            if last_endorsement != BlockNumberFor::<T>::from(0u32) {
                let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
                let last_u64: u64 = TryInto::<u64>::try_into(last_endorsement).unwrap_or(0);
                let blocks_since: u32 = current_u64.saturating_sub(last_u64) as u32;
                ensure!(
                    blocks_since >= blocks_per_month,
                    Error::<T>::EndorsementTooFrequent
                );
            }

            // Record endorsement
            let endorsements = PeerEndorsements::<T>::get(&endorsee);
            PeerEndorsements::<T>::insert(&endorsee, endorsements.saturating_add(1));
            LastEndorsement::<T>::insert(&key, current_block);

            // Update endorsee's SRS
            Self::update_srs_internal(&endorsee)?;

            Self::deposit_event(Event::PeerEndorsed {
                endorser: who,
                endorsee,
                endorsement_type: endorsement_type.as_u8(),
            });

            Ok(())
        }

        /// Toggle SRS public display
        /// 
        /// Allows users to control whether their SRS is publicly visible
        /// or shown as an anonymous hash.
        /// 
        /// # Arguments
        /// * `origin` - Signed account
        /// * `public` - Whether to make SRS publicly visible
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_srs_privacy())]
        pub fn set_srs_privacy(
            origin: OriginFor<T>,
            public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_block = frame_system::Pallet::<T>::block_number();

            // Verify account is BelizeID verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&who, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );

            SocialResponsibilityScores::<T>::mutate(&who, |maybe_srs| {
                if let Some(ref mut srs) = maybe_srs {
                    srs.public_display = public;
                    
                    // Generate anonymous hash if not public
                    if !public {
                        let hash_input = (who.clone(), srs.score);
                        srs.anonymous_hash = Some(sp_io::hashing::blake2_256(&hash_input.encode()).into());
                    } else {
                        srs.anonymous_hash = None;
                    }
                }
            });

            Self::deposit_event(Event::SRSPrivacyUpdated {
                account: who,
                public_display: public,
            });

            Ok(())
        }

        // ================================
        // Phase 3: Community Fund Management
        // ================================

        /// Submit a community proposal
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::submit_community_proposal())]
        pub fn submit_community_proposal(
            origin: OriginFor<T>,
            proposal_type_code: u8,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            title: BoundedVec<u8, ConstU32<128>>,
            description: BoundedVec<u8, ConstU32<1024>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Convert u8 code to CommunityProposalType enum
            let proposal_type = CommunityProposalType::from_u8(proposal_type_code)
                .ok_or(Error::<T>::InvalidProposalType)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // Verify account is BelizeID verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&who, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );

            // Calculate 10% deposit
            let deposit = amount / 10u32.into();
            
            // Reserve deposit from proposer
            T::Currency::reserve(&who, deposit)?;

            // Get next proposal ID
            let proposal_id = ProposalCount::<T>::get();
            ProposalCount::<T>::put(proposal_id.saturating_add(1));

            // Calculate voting deadline (7 days = ~100,800 blocks at 6s/block)
            let voting_deadline = current_block + 100_800u32.into();

            // Run ethics filter check
            let ethics_passed = Self::check_ethics_filter(&who, &beneficiary, &proposal_type)?;

            // Determine initial status
            let initial_status = if ethics_passed {
                ProposalStatus::Active
            } else {
                // Flag for ethics review
                let reason: BoundedVec<u8, ConstU32<128>> = b"Requires ethics council review"
                    .to_vec()
                    .try_into()
                    .unwrap_or_default();
                Self::deposit_event(Event::ProposalFlaggedForReview {
                    proposal_id,
                    reason,
                });
                ProposalStatus::EthicsReview
            };

            // Create proposal
            let proposal = CommunityProposal {
                proposer: who.clone(),
                proposal_type: proposal_type.clone(),
                beneficiary: beneficiary.clone(),
                amount,
                title: title.clone(),
                description,
                deposit,
                status: initial_status,
                submission_block: current_block,
                voting_deadline,
                votes_for: 0,
                votes_against: 0,
                total_votes: 0,
            };

            CommunityProposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer: who,
                proposal_type: proposal_type.as_u8(),
                amount,
                deposit,
            });

            Ok(())
        }

        /// Vote on a community proposal (SRS-weighted)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::vote_community_proposal())]
        pub fn vote_community_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // Verify account is BelizeID verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(&who, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::NotVerified
            );

            // Get proposal
            let mut proposal = CommunityProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Check proposal is active
            ensure!(proposal.status == ProposalStatus::Active, Error::<T>::InvalidProposalStatus);

            // Check voting period hasn't ended
            ensure!(current_block <= proposal.voting_deadline, Error::<T>::VotingPeriodEnded);

            // Check hasn't already voted
            ensure!(
                !ProposalVotes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            // Get SRS weight (default to 1 if no SRS)
            let weight = if let Some(srs) = Self::get_srs(&who) {
                srs.score
            } else {
                1
            };

            // Record vote
            let vote = Vote { approve, weight };
            ProposalVotes::<T>::insert(proposal_id, &who, vote);

            // Update proposal vote counts
            if approve {
                proposal.votes_for = proposal.votes_for.saturating_add(weight);
            } else {
                proposal.votes_against = proposal.votes_against.saturating_add(weight);
            }
            proposal.total_votes = proposal.total_votes.saturating_add(1);

            CommunityProposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::ProposalVoted {
                proposal_id,
                voter: who,
                approve,
                weight,
            });

            Ok(())
        }

        /// Finalize a community proposal (anyone can call after deadline)
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::finalize_community_proposal())]
        pub fn finalize_community_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Get proposal
            let mut proposal = CommunityProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Check proposal is active
            ensure!(proposal.status == ProposalStatus::Active, Error::<T>::InvalidProposalStatus);

            // Check voting period has ended
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block > proposal.voting_deadline, Error::<T>::VotingPeriodEnded);

            // Determine if approved (simple majority of weighted votes)
            let approved = proposal.votes_for > proposal.votes_against;

            if approved {
                // Transfer funds from community treasury to beneficiary
                proposal.status = ProposalStatus::Approved;

                // Execute treasury transfer
                let treasury_account = T::CommunityTreasuryAccount::get();
                let transfer_result = T::Currency::transfer(
                    &treasury_account,
                    &proposal.beneficiary,
                    proposal.amount,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                );

                match transfer_result {
                    Ok(()) => {
                        // Unreserve deposit on successful execution
                        T::Currency::unreserve(&proposal.proposer, proposal.deposit);

                        Self::deposit_event(Event::ProposalExecuted {
                            proposal_id,
                            beneficiary: proposal.beneficiary.clone(),
                            amount: proposal.amount,
                        });

                        Self::deposit_event(Event::DepositReturned {
                            proposal_id,
                            proposer: proposal.proposer.clone(),
                            amount: proposal.deposit,
                        });
                    }
                    Err(_) => {
                        // Treasury transfer failed - mark as rejected and slash deposit
                        proposal.status = ProposalStatus::Rejected;
                        let _ = T::Currency::slash_reserved(&proposal.proposer, proposal.deposit);
                        
                        let reason: BoundedVec<u8, ConstU32<64>> = b"Treasury transfer failed"
                            .to_vec()
                            .try_into()
                            .unwrap_or_default();
                        Self::deposit_event(Event::ProposalRejected {
                            proposal_id,
                            reason,
                        });
                    }
                }
            } else {
                // Reject proposal and slash deposit
                proposal.status = ProposalStatus::Rejected;

                // Slash deposit (goes to treasury in production)
                let _ = T::Currency::slash_reserved(&proposal.proposer, proposal.deposit);

                let reason: BoundedVec<u8, ConstU32<64>> = b"Failed to meet majority threshold"
                    .to_vec()
                    .try_into()
                    .unwrap_or_default();
                Self::deposit_event(Event::ProposalRejected {
                    proposal_id,
                    reason,
                });
            }

            // Save vote counts before moving proposal
            let votes_for = proposal.votes_for;
            let votes_against = proposal.votes_against;

            CommunityProposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::ProposalFinalized {
                proposal_id,
                approved,
                votes_for,
                votes_against,
            });

            Ok(())
        }

        // ================================
        // Phase 4: Ethics Filter & Sanctioning
        // ================================

        /// Sanction an account (governance/council only)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::sanction_account())]
        pub fn sanction_account(
            origin: OriginFor<T>,
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        ) -> DispatchResult {
            // Only governance or ethics council can sanction
            T::GovernanceOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let block_u32: u32 = current_block.saturated_into();

            let sanction = SanctionStatus {
                active: true,
                reason: reason.clone(),
                sanctioned_at: block_u32,
            };

            SanctionedAccounts::<T>::insert(&account, sanction);

            Self::deposit_event(Event::AccountSanctioned {
                account,
                reason,
            });

            Ok(())
        }

        /// Lift sanction from an account (governance/council only)
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::lift_sanction())]
        pub fn lift_sanction(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            SanctionedAccounts::<T>::remove(&account);

            Self::deposit_event(Event::SanctionLifted {
                account,
            });

            Ok(())
        }

        /// Ethics council vote on proposal under review
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::ethics_council_vote())]
        pub fn ethics_council_vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if caller is ethics council member
            let config = EthicsFilterConfig::<T>::get();
            ensure!(
                config.council_members.contains(&who),
                Error::<T>::NotEthicsCouncilMember
            );

            // Get proposal
            let proposal = CommunityProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Only vote on proposals in EthicsReview status
            ensure!(
                proposal.status == ProposalStatus::EthicsReview,
                Error::<T>::InvalidProposalStatus
            );

            // Check not already voted
            ensure!(
                !EthicsCouncilVotes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVotedInCouncil
            );

            // Record vote
            EthicsCouncilVotes::<T>::insert(proposal_id, &who, approve);

            Self::deposit_event(Event::EthicsCouncilVoted {
                proposal_id,
                council_member: who,
                approve,
            });

            // Check if enough votes to finalize
            Self::try_finalize_ethics_review(proposal_id)?;

            Ok(())
        }

        // ================================
        // Phase 5: Incentive Programs
        // ================================

        /// Complete an education module
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::complete_education_module())]
        pub fn complete_education_module(
            origin: OriginFor<T>,
            module_id: u32,
            _completion_proof: BoundedVec<u8, ConstU32<256>>, // Future: verify proof
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get module
            let mut module = EducationModules::<T>::get(module_id)
                .ok_or(Error::<T>::ModuleNotFound)?;

            // Check if module is active
            ensure!(module.active, Error::<T>::ModuleInactive);

            // Check if already completed
            ensure!(
                !CompletedEducation::<T>::contains_key(&who, module_id),
                Error::<T>::AlreadyCompleted
            );

            // Check capacity
            if let Some(max) = module.max_completions {
                ensure!(module.total_completions < max, Error::<T>::ModuleCapReached);
            }

            // Record completion
            let current_block = frame_system::Pallet::<T>::block_number();
            let block_u32: u32 = current_block.saturated_into();

            let completion = CompletionData {
                completed_at: block_u32,
                reward_claimed: true, // Auto-claim on completion
            };

            CompletedEducation::<T>::insert(&who, module_id, completion);

            // Update module completion count
            module.total_completions = module.total_completions.saturating_add(1);
            EducationModules::<T>::insert(module_id, module.clone());

            // Update SRS - education completion increases score
            Self::update_srs_after_education(&who);

            Self::deposit_event(Event::EducationModuleCompleted {
                account: who.clone(),
                module_id,
                reward_amount: module.reward_amount,
            });

            Self::deposit_event(Event::EducationRewardClaimed {
                account: who,
                module_id,
                amount: module.reward_amount,
            });

            Ok(())
        }

        /// Contribute to a green/sustainability project
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::contribute_to_green_project())]
        pub fn contribute_to_green_project(
            origin: OriginFor<T>,
            project_id: u32,
            amount: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get project
            let mut project = GreenProjects::<T>::get(project_id)
                .ok_or(Error::<T>::ProjectNotFound)?;

            // Check if project is active
            ensure!(project.active, Error::<T>::ProjectInactive);

            // Update contribution tracking
            let current_contribution = GreenContributions::<T>::get(&who, project_id);
            let new_contribution = current_contribution.saturating_add(amount);
            GreenContributions::<T>::insert(&who, project_id, new_contribution);

            // Update project totals
            let old_amount = project.amount_contributed;
            project.amount_contributed = project.amount_contributed.saturating_add(amount);
            
            // Increment contributor count if this is first contribution
            if current_contribution == 0 {
                project.total_contributors = project.total_contributors.saturating_add(1);
            }

            GreenProjects::<T>::insert(project_id, project.clone());

            // Update SRS - green contributions increase sustainability score
            Self::update_srs_after_green_contribution(&who);

            Self::deposit_event(Event::GreenContributionMade {
                account: who,
                project_id,
                amount,
                total_contributed: new_contribution,
            });

            // Check for milestones (every 100,000 units)
            let old_milestone = old_amount / 100_000;
            let new_milestone = project.amount_contributed / 100_000;
            
            if new_milestone > old_milestone {
                Self::deposit_event(Event::GreenMilestoneReached {
                    project_id,
                    milestone_amount: project.amount_contributed,
                    total_contributors: project.total_contributors,
                });
            }

            Ok(())
        }

        /// Claim referral reward (one-time per referee)
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::claim_referral_reward())]
        pub fn claim_referral_reward(
            origin: OriginFor<T>,
            referee: T::AccountId, // The new user who was referred
        ) -> DispatchResult {
            let referrer = ensure_signed(origin)?;

            // Prevent self-referral
            ensure!(referrer != referee, Error::<T>::InvalidReferral);

            // Check if already claimed
            ensure!(
                !ReferralClaimed::<T>::get(&referee, &referrer),
                Error::<T>::ReferralAlreadyClaimed
            );

            // Verify referee exists (has some SRS data)
            ensure!(
                SocialResponsibilityScores::<T>::contains_key(&referee),
                Error::<T>::InvalidReferral
            );

            // Mark as claimed
            ReferralClaimed::<T>::insert(&referee, &referrer, true);

            // Update referrer data
            let mut referral_data = ReferralData::<T>::get(&referrer);
            referral_data.total_referrals = referral_data.total_referrals.saturating_add(1);
            
            // Reward calculation: 1000 base + 100 per existing referral
            let reward = 1000u64.saturating_add(
                (referral_data.total_referrals.saturating_sub(1) as u64).saturating_mul(100)
            );
            
            referral_data.total_rewards_earned = referral_data.total_rewards_earned.saturating_add(reward);
            ReferralData::<T>::insert(&referrer, referral_data.clone());

            Self::deposit_event(Event::ReferralClaimed {
                referee: referee.clone(),
                referrer: referrer.clone(),
            });

            Self::deposit_event(Event::ReferralRewardPaid {
                referrer,
                amount: reward,
                total_referrals: referral_data.total_referrals,
            });

            Ok(())
        }
    }

    // ================================
    // Internal Functions - SRS Calculation
    // ================================

    impl<T: Config> Pallet<T> {
        /// Get SRS data for an account
        pub fn get_srs(account: &T::AccountId) -> Option<SRSData<BlockNumberFor<T>>> {
            SocialResponsibilityScores::<T>::get(account)
        }

        /// Internal SRS update with event emission
        fn update_srs_internal(account: &T::AccountId) -> DispatchResult {
            let old_srs = Self::get_srs(account);
            let old_score = old_srs.as_ref().map(|s| s.score).unwrap_or(0);
            let old_tier = old_srs.as_ref().map(|s| s.tier.clone()).unwrap_or(SRSTier::Bronze);

            let new_score = Self::calculate_srs(account);
            let new_tier = Self::score_to_tier(new_score);
            
            let current_block = frame_system::Pallet::<T>::block_number();

            let governance_score = Self::calculate_governance_score(account);
            let education_score = Self::calculate_education_score(account);
            let sustainability_score = Self::calculate_sustainability_score(account);
            let participation_score = Self::calculate_participation_score(account);
            let peer_endorsements = Self::calculate_endorsement_score(account);
            let honesty_rating = Self::calculate_honesty_score(account);

            let srs_data = SRSData {
                score: new_score,
                governance_score,
                education_score,
                sustainability_score,
                participation_score,
                peer_endorsements,
                honesty_rating,
                last_updated: current_block,
                tier: new_tier.clone(),
                total_contributions: ParticipationHistory::<T>::get(account).len() as u32,
                public_display: old_srs.as_ref().map(|s| s.public_display).unwrap_or(true),
                anonymous_hash: old_srs.and_then(|s| s.anonymous_hash),
            };

            SocialResponsibilityScores::<T>::insert(account, srs_data);

            Self::deposit_event(Event::SRSUpdated {
                account: account.clone(),
                old_score,
                new_score,
                old_tier: old_tier.as_u8(),
                new_tier: new_tier.as_u8(),
            });

            Ok(())
        }

        /// Calculate total SRS from all participation factors
        pub fn calculate_srs(account: &T::AccountId) -> u32 {
            let governance = Self::calculate_governance_score(account);
            let education = Self::calculate_education_score(account);
            let sustainability = Self::calculate_sustainability_score(account);
            let participation = Self::calculate_participation_score(account);
            let endorsements = Self::calculate_endorsement_score(account);
            let honesty = Self::calculate_honesty_score(account);

            governance
                .saturating_add(education)
                .saturating_add(sustainability)
                .saturating_add(participation)
                .saturating_add(endorsements)
                .saturating_add(honesty)
                .min(10_000)
        }

        /// Time-weighted governance participation (0-2,500)
        fn calculate_governance_score(account: &T::AccountId) -> u32 {
            let history = ParticipationHistory::<T>::get(account);
            let current_block = frame_system::Pallet::<T>::block_number();

            let mut score = 0u32;
            for record in history.iter() {
                let base_points = match record.activity_type {
                    ActivityType::ProposalSubmission => 100,
                    ActivityType::VoteCast => 25,
                    ActivityType::ProposalApproved => 200,
                    ActivityType::CouncilMembership => 500,
                    _ => 0,
                };

                score = score.saturating_add(base_points);

                // Time decay: 10% bonus per 6 months of sustained activity
                let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
                let record_u64: u64 = TryInto::<u64>::try_into(record.block_number).unwrap_or(0);
                let blocks_ago_u32: u32 = current_u64.saturating_sub(record_u64) as u32;
                let blocks_per_6_months: u32 = 6 * 24 * 30 * 10 * 6; // ~10s blocks = ~1 month
                let months_6: u32 = blocks_ago_u32 / blocks_per_6_months;
                
                if months_6 >= 1u32 {
                    score = score.saturating_add(base_points / 10); // 10% bonus after 6 months
                }
            }

            score.min(2_500)
        }

        /// General participation (0-2,500)
        fn calculate_participation_score(account: &T::AccountId) -> u32 {
            let history = ParticipationHistory::<T>::get(account);
            (history.len() as u32 * 50).min(2_500)
        }

        /// Peer endorsements (0-1,000)
        fn calculate_endorsement_score(account: &T::AccountId) -> u32 {
            let endorsements = PeerEndorsements::<T>::get(account);
            (endorsements * 10).min(1_000)
        }

        /// Honesty rating from proposal outcomes (0-1,000)
        fn calculate_honesty_score(account: &T::AccountId) -> u32 {
            let proposals: ProposalStats = UserProposals::<T>::get(account);
            if proposals.total == 0u32 {
                return 500; // Neutral start
            }

            let success_rate = (proposals.approved * 1000u32) / proposals.total;
            success_rate.min(1_000)
        }

        /// Education score from completed modules (0-2,000) - Phase 5
        fn calculate_education_score(account: &T::AccountId) -> u32 {
            let completed_count = CompletedEducation::<T>::iter_prefix(account).count() as u32;
            
            // Base points: 100 per module
            let base_score = completed_count.saturating_mul(100);
            
            // Bonus for consistent learning (5+ modules = +500, 10+ = +1000)
            let bonus = match completed_count {
                0..=4 => 0,
                5..=9 => 500,
                _ => 1_000,
            };
            
            base_score.saturating_add(bonus).min(2_000)
        }

        /// Sustainability score from green contributions (0-1,500) - Phase 5
        fn calculate_sustainability_score(account: &T::AccountId) -> u32 {
            let contributions = GreenContributions::<T>::iter_prefix(account);
            
            let mut total_contributed = 0u64;
            let mut project_count = 0u32;
            
            for (_project_id, amount) in contributions {
                total_contributed = total_contributed.saturating_add(amount);
                project_count = project_count.saturating_add(1);
            }
            
            // Base score: 1 point per 100 units contributed
            let amount_score = (total_contributed / 100) as u32;
            
            // Diversity bonus: 100 points per unique project
            let diversity_bonus = project_count.saturating_mul(100);
            
            amount_score.saturating_add(diversity_bonus).min(1_500)
        }

        /// Convert score to tier
        fn score_to_tier(score: u32) -> SRSTier {
            match score {
                0..=2_499 => SRSTier::Bronze,
                2_500..=4_999 => SRSTier::Silver,
                5_000..=7_499 => SRSTier::Gold,
                7_500..=9_999 => SRSTier::Platinum,
                _ => SRSTier::Diamond,
            }
        }

        // ================================
        // Phase 2: Fee Calculation Helpers
        // ================================

        /// Calculate effective fee after SRS-based discount
        /// Returns (discounted_fee, discount_percentage)
        pub fn calculate_fee_discount(account: &T::AccountId, original_fee: BalanceOf<T>) -> (BalanceOf<T>, u32) {
            // Get SRS tier
            let tier = match SocialResponsibilityScores::<T>::get(account) {
                Some(srs) => srs.tier,
                None => return (original_fee, 0), // No discount without SRS
            };

            // Tier-based discount percentages
            let discount_percentage = match tier {
                SRSTier::Bronze => 0,    // 0% discount
                SRSTier::Silver => 25,   // 25% discount
                SRSTier::Gold => 50,     // 50% discount
                SRSTier::Platinum => 75, // 75% discount
                SRSTier::Diamond => 90,  // 90% discount
            };

            if discount_percentage == 0 {
                return (original_fee, 0);
            }

            // Calculate discounted fee
            let discount = original_fee * discount_percentage.into() / 100u32.into();
            let discounted_fee = if original_fee > discount {
                original_fee - discount
            } else {
                original_fee // Should never happen, but return original if calculation error
            };

            (discounted_fee, discount_percentage)
        }

        /// Check if account can use fee exemption (within monthly limit)
        /// Returns true if within limit, false if exceeded
        pub fn check_fee_exemption_limit(account: &T::AccountId, exemption_amount: BalanceOf<T>) -> bool {
            let current_block = frame_system::Pallet::<T>::block_number();
            let monthly_limit: BalanceOf<T> = T::FeeExemptionMonthlyLimit::get().into();

            // Get or create fee exemption data
            let mut fee_data = FeeExemptionUsage::<T>::get(account).unwrap_or_else(|| {
                FeeExemptionData {
                    used_this_month: 0u32.into(),
                    last_reset_block: current_block,
                }
            });

            // Check if month has passed (30 days = ~30 * 24 * 60 * 10 blocks)
            let blocks_per_month: u32 = 30 * 24 * 60 * 10;
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let reset_u64: u64 = TryInto::<u64>::try_into(fee_data.last_reset_block).unwrap_or(0);
            let blocks_since_reset: u32 = current_u64.saturating_sub(reset_u64) as u32;

            if blocks_since_reset >= blocks_per_month {
                // Reset monthly usage
                fee_data.used_this_month = 0u32.into();
                fee_data.last_reset_block = current_block;
                FeeExemptionUsage::<T>::insert(account, fee_data.clone());
                
                Self::deposit_event(Event::FeeExemptionReset {
                    account: account.clone(),
                });
            }

            // Check if adding this exemption would exceed limit
            match fee_data.used_this_month.checked_add(&exemption_amount) {
                Some(new_total) => new_total <= monthly_limit,
                None => false, // Overflow means exceeded limit
            }
        }

        /// Apply fee exemption and update usage tracking
        pub fn apply_fee_exemption(
            account: &T::AccountId,
            original_fee: BalanceOf<T>,
            discounted_fee: BalanceOf<T>,
        ) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let exemption_amount = if original_fee > discounted_fee {
                original_fee - discounted_fee
            } else {
                return Ok(()); // No exemption to apply
            };

            // Get or create fee exemption data
            let mut fee_data = FeeExemptionUsage::<T>::get(account).unwrap_or_else(|| {
                FeeExemptionData {
                    used_this_month: 0u32.into(),
                    last_reset_block: current_block,
                }
            });

            // Update usage
            fee_data.used_this_month = fee_data.used_this_month.checked_add(&exemption_amount).unwrap_or(fee_data.used_this_month);
            FeeExemptionUsage::<T>::insert(account, fee_data);

            Ok(())
        }

        // ================================
        // Phase 4: Ethics Filter Functions
        // ================================

        /// Check if proposal passes ethics filter
        pub fn check_ethics_filter(
            proposer: &T::AccountId,
            beneficiary: &T::AccountId,
            _proposal_type: &CommunityProposalType,
        ) -> Result<bool, DispatchError> {
            let config = EthicsFilterConfig::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();

            // Check if beneficiary is sanctioned
            if let Some(sanction) = SanctionedAccounts::<T>::get(beneficiary) {
                if sanction.active {
                    return Ok(false); // Requires council override
                }
            }

            // Check if beneficiary is verified
            ensure!(
                T::BelizeKyc::is_kyc_verified(beneficiary, pallet_belize_identity::KycLevel::L0, current_block),
                Error::<T>::BeneficiaryNotVerified
            );

            // Check proposer's honesty rating
            if let Some(_srs) = Self::get_srs(proposer) {
                let honesty = Self::calculate_honesty_score(proposer);
                if honesty < config.min_honesty_rating {
                    return Ok(false); // Low honesty, requires review
                }
            }

            // All checks passed
            Ok(true)
        }

        /// Try to finalize ethics review if enough votes
        fn try_finalize_ethics_review(proposal_id: u32) -> DispatchResult {
            let config = EthicsFilterConfig::<T>::get();
            let mut proposal = CommunityProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // Count votes
            let mut votes_for = 0u32;
            let mut votes_against = 0u32;
            let mut total_votes = 0u32;

            for member in config.council_members.iter() {
                if let Some(vote) = EthicsCouncilVotes::<T>::get(proposal_id, member) {
                    total_votes += 1;
                    if vote {
                        votes_for += 1;
                    } else {
                        votes_against += 1;
                    }
                }
            }

            // Check if we have minimum required votes
            if total_votes >= config.min_council_votes {
                // Determine outcome
                let approved = votes_for > votes_against;

                if approved {
                    // Move to Active status for community voting
                    proposal.status = ProposalStatus::Active;
                    CommunityProposals::<T>::insert(proposal_id, proposal);
                } else {
                    // Reject and return deposit
                    proposal.status = ProposalStatus::Rejected;
                    T::Currency::unreserve(&proposal.proposer, proposal.deposit);
                    CommunityProposals::<T>::insert(proposal_id, proposal);
                }

                Self::deposit_event(Event::EthicsDecisionFinalized {
                    proposal_id,
                    approved,
                    votes_for,
                    votes_against,
                });
            }

            Ok(())
        }

        // ================================
        // Phase 5: Incentive Helper Functions
        // ================================

        /// Update SRS after completing education module
        pub fn update_srs_after_education(account: &T::AccountId) {
            // Recalculate SRS with new education score
            let old_srs = Self::get_srs(account);
            let old_score = old_srs.as_ref().map(|s| s.score).unwrap_or(0);
            let old_tier = old_srs.as_ref().map(|s| s.tier.clone()).unwrap_or(SRSTier::Bronze);

            let new_score = Self::calculate_srs(account);
            let new_tier = Self::score_to_tier(new_score);
            
            let current_block = frame_system::Pallet::<T>::block_number();

            let governance_score = Self::calculate_governance_score(account);
            let education_score = Self::calculate_education_score(account);
            let sustainability_score = Self::calculate_sustainability_score(account);
            let participation_score = Self::calculate_participation_score(account);
            let peer_endorsements = Self::calculate_endorsement_score(account);
            let honesty_rating = Self::calculate_honesty_score(account);

            let srs_data = SRSData {
                score: new_score,
                governance_score,
                education_score,
                sustainability_score,
                participation_score,
                peer_endorsements,
                honesty_rating,
                last_updated: current_block,
                tier: new_tier.clone(),
                total_contributions: ParticipationHistory::<T>::get(account).len() as u32,
                public_display: old_srs.as_ref().map(|s| s.public_display).unwrap_or(true),
                anonymous_hash: old_srs.and_then(|s| s.anonymous_hash),
            };

            SocialResponsibilityScores::<T>::insert(account, srs_data);

            Self::deposit_event(Event::SRSUpdated {
                account: account.clone(),
                old_score,
                new_score,
                old_tier: old_tier.as_u8(),
                new_tier: new_tier.as_u8(),
            });
        }

        /// Update SRS after green project contribution
        fn update_srs_after_green_contribution(account: &T::AccountId) {
            // Same as education update - full recalculation
            Self::update_srs_after_education(account);
        }
    }

    // ================================
    // Genesis Configuration (Phase 5)
    // ================================

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    #[allow(clippy::type_complexity)]
    pub struct GenesisConfig<T: Config> {
        /// Initial education modules (Phase 5)
        pub education_modules: Vec<(
            u32,                                    // module_id
            Vec<u8>,                                // title
            Vec<u8>,                                // description
            BalanceOf<T>,                           // reward_amount
            u32,                                    // capacity
            bool,                                   // is_active
        )>,
        /// Initial green projects (Phase 5)
        pub green_projects: Vec<(
            u32,                                    // project_id
            Vec<u8>,                                // title
            Vec<u8>,                                // description
            BalanceOf<T>,                           // funding_goal
            BalanceOf<T>,                           // current_funding
            bool,                                   // is_active
        )>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            use frame_support::BoundedVec;
            use sp_core::ConstU32;
            use crate::types::{EducationModule, GreenProject, ProjectType};
            
            // Initialize education modules
            for (module_id, title, description, reward_amount, capacity, is_active) in &self.education_modules {
                let bounded_title: BoundedVec<u8, ConstU32<128>> = title.clone()
                    .try_into()
                    .expect("Title too long for BoundedVec<128>");
                let bounded_description: BoundedVec<u8, ConstU32<256>> = description.clone()
                    .try_into()
                    .expect("Description too long for BoundedVec<256>");
                
                // Convert BalanceOf<T> to u64 for storage
                let reward_u64: u64 = (*reward_amount).try_into()
                    .unwrap_or(0u64);
                
                let module = EducationModule {
                    id: *module_id,
                    title: bounded_title,
                    description: bounded_description,
                    reward_amount: reward_u64,
                    total_completions: 0,
                    max_completions: Some(*capacity),
                    active: *is_active,
                };
                EducationModules::<T>::insert(module_id, module);
            }
            
            // Initialize green projects
            for (project_id, title, _description, funding_goal, current_funding, is_active) in &self.green_projects {
                let bounded_title: BoundedVec<u8, ConstU32<128>> = title.clone()
                    .try_into()
                    .expect("Title too long for BoundedVec<128>");
                
                // Convert BalanceOf<T> to u64 for storage
                let _goal_u64: u64 = (*funding_goal).try_into()
                    .unwrap_or(0u64);
                let current_u64: u64 = (*current_funding).try_into()
                    .unwrap_or(0u64);
                
                let project = GreenProject {
                    id: *project_id,
                    title: bounded_title,
                    project_type: ProjectType::CleanEnergy, // Default type
                    amount_contributed: current_u64,
                    total_contributors: 0,
                    active: *is_active,
                };
                GreenProjects::<T>::insert(project_id, project);
            }
        }
    }
}

// ================================
// Exported Traits for Integration
// ================================

/// Trait for exporting community rank to governance pallet
pub trait CommunityRank<AccountId> {
    fn get_community_rank(account: &AccountId) -> u32;
    fn get_srs_tier(account: &AccountId) -> SRSTier;
}

impl<T: pallet::Config> CommunityRank<T::AccountId> for pallet::Pallet<T> {
    fn get_community_rank(account: &T::AccountId) -> u32 {
        let srs = Self::get_srs(account);
        match srs {
            Some(data) => match data.tier {
                SRSTier::Bronze => 100,
                SRSTier::Silver => 200,
                SRSTier::Gold => 400,
                SRSTier::Platinum => 700,
                SRSTier::Diamond => 1000,
            },
            None => 0, // No SRS record
        }
    }

    fn get_srs_tier(account: &T::AccountId) -> SRSTier {
        Self::get_srs(account)
            .map(|data| data.tier)
            .unwrap_or(SRSTier::Bronze)
    }
}

/// Trait for calculating fees with SRS-based discounts (exported to economy pallet)
pub trait FeeCalculator<AccountId, Balance> {
    /// Calculate effective fee after applying SRS tier discount
    /// Returns (discounted_fee, discount_percentage, within_limit)
    fn calculate_effective_fee(account: &AccountId, original_fee: Balance) -> (Balance, u32, bool);
    
    /// Apply fee discount and track usage
    fn apply_fee_discount(account: &AccountId, original_fee: Balance, discounted_fee: Balance) -> Result<(), &'static str>;
}

impl<T: Config> FeeCalculator<T::AccountId, BalanceOf<T>> for Pallet<T> {
    fn calculate_effective_fee(account: &T::AccountId, original_fee: BalanceOf<T>) -> (BalanceOf<T>, u32, bool) {
        let (discounted_fee, discount_percentage) = Self::calculate_fee_discount(account, original_fee);
        
        if discount_percentage == 0 {
            // No discount, so no limit check needed
            return (original_fee, 0, true);
        }

        // Calculate exemption amount by comparison
        let within_limit = if original_fee > discounted_fee {
            let exemption_amount = original_fee - discounted_fee;
            Self::check_fee_exemption_limit(account, exemption_amount)
        } else {
            true // No exemption needed
        };

        if within_limit {
            (discounted_fee, discount_percentage, true)
        } else {
            // Exceeded limit, charge full fee
            (original_fee, 0, false)
        }
    }

    fn apply_fee_discount(
        account: &T::AccountId,
        original_fee: BalanceOf<T>,
        discounted_fee: BalanceOf<T>,
    ) -> Result<(), &'static str> {
        Self::apply_fee_exemption(account, original_fee, discounted_fee)
            .map_err(|_| "Failed to apply fee exemption")
    }
}

// ================================
// Phase 6: Cross-Pallet Integration Traits
// ================================

/// Trait for staking pallet to contribute Proof of Useful Work (PoUW) scores to SRS
pub trait PoUWContributor<AccountId> {
    /// Record PoUW contribution from federated learning or other useful work
    /// quality_score: 0-10,000 (percentage-based, 10,000 = 100%)
    /// timeliness_score: 0-10,000
    /// honesty_score: 0-10,000
    fn record_pouw_contribution(
        account: &AccountId,
        quality_score: u32,
        timeliness_score: u32,
        honesty_score: u32,
    ) -> Result<(), &'static str>;
    
    /// Get current PoUW contribution for an account
    fn get_pouw_score(account: &AccountId) -> u32;
}

impl<T: Config> PoUWContributor<T::AccountId> for Pallet<T> {
    fn record_pouw_contribution(
        account: &T::AccountId,
        quality_score: u32,
        timeliness_score: u32,
        honesty_score: u32,
    ) -> Result<(), &'static str> {
        // Calculate weighted PoUW score: Quality (40%) + Timeliness (30%) + Honesty (30%)
        let weighted_score = (quality_score * 40 / 100)
            .saturating_add(timeliness_score * 30 / 100)
            .saturating_add(honesty_score * 30 / 100);
        
        // Record as participation activity
        let current_block = frame_system::Pallet::<T>::block_number();
        
        let record = ParticipationRecord {
            activity_type: ActivityType::PoUWContribution(weighted_score),
            block_number: current_block,
            value: weighted_score,
        };
        
        // Add to participation history
        ParticipationHistory::<T>::try_mutate(account, |history| {
            history.try_push(record)
                .map_err(|_| "Participation history full")?;
            Ok::<(), &'static str>(())
        })?;
        
        // Note: SRS will be recalculated next time it's queried or updated
        
        Ok(())
    }
    
    fn get_pouw_score(account: &T::AccountId) -> u32 {
        // Extract PoUW-specific participation records
        let history = ParticipationHistory::<T>::get(account);
        let pouw_records: Vec<_> = history
            .iter()
            .filter(|r| matches!(r.activity_type, ActivityType::PoUWContribution(_)))
            .collect();
        
        if pouw_records.is_empty() {
            return 0;
        }
        
        // Average of recent PoUW contributions (up to last 10)
        let recent: Vec<_> = pouw_records.iter().rev().take(10).collect();
        let total: u32 = recent.iter().map(|r| r.value).sum();
        total / (recent.len() as u32)
    }
}

/// Trait for governance pallet to record participation in proposals and voting
pub trait GovernanceParticipation<AccountId> {
    /// Record proposal submission in governance
    fn record_proposal_submission(account: &AccountId) -> Result<(), &'static str>;
    
    /// Record vote cast in governance
    fn record_vote_cast(account: &AccountId) -> Result<(), &'static str>;
    
    /// Record proposal approval (proposal passed)
    fn record_proposal_approval(account: &AccountId) -> Result<(), &'static str>;
    
    /// Record council membership activity
    fn record_council_activity(account: &AccountId) -> Result<(), &'static str>;
}

impl<T: Config> GovernanceParticipation<T::AccountId> for Pallet<T> {
    fn record_proposal_submission(account: &T::AccountId) -> Result<(), &'static str> {
        let current_block = frame_system::Pallet::<T>::block_number();
        
        let record = ParticipationRecord {
            activity_type: ActivityType::ProposalSubmission,
            block_number: current_block,
            value: 100, // Base score for proposal submission
        };
        
        ParticipationHistory::<T>::try_mutate(account, |history| {
            history.try_push(record)
                .map_err(|_| "Participation history full")?;
            Ok::<(), &'static str>(())
        })?;
        
        // Update proposal stats
        UserProposals::<T>::mutate(account, |stats| {
            stats.total = stats.total.saturating_add(1);
        });
        
        Ok(())
    }
    
    fn record_vote_cast(account: &T::AccountId) -> Result<(), &'static str> {
        let current_block = frame_system::Pallet::<T>::block_number();
        
        let record = ParticipationRecord {
            activity_type: ActivityType::VoteCast,
            block_number: current_block,
            value: 25, // Base score for voting
        };
        
        ParticipationHistory::<T>::try_mutate(account, |history| {
            history.try_push(record)
                .map_err(|_| "Participation history full")?;
            Ok::<(), &'static str>(())
        })?;
        
        Ok(())
    }
    
    fn record_proposal_approval(account: &T::AccountId) -> Result<(), &'static str> {
        let current_block = frame_system::Pallet::<T>::block_number();
        
        let record = ParticipationRecord {
            activity_type: ActivityType::ProposalApproved,
            block_number: current_block,
            value: 200, // High score for successful proposal
        };
        
        ParticipationHistory::<T>::try_mutate(account, |history| {
            history.try_push(record)
                .map_err(|_| "Participation history full")?;
            Ok::<(), &'static str>(())
        })?;
        
        // Update proposal stats
        UserProposals::<T>::mutate(account, |stats| {
            stats.approved = stats.approved.saturating_add(1);
        });
        
        Ok(())
    }
    
    fn record_council_activity(account: &T::AccountId) -> Result<(), &'static str> {
        let current_block = frame_system::Pallet::<T>::block_number();
        
        let record = ParticipationRecord {
            activity_type: ActivityType::CouncilMembership,
            block_number: current_block,
            value: 500, // Score for council activity to align with governance weighting
        };
        
        ParticipationHistory::<T>::try_mutate(account, |history| {
            history.try_push(record)
                .map_err(|_| "Participation history full")?;
            Ok::<(), &'static str>(())
        })?;
        
        Ok(())
    }
}
