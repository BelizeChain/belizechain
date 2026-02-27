//! BelizeChain Runtime - Substrate FRAME v42
//! 
//! Sovereign blockchain infrastructure for Belize combining:
//! - Traditional blockchain governance
//! - Federated AI integration (Proof of Useful Work)
//! - Quantum-resistant cryptography
//! - National digital identity system

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// Make the WASM binary available
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Runtime upgrade utilities
pub mod migrations;

extern crate alloc;
use alloc::borrow::Cow;

use frame_support::{
    construct_runtime,
    derive_impl,
    genesis_builder_helper::{build_state, get_preset},
    parameter_types,
    traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, Get},
    weights::{
        constants::WEIGHT_REF_TIME_PER_SECOND, Weight,
    },
    PalletId,
};
use frame_system::EnsureRoot;
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H256};
use sp_runtime::{
    generic, impl_opaque_keys,
    traits::{
        AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor,
        Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature, Perbill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// Re-exports for downstream
pub use frame_support::{
    traits::{KeyOwnerProofSystem, Randomness, StorageInfo},
    weights::{constants::RocksDbWeight, IdentityFee},
    StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{FungibleAdapter, Multiplier, TargetedFeeAdjustment};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::Permill;

/// Type aliases
pub type BlockNumber = u32;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type Nonce = u32;
pub type Hash = H256;

/// Opaque types for light client support
pub mod opaque {
    use super::*;
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("belizechain"),
    impl_name: Cow::Borrowed("belizechain"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

// Currency and time unit constants for readable configuration
pub const DOLLARS: Balance = 1_000_000_000_000; // 1 DALLA = 10^12 base units (12 decimals)
pub const MINUTES: BlockNumber = 10; // Blocks per minute (6-second block time: 60s ÷ 6s = 10 blocks)
pub const DAYS: BlockNumber = 14_400; // Blocks per day (assuming 6-second block time: 86400s/day ÷ 6s/block = 14400 blocks)

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 1981; // Belize independence year
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    type AccountId = AccountId;
    type RuntimeCall = RuntimeCall;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Self>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<3000>;
    type WeightInfo = ();
}

pub const EXISTENTIAL_DEPOSIT: u128 = 1_000_000_000; // 0.001 DALLA

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<50>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

// FeeMultiplier not needed with () update strategy

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// SECURITY WARNING: `pallet_insecure_randomness_collective_flip` uses block-hash
// based randomness that is manipulable by block producers. It MUST NOT be relied
// upon for security-critical decisions (e.g. financial lotteries, leader election).
// TODO(security): Migrate to BABE epoch-randomness or an off-chain VRF oracle
// before mainnet launch. Tracking issue: RANDOMNESS-MIGRATION.
impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

// ==================== GEM SMART CONTRACT PLATFORM ====================
// Configuration for pallet-contracts (ink! smart contract execution)
// This enables the GEM smart contract ecosystem for BelizeChain

// Constants for smart contract limits
parameter_types! {
    pub const DepositPerItem: Balance = 1_000_000_000; // 1 DALLA per storage item
    pub const DepositPerByte: Balance = 100_000; // 0.0001 DALLA per byte
    pub const DefaultDepositLimit: Balance = 1_000_000_000_000; // 1000 DALLA max deposit
    pub const MaxCodeLen: u32 = 128 * 1024; // 128 KB max contract code size (reduced for CallStack safety)
    pub const MaxStorageKeyLen: u32 = 128; // 128 bytes max storage key length
    pub DeletionQueueDepth: u32 = 128;
    pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
    pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
    pub MaxDelegateDependencies: u32 = 32;
}

impl pallet_contracts::Config for Runtime {
    type Time = Timestamp;
    type Randomness = RandomnessCollectiveFlip;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type CallFilter = frame_support::traits::Everything;
    type DepositPerItem = DepositPerItem;
    type DepositPerByte = DepositPerByte;
    type DefaultDepositLimit = DefaultDepositLimit;
    type CallStack = [pallet_contracts::Frame<Self>; 5];
    type WeightPrice = pallet_transaction_payment::Pallet<Self>;
    type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
    type ChainExtension = ();
    type Schedule = Schedule;
    type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
    type MaxCodeLen = MaxCodeLen;
    type MaxStorageKeyLen = MaxStorageKeyLen;
    type MaxTransientStorageSize = ConstU32<262_144>; // 256 KB transient storage
    type UnsafeUnstableInterface = ConstBool<false>;
    type UploadOrigin = frame_system::EnsureSigned<AccountId>;
    type InstantiateOrigin = frame_system::EnsureSigned<AccountId>;
    type MaxDebugBufferLen = ConstU32<262_144>; // 256 KB debug buffer
    type RuntimeHoldReason = RuntimeHoldReason;
    type Migrations = ();
    type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
    type MaxDelegateDependencies = MaxDelegateDependencies;
    type Debug = ();
    type Environment = ();
    type ApiVersion = ();
    type Xcm = ();
}

// BelizeChain Configuration Parameters
parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const CommunityTreasuryPalletId: PalletId = PalletId(*b"py/comty");
    pub const IdentityPalletId: PalletId = PalletId(*b"py/ident");
    pub const PayrollPalletId: PalletId = PalletId(*b"py/payrl");
    pub const BelizeXPalletId: PalletId = PalletId(*b"py/bzdex");
    pub const MaxDallaSupply: Balance = 501_000_000_000_000_000; // 501B DALLA
    pub const MinValidatorStake: Balance = 100_000_000_000; // 100 DALLA
    pub const BaseReward: Balance = 1_000_000_000; // 1 DALLA per block
    pub const EpochDuration: BlockNumber = 14_400; // ~24 hours
    pub const MinimumDeposit: Balance = 10_000_000_000; // 10 DALLA
    pub const VotingPeriod: BlockNumber = 7_200; // ~12 hours
    pub const LaunchPeriod: BlockNumber = 14_400; // ~24 hours
    pub const MinimumPayment: Balance = 1_000_000_000; // 1 DALLA
    pub const MinBridgeAmount: Balance = 50_000_000_000; // 50 DALLA
    pub const BridgeFeeRate: u32 = 100; // 1%
    pub const PQSignatureThreshold: u32 = 3; // 3 of 5 validators
}

// Treasury account — derived deterministically from TreasuryPalletId
pub struct TreasuryAccount;
impl Get<AccountId> for TreasuryAccount {
    fn get() -> AccountId {
        TreasuryPalletId::get().into_account_truncating()
    }
}

// Community treasury account (10% of treasury for community proposals)
pub struct CommunityTreasuryAccount;
impl Get<AccountId> for CommunityTreasuryAccount {
    fn get() -> AccountId {
        CommunityTreasuryPalletId::get().into_account_truncating()
    }
}

// BelizeChain Custom Pallet Configurations

impl pallet_belize_economy::Config for Runtime {
    type Currency = Balances;
    type Treasury = TreasuryAccount;
    type UnixTime = Timestamp;
    type WeightInfo = pallet_belize_economy::SubstrateWeight<Runtime>;
    type MaxSupply = MaxDallaSupply;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type Oracle = EconomyOracleProvider;
}

impl pallet_belize_identity::Config for Runtime {
    type Currency = Balances;
    type PalletId = IdentityPalletId;
    type Treasury = TreasuryAccount;
    type AdminOrigin = EnsureRoot<AccountId>;
    type RevokeOrigin = EnsureRoot<AccountId>;
    type Oracle = IdentityOracleProvider;
    type MaxAccountsPerIdentity = ConstU32<5>;
    type MaxNameLen = ConstU32<64>;
    type MaxAnchorLen = ConstU32<128>;
    type MaxIssuerCount = ConstU32<10>;
    type KycValidityBlocks = ConstU32<1_051_200>; // ~2 years
    type KycGraceBlocks = ConstU32<14_400>; // ~1 day
    type MaxHistoryLen = ConstU32<100>;
    type WeightInfo = ();
}

impl pallet_belize_governance::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type CouncilOrigin = EnsureRoot<AccountId>;
    type CommunityOrigin = EnsureRoot<AccountId>;
    type ComplianceProvider = GovernanceComplianceProvider;
    type CommunityParticipation = GovernanceCommunityProvider;
    type MinimumDeposit = MinimumDeposit;
    type VotingPeriod = VotingPeriod;
    type LaunchPeriod = LaunchPeriod;
    type WeightInfo = ();
    type MaxCandidatesPerElection = ConstU32<200>;
}

use pallet_belize_compliance::VerificationLevel;

parameter_types! {
    pub const MinValidatorVerificationLevel: VerificationLevel = VerificationLevel::Enhanced;
    pub const MinGovernanceVerificationLevel: VerificationLevel = VerificationLevel::Government;
    pub const MinTreasuryVerificationLevel: VerificationLevel = VerificationLevel::Government;
    pub const VerificationValidityBlocks: BlockNumber = 100_800; // ~1 week
}

impl pallet_belize_compliance::Config for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type ComplianceOrigin = EnsureRoot<AccountId>;
    type SanctionsOrigin = EnsureRoot<AccountId>;
    type MinValidatorVerification = MinValidatorVerificationLevel;
    type MinGovernanceVerification = MinGovernanceVerificationLevel;
    type MinTreasuryVerification = MinTreasuryVerificationLevel;
    type TravelRuleThreshold = ConstU128<100_000_000_000>; // 100 DALLA
    type VerificationValidityPeriod = VerificationValidityBlocks;
    type MaxAuditRecords = ConstU32<1000>;
    type MaxSuspiciousActivityReports = ConstU32<500>;
    type WeightInfo = ();
}

impl pallet_belize_staking::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type Identity = StakingIdentityProvider;
    type OracleVerifier = StakingOracleVerifier;
    type MaxValidators = ConstU32<100>;
    type MinValidatorStake = MinValidatorStake;
    type BaseReward = BaseReward;
    type EpochDuration = EpochDuration;
    type UnbondingPeriod = ConstU32<14_400>; // ~24 hours at 6s blocks
    type WeightInfo = ();
}

