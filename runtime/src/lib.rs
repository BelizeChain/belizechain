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
use pallet_collective::{EnsureMember, EnsureProportionMoreThan, EnsureProportionAtLeast};
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_babe::AuthorityId as BabeId;
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
            pub babe: Babe,
            pub grandpa: Grandpa,
        }
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("belizechain"),
    impl_name: Cow::Borrowed("belizechain"),
    authoring_version: 1,
    // AR-2: bumped to 101 — pallet_session added (validator rotation enabled).
    spec_version: 103,
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

parameter_types! {
    /// BABE epoch duration in slots (= blocks at 6 s/slot).
    /// Aligned with SessionPeriod so epoch boundary = session boundary.
    pub const BabeEpochDuration: u64 = 14_400; // ~24 hours
    /// Expected block time in milliseconds.
    pub const ExpectedBlockTime: u64 = 6000; // 6 seconds
    /// How long equivocation reports remain valid (10 epochs).
    pub const ReportLongevity: u64 =
        BabeEpochDuration::get() * 10;
}

/// Genesis epoch configuration for BABE.
/// c = (1, 4): each VRF slot has a 1-in-4 chance of being a primary slot.
/// PrimaryAndSecondaryVRFSlots: empty primary slots are filled by secondary
/// VRF winners, guaranteeing block liveness even with skewed PoUW weights.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: (1, 4),
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

impl pallet_babe::Config for Runtime {
    type EpochDuration = BabeEpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

// ── Equivocation / Offences infrastructure ───────────────────────────────────

/// Trivial full-identification: we have no Substrate-native staking, so the
/// identification payload carried in equivocation reports is just unit.
pub struct FullIdentificationOf;
impl sp_runtime::traits::Convert<AccountId, Option<()>> for FullIdentificationOf {
    fn convert(_: AccountId) -> Option<()> {
        Some(())
    }
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = ();
}

impl pallet_session::historical::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type FullIdentification = ();
    type FullIdentificationOf = FullIdentificationOf;
}

/// Bridges `pallet_offences` reports into `pallet_belize_staking::slash_validator`.
///
/// When BABE or GRANDPA equivocation is detected and reported via `pallet_offences`,
/// this handler applies the computed slash fraction against the offending validator's
/// stake using the existing PoUW slashing infrastructure with `ConsensusViolation`.
pub struct BelizeSlashHandler;

impl sp_staking::offence::OnOffenceHandler<
    AccountId,
    pallet_session::historical::IdentificationTuple<Runtime>,
    Weight,
> for BelizeSlashHandler {
    fn on_offence(
        offenders: &[sp_staking::offence::OffenceDetails<
            AccountId,
            pallet_session::historical::IdentificationTuple<Runtime>,
        >],
        slash_fraction: &[Perbill],
        _session: sp_staking::SessionIndex,
    ) -> Weight {
        for (detail, &fraction) in offenders.iter().zip(slash_fraction.iter()) {
            let (ref account, _) = detail.offender;
            // Best-effort: ignore errors (validator may have already left)
            let _ = pallet_belize_staking::Pallet::<Runtime>::slash_validator(
                account,
                fraction,
                pallet_belize_staking::SlashReason::ConsensusViolation,
            );
        }
        Weight::zero()
    }
}

impl pallet_offences::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Runtime>;
    type OnOffenceHandler = BelizeSlashHandler;
}

// ── AR-2: pallet_session — Validator rotation ────────────────────────────────
//
// Before this fix all validator rotation lived in `pallet_belize_staking`
// storage but was never surfaced to BABE/GRANDPA, meaning the block-production
// authority set was frozen at genesis.  Adding `pallet_session` bridges the two
// layers so that:
//  1. When a validator calls `join_validators` / `leave_validators` the new set
//     is queued for the next session boundary.
//  2. At each `SessionPeriod` boundary `BelizeSessionManager::new_session` reads
//     the live `Validators` map and returns the updated authority list.
//  3. BABE rotates its VRF authority set; GRANDPA rotates its voter set.
//  4. Compromised keys can be rotated via `Session::set_keys` without a runtime
//     upgrade (fixing the key-rotation gap identified in AR-2).

parameter_types! {
    /// Session duration: 1 epoch (≈24 hours at 6 s/block = 14 400 blocks).
    pub const SessionPeriod: BlockNumber = 14_400;
    /// Session offset: start immediately from block 0.
    pub const SessionOffset: BlockNumber = 0;
}

/// Bridges `pallet_belize_staking` to `pallet_session`.
///
/// `new_session` is called by `pallet_session` at each epoch boundary.  It reads
/// the live validator set from `pallet_belize_staking::Validators` and returns the
/// accounts as the next authority list.  `pallet_session` will then signal BABE
/// and GRANDPA to rotate to this new set at the start of the following session.
pub struct BelizeSessionManager;

impl pallet_session::SessionManager<AccountId> for BelizeSessionManager {
    fn new_session(_session_index: u32) -> Option<Vec<AccountId>> {
        let validators: Vec<AccountId> = pallet_belize_staking::Validators::<Runtime>::iter_keys().collect();
        if validators.is_empty() {
            None // Keep existing set if staking is empty (genesis bootstrap)
        } else {
            Some(validators)
        }
    }

    fn end_session(_end_index: u32) {}

    fn start_session(_start_index: u32) {
        inject_pouw_weights();
    }
}

