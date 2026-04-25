#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Quantum Computing Pallet
//!
//! This pallet enables blockchain-native quantum computing integration:
//! 1. On-chain quantum job registry and tracking
//! 2. Quantum result verification with zero-knowledge proofs
//! 3. Achievement NFT minting for quantum milestones
//! 4. Integration with Proof of Useful Work consensus
//! 5. Decentralized quantum resource marketplace

use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, Get, ExistenceRequirement, WithdrawReasons,
    },
    BoundedVec,
    weights::{Weight, constants::RocksDbWeight},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{SaturatedConversion, Saturating, Zero},
};
use scale_info::TypeInfo;

pub use pallet::*;

pub mod weights;

/// Maximum quantum job ID length (UUID format: "550e8400-e29b-41d4-a716-446655440000")
pub const MAX_JOB_ID_LENGTH: u32 = 64;

/// Maximum backend name length
pub const MAX_BACKEND_LENGTH: u32 = 32;

/// Maximum verification proof size (computation commitment hash)
/// NOTE: This is a hash commitment, NOT a zero-knowledge proof.
/// Full ZK-SNARK verification is roadmapped for 2028.
/// Current verification uses multi-validator consensus (Phase 2.3).
pub const MAX_PROOF_SIZE: u32 = 256;

/// Maximum metadata URI length (IPFS/Arweave CID)
pub const MAX_METADATA_URI_LENGTH: u32 = 256;

/// Maximum result data size stored on-chain (hash only, full data off-chain)
pub const MAX_RESULT_HASH_SIZE: u32 = 32;

/// Maximum circuit size (OpenQASM circuit definition)
pub const MAX_CIRCUIT_SIZE: u32 = 4096;