impl pallet_belize_oracle::Config for Runtime {
    type OracleAdminOrigin = EnsureRoot<AccountId>;
    type MaxOperators = ConstU32<10>;
    type MaxDataStaleness = ConstU32<100>;
    type MinConsensusOperators = ConstU32<3>;
    type WeightInfo = pallet_belize_oracle::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_payroll::Config for Runtime {
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type PalletId = PayrollPalletId;
    type MaxEmployees = ConstU32<10_000>;
    type MaxDeductions = ConstU32<10>;
    type MaxDepartments = ConstU32<100>;
    type MinimumPayment = MinimumPayment;
    type MaxSchedulesPerBlock = ConstU32<50>;
    type VerifierOrigin = EnsureRoot<AccountId>;
    type Oracle = PayrollOracleProvider;
    type WeightInfo = ();
}

impl pallet_belize_interoperability::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type Time = Timestamp;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type Treasury = TreasuryAccount;
    type Identity = InteroperabilityIdentityProvider;
    type MaxBridgeValidators = ConstU32<21>;
    type MinBridgeAmount = MinBridgeAmount;
    type BridgeFeeRate = BridgeFeeRate;
    type PQSignatureThreshold = PQSignatureThreshold;
    // Challenge period: 100 blocks ≈ 10 minutes at 6s block time (§4.4b)
    type ChallengePeriod = ConstU32<100>;
    type WeightInfo = ();
}