/// Override BABE authority weights with PoUW quality scores.
///
/// Called in `start_session` AFTER `pallet_babe::OneSessionHandler`
/// has set `weight = 1` for every authority.  Rewrites `Authorities`
/// and `NextAuthorities` so each validator's VRF winning probability
/// is proportional to their `quality_score` (range 1–100).
fn inject_pouw_weights() {
    let babe_key_type = KeyTypeId(*b"babe");

    // Current epoch authorities
    let mut current = pallet_babe::Authorities::<Runtime>::get().into_inner();
    for (id, w) in current.iter_mut() {
        if let Some(acct) =
            pallet_session::Pallet::<Runtime>::key_owner(babe_key_type, id.as_ref())
        {
            *w = pallet_belize_staking::Validators::<Runtime>::get(&acct)
                .map(|info| (info.quality_score as u64).max(1))
                .unwrap_or(1);
        }
    }
    pallet_babe::Authorities::<Runtime>::put(
        frame_support::WeakBoundedVec::force_from(current, Some("PoUW weights")),
    );

    // Queued next-epoch authorities
    let mut next = pallet_babe::NextAuthorities::<Runtime>::get().into_inner();
    for (id, w) in next.iter_mut() {
        if let Some(acct) =
            pallet_session::Pallet::<Runtime>::key_owner(babe_key_type, id.as_ref())
        {
            *w = pallet_belize_staking::Validators::<Runtime>::get(&acct)
                .map(|info| (info.quality_score as u64).max(1))
                .unwrap_or(1);
        }
    }
    pallet_babe::NextAuthorities::<Runtime>::put(
        frame_support::WeakBoundedVec::force_from(next, Some("PoUW weights")),
    );
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId;
    /// Identity conversion: AccountId is already the ValidatorId.
    type ValidatorIdOf = sp_runtime::traits::ConvertInto;
    /// End sessions on a fixed block-count schedule.
    type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
    /// Read validator set from BelizeChain staking pallet.
    type SessionManager = BelizeSessionManager;
    /// Let session keys (BABE + GRANDPA) drive the authority rotation.
    type SessionHandler = <opaque::SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
    type Keys = opaque::SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
    /// No validator disabling on this permissioned chain.
    type DisablingStrategy = ();
    /// Balances pallet handles key-deposit holds.
    type Currency = Balances;
    /// Zero deposit required for session keys on a permissioned chain.
    type KeyDeposit = ConstU128<0>;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Babe;
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
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
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
    pub const MinValidatorStake: Balance = 100_000_000_000_000; // 100 DALLA (12 decimals)
    pub const BaseReward: Balance = 1_000_000_000; // 1 DALLA per block
    pub const EpochDuration: BlockNumber = 14_400; // ~24 hours
    pub const MinimumDeposit: Balance = 10_000_000_000; // 10 DALLA
    pub const VotingPeriod: BlockNumber = 7_200; // ~12 hours
    pub const LaunchPeriod: BlockNumber = 14_400; // ~24 hours

    // ── Phase 1 governance structural constants ──────────────────────────────
    /// Maximum consecutive terms before a mandatory break (1B — consecutive term cap).
    pub const MaxConsecutiveTerms: u8 = 2;
    /// Maximum number of extensions for permanent-role members (1C).
    pub const MaxPermanentTermExtensions: u8 = 3;
    /// Minimum blocks between successive proposals from the same account (1D).
    /// Default: 43,200 blocks ≈ 3 days at 6 s/block.
    pub const ProposalCooldown: BlockNumber = 43_200;
    /// Enactment delay for Constitutional proposals: 201,600 blocks ≈ 14 days (1E).
    pub const EnactmentPeriodConstitutional: BlockNumber = 201_600;
    /// Enactment delay for Economic proposals: 100,800 blocks ≈ 7 days (1E).
    pub const EnactmentPeriodEconomic: BlockNumber = 100_800;
    /// Enactment delay for Standard / Technical / Community proposals: 50,400 ≈ 3.5 days (1E).
    pub const EnactmentPeriodStandard: BlockNumber = 50_400;

    // ── Phase 2 economic fairness constants ───────────────────────────────────
    /// PalletId for the public-goods treasury (Phase 2C).
    pub const PublicGoodsPalletId: PalletId = PalletId(*b"bz/pubgd");
    /// Percentage of annual inflation routed to the public-goods treasury (Phase 2C).
    /// 10 = 10% of inflation; remaining 90% goes to main treasury.
    pub const PublicGoodsRoutingPercent: u8 = 10;
    /// Enable quadratic voting (Phase 2A). Governance can disable by proposing a runtime upgrade.
    pub const QuadraticVotingEnabled: bool = true;
    /// Hard ceiling on any account's vote weight regardless of stake (Phase 2A).
    pub const MaxVotingUnits: u32 = 10_000;
    /// How many DALLA tokens equal 1 vote unit for QV calculation (Phase 2A).
    /// 100 DALLA = 100 * 10^9 (9 decimal places) = 100_000_000_000.
    pub const StakeUnitSize: Balance = 100_000_000_000;
    /// Balance threshold above which an account is considered a "large holder" (Phase 2B).
    /// 50,000 DALLA = 50_000 * 10^9.
    pub const LargeHolderStakeThreshold: Balance = 50_000_000_000_000;
    /// Minimum governance participation rate (%) required for large holders
    /// before a 50% voting-weight penalty is applied (Phase 2B).
    pub const LargeHolderMinParticipationRate: u8 = 30;

    // ── Phase 3 oracle plurality + SRS attestation constants ─────────────────────
    /// Minimum distinct oracle operators whose KYC submissions must agree
    /// before identity data is committed on-chain (Phase 3A).
    pub const MinOracleAgreement: u32 = 3;
    /// Minimum distinct oracle attestations required before high-value SRS
    /// activities (ProposalApproved / CouncilMembership) are accepted
    /// from a self-reporting user (Phase 3B).
    pub const MinAttestationsRequired: u32 = 2;

    // ── Phase 4C: Exit right ──────────────────────────────────────────────────
    /// Validity period for exit proofs (~30 days at 6 s/block).
    pub const ExitProofValidity: BlockNumber = 432_000;

    // ── Phase 5A: Behavior flag cooldown ─────────────────────────────────────
    /// Cooldown length imposed on a flagged account (~24 h at 6 s/block).
    pub const BehaviorCooldownBlocks: BlockNumber = 14_400;

    // ── Phase 5B: Wellbeing treasury ──────────────────────────────────────────
    /// PalletId for the wellbeing sub-treasury (Phase 5B).
    pub const WellbeingPalletId: PalletId = PalletId(*b"bz/wlbng");
    /// Percentage of the public-goods annual inflation carved to wellbeing treasury.
    /// 10 = 10% of the PG share (= 1% of total annual inflation at default settings).
    pub const WellbeingFundPercent: u8 = 10;

    // ── Phase 5C: Content moderation thresholds ────────────────────────────────
    /// Unique community flags needed to auto-queue content for review.
    pub const ModerationFlagThreshold: u32 = 5;
    /// Nawal risk score above which content is auto-queued (0–100).
    pub const ModerationNawalAutoQueueScore: u8 = 50;

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

// Public-goods treasury — receives Phase 2C inflation routing.
// Governed separately from the main treasury via governance proposals.
pub struct PublicGoodsTreasuryAccount;
impl Get<AccountId> for PublicGoodsTreasuryAccount {
    fn get() -> AccountId {
        PublicGoodsPalletId::get().into_account_truncating()
    }
}

// Wellbeing treasury — Phase 5B carved from public-goods inflation share.
// Funded via annual inflation routing and `WellbeingFunding` governance proposals.
pub struct WellbeingTreasuryAccount;
impl Get<AccountId> for WellbeingTreasuryAccount {
    fn get() -> AccountId {
        WellbeingPalletId::get().into_account_truncating()
    }
}

// ==================== PHASE 0: MULTI-HOUSE GOVERNANCE COLLECTIVES ====================
//
// Three collectively-governed origin groups replace the single EnsureRoot / Sudo key
// that previously controlled all governance operations.
//
//   TechnicalCouncil (Instance1) — max 12 technical members; controls protocol-level
//     and security-sensitive origins (Oracle, Identity, AI Authority, Compliance).
//
//   GovernanceCouncil (Instance2) — max 32 members, maps to on-chain CouncilMembers;
//     controls democratic governance origins (Community, LandLedger, BNS, Mesh, BelizeX).
//
// SetMembersOrigin for both collectives remains EnsureRoot to allow bootstrapping
// the initial member sets via Sudo during testnet. The Sudo scheduled-removal plan
// (Phase 1, Step 1A) will restrict this path before mainnet via on_initialize enforcement.

parameter_types! {
    /// How long a motion stays open for voting before it expires.
    pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;   // 5 days
    pub const GovernanceMotionDuration: BlockNumber = 7 * DAYS;  // 7 days
    /// Max concurrent pending motions per collective.
    pub const TechnicalMaxProposals: u32 = 20;
    pub const GovernanceMaxProposals: u32 = 50;
    /// Member caps per collective.
    pub const TechnicalMaxMembers: u32 = 12;
    pub const GovernanceMaxMembers: u32 = 32;
    /// Max weight of a single collective proposal (75% of block weight).
    pub TechnicalMaxProposalWeight: Weight =
        Perbill::from_percent(75) * BlockWeights::get().max_block;
    pub GovernanceMaxProposalWeight: Weight =
        Perbill::from_percent(75) * BlockWeights::get().max_block;
}

type TechnicalCouncilInstance = pallet_collective::Instance1;
type GovernanceCouncilInstance = pallet_collective::Instance2;

impl pallet_collective::Config<TechnicalCouncilInstance> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    /// EnsureRoot is allowed here ONLY for bootstrapping initial members via Sudo.
    /// Sudo scheduled-removal (Phase 0, Step 0.4) restricts this path before mainnet.
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    /// EnsureRoot can disapprove proposals during bootstrap; replaced by governance post-launch.
    type DisapproveOrigin = EnsureRoot<Self::AccountId>;
    /// EnsureRoot can kill proposals during bootstrap; replaced by governance post-launch.
    type KillOrigin = EnsureRoot<Self::AccountId>;
    /// No deposit/consideration required for collective proposals.
    type Consideration = ();
    type MaxProposalWeight = TechnicalMaxProposalWeight;
}

