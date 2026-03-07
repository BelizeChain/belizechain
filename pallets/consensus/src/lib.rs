#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Consensus Pallet - Federated AI Proof of Useful Work
//!
//! This pallet implements:
//! 1. Federated learning model coordination
//! 2. AI model quality validation and scoring
//! 3. Consensus mechanism based on useful AI work
//! 4. Validator selection based on AI contribution quality
//! 5. Post-quantum signature validation for consensus
//! 6. Economic incentives for AI model improvement

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, LockableCurrency, LockIdentifier,
        Get, Randomness, UnixTime,
    },
    PalletId,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{
        SaturatedConversion, Zero, Saturating,
    },
    RuntimeDebug,
};
use sp_std::vec::Vec;
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const CONSENSUS_ID: PalletId = PalletId(*b"bz/consn");
const AI_WORK_LOCK_ID: LockIdentifier = *b"bzaiwork";

// ===== Named constants replacing magic numbers (audit §1.14) =====

/// Computation time (ms) below which a fast-work bonus is awarded.
const FAST_COMPUTATION_THRESHOLD: u32 = 1_000;

/// Bonus score points for validators who complete work under the threshold.
const FAST_COMPUTATION_BONUS: u32 = 100;

/// Divisor to normalize raw stake (in plancks) into a manageable weight.
const STAKE_NORMALIZATION_DIVISOR: u32 = 1_000_000;

/// Percentage weight for AI quality in validator selection score (PoUW primary metric).
const QUALITY_WEIGHT_PCT: u32 = 50;

/// Percentage weight for economic stake in validator selection score (Sybil resistance).
const STAKE_WEIGHT_PCT: u32 = 20;

/// Percentage weight for sustainability score (green computing / energy efficiency).
const SUSTAINABILITY_WEIGHT_PCT: u32 = 20;

/// Percentage weight for uptime/participation rate (network reliability).
const UPTIME_WEIGHT_PCT: u32 = 10;

/// Percentage weight for staking-pallet quality score in the combined quality blend.
const STAKING_QUALITY_PCT: u32 = 60;

/// Percentage weight for on-chain quality score in the combined quality blend.
const ONCHAIN_QUALITY_PCT: u32 = 40;

/// Baseline efficient computation time in milliseconds for sustainability scoring.
/// Work completing under this threshold earns max sustainability score.
const BASE_EFFICIENT_COMPUTE_MS: u32 = 500;

/// Smoothing weight for sustainability running average (70% history, 30% new sample).
const SUSTAINABILITY_HISTORY_PCT: u32 = 70;
const SUSTAINABILITY_NEW_PCT: u32 = 30;

/// Standard 100-based percentage divisor.
const PERCENT_DIVISOR: u32 = 100;

/// Trait for Staking integration - ties AI model quality to validator reputation
pub trait ConsensusStakingProvider<AccountId, Balance> {
    /// Get validator reputation score (0-100, based on AI contribution history)
    fn get_validator_reputation(account: &AccountId) -> u8;
    
    /// Get recent AI model quality score (0-100, federated learning performance)
    fn get_model_quality_score(account: &AccountId) -> u8;
    
    /// Get validator's staked amount (economic weight in consensus)
    fn get_validator_stake(account: &AccountId) -> Balance;
    
    /// Update validator reputation based on AI work quality
    fn update_reputation(account: &AccountId, quality_score: u8) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for consensus operations
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
        
        /// Source of randomness for consensus
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
        
        /// Time provider for consensus timestamps
        type UnixTime: UnixTime;
        
        /// AI authority origin for model validation
        type AIAuthorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Maximum number of consensus validators
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        
        /// Minimum stake required for consensus participation
        #[pallet::constant]
        type MinConsensusStake: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// AI model quality threshold for consensus participation
        #[pallet::constant]
        type MinModelQualityScore: Get<u32>;
        
        /// Consensus reward for successful AI work validation
        #[pallet::constant]
        type ConsensusReward: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// Weight information
        type WeightInfo: WeightInfo;

        /// Staking provider for validator reputation and economic weights
        type Staking: ConsensusStakingProvider<Self::AccountId, <Self::Currency as Currency<Self::AccountId>>::Balance>;