impl pallet_belize_belizex::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type TradingFeeRate = ConstU32<30>; // 0.3%
    type TourismDiscountRate = ConstU32<50>; // 0.5% discount
    type MinLiquidityAmount = ConstU128<1_000_000_000_000>; // 1 DALLA
    type Kyc = BelizeXKycProvider;
    type ProtocolFeeToTreasuryBps = ConstU32<3000>; // 30% to treasury
    type Oracle = BelizeXOracleProvider;
    type MaxOracleDeviationBps = ConstU32<500>; // 5% max deviation from Oracle rate for bBZD trades
    type DexPalletId = BelizeXPalletId;
    type TourismOrigin = EnsureRoot<AccountId>;
    type PairListingOrigin = EnsureRoot<AccountId>;
    type Treasury = TreasuryAccount;
    type WeightInfo = ();
}

impl pallet_belize_landledger::Config for Runtime {
    type Currency = Balances;
    type GovernmentOrigin = EnsureRoot<AccountId>;
    type SurveyorOrigin = EnsureRoot<AccountId>;
    type EnvironmentalOrigin = EnsureRoot<AccountId>;
    type Oracle = LandLedgerOracleProvider;
    type RegistrationDeposit = ConstU128<100_000_000_000>; // 100 DALLA
    type TransferTaxRate = ConstU32<10>; // 0.1%
    type MaxDescriptionLength = ConstU32<256>;
    type WeightInfo = ();
}