impl pallet_collective::Config<GovernanceCouncilInstance> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = GovernanceMotionDuration;
    type MaxProposals = GovernanceMaxProposals;
    type MaxMembers = GovernanceMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    /// EnsureRoot allowed ONLY for bootstrapping initial members via Sudo.
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    /// EnsureRoot can disapprove proposals during bootstrap; replaced by governance post-launch.
    type DisapproveOrigin = EnsureRoot<Self::AccountId>;
    /// EnsureRoot can kill proposals during bootstrap; replaced by governance post-launch.
    type KillOrigin = EnsureRoot<Self::AccountId>;
    /// No deposit/consideration required for collective proposals.
    type Consideration = ();
    type MaxProposalWeight = GovernanceMaxProposalWeight;
}

// ── Origin type aliases ───────────────────────────────────────────────────────
// These replace the 20 EnsureRoot<AccountId> usages in pallet configs below.
// EnsureRoot is kept only for pallet_sudo itself and SetMembersOrigin above.

/// Any single member of TechnicalCouncil (low-risk admin operations).
pub type TechnicalCouncilMember =
    EnsureMember<AccountId, TechnicalCouncilInstance>;

/// Simple majority (>1/2) of TechnicalCouncil — standard technical decisions.
pub type TechnicalCouncilMajority =
    EnsureProportionMoreThan<AccountId, TechnicalCouncilInstance, 1, 2>;

/// Supermajority (>2/3) of TechnicalCouncil — sensitive compliance/security.
pub type TechnicalCouncilSuperMajority =
    EnsureProportionMoreThan<AccountId, TechnicalCouncilInstance, 2, 3>;

/// Three quarters (>=3/4) of TechnicalCouncil — emergency and critical actions.
pub type TechnicalCouncilThreeQuarters =
    EnsureProportionAtLeast<AccountId, TechnicalCouncilInstance, 3, 4>;

/// Simple majority (>1/2) of GovernanceCouncil — standard democratic decisions.
pub type GovernanceCouncilMajority =
    EnsureProportionMoreThan<AccountId, GovernanceCouncilInstance, 1, 2>;

/// Supermajority (>2/3) of GovernanceCouncil — sanctions and punitive actions.
pub type GovernanceCouncilSuperMajority =
    EnsureProportionMoreThan<AccountId, GovernanceCouncilInstance, 2, 3>;

// ==================== END PHASE 0 COLLECTIVES ====================

// BelizeChain Custom Pallet Configurations
//
// TODO(PRODUCTION): The following pallets use `WeightInfo = ()` (unit weights),
// meaning all extrinsics are assigned zero weight. Before mainnet launch, run
// `frame-benchmarking` for each pallet and replace `()` with proper weight structs.
// Critical pallets that MUST be benchmarked (economic/security impact):
//   - pallet_belize_staking (validator stake operations)
//   - pallet_belize_governance (proposals, voting, treasury)
//   - pallet_belize_consensus (PoUW rounds, rewards)
//   - pallet_belize_compliance (KYC/AML operations)
//   - pallet_belize_identity (SSN/passport attestations)
//   - pallet_belize_interoperability (bridge transfers)
//   - pallet_belize_belizex (DEX trades, liquidity)
//   - pallet_belize_payroll (salary disbursements)
//   - pallet_belize_landledger (property transfers)
//   - pallet_belize_mesh (emergency alerts, node rewards)
//   - pallet_belize_quantum (quantum jobs, NFT minting)
//   - pallet_belize_community (education/referral rewards)

