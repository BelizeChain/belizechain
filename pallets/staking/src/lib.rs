#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Staking Pallet - Proof of Useful Work (PoUW) Consensus
//!
//! This pallet implements a novel consensus mechanism where validators:
//! 1. Secure the blockchain through traditional staking
//! 2. Execute federated learning tasks on private datasets
//! 3. Submit verifiable model deltas with computation commitments
//! 4. Get rewarded based on contribution quality, timeliness, and honesty

extern crate alloc;

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, LockableCurrency, LockIdentifier,
        Get, Randomness,
    },
    BoundedVec,
    weights::{Weight, constants::RocksDbWeight},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{
        Zero, SaturatedConversion
    },
    Perbill, RuntimeDebug,
};
use scale_info::TypeInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const STAKING_ID: LockIdentifier = *b"bzstking";

/// Minimum staking amount for PoUW validators
pub const MIN_VALIDATOR_STAKE: u128 = 10_000 * 1_000_000; // 10K DALLA (6 decimals)

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::traits::Saturating;

    /// Balance type alias
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Provider trait for Oracle pallet verification
    pub trait OracleVerifier<AccountId> {
        fn is_authorized_operator(who: &AccountId) -> bool;
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for validator staking
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
        
        /// Oracle verification provider
        type OracleVerifier: OracleVerifier<Self::AccountId>;
        
        /// Source of randomness for validator selection
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
        
        /// Identity provider for validator KYC verification
        type Identity: StakingIdentityProvider<Self::AccountId>;
        
        /// Maximum number of validators
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        
        /// Minimum staking amount for validators
        #[pallet::constant]
        type MinValidatorStake: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// Base reward per epoch for validators
        #[pallet::constant]
        type BaseReward: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// Number of blocks per PoUW epoch
        #[pallet::constant]
        type EpochDuration: Get<BlockNumberFor<Self>>;

        /// Number of blocks a validator must wait after requesting unbond
        /// before they can withdraw their stake (default: 14400 blocks = ~24 hours)
        #[pallet::constant]
        type UnbondingPeriod: Get<BlockNumberFor<Self>>;
        
        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    /// Identity provider trait for Staking pallet
    /// Allows Staking pallet to verify validator KYC requirements
    pub trait StakingIdentityProvider<AccountId> {
        /// Get KYC level for validator
        fn get_kyc_level(account: &AccountId) -> Option<u8>;
        
        /// Check if validator meets minimum KYC requirement
        fn meets_validator_kyc(account: &AccountId) -> bool;
        
        /// Check if account is sanctioned
        fn is_sanctioned(account: &AccountId) -> bool;
    }

    /// Validator information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorInfo<AccountId, Balance, BlockNumber> {
        /// Validator account ID
        pub account: AccountId,
        /// Amount staked
        pub stake: Balance,
        /// Compute capacity score
        pub compute_capacity: u32,
        /// Geographic location (for distribution)
        pub location: BoundedVec<u8, ConstU32<64>>,
        /// Compliance score
        pub compliance_score: u8,
        /// Last federated learning contribution
        pub last_fl_contribution: BlockNumber,
        /// Quality score (0-100)
        pub quality_score: u8,
        /// Timeliness score (0-100)
        pub timeliness_score: u8,
        /// Honesty score (0-100)
        pub honesty_score: u8,
        /// Total PoUW contributions
        pub total_contributions: u32,
    }

    /// Federated learning task
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct FederatedLearningTask {
        /// Task ID
        pub task_id: u32,
        /// Model parameters hash
        pub model_hash: [u8; 32],
        /// Expected computation time
        pub computation_time: u32,
        /// Reward multiplier for this task
        pub reward_multiplier: Perbill,
        /// Deadline for submissions
        pub deadline: u32,
    }

    /// Model delta submission
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ModelDelta {
        /// Validator who submitted
        pub validator: BoundedVec<u8, ConstU32<32>>, // AccountId encoded
        /// Encrypted model delta (AES-GCM ciphertext)
        pub encrypted_delta: BoundedVec<u8, ConstU32<1024>>,
        /// Computation commitment: blake2_256(model_weights || nonce || task_id)
        /// Proves the validator performed computation without revealing model data.
        /// NOT a zero-knowledge proof — honest hash commitment only.
        pub computation_commitment: [u8; 32],
        /// Submission timestamp
        pub submitted_at: u32,
        /// Computation log hash: blake2_256(timestamped_execution_log)
        pub computation_log: [u8; 32],
    }

    /// Quantum job contribution for PoUW rewards
    /// Links quantum computing execution to validator staking
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct QuantumContribution {
        /// Quantum job ID from Kinich
        pub job_id: BoundedVec<u8, ConstU32<64>>,
        /// Validator who executed the job
        pub validator: BoundedVec<u8, ConstU32<32>>, // AccountId encoded
        /// Number of qubits in circuit
        pub num_qubits: u16,
        /// Circuit depth (complexity metric)
        pub circuit_depth: u32,
        /// Number of shots executed
        pub num_shots: u32,
        /// Accuracy score after error mitigation (0-100)
        pub accuracy_score: u8,
        /// Block number when job completed
        pub completed_at: u32,
        /// Computation complexity score (qubits × depth)
        pub computation_score: u32,
    }

    /// Aggregated quantum statistics per validator per epoch
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct ValidatorQuantumStats {
        /// Total quantum jobs executed this epoch
        pub jobs_executed: u32,
        /// Average accuracy score across all jobs
        pub avg_accuracy: u8,
        /// Total computation score (sum of all job complexities)
        pub total_complexity: u64,
        /// Total qubits processed
        pub total_qubits: u64,
        /// Total shots executed
        pub total_shots: u64,
        /// Quantum contribution score (0-100)
        pub quantum_score: u8,
    }

    #[allow(clippy::type_complexity)]
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    /// Current active validators
    pub type Validators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorInfo<T::AccountId, <T::Currency as Currency<T::AccountId>>::Balance, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    /// Number of active validators (kept to avoid unbounded iteration for count)
    pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    /// Current PoUW epoch number
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_fl_task)]
    /// Current active federated learning task
    pub type ActiveFLTask<T: Config> = StorageValue<_, FederatedLearningTask>;

    #[pallet::storage]
    #[pallet::getter(fn model_submissions)]
    /// Model delta submissions for current epoch
    pub type ModelSubmissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ModelDelta,
    >;

    #[pallet::storage]
    #[pallet::getter(fn epoch_rewards)]
    /// Rewards accumulated per epoch
    pub type EpochRewards<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn slashing_spans)]
    /// Slashing information for validators
    pub type SlashingSpans<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Number of times slashed
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pending_unbonds)]
    /// Validators in the unbonding period: maps account to the block at which they can withdraw
    pub type PendingUnbonds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (BalanceOf<T>, BlockNumberFor<T>), // (stake, unlock_at)
    >;

    // ═══════════════════════════════════════════════════════════════════
    // QUANTUM COMPUTING INTEGRATION (Phase 2.2)
    // ═══════════════════════════════════════════════════════════════════

    #[pallet::storage]
    #[pallet::getter(fn quantum_contributions)]
    /// Quantum job contributions by job ID
    pub type QuantumContributions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<64>>, // job_id
        QuantumContribution,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_quantum_stats)]
    /// Quantum statistics per validator for current epoch
    pub type ValidatorQuantumStatsMap<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorQuantumStats,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn epoch_quantum_jobs)]
    /// Total quantum jobs executed in current epoch
    pub type EpochQuantumJobs<T: Config> = StorageValue<_, u32, ValueQuery>;

    // ═══════════════════════════════════════════════════════════════════
    // DOMAIN-SPECIFIC PoUW INTEGRATION (Phase 3)
    // ═══════════════════════════════════════════════════════════════════

    /// Domain index matching Oracle pallet
    /// Must stay synchronized with pallets/oracle/src/types.rs ModelDomain
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DomainIndex {
        General = 0,
        AgriTech = 1,
        Marine = 2,
        Education = 3,
        Tech = 4,
    }

    /// Domain-specific contribution statistics
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct DomainStats {
        /// Number of contributions to this domain
        pub contribution_count: u32,
        /// Average quality score for this domain
        pub avg_quality: u8,
        /// Total data volume processed (in KB)
        pub total_volume: u64,
        /// Last contribution block number
        pub last_contribution: u32,
    }

    /// Operator domain breakdown for current epoch
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct OperatorDomainBreakdown {
        pub general: DomainStats,
        pub agritech: DomainStats,
        pub marine: DomainStats,
        pub education: DomainStats,
        pub tech: DomainStats,
    }

    #[pallet::storage]
    #[pallet::getter(fn operator_domain_stats)]
    /// Domain-specific contribution statistics per operator
    pub type OperatorDomainStatsMap<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        OperatorDomainBreakdown,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn epoch_domain_contributions)]
    /// Total contributions per domain in current epoch
    pub type EpochDomainContributions<T: Config> = StorageValue<_, OperatorDomainBreakdown, ValueQuery>;

    // GenesisConfig removed for Substrate v42 compatibility

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Validator joined the active set
        ValidatorJoined {
            validator: T::AccountId,
            stake: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Validator left the active set
        ValidatorLeft {
            validator: T::AccountId,
        },
        /// New federated learning task assigned
        FLTaskAssigned {
            task_id: u32,
            deadline: BlockNumberFor<T>,
        },
        /// Model delta submitted by validator
        ModelDeltaSubmitted {
            validator: T::AccountId,
            task_id: u32,
            quality_score: u8,
        },
        /// Rewards distributed for epoch
        RewardsDistributed {
            epoch: u32,
            total_rewards: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Validator slashed for violation
        ValidatorSlashed {
            validator: T::AccountId,
            slash_amount: <T::Currency as Currency<T::AccountId>>::Balance,
            reason: u8, // SlashReason as u8
        },
        /// Epoch completed
        EpochCompleted {
            epoch: u32,
            participants: u32,
        },
        /// Quantum contribution recorded (Phase 2.2)
        QuantumContributionRecorded {
            job_id: BoundedVec<u8, ConstU32<64>>,
            validator: T::AccountId,
            quantum_score: u32,
        },
        /// Domain-specific contribution recorded (Phase 3)
        DomainContributionRecorded {
            operator: T::AccountId,
            domain: u8, // DomainIndex as u8
            quality_score: u8,
            volume: u32,
        },
        /// PoUW rewards claimed with domain bonus (Phase 3)
        PouWRewardsClaimedWithBonus {
            operator: T::AccountId,
            base_reward: <T::Currency as Currency<T::AccountId>>::Balance,
            domain_bonus: <T::Currency as Currency<T::AccountId>>::Balance,
            total_reward: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Validator started unbonding period
        UnbondingStarted {
            validator: T::AccountId,
            stake: <T::Currency as Currency<T::AccountId>>::Balance,
            unlock_at: BlockNumberFor<T>,
        },
        /// Validator withdrew unbonded stake
        StakeWithdrawn {
            validator: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Stake amount below minimum required
        InsufficientStake,
        /// Validator already active
        ValidatorAlreadyActive,
        /// Validator not found
        ValidatorNotFound,
        /// No active federated learning task
        NoActiveFLTask,
        /// Model delta already submitted for current task
        ModelDeltaAlreadySubmitted,
        /// Invalid computation commitment (all zeros or duplicate)
        InvalidComputationCommitment,
        /// Submission deadline exceeded
        SubmissionDeadlineExceeded,
        /// Maximum validators reached
        MaxValidatorsReached,
        /// Invalid compute capacity specification
        InvalidComputeCapacity,
        /// Compliance score too low
        InsufficientComplianceScore,
        /// Validator does not meet KYC requirements
        ValidatorKycInsufficient,
        /// Validator account is sanctioned
        ValidatorSanctioned,
        /// Quantum job already recorded (Phase 2.2)
        QuantumJobAlreadyRecorded,
        /// Invalid accuracy score provided (must be 0-100)
        InvalidAccuracyScore,
        /// Invalid domain index (Phase 3)
        InvalidDomainIndex,
        /// Operator has no domain contributions (Phase 3)
        NoDomainContributions,
        /// Caller is not an authorized Oracle operator
        NotAuthorizedOracle,
        /// Validator is already unbonding
        AlreadyUnbonding,
        /// No pending unbond found
        NoPendingUnbond,
        /// Unbonding period not yet elapsed
        UnbondingNotReady,
        /// Invalid slash reason code
        InvalidSlashReason,
    }

    /// Reasons for slashing validators
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum SlashReason {
        /// Invalid computation proof submitted
        InvalidProof,
        /// Missed submission deadline
        MissedDeadline,
        /// Privacy breach detected
        PrivacyBreach,
        /// Model poisoning attempt
        ModelPoisoning,
        /// Consensus violation (double signing, etc.)
        ConsensusViolation,
    }

    impl SlashReason {
        /// Convert SlashReason to u8
        pub fn as_u8(&self) -> u8 {
            match self {
                SlashReason::InvalidProof => 0,
                SlashReason::MissedDeadline => 1,
                SlashReason::PrivacyBreach => 2,
                SlashReason::ModelPoisoning => 3,
                SlashReason::ConsensusViolation => 4,
            }
        }

        /// Convert u8 to SlashReason
        pub fn from_u8(value: u8) -> Option<Self> {
            match value {
                0 => Some(SlashReason::InvalidProof),
                1 => Some(SlashReason::MissedDeadline),
                2 => Some(SlashReason::PrivacyBreach),
                3 => Some(SlashReason::ModelPoisoning),
                4 => Some(SlashReason::ConsensusViolation),
                _ => None,
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join as a PoUW validator
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::join_validators())]
        pub fn join_validators(
            origin: OriginFor<T>,
            stake: <T::Currency as Currency<T::AccountId>>::Balance,
            compute_capacity: u32,
            location: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: Record participation in Community pallet via trait (Phase 6 integration)
            // Activity code 2 = ValidatorStaking
            // let _ = T::CommunityParticipation::record_validator_activity(&who);

            // Verify validator meets KYC requirements (Level 3 - Enhanced verification required)
            ensure!(
                T::Identity::meets_validator_kyc(&who),
                Error::<T>::ValidatorKycInsufficient
            );
            
            // Check sanctions list
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::ValidatorSanctioned
            );

            // Ensure minimum stake
            ensure!(stake >= T::MinValidatorStake::get(), Error::<T>::InsufficientStake);

            // Check if validator already active
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyActive);

            // Check maximum validators limit using tracked count; lazily backfill if zero
            let mut validator_count = ValidatorCount::<T>::get();
            if validator_count == 0 {
                let counted = Validators::<T>::iter_keys().fold(0u32, |acc, _| acc.saturating_add(1));
                if counted > 0 {
                    ValidatorCount::<T>::put(counted);
                    validator_count = counted;
                }
            }
            ensure!(validator_count < T::MaxValidators::get(), Error::<T>::MaxValidatorsReached);

            // Validate compute capacity
            ensure!(compute_capacity >= 50, Error::<T>::InvalidComputeCapacity);

            // Ensure the user has enough free balance for the lock
            ensure!(
                T::Currency::free_balance(&who) >= stake,
                Error::<T>::InsufficientStake
            );

            // Lock the stake for staking (lock prevents spending, no reserve needed)
            T::Currency::set_lock(
                STAKING_ID,
                &who,
                stake,
                frame_support::traits::WithdrawReasons::all(),
            );

            // Create validator info
            let validator_info = ValidatorInfo {
                account: who.clone(),
                stake,
                compute_capacity,
                location,
                compliance_score: 100, // Start with full compliance
                last_fl_contribution: frame_system::Pallet::<T>::block_number(),
                quality_score: 80, // Starting quality score
                timeliness_score: 90, // Starting timeliness score
                honesty_score: 95, // Starting honesty score
                total_contributions: 0,
            };

            Validators::<T>::insert(&who, validator_info);
            ValidatorCount::<T>::put(validator_count.saturating_add(1));

            Self::deposit_event(Event::ValidatorJoined { validator: who, stake });

            Ok(())
        }

        /// Leave the validator set and start the unbonding period
        ///
        /// The validator is removed from the active set immediately, but their
        /// stake remains locked for `UnbondingPeriod` blocks. Call `withdraw_unbonded`
        /// after the period elapses to reclaim funds.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::leave_validators())]
        pub fn leave_validators(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure validator exists
            let validator_info = Validators::<T>::get(&who).ok_or(Error::<T>::ValidatorNotFound)?;

            // Ensure not already unbonding
            ensure!(!PendingUnbonds::<T>::contains_key(&who), Error::<T>::AlreadyUnbonding);

            // Remove validator from active set and decrement count
            Validators::<T>::remove(&who);
            ValidatorCount::<T>::mutate(|c| { *c = c.saturating_sub(1); });

            // Keep the lock in place during unbonding — record unlock time
            let current_block = frame_system::Pallet::<T>::block_number();
            let unlock_at = current_block.saturating_add(T::UnbondingPeriod::get());
            PendingUnbonds::<T>::insert(&who, (validator_info.stake, unlock_at));

            Self::deposit_event(Event::UnbondingStarted {
                validator: who,
                stake: validator_info.stake,
                unlock_at,
            });

            Ok(())
        }

        /// Withdraw stake after the unbonding period has elapsed
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::leave_validators())]
        pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (stake, unlock_at) = PendingUnbonds::<T>::get(&who)
                .ok_or(Error::<T>::NoPendingUnbond)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block >= unlock_at, Error::<T>::UnbondingNotReady);

            // Remove pending unbond entry and release the lock
            PendingUnbonds::<T>::remove(&who);
            T::Currency::remove_lock(STAKING_ID, &who);

            Self::deposit_event(Event::StakeWithdrawn {
                validator: who,
                amount: stake,
            });

            Ok(())
        }

        /// Report a validator offense and apply slashing.
        /// Root-only extrinsic for governance / consensus to slash misbehaving validators.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::report_validator_offense())]
        pub fn report_validator_offense(
            origin: OriginFor<T>,
            validator: T::AccountId,
            slash_percent: u32,
            reason_code: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let reason = SlashReason::from_u8(reason_code)
                .ok_or(Error::<T>::InvalidSlashReason)?;

            ensure!(
                Validators::<T>::contains_key(&validator),
                Error::<T>::ValidatorNotFound
            );

            let slash_percentage = Perbill::from_percent(slash_percent.min(100));
            Self::slash_validator(&validator, slash_percentage, reason)?;

            Ok(())
        }

        /// Submit model delta for federated learning task
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::submit_model_delta())]
        pub fn submit_model_delta(
            origin: OriginFor<T>,
            task_id: u32,
            encrypted_delta: BoundedVec<u8, ConstU32<1024>>,
            computation_commitment: [u8; 32],
            computation_log: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure validator is active
            let mut validator_info = Validators::<T>::get(&who).ok_or(Error::<T>::ValidatorNotFound)?;

            // Ensure active FL task exists
            let active_task = ActiveFLTask::<T>::get().ok_or(Error::<T>::NoActiveFLTask)?;
            ensure!(active_task.task_id == task_id, Error::<T>::NoActiveFLTask);

            // Ensure not already submitted
            ensure!(!ModelSubmissions::<T>::contains_key(&who), Error::<T>::ModelDeltaAlreadySubmitted);

            // Check deadline
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= BlockNumberFor::<T>::from(active_task.deadline),
                Error::<T>::SubmissionDeadlineExceeded
            );

            // Validate computation commitment:
            // 1. Must not be all zeros (empty/uncomputed)
            // 2. Must not be all same byte (trivial commitment)
            // 3. Encrypted delta must be non-empty (at least 16 bytes for AES-GCM minimum)
            ensure!(
                computation_commitment != [0u8; 32],
                Error::<T>::InvalidComputationCommitment
            );
            ensure!(
                !computation_commitment.iter().all(|&b| b == computation_commitment[0]),
                Error::<T>::InvalidComputationCommitment
            );
            ensure!(
                encrypted_delta.len() >= 16,
                Error::<T>::InvalidComputationCommitment
            );
            // Verify computation_log is non-trivial
            ensure!(
                computation_log != [0u8; 32],
                Error::<T>::InvalidComputationCommitment
            );

            // Calculate scores based on submission quality and timeliness
            let quality_score = Self::evaluate_model_quality(&encrypted_delta);
            let timeliness_score = Self::calculate_timeliness_score(current_block, active_task.deadline);

            // Create model delta submission
            let model_delta = ModelDelta {
                validator: who.encode().try_into().unwrap_or_default(),
                encrypted_delta,
                computation_commitment,
                // SAFETY: BlockNumber fits in u32 (runtime uses u32 block numbers)
                submitted_at: current_block.saturated_into::<u32>(),
                computation_log,
            };

            ModelSubmissions::<T>::insert(&who, model_delta);

            // Update validator scores
            validator_info.quality_score = quality_score;
            validator_info.timeliness_score = timeliness_score;
            validator_info.total_contributions = validator_info.total_contributions.saturating_add(1);
            validator_info.last_fl_contribution = current_block;

            Validators::<T>::insert(&who, validator_info);

            Self::deposit_event(Event::ModelDeltaSubmitted {
                validator: who,
                task_id,
                quality_score: 80u8,
            });            Ok(())
        }

        /// Assign new federated learning task (only callable by governance/sudo)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::assign_fl_task())]
        pub fn assign_fl_task(
            origin: OriginFor<T>,
            task_id: u32,
            model_hash: [u8; 32],
            computation_time: u32,
            reward_multiplier: Perbill,
            deadline_blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let deadline_blocks_u64: u64 = TryInto::<u64>::try_into(deadline_blocks).unwrap_or(0);
            let deadline_u64 = current_u64.saturating_add(deadline_blocks_u64);
            // SAFETY: deadline_u64 derived from block numbers which fit in BlockNumberFor<T>
            let deadline: BlockNumberFor<T> = deadline_u64.saturated_into();

            let fl_task = FederatedLearningTask {
                task_id,
                model_hash,
                computation_time,
                reward_multiplier,
                // SAFETY: BlockNumber fits in u32 (runtime uses u32 block numbers)
                deadline: deadline.saturated_into::<u32>(),
            };

            ActiveFLTask::<T>::put(&fl_task);

            // Clear previous submissions
            let _ = ModelSubmissions::<T>::clear(u32::MAX, None);

            Self::deposit_event(Event::FLTaskAssigned {
                task_id,
                deadline,
            });

            Ok(())
        }

        /// Distribute rewards for completed epoch
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::distribute_rewards())]
    #[allow(clippy::type_complexity)]
    pub fn distribute_rewards(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;

            let current_epoch = Self::current_epoch();
            let base_reward = T::BaseReward::get();
            
            let mut total_distributed: <T::Currency as Currency<T::AccountId>>::Balance = Zero::zero();
            let mut participant_count = 0u32;

            // Calculate and distribute rewards to validators
            // SECURITY: Bounded iteration prevents DoS via storage bloat (§1.7)
            let max_validators = T::MaxValidators::get() as usize;
            for (validator_id, validator_info) in Validators::<T>::iter().take(max_validators) {
                if let Some(_model_submission) = ModelSubmissions::<T>::get(&validator_id) {
                    let reward = Self::calculate_validator_reward(&validator_info, base_reward);
                    
                    // Mint rewards to validator
                    let _ = T::Currency::deposit_creating(&validator_id, reward);
                    
                    total_distributed = total_distributed.saturating_add(reward);
                    participant_count = participant_count.saturating_add(1);
                }
            }

            // Record epoch rewards
            EpochRewards::<T>::insert(current_epoch, total_distributed);

            // Advance to next epoch
            CurrentEpoch::<T>::put(current_epoch.saturating_add(1));

            // Clear submissions for next epoch
            let _ = ModelSubmissions::<T>::clear(u32::MAX, None);

            // Clear quantum stats for next epoch (Phase 2.2)
            let _ = ValidatorQuantumStatsMap::<T>::clear(u32::MAX, None);
            EpochQuantumJobs::<T>::kill();

            Self::deposit_event(Event::RewardsDistributed {
                epoch: current_epoch,
                total_rewards: total_distributed,
            });

            Self::deposit_event(Event::EpochCompleted {
                epoch: current_epoch,
                participants: participant_count,
            });

            Ok(())
        }

        /// Record quantum job contribution for validator rewards (Phase 2.2)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::record_quantum_contribution())]
        pub fn record_quantum_contribution(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<64>>,
            validator: T::AccountId,
            num_qubits: u16,
            circuit_depth: u32,
            num_shots: u32,
            accuracy_score: u8,
        ) -> DispatchResult {
            // Only Quantum pallet or root can record contributions
            let caller = ensure_signed_or_root(origin)?;
            
            // If signed, verify caller is an authorized Oracle operator
            if let Some(who) = caller {
                ensure!(
                    T::OracleVerifier::is_authorized_operator(&who),
                    Error::<T>::NotAuthorizedOracle
                );
            }

            // Ensure validator exists
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotFound);

            // Ensure job not already recorded
            ensure!(!QuantumContributions::<T>::contains_key(&job_id), Error::<T>::QuantumJobAlreadyRecorded);

            // Validate accuracy score (0-100)
            ensure!(accuracy_score <= 100, Error::<T>::InvalidAccuracyScore);

            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Calculate computation score based on circuit complexity
            // Formula: (qubits × depth) / 100, capped at u32::MAX
            let complexity: u64 = (num_qubits as u64)
                .saturating_mul(circuit_depth as u64);
            let computation_score: u32 = (complexity / 100).min(u32::MAX as u64) as u32;

            // Create quantum contribution record
            let contribution = QuantumContribution {
                job_id: job_id.clone(),
                validator: validator.encode().try_into().unwrap_or_default(),
                num_qubits,
                circuit_depth,
                num_shots,
                accuracy_score,
                // SAFETY: BlockNumber fits in u32 (runtime uses u32 block numbers)
                completed_at: current_block.saturated_into::<u32>(),
                computation_score,
            };

            // Store contribution
            QuantumContributions::<T>::insert(&job_id, contribution);

            // Update validator quantum stats
            ValidatorQuantumStatsMap::<T>::mutate(&validator, |stats| {
                stats.jobs_executed = stats.jobs_executed.saturating_add(1);
                
                // Update rolling average accuracy
                let total_accuracy = (stats.avg_accuracy as u32)
                    .saturating_mul(stats.jobs_executed.saturating_sub(1))
                    .saturating_add(accuracy_score as u32);
                stats.avg_accuracy = (total_accuracy / stats.jobs_executed) as u8;
                
                // Update totals
                stats.total_complexity = stats.total_complexity.saturating_add(complexity);
                stats.total_qubits = stats.total_qubits.saturating_add(num_qubits as u64);
                stats.total_shots = stats.total_shots.saturating_add(num_shots as u64);
                
                // Calculate quantum score (0-100)
                // Formula: (job_count/100 × 40%) + (avg_accuracy × 35%) + (complexity/1000 × 25%)
                let job_score = ((stats.jobs_executed.min(100) * 40) / 100) as u8;
                let accuracy_contribution = ((stats.avg_accuracy as u32 * 35) / 100) as u8;
                let complexity_contribution = ((stats.total_complexity.min(100_000) / 1000 * 25) / 100) as u8;
                
                stats.quantum_score = job_score
                    .saturating_add(accuracy_contribution)
                    .saturating_add(complexity_contribution)
                    .min(100);
            });

            // Increment epoch job counter
            EpochQuantumJobs::<T>::mutate(|count| {
                *count = count.saturating_add(1);
            });

            Self::deposit_event(Event::QuantumContributionRecorded {
                job_id,
                validator,
                quantum_score: computation_score,
            });

            Ok(())
        }

        /// Force join a validator (testing/admin only - bypasses KYC checks)
        /// 
        /// WARNING: This extrinsic bypasses all KYC and sanction checks.
        /// Should ONLY be used for testing or emergency administrative actions.
        /// In production, always use `join_validators` which enforces proper compliance.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::join_validators())]
        pub fn force_join_validator(
            origin: OriginFor<T>,
            who: T::AccountId,
            stake: <T::Currency as Currency<T::AccountId>>::Balance,
            compute_capacity: u32,
            location: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            // Require root/sudo origin
            ensure_root(origin)?;

            // Ensure minimum stake (keep this check for safety)
            ensure!(stake >= T::MinValidatorStake::get(), Error::<T>::InsufficientStake);

            // Check if validator already active
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyActive);

            // Check maximum validators limit using tracked count; lazily backfill if zero
            let mut validator_count = ValidatorCount::<T>::get();
            if validator_count == 0 {
                let counted = Validators::<T>::iter_keys().fold(0u32, |acc, _| acc.saturating_add(1));
                if counted > 0 {
                    ValidatorCount::<T>::put(counted);
                    validator_count = counted;
                }
            }
            ensure!(validator_count < T::MaxValidators::get(), Error::<T>::MaxValidatorsReached);

            // Validate compute capacity
            ensure!(compute_capacity >= 50, Error::<T>::InvalidComputeCapacity);

            // Ensure the user has enough free balance for the lock
            ensure!(
                T::Currency::free_balance(&who) >= stake,
                Error::<T>::InsufficientStake
            );

            // Lock the stake for staking (lock prevents spending, no reserve needed)
            T::Currency::set_lock(
                STAKING_ID,
                &who,
                stake,
                frame_support::traits::WithdrawReasons::all(),
            );

            // Create validator info
            let validator_info = ValidatorInfo {
                account: who.clone(),
                stake,
                compute_capacity,
                location,
                compliance_score: 100, // Start with full compliance
                last_fl_contribution: frame_system::Pallet::<T>::block_number(),
                quality_score: 80, // Starting quality score
                timeliness_score: 90, // Starting timeliness score
                honesty_score: 95, // Starting honesty score
                total_contributions: 0,
            };

            Validators::<T>::insert(&who, validator_info);
            ValidatorCount::<T>::mutate(|c| { *c = c.saturating_add(1); });

            Self::deposit_event(Event::ValidatorJoined { validator: who, stake });

            Ok(())
        }

        /// Record domain-specific contribution for PoUW rewards (Phase 3)
        /// Called by Oracle pallet when processing IoT data submissions
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::record_domain_contribution())]
        pub fn record_domain_contribution(
            origin: OriginFor<T>,
            operator: T::AccountId,
            domain: u8, // DomainIndex as u8
            quality_score: u8,
            volume_kb: u32,
        ) -> DispatchResult {
            // Only Oracle pallet or root can record contributions
            let caller = ensure_signed_or_root(origin)?;
            
            // If signed, verify caller is an authorized Oracle operator
            if let Some(who) = caller {
                ensure!(
                    T::OracleVerifier::is_authorized_operator(&who),
                    Error::<T>::NotAuthorizedOracle
                );
            }

            // Validate domain index (0-4)
            ensure!(domain <= 4, Error::<T>::InvalidDomainIndex);

            // Validate quality score (0-100)
            ensure!(quality_score <= 100, Error::<T>::InvalidAccuracyScore);

            let current_block = frame_system::Pallet::<T>::block_number();
            // SAFETY: BlockNumber fits in u32 (runtime uses u32 block numbers)
            let current_block_u32 = current_block.saturated_into::<u32>();

            // Update operator domain stats
            OperatorDomainStatsMap::<T>::mutate(&operator, |breakdown| {
                let stats = match domain {
                    0 => &mut breakdown.general,
                    1 => &mut breakdown.agritech,
                    2 => &mut breakdown.marine,
                    3 => &mut breakdown.education,
                    4 => &mut breakdown.tech,
                    _ => return, // Already validated above
                };

                // Update contribution count
                stats.contribution_count = stats.contribution_count.saturating_add(1);

                // Update rolling average quality
                let total_quality = (stats.avg_quality as u32)
                    .saturating_mul(stats.contribution_count.saturating_sub(1))
                    .saturating_add(quality_score as u32);
                stats.avg_quality = (total_quality / stats.contribution_count) as u8;

                // Update total volume
                stats.total_volume = stats.total_volume.saturating_add(volume_kb as u64);

                // Update last contribution block
                stats.last_contribution = current_block_u32;
            });

            // Update epoch domain contributions
            EpochDomainContributions::<T>::mutate(|breakdown| {
                let stats = match domain {
                    0 => &mut breakdown.general,
                    1 => &mut breakdown.agritech,
                    2 => &mut breakdown.marine,
                    3 => &mut breakdown.education,
                    4 => &mut breakdown.tech,
                    _ => return,
                };

                stats.contribution_count = stats.contribution_count.saturating_add(1);
                stats.total_volume = stats.total_volume.saturating_add(volume_kb as u64);
            });

            Self::deposit_event(Event::DomainContributionRecorded {
                operator,
                domain,
                quality_score,
                volume: volume_kb,
            });

            Ok(())
        }

        /// Claim PoUW rewards with domain-specific bonus multipliers (Phase 3)
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::claim_pouw_with_domain_bonus())]
        pub fn claim_pouw_with_domain_bonus(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure operator is a validator
            let validator_info = Validators::<T>::get(&who)
                .ok_or(Error::<T>::ValidatorNotFound)?;

            // Get domain statistics
            let domain_breakdown = OperatorDomainStatsMap::<T>::get(&who);

            // Ensure operator has domain contributions
            let has_contributions = domain_breakdown.general.contribution_count > 0
                || domain_breakdown.agritech.contribution_count > 0
                || domain_breakdown.marine.contribution_count > 0
                || domain_breakdown.education.contribution_count > 0
                || domain_breakdown.tech.contribution_count > 0;
            
            ensure!(has_contributions, Error::<T>::NoDomainContributions);

            // Calculate base PoUW reward (existing FL + Quantum)
            let base_reward = Self::calculate_validator_reward(&validator_info, T::BaseReward::get());

            // Calculate domain bonus using fixed-point arithmetic (10_000 = 1.0x)
            let domain_bonus = Self::calculate_domain_bonus(&who, &domain_breakdown);

            // SAFETY: Balance is u128-backed; converting Balance → u128 is lossless
            let base_reward_u128: u128 = base_reward.saturated_into();
            
            // Apply domain bonus multiplier (fixed-point: 10_000 = 1.0x)
            let bonus_reward_u128 = base_reward_u128
                .saturating_mul(domain_bonus)
                .saturating_sub(base_reward_u128.saturating_mul(10_000)) // Subtract base to get only bonus
                / 10_000;

            // SAFETY: bonus_reward_u128 ≤ base_reward_u128 (bounded by domain_bonus multiplier), fits in Balance
            let bonus_reward = bonus_reward_u128.saturated_into();
            let total_reward = base_reward.saturating_add(bonus_reward);

            // Mint rewards to operator
            let _ = T::Currency::deposit_creating(&who, total_reward);

            Self::deposit_event(Event::PouWRewardsClaimedWithBonus {
                operator: who,
                base_reward,
                domain_bonus: bonus_reward,
                total_reward,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate domain-specific bonus multiplier (Phase 3)
        /// 
        /// Returns fixed-point multiplier where 10_000 = 1.0x
        /// Domain multipliers:
        /// - AgriTech: 1.5x (15_000) - Critical priority
        /// - Marine: 1.4x (14_000) - High priority
        /// - Education: 1.3x (13_000) - High priority
        /// - Tech: 1.1x (11_000) - Medium priority
        /// - General: 1.0x (10_000) - Baseline
        fn calculate_domain_bonus(
            _operator: &T::AccountId,
            breakdown: &OperatorDomainBreakdown,
        ) -> u128 {
            // Domain multipliers (fixed-point: 10_000 = 1.0x)
            const AGRITECH_MULTIPLIER: u128 = 15_000; // 1.5x
            const MARINE_MULTIPLIER: u128 = 14_000;   // 1.4x
            const EDUCATION_MULTIPLIER: u128 = 13_000; // 1.3x
            const TECH_MULTIPLIER: u128 = 11_000;     // 1.1x
            const GENERAL_MULTIPLIER: u128 = 10_000;  // 1.0x

            // Calculate weighted bonus based on contribution counts and quality
            let mut total_weighted_score: u128 = 0;
            let mut total_contributions: u128 = 0;

            // AgriTech contributions
            if breakdown.agritech.contribution_count > 0 {
                let weight = (breakdown.agritech.contribution_count as u128)
                    .saturating_mul(breakdown.agritech.avg_quality as u128);
                total_weighted_score = total_weighted_score.saturating_add(
                    weight.saturating_mul(AGRITECH_MULTIPLIER) / 100
                );
                total_contributions = total_contributions.saturating_add(
                    breakdown.agritech.contribution_count as u128
                );
            }

            // Marine contributions
            if breakdown.marine.contribution_count > 0 {
                let weight = (breakdown.marine.contribution_count as u128)
                    .saturating_mul(breakdown.marine.avg_quality as u128);
                total_weighted_score = total_weighted_score.saturating_add(
                    weight.saturating_mul(MARINE_MULTIPLIER) / 100
                );
                total_contributions = total_contributions.saturating_add(
                    breakdown.marine.contribution_count as u128
                );
            }

            // Education contributions
            if breakdown.education.contribution_count > 0 {
                let weight = (breakdown.education.contribution_count as u128)
                    .saturating_mul(breakdown.education.avg_quality as u128);
                total_weighted_score = total_weighted_score.saturating_add(
                    weight.saturating_mul(EDUCATION_MULTIPLIER) / 100
                );
                total_contributions = total_contributions.saturating_add(
                    breakdown.education.contribution_count as u128
                );
            }

            // Tech contributions
            if breakdown.tech.contribution_count > 0 {
                let weight = (breakdown.tech.contribution_count as u128)
                    .saturating_mul(breakdown.tech.avg_quality as u128);
                total_weighted_score = total_weighted_score.saturating_add(
                    weight.saturating_mul(TECH_MULTIPLIER) / 100
                );
                total_contributions = total_contributions.saturating_add(
                    breakdown.tech.contribution_count as u128
                );
            }

            // General contributions
            if breakdown.general.contribution_count > 0 {
                let weight = (breakdown.general.contribution_count as u128)
                    .saturating_mul(breakdown.general.avg_quality as u128);
                total_weighted_score = total_weighted_score.saturating_add(
                    weight.saturating_mul(GENERAL_MULTIPLIER) / 100
                );
                total_contributions = total_contributions.saturating_add(
                    breakdown.general.contribution_count as u128
                );
            }

            // Calculate average multiplier
            if total_contributions > 0 {
                total_weighted_score / total_contributions
            } else {
                GENERAL_MULTIPLIER // Default to 1.0x if no contributions
            }
        }

        /// Evaluate model contribution quality based on delta size and structure
        fn evaluate_model_quality(encrypted_delta: &[u8]) -> u8 {
            // Quality scoring based on encrypted delta characteristics:
            // - Larger deltas indicate more model parameters updated (more work done)
            // - Very small deltas may indicate trivial/no-op contributions
            // - Score capped at 100
            let delta_size_score = match encrypted_delta.len() {
                0..=31 => 10u32,      // Suspiciously small — likely no real computation
                32..=127 => 30,       // Minimal update
                128..=511 => 60,      // Moderate update
                512..=1024 => 80,     // Substantial update
                _ => 80,              // Capped (BoundedVec already limits to 1024)
            };

            // Entropy check: if all bytes are the same, it's trivially generated
            let first_byte = encrypted_delta.first().copied().unwrap_or(0);
            let entropy_penalty = if encrypted_delta.iter().all(|&b| b == first_byte) {
                30u32 // Penalize trivial/constant data
            } else {
                0
            };

            delta_size_score.saturating_sub(entropy_penalty).min(100) as u8
        }

        /// Calculate timeliness score based on submission time
        fn calculate_timeliness_score(submission_block: BlockNumberFor<T>, deadline: u32) -> u8 {
            let deadline_block = BlockNumberFor::<T>::from(deadline);
            
            if submission_block <= deadline_block {
                let submission_u64: u64 = TryInto::<u64>::try_into(submission_block).unwrap_or(0);
                let deadline_u64: u64 = TryInto::<u64>::try_into(deadline_block).unwrap_or(0);
                let blocks_remaining_u64 = deadline_u64.saturating_sub(submission_u64);
                let total_blocks = BlockNumberFor::<T>::from(deadline);
                
                // Earlier submissions get higher scores
                // SAFETY: BlockNumber fits in u32 (runtime uses u32 block numbers)
                let ratio = blocks_remaining_u64 * 100 / total_blocks.saturated_into::<u32>() as u64;
                ratio.min(100) as u8
            } else {
                0 // Late submission
            }
        }

        /// Calculate final reward for validator based on contribution scores (Phase 2.2 Enhanced)
        /// New formula: FL Quality(25%) + FL Timeliness(20%) + FL Honesty(20%) + Quantum(30%) + Bonus(5%)
        #[allow(clippy::type_complexity)]
        fn calculate_validator_reward(
            validator_info: &ValidatorInfo<T::AccountId, <T::Currency as Currency<T::AccountId>>::Balance, BlockNumberFor<T>>,
            base_reward: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> <T::Currency as Currency<T::AccountId>>::Balance {
            // Updated PoUW weights (Phase 2.2)
            let quality_weight = Perbill::from_percent(25);      // Reduced from 40%
            let timeliness_weight = Perbill::from_percent(20);   // Reduced from 30%
            let honesty_weight = Perbill::from_percent(20);      // Reduced from 30%
            let quantum_weight = Perbill::from_percent(30);      // NEW: Quantum computing contribution
            let bonus_weight = Perbill::from_percent(5);         // Stake/uptime bonus

            // Calculate FL contribution rewards
            let quality_reward = quality_weight * Perbill::from_percent(validator_info.quality_score as u32) * base_reward;
            let timeliness_reward = timeliness_weight * Perbill::from_percent(validator_info.timeliness_score as u32) * base_reward;
            let honesty_reward = honesty_weight * Perbill::from_percent(validator_info.honesty_score as u32) * base_reward;

            // Calculate quantum contribution reward
            let quantum_stats = ValidatorQuantumStatsMap::<T>::get(&validator_info.account);
            let quantum_reward = quantum_weight * Perbill::from_percent(quantum_stats.quantum_score as u32) * base_reward;

            // Calculate stake/uptime bonus (simplified - in production would track uptime)
            let stake_bonus = bonus_weight * base_reward;

            // Combine all rewards
            
            quality_reward
                .saturating_add(timeliness_reward)
                .saturating_add(honesty_reward)
                .saturating_add(quantum_reward)
                .saturating_add(stake_bonus)
        }

        /// Slash validator for violations
        pub fn slash_validator(
            validator: &T::AccountId,
            slash_percentage: Perbill,
            reason: SlashReason,
        ) -> DispatchResult {
            if let Some(mut validator_info) = Validators::<T>::get(validator) {
            let slash_amount = slash_percentage * validator_info.stake;
            
            // Reduce stake
            validator_info.stake = validator_info.stake.saturating_sub(slash_amount);
            
            // Slash from free balance (lock-only model, no reserved balance)
            let (_imbalance, _remaining) = T::Currency::slash(validator, slash_amount);
            // Update the lock to reflect reduced stake
            T::Currency::set_lock(
                STAKING_ID,
                validator,
                validator_info.stake,
                frame_support::traits::WithdrawReasons::all(),
            );                // Update slashing record
                let current_slashes = SlashingSpans::<T>::get(validator);
                SlashingSpans::<T>::insert(validator, current_slashes.saturating_add(1));
                
                // Update validator info
                Validators::<T>::insert(validator, validator_info);
                
                Self::deposit_event(Event::ValidatorSlashed {
                    validator: validator.clone(),
                    slash_amount,
                    reason: reason.as_u8(),
                });
            }
            
            Ok(())
        }
    }

    // Migration intentionally deferred: ValidatorCount is maintained forward-only; a
    // one-off migration to backfill count can be added in a future version when needed.
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn join_validators() -> Weight;
    fn leave_validators() -> Weight;
    fn withdraw_unbonded() -> Weight;
    fn submit_model_delta() -> Weight;
    fn assign_fl_task() -> Weight;
    fn distribute_rewards() -> Weight;
    fn record_quantum_contribution() -> Weight;
    fn record_domain_contribution() -> Weight;
    fn claim_pouw_with_domain_bonus() -> Weight;
    fn report_validator_offense() -> Weight;
}

impl WeightInfo for () {
    fn join_validators() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn leave_validators() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn withdraw_unbonded() -> Weight {
        Weight::from_parts(8_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn submit_model_delta() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn assign_fl_task() -> Weight {
        Weight::from_parts(5_000_000, 0)
        .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn distribute_rewards() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn record_quantum_contribution() -> Weight {
        Weight::from_parts(12_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // Check validator + check job not recorded
            .saturating_add(RocksDbWeight::get().writes(3))  // Store contribution + update stats + increment counter
    }
    fn record_domain_contribution() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))  // Read operator stats
            .saturating_add(RocksDbWeight::get().writes(2))  // Update operator stats + epoch stats
    }
    fn claim_pouw_with_domain_bonus() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // Read validator info + domain stats
            .saturating_add(RocksDbWeight::get().writes(1))  // Mint rewards
    }
    fn report_validator_offense() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // Read validator + slashing spans
            .saturating_add(RocksDbWeight::get().writes(3))  // Update validator + slashing spans + balance
    }
}