impl pallet_belize_consensus::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type UnixTime = Timestamp;
    type AIAuthorityOrigin = EnsureRoot<AccountId>;
    type Staking = ConsensusStakingProvider;
    type MaxValidators = ConstU32<100>;
    type MinConsensusStake = ConstU128<50_000_000_000>; // 50 DALLA
    type MinModelQualityScore = ConstU32<60>; // Minimum 60% quality
    type ConsensusReward = ConstU128<5_000_000_000>; // 5 DALLA per epoch
    type WeightInfo = ();
}

impl pallet_belize_quantum::Config for Runtime {
    type Currency = Balances;
    type MaxActiveJobs = ConstU32<100>;  // Maximum 100 concurrent quantum jobs
    type DallaPerQubit = ConstU128<1_000_000>;  // 0.000001 DALLA per qubit
    type DallaPerShot = ConstU128<100_000>;  // 0.0000001 DALLA per shot
    type NFTMintingFee = ConstU128<500_000_000_000>;  // 0.5 DALLA to mint NFT
    type WeightInfo = ();  // Use default weight implementation
}

impl pallet_belize_community::Config for Runtime {
    type WeightInfo = ();
    type Currency = Balances;
    type BelizeKyc = Identity;  // Use Identity pallet for KYC verification
    type ProposalDepositPercentage = ConstU32<10>;  // 10% deposit for proposals
    type FeeExemptionMonthlyLimit = ConstU32<100>;  // 100 dBZD monthly limit
    type EducationRewardAmount = ConstU128<{ 50 * DOLLARS }>;  // 50 DALLA per module
    type ReferralRewardAmount = ConstU128<{ 100 * DOLLARS }>;  // 100 DALLA referral bonus
    type CommunityVotingPeriod = ConstU32<{ 7 * DAYS }>;  // 7-day voting period
    type CommunityTreasuryAccount = CommunityTreasuryAccount;  // Community treasury
    type GovernanceOrigin = EnsureRoot<AccountId>;  // Governance can sanction
    type MaxTitleLength = ConstU32<128>;  // Max 128 chars for title
    type MaxDescriptionLength = ConstU32<1024>;  // Max 1024 chars for description
    type MaxParticipationHistory = ConstU32<100>;  // Track 100 participation events
}

parameter_types! {
    pub const BnsPalletId: PalletId = PalletId(*b"py/bznss");
}

// BNS treasury account — derived from BnsPalletId for deterministic addressing
pub struct BnsTreasuryAccount;
impl Get<AccountId> for BnsTreasuryAccount {
    fn get() -> AccountId {
        BnsPalletId::get().into_account_truncating()
    }
}

impl pallet_belize_bns::Config for Runtime {
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type Treasury = BnsTreasuryAccount;
    type MaxDomainsPerAccount = ConstU32<100>;  // Max 100 domains per account
    type MaxDomainLength = ConstU32<64>;  // Max 64 chars for domain name
    type MaxTextRecords = ConstU32<20>;  // Max 20 text records
    type MinDomainLength = ConstU32<3>;  // Min 3 chars for domain
    type WeightInfo = pallet_belize_bns::weights::SubstrateWeight<Runtime>;  // Use proper weight implementation
    type Identity = BnsIdentityProvider;  // KYC integration
    type GovernanceOrigin = EnsureRoot<AccountId>;  // Governance for external domain verification
}