impl pallet_belize_economy::Config for Runtime {
    type Currency = Balances;
    type Treasury = TreasuryAccount;
    type UnixTime = Timestamp;
    type WeightInfo = pallet_belize_economy::SubstrateWeight<Runtime>;
    type MaxSupply = MaxDallaSupply;
    type GovernanceOrigin = GovernanceCouncilMajority;
    type Oracle = EconomyOracleProvider;
    type MaxMintPerBlock = ConstU32<3>;
    // Phase 2C — progressive public-goods routing
    type PublicGoodsTreasury = PublicGoodsTreasuryAccount;
    type PublicGoodsRoutingPercent = PublicGoodsRoutingPercent;
    // Phase 5B — wellbeing sub-treasury carved from PG share
    type WellbeingTreasury = WellbeingTreasuryAccount;
    type WellbeingFundPercent = WellbeingFundPercent;
}

impl pallet_belize_identity::Config for Runtime {
    type Currency = Balances;
    type PalletId = IdentityPalletId;
    type Treasury = TreasuryAccount;
    type AdminOrigin = TechnicalCouncilSuperMajority;
    type RevokeOrigin = TechnicalCouncilSuperMajority;
    type Oracle = IdentityOracleProvider;
    type MaxAccountsPerIdentity = ConstU32<5>;
    type MaxNameLen = ConstU32<64>;
    type MaxAnchorLen = ConstU32<128>;
    type MaxIssuerCount = ConstU32<10>;
    type KycValidityBlocks = ConstU32<1_051_200>; // ~2 years
    type KycGraceBlocks = ConstU32<14_400>; // ~1 day
    type MaxHistoryLen = ConstU32<100>;
    type WeightInfo = pallet_belize_identity::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_governance::Config for Runtime {
    type Currency = Balances;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type CouncilOrigin = TechnicalCouncilMajority;
    type CommunityOrigin = GovernanceCouncilMajority;
    type ComplianceProvider = GovernanceComplianceProvider;
    type CommunityParticipation = GovernanceCommunityProvider;
    type MinimumDeposit = MinimumDeposit;
    type VotingPeriod = VotingPeriod;
    type LaunchPeriod = LaunchPeriod;
    type WeightInfo = pallet_belize_governance::weights::SubstrateWeight<Runtime>;
    type MaxCandidatesPerElection = ConstU32<200>;
    // Phase 1 — governance structural fixes
    type MaxConsecutiveTerms = MaxConsecutiveTerms;
    type MaxPermanentTermExtensions = MaxPermanentTermExtensions;
    type ProposalCooldown = ProposalCooldown;
    type EnactmentPeriodConstitutional = EnactmentPeriodConstitutional;
    type EnactmentPeriodEconomic = EnactmentPeriodEconomic;
    type EnactmentPeriodStandard = EnactmentPeriodStandard;
    // Phase 2A — quadratic voting + vote-weight cap
    type QuadraticVotingEnabled = QuadraticVotingEnabled;
    type MaxVotingUnits = MaxVotingUnits;
    type StakeUnitSize = StakeUnitSize;
    // Phase 2B — wealth → responsibility
    type LargeHolderStakeThreshold = LargeHolderStakeThreshold;
    type LargeHolderMinParticipationRate = LargeHolderMinParticipationRate;
    // Phase 4C — exit right
    type ExitProofValidity = ExitProofValidity;
    // Phase 5A — behavior flag circuit breaker
    type BehaviorFlags = GovernanceBehaviorProvider;
    // Phase 6A — dual-house constitutional ratification
    type DualHouseProvider = DualHouseMembershipProvider;
    // Phase 6B — constitutional parameter lock admin
    // Bootstrap: TechnicalCouncilSuperMajority (2/3 threshold).
    // TODO: replace with compound dual-council origin before mainnet.
    type ConstitutionalAdminOrigin = TechnicalCouncilSuperMajority;
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
    type ComplianceOrigin = TechnicalCouncilSuperMajority;
    type SanctionsOrigin = GovernanceCouncilSuperMajority;
    type MinValidatorVerification = MinValidatorVerificationLevel;
    type MinGovernanceVerification = MinGovernanceVerificationLevel;
    type MinTreasuryVerification = MinTreasuryVerificationLevel;
    type TravelRuleThreshold = ConstU128<100_000_000_000>; // 100 DALLA
    type VerificationValidityPeriod = VerificationValidityBlocks;
    type MaxAuditRecords = ConstU32<1000>;
    type MaxSuspiciousActivityReports = ConstU32<500>;
    type WeightInfo = pallet_belize_compliance::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_staking::Config for Runtime {
    type Currency = Balances;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type Identity = StakingIdentityProvider;
    type OracleVerifier = StakingOracleVerifier;
    type MaxValidators = ConstU32<100>;
    type MinValidatorStake = MinValidatorStake;
    type BaseReward = BaseReward;
    type EpochDuration = EpochDuration;
    type UnbondingPeriod = ConstU32<14_400>; // ~24 hours at 6s blocks
    type MaxSupply = MaxDallaSupply;
    type WeightInfo = pallet_belize_staking::weights::SubstrateWeight<Runtime>;
    // Phase 4A — first-offense slash intercept via justice pallet
    type JusticeProvider = StakingJusticeProvider;
}

impl pallet_belize_oracle::Config for Runtime {
    type OracleAdminOrigin = TechnicalCouncilMember;
    // AR-7: wire token transfer capability so oracle rewards are actually disbursed
    type Currency = Balances;
    type TreasuryAccount = TreasuryAccount;
    type MaxOperators = ConstU32<10>;
    type MaxDataStaleness = ConstU32<100>;
    type MinConsensusOperators = ConstU32<3>;
    // Phase 3A: minimum distinct oracle agreements for KYC plurality
    type MinOracleAgreement = MinOracleAgreement;
    // Phase 5A: behavior-flag cooldown length
    type BehaviorCooldownBlocks = BehaviorCooldownBlocks;
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
    type VerifierOrigin = TechnicalCouncilMember;
    type Oracle = PayrollOracleProvider;
    type WeightInfo = pallet_belize_payroll::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_interoperability::Config for Runtime {
    type Currency = Balances;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type Time = Timestamp;
    type GovernanceOrigin = GovernanceCouncilMajority;
    type Treasury = TreasuryAccount;
    type Identity = InteroperabilityIdentityProvider;
    type MaxBridgeValidators = ConstU32<21>;
    type MinBridgeAmount = MinBridgeAmount;
    type BridgeFeeRate = BridgeFeeRate;
    type PQSignatureThreshold = PQSignatureThreshold;
    // Challenge period: 100 blocks ≈ 10 minutes at 6s block time (§4.4b)
    type ChallengePeriod = ConstU32<100>;
    type WeightInfo = pallet_belize_interoperability::weights::SubstrateWeight<Runtime>;
    type MaxBridgePerBlock = ConstU32<5>;
    // AR-6: Passthrough verifier — replace with a real Falcon/Dilithium
    // host-function verifier before mainnet deployment.
    type PQVerifier = pallet_belize_interoperability::PassthroughPQVerifier;
}

impl pallet_belize_belizex::Config for Runtime {
    type Currency = Balances;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type TradingFeeRate = ConstU32<30>; // 0.3%
    type TourismDiscountRate = ConstU32<50>; // 0.5% discount
    type MinLiquidityAmount = ConstU128<1_000_000_000_000>; // 1 DALLA
    type Kyc = BelizeXKycProvider;
    type ProtocolFeeToTreasuryBps = ConstU32<3000>; // 30% to treasury
    type Oracle = BelizeXOracleProvider;
    type MaxOracleDeviationBps = ConstU32<500>; // 5% max deviation from Oracle rate for bBZD trades
    type DexPalletId = BelizeXPalletId;
    type TourismOrigin = GovernanceCouncilMajority;
    type PairListingOrigin = TechnicalCouncilMajority;
    type Treasury = TreasuryAccount;
    type WeightInfo = pallet_belize_belizex::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_landledger::Config for Runtime {
    type Currency = Balances;
    type GovernmentOrigin = GovernanceCouncilMajority;
    type SurveyorOrigin = TechnicalCouncilMember;
    type EnvironmentalOrigin = GovernanceCouncilMajority;
    type Oracle = LandLedgerOracleProvider;
    type RegistrationDeposit = ConstU128<100_000_000_000>; // 100 DALLA
    type TransferTaxRate = ConstU32<10>; // 0.1%
    type MaxDescriptionLength = ConstU32<256>;
    type WeightInfo = pallet_belize_landledger::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_consensus::Config for Runtime {
    type Currency = Balances;
    type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
    type UnixTime = Timestamp;
    type AIAuthorityOrigin = TechnicalCouncilMember;
    type Staking = ConsensusStakingProvider;
    type MaxValidators = ConstU32<100>;
    type MinConsensusStake = ConstU128<50_000_000_000>; // 50 DALLA
    type MinModelQualityScore = ConstU32<60>; // Minimum 60% quality
    type ConsensusReward = ConstU128<5_000_000_000>; // 5 DALLA per epoch
    type WeightInfo = pallet_belize_consensus::weights::SubstrateWeight<Runtime>;
    type MaxSubmitPerBlock = ConstU32<3>;
}

impl pallet_belize_quantum::Config for Runtime {
    type Currency = Balances;
    type MaxActiveJobs = ConstU32<100>;  // Maximum 100 concurrent quantum jobs
    type DallaPerQubit = ConstU128<1_000_000>;  // 0.000001 DALLA per qubit
    type DallaPerShot = ConstU128<100_000>;  // 0.0000001 DALLA per shot
    type NFTMintingFee = ConstU128<500_000_000_000>;  // 0.5 DALLA to mint NFT
    type Treasury = TreasuryAccount;  // Q-1 FIX: marketplace fees go to treasury
    type WeightInfo = pallet_belize_quantum::weights::SubstrateWeight<Runtime>;
}

impl pallet_belize_community::Config for Runtime {
    type WeightInfo = pallet_belize_community::weights::SubstrateWeight<Runtime>;
    type Currency = Balances;
    type BelizeKyc = CommunityKycProvider;  // Use wrapper with benchmark bypass
    type ProposalDepositPercentage = ConstU32<10>;  // 10% deposit for proposals
    type FeeExemptionMonthlyLimit = ConstU32<100>;  // 100 dBZD monthly limit
    type EducationRewardAmount = ConstU128<{ 50 * DOLLARS }>;  // 50 DALLA per module
    type ReferralRewardAmount = ConstU128<{ 100 * DOLLARS }>;  // 100 DALLA referral bonus
    type CommunityVotingPeriod = ConstU32<{ 7 * DAYS }>;  // 7-day voting period
    type CommunityTreasuryAccount = CommunityTreasuryAccount;  // Community treasury
    type GovernanceOrigin = GovernanceCouncilMajority;  // Governance can sanction
    type MaxTitleLength = ConstU32<128>;  // Max 128 chars for title
    type MaxDescriptionLength = ConstU32<1024>;  // Max 1024 chars for description
    type MaxParticipationHistory = ConstU32<100>;  // Track 100 participation events
    type MaxSupply = MaxDallaSupply;  // CM-1/CM-2 FIX: same supply cap as economy pallet
    // Phase 3B: SRS oracle attestation for high-value activities
    type MinAttestationsRequired = MinAttestationsRequired;
    type OracleAttestationOrigin = TechnicalCouncilMember;
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
    type GovernanceOrigin = GovernanceCouncilMajority;  // Governance for external domain verification
}

impl pallet_belize_mesh::Config for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type Identity = MeshIdentityProviderImpl;
    type GovernanceOrigin = GovernanceCouncilMajority;
    type EmergencyOrigin = TechnicalCouncilThreeQuarters;  // NEMO / government emergency authority
    type MaxMeshNodes = ConstU32<5_000>;  // 5,000 Meshtastic nodes across Belize
    type MaxPendingMeshTx = ConstU32<1_000>;  // 1,000 pending off-grid transactions
    type MaxActiveAlerts = ConstU32<50>;  // 50 concurrent emergency alerts
    type MaxRelayProofsPerClaim = ConstU32<100>;  // 100 relay proofs per reward claim
    type RelayRewardPerTransaction = ConstU128<{ DOLLARS / 10 }>;  // 0.1 DALLA per tx relay
    type RelayRewardPerBlockHeader = ConstU128<{ 5 * DOLLARS / 100 }>;  // 0.05 DALLA per block header
    type RelayRewardPerEmergencyAlert = ConstU128<{ 5 * DOLLARS / 10 }>;  // 0.5 DALLA per emergency relay
    type NodeRegistrationDeposit = ConstU128<{ 10 * DOLLARS }>;  // 10 DALLA deposit to register node
    type HeartbeatTimeout = ConstU32<{ 10 * MINUTES }>;  // Node inactive after 10 minutes no heartbeat
    type WeightInfo = pallet_belize_mesh::weights::SubstrateWeight<Runtime>;
}

// =============================================================================
// Phase 4A: Justice Pallet Configuration
// Dispute resolution with mediator council and cooling-off rehabilitation.
// =============================================================================
parameter_types! {
    /// Bond required to open a dispute — prevents frivolous filings.
    pub const JusticeOpenDisputeBond: Balance = 100 * DOLLARS;   // 100 DALLA
    /// Cooling-off / rehabilitation period: ~90 days at 6 s/block.
    pub const JusticeCoolingOffPeriod: BlockNumber = 1_296_000;
}

impl pallet_belize_justice::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    /// Governance council majority required for appeals (constitutional-grade).
    type GovernanceOrigin = GovernanceCouncilMajority;
    /// Technical council member can act as mediator.
    type MediatorOrigin = TechnicalCouncilMember;
    type OpenDisputeBond = JusticeOpenDisputeBond;
    type CoolingOffPeriod = JusticeCoolingOffPeriod;
    type MaxMediators = ConstU32<20>;
    type WeightInfo = pallet_belize_justice::weights::SubstrateWeight<Runtime>;
}

// =============================================================================
// Phase 4B: Whistleblower Pallet Configuration
// Pseudonymous reporting with tiered rewards held in escrow.
// =============================================================================
parameter_types! {
    /// Bond required to submit a report — slashed if report is dismissed as bad-faith.
    pub const WhistleblowerReportBond: Balance = 10 * DOLLARS;      // 10 DALLA
    /// Reward paid to a verified fraud reporter (e.g. false KYC attestation).
    pub const WhistleblowerFraudReward: Balance = 2_000 * DOLLARS;  // 2,000 DALLA
    /// Reward paid to a verified systematic-abuse reporter.
    pub const WhistleblowerAbuseReward: Balance = 5_000 * DOLLARS;  // 5,000 DALLA
    /// Reward paid to a verified chain-exploit reporter.
    pub const WhistleblowerExploitReward: Balance = 20_000 * DOLLARS; // 20,000 DALLA
}

impl pallet_belize_whistleblower::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    /// Technical council member reviews incoming reports.
    type ReviewerOrigin = TechnicalCouncilMember;
    /// Governance council majority required to fund the whistleblower pool via governance.
    type GovernanceOrigin = GovernanceCouncilMajority;
    type ReportBond = WhistleblowerReportBond;
    type FraudReward = WhistleblowerFraudReward;
    type AbuseReward = WhistleblowerAbuseReward;
    type ExploitReward = WhistleblowerExploitReward;
    type WeightInfo = pallet_belize_whistleblower::weights::SubstrateWeight<Runtime>;
}

// =============================================================================
// Phase 5C: Content Moderation Pallet Configuration
// Community-driven flagging + Nawal AI auto-queuing + moderator rulings.
// =============================================================================
impl pallet_belize_moderation::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    /// Governance council majority adds/removes moderators.
    type ModeratorAdminOrigin = GovernanceCouncilMajority;
    /// Technical council member (oracle operator) submits Nawal risk scores.
    type NawalOracleOrigin = TechnicalCouncilMember;
    type FlagThreshold = ModerationFlagThreshold;
    type NawalAutoQueueScore = ModerationNawalAutoQueueScore;
    type MaxModerators = ConstU32<50>;
    type WeightInfo = pallet_belize_moderation::weights::SubstrateWeight<Runtime>;
}

// Construct runtime
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Babe: pallet_babe,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        // Phase 0: Multi-house governance collectives
        // TechnicalCouncil  (Instance1) — protocol-level & security-sensitive decisions
        // GovernanceCouncil (Instance2) — democratic governance and community decisions
        TechnicalCouncil: pallet_collective::<Instance1>,
        GovernanceCouncil: pallet_collective::<Instance2>,
        // AR-2: pallet_session enables on-chain validator set rotation.
        // At each SessionPeriod boundary (≈24 h) BelizeSessionManager reads
        // pallet_belize_staking::Validators and rotates BABE/GRANDPA authority sets.
        Session: pallet_session,
        Authorship: pallet_authorship,
        Historical: pallet_session::historical,
        Offences: pallet_offences,

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
        // Phase 4A: Dispute resolution, mediator rulings, rehabilitation tracking
        BelizeJustice: pallet_belize_justice,
        // Phase 4B: Pseudonymous whistleblower reports with escrowed rewards
        BelizeWhistleblower: pallet_belize_whistleblower,
        // Phase 5C: Community content moderation + Nawal AI auto-queuing
        BelizeModeration: pallet_belize_moderation,
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

/// Phase 4A: Bridge justice pallet first-offense intercept to the staking pallet.
/// First-offense validator slashes are escrowed in the justice pallet and subject
/// to a cooling-off period and mediation before execution.  Repeat offenses bypass
/// this path and are slashed immediately by the staking pallet.
///
/// MUST be defined after construct_runtime! because it calls BelizeJustice::.
pub struct StakingJusticeProvider;
impl pallet_belize_staking::JusticeProvider<AccountId, Balance> for StakingJusticeProvider {
    fn try_escrow_slash(account: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
        BelizeJustice::escrow_slash(account, amount)
    }
    fn has_pending_review(account: &AccountId) -> bool {
        BelizeJustice::has_pending_review(account)
    }
}

/// Phase 5A: Bridge from Oracle behavior-flag storage to Governance pallet trait.
/// MUST be defined after construct_runtime! because it calls Oracle::<Runtime>::.
pub struct GovernanceBehaviorProvider;
impl pallet_belize_governance::BehaviorFlagProvider<AccountId, BlockNumber>
    for GovernanceBehaviorProvider
{
    fn has_active_flag(account: &AccountId) -> bool {
        Oracle::has_active_behavior_flag(account)
    }
    fn cooldown_end(account: &AccountId) -> Option<BlockNumber> {
        Oracle::behavior_cooldown_end(account)
    }
}

/// Phase 6A: Bridge pallet_collective membership lists to the governance
/// `DualHouseProvider` trait so constitutional ratification can verify that
/// the caller is a member of TechnicalCouncil or GovernanceCouncil.
///
/// Invariant: Membership reads are O(N) on the current council size.  Council
/// sizes are bounded (≤ MaxMembers in pallet_collective Config), so this is
/// deterministic.  MUST be defined after construct_runtime!.
pub struct DualHouseMembershipProvider;
impl pallet_belize_governance::DualHouseProvider<AccountId> for DualHouseMembershipProvider {
    fn is_technical_house_member(account: &AccountId) -> bool {
        pallet_collective::Members::<Runtime, TechnicalCouncilInstance>::get()
            .contains(account)
    }
    fn is_governance_house_member(account: &AccountId) -> bool {
        pallet_collective::Members::<Runtime, GovernanceCouncilInstance>::get()
            .contains(account)
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

// ── AR-12: Shared provider helpers ───────────────────────────────────────────
//
// All cross-pallet KYC and sanctions queries MUST go through these three helpers.
// Benefits:
//  - Single reading point for the benchmark short-circuit logic.
//  - Audit surface for which pallet is the authoritative source at a glance.
//  - Future-proof: swap the underlying pallet call in one place.
pub mod providers {
    use super::*;

    /// Canonical KYC level query for this runtime.
    ///
    /// Returns `None` if the account has never been verified.
    /// In `runtime-benchmarks` mode always returns `Some(3)` (Enhanced) so that
    /// benchmarks can call any extrinsic without KYC scaffolding.
    #[inline]
    pub fn runtime_kyc_level(account: &AccountId) -> Option<u8> {
        #[cfg(feature = "runtime-benchmarks")]
        return Some(3);
        #[cfg(not(feature = "runtime-benchmarks"))]
        Identity::get_verified_kyc_level(account)
    }

    /// Canonical KYC level query via the Oracle pallet (secondary source).
    ///
    /// Used by providers that bridge to oracle-integrated external data.
    #[inline]
    pub fn oracle_kyc_level(account: &AccountId) -> Option<u8> {
        #[cfg(feature = "runtime-benchmarks")]
        return Some(3);
        #[cfg(not(feature = "runtime-benchmarks"))]
        Oracle::get_kyc_level(account).map(|level| level as u8)
    }

    /// Returns `true` when the account's KYC level meets or exceeds `required`.
    ///
    /// Defaults to `false` when the account has no KYC record.
    #[inline]
    pub fn meets_kyc_requirement(account: &AccountId, required: u8) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        return Identity::meets_kyc_requirement_level(account, required);
    }

    /// Returns `true` when `account` appears on a sanctions list.
    ///
    /// Checks both the Identity pallet (primary) and Oracle pallet (secondary).
    /// In `runtime-benchmarks` mode always returns `false`.
    #[inline]
    pub fn runtime_is_sanctioned(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return false;
        #[cfg(not(feature = "runtime-benchmarks"))]
        return Identity::is_account_sanctioned(account) || Oracle::is_sanctioned(account);
    }
}

/// Community KYC wrapper — bypasses identity checks in benchmark mode
pub struct CommunityKycProvider;
impl pallet_belize_identity::BelizeKyc<AccountId, BlockNumber> for CommunityKycProvider {
    fn is_kyc_verified(
        who: &AccountId,
        level: pallet_belize_identity::KycLevel,
        now: BlockNumber,
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        return <Identity as pallet_belize_identity::BelizeKyc<AccountId, BlockNumber>>::is_kyc_verified(who, level, now);
    }
}

/// Oracle provider for Economy pallet - Merchant verification ONLY
/// 
/// bBZD peg is always 1:1 with BZD (USDC-style fiat-backed).
/// Oracle is NOT used for exchange rates - only for merchant verification.
pub struct EconomyOracleProvider;
impl pallet_belize_economy::OracleProvider<AccountId> for EconomyOracleProvider {
    fn is_merchant_verified(merchant: &AccountId, category: u8) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            if let Some(cat) = Oracle::get_merchant_category(merchant) {
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
    }
    fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        return Oracle::meets_kyc_requirement(account, match required_level {
            0 => pallet_belize_oracle::types::KycLevel::None,
            1 => pallet_belize_oracle::types::KycLevel::Basic,
            2 => pallet_belize_oracle::types::KycLevel::Enhanced,
            _ => pallet_belize_oracle::types::KycLevel::Full,
        });
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        providers::runtime_is_sanctioned(account)
    }
}

/// Oracle provider for Identity pallet (Phase 2)
pub struct IdentityOracleProvider;
impl pallet_belize_identity::IdentityOracleProvider<AccountId> for IdentityOracleProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        providers::oracle_kyc_level(account)
    }
    fn meets_kyc_requirement(account: &AccountId, level: u8) -> bool {
        providers::oracle_kyc_level(account).is_some_and(|kyc| kyc >= level)
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        providers::runtime_is_sanctioned(account)
    }
}