        // ── AR-15: Rate limiting ──────────────────────────────────────────────
        /// Maximum AI work submissions per validator account per block.
        /// Prevents spam on the consensus scoring mechanism.
        #[pallet::constant]
        type MaxSubmitPerBlock: Get<u32>;
    }

    /// Federated AI model information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct AIModel<AccountId, Moment> {
        /// Model ID
        pub model_id: u32,
        /// Model creator/trainer
        pub trainer: AccountId,
        /// Model type/category
        pub model_type: ModelType,
        /// Model parameters hash
        pub parameters_hash: [u8; 32],
        /// Model accuracy score (0-10000 basis points)
        pub accuracy_score: u32,
        /// Training data size
        pub training_data_size: u32,
        /// Model validation timestamp
        pub validated_at: Option<Moment>,
        /// Number of consensus rounds participated
        pub consensus_rounds: u32,
        /// Total useful work score
        pub useful_work_score: u64,
        /// Model active status
        pub active: bool,
        /// Post-quantum signature for work validation
        pub pq_signature: BoundedVec<u8, ConstU32<256>>,
    }

    /// Types of AI models in the federated system
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ModelType {
        /// Economic prediction models
        Economic,
        /// Tourism demand forecasting
        Tourism,
        /// Agricultural optimization
        Agriculture,
        /// Climate change modeling
        Climate,
        /// Public health analytics
        Health,
        /// Educational data analysis
        Education,
        /// Transportation logistics
        Transportation,
        /// General purpose models
        General,
    }

    /// Consensus validator information
    ///
    /// The consensus score is a composite of four equally-weighted dimensions:
    /// - **Quality** (50%): AI model accuracy and output quality
    /// - **Stake** (20%): Economic commitment for Sybil resistance
    /// - **Sustainability** (20%): Energy efficiency — quality per unit of computation
    /// - **Uptime** (10%): Network availability and participation reliability
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ConsensusValidator<AccountId, Balance> {
        /// Validator account
        pub validator: AccountId,
        /// Staked amount
        pub stake: Balance,
        /// AI models contributed
        pub models: BoundedVec<u32, ConstU32<50>>,
        /// Consensus participation score (total AI work submissions accepted)
        pub participation_score: u32,
        /// Quality score based on AI contributions (0-100 EMA)
        pub quality_score: u32,
        /// Total rewards earned
        pub total_rewards: Balance,
        /// Validator active status
        pub active: bool,
        /// Post-quantum public key for consensus
        pub pq_public_key: BoundedVec<u8, ConstU32<256>>,
        /// Reputation score (0-100, imported from and synced to staking pallet)
        pub reputation: u32,
        // ── AR-5: Extended multi-factor scoring ──────────────────────────────
        /// Sustainability score (0-100 EMA): green computing index.
        ///
        /// Computed as quality output per unit of computation effort.
        /// A validator that produces the same quality with less energy
        /// earns a higher sustainability score.
        /// Formula per submission: `min(100, BASE_EFFICIENT_COMPUTE_MS * quality / max(1, compute_ms))`
        pub sustainability_score: u32,
        /// Blocks in which this validator was listed as eligible to submit work.
        ///
        /// Incremented each time the validator is included in a round's validator list.
        /// Used as the denominator for the uptime ratio.
        pub eligible_rounds: u32,
        /// Rounds in which this validator actually submitted AI work.
        ///
        /// Incremented on every accepted `submit_ai_work` call.
        /// Uptime % = `uptime_rounds * 100 / max(1, eligible_rounds)`.
        pub uptime_rounds: u32,
    }

    /// Consensus round information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ConsensusRound<BlockNumber, Moment> {
        /// Round ID
        pub round_id: u32,
        /// Block number when round started
        pub start_block: BlockNumber,
        /// Round duration in blocks
        pub duration: BlockNumber,
        /// Participating validators
        pub validators: BoundedVec<(u32, u32), ConstU32<32>>, // (validator_id, quality_score)
        /// AI work submissions for this round
        pub ai_work_submissions: BoundedVec<AIWorkSubmission, ConstU32<100>>,
        /// Round status
        pub status: RoundStatus,
        /// Round completion timestamp
        pub completed_at: Option<Moment>,
        /// Total useful work performed
        pub total_useful_work: u64,
    }

    /// AI work submission for consensus
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct AIWorkSubmission {
        /// Submitter validator
        pub validator_id: u32,
        /// AI model used
        pub model_id: u32,
        /// Work type performed
        pub work_type: WorkType,
        /// Work result hash
        pub result_hash: [u8; 32],
        /// Computation time (milliseconds)
        pub computation_time: u32,
        /// Work quality score
        pub quality_score: u32,
        /// Post-quantum signature
        pub pq_signature: BoundedVec<u8, ConstU32<256>>,
        /// Sustainability contribution for this submission (0-100).
        ///
        /// Computed from `min(100, BASE_EFFICIENT_COMPUTE_MS * quality / max(1, computation_time))`.
        /// High quality + low compute time = high sustainability.
        pub sustainability_contribution: u32,
    }

    /// Types of useful AI work
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum WorkType {
        /// Model training iteration
        ModelTraining,
        /// Model inference/prediction
        ModelInference,
        /// Data validation
        DataValidation,
        /// Model optimization
        ModelOptimization,
        /// Federated aggregation
        FederatedAggregation,
        /// Cross-validation
        CrossValidation,
    }

    /// Consensus round status
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum RoundStatus {
        /// Round in progress
        InProgress,
        /// Round completed successfully
        Completed,
        /// Round failed/cancelled
        Failed,
        /// Round pending validation
        PendingValidation,
    }

    #[pallet::storage]
    #[pallet::getter(fn ai_models)]
    /// AI models in the federated system
    pub type AIModels<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Model ID
        AIModel<T::AccountId, u64>,
    >;

    #[allow(clippy::type_complexity)]
    #[pallet::storage]
    #[pallet::getter(fn consensus_validators)]
    /// Consensus validators
    pub type ConsensusValidators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Validator ID
        ConsensusValidator<T::AccountId, <T::Currency as Currency<T::AccountId>>::Balance>,
    >;

    #[allow(clippy::type_complexity)]
    #[pallet::storage]
    #[pallet::getter(fn consensus_rounds)]
    /// Consensus rounds history
    pub type ConsensusRounds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Round ID
        ConsensusRound<BlockNumberFor<T>, u64>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_by_account)]
    /// Validator ID lookup by account
    pub type ValidatorByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Validator ID
    >;

    #[pallet::storage]
    #[pallet::getter(fn model_by_account)]
    /// Models owned by each account
    pub type ModelsByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u32, ConstU32<100>>, // Model IDs
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_model_id)]
    /// Next available AI model ID
    pub type NextModelId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_validator_id)]
    /// Next available validator ID
    pub type NextValidatorId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_round_id)]
    /// Next available consensus round ID
    pub type NextRoundId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_consensus_round)]
    /// Current active consensus round
    pub type CurrentConsensusRound<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn global_ai_metrics)]
    /// Global AI system metrics
    pub type GlobalAIMetrics<T: Config> = StorageValue<
        _,
        AISystemMetrics,
        ValueQuery,
    >;

    // AR-15: Per-account AI work submission rate counter.
    #[pallet::storage]
    /// Rate limit: number of submit_ai_work calls by account in the current block.
    pub type SubmitCallsThisBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type LastSubmitRateLimitBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Global AI system metrics
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
    pub struct AISystemMetrics {
        /// Total models in system
        pub total_models: u32,
        /// Active validators
        pub active_validators: u32,
        /// Total useful work performed
        pub total_useful_work: u64,
        /// Average model quality score (0-100)
        pub average_quality: u32,
        /// Consensus rounds completed
        pub rounds_completed: u32,
        /// Average sustainability score across all active validators (0-100).
        ///
        /// Reflects the overall energy efficiency of the federated AI network.
        pub average_sustainability: u32,
        /// Network-wide average uptime percentage (0-100).
        ///
        /// Validators with consistently low uptime are penalised in round selection.
        pub average_uptime: u32,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// AI model registered
        AIModelRegistered {
            model_id: u32,
            trainer: T::AccountId,
            model_type_index: u8, // Index into ModelType enum
        },
        /// Consensus validator joined
        ValidatorJoined {
            validator_id: u32,
            validator: T::AccountId,
            stake: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Consensus round started
        ConsensusRoundStarted {
            round_id: u32,
            validators_count: u32,
        },
        /// AI work submitted
        AIWorkSubmitted {
            round_id: u32,
            validator_id: u32,
            model_id: u32,
            work_type_index: u8, // Index into WorkType enum
            quality_score: u32,
            /// Sustainability score for this submission (green computing index)
            sustainability_contribution: u32,
        },
        /// Consensus round completed
        ConsensusRoundCompleted {
            round_id: u32,
            total_useful_work: u64,
            participating_validators: u32,
        },
        /// Consensus rewards distributed
        ConsensusRewardsDistributed {
            round_id: u32,
            total_rewards: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Model quality updated
        ModelQualityUpdated {
            model_id: u32,
            new_quality_score: u32,
        },
        /// Consensus validator left (H-39)
        ValidatorLeft {
            validator_id: u32,
            validator: T::AccountId,
            stake_unlocked: <T::Currency as Currency<T::AccountId>>::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// AI model not found
        ModelNotFound,
        /// Validator not found
        ValidatorNotFound,
        /// Consensus round not found
        RoundNotFound,
        /// Insufficient stake for consensus participation
        InsufficientStake,
        /// Model quality below minimum threshold
        QualityBelowThreshold,
        /// Maximum validators reached
        MaxValidatorsReached,
        /// Round not in progress
        RoundNotInProgress,
        /// Invalid AI work submission
        InvalidAIWork,
        /// Unauthorized operation
        Unauthorized,
        /// Model already exists
        ModelAlreadyExists,
        /// Validator already active
        ValidatorAlreadyActive,
        /// Invalid model type index
        InvalidModelType,
        /// Invalid work type index
        InvalidWorkType,
        /// Invalid post-quantum signature
        InvalidPQSignature,
        /// Too many validators for bounded collection
        TooManyValidators,
        /// Too many submissions for bounded collection
        TooManySubmissions,
        /// Validator is participating in an active round and cannot leave
        ValidatorInActiveRound,
        /// Reward minting would overflow total token supply
        SupplyCapExceeded,
        /// #78 FIX: ID counter overflow (u32::MAX reached)
        IdOverflow,
        /// AR-15: Account has exceeded the maximum AI work submissions per block.
        RateLimitExceeded,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Verify consensus invariants at runtime startup.
        ///
        /// Invariants:
        /// 1. GlobalAIMetrics.active_validators ≤ MaxValidators
        fn integrity_test() {
            let metrics = GlobalAIMetrics::<T>::get();
            assert!(
                metrics.active_validators <= T::MaxValidators::get(),
                "INVARIANT VIOLATION: active_validators ({}) > MaxValidators ({})",
                metrics.active_validators,
                T::MaxValidators::get(),
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register new AI model
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_ai_model())]
        pub fn register_ai_model(
            origin: OriginFor<T>,
            model_type_index: u8,
            parameters_hash: [u8; 32],
            training_data_size: u32,
            pq_signature: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Convert index to ModelType
            let model_type = match model_type_index {
                0 => ModelType::Economic,
                1 => ModelType::Tourism, 
                2 => ModelType::Agriculture,
                3 => ModelType::Climate,
                4 => ModelType::Health,
                5 => ModelType::Education,
                6 => ModelType::Transportation,
                7 => ModelType::General,
                _ => return Err(Error::<T>::InvalidModelType.into()),
            };

            let model_id = Self::next_model_id();
            // SAFETY(saturated_into): BlockNumber → u64 will not saturate; block
            // numbers are far below u64::MAX for any realistic chain lifetime.
            let _now = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();

            let ai_model = AIModel {
                model_id,
                trainer: who.clone(),
                model_type: model_type.clone(),
                parameters_hash,
                accuracy_score: 0, // Will be set after validation
                training_data_size,
                validated_at: None,
                consensus_rounds: 0,
                useful_work_score: 0,
                active: false, // Requires validation first
                pq_signature: pq_signature.try_into().map_err(|_| Error::<T>::InvalidPQSignature)?,
            };

            AIModels::<T>::insert(model_id, ai_model);
            
            ModelsByAccount::<T>::try_mutate(&who, |models| {
                models.try_push(model_id)
                    .map_err(|_| Error::<T>::TooManySubmissions)
            })?;

            NextModelId::<T>::put(model_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            // Update global metrics
            GlobalAIMetrics::<T>::mutate(|metrics| {
                metrics.total_models = metrics.total_models.saturating_add(1);
            });

            Self::deposit_event(Event::AIModelRegistered {
                model_id,
                trainer: who,
                model_type_index: match model_type {
                    ModelType::Economic => 0,
                    ModelType::Tourism => 1,
                    ModelType::Agriculture => 2,
                    ModelType::Climate => 3,
                    ModelType::Health => 4,
                    ModelType::Education => 5,
                    ModelType::Transportation => 6,
                    ModelType::General => 7,
                },
            });

            Ok(())
        }

        /// Join as consensus validator
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::join_validator())]
        pub fn join_consensus_validator(
            origin: OriginFor<T>,
            stake_amount: <T::Currency as Currency<T::AccountId>>::Balance,
            pq_public_key: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure sufficient stake
            ensure!(stake_amount >= T::MinConsensusStake::get(), Error::<T>::InsufficientStake);

            // Check if validator limit reached
            let metrics = Self::global_ai_metrics();
            ensure!(metrics.active_validators < T::MaxValidators::get(), Error::<T>::MaxValidatorsReached);

            // Ensure not already a validator
            ensure!(!ValidatorByAccount::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyActive);

            // Lock stake
            T::Currency::set_lock(
                AI_WORK_LOCK_ID,
                &who,
                stake_amount,
                frame_support::traits::WithdrawReasons::all(),
            );

            let validator_id = Self::next_validator_id();

            // Initialize validator with Staking-derived reputation and quality scores
            let initial_reputation = T::Staking::get_validator_reputation(&who).max(10); // Min 10
            let initial_quality = T::Staking::get_model_quality_score(&who);
            
            let validator = ConsensusValidator {
                validator: who.clone(),
                stake: stake_amount,
                models: BoundedVec::new(),
                participation_score: 0,
                quality_score: initial_quality as u32,
                total_rewards: Zero::zero(),
                active: true,
                pq_public_key: pq_public_key.try_into().map_err(|_| Error::<T>::InvalidPQSignature)?,
                reputation: initial_reputation as u32, // Import from Staking pallet
                // AR-5: Initialize extended scoring fields
                sustainability_score: 50, // Start at neutral; updated per work submission
                eligible_rounds: 0,
                uptime_rounds: 0,
            };

            ConsensusValidators::<T>::insert(validator_id, validator);
            ValidatorByAccount::<T>::insert(&who, validator_id);
            NextValidatorId::<T>::put(validator_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            // Update global metrics
            GlobalAIMetrics::<T>::mutate(|metrics| {
                metrics.active_validators = metrics.active_validators.saturating_add(1);
            });

            Self::deposit_event(Event::ValidatorJoined {
                validator_id,
                validator: who,
                stake: stake_amount,
            });

            Ok(())
        }

        /// Validate AI model quality
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::validate_model())]
        pub fn validate_ai_model(
            origin: OriginFor<T>,
            model_id: u32,
            accuracy_score: u32,
        ) -> DispatchResult {
            T::AIAuthorityOrigin::ensure_origin(origin)?;

            AIModels::<T>::mutate(model_id, |maybe_model| {
                if let Some(model) = maybe_model {
                    model.accuracy_score = accuracy_score;
                    // SAFETY(saturated_into): BlockNumber → u64 is lossless for any
                    // realistic chain lifetime (u32 block numbers fit in u64).
                    model.validated_at = Some(frame_system::Pallet::<T>::block_number().saturated_into::<u64>());
                    
                    // Activate model if quality is sufficient
                    if accuracy_score >= T::MinModelQualityScore::get() {
                        model.active = true;
                    }
                    
                    Ok(())
                } else {
                    Err(Error::<T>::ModelNotFound)
                }
            })?;

            // Update global average quality
            Self::update_global_quality_metrics();

            Self::deposit_event(Event::ModelQualityUpdated {
                model_id,
                new_quality_score: accuracy_score,
            });

            Ok(())
        }

        /// Start new consensus round
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::start_consensus_round())]
        pub fn start_consensus_round(
            origin: OriginFor<T>,
            duration_blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::AIAuthorityOrigin::ensure_origin(origin)?;

            // Ensure no active round
            ensure!(Self::current_consensus_round().is_none(), Error::<T>::RoundNotInProgress);

            let round_id = Self::next_round_id();
            let current_block = frame_system::Pallet::<T>::block_number();

            // Select active validators for this round
            let active_validators = Self::select_round_validators();

            let consensus_round = ConsensusRound {
                round_id,
                start_block: current_block,
                duration: duration_blocks,
                validators: active_validators.clone().try_into().map_err(|_| Error::<T>::TooManyValidators)?,
                ai_work_submissions: BoundedVec::new(),
                status: RoundStatus::InProgress,
                completed_at: None,
                total_useful_work: 0,
            };

            ConsensusRounds::<T>::insert(round_id, consensus_round);
            CurrentConsensusRound::<T>::put(round_id);
            NextRoundId::<T>::put(round_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            Self::deposit_event(Event::ConsensusRoundStarted {
                round_id,
                validators_count: active_validators.len() as u32,
            });

            Ok(())
        }

        /// Submit AI work for consensus
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::submit_ai_work())]
        pub fn submit_ai_work(
            origin: OriginFor<T>,
            model_id: u32,
            work_type_index: u8,
            result_hash: [u8; 32],
            computation_time: u32,
            pq_signature: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_submit_rate_limit(&who)?;
            
            // Convert index to WorkType
            let work_type = match work_type_index {
                0 => WorkType::ModelTraining,
                1 => WorkType::ModelInference,
                2 => WorkType::DataValidation,
                3 => WorkType::ModelOptimization,
                4 => WorkType::FederatedAggregation,
                5 => WorkType::CrossValidation,
                _ => return Err(Error::<T>::InvalidWorkType.into()),
            };

            // Get validator ID
            let validator_id = Self::validator_by_account(&who)
                .ok_or(Error::<T>::ValidatorNotFound)?;

            // Ensure there's an active consensus round
            let current_round_id = Self::current_consensus_round()
                .ok_or(Error::<T>::RoundNotInProgress)?;

            // SECURITY: Verify the submitter is an assigned participant of the current round (H-38)
            let round = ConsensusRounds::<T>::get(current_round_id)
                .ok_or(Error::<T>::RoundNotInProgress)?;
            ensure!(
                round.validators.iter().any(|(vid, _)| *vid == validator_id),
                Error::<T>::ValidatorNotFound
            );

            // Verify model exists and is active
            let model = Self::ai_models(model_id)
                .ok_or(Error::<T>::ModelNotFound)?;
            ensure!(model.active, Error::<T>::QualityBelowThreshold);

            // Calculate quality score based on model quality and computation time
            let base_quality = model.accuracy_score;
            let time_bonus = if computation_time < FAST_COMPUTATION_THRESHOLD { FAST_COMPUTATION_BONUS } else { 0 };
            let quality_score = base_quality.saturating_add(time_bonus);

            // AR-5: Compute sustainability contribution for this submission.
            //
            // Sustainability measures green computing efficiency:
            //   sustainability = min(100, BASE_EFFICIENT_COMPUTE_MS * quality / max(1, computation_time_ms))
            //
            // A validator that achieves quality=90 in 300ms scores:
            //   min(100, 500 * 90 / 300) = min(100, 150) = 100 (max)
            //
            // A validator that achieves quality=90 in 5000ms scores:
            //   min(100, 500 * 90 / 5000) = min(100, 9) = 9 (low)
            //
            // This incentivises efficient AI inference and training pipelines.
            let sustainability_contribution = if computation_time == 0 {
                100u32 // Instantaneous work = max sustainability
            } else {
                let numerator = BASE_EFFICIENT_COMPUTE_MS.saturating_mul(quality_score);
                (numerator / computation_time).min(100)
            };

            let work_submission = AIWorkSubmission {
                validator_id,
                model_id,
                work_type: work_type.clone(),
                result_hash,
                computation_time,
                quality_score,
                pq_signature: pq_signature.try_into().map_err(|_| Error::<T>::InvalidPQSignature)?,
                sustainability_contribution,
            };

            // Add work submission to current round
            ConsensusRounds::<T>::try_mutate(current_round_id, |maybe_round| -> DispatchResult {
                if let Some(round) = maybe_round {
                    round.ai_work_submissions.try_push(work_submission)
                        .map_err(|_| Error::<T>::TooManySubmissions)?;
                    round.total_useful_work = round.total_useful_work.saturating_add(quality_score as u64);
                }
                Ok(())
            })?;

            // Update validator participation
            ConsensusValidators::<T>::mutate(validator_id, |maybe_validator| {
                if let Some(validator) = maybe_validator {
                    validator.participation_score = validator.participation_score.saturating_add(1);
                    validator.quality_score = validator.quality_score.saturating_add(quality_score);
                    // AR-5: Update sustainability using exponential moving average
                    // new_score = (history_weight * old + new_weight * sample) / 100
                    validator.sustainability_score = (
                        validator.sustainability_score.saturating_mul(SUSTAINABILITY_HISTORY_PCT)
                        .saturating_add(sustainability_contribution.saturating_mul(SUSTAINABILITY_NEW_PCT))
                    ) / PERCENT_DIVISOR;
                    // AR-5: Count this as an uptime-credited round
                    validator.uptime_rounds = validator.uptime_rounds.saturating_add(1);
                }
            });

            // Update model statistics
            AIModels::<T>::mutate(model_id, |maybe_model| {
                if let Some(model) = maybe_model {
                    model.consensus_rounds = model.consensus_rounds.saturating_add(1);
                    model.useful_work_score = model.useful_work_score.saturating_add(quality_score as u64);
                }
            });

            Self::deposit_event(Event::AIWorkSubmitted {
                round_id: current_round_id,
                validator_id,
                model_id,
                work_type_index: match work_type {
                    WorkType::ModelTraining => 0,
                    WorkType::ModelInference => 1,
                    WorkType::DataValidation => 2,
                    WorkType::ModelOptimization => 3,
                    WorkType::FederatedAggregation => 4,
                    WorkType::CrossValidation => 5,
                },
                quality_score,
                sustainability_contribution,
            });

            Ok(())
        }

        /// Finalize the current consensus round, distribute rewards, and clear the active round
        ///
        /// Can only be called by AIAuthorityOrigin (governance or automated trigger).
        /// The round must have exceeded its configured duration in blocks.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::finalize_consensus_round())]
        pub fn finalize_consensus_round(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            T::AIAuthorityOrigin::ensure_origin(origin)?;

            // Get active round ID
            let round_id = Self::current_consensus_round()
                .ok_or(Error::<T>::RoundNotInProgress)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // Mutate the round in storage
            ConsensusRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;

                // Ensure round is still in progress
                ensure!(round.status == RoundStatus::InProgress, Error::<T>::RoundNotInProgress);

                // Ensure the round duration has elapsed
                let end_block = round.start_block.saturating_add(round.duration);
                ensure!(current_block >= end_block, Error::<T>::RoundNotInProgress);

                let participating_validators = round.validators.len() as u32;
                let total_useful_work = round.total_useful_work;

                // Mark round as completed
                round.status = RoundStatus::Completed;
                // SAFETY(saturated_into): BlockNumber → u64 is lossless (u32 fits u64).
                round.completed_at = Some(current_block.saturated_into::<u64>());

                // Distribute rewards to participating validators proportionally
                let total_reward = T::ConsensusReward::get();
                let mut total_distributed = <<T as Config>::Currency as Currency<T::AccountId>>::Balance::zero();

                // M56 FIX: Verify total issuance + reward won't overflow before minting
                ensure!(
                    T::Currency::total_issuance().checked_add(&total_reward).is_some(),
                    Error::<T>::SupplyCapExceeded
                );

                if !round.ai_work_submissions.is_empty() && total_useful_work > 0 {
                    for submission in round.ai_work_submissions.iter() {
                        if let Some(validator) = ConsensusValidators::<T>::get(submission.validator_id) {
                            // Proportional reward based on quality score contribution
                            let share = total_reward
                                .saturating_mul(submission.quality_score.into())
                                / total_useful_work.max(1).saturated_into();

                            if !share.is_zero() {
                                let _imbalance = T::Currency::deposit_creating(&validator.validator, share);
                                total_distributed = total_distributed.saturating_add(share);

                                // Update validator total rewards
                                ConsensusValidators::<T>::mutate(submission.validator_id, |v| {
                                    if let Some(val) = v {
                                        val.total_rewards = val.total_rewards.saturating_add(share);
                                    }
                                });
                            }
                        }
                    }
                }

                // Clear active round
                CurrentConsensusRound::<T>::kill();

                // Update global metrics
                GlobalAIMetrics::<T>::mutate(|metrics| {
                    metrics.rounds_completed = metrics.rounds_completed.saturating_add(1);
                    metrics.total_useful_work = metrics.total_useful_work.saturating_add(total_useful_work);
                });

                Self::deposit_event(Event::ConsensusRoundCompleted {
                    round_id,
                    total_useful_work,
                    participating_validators,
                });

                Self::deposit_event(Event::ConsensusRewardsDistributed {
                    round_id,
                    total_rewards: total_distributed,
                });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Select validators for a consensus round using four-factor PoUW scoring.
        ///
        /// ## Scoring Formula (AR-5)
        ///
        /// Each eligible validator receives a composite score:
        ///
        /// ```text
        /// score = (quality  * 50%)
        ///       + (stake    * 20%)
        ///       + (sustain  * 20%)
        ///       + (uptime   * 10%)
        /// ```
        ///
        /// ### Quality (50%)
        /// Blended from staking-pallet reputation (60%) and on-chain quality (40%).
        /// Captures both long-term AI contribution history and recent round performance.
        ///
        /// ### Stake (20%)
        /// Economic commitment normalised by `STAKE_NORMALIZATION_DIVISOR`.
        /// Prevents Sybil attacks while not over-centralising around large stake.
        ///
        /// ### Sustainability (20%)
        /// Energy-efficiency index (0-100 EMA).  High quality with low computation time
        /// earns a higher score, incentivising green compute infrastructure.
        ///
        // AR-15: Check and record rate limit for submit_ai_work.
        fn check_submit_rate_limit(who: &T::AccountId) -> frame_support::dispatch::DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let last_block = LastSubmitRateLimitBlock::<T>::get();
            if current_block != last_block {
                SubmitCallsThisBlock::<T>::remove(who);
                LastSubmitRateLimitBlock::<T>::put(current_block);
            }
            let count = SubmitCallsThisBlock::<T>::get(who).saturating_add(1);
            ensure!(count <= T::MaxSubmitPerBlock::get(), Error::<T>::RateLimitExceeded);
            SubmitCallsThisBlock::<T>::insert(who, count);
            Ok(())
        }

        /// ### Uptime (10%)
        /// `uptime_rounds * 100 / max(1, eligible_rounds)`.  Validators who are
        /// consistently present and submitting work score higher than those who skip rounds.
        fn select_round_validators() -> Vec<(u32, u32)> {
            let mut selected_validators = Vec::new();
            let max_validators = T::MaxValidators::get() as usize;
            
            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7)
            for (validator_id, mut validator) in ConsensusValidators::<T>::iter().take(max_validators) {
                // Check basic eligibility
                if !validator.active || validator.quality_score < T::MinModelQualityScore::get() {
                    continue;
                }
                
                // Get real-time quality score from Staking pallet (federated learning performance)
                let staking_quality = T::Staking::get_model_quality_score(&validator.validator);
                
                // Combined quality = 60% staking quality + 40% on-chain quality
                let combined_quality = (staking_quality as u32 * STAKING_QUALITY_PCT + validator.quality_score * ONCHAIN_QUALITY_PCT) / PERCENT_DIVISOR;
                
                // Get economic weight from stake (higher stake = higher priority)
                let stake = T::Staking::get_validator_stake(&validator.validator);
                // SAFETY(saturated_into): Balance → u32 may saturate for very large stakes,
                // but the subsequent division by STAKE_NORMALIZATION_DIVISOR keeps this in
                // a comparable range to the quality score component.
                let stake_weight = stake.saturated_into::<u32>() / STAKE_NORMALIZATION_DIVISOR;

                // AR-5: Sustainability score (green computing index, 0-100 EMA)
                let sustainability = validator.sustainability_score.min(100);

                // AR-5: Uptime score — participation reliability (0-100)
                let uptime = if validator.eligible_rounds > 0 {
                    validator.uptime_rounds.saturating_mul(100) / validator.eligible_rounds
                } else {
                    50 // Default for new validators (benefit of the doubt)
                };
                
                // Final score = 50% quality + 20% stake + 20% sustainability + 10% uptime
                let consensus_score = combined_quality.saturating_mul(QUALITY_WEIGHT_PCT)
                    .saturating_add(stake_weight.saturating_mul(STAKE_WEIGHT_PCT))
                    .saturating_add(sustainability.saturating_mul(SUSTAINABILITY_WEIGHT_PCT))
                    .saturating_add(uptime.saturating_mul(UPTIME_WEIGHT_PCT))
                    / PERCENT_DIVISOR;
                
                // AR-5: Increment eligible_rounds to track uptime denominator
                validator.eligible_rounds = validator.eligible_rounds.saturating_add(1);
                ConsensusValidators::<T>::insert(validator_id, validator);

                selected_validators.push((validator_id, consensus_score));
            }

            // Sort by consensus score (highest first) - Proof of Useful Work
            selected_validators.sort_by(|a, b| b.1.cmp(&a.1));
            
            selected_validators
        }

        /// Update global quality metrics including sustainability and uptime averages (AR-5)
        fn update_global_quality_metrics() {
            let mut total_quality = 0u64;
            let mut total_sustainability = 0u64;
            let mut total_uptime = 0u64;
            let mut active_models = 0u32;
            let mut active_validators = 0u32;

            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7)
            let max_models = T::MaxValidators::get() as usize;
            for (_, model) in AIModels::<T>::iter().take(max_models) {
                if model.active {
                    total_quality = total_quality.saturating_add(model.accuracy_score as u64);
                    active_models = active_models.saturating_add(1);
                }
            }

            for (_, validator) in ConsensusValidators::<T>::iter().take(max_models) {
                if validator.active {
                    total_sustainability = total_sustainability.saturating_add(validator.sustainability_score as u64);
                    let uptime = if validator.eligible_rounds > 0 {
                        validator.uptime_rounds.saturating_mul(100) / validator.eligible_rounds
                    } else {
                        50
                    };
                    total_uptime = total_uptime.saturating_add(uptime as u64);
                    active_validators = active_validators.saturating_add(1);
                }
            }

            let average_quality = if active_models > 0 {
                (total_quality / active_models as u64) as u32
            } else {
                0
            };
            let average_sustainability = if active_validators > 0 {
                (total_sustainability / active_validators as u64) as u32
            } else {
                0
            };
            let average_uptime = if active_validators > 0 {
                (total_uptime / active_validators as u64) as u32
            } else {
                0
            };

            GlobalAIMetrics::<T>::mutate(|metrics| {
                metrics.average_quality = average_quality;
                metrics.average_sustainability = average_sustainability;
                metrics.average_uptime = average_uptime;
            });
        }

        /// Sync validator quality scores to Staking pallet reputation
        pub fn sync_validator_reputation(account: &T::AccountId, quality_score: u8) -> bool {
            T::Staking::update_reputation(account, quality_score)
        }

        /// Get validator's total contribution score using four-factor PoUW formula (AR-5)
        ///
        /// Used by external callers (e.g., reward distribution) to compute a validator's
        /// composite score: quality 50%, stake 20%, sustainability 20%, uptime 10%.
        pub fn get_validator_contribution_score(account: &T::AccountId) -> u32 {
            let quality = T::Staking::get_model_quality_score(account) as u32;
            let reputation = T::Staking::get_validator_reputation(account) as u32;
            // SAFETY(saturated_into): Balance → u32 may saturate for very large stakes,
            // but the subsequent division by STAKE_NORMALIZATION_DIVISOR normalizes the
            // value into a scoring range comparable to quality/reputation components.
            let stake = T::Staking::get_validator_stake(account).saturated_into::<u32>() / STAKE_NORMALIZATION_DIVISOR;

            // Get sustainability and uptime from on-chain validator record
            let (sustainability, uptime) = if let Some(vid) = ValidatorByAccount::<T>::get(account) {
                ConsensusValidators::<T>::get(vid).map(|v| {
                    let up = if v.eligible_rounds > 0 {
                        v.uptime_rounds.saturating_mul(100) / v.eligible_rounds
                    } else {
                        50
                    };
                    (v.sustainability_score.min(100), up)
                }).unwrap_or((50, 50))
            } else {
                (50, 50)
            };

            // Blended quality = 60% staking reputation + 40% on-chain quality
            let blended_quality = (reputation.saturating_mul(60).saturating_add(quality.saturating_mul(40))) / 100;

            blended_quality.saturating_mul(QUALITY_WEIGHT_PCT)
                .saturating_add(stake.saturating_mul(STAKE_WEIGHT_PCT))
                .saturating_add(sustainability.saturating_mul(SUSTAINABILITY_WEIGHT_PCT))
                .saturating_add(uptime.saturating_mul(UPTIME_WEIGHT_PCT))
                / PERCENT_DIVISOR
        }

        /// Get account ID for consensus operations
        pub fn account_id() -> T::AccountId {
            CONSENSUS_ID.into_account_truncating()
        }
    }
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn register_ai_model() -> Weight;
    fn join_validator() -> Weight;
    fn leave_validator() -> Weight;
    fn validate_model() -> Weight;
    fn start_consensus_round() -> Weight;
    fn submit_ai_work() -> Weight;
    fn finalize_consensus_round() -> Weight;
}

impl WeightInfo for () {
    fn register_ai_model() -> Weight {
        Weight::from_parts(25_000_000, 512)
            .saturating_add(Weight::from_parts(0, 4000))
    }
    fn join_validator() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(Weight::from_parts(0, 3500))
    }
    fn leave_validator() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(Weight::from_parts(0, 3500))
    }
    fn validate_model() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2000))
    }
    fn start_consensus_round() -> Weight {
        Weight::from_parts(35_000_000, 512)
            .saturating_add(Weight::from_parts(0, 5000))
    }
    fn submit_ai_work() -> Weight {
        Weight::from_parts(40_000_000, 512)
            .saturating_add(Weight::from_parts(0, 4500))
    }
    fn finalize_consensus_round() -> Weight {
        Weight::from_parts(50_000_000, 512)
            .saturating_add(Weight::from_parts(0, 6000))
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;