impl pallet_belize_mesh::Config for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type Identity = MeshIdentityProviderImpl;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type EmergencyOrigin = EnsureRoot<AccountId>;  // NEMO / government emergency authority
    type MaxMeshNodes = ConstU32<5_000>;  // 5,000 Meshtastic nodes across Belize
    type MaxPendingMeshTx = ConstU32<1_000>;  // 1,000 pending off-grid transactions
    type MaxActiveAlerts = ConstU32<50>;  // 50 concurrent emergency alerts
    type MaxRelayProofsPerClaim = ConstU32<100>;  // 100 relay proofs per reward claim
    type RelayRewardPerTransaction = ConstU128<{ DOLLARS / 10 }>;  // 0.1 DALLA per tx relay
    type RelayRewardPerBlockHeader = ConstU128<{ 5 * DOLLARS / 100 }>;  // 0.05 DALLA per block header
    type RelayRewardPerEmergencyAlert = ConstU128<{ 5 * DOLLARS / 10 }>;  // 0.5 DALLA per emergency relay
    type NodeRegistrationDeposit = ConstU128<{ 10 * DOLLARS }>;  // 10 DALLA deposit to register node
    type HeartbeatTimeout = ConstU32<{ 10 * MINUTES }>;  // Node inactive after 10 minutes no heartbeat
    type WeightInfo = ();
}

// Construct runtime
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,

        // Spike Smart Contract Platform
        Contracts: pallet_contracts,

        // BelizeChain Pallets
        Economy: pallet_belize_economy,
        Identity: pallet_belize_identity,
        Governance: pallet_belize_governance,
        Compliance: pallet_belize_compliance,
        Staking: pallet_belize_staking,
        Oracle: pallet_belize_oracle,
        Payroll: pallet_belize_payroll,
        Interoperability: pallet_belize_interoperability,
        BelizeX: pallet_belize_belizex,
        LandLedger: pallet_belize_landledger,
        Consensus: pallet_belize_consensus,
        Quantum: pallet_belize_quantum,
        Community: pallet_belize_community,
        Bns: pallet_belize_bns,
        Mesh: pallet_belize_mesh,
    }
);

// Cross-Pallet Provider Implementations (AFTER Runtime is constructed)

// Phase 1 & 2 Providers (Economy, Payroll, Identity, LandLedger, Staking)

/// Oracle verifier for Staking pallet (Phase 3)
pub struct StakingOracleVerifier;
impl pallet_belize_staking::OracleVerifier<AccountId> for StakingOracleVerifier {
    fn is_authorized_operator(who: &AccountId) -> bool {
        Oracle::oracle_operators(who)
    }
}

// =============================================================================
// Cross-Pallet Provider Implementations
// =============================================================================
//
// ## Provider Consolidation Roadmap (Audit §3.8)
//
// The runtime currently defines 14 provider structs to wire pallets together via
// trait implementations. This is a recognized architectural concern:
//
// - Many providers still return hardcoded or placeholder values (see P2 §2.7 fixes
//   for partial remediation — e.g. LandLedger ownership now uses real storage).
// - Future phases should consolidate providers into a shared `RuntimeProviders`
//   module that groups related trait impls and makes it easy to audit which
//   cross-pallet queries are "real" vs. stubbed.
// - Consider a macro-based approach (e.g. `impl_providers!`) to reduce boilerplate
//   and enforce that every provider method has an explicit "stub" or "live" label.
// =============================================================================

/// Oracle provider for Economy pallet - Merchant verification ONLY
/// 
/// bBZD peg is always 1:1 with BZD (USDC-style fiat-backed).
/// Oracle is NOT used for exchange rates - only for merchant verification.
pub struct EconomyOracleProvider;
impl pallet_belize_economy::OracleProvider<AccountId> for EconomyOracleProvider {
    fn is_merchant_verified(merchant: &AccountId, category: u8) -> bool {
        if let Some(cat) = Oracle::get_merchant_category(merchant) {
            // MerchantCategory as_u8 helper
            let cat_u8 = match cat {
                pallet_belize_oracle::types::MerchantCategory::Accommodation => 0,
                pallet_belize_oracle::types::MerchantCategory::FoodBeverage => 1,
                pallet_belize_oracle::types::MerchantCategory::TourOperator => 2,
                pallet_belize_oracle::types::MerchantCategory::Transportation => 3,
                pallet_belize_oracle::types::MerchantCategory::Retail => 4,
                pallet_belize_oracle::types::MerchantCategory::Other => 5,
            };
            cat_u8 == category
        } else {
            false
        }
    }
    fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool {
        Oracle::meets_kyc_requirement(account, match required_level {
            0 => pallet_belize_oracle::types::KycLevel::None,
            1 => pallet_belize_oracle::types::KycLevel::Basic,
            2 => pallet_belize_oracle::types::KycLevel::Enhanced,
            _ => pallet_belize_oracle::types::KycLevel::Full,
        })
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        Oracle::is_sanctioned(account)
    }
}