/// Oracle provider for Payroll pallet (Phase 1)
pub struct PayrollOracleProvider;
impl pallet_belize_payroll::PayrollOracleProvider<AccountId> for PayrollOracleProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        providers::runtime_kyc_level(account)
    }
    fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool {
        providers::meets_kyc_requirement(account, required_level)
    }
}

/// Identity provider for Staking pallet (Phase 2)
pub struct StakingIdentityProvider;
impl pallet_belize_staking::StakingIdentityProvider<AccountId> for StakingIdentityProvider {
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        providers::runtime_kyc_level(account)
    }
    fn meets_validator_kyc(account: &AccountId) -> bool {
        providers::meets_kyc_requirement(account, 2) // Level 2+ for validators
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        providers::runtime_is_sanctioned(account)
    }
}

/// Compliance provider for Governance pallet
pub struct GovernanceComplianceProvider;
impl pallet_belize_governance::ComplianceCheck<AccountId> for GovernanceComplianceProvider {
    fn can_participate_in_governance(account: &AccountId) -> bool {
        providers::runtime_kyc_level(account).unwrap_or(0) >= 1
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
        providers::runtime_kyc_level(account)
    }
    fn is_sanctioned(account: &AccountId) -> bool {
        providers::runtime_is_sanctioned(account)
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
    
    fn get_trading_volume_tier(account: &AccountId) -> u8 {
        // X-8 FIX: Basic volume tier based on LP token holdings
        // Full per-trade volume tracking deferred to Phase 4
        let lp_balance = BelizeX::get_lp_balance(account);
        match lp_balance {
            0 => 0,                      // No LP activity
            1..=999_999_999_999 => 1,    // < 1 DALLA equivalent
            1_000_000_000_000..=9_999_999_999_999 => 2, // 1-10 DALLA
            _ => 3,                      // 10+ DALLA
        }
    }
}

/// KYC provider for BelizeX (checks Identity pallet)
pub struct BelizeXKycProvider;

impl pallet_belize_belizex::KycCheck<AccountId> for BelizeXKycProvider {
    fn is_kyc_ok(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        { Identity::get_verified_kyc_level(account).unwrap_or(0) >= 1 }
    }
}

/// Identity provider for Interoperability bridges (Phase 3)
pub struct InteroperabilityIdentityProvider;

impl pallet_belize_interoperability::InteroperabilityIdentityProvider<AccountId> 
    for InteroperabilityIdentityProvider 
{
    fn get_kyc_level(account: &AccountId) -> Option<u8> {
        #[cfg(feature = "runtime-benchmarks")]
        return Some(3);
        #[cfg(not(feature = "runtime-benchmarks"))]
        Identity::get_verified_kyc_level(account)
    }
    
    fn verify_bridge_operator(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        Identity::meets_kyc_requirement_level(account, 3)
    }
    
    fn is_sanctioned(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return false;
        #[cfg(not(feature = "runtime-benchmarks"))]
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
        #[cfg(feature = "runtime-benchmarks")]
        return 3;
        #[cfg(not(feature = "runtime-benchmarks"))]
        Identity::get_verified_kyc_level(account).unwrap_or(0)
    }

    fn is_emergency_authority(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            // Level 3 KYC (Full) + not sanctioned = authorized for emergency alerts
            // In production, this would check a dedicated NEMO authority registry
            let current_block = System::block_number();
            use pallet_belize_identity::{KycLevel, KycState};
            Identity::kyc_state(account, KycLevel::L3, current_block) == KycState::Valid
                && !Oracle::is_sanctioned(account)
        }
    }