/// Maximum result size stored on-chain
pub const MAX_RESULT_SIZE: u32 = 2048;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::prelude::*;
    extern crate alloc;
    use alloc::format;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for quantum job payments
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Maximum number of active quantum jobs per account
        #[pallet::constant]
        type MaxActiveJobs: Get<u32>;

        /// Cost per qubit in quantum circuit (in DALLA smallest unit)
        #[pallet::constant]
        type DallaPerQubit: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Cost per shot in quantum execution (in DALLA smallest unit)
        #[pallet::constant]
        type DallaPerShot: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Fee for minting achievement NFT (in DALLA smallest unit)
        #[pallet::constant]
        type NFTMintingFee: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Treasury account for marketplace fees (Q-1 FIX)
        #[pallet::constant]
        type Treasury: Get<Self::AccountId>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    // ========== TYPE DEFINITIONS (Inside pallet module for Substrate v42 compatibility) ==========

    /// Status of a quantum job
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum JobStatus {
        Pending,
        Running,
        Completed,
        Failed,
        Cancelled,
    }

    /// Verification status for quantum results
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum VerificationStatus {
        Unverified,
        Verifying,
        Verified,
        Failed,
    }

    /// Quantum backend types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum QuantumBackend {
        AzureIonQ,
        AzureQuantinuum,
        AzureRigetti,
        IBMQuantum,
        Qiskit,
        SpinQGemini,
        SpinQTriangulum,
        Other,
    }

    /// Achievement types for quantum NFTs
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum AchievementType {
        FirstQuantumJob,
        GroverAlgorithm,
        ShorAlgorithm,
        QuantumFourierTransform,
        VQEAlgorithm,
        QAOAAlgorithm,
        Accuracy95,
        Accuracy99,
        VolumeContributor100,
        VolumeContributor1000,
        ErrorMitigationChampion,
        Custom,
    }

    /// Vote for verification consensus
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum VerificationVote {
        Approve,
        Reject,
        Abstain,
    }

    /// NFT Rarity Tiers
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum NFTRarity {
        Common,
        Rare,
        Epic,
        Legendary,
    }

    /// NFT Categories
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum NFTCategory {
        Volume,
        Accuracy,
        Complexity,
        Speed,
        Algorithm,
        Special,
    }

    /// Cross-Chain Destination
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum ChainDestination {
        Ethereum,
        Polkadot,
        Kusama,
        Parachain(u32),
        BelizeChain,
    }

    impl QuantumBackend {
        /// Convert backend enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                QuantumBackend::AzureIonQ => 0,
                QuantumBackend::AzureQuantinuum => 1,
                QuantumBackend::AzureRigetti => 2,
                QuantumBackend::IBMQuantum => 3,
                QuantumBackend::Qiskit => 4,
                QuantumBackend::SpinQGemini => 5,
                QuantumBackend::SpinQTriangulum => 6,
                QuantumBackend::Other => 7,
            }
        }

        /// Convert backend enum to string representation
        pub fn as_str(&self) -> &'static str {
            match self {
                QuantumBackend::AzureIonQ => "azure_ionq",
                QuantumBackend::AzureQuantinuum => "azure_quantinuum",
                QuantumBackend::AzureRigetti => "azure_rigetti",
                QuantumBackend::IBMQuantum => "ibm_quantum",
                QuantumBackend::Qiskit => "qiskit",
                QuantumBackend::SpinQGemini => "spinq_gemini",
                QuantumBackend::SpinQTriangulum => "spinq_triangulum",
                QuantumBackend::Other => "other",
            }
        }

        /// Parse backend from string (legacy helper - use FromStr trait instead)
        #[allow(clippy::should_implement_trait)]
        pub fn from_str(s: &str) -> Option<Self> {
            match s {
                "azure_ionq" => Some(QuantumBackend::AzureIonQ),
                "azure_quantinuum" => Some(QuantumBackend::AzureQuantinuum),
                "azure_rigetti" => Some(QuantumBackend::AzureRigetti),
                "ibm_quantum" => Some(QuantumBackend::IBMQuantum),
                "qiskit" => Some(QuantumBackend::Qiskit),
                "spinq_gemini" => Some(QuantumBackend::SpinQGemini),
                "spinq_triangulum" => Some(QuantumBackend::SpinQTriangulum),
                _ => Some(QuantumBackend::Other),
            }
        }
    }

    impl JobStatus {
        /// Convert status enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                JobStatus::Pending => 0,
                JobStatus::Running => 1,
                JobStatus::Completed => 2,
                JobStatus::Failed => 3,
                JobStatus::Cancelled => 4,
            }
        }
    }

    impl VerificationStatus {
        /// Convert verification status enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                VerificationStatus::Unverified => 0,
                VerificationStatus::Verifying => 1,
                VerificationStatus::Verified => 2,
                VerificationStatus::Failed => 3,
            }
        }
    }

    impl AchievementType {
        /// Convert achievement type enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                AchievementType::FirstQuantumJob => 0,
                AchievementType::GroverAlgorithm => 1,
                AchievementType::ShorAlgorithm => 2,
                AchievementType::QuantumFourierTransform => 3,
                AchievementType::VQEAlgorithm => 4,
                AchievementType::QAOAAlgorithm => 5,
                AchievementType::Accuracy95 => 6,
                AchievementType::Accuracy99 => 7,
                AchievementType::VolumeContributor100 => 8,
                AchievementType::VolumeContributor1000 => 9,
                AchievementType::ErrorMitigationChampion => 10,
                AchievementType::Custom => 11,
            }
        }
    }

    impl VerificationVote {
        /// Convert verification vote enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                VerificationVote::Approve => 0,
                VerificationVote::Reject => 1,
                VerificationVote::Abstain => 2,
            }
        }
    }

    impl ChainDestination {
        /// Convert chain destination enum to u8 index
        pub fn to_index(&self) -> u8 {
            match self {
                ChainDestination::Ethereum => 0,
                ChainDestination::Polkadot => 1,
                ChainDestination::Kusama => 2,
                ChainDestination::Parachain(_) => 3,
                ChainDestination::BelizeChain => 4,
            }
        }
    }

    /// On-chain quantum job record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct QuantumJob<AccountId, Balance, BlockNumber> {
        /// Unique job identifier (UUID from Kinich)
        pub job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        /// Account that submitted the job
        pub submitter: AccountId,
        /// Quantum backend used for execution
        pub backend: QuantumBackend,
        /// Hash of quantum circuit (SHA-256)
        pub circuit_hash: [u8; 32],
        /// Number of qubits in circuit
        pub num_qubits: u16,
        /// Circuit depth
        pub circuit_depth: u32,
        /// Number of shots requested
        pub num_shots: u32,
        /// Current job status
        pub status: JobStatus,
        /// Block number when job was submitted
        pub submission_time: BlockNumber,
        /// Block number when job completed (if completed)
        pub completion_time: Option<BlockNumber>,
        /// Hash of result data (SHA-256, full data stored off-chain)
        pub result_hash: Option<[u8; 32]>,
        /// Verification status
        pub verification_status: VerificationStatus,
        /// Cost in DALLA tokens
        pub dalla_cost: Balance,
        /// Account that executed the job (validator)
        pub executor: Option<AccountId>,
    }

    /// Quantum result record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct QuantumResult {
        /// Job identifier
        pub job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        /// Hash of complete result JSON (stored off-chain in IPFS/Arweave)
        pub result_data_hash: [u8; 32],
        /// Computation commitment: hash(result_data || executor_key || job_params)
        /// NOT a zero-knowledge proof — used for off-chain audit verification.
        /// Real verification happens via multi-validator consensus (Phase 2.3).
        pub verification_proof: BoundedVec<u8, ConstU32<MAX_PROOF_SIZE>>,
        /// Accuracy score after error mitigation (0-100)
        pub accuracy_score: u8,
        /// Validator/executor account
        pub validator: BoundedVec<u8, ConstU32<32>>, // AccountId as bytes
        /// Block number when result was recorded
        pub recorded_at: u32,
    }

    /// Quantum achievement NFT (Phase 2.3 Enhanced)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct QuantumAchievement<AccountId, BlockNumber> {
        /// Unique NFT identifier
        pub nft_id: u64,
        /// Associated quantum job
        pub job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        /// Achievement type
        pub achievement_type: AchievementType,
        /// NFT owner
        pub owner: AccountId,
        /// Original minter — immutable after creation; royalties go here (Q-2 FIX)
        pub original_minter: AccountId,
        /// Metadata URI (IPFS/Arweave)
        pub metadata_uri: BoundedVec<u8, ConstU32<MAX_METADATA_URI_LENGTH>>,
        /// Block when NFT was minted
        pub minted_at: BlockNumber,
        /// Whether NFT is transferable
        pub transferable: bool,
        /// NFT rarity tier (Phase 2.3.2)
        pub rarity: NFTRarity,
        /// NFT category (Phase 2.3.2)
        pub category: NFTCategory,
        /// Rarity score (0-1000) for uniqueness
        pub rarity_score: u32,
        /// Number of qubits in associated circuit
        pub circuit_qubits: u16,
        /// Accuracy score of result (0-100)
        pub accuracy: u8,
    }

    // ========== PHASE 2.3: MULTI-VALIDATOR VERIFICATION ==========

    /// Individual validator's verification
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorVerification {
        /// Validator account (as bytes for storage)
        pub validator: BoundedVec<u8, ConstU32<32>>,
        /// Verification vote
        pub vote: VerificationVote,
        /// Confidence score (0-100)
        pub confidence: u8,
        /// Block when verification was submitted
        pub submitted_at: u32,
        /// Hash of validator's independent result
        pub result_hash: [u8; 32],
    }

    /// Verification request for multi-validator consensus
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct VerificationRequest {
        /// Job being verified
        pub job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        /// Required number of verifications (default: 3)
        pub required_verifications: u8,
        /// Validators who have verified
        pub verifications: BoundedVec<ValidatorVerification, ConstU32<10>>,
        /// Number of approval votes
        pub approvals: u8,
        /// Number of rejection votes
        pub rejections: u8,
        /// Whether consensus has been reached
        pub consensus_reached: bool,
        /// Final consensus result (if reached)
        pub consensus_result: Option<bool>,
        /// Block when request was created
        pub created_at: u32,
        /// Deadline for verification (blocks)
        pub deadline: u32,
    }

    /// NFT Marketplace Listing (Phase 2.3.3)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct NFTListing<AccountId, Balance, BlockNumber> {
        /// NFT being listed
        pub nft_id: u64,
        /// Original NFT owner (for royalties)
        pub original_minter: AccountId,
        /// Current seller
        pub seller: AccountId,
        /// Fixed sale price
        pub price: Balance,
        /// Block number when listing expires
        pub expiry: BlockNumber,
        /// Block when listed
        pub listed_at: BlockNumber,
    }

    /// NFT Auction (Phase 2.3.3)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct AuctionListing<AccountId, Balance, BlockNumber> {
        /// NFT being auctioned
        pub nft_id: u64,
        /// Original NFT owner (for royalties)
        pub original_minter: AccountId,
        /// Auction creator
        pub seller: AccountId,
        /// Starting bid price
        pub start_price: Balance,
        /// Current highest bid
        pub current_bid: Balance,
        /// Current highest bidder (if any)
        pub current_bidder: Option<AccountId>,
        /// Block when auction ends
        pub end_block: BlockNumber,
        /// Block when auction started
        pub started_at: BlockNumber,
    }

    /// Bridge Request (Phase 2.3.4)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct BridgeRequest<AccountId, BlockNumber> {
        /// NFT being bridged
        pub nft_id: u64,
        /// Original owner on source chain
        pub owner: AccountId,
        /// Destination chain
        pub destination: ChainDestination,
        /// Recipient address on destination (encoded as bytes)
        pub recipient: BoundedVec<u8, ConstU32<64>>,
        /// Bridge request timestamp
        pub requested_at: BlockNumber,
        /// Whether bridge has been claimed on destination
        pub claimed: bool,
        /// Bridge claim transaction hash (if claimed)
        pub claim_tx_hash: Option<[u8; 32]>,
    }

    // Type aliases to reduce complexity
    pub type JobId = BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>;
    pub type QuantumJobOf<T> = QuantumJob<
        <T as frame_system::Config>::AccountId,
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        BlockNumberFor<T>
    >;
    pub type NFTListingOf<T> = NFTListing<
        <T as frame_system::Config>::AccountId,
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        BlockNumberFor<T>
    >;
    pub type AuctionListingOf<T> = AuctionListing<
        <T as frame_system::Config>::AccountId,
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        BlockNumberFor<T>
    >;

    /// Storage: Quantum jobs by job ID
    #[pallet::storage]
    #[pallet::getter(fn quantum_jobs)]
    pub type QuantumJobs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        JobId,
        QuantumJobOf<T>,
    >;

    /// Storage: Jobs by submitter account
    #[pallet::storage]
    #[pallet::getter(fn jobs_by_account)]
    pub type JobsByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, ConstU32<100>>,
        ValueQuery,
    >;

    /// Storage: Quantum results by job ID
    #[pallet::storage]
    #[pallet::getter(fn quantum_results)]
    pub type QuantumResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        QuantumResult,
    >;

    /// Storage: Quantum achievement NFTs
    #[pallet::storage]
    #[pallet::getter(fn quantum_achievements)]
    pub type QuantumAchievements<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // NFT ID
        QuantumAchievement<T::AccountId, BlockNumberFor<T>>,
    >;

    /// Storage: NFT counter for unique IDs
    #[pallet::storage]
    #[pallet::getter(fn nft_counter)]
    pub type NFTCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Storage: Total quantum jobs submitted
    #[pallet::storage]
    #[pallet::getter(fn total_quantum_jobs)]
    pub type TotalQuantumJobs<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Storage: Total DALLA spent on quantum computing
    #[pallet::storage]
    #[pallet::getter(fn total_dalla_spent)]
    pub type TotalDallaSpent<T: Config> = StorageValue<
        _,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    /// Storage: Account quantum statistics
    #[pallet::storage]
    #[pallet::getter(fn account_stats)]
    pub type AccountStats<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        QuantumAccountStats<<T::Currency as Currency<T::AccountId>>::Balance>,
        ValueQuery,
    >;

    /// Quantum computing statistics per account
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
    pub struct QuantumAccountStats<Balance> {
        /// Total jobs submitted
        pub total_jobs: u32,
        /// Completed jobs
        pub completed_jobs: u32,
        /// Failed jobs
        pub failed_jobs: u32,
        /// Total DALLA spent
        pub total_spent: Balance,
        /// Total qubits used
        pub total_qubits: u64,
        /// Total shots executed
        pub total_shots: u64,
        /// NFTs earned
        pub nfts_earned: u32,
    }

    // ========== PHASE 2.3: NEW STORAGE ITEMS ==========

    /// Storage: Verification requests for multi-validator consensus
    #[pallet::storage]
    #[pallet::getter(fn verification_requests)]
    pub type VerificationRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        VerificationRequest,
    >;

    /// Storage: Validator verification reputation scores
    #[pallet::storage]
    #[pallet::getter(fn validator_reputation)]
    pub type ValidatorReputation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Reputation score (0-1000)
        ValueQuery,
    >;

    /// Storage: Job counter for unique IDs
    #[pallet::storage]
    #[pallet::getter(fn job_counter)]
    pub type JobCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Storage: NFT marketplace listings (Phase 2.3.3)
    #[pallet::storage]
    #[pallet::getter(fn nft_listings)]
    pub type NFTListings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // NFT ID
        NFTListingOf<T>,
    >;

    /// Storage: NFT auctions (Phase 2.3.3)
    #[pallet::storage]
    #[pallet::getter(fn nft_auctions)]
    pub type NFTAuctions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // NFT ID
        AuctionListingOf<T>,
    >;

    /// Storage: Listing counter for marketplace
    #[pallet::storage]
    #[pallet::getter(fn listing_counter)]
    pub type ListingCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Storage: Cross-chain bridge requests (Phase 2.3.4)
    #[pallet::storage]
    #[pallet::getter(fn bridge_requests)]
    pub type BridgeRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // NFT ID
        BridgeRequest<T::AccountId, BlockNumberFor<T>>,
    >;

    /// Storage: Bridge counter for tracking requests
    #[pallet::storage]
    #[pallet::getter(fn bridge_counter)]
    pub type BridgeCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Quantum job submitted to blockchain
        QuantumJobSubmitted {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            submitter: T::AccountId,
            backend_index: u8,  // Index into QuantumBackend enum
            dalla_cost: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Quantum job status updated
        JobStatusUpdated {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            status_index: u8,  // Index into JobStatus enum
        },
        /// Quantum result recorded on-chain
        QuantumResultRecorded {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            executor: T::AccountId,
            accuracy_score: u8,
        },
        /// Quantum result verified
        ResultVerified {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            verification_status_index: u8,  // Index into VerificationStatus enum
        },
        /// Achievement NFT minted
        AchievementNFTMinted {
            nft_id: u64,
            owner: T::AccountId,
            achievement_type_index: u8,  // Index into AchievementType enum
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
        },
        /// NFT transferred to new owner
        NFTTransferred {
            nft_id: u64,
            from: T::AccountId,
            to: T::AccountId,
        },
        // ========== PHASE 2.3: NEW EVENTS ==========
        /// Verification request created for multi-validator consensus
        VerificationRequested {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            required_verifications: u8,
            deadline: u32,
        },
        /// Validator submitted verification vote
        VerificationSubmitted {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            validator: T::AccountId,
            vote_index: u8,  // Index into VerificationVote enum
            confidence: u8,
        },
        /// Verification consensus reached
        VerificationConsensusReached {
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            result: bool, // true = approved, false = rejected
            approvals: u8,
            rejections: u8,
        },
        /// Validator reputation updated
        ReputationUpdated {
            validator: T::AccountId,
            new_reputation: u32,
        },
        /// NFT listed on marketplace (Phase 2.3.3)
        NFTListed {
            nft_id: u64,
            seller: T::AccountId,
            price: <T::Currency as Currency<T::AccountId>>::Balance,
            expiry: BlockNumberFor<T>,
        },
        /// NFT purchased from marketplace (Phase 2.3.3)
        NFTPurchased {
            nft_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: <T::Currency as Currency<T::AccountId>>::Balance,
            royalty: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// NFT delisted from marketplace (Phase 2.3.3)
        NFTDelisted {
            nft_id: u64,
            seller: T::AccountId,
        },
        // AuctionCreated, BidPlaced, AuctionFinalized, BridgeClaimed
        // removed (E-7): orphaned events, never emitted via deposit_event.
        /// NFT bridge initiated (Phase 2.3.4)
        BridgeInitiated {
            nft_id: u64,
            owner: T::AccountId,
            destination_index: u8,  // Index into ChainDestination enum
            recipient: BoundedVec<u8, ConstU32<64>>,
        },
        /// NFT bridge cancelled by owner (Phase 2.3.4)
        BridgeCancelled {
            nft_id: u64,
            owner: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Job ID already exists
        JobAlreadyExists,
        /// Job not found
        JobNotFound,
        /// Result not found
        ResultNotFound,
        /// NFT not found
        NFTNotFound,
        /// Invalid job status transition
        InvalidStatusTransition,
        /// Insufficient balance for quantum job
        InsufficientBalance,
        /// Maximum active jobs reached
        MaxActiveJobsReached,
        /// Invalid verification proof
        InvalidVerificationProof,
        /// Not authorized to perform action
        NotAuthorized,
        /// NFT is not transferable
        NFTNotTransferable,
        /// Invalid circuit parameters
        InvalidCircuitParameters,
        /// Job already completed
        JobAlreadyCompleted,
        /// Result already recorded
        ResultAlreadyRecorded,
        // ========== PHASE 2.3: NEW ERRORS ==========
        /// Verification request not found
        VerificationRequestNotFound,
        /// Verification request already exists
        VerificationRequestAlreadyExists,
        /// Validator already verified this job
        ValidatorAlreadyVerified,
        /// Verification deadline passed
        VerificationDeadlinePassed,
        /// Consensus already reached
        ConsensusAlreadyReached,
        /// Insufficient validators for consensus
        InsufficientValidators,
        /// Invalid verification vote
        InvalidVerificationVote,
        // ========== PHASE 2.3.3: MARKETPLACE ERRORS ==========
        /// NFT already listed or in auction
        NFTAlreadyListed,
        /// NFT listing not found
        ListingNotFound,
        /// Auction not found
        AuctionNotFound,
        /// Auction has already ended
        AuctionEnded,
        /// Auction is still active
        AuctionStillActive,
        /// Bid too low
        BidTooLow,
        /// Cannot buy own NFT
        CannotBuyOwnNFT,
        /// Listing expired
        ListingExpired,
        /// Invalid metadata URI
        InvalidMetadataURI,
        // ========== PHASE 2.3.4: BRIDGE ERRORS ==========
        /// Bridge request not found
        BridgeRequestNotFound,
        /// Bridge already initiated for this NFT
        BridgeAlreadyInitiated,
        /// Invalid recipient address
        InvalidRecipientAddress,
        /// NFT locked in bridge
        NFTLockedInBridge,
        /// Bridge already claimed on destination chain
        BridgeAlreadyClaimed,
        /// Arithmetic overflow in financial calculation
        ArithmeticOverflow,
        /// Caller does not have sufficient reputation
        InsufficientReputation,
        /// Executor or submitter cannot verify their own job
        ExecutorCannotVerify,
        /// AUDIT FIX (C-Q1): Job has no executor assigned — cannot pay
        ExecutorNotAssigned,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a quantum job to the blockchain
        ///
        /// This extrinsic records a quantum job submission on-chain, reserves DALLA
        /// payment, and emits an event for off-chain quantum executors.
        ///
        /// # Arguments
        /// * `origin` - The submitter account
        /// * `job_id` - Unique job identifier (from Kinich)
        /// * `backend` - Quantum backend to use
        /// * `circuit_hash` - SHA-256 hash of quantum circuit
        /// * `num_qubits` - Number of qubits in circuit
        /// * `circuit_depth` - Depth of circuit
        /// * `num_shots` - Number of measurements
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_quantum_job())]
        pub fn submit_quantum_job(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            backend_index: u8,  // Index into QuantumBackend enum
            circuit_hash: [u8; 32],
            num_qubits: u16,
            circuit_depth: u32,
            num_shots: u32,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            
            // Convert index to enum with validation
            let backend = match backend_index {
                0 => QuantumBackend::AzureIonQ,
                1 => QuantumBackend::AzureQuantinuum,
                2 => QuantumBackend::AzureRigetti,
                3 => QuantumBackend::IBMQuantum,
                4 => QuantumBackend::Qiskit,
                5 => QuantumBackend::SpinQGemini,
                6 => QuantumBackend::SpinQTriangulum,
                _ => QuantumBackend::Other,
            };

            // Ensure job doesn't already exist
            ensure!(!QuantumJobs::<T>::contains_key(&job_id), Error::<T>::JobAlreadyExists);

            // Validate circuit parameters
            ensure!(num_qubits > 0 && num_qubits <= 100, Error::<T>::InvalidCircuitParameters);
            ensure!(num_shots > 0 && num_shots <= 1_000_000, Error::<T>::InvalidCircuitParameters);

            // Calculate cost: (qubits * qubit_cost) + (shots * shot_cost)
            let qubit_cost = T::DallaPerQubit::get()
                .saturating_mul((num_qubits as u32).into());
            let shot_cost = T::DallaPerShot::get()
                .saturating_mul(num_shots.into());
            let total_cost = qubit_cost.saturating_add(shot_cost);

            // Reserve payment from submitter
            T::Currency::reserve(&submitter, total_cost)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // Create quantum job record
            let current_block = frame_system::Pallet::<T>::block_number();
            let quantum_job = QuantumJob {
                job_id: job_id.clone(),
                submitter: submitter.clone(),
                backend: backend.clone(),
                circuit_hash,
                num_qubits,
                circuit_depth,
                num_shots,
                status: JobStatus::Pending,
                submission_time: current_block,
                completion_time: None,
                result_hash: None,
                verification_status: VerificationStatus::Unverified,
                dalla_cost: total_cost,
                executor: None,
            };

            // Store job
            QuantumJobs::<T>::insert(&job_id, quantum_job);

            // Add to submitter's job list
            JobsByAccount::<T>::try_mutate(&submitter, |jobs| {
                jobs.try_push(job_id.clone())
                    .map_err(|_| Error::<T>::MaxActiveJobsReached)
            })?;

            // Update statistics
            TotalQuantumJobs::<T>::mutate(|total| *total = total.saturating_add(1));
            TotalDallaSpent::<T>::mutate(|total| *total = total.saturating_add(total_cost));

            AccountStats::<T>::mutate(&submitter, |stats| {
                stats.total_jobs = stats.total_jobs.saturating_add(1);
                stats.total_spent = stats.total_spent.saturating_add(total_cost);
                stats.total_qubits = stats.total_qubits.saturating_add(num_qubits as u64);
                stats.total_shots = stats.total_shots.saturating_add(num_shots as u64);
            });

            Self::deposit_event(Event::QuantumJobSubmitted {
                job_id,
                submitter,
                backend_index: backend.to_index(),
                dalla_cost: total_cost,
            });

            Ok(())
        }

        /// Update quantum job status
        ///
        /// Only callable by the job executor or governance.
        ///
        /// # Arguments
        /// * `origin` - Executor or root
        /// * `job_id` - Job identifier
        /// * `new_status` - New job status
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_job_status())]
        pub fn update_job_status(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            new_status_index: u8,  // Index into JobStatus enum
        ) -> DispatchResult {
            // Convert index to enum with validation
            let new_status = match new_status_index {
                0 => JobStatus::Pending,
                1 => JobStatus::Running,
                2 => JobStatus::Completed,
                3 => JobStatus::Failed,
                _ => JobStatus::Cancelled,
            };
            let caller = ensure_signed(origin)?;

            QuantumJobs::<T>::try_mutate(&job_id, |maybe_job| {
                let job = maybe_job.as_mut().ok_or(Error::<T>::JobNotFound)?;

                // Authorization check: must be submitter, executor, or root
                ensure!(
                    caller == job.submitter || 
                    job.executor.as_ref() == Some(&caller),
                    Error::<T>::NotAuthorized
                );

                // Validate status transition
                match (&job.status, &new_status) {
                    (JobStatus::Pending, JobStatus::Running) => {
                        // Submitter or assigned executor can start the job
                        ensure!(
                            caller == job.submitter ||
                            job.executor.as_ref() == Some(&caller),
                            Error::<T>::NotAuthorized
                        );
                    },
                    (JobStatus::Running, JobStatus::Completed) => {
                        job.completion_time = Some(frame_system::Pallet::<T>::block_number());
                        
                        // Update account stats
                        AccountStats::<T>::mutate(&job.submitter, |stats| {
                            stats.completed_jobs = stats.completed_jobs.saturating_add(1);
                        });
                    },
                    (JobStatus::Running, JobStatus::Failed) => {
                        job.completion_time = Some(frame_system::Pallet::<T>::block_number());
                        
                        // Refund payment on failure
                        let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
                        
                        // Update account stats
                        AccountStats::<T>::mutate(&job.submitter, |stats| {
                            stats.failed_jobs = stats.failed_jobs.saturating_add(1);
                        });
                    },
                    (_, JobStatus::Cancelled) => {
                        // Only submitter can cancel, and only from Pending or Running
                        ensure!(caller == job.submitter, Error::<T>::NotAuthorized);
                        ensure!(
                            job.status == JobStatus::Pending || job.status == JobStatus::Running,
                            Error::<T>::InvalidStatusTransition
                        );
                        let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
                    },
                    _ => return Err(Error::<T>::InvalidStatusTransition.into()),
                }

                job.status = new_status.clone();

                Self::deposit_event(Event::JobStatusUpdated {
                    job_id: job_id.clone(),
                    status_index: new_status.to_index(),
                });

                Ok(())
            })
        }

        /// Record quantum result on-chain
        ///
        /// Stores the result hash and computation commitment for a completed quantum job.
        /// The result is NOT automatically verified — it enters `Verifying` status and
        /// must pass multi-validator consensus (Phase 2.3) or root verification before
        /// the executor receives payment.
        ///
        /// # Arguments
        /// * `origin` - Executor account
        /// * `job_id` - Job identifier
        /// * `result_data_hash` - SHA-256 hash of result JSON (stored off-chain)
        /// * `verification_proof` - Computation commitment hash for audit trail
        /// * `accuracy_score` - Accuracy after error mitigation (0-100)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::record_quantum_result())]
        pub fn record_quantum_result(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            result_data_hash: [u8; 32],
            verification_proof: BoundedVec<u8, ConstU32<MAX_PROOF_SIZE>>,
            accuracy_score: u8,
        ) -> DispatchResult {
            let executor = ensure_signed(origin)?;

            // Only accounts with minimum reputation can act as executors
            ensure!(
                ValidatorReputation::<T>::get(&executor) >= 100,
                Error::<T>::InsufficientReputation
            );

            // Validate computation commitment is structurally sound:
            // - Must be at least 32 bytes (a valid hash commitment)
            // - result_data_hash must not be all zeros (no empty result)
            // - accuracy_score must be in valid range
            ensure!(
                verification_proof.len() >= 32,
                Error::<T>::InvalidVerificationProof
            );
            ensure!(
                result_data_hash != [0u8; 32],
                Error::<T>::InvalidVerificationProof
            );
            ensure!(
                accuracy_score <= 100,
                Error::<T>::InvalidCircuitParameters
            );

            // Ensure job exists and is in correct status
            QuantumJobs::<T>::try_mutate(&job_id, |maybe_job| {
                let job = maybe_job.as_mut().ok_or(Error::<T>::JobNotFound)?;
                
                ensure!(job.status == JobStatus::Running || job.status == JobStatus::Completed, 
                    Error::<T>::InvalidStatusTransition);
                
                // Ensure result not already recorded
                ensure!(!QuantumResults::<T>::contains_key(&job_id), 
                    Error::<T>::ResultAlreadyRecorded);

                // Update job with result — stays in Verifying until multi-validator
                // consensus or root verification approves it
                job.result_hash = Some(result_data_hash);
                job.executor = Some(executor.clone());
                job.verification_status = VerificationStatus::Verifying;

                if job.status == JobStatus::Running {
                    job.status = JobStatus::Completed;
                    job.completion_time = Some(frame_system::Pallet::<T>::block_number());
                }

                Ok::<(), DispatchError>(())
            })?;

            // Create result record
            // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
            let quantum_result = QuantumResult {
                job_id: job_id.clone(),
                result_data_hash,
                verification_proof,
                accuracy_score,
                validator: executor.encode().try_into().unwrap_or_default(),
                recorded_at: current_block,
            };

            QuantumResults::<T>::insert(&job_id, quantum_result);

            Self::deposit_event(Event::QuantumResultRecorded {
                job_id,
                executor,
                accuracy_score,
            });

            Ok(())
        }

        /// Verify quantum result (root emergency override)
        ///
        /// Primary verification path is multi-validator consensus (Phase 2.3).
        /// This root-only function serves as emergency override for when:
        /// - Multi-validator consensus fails to reach quorum
        /// - Time-critical results need expedited verification
        ///
        /// In production, prefer `request_verification` + `submit_verification` flow.
        ///
        /// # Arguments
        /// * `origin` - Root only (emergency override)
        /// * `job_id` - Job identifier
        /// * `verification_passed` - Whether verification passed
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::verify_quantum_result())]
        pub fn verify_quantum_result(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            verification_passed: bool,
        ) -> DispatchResult {
            // Only root can use emergency verification override
            ensure_root(origin)?;

            // Ensure result exists before allowing verification
            ensure!(
                QuantumResults::<T>::contains_key(&job_id),
                Error::<T>::ResultNotFound
            );

            QuantumJobs::<T>::try_mutate(&job_id, |maybe_job| {
                let job = maybe_job.as_mut().ok_or(Error::<T>::JobNotFound)?;

                let new_status = if verification_passed {
                    VerificationStatus::Verified
                } else {
                    VerificationStatus::Failed
                };

                job.verification_status = new_status.clone();

                // If verification failed, refund the submitter
                if !verification_passed {
                    let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
                } else {
                    // AUDIT FIX (C-Q1): Reject if no executor assigned — prevents
                    // funds from being locked indefinitely with no payee.
                    let executor = job.executor.as_ref()
                        .ok_or(Error::<T>::ExecutorNotAssigned)?;
                    // If verified, transfer payment to executor
                    let _ = T::Currency::repatriate_reserved(
                        &job.submitter,
                        executor,
                        job.dalla_cost,
                        frame_support::traits::BalanceStatus::Free,
                    );
                }

                Self::deposit_event(Event::ResultVerified {
                    job_id: job_id.clone(),
                    verification_status_index: new_status.to_index(),
                });

                Ok(())
            })
        }

        /// Mint quantum achievement NFT
        ///
        /// Creates an NFT for quantum computing achievements.
        ///
        /// # Arguments
        /// * `origin` - NFT recipient
        /// * `job_id` - Associated quantum job
        /// * `achievement_type` - Type of achievement
        /// * `metadata_uri` - IPFS/Arweave URI for metadata
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::mint_achievement_nft())]
        pub fn mint_achievement_nft(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            achievement_type_index: u8,  // Index into AchievementType enum
            transferable: bool,
            _circuit_qubits: u16,
            _accuracy: u8,
        ) -> DispatchResult {
            // Convert index to enum with validation
            let achievement_type = match achievement_type_index {
                0 => AchievementType::FirstQuantumJob,
                1 => AchievementType::GroverAlgorithm,
                2 => AchievementType::ShorAlgorithm,
                3 => AchievementType::QuantumFourierTransform,
                4 => AchievementType::VQEAlgorithm,
                5 => AchievementType::QAOAAlgorithm,
                6 => AchievementType::Accuracy95,
                7 => AchievementType::Accuracy99,
                8 => AchievementType::VolumeContributor100,
                9 => AchievementType::VolumeContributor1000,
                10 => AchievementType::ErrorMitigationChampion,
                _ => AchievementType::Custom,
            };
            let owner = ensure_signed(origin)?;

            // Ensure job exists and belongs to owner
            let job = QuantumJobs::<T>::get(&job_id).ok_or(Error::<T>::JobNotFound)?;
            ensure!(job.submitter == owner || job.executor.as_ref() == Some(&owner), 
                Error::<T>::NotAuthorized);

            // Require job verification before minting
            ensure!(
                job.verification_status == VerificationStatus::Verified,
                Error::<T>::InvalidVerificationProof
            );

            // Use actual job data instead of caller-supplied values
            let circuit_qubits = job.num_qubits;
            let accuracy = QuantumResults::<T>::get(&job_id)
                .map(|r| r.accuracy_score)
                .unwrap_or(0);

            // Charge minting fee (burned as deflationary mechanism)
            // TODO: Route to treasury when Treasury config type is added
            let mint_fee = T::NFTMintingFee::get();
            let _imbalance = T::Currency::withdraw(
                &owner,
                mint_fee,
                WithdrawReasons::FEE,
                ExistenceRequirement::KeepAlive,
            ).map_err(|_| Error::<T>::InsufficientBalance)?;

            // Check if job was verified (Phase 2.3.2)
            let verification_approved = true; // Already enforced via ensure! above

            // Calculate NFT attributes using rarity algorithm (Phase 2.3.2)
            let (rarity, rarity_score, category) = Self::calculate_nft_attributes(
                &achievement_type,
                accuracy,
                circuit_qubits,
                verification_approved,
            );

            // Generate unique NFT ID with overflow protection
            let nft_id = NFTCounter::<T>::try_mutate(|counter| {
                let id = *counter;
                *counter = counter.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
                Ok::<u64, Error<T>>(id)
            })?;

            // Generate dynamic metadata URI (Phase 2.3.2)
            let metadata_uri = Self::generate_metadata_uri(
                nft_id,
                &achievement_type,
                &rarity,
                &category,
                rarity_score,
                circuit_qubits,
                accuracy,
            )?;

            // Create enhanced NFT (Phase 2.3.2)
            let current_block = frame_system::Pallet::<T>::block_number();
            let achievement = QuantumAchievement {
                nft_id,
                job_id: job_id.clone(),
                achievement_type: achievement_type.clone(),
                owner: owner.clone(),
                original_minter: owner.clone(), // Q-2 FIX: immutable original minter
                metadata_uri,
                minted_at: current_block,
                transferable,
                rarity: rarity.clone(),
                category: category.clone(),
                rarity_score,
                circuit_qubits,
                accuracy,
            };

            QuantumAchievements::<T>::insert(nft_id, achievement);

            // Update account stats
            AccountStats::<T>::mutate(&owner, |stats| {
                stats.nfts_earned = stats.nfts_earned.saturating_add(1);
            });

            Self::deposit_event(Event::AchievementNFTMinted {
                nft_id,
                owner,
                achievement_type_index: achievement_type.to_index(),
                job_id,
            });

            Ok(())
        }

        /// Transfer NFT to another account
        ///
        /// # Arguments
        /// * `origin` - Current NFT owner
        /// * `nft_id` - NFT identifier
        /// * `to` - Recipient account
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::transfer_nft())]
        pub fn transfer_nft(
            origin: OriginFor<T>,
            nft_id: u64,
            to: T::AccountId,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            // Ensure NFT is not locked in a bridge request
            ensure!(!BridgeRequests::<T>::contains_key(nft_id), Error::<T>::NFTLockedInBridge);

            QuantumAchievements::<T>::try_mutate(nft_id, |maybe_nft| {
                let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;

                // Check ownership
                ensure!(nft.owner == from, Error::<T>::NotAuthorized);

                // Check if transferable
                ensure!(nft.transferable, Error::<T>::NFTNotTransferable);

                // Transfer NFT
                nft.owner = to.clone();

                Self::deposit_event(Event::NFTTransferred {
                    nft_id,
                    from,
                    to,
                });

                Ok(())
            })
        }

        // ========== PHASE 2.3.3: NFT MARKETPLACE ==========

        /// List NFT for fixed-price sale
        ///
        /// # Arguments
        /// * `origin` - NFT owner
        /// * `nft_id` - NFT to list
        /// * `price` - Sale price in DALLA
        /// * `duration` - Blocks until expiry
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::list_nft())]
        pub fn list_nft(
            origin: OriginFor<T>,
            nft_id: u64,
            price: <T::Currency as Currency<T::AccountId>>::Balance,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            // Get NFT and verify ownership
            let nft = QuantumAchievements::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound)?;
            ensure!(nft.owner == seller, Error::<T>::NotAuthorized);
            ensure!(nft.transferable, Error::<T>::NFTNotTransferable);

            // Ensure not already listed or in auction
            ensure!(!NFTListings::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);
            ensure!(!NFTAuctions::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);

            // Create listing
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let duration_u64: u64 = TryInto::<u64>::try_into(duration).unwrap_or(0);
            let expiry_u64 = current_u64.saturating_add(duration_u64);
            // SAFETY(saturated_into): u64 → BlockNumberFor<T>. The expiry is derived from current block + duration, both bounded by chain lifetime.
            let expiry: BlockNumberFor<T> = expiry_u64.saturated_into();

            // Q-2 FIX: Use the immutable original_minter from the NFT struct
            let original_minter = nft.original_minter.clone();

            let listing = NFTListing {
                nft_id,
                original_minter,
                seller: seller.clone(),
                price,
                expiry,
                listed_at: current_block,
            };

            NFTListings::<T>::insert(nft_id, listing);
            ListingCounter::<T>::mutate(|c| *c = c.saturating_add(1));

            Self::deposit_event(Event::NFTListed {
                nft_id,
                seller,
                price,
                expiry,
            });

            Ok(())
        }

        /// Buy NFT from marketplace listing
        ///
        /// # Arguments
        /// * `origin` - Buyer
        /// * `nft_id` - NFT to purchase
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::buy_nft())]
        pub fn buy_nft(
            origin: OriginFor<T>,
            nft_id: u64,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            // Get listing
            let listing = NFTListings::<T>::get(nft_id).ok_or(Error::<T>::ListingNotFound)?;

            // Check expiry
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= listing.expiry, Error::<T>::ListingExpired);

            // Cannot buy own NFT
            ensure!(buyer != listing.seller, Error::<T>::CannotBuyOwnNFT);

            // Calculate fees
            let sale_price = listing.price;
            let royalty_rate = 5u32; // 5%
            let marketplace_fee_rate = 2u32; // 2%

            let royalty = sale_price.saturating_mul(royalty_rate.into()) / 100u32.into();
            let marketplace_fee = sale_price.saturating_mul(marketplace_fee_rate.into()) / 100u32.into();
            let seller_amount = sale_price
                .saturating_sub(royalty)
                .saturating_sub(marketplace_fee);

            // Q-1 FIX: Route marketplace fee to treasury account
            let treasury = T::Treasury::get();

            // Transfer seller amount
            T::Currency::transfer(&buyer, &listing.seller, seller_amount, ExistenceRequirement::KeepAlive)?;
            
            // Transfer marketplace fee to treasury
            if marketplace_fee > Zero::zero() {
                T::Currency::transfer(&buyer, &treasury, marketplace_fee, ExistenceRequirement::KeepAlive)?;
            }
            
            // Pay royalty to original minter if not seller
            if listing.original_minter != listing.seller {
                T::Currency::transfer(&buyer, &listing.original_minter, royalty, ExistenceRequirement::KeepAlive)?;
            } else {
                // Original minter is selling — add royalty back to seller
                T::Currency::transfer(&buyer, &listing.seller, royalty, ExistenceRequirement::KeepAlive)?;
            }

            // Transfer NFT
            QuantumAchievements::<T>::try_mutate(nft_id, |maybe_nft| {
                let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;
                nft.owner = buyer.clone();
                Ok::<(), Error<T>>(())
            })?;

            // Remove listing
            NFTListings::<T>::remove(nft_id);

            Self::deposit_event(Event::NFTPurchased {
                nft_id,
                seller: listing.seller,
                buyer,
                price: sale_price,
                royalty,
            });

            Ok(())
        }

        /// Delist NFT from marketplace
        ///
        /// # Arguments
        /// * `origin` - NFT owner/seller
        /// * `nft_id` - NFT to delist
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::delist_nft())]
        pub fn delist_nft(
            origin: OriginFor<T>,
            nft_id: u64,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            // Get listing
            let listing = NFTListings::<T>::get(nft_id).ok_or(Error::<T>::ListingNotFound)?;

            // Verify ownership
            ensure!(listing.seller == seller, Error::<T>::NotAuthorized);

            // Remove listing
            NFTListings::<T>::remove(nft_id);

            Self::deposit_event(Event::NFTDelisted {
                nft_id,
                seller,
            });

            Ok(())
        }

        // ========== PHASE 2.3.4: CROSS-CHAIN BRIDGE ==========

        /// Bridge NFT to Ethereum (ERC-721 compatible)
        ///
        /// Locks NFT on BelizeChain and emits event for bridge relayer.
        ///
        /// # Arguments
        /// * `origin` - NFT owner
        /// * `nft_id` - NFT to bridge
        /// * `recipient` - Ethereum address (20 bytes)
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::bridge_to_ethereum())]
        pub fn bridge_to_ethereum(
            origin: OriginFor<T>,
            nft_id: u64,
            recipient: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            // Verify NFT ownership
            let nft = QuantumAchievements::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound)?;
            ensure!(nft.owner == owner, Error::<T>::NotAuthorized);
            ensure!(nft.transferable, Error::<T>::NFTNotTransferable);

            // Ensure not listed or in auction
            ensure!(!NFTListings::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);
            ensure!(!NFTAuctions::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);

            // Ensure not already bridged
            ensure!(!BridgeRequests::<T>::contains_key(nft_id), Error::<T>::BridgeAlreadyInitiated);

            // Validate Ethereum address format (20 bytes)
            ensure!(recipient.len() == 20 || recipient.len() == 42, Error::<T>::InvalidRecipientAddress);

            // Create bridge request
            let current_block = frame_system::Pallet::<T>::block_number();
            let bridge_request = BridgeRequest {
                nft_id,
                owner: owner.clone(),
                destination: ChainDestination::Ethereum,
                recipient: recipient.clone(),
                requested_at: current_block,
                claimed: false,
                claim_tx_hash: None,
            };

            BridgeRequests::<T>::insert(nft_id, bridge_request);
            BridgeCounter::<T>::mutate(|c| *c = c.saturating_add(1));

            // Lock NFT by setting transferable to false — prevents transfer/list/auction
            QuantumAchievements::<T>::mutate(nft_id, |maybe_nft| {
                if let Some(nft) = maybe_nft {
                    nft.transferable = false;
                }
            });

            Self::deposit_event(Event::BridgeInitiated {
                nft_id,
                owner,
                destination_index: ChainDestination::Ethereum.to_index(),
                recipient,
            });

            Ok(())
        }

        /// Bridge NFT to Polkadot parachain via XCM
        ///
        /// Locks NFT on BelizeChain and sends XCM message to destination parachain.
        ///
        /// # Arguments
        /// * `origin` - NFT owner
        /// * `nft_id` - NFT to bridge
        /// * `parachain_id` - Destination parachain ID
        /// * `recipient` - Recipient account on parachain
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::bridge_to_parachain())]
        pub fn bridge_to_parachain(
            origin: OriginFor<T>,
            nft_id: u64,
            parachain_id: u32,
            recipient: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            // Verify NFT ownership
            let nft = QuantumAchievements::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound)?;
            ensure!(nft.owner == owner, Error::<T>::NotAuthorized);
            ensure!(nft.transferable, Error::<T>::NFTNotTransferable);

            // Ensure not listed or in auction
            ensure!(!NFTListings::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);
            ensure!(!NFTAuctions::<T>::contains_key(nft_id), Error::<T>::NFTAlreadyListed);

            // Ensure not already bridged
            ensure!(!BridgeRequests::<T>::contains_key(nft_id), Error::<T>::BridgeAlreadyInitiated);

            // Create bridge request
            let current_block = frame_system::Pallet::<T>::block_number();
            let bridge_request = BridgeRequest {
                nft_id,
                owner: owner.clone(),
                destination: ChainDestination::Parachain(parachain_id),
                recipient: recipient.clone(),
                requested_at: current_block,
                claimed: false,
                claim_tx_hash: None,
            };

            BridgeRequests::<T>::insert(nft_id, bridge_request);
            BridgeCounter::<T>::mutate(|c| *c = c.saturating_add(1));

            // Lock NFT by setting transferable to false — prevents transfer/list/auction
            QuantumAchievements::<T>::mutate(nft_id, |maybe_nft| {
                if let Some(nft) = maybe_nft {
                    nft.transferable = false;
                }
            });

            // In production: Send XCM message to parachain
            // xcm::send_xcm(destination, message)?;

            Self::deposit_event(Event::BridgeInitiated {
                nft_id,
                owner,
                destination_index: ChainDestination::Parachain(parachain_id).to_index(),
                recipient,
            });

            Ok(())
        }

        /// Cancel a pending bridge request and unlock the NFT
        ///
        /// Only the NFT owner can cancel. Cannot cancel if already claimed.
        ///
        /// # Arguments
        /// * `origin` - NFT owner
        /// * `nft_id` - Bridged NFT to cancel
        #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::cancel_bridge())]
        pub fn cancel_bridge(
            origin: OriginFor<T>,
            nft_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get and validate bridge request
            let bridge_request = BridgeRequests::<T>::get(nft_id)
                .ok_or(Error::<T>::BridgeRequestNotFound)?;
            ensure!(bridge_request.owner == who, Error::<T>::NotAuthorized);
            ensure!(!bridge_request.claimed, Error::<T>::BridgeAlreadyClaimed);

            // Remove bridge request
            BridgeRequests::<T>::remove(nft_id);

            // Unlock NFT by restoring transferable flag
            QuantumAchievements::<T>::mutate(nft_id, |maybe_nft| {
                if let Some(nft) = maybe_nft {
                    nft.transferable = true;
                }
            });

            Self::deposit_event(Event::BridgeCancelled {
                nft_id,
                owner: who,
            });

            Ok(())
        }

        // ========== PHASE 2.3: MULTI-VALIDATOR VERIFICATION ==========

        /// Request multi-validator verification for quantum result
        ///
        /// Creates a verification request that requires multiple validators
        /// to independently verify the quantum computation result.
        ///
        /// # Arguments
        /// * `origin` - Job submitter or root
        /// * `job_id` - Job to verify
        /// * `required_verifications` - Number of validators required (default: 3)
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::request_verification())]
        pub fn request_verification(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            required_verifications: u8,
        ) -> DispatchResult {
            let requester = ensure_signed(origin)?;

            // Ensure job exists and has result
            let job = QuantumJobs::<T>::get(&job_id).ok_or(Error::<T>::JobNotFound)?;
            ensure!(QuantumResults::<T>::contains_key(&job_id), Error::<T>::ResultNotFound);

            // Verify requester is submitter
            ensure!(job.submitter == requester, Error::<T>::NotAuthorized);

            // Ensure no existing verification request
            ensure!(
                !VerificationRequests::<T>::contains_key(&job_id),
                Error::<T>::VerificationRequestAlreadyExists
            );

            // Validate required_verifications (minimum 2, maximum 10)
            let required = required_verifications.clamp(2, 10);

            let current_block = frame_system::Pallet::<T>::block_number();
            // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
            let deadline = current_block.saturated_into::<u32>() + 100; // 100 blocks deadline

            // Create verification request
            let request = VerificationRequest {
                job_id: job_id.clone(),
                required_verifications: required,
                verifications: BoundedVec::default(),
                approvals: 0,
                rejections: 0,
                consensus_reached: false,
                consensus_result: None,
                // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                created_at: current_block.saturated_into::<u32>(),
                deadline,
            };

            VerificationRequests::<T>::insert(&job_id, request);

            Self::deposit_event(Event::VerificationRequested {
                job_id,
                required_verifications: required,
                deadline,
            });

            Ok(())
        }

        /// Submit verification vote for quantum result
        ///
        /// Validators independently verify quantum results and submit their votes.
        ///
        /// # Arguments
        /// * `origin` - Validator account
        /// * `job_id` - Job being verified
        /// * `vote` - Approve/Reject/Abstain
        /// * `confidence` - Confidence score 0-100
        /// * `result_hash` - Hash of validator's independent result
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::submit_verification())]
        pub fn submit_verification(
            origin: OriginFor<T>,
            job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
            vote_index: u8,  // Index into VerificationVote enum
            confidence: u8,
            result_hash: [u8; 32],
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;

            // Require minimum reputation to participate in verification
            ensure!(
                ValidatorReputation::<T>::get(&validator) >= 50,
                Error::<T>::InsufficientReputation
            );

            // Prevent executor and submitter from verifying their own job
            let job = QuantumJobs::<T>::get(&job_id).ok_or(Error::<T>::JobNotFound)?;
            ensure!(job.executor.as_ref() != Some(&validator), Error::<T>::ExecutorCannotVerify);
            ensure!(job.submitter != validator, Error::<T>::ExecutorCannotVerify);
            
            // Convert index to enum with validation
            let vote = match vote_index {
                0 => VerificationVote::Approve,
                1 => VerificationVote::Reject,
                _ => VerificationVote::Abstain,
            };

            // Ensure confidence is valid (0-100)
            ensure!(confidence <= 100, Error::<T>::InvalidVerificationVote);

            // Ensure verification request exists
            let mut request = VerificationRequests::<T>::get(&job_id)
                .ok_or(Error::<T>::VerificationRequestNotFound)?;

            // Check consensus not already reached
            ensure!(!request.consensus_reached, Error::<T>::ConsensusAlreadyReached);

            // Check deadline
            let current_block = frame_system::Pallet::<T>::block_number();
            // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
            ensure!(
                current_block.saturated_into::<u32>() <= request.deadline,
                Error::<T>::VerificationDeadlinePassed
            );

            // Check validator hasn't already verified
            let validator_bytes: BoundedVec<u8, ConstU32<32>> = 
                validator.encode().try_into().unwrap_or_default();
            
            ensure!(
                !request.verifications.iter().any(|v| v.validator == validator_bytes),
                Error::<T>::ValidatorAlreadyVerified
            );

            // Create verification
            let verification = ValidatorVerification {
                validator: validator_bytes,
                vote: vote.clone(),
                confidence,
                // SAFETY(saturated_into): BlockNumber → u32 is lossless; BelizeChain uses u32 block numbers.
                submitted_at: current_block.saturated_into::<u32>(),
                result_hash,
            };

            // Add verification first (bounded to 10 max) — must succeed before updating counts
            request.verifications.try_push(verification)
                .map_err(|_| Error::<T>::ConsensusAlreadyReached)?;

            // Update vote counts only after successful push
            match vote {
                VerificationVote::Approve => request.approvals = request.approvals.saturating_add(1),
                VerificationVote::Reject => request.rejections = request.rejections.saturating_add(1),
                VerificationVote::Abstain => {}, // Abstain doesn't count
            }

            // Check if consensus reached
            let total_votes = request.approvals + request.rejections;
            if total_votes >= request.required_verifications {
                request.consensus_reached = true;
                request.consensus_result = Some(request.approvals > request.rejections);

                // Update job verification status
                QuantumJobs::<T>::try_mutate(&job_id, |maybe_job| {
                    if let Some(job) = maybe_job.as_mut() {
                        job.verification_status = if request.approvals > request.rejections {
                            VerificationStatus::Verified
                        } else {
                            VerificationStatus::Failed
                        };

                        // Handle payment based on verification result
                        if request.approvals > request.rejections {
                            // AUDIT FIX (C-Q1): Guard against None executor —
                            // refund submitter if no executor assigned.
                            if let Some(executor) = &job.executor {
                                let _ = T::Currency::repatriate_reserved(
                                    &job.submitter,
                                    executor,
                                    job.dalla_cost,
                                    frame_support::traits::BalanceStatus::Free,
                                );
                            } else {
                                // No executor: refund submitter instead of swallowing funds
                                let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
                            }
                        } else {
                            // Rejected: Refund submitter
                            let _ = T::Currency::unreserve(&job.submitter, job.dalla_cost);
                        }
                    }
                    Ok::<(), DispatchError>(())
                })?;

                // Update validator reputations
                Self::update_validator_reputations(&request);

                // Consensus result should always be set when threshold is reached
                // Use unwrap_or(false) as safe fallback
                let consensus_result = request.consensus_result.unwrap_or(false);
                
                Self::deposit_event(Event::VerificationConsensusReached {
                    job_id: job_id.clone(),
                    result: consensus_result,
                    approvals: request.approvals,
                    rejections: request.rejections,
                });
            }

            // Store updated request
            VerificationRequests::<T>::insert(&job_id, request);

            Self::deposit_event(Event::VerificationSubmitted {
                job_id,
                validator,
                vote_index: vote.to_index(),
                confidence,
            });

            Ok(())
        }
    }

    // Helper methods
    impl<T: Config> Pallet<T> {
        /// Get total active jobs for an account
        pub fn get_active_job_count(account: &T::AccountId) -> u32 {
            let jobs = JobsByAccount::<T>::get(account);
            jobs.iter()
                .filter(|job_id| {
                    QuantumJobs::<T>::get(job_id)
                        .map(|job| {
                            job.status == JobStatus::Pending || job.status == JobStatus::Running
                        })
                        .unwrap_or(false)
                })
                .count() as u32
        }

        /// Check if account has earned specific achievement
        pub fn has_achievement(account: &T::AccountId, achievement: AchievementType) -> bool {
            QuantumAchievements::<T>::iter()
                .any(|(_, nft)| nft.owner == *account && nft.achievement_type == achievement)
        }

        /// Update validator reputations based on verification results (Phase 2.3)
        fn update_validator_reputations(request: &VerificationRequest) {
            if let Some(consensus_result) = request.consensus_result {
                // Get original result for comparison
                if let Some(_original_result) = QuantumResults::<T>::get(&request.job_id) {
                    for verification in &request.verifications {
                        // Decode validator account
                        if let Ok(validator_account) = T::AccountId::decode(&mut &verification.validator[..]) {
                            let mut reputation = ValidatorReputation::<T>::get(&validator_account);

                            // Update reputation based on vote accuracy
                            let voted_correctly = match verification.vote {
                                VerificationVote::Approve => consensus_result,
                                VerificationVote::Reject => !consensus_result,
                                VerificationVote::Abstain => continue, // No reputation change
                            };

                            if voted_correctly {
                                // Reward correct vote (max 1000)
                                reputation = reputation.saturating_add(10).min(1000);
                            } else {
                                // Penalize incorrect vote (min 0)
                                reputation = reputation.saturating_sub(20);
                            }

                            ValidatorReputation::<T>::insert(&validator_account, reputation);

                            Self::deposit_event(Event::ReputationUpdated {
                                validator: validator_account,
                                new_reputation: reputation,
                            });
                        }
                    }
                }
            }
        }

        /// Calculate NFT rarity score (Phase 2.3.2)
        /// Returns (rarity_tier, rarity_score 0-1000, category)
        fn calculate_nft_attributes(
            achievement_type: &AchievementType,
            accuracy: u8,
            qubits: u16,
            verification_approved: bool,
        ) -> (NFTRarity, u32, NFTCategory) {
            let mut score: u32 = 0;

            // Base score from achievement type
            let (base_score, category) = match achievement_type {
                AchievementType::FirstQuantumJob => (100, NFTCategory::Special),
                AchievementType::VolumeContributor100 => (150, NFTCategory::Volume),
                AchievementType::VolumeContributor1000 => (300, NFTCategory::Volume),
                AchievementType::Accuracy95 => (200, NFTCategory::Accuracy),
                AchievementType::Accuracy99 => (400, NFTCategory::Accuracy),
                AchievementType::GroverAlgorithm => (350, NFTCategory::Algorithm),
                AchievementType::ShorAlgorithm => (500, NFTCategory::Algorithm),
                AchievementType::QuantumFourierTransform => (400, NFTCategory::Algorithm),
                AchievementType::VQEAlgorithm => (380, NFTCategory::Algorithm),
                AchievementType::QAOAAlgorithm => (420, NFTCategory::Algorithm),
                AchievementType::ErrorMitigationChampion => (450, NFTCategory::Special),
                AchievementType::Custom => (100, NFTCategory::Special),
            };
            score = score.saturating_add(base_score);

            // Accuracy bonus (0-300 points)
            let accuracy_bonus = match accuracy {
                0..=79 => 0,
                80..=89 => 50,
                90..=94 => 100,
                95..=98 => 200,
                99..=100 => 300,
                _ => 300, // Handle values > 100 (shouldn't happen, but compiler needs it)
            };
            score = score.saturating_add(accuracy_bonus);

            // Qubit complexity bonus (0-300 points)
            let qubit_bonus = match qubits {
                0..=9 => 0,
                10..=19 => 75,
                20..=49 => 150,
                50..=99 => 250,
                _ => 300, // 100+ qubits
            };
            score = score.saturating_add(qubit_bonus);

            // Verification consensus bonus (0-100 points)
            if verification_approved {
                score = score.saturating_add(100);
            }

            // Determine rarity tier based on final score
            let rarity = match score {
                0..=249 => NFTRarity::Common,
                250..=499 => NFTRarity::Rare,
                500..=749 => NFTRarity::Epic,
                _ => NFTRarity::Legendary, // 750-1000
            };

            // Cap score at 1000
            let final_score = score.min(1000);

            (rarity, final_score, category)
        }

        /// Generate dynamic metadata URI for NFT (Phase 2.3.2)
        fn generate_metadata_uri(
            nft_id: u64,
            _achievement_type: &AchievementType,
            rarity: &NFTRarity,
            category: &NFTCategory,
            _rarity_score: u32,
            _qubits: u16,
            _accuracy: u8,
        ) -> Result<BoundedVec<u8, ConstU32<MAX_METADATA_URI_LENGTH>>, Error<T>> {
            // Format: ipfs://Qm.../belizechain/{rarity}/{category}/{nft_id}.json
            // In production, this would upload JSON metadata to IPFS
            let rarity_str = match rarity {
                NFTRarity::Common => "common",
                NFTRarity::Rare => "rare",
                NFTRarity::Epic => "epic",
                NFTRarity::Legendary => "legendary",
            };

            let category_str = match category {
                NFTCategory::Volume => "volume",
                NFTCategory::Accuracy => "accuracy",
                NFTCategory::Complexity => "complexity",
                NFTCategory::Speed => "speed",
                NFTCategory::Algorithm => "algorithm",
                NFTCategory::Special => "special",
            };

            let uri = format!(
                "ipfs://QmBelizeChain/{}/{}/{}.json",
                rarity_str, category_str, nft_id
            );

            BoundedVec::try_from(uri.into_bytes())
                .map_err(|_| Error::<T>::InvalidMetadataURI)
        }
    }
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn submit_quantum_job() -> Weight;
    fn update_job_status() -> Weight;
    fn record_quantum_result() -> Weight;
    fn verify_quantum_result() -> Weight;
    fn mint_achievement_nft() -> Weight;
    fn transfer_nft() -> Weight;
    fn list_nft() -> Weight;
    fn buy_nft() -> Weight;
    fn delist_nft() -> Weight;
    fn bridge_to_ethereum() -> Weight;
    fn bridge_to_parachain() -> Weight;
    fn request_verification() -> Weight;
    fn submit_verification() -> Weight;
    fn cancel_bridge() -> Weight;
}