/// Oracle provider for Identity pallet (Phase 2)
pub struct IdentityOracleProvider;
impl pallet_belize_identity::IdentityOracleProvider<AccountId> for IdentityOracleProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Oracle::get_kyc_level(account).map(|level| level as u8)
    }
    fn meets_kyc_requirement(account: &AccountId, level: u8) -> bool {
        if let Some(kyc) = Self::get_kyc_level(account) {
            kyc >= level
        } else {
            false
        }
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        Oracle::is_sanctioned(account)
    }
}

/// Oracle provider for Payroll pallet (Phase 1)
pub struct PayrollOracleProvider;
impl pallet_belize_payroll::PayrollOracleProvider<AccountId> for PayrollOracleProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool {
        Identity::meets_kyc_requirement_level(account, required_level)
    }
}

/// Identity provider for Staking pallet (Phase 2)
pub struct StakingIdentityProvider;
impl pallet_belize_staking::StakingIdentityProvider<AccountId> for StakingIdentityProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    fn meets_validator_kyc(account: &AccountId) -> bool {
        Identity::meets_kyc_requirement_level(account, 2) // Level 2+ for validators
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        Identity::is_account_sanctioned(account)
    }
}

/// Compliance provider for Governance pallet
pub struct GovernanceComplianceProvider;
impl pallet_belize_governance::ComplianceCheck<AccountId> for GovernanceComplianceProvider {
    fn can_participate_in_governance(account: &AccountId) -> bool {
        Identity::get_verified_kyc_level(account).unwrap_or(0) >= 1
    }
}

/// Community participation provider for Governance pallet (Phase 6 integration)
pub struct GovernanceCommunityProvider;
impl pallet_belize_community::GovernanceParticipation<AccountId> for GovernanceCommunityProvider {
    fn record_proposal_submission(account: &AccountId) -> Result<(), &'static str> {
        Community::record_proposal_submission(account)
    }
    
    fn record_vote_cast(account: &AccountId) -> Result<(), &'static str> {
        Community::record_vote_cast(account)
    }
    
    fn record_proposal_approval(account: &AccountId) -> Result<(), &'static str> {
        Community::record_proposal_approval(account)
    }
    
    fn record_council_activity(account: &AccountId) -> Result<(), &'static str> {
        Community::record_council_activity(account)
    }
}

/// Oracle provider for LandLedger pallet (Phase 2)
pub struct LandLedgerOracleProvider;
impl pallet_belize_landledger::LandLedgerOracleProvider<AccountId> for LandLedgerOracleProvider {
    fn verify_land_owner(property_id: u32, account: &AccountId) -> bool {
        // Delegate to on-chain LandLedger storage — check that `account` holds `property_id`
        LandLedger::property_owners(account).contains(&property_id)
    }
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        Identity::is_account_sanctioned(account)
    }
}

/// Community FeeCalculator provider for Economy pallet (Phase 2 - Community Pallet)
pub struct CommunityFeeCalculatorProvider;
impl pallet_belize_community::FeeCalculator<AccountId, Balance> for CommunityFeeCalculatorProvider {
    fn calculate_effective_fee(
        account: &AccountId,
        original_fee: Balance,
    ) -> (Balance, u32, bool) {
        Community::calculate_effective_fee(account, original_fee)
    }
    
    fn apply_fee_discount(
        account: &AccountId,
        original_fee: Balance,
        discounted_fee: Balance,
    ) -> Result<(), &'static str> {
        Community::apply_fee_discount(account, original_fee, discounted_fee)
    }
}