    fn is_validator(account: &AccountId) -> bool {
        pallet_belize_staking::Validators::<Runtime>::contains_key(account)
    }
}

/// BNS Identity provider - integrates KYC checks for domain registration
pub struct BnsIdentityProvider;

impl pallet_belize_bns::BnsIdentityProvider<AccountId> for BnsIdentityProvider {
    fn can_register_domain(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let current_block = System::block_number();
            use pallet_belize_identity::{KycLevel, KycState};
            Identity::kyc_state(account, KycLevel::L1, current_block) == KycState::Valid 
                && !Oracle::is_sanctioned(account)
        }
    }

    fn can_register_verified(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return true;
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let current_block = System::block_number();
            use pallet_belize_identity::{KycLevel, KycState};
            Identity::kyc_state(account, KycLevel::L3, current_block) == KycState::Valid 
                && !Oracle::is_sanctioned(account)
        }
    }

    fn is_sanctioned(account: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        return false;
        #[cfg(not(feature = "runtime-benchmarks"))]
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

// Offchain transaction support — required by BABE/GRANDPA equivocation reporting
impl<C> frame_system::offchain::CreateTransactionBase<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type RuntimeCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateBare<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_bare(call: RuntimeCall) -> UncheckedExtrinsic {
        generic::UncheckedExtrinsic::new_bare(call).into()
    }
}

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