impl WeightInfo for () {
    fn submit_quantum_job() -> Weight {
        Weight::from_parts(25_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(5))
    }
    fn update_job_status() -> Weight {
        Weight::from_parts(15_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn record_quantum_result() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn verify_quantum_result() -> Weight {
        Weight::from_parts(15_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn mint_achievement_nft() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    fn transfer_nft() -> Weight {
        Weight::from_parts(10_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn list_nft() -> Weight {
        Weight::from_parts(12_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))  // Read NFT + check listings + auctions
            .saturating_add(RocksDbWeight::get().writes(2))  // Create listing + counter
    }
    fn buy_nft() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(2))  // Read listing + NFT
            .saturating_add(RocksDbWeight::get().writes(2))  // Transfer NFT + remove listing
    }
    fn delist_nft() -> Weight {
        Weight::from_parts(10_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(1))  // Read listing
            .saturating_add(RocksDbWeight::get().writes(1))  // Remove listing
    }
    fn bridge_to_ethereum() -> Weight {
        Weight::from_parts(18_000_000, 4096)
            .saturating_add(RocksDbWeight::get().reads(4))  // Read NFT + check listings/auctions + bridge requests
            .saturating_add(RocksDbWeight::get().writes(2))  // Create bridge request + counter
    }
    fn bridge_to_parachain() -> Weight {
        Weight::from_parts(20_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(4))  // Read NFT + check listings/auctions + bridge requests
            .saturating_add(RocksDbWeight::get().writes(2))  // Create bridge request + counter + XCM send
    }
    fn request_verification() -> Weight {
        Weight::from_parts(15_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(2))  // Read job + result
            .saturating_add(RocksDbWeight::get().writes(1))  // Create verification request
    }
    fn submit_verification() -> Weight {
        Weight::from_parts(20_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))  // Read request + job + result
            .saturating_add(RocksDbWeight::get().writes(3))  // Update request + job + reputation
    }
    fn cancel_bridge() -> Weight {
        Weight::from_parts(12_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))  // Read bridge request + NFT
            .saturating_add(RocksDbWeight::get().writes(2))  // Remove bridge request + unlock NFT
    }
}

// Benchmarking module - Disabled due to API mismatches
// NOTE: Using estimated weights in WeightInfo implementation above
// For production deployment, run benchmarks with:
//   cargo build --release --features runtime-benchmarks
//   ./target/release/belizechain-node benchmark pallet --pallet pallet_belize_quantum
// Then update weights.rs with benchmark results
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// Testing modules
#[cfg(test)]
pub mod mock;

#[cfg(test)]
pub mod tests;