// Phase 3 Providers (BelizeX, Interoperability, Consensus)

/// Oracle provider for BelizeX DEX (Phase 3)
pub struct BelizeXOracleProvider;

impl pallet_belize_belizex::BelizeXOracleProvider<AccountId> for BelizeXOracleProvider {
    fn get_crypto_exchange_rate(_base_asset: u8, quote_asset: u8) -> Option<u128> {
        use pallet_belize_oracle::types::{Currency, CurrencyPair};
        // For BBZD trades, consult USD/BZD rate from Oracle
        let bbzd_id = pallet_belize_belizex::AssetId::BBZD.as_u8();
        if quote_asset == bbzd_id {
            return Oracle::get_exchange_rate(CurrencyPair { base: Currency::USD, quote: Currency::BZD });
        }
        // Otherwise, no crypto rate available via Oracle for this pair
        None
    }
    
    fn is_tourism_merchant(account: &AccountId) -> bool {
        Oracle::get_merchant_category(account).is_some()
    }
    
    fn get_trading_volume_tier(_account: &AccountId) -> u8 {
        // TODO(Phase 4): Implement volume tracking in BelizeX pallet and delegate here.
        // Until then, all accounts default to tier 0 (no volume discount).
        0
    }
}

/// KYC provider for BelizeX (checks Identity pallet)
pub struct BelizeXKycProvider;

impl pallet_belize_belizex::KycCheck<AccountId> for BelizeXKycProvider {
    fn is_kyc_ok(account: &AccountId) -> bool {
        Identity::get_verified_kyc_level(account).unwrap_or(0) >= 1
    }
}

/// Identity provider for Interoperability bridges (Phase 3)
pub struct InteroperabilityIdentityProvider;

impl pallet_belize_interoperability::InteroperabilityIdentityProvider<AccountId> 
    for InteroperabilityIdentityProvider 
{
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        Identity::get_verified_kyc_level(account)
    }
    
    fn verify_bridge_operator(account: &AccountId) -> bool {
        Identity::meets_kyc_requirement_level(account, 3)
    }
    
    fn is_sanctioned(account: &AccountId) -> bool {
        Identity::is_account_sanctioned(account)
    }
}

/// Staking provider for Consensus pallet (Phase 3)
pub struct ConsensusStakingProvider;

impl pallet_belize_consensus::ConsensusStakingProvider<AccountId, Balance> 
    for ConsensusStakingProvider 
{
    fn get_validator_reputation(account: &AccountId) -> u8 {
        if let Some(validator) = pallet_belize_staking::Validators::<Runtime>::get(account) {
            let avg = (validator.quality_score as u16 
                + validator.timeliness_score as u16 
                + validator.honesty_score as u16) / 3;
            avg.min(100) as u8
        } else {
            10
        }
    }
    
    fn get_model_quality_score(account: &AccountId) -> u8 {
        pallet_belize_staking::Validators::<Runtime>::get(account)
            .map(|v| v.quality_score)
            .unwrap_or(0)
    }
    
    fn get_validator_stake(account: &AccountId) -> Balance {
        pallet_belize_staking::Validators::<Runtime>::get(account)
            .map(|v| v.stake)
            .unwrap_or(0)
    }
    
    fn update_reputation(account: &AccountId, quality_score: u8) -> bool {
        if let Some(mut validator) = pallet_belize_staking::Validators::<Runtime>::get(account) {
            let smoothed = (validator.quality_score as u32 * 70 + quality_score as u32 * 30) / 100;
            validator.quality_score = smoothed.min(100) as u8;
            pallet_belize_staking::Validators::<Runtime>::insert(account, validator);
            true
        } else {
            false
        }
    }
}

/// Mesh network Identity provider - integrates KYC and authority checks for Meshtastic nodes
pub struct MeshIdentityProviderImpl;

impl pallet_belize_mesh::MeshIdentityProvider<AccountId> for MeshIdentityProviderImpl {
    fn get_kyc_level(account: &AccountId) -> u8 {
        Identity::get_verified_kyc_level(account).unwrap_or(0)
    }