// Benchmark configuration
#[cfg(feature = "runtime-benchmarks")]
impl frame_benchmarking::baseline::Config for Runtime {}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        [pallet_sudo, Sudo]
        [pallet_belize_economy, Economy]
        [pallet_belize_identity, Identity]
        [pallet_belize_governance, Governance]
        [pallet_belize_compliance, Compliance]
        [pallet_belize_staking, Staking]
        [pallet_belize_oracle, Oracle]
        [pallet_belize_community, Community]
        [pallet_belize_payroll, Payroll]
        [pallet_belize_interoperability, Interoperability]
        [pallet_belize_belizex, BelizeX]
        [pallet_belize_landledger, LandLedger]
        [pallet_belize_consensus, Consensus]
        [pallet_belize_quantum, Quantum]
        [pallet_belize_bns, Bns]
        [pallet_belize_mesh, Mesh]
    );
}

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

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: BabeEpochDuration::get(),
                c: epoch_config.c,
                authorities: Babe::authorities().to_vec(),
                randomness: Babe::randomness(),
                allowed_slots: epoch_config.allowed_slots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: BabeId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;
            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;
            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
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
            equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;
            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            use codec::Encode;
            Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
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

    // ── AR-11: Compliance runtime API implementation ──────────────────────────
    impl pallet_belize_compliance::runtime_api::ComplianceApi<Block> for Runtime {
        fn get_account_compliance_history(
            account: sp_runtime::AccountId32,
        ) -> sp_std::vec::Vec<pallet_belize_compliance::runtime_api::ComplianceAuditEntry> {
            use codec::Encode as _;
            pallet_belize_compliance::AuditRecords::<Runtime>::get(&account)
                .into_iter()
                .map(|r| pallet_belize_compliance::runtime_api::ComplianceAuditEntry {
                    action_type: r.action_type.encode().first().copied().unwrap_or(0),
                    timestamp: r.timestamp,
                    block_number: r.block_number,
                    success: r.success,
                    details: r.details.into_inner(),
                })
                .collect()
        }

        fn get_suspicious_activity_reports(
            account: sp_runtime::AccountId32,
        ) -> sp_std::vec::Vec<pallet_belize_compliance::runtime_api::SuspiciousActivityEntry> {
            use codec::Encode as _;
            pallet_belize_compliance::SuspiciousActivities::<Runtime>::get(&account)
                .into_iter()
                .map(|(activity_type, block_number, description)| {
                    pallet_belize_compliance::runtime_api::SuspiciousActivityEntry {
                        activity_type: activity_type.encode().first().copied().unwrap_or(0),
                        block_number: block_number,
                        description: description.into_inner(),
                    }
                })
                .collect()
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

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();
            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
            use frame_benchmarking::{baseline, BenchmarkBatch};
            use baseline::Pallet as BaselineBench;

            use sp_storage::TrackedStorageKey;
            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);
            Ok(batches)
        }
    }
}