    fn is_emergency_authority(account: &AccountId) -> bool {
        // Level 3 KYC (Full) + not sanctioned = authorized for emergency alerts
        // In production, this would check a dedicated NEMO authority registry
        let current_block = System::block_number();
        use pallet_belize_identity::{KycLevel, KycState};
        Identity::kyc_state(account, KycLevel::L3, current_block) == KycState::Valid
            && !Oracle::is_sanctioned(account)
    }

    fn is_validator(account: &AccountId) -> bool {
        pallet_belize_staking::Validators::<Runtime>::contains_key(account)
    }
}

/// BNS Identity provider - integrates KYC checks for domain registration
pub struct BnsIdentityProvider;

impl pallet_belize_bns::BnsIdentityProvider<AccountId> for BnsIdentityProvider {
    fn can_register_domain(account: &AccountId) -> bool {
        // Must have at least Level 1 KYC and not be sanctioned  
        let current_block = System::block_number();
        use pallet_belize_identity::{KycLevel, KycState};
        Identity::kyc_state(account, KycLevel::L1, current_block) == KycState::Valid 
            && !Oracle::is_sanctioned(account)
    }

    fn can_register_verified(account: &AccountId) -> bool {
        // Verified domains require Level 3 KYC (full verification)
        let current_block = System::block_number();
        use pallet_belize_identity::{KycLevel, KycState};
        Identity::kyc_state(account, KycLevel::L3, current_block) == KycState::Valid 
            && !Oracle::is_sanctioned(account)
    }

    fn is_sanctioned(account: &AccountId) -> bool {
        Oracle::is_sanctioned(account)
    }
}

// Runtime type definitions
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    migrations::CoordinatedUpgrade<Runtime>,
>;

// Type alias for contract events (required by Contracts API)
pub type EventRecord = frame_system::EventRecord<
    <Runtime as frame_system::Config>::RuntimeEvent,
    <Runtime as frame_system::Config>::Hash,
>;

// Runtime APIs implementation
impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: <Block as BlockT>::LazyBlock) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: <Block as BlockT>::LazyBlock,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec![]
        }
    }

    // The Gem Smart Contract Platform - Runtime API
    impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, EventRecord> for Runtime {
        fn call(
            origin: AccountId,
            dest: AccountId,
            value: Balance,
            gas_limit: Option<Weight>,
            storage_deposit_limit: Option<Balance>,
            input_data: Vec<u8>,
        ) -> pallet_contracts::ContractExecResult<Balance, EventRecord> {
            let gas_limit = gas_limit.unwrap_or(BlockWeights::get().max_block);
            Contracts::bare_call(
                origin,
                dest,
                value,
                gas_limit,
                storage_deposit_limit,
                input_data,
                pallet_contracts::DebugInfo::UnsafeDebug,
                pallet_contracts::CollectEvents::UnsafeCollect,
                pallet_contracts::Determinism::Enforced,
            )
        }

        fn instantiate(
            origin: AccountId,
            value: Balance,
            gas_limit: Option<Weight>,
            storage_deposit_limit: Option<Balance>,
            code: pallet_contracts::Code<Hash>,
            data: Vec<u8>,
            salt: Vec<u8>,
        ) -> pallet_contracts::ContractInstantiateResult<AccountId, Balance, EventRecord> {
            let gas_limit = gas_limit.unwrap_or(BlockWeights::get().max_block);
            Contracts::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                code,
                data,
                salt,
                pallet_contracts::DebugInfo::UnsafeDebug,
                pallet_contracts::CollectEvents::UnsafeCollect,
            )
        }

        fn upload_code(
            origin: AccountId,
            code: Vec<u8>,
            storage_deposit_limit: Option<Balance>,
            determinism: pallet_contracts::Determinism,
        ) -> pallet_contracts::CodeUploadResult<Hash, Balance> {
            Contracts::bare_upload_code(origin, code, storage_deposit_limit, determinism)
        }

        fn get_storage(
            address: AccountId,
            key: Vec<u8>,
        ) -> pallet_contracts::GetStorageResult {
            Contracts::get_storage(address, key)
        }
    }
}
