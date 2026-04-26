#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Interoperability Pallet - Sovereign Cross-Chain Infrastructure
//!
//! This pallet implements:
//! 1. Post-quantum multi-signature bridges for: ETH, SOL, DOT, BTC, XCM, BNB, ICP
//! 2. DALLA and bBZD bridging only (sovereign asset control)
//! 3. Independent sovereignty - NO Polkadot parachain association
//! 4. Liquidity pool-based bridges with treasury fee collection
//! 5. Governance-controlled bridge parameters
//! 6. On-chain bridge state management
//! 7. Cross-chain message passing with quantum security

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, Get, LockIdentifier, LockableCurrency, Randomness, ReservableCurrency, Time,
    },
    BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, SaturatedConversion, Saturating},
    Perbill,
};
use sp_std::{convert::TryInto, vec::Vec};

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const BRIDGE_LOCK_ID: LockIdentifier = *b"bzbridge";

/// Trait for Identity integration - cross-chain KYC verification
pub trait InteroperabilityIdentityProvider<AccountId> {
    /// Get KYC level for cross-chain operations (0-3)
    fn get_kyc_level(account: &AccountId) -> Option<u8>;

    /// Verify bridge operator has Level 3 (Full) KYC for validator role
    fn verify_bridge_operator(account: &AccountId) -> bool;

    /// Check if account is sanctioned (cross-chain compliance)
    fn is_sanctioned(account: &AccountId) -> bool;
}

/// P0-1: Trait for checking oracle operator authorization.
///
/// Used by `confirm_burn_proof` to verify the caller is a registered
/// oracle operator before accepting their burn attestation.
pub trait OracleOperatorCheck<AccountId> {
    /// Return `true` if `who` is a registered oracle operator.
    fn is_oracle_operator(who: &AccountId) -> bool;
}

/// AR-6: Pluggable post-quantum signature verifier.
///
/// Implement this with a Falcon-1024 or Dilithium5 host-function extension for a
/// production deployment.  The runtime wires a concrete type via
/// `type PQVerifier = …` in the `Config` implementation.
///
/// # Determinism requirement
/// `verify` MUST be deterministic: the same inputs MUST always yield the same
/// result across all nodes and software versions, or the chain will fork.
pub trait PQSignatureVerifier {
    /// Return `true` if `signature` is a valid post-quantum signature over
    /// `message` by the key `pubkey`.
    ///
    /// `pubkey`  — raw public-key bytes (Falcon-1024: 1793 B, Dilithium5: 2592 B)
    /// `message` — the signed message bytes (typically the bridge tx hash)
    /// `signature` — raw signature bytes (up to 96 bytes per spec here, or more
    ///               depending on the scheme; callers must ensure the slice is
    ///               the complete signature)
    fn verify(pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool;
}

/// Default passthrough verifier — accepts any signature of length ≥ 64 bytes.
///
/// **WARNING**: This is NOT a real cryptographic verifier.  It exists solely to
/// allow the pallet to compile and to be replaced by a real implementation.
/// Using this in production leaves bridge signatures UNVERIFIED (AR-6).
///
/// TODO (AR-6): Replace with a host-function-backed Falcon or Dilithium verifier
/// before mainnet deployment.
pub struct PassthroughPQVerifier;

impl PQSignatureVerifier for PassthroughPQVerifier {
    fn verify(_pubkey: &[u8], _message: &[u8], _signature: &[u8]) -> bool {
        // AR-6 SECURITY GUARD: refuse to operate in production builds.
        // This verifier is NOT cryptographically secure.
        // Replace with a real Falcon/Dilithium implementation before mainnet.
        #[cfg(not(any(test, feature = "runtime-benchmarks")))]
        {
            panic!(
                "SECURITY: PassthroughPQVerifier must be replaced with a real \
                 Falcon/Dilithium verifier before mainnet deployment (AR-6)."
            );
        }
        // Structural length check only (test/benchmark builds).
        #[cfg(any(test, feature = "runtime-benchmarks"))]
        {
            _signature.len() >= 64
        }
    }
}

/// Production ML-DSA-87 (NIST FIPS 204 §6) post-quantum signature verifier.
///
/// Uses `fips204::ml_dsa_87` — pure Rust, no_std/WASM-safe, zero C FFI.
/// All error paths (wrong key length, wrong sig length, parse failure,
/// bad signature) return `false` without panicking.
///
/// Cross-protocol replay is prevented by the domain-separation context
/// `b"belizechain-bridge-v1"`, which must be used by all bridge validators
/// when signing bridge transaction digests.
///
/// Sizes (NIST FIPS 204 ML-DSA-87):
///   public key : fips204::ml_dsa_87::PK_LEN  (2592 bytes)
///   signature  : fips204::ml_dsa_87::SIG_LEN  (4627 bytes)
pub struct MLDsaVerifier;

impl PQSignatureVerifier for MLDsaVerifier {
    fn verify(pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool {
        use fips204::ml_dsa_87;
        use fips204::traits::{SerDes, Verifier};

        // Exact-length guards — reject before any deserialization.
        if pubkey.len() != ml_dsa_87::PK_LEN {
            return false;
        }
        if signature.len() != ml_dsa_87::SIG_LEN {
            return false;
        }

        // Convert &[u8] → &[u8; N].  Cannot panic: length verified above.
        let pk_arr: &[u8; ml_dsa_87::PK_LEN] = match pubkey.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let sig_arr: &[u8; ml_dsa_87::SIG_LEN] = match signature.try_into() {
            Ok(a) => a,
            Err(_) => return false,
        };

        // Deserialize public key.
        let pk = match ml_dsa_87::PublicKey::try_from_bytes(*pk_arr) {
            Ok(k) => k,
            Err(_) => return false,
        };

        // Domain-separated verification — prevents cross-protocol replay.
        pk.verify(message, sig_arr, b"belizechain-bridge-v1")
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency used for bridging operations
        type Currency: Currency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + LockableCurrency<Self::AccountId>;

        /// Source of randomness for bridge operations
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// Time provider for bridge timestamps
        type Time: Time;

        /// Governance origin for bridge parameter updates
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Treasury account for bridge fees
        type Treasury: Get<Self::AccountId>;

        /// Maximum number of bridge validators per chain
        #[pallet::constant]
        type MaxBridgeValidators: Get<u32>;

        /// Minimum bridge transaction amount
        #[pallet::constant]
        type MinBridgeAmount: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Bridge transaction fee percentage (in basis points)
        #[pallet::constant]
        type BridgeFeeRate: Get<u32>;

        /// Post-quantum signature threshold
        #[pallet::constant]
        type PQSignatureThreshold: Get<u32>;

        /// Challenge period in blocks before a bridge transaction is finalized.
        /// During this window, validators can dispute a transaction. (§4.4b)
        #[pallet::constant]
        type ChallengePeriod: Get<BlockNumberFor<Self>>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Identity provider for cross-chain KYC verification
        type Identity: InteroperabilityIdentityProvider<Self::AccountId>;

        // ── AR-15: Rate limiting ──────────────────────────────────────────────
        /// Maximum bridge initiations a single account may call per block.
        /// Prevents thundering-herd attacks on the bridge lock mechanism.
        #[pallet::constant]
        type MaxBridgePerBlock: Get<u32>;

        // ── AR-6: Post-quantum signature verifier ─────────────────────────────
        /// Pluggable post-quantum signature verification.
        ///
        /// Implement this with your Falcon/Dilithium host-function extension for
        /// production. The default `PassthroughPQVerifier` accepts any
        /// well-formed byte sequence so the rest of the pallet compiles without a
        /// real PQ crypto library.
        ///
        /// # Safety
        /// A production deployment MUST replace `PassthroughPQVerifier` with a
        /// verifier backed by a host function or verified off-chain worker
        /// attestation; using the passthrough in production renders bridge
        /// signatures unverified. (AR-6)
        type PQVerifier: PQSignatureVerifier;

        /// Pallet ID used to derive the escrow account for bridged assets.
        /// CONS-027: Escrow model replaces balance locks for cross-user unlocks.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        // ── P0-1: Oracle-attested burn verification ───────────────────────────
        /// Oracle operator authorization check.
        /// Wired to the Oracle pallet's `OracleOperators` storage in production.
        type OracleCheck: OracleOperatorCheck<Self::AccountId>;

        /// Minimum number of independent oracle confirmations required before
        /// validators may sign a `BurnAndUnlock` bridge transaction.
        #[pallet::constant]
        type MinOracleConfirmations: Get<u32>;
    }

    /// Supported bridge chains.
    ///
    /// CONS-011: This enum contains 51 variants — each chain expands the cross-chain
    /// attack surface and requires a functioning oracle/relayer. New chains should only
    /// be added via governance proposal after security review. Chains without active
    /// validators or oracle coverage should be disabled via `ChainConfigurations`.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeChain {
        // L1s and Major Networks
        Bitcoin,           // 0
        Ethereum,          // 1
        Solana,            // 2
        BinanceSmartChain, // 3
        Tron,              // 4
        Ripple,            // 5 (XRP Ledger)
        Cardano,           // 6
        Dogecoin,          // 7
        Polygon,           // 8
        Litecoin,          // 9
        Polkadot,          // 10 (as external chain)
        Avalanche,         // 11
        CosmosHub,         // 12
        Ton,               // 13
        InternetComputer,  // 14
        Near,              // 15
        Stellar,           // 16
        Algorand,          // 17
        Tezos,             // 18
        EOS,               // 19
        Hedera,            // 20
        Fantom,            // 21
        Aptos,             // 22
        Sui,               // 23
        Kava,              // 24
        Celo,              // 25
        Harmony,           // 26
        Cronos,            // 27
        Thorchain,         // 28
        Gnosis,            // 29
        ArbitrumOne,       // 30 (L2)
        Optimism,          // 31 (L2)
        Base,              // 32 (L2)
        ZkSyncEra,         // 33 (L2)
        Linea,             // 34 (L2)
        Scroll,            // 35 (L2)
        Mantle,            // 36 (L2)
        PolygonZkEvm,      // 37 (L2)
        Metis,             // 38 (L2)
        Boba,              // 39 (L2)
        Zora,              // 40 (L2)
        Moonbeam,          // 41 (Polkadot EVM)
        Moonriver,         // 42 (Kusama EVM)
        Kusama,            // 43
        OKTC,              // 44
        Waves,             // 45
        Qtum,              // 46
        BitTorrentChain,   // 47
        ICON,              // 48
        VeChain,           // 49
        // Generic XCM marker (for aggregation)
        XCM, // 50
    }

    /// Bridgeable assets (DALLA and bBZD only for sovereignty)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeAsset {
        /// Native DALLA token
        DALLA,
        /// Belizean Dollar stablecoin
        BBZD,
    }

    /// Bridge operation types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeOperation {
        /// Lock assets on BelizeChain, mint on target chain
        LockAndMint {
            target_chain: BridgeChain,
            target_address: BoundedVec<u8, ConstU32<128>>,
            amount: u128,
            asset: BridgeAsset,
        },
        /// Burn assets on target chain, unlock on BelizeChain
        BurnAndUnlock {
            source_chain: BridgeChain,
            source_tx_hash: BoundedVec<u8, ConstU32<64>>,
            amount: u128,
            asset: BridgeAsset,
            recipient: BoundedVec<u8, ConstU32<64>>, // AccountId encoded
        },
        /// Cross-chain message passing
        MessagePassing {
            target_chain: BridgeChain,
            message_hash: [u8; 32],
            payload: BoundedVec<u8, ConstU32<2048>>,
        },
    }

    /// Bridge transaction status
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeStatus {
        /// Transaction initiated
        Initiated,
        /// Awaiting post-quantum signatures
        AwaitingSignatures,
        /// Signatures collected, ready for execution
        ReadyForExecution,
        /// Successfully executed
        Executed,
        /// Failed execution
        Failed,
        /// Disputed transaction
        Disputed,
        /// Cancelled by governance
        Cancelled,
        /// Finalized after challenge period elapsed without dispute (§4.4b)
        Finalized,
    }

    /// Bridge validator information for post-quantum security
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct BridgeValidator<AccountId> {
        /// Validator account
        pub account: AccountId,
        /// Post-quantum public key (ML-DSA-87, FIPS 204)
        pub pq_public_key: BoundedVec<u8, ConstU32<2592>>,
        /// Supported chains
        pub supported_chains: BoundedVec<BridgeChain, ConstU32<64>>,
        /// Stake amount
        pub stake: u128,
        /// Reliability score
        pub reliability_score: u8,
        /// Total signatures provided
        pub signatures_count: u32,
        /// Failed signatures count
        pub failed_signatures: u32,
    }

    /// Bridge liquidity pool
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct LiquidityPool<AccountId> {
        /// Pool ID
        pub pool_id: u32,
        /// Target chain
        pub chain: BridgeChain,
        /// Supported asset
        pub asset: BridgeAsset,
        /// BelizeChain liquidity
        pub belizechain_liquidity: u128,
        /// External chain liquidity
        pub external_liquidity: u128,
        /// Pool creator/manager
        pub manager: AccountId,
        /// Fee rate (basis points)
        pub fee_rate: u32,
        /// Total volume
        pub total_volume: u128,
        /// Pool status
        pub active: bool,
    }

    /// Bridge transaction record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct BridgeTransaction<AccountId, BlockNumber> {
        /// Transaction ID
        pub tx_id: u32,
        /// Initiator account
        pub initiator: AccountId,
        /// Bridge operation details
        pub operation: BridgeOperation,
        /// Current status
        pub status: BridgeStatus,
        /// Required signatures
        pub required_signatures: u32,
        /// Collected signatures
        pub collected_signatures: u32,
        /// Post-quantum signature data (ML-DSA-87)
        pub pq_signatures: BoundedVec<(AccountId, BoundedVec<u8, ConstU32<4627>>), ConstU32<32>>, // CONS-024: raised from 5 to 32 to support full validator set (validator, ML-DSA-87 signature)
        /// Transaction fee
        pub fee: u128,
        /// Initiation block
        pub initiated_at: BlockNumber,
        /// Completion block
        pub completed_at: Option<BlockNumber>,
        /// External chain confirmation
        pub external_confirmation: Option<BoundedVec<u8, ConstU32<128>>>,
        /// Dispute information
        pub dispute_info: Option<BoundedVec<u8, ConstU32<256>>>,
    }

    /// Cross-chain message
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct CrossChainMessage {
        /// Message ID
        pub message_id: u32,
        /// Source chain (always BelizeChain for outgoing)
        pub source_chain: BridgeChain,
        /// Target chain
        pub target_chain: BridgeChain,
        /// Message payload
        pub payload: BoundedVec<u8, ConstU32<2048>>,
        /// Message hash for integrity
        pub message_hash: [u8; 32],
        /// Delivery status
        pub delivered: bool,
        /// Timestamp
        pub timestamp: u64,
    }

    #[pallet::storage]
    #[pallet::getter(fn bridge_validators)]
    /// Registered post-quantum bridge validators
    pub type BridgeValidators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BridgeValidator<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn liquidity_pools)]
    /// Bridge liquidity pools
    pub type LiquidityPools<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Pool ID
        LiquidityPool<T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn bridge_transactions)]
    /// Bridge transaction records
    pub type BridgeTransactions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Transaction ID
        BridgeTransaction<T::AccountId, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn cross_chain_messages)]
    /// Cross-chain message records
    pub type CrossChainMessages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Message ID
        CrossChainMessage,
    >;

    #[pallet::storage]
    #[pallet::getter(fn chain_configurations)]
    /// Configuration for each supported chain
    pub type ChainConfigurations<T: Config> =
        StorageMap<_, Blake2_128Concat, BridgeChain, ChainConfig>;

    #[pallet::storage]
    #[pallet::getter(fn next_tx_id)]
    /// Next available transaction ID
    pub type NextTxId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    /// Next available pool ID
    pub type NextPoolId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_message_id)]
    /// Next available message ID
    pub type NextMessageId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_locked_assets)]
    /// Total locked assets per chain and asset type
    pub type TotalLockedAssets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BridgeChain,
        Blake2_128Concat,
        BridgeAsset,
        u128,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pending_finalizations)]
    /// Bridge transactions awaiting finalization after challenge period (§4.4b).
    /// Key: tx_id → block number when challenge period expires.
    ///
    /// CONS-026: Iteration is bounded to MAX_FINALIZE_PER_IDLE (20) in `on_idle`,
    /// preventing unbounded runtime. Governance can manually finalize or cancel
    /// stale entries via `resolve_dispute` / `cancel_bridge_transaction`.
    pub type PendingFinalizations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Transaction ID
        BlockNumberFor<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_bridge_locks)]
    /// Cumulative bridge-locked amount per user (B-2 fix: prevents set_lock overwrite).
    /// Incremented on initiate_bridge, decremented on process_unlock.
    pub type UserBridgeLocks<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    // AR-15: Per-account bridge initiation rate counter.
    #[pallet::storage]
    /// Rate limit: (block_number, count) of initiate_bridge calls by account.
    /// P0-5: Stores block number per entry — no clear(u32::MAX) needed.
    pub type BridgeCallsThisBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (BlockNumberFor<T>, u32), ValueQuery>;

    #[pallet::storage]
    pub type LastBridgeRateLimitBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    // ── P0-1: Oracle burn-proof confirmation tracking ─────────────────────
    #[pallet::storage]
    /// Per-transaction burn confirmations: (tx_id, oracle_account) → confirmed.
    /// Prevents duplicate attestations from the same oracle operator.
    pub type BurnConfirmations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32, // tx_id
        Blake2_128Concat,
        T::AccountId, // oracle operator
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn burn_confirmation_count)]
    /// Number of distinct oracle confirmations received per BurnAndUnlock transaction.
    pub type BurnConfirmationCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // tx_id
        u32,
        ValueQuery,
    >;

    /// Chain-specific configuration
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct ChainConfig {
        /// Chain enabled for bridging
        pub enabled: bool,
        /// Minimum confirmation blocks
        pub min_confirmations: u32,
        /// Maximum transaction amount
        pub max_amount: u128,
        /// Chain-specific fee rate
        pub fee_rate: u32,
        /// Post-quantum signature requirement
        pub pq_signatures_required: u32,
        /// Chain endpoint URL
        pub rpc_endpoint: BoundedVec<u8, ConstU32<256>>,
        /// Contract address (for smart contract chains)
        pub contract_address: Option<BoundedVec<u8, ConstU32<64>>>,
    }

    // GenesisConfig removed for Substrate v42 compatibility
    // Chain configurations can be initialized through governance or extrinsics

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Bridge transaction initiated
        BridgeTransactionInitiated {
            tx_id: u32,
            initiator: T::AccountId,
            target_chain: u8,
            amount: u128,
            asset: u8,
        },
        /// Bridge transaction executed
        BridgeTransactionExecuted {
            tx_id: u32,
            target_chain: u8,
            external_tx_hash: Vec<u8>,
        },
        /// Liquidity pool created
        LiquidityPoolCreated {
            pool_id: u32,
            chain: u8,
            asset: u8,
            manager: T::AccountId,
        },
        /// Assets locked for bridging
        AssetsLocked {
            account: T::AccountId,
            amount: u128,
            asset: u8,
            target_chain: u8,
        },
        /// Assets unlocked from bridge
        AssetsUnlocked {
            account: T::AccountId,
            amount: u128,
            asset: u8,
            source_chain: u8,
        },
        /// Bridge validator registered
        BridgeValidatorRegistered {
            validator: T::AccountId,
            supported_chains: Vec<u8>,
        },
        /// Post-quantum signature provided
        PQSignatureProvided {
            tx_id: u32,
            validator: T::AccountId,
            signatures_collected: u32,
        },
        /// Cross-chain message sent
        CrossChainMessageSent {
            message_id: u32,
            target_chain: u8,
            message_hash: [u8; 32],
        },
        /// Bridge configuration updated
        BridgeConfigUpdated { chain: u8, fee_rate: u32 },
        /// Bridge fee collected
        BridgeFeeCollected { amount: u128, asset: u8 },
        /// Bridge transaction disputed during challenge period (§4.4b)
        BridgeTransactionDisputed { tx_id: u32, disputer: T::AccountId },
        /// Bridge transaction finalized after challenge period elapsed (§4.4b)
        BridgeTransactionFinalized { tx_id: u32 },
        /// CONS-003: Warning emitted when an incoming unlock is submitted without
        /// an on-chain burn proof. The burn is attested only by validator signatures.
        UnverifiedBurnProofWarning {
            tx_id: u32,
            submitted_by: T::AccountId,
        },
        /// P0-1: Oracle operator confirmed burn proof for a BurnAndUnlock transaction
        BurnProofConfirmed {
            tx_id: u32,
            oracle: T::AccountId,
            confirmations: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Bridge transaction not found
        TransactionNotFound,
        /// Unsupported bridge chain
        UnsupportedChain,
        /// Unsupported asset for bridging
        UnsupportedAsset,
        /// Insufficient balance for bridging
        InsufficientBalance,
        /// Amount below minimum bridge threshold
        BelowMinimumAmount,
        /// Amount exceeds maximum bridge limit
        ExceedsMaximumAmount,
        /// Bridge validator not registered
        ValidatorNotRegistered,
        /// Invalid post-quantum signature
        InvalidPQSignature,
        /// Insufficient signatures collected
        InsufficientSignatures,
        /// Liquidity pool not found
        PoolNotFound,
        /// Insufficient liquidity in pool
        InsufficientLiquidity,
        /// Bridge temporarily disabled
        BridgeDisabled,
        /// Unauthorized bridge operation
        UnauthorizedOperation,
        /// External chain confirmation failed
        ExternalConfirmationFailed,
        /// Bridge transaction already executed
        AlreadyExecuted,
        /// KYC verification required (Level 2+ for bridges, Level 3 for validators)
        KycRequired,
        /// Account is sanctioned and cannot perform cross-chain operations
        AccountSanctioned,
        /// Bridge operator KYC insufficient (Level 3 Full KYC required)
        BridgeOperatorKycInsufficient,
        /// Invalid bridge configuration
        InvalidConfiguration,
        /// Transaction is not in a disputable state (must be ReadyForExecution)
        NotDisputable,
        /// Challenge period has not elapsed yet
        ChallengePeriodActive,
        /// B-9 FIX: ID counter overflow (u32::MAX reached)
        IdOverflow,
        /// AR-15: Account has exceeded the maximum bridge initiations per block.
        RateLimitExceeded,
        /// P0-1: Caller is not a registered oracle operator
        NotOracleOperator,
        /// P0-1: Oracle operator has already confirmed this burn
        AlreadyConfirmedBurn,
        /// P0-1: Transaction is not a BurnAndUnlock operation
        NotBurnTransaction,
        /// P0-1: Insufficient oracle confirmations for burn proof
        BurnNotVerifiedByOracle,
    }

    // ===== HOOKS (§4.4b Challenge Period Finalization) =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // AR-10: Bridge finalization moved from on_initialize to on_idle.
        // Challenge-period finalization is safety-preserving (always runs eventually)
        // but not consensus-mandatory at block start, so it should yield to user transactions.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // P0-6: Ensure escrow account stays alive for KeepAlive transfers.
            // If escrow balance is below existential deposit, endow it to prevent
            // KeepAlive rejections on the last unlock.
            let escrow = Self::escrow_account();
            let ed = T::Currency::minimum_balance();
            let balance = T::Currency::free_balance(&escrow);
            if balance < ed {
                let deficit = ed.saturating_sub(balance);
                let _ = T::Currency::deposit_creating(&escrow, deficit);
            }
            Weight::from_parts(5_000_000, 512)
        }

        /// Finalize expired bridge challenge windows using leftover block weight.
        ///
        /// Processes at most 20 finalizations per block, gated by `remaining_weight`.
        fn on_idle(n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let per_finalization_weight = Weight::from_parts(15_000_000, 1_024)
                .saturating_add(T::DbWeight::get().reads(2))
                .saturating_add(T::DbWeight::get().writes(2));

            if remaining_weight.ref_time() < per_finalization_weight.ref_time() {
                return Weight::zero();
            }

            let mut weight = Weight::from_parts(5_000_000, 512);
            let mut finalized_ids = Vec::new();

            // SECURITY: Bounded iteration — process at most 20 finalizations per block
            for (tx_id, expiry_block) in PendingFinalizations::<T>::iter().take(20) {
                if weight.saturating_add(per_finalization_weight).ref_time()
                    > remaining_weight.ref_time()
                {
                    break;
                }
                if n >= expiry_block {
                    // Challenge period has elapsed — finalize the transaction
                    if let Some(mut bridge_tx) = BridgeTransactions::<T>::get(tx_id) {
                        if bridge_tx.status == BridgeStatus::ReadyForExecution {
                            bridge_tx.status = BridgeStatus::Finalized;
                            bridge_tx.completed_at = Some(n);
                            BridgeTransactions::<T>::insert(tx_id, bridge_tx);
                            Self::deposit_event(Event::BridgeTransactionFinalized { tx_id });
                        }
                    }
                    finalized_ids.push(tx_id);
                    weight = weight.saturating_add(per_finalization_weight);
                }
            }

            for tx_id in finalized_ids {
                PendingFinalizations::<T>::remove(tx_id);
            }

            weight
        }

        /// Verify bridge invariants at runtime startup.
        ///
        /// Invariants:
        /// 1. BridgeFeeRate ≤ 10_000 basis points (≤ 100%)
        fn integrity_test() {
            let fee_rate = T::BridgeFeeRate::get();
            assert!(
                fee_rate <= 10_000,
                "INVARIANT VIOLATION: BridgeFeeRate ({}) exceeds 10_000 basis points (100%)",
                fee_rate,
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Initiate bridge transaction (lock and mint)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::initiate_bridge())]
        pub fn initiate_bridge(
            origin: OriginFor<T>,
            target_chain_index: u8,
            target_address: Vec<u8>,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            asset_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_bridge_rate_limit(&who)?;

            // Cross-chain KYC verification (minimum Level 2 Enhanced KYC for bridges)
            let kyc_level = T::Identity::get_kyc_level(&who).unwrap_or(0);
            ensure!(kyc_level >= 2, Error::<T>::KycRequired);

            // Sanctions screening for cross-chain compliance
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            let target_chain =
                Self::decode_chain(target_chain_index).ok_or(Error::<T>::UnsupportedChain)?;
            let asset = Self::decode_asset(asset_index).ok_or(Error::<T>::UnsupportedAsset)?;

            // Validate chain is supported and enabled
            let chain_config =
                Self::chain_configurations(&target_chain).ok_or(Error::<T>::UnsupportedChain)?;
            ensure!(chain_config.enabled, Error::<T>::BridgeDisabled);

            // Validate amount
            ensure!(
                amount >= T::MinBridgeAmount::get(),
                Error::<T>::BelowMinimumAmount
            );
            // SAFETY: Balance is u128-backed; Balance → u128 is lossless
            ensure!(
                amount.saturated_into::<u128>() <= chain_config.max_amount,
                Error::<T>::ExceedsMaximumAmount
            );

            // Ensure sufficient balance
            ensure!(
                T::Currency::free_balance(&who) >= amount,
                Error::<T>::InsufficientBalance
            );

            // Calculate bridge fee
            // M53 FIX: fee_rate is in basis points (1bp = 0.01% = 1/10,000)
            // Perbill::from_parts treats argument as parts-per-billion, so convert properly
            let fee_amount = Perbill::from_rational(chain_config.fee_rate, 10_000u32) * amount;
            let net_amount = amount.saturating_sub(fee_amount);

            // CONS-027: Transfer net_amount to escrow account instead of locking.
            // This enables cross-user unlocks without balance lock interference.
            let escrow = Self::escrow_account();
            T::Currency::transfer(
                &who,
                &escrow,
                net_amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            // Track cumulative bridged amounts for accounting/events.
            let amount_u128: u128 = net_amount.saturated_into();
            UserBridgeLocks::<T>::mutate(&who, |locked| {
                *locked = locked.saturating_add(amount_u128);
            });

            // Transfer fee to treasury
            let treasury = T::Treasury::get();
            T::Currency::transfer(
                &who,
                &treasury,
                fee_amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let tx_id = Self::next_tx_id();
            let current_block = frame_system::Pallet::<T>::block_number();

            let target_address_bounded: BoundedVec<u8, ConstU32<128>> =
                target_address
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidConfiguration)?;

            let bridge_operation = BridgeOperation::LockAndMint {
                target_chain: target_chain.clone(),
                target_address: target_address_bounded,
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                amount: net_amount.saturated_into::<u128>(),
                asset: asset.clone(),
            };

            let bridge_tx = BridgeTransaction {
                tx_id,
                initiator: who.clone(),
                operation: bridge_operation,
                status: BridgeStatus::Initiated,
                required_signatures: chain_config.pq_signatures_required,
                collected_signatures: 0,
                pq_signatures: BoundedVec::default(),
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                fee: fee_amount.saturated_into::<u128>(),
                initiated_at: current_block,
                completed_at: None,
                external_confirmation: None,
                dispute_info: None,
            };

            BridgeTransactions::<T>::insert(tx_id, bridge_tx);
            NextTxId::<T>::put(tx_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            // Update total locked assets
            TotalLockedAssets::<T>::mutate(&target_chain, &asset, |total| {
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                *total = total.saturating_add(net_amount.saturated_into::<u128>());
            });

            Self::deposit_event(Event::BridgeTransactionInitiated {
                tx_id,
                initiator: who.clone(),
                target_chain: Self::encode_chain(&target_chain),
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                amount: amount.saturated_into::<u128>(),
                asset: Self::encode_asset(&asset),
            });

            Self::deposit_event(Event::AssetsLocked {
                account: who,
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                amount: net_amount.saturated_into::<u128>(),
                asset: Self::encode_asset(&asset),
                target_chain: Self::encode_chain(&target_chain),
            });

            Ok(())
        }

        /// Provide post-quantum signature for bridge transaction
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::provide_signature())]
        pub fn provide_pq_signature(
            origin: OriginFor<T>,
            tx_id: u32,
            pq_signature: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Verify bridge operator has Level 3 Full KYC (national security requirement)
            ensure!(
                T::Identity::verify_bridge_operator(&who),
                Error::<T>::BridgeOperatorKycInsufficient
            );

            // Sanctions check for validator operations
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            // Ensure validator is registered
            let _validator =
                Self::bridge_validators(&who).ok_or(Error::<T>::ValidatorNotRegistered)?;

            let mut bridge_tx =
                Self::bridge_transactions(tx_id).ok_or(Error::<T>::TransactionNotFound)?;

            // Ensure transaction is awaiting signatures
            ensure!(
                matches!(
                    bridge_tx.status,
                    BridgeStatus::Initiated | BridgeStatus::AwaitingSignatures
                ),
                Error::<T>::AlreadyExecuted
            );

            // P0-1: For BurnAndUnlock operations, require minimum oracle confirmations
            // before accepting validator signatures. This closes the CONS-003 gap where
            // colluding validators could fabricate burns without independent verification.
            if matches!(bridge_tx.operation, BridgeOperation::BurnAndUnlock { .. }) {
                let confirmations = BurnConfirmationCount::<T>::get(tx_id);
                ensure!(
                    confirmations >= T::MinOracleConfirmations::get(),
                    Error::<T>::BurnNotVerifiedByOracle
                );
            }

            // Validate post-quantum signature via the configured verifier (AR-6).
            // CONS-001 FIX: canonical message binds tx_id + all value-critical fields,
            // preventing cross-transaction replay (same tx_id, different amount/recipient/chain).
            {
                let validator_record =
                    BridgeValidators::<T>::get(&who).ok_or(Error::<T>::ValidatorNotRegistered)?;
                // Build canonical binding committing to every value-critical field.
                let canonical_message = match &bridge_tx.operation {
                    BridgeOperation::LockAndMint {
                        target_chain,
                        target_address,
                        amount,
                        asset,
                    } => (
                        tx_id,
                        amount,
                        target_chain,
                        target_address.as_slice(),
                        asset,
                    )
                        .encode(),
                    BridgeOperation::BurnAndUnlock {
                        source_chain,
                        source_tx_hash,
                        amount,
                        asset,
                        recipient,
                    } => (
                        tx_id,
                        amount,
                        source_chain,
                        source_tx_hash.as_slice(),
                        asset,
                        recipient.as_slice(),
                    )
                        .encode(),
                    BridgeOperation::MessagePassing {
                        target_chain,
                        message_hash,
                        ..
                    } => (tx_id, target_chain, message_hash).encode(),
                };
                ensure!(
                    T::PQVerifier::verify(
                        validator_record.pq_public_key.as_slice(),
                        &canonical_message,
                        &pq_signature,
                    ),
                    Error::<T>::InvalidPQSignature
                );
            }

            // Add signature
            // M52 FIX: Prevent duplicate signatures from same validator
            ensure!(
                !bridge_tx.pq_signatures.iter().any(|(v, _)| v == &who),
                Error::<T>::AlreadyExecuted
            );
            let sig: BoundedVec<u8, ConstU32<4627>> = pq_signature
                .try_into()
                .map_err(|_| Error::<T>::InvalidPQSignature)?;
            bridge_tx
                .pq_signatures
                .try_push((who.clone(), sig))
                .map_err(|_| Error::<T>::InvalidConfiguration)?;
            bridge_tx.collected_signatures = bridge_tx.collected_signatures.saturating_add(1);

            // Update status based on signature count
            if bridge_tx.collected_signatures >= bridge_tx.required_signatures {
                bridge_tx.status = BridgeStatus::ReadyForExecution;
                // Register challenge period — transaction will auto-finalize after
                // ChallengePeriod blocks if no dispute is filed (§4.4b)
                let current_block = frame_system::Pallet::<T>::block_number();
                let expiry = current_block.saturating_add(T::ChallengePeriod::get());
                PendingFinalizations::<T>::insert(tx_id, expiry);
            } else {
                bridge_tx.status = BridgeStatus::AwaitingSignatures;
            }

            let signatures_collected = bridge_tx.collected_signatures;
            BridgeTransactions::<T>::insert(tx_id, bridge_tx);

            Self::deposit_event(Event::PQSignatureProvided {
                tx_id,
                validator: who,
                signatures_collected,
            });

            Ok(())
        }

        /// Create liquidity pool for bridge
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_liquidity_pool())]
        pub fn create_liquidity_pool(
            origin: OriginFor<T>,
            chain_index: u8,
            asset_index: u8,
            initial_liquidity: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let chain = Self::decode_chain(chain_index).ok_or(Error::<T>::UnsupportedChain)?;
            let asset = Self::decode_asset(asset_index).ok_or(Error::<T>::UnsupportedAsset)?;

            // Ensure sufficient balance
            ensure!(
                T::Currency::free_balance(&who) >= initial_liquidity,
                Error::<T>::InsufficientBalance
            );

            // Lock liquidity
            T::Currency::reserve(&who, initial_liquidity)?;

            let pool_id = Self::next_pool_id();

            let pool = LiquidityPool {
                pool_id,
                chain: chain.clone(),
                asset: asset.clone(),
                // SAFETY: Balance is u128-backed; Balance → u128 is lossless
                belizechain_liquidity: initial_liquidity.saturated_into::<u128>(),
                external_liquidity: 0, // Will be updated externally
                manager: who.clone(),
                fee_rate: T::BridgeFeeRate::get(),
                total_volume: 0,
                active: true,
            };

            LiquidityPools::<T>::insert(pool_id, pool);
            NextPoolId::<T>::put(pool_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            Self::deposit_event(Event::LiquidityPoolCreated {
                pool_id,
                chain: Self::encode_chain(&chain),
                asset: Self::encode_asset(&asset),
                manager: who,
            });

            Ok(())
        }

        /// Process burn and unlock from external chain (B-1 fix: requires multi-sig via BridgeTransaction)
        ///
        /// The BurnAndUnlock BridgeTransaction must have been created via `submit_incoming_unlock`
        /// and finalized through multi-sig + challenge period before this can execute.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::process_unlock())]
        pub fn process_unlock(origin: OriginFor<T>, tx_id: u32) -> DispatchResult {
            // Only bridge validators can execute finalized unlocks
            let who = ensure_signed(origin)?;
            let _validator =
                Self::bridge_validators(&who).ok_or(Error::<T>::ValidatorNotRegistered)?;

            // B-1 fix: Verify BridgeTransaction exists and is Finalized (multi-sig collected + challenge period passed)
            let mut bridge_tx =
                Self::bridge_transactions(tx_id).ok_or(Error::<T>::TransactionNotFound)?;

            ensure!(
                bridge_tx.status == BridgeStatus::Finalized,
                Error::<T>::InsufficientSignatures
            );

            // Extract BurnAndUnlock parameters from the verified transaction
            let (source_chain, _source_tx_hash, amount, asset, recipient_bytes) =
                match &bridge_tx.operation {
                    BridgeOperation::BurnAndUnlock {
                        source_chain,
                        source_tx_hash,
                        amount,
                        asset,
                        recipient,
                    } => (
                        source_chain.clone(),
                        source_tx_hash.clone(),
                        *amount,
                        asset.clone(),
                        recipient.clone(),
                    ),
                    _ => return Err(Error::<T>::UnauthorizedOperation.into()),
                };

            // Verify TotalLockedAssets has enough to cover the unlock
            let locked = TotalLockedAssets::<T>::get(&source_chain, &asset);
            ensure!(locked >= amount, Error::<T>::InsufficientLiquidity);

            // Decode recipient AccountId from bounded bytes
            let recipient = T::AccountId::decode(&mut recipient_bytes.as_slice())
                .map_err(|_| Error::<T>::InvalidConfiguration)?;

            // CONS-027: Transfer from escrow to recipient. This supports cross-user
            // unlocks — the recipient need not be the original locker.
            // P0-6: KeepAlive prevents escrow account reaping (losing other pending bridge funds).
            let _amount_balance: <T::Currency as Currency<T::AccountId>>::Balance =
                amount.saturated_into();
            let escrow = Self::escrow_account();
            T::Currency::transfer(
                &escrow,
                &recipient,
                _amount_balance,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            // Update accounting tracker.
            UserBridgeLocks::<T>::mutate(&recipient, |locked| {
                *locked = locked.saturating_sub(amount);
            });

            // Update total locked assets
            TotalLockedAssets::<T>::mutate(&source_chain, &asset, |total| {
                *total = total.saturating_sub(amount);
            });

            // Mark BridgeTransaction as Executed to prevent replay
            bridge_tx.status = BridgeStatus::Executed;
            bridge_tx.completed_at = Some(frame_system::Pallet::<T>::block_number());
            BridgeTransactions::<T>::insert(tx_id, bridge_tx);

            Self::deposit_event(Event::AssetsUnlocked {
                account: recipient,
                amount,
                asset: Self::encode_asset(&asset),
                source_chain: Self::encode_chain(&source_chain),
            });

            Ok(())
        }

        /// Submit incoming unlock request from external chain (B-1 fix: creates BurnAndUnlock BridgeTransaction)
        ///
        /// Called by a bridge validator who observes a burn event on an external chain.
        /// The transaction must then be signed by multiple validators via `provide_pq_signature`
        /// and survive the challenge period before `process_unlock` can execute it.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::initiate_bridge())]
        pub fn submit_incoming_unlock(
            origin: OriginFor<T>,
            source_chain_index: u8,
            source_tx_hash: Vec<u8>,
            recipient: T::AccountId,
            amount: u128,
            asset_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered validators with Level 3 KYC can submit incoming unlocks
            ensure!(
                T::Identity::verify_bridge_operator(&who),
                Error::<T>::BridgeOperatorKycInsufficient
            );
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );
            let _validator =
                Self::bridge_validators(&who).ok_or(Error::<T>::ValidatorNotRegistered)?;

            let source_chain =
                Self::decode_chain(source_chain_index).ok_or(Error::<T>::UnsupportedChain)?;
            let asset = Self::decode_asset(asset_index).ok_or(Error::<T>::UnsupportedAsset)?;

            // Verify chain is supported and enabled
            let chain_config =
                Self::chain_configurations(&source_chain).ok_or(Error::<T>::UnsupportedChain)?;
            ensure!(chain_config.enabled, Error::<T>::BridgeDisabled);

            // Verify TotalLockedAssets has enough to support this unlock
            let locked = TotalLockedAssets::<T>::get(&source_chain, &asset);
            ensure!(locked >= amount, Error::<T>::InsufficientLiquidity);

            let tx_id = Self::next_tx_id();
            let current_block = frame_system::Pallet::<T>::block_number();

            let source_tx_hash_bounded: BoundedVec<u8, ConstU32<64>> = source_tx_hash
                .try_into()
                .map_err(|_| Error::<T>::InvalidConfiguration)?;

            let recipient_encoded = recipient.encode();
            let recipient_bounded: BoundedVec<u8, ConstU32<64>> = recipient_encoded
                .try_into()
                .map_err(|_| Error::<T>::InvalidConfiguration)?;

            let bridge_operation = BridgeOperation::BurnAndUnlock {
                source_chain: source_chain.clone(),
                source_tx_hash: source_tx_hash_bounded,
                amount,
                asset: asset.clone(),
                recipient: recipient_bounded,
            };

            let bridge_tx = BridgeTransaction {
                tx_id,
                initiator: who.clone(),
                operation: bridge_operation,
                status: BridgeStatus::Initiated,
                required_signatures: chain_config.pq_signatures_required,
                collected_signatures: 0,
                pq_signatures: BoundedVec::default(),
                fee: 0, // No fee for incoming unlocks
                initiated_at: current_block,
                completed_at: None,
                external_confirmation: None,
                dispute_info: None,
            };

            BridgeTransactions::<T>::insert(tx_id, bridge_tx);
            NextTxId::<T>::put(tx_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            Self::deposit_event(Event::BridgeTransactionInitiated {
                tx_id,
                initiator: who.clone(),
                target_chain: Self::encode_chain(&source_chain),
                amount,
                asset: Self::encode_asset(&asset),
            });

            // CONS-003: Emit warning — no on-chain burn proof; relying on validator attestation only.
            Self::deposit_event(Event::UnverifiedBurnProofWarning {
                tx_id,
                submitted_by: who,
            });

            Ok(())
        }

        /// Send cross-chain message (H-36 fix: KYC, sanctions, fee)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::send_message())]
        pub fn send_cross_chain_message(
            origin: OriginFor<T>,
            target_chain_index: u8,
            payload: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // H-36: Require Level 2+ KYC for cross-chain messaging
            let kyc_level = T::Identity::get_kyc_level(&who).unwrap_or(0);
            ensure!(kyc_level >= 2, Error::<T>::KycRequired);

            // H-36: Sanctions screening
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            let target_chain =
                Self::decode_chain(target_chain_index).ok_or(Error::<T>::UnsupportedChain)?;

            // H-36: Charge a messaging fee (MinBridgeAmount sent to treasury)
            let fee = T::MinBridgeAmount::get();
            let treasury = T::Treasury::get();
            T::Currency::transfer(
                &who,
                &treasury,
                fee,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            let message_id = Self::next_message_id();
            let payload_bounded: BoundedVec<u8, ConstU32<2048>> = payload
                .try_into()
                .map_err(|_| Error::<T>::InvalidConfiguration)?;
            let message_hash = sp_core::blake2_256(payload_bounded.as_slice());
            // SAFETY: BlockNumber fits in u64 (runtime uses u32 block numbers)
            let timestamp = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();

            let message = CrossChainMessage {
                message_id,
                source_chain: BridgeChain::Polkadot, // BelizeChain as source
                target_chain: target_chain.clone(),
                payload: payload_bounded,
                message_hash,
                delivered: false,
                timestamp,
            };

            CrossChainMessages::<T>::insert(message_id, message);
            NextMessageId::<T>::put(message_id.checked_add(1).ok_or(Error::<T>::IdOverflow)?);

            Self::deposit_event(Event::CrossChainMessageSent {
                message_id,
                target_chain: Self::encode_chain(&target_chain),
                message_hash,
            });

            Ok(())
        }

        /// Update bridge configuration (governance only)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::update_config())]
        pub fn update_bridge_config(
            origin: OriginFor<T>,
            chain_index: u8,
            enabled: bool,
            fee_rate: u32,
            max_amount: u128,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let chain = Self::decode_chain(chain_index).ok_or(Error::<T>::UnsupportedChain)?;

            // CRIT-1 FIX: Cap fee_rate to 10_000 (100%)
            ensure!(fee_rate <= 10_000, Error::<T>::InvalidConfiguration);

            // CRIT-2 FIX: Ensure config exists before updating (no silent no-op)
            ensure!(
                ChainConfigurations::<T>::contains_key(&chain),
                Error::<T>::UnsupportedChain
            );

            ChainConfigurations::<T>::mutate(&chain, |maybe_config| {
                if let Some(config) = maybe_config {
                    config.enabled = enabled;
                    config.fee_rate = fee_rate;
                    config.max_amount = max_amount;
                }
            });

            Self::deposit_event(Event::BridgeConfigUpdated {
                chain: Self::encode_chain(&chain),
                fee_rate,
            });

            Ok(())
        }

        /// Dispute a bridge transaction during challenge period (§4.4b)
        ///
        /// Only registered bridge validators can file disputes. If a dispute is
        /// accepted, the transaction moves to `Disputed` status and is blocked
        /// from automatic finalization. Governance must then resolve the dispute.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::update_config())] // Re-use weight; lightweight operation
        pub fn dispute_bridge_transaction(
            origin: OriginFor<T>,
            tx_id: u32,
            reason: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered bridge validators can dispute
            ensure!(
                Self::bridge_validators(&who).is_some(),
                Error::<T>::ValidatorNotRegistered
            );

            // Verify bridge operator KYC
            ensure!(
                T::Identity::verify_bridge_operator(&who),
                Error::<T>::BridgeOperatorKycInsufficient
            );

            let mut bridge_tx =
                Self::bridge_transactions(tx_id).ok_or(Error::<T>::TransactionNotFound)?;

            // Can only dispute transactions in ReadyForExecution (challenge window)
            ensure!(
                bridge_tx.status == BridgeStatus::ReadyForExecution,
                Error::<T>::NotDisputable
            );

            // Verify we are still within the challenge period
            ensure!(
                PendingFinalizations::<T>::contains_key(tx_id),
                Error::<T>::NotDisputable
            );

            // CRIT-3 FIX: Dispute bond based on tx amount with minimum floor.
            // Original used fee/20 which was 0 for incoming unlocks (zero-cost griefing).
            let min_bond = T::MinBridgeAmount::get();
            let tx_amount: <T::Currency as Currency<T::AccountId>>::Balance =
                match &bridge_tx.operation {
                    BridgeOperation::LockAndMint { amount, .. }
                    | BridgeOperation::BurnAndUnlock { amount, .. } => (*amount).saturated_into(),
                    BridgeOperation::MessagePassing { .. } => min_bond,
                };
            let amount_based_bond = tx_amount / 20u128.saturated_into(); // 5% of tx value
            let dispute_bond = core::cmp::max(amount_based_bond, min_bond);
            T::Currency::reserve(&who, dispute_bond)?;

            // Mark transaction as disputed
            bridge_tx.status = BridgeStatus::Disputed;
            let reason_bounded: BoundedVec<u8, ConstU32<256>> = reason
                .try_into()
                .map_err(|_| Error::<T>::InvalidConfiguration)?;
            bridge_tx.dispute_info = Some(reason_bounded);

            BridgeTransactions::<T>::insert(tx_id, bridge_tx);

            // Remove from pending finalization — governance must resolve
            PendingFinalizations::<T>::remove(tx_id);

            Self::deposit_event(Event::BridgeTransactionDisputed {
                tx_id,
                disputer: who,
            });

            Ok(())
        }

        /// Register as a bridge validator (H-37 fix: writeable path for BridgeValidators)
        ///
        /// Requires Level 3 KYC (full identity verification) and a minimum stake.
        /// The caller provides a post-quantum public key and the chains they support.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::initiate_bridge())]
        pub fn register_bridge_validator(
            origin: OriginFor<T>,
            pq_public_key: Vec<u8>,
            supported_chain_indices: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require Level 3 KYC for validator registration
            ensure!(
                T::Identity::verify_bridge_operator(&who),
                Error::<T>::BridgeOperatorKycInsufficient
            );
            // Sanctions screening
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            // Must not already be registered
            ensure!(
                !BridgeValidators::<T>::contains_key(&who),
                Error::<T>::AlreadyExecuted // reuse: already registered
            );

            // CONS-013: Validator stake is 10× MinBridgeAmount to ensure meaningful skin-in-the-game.
            let stake: <T::Currency as Currency<T::AccountId>>::Balance =
                T::MinBridgeAmount::get().saturating_mul(10u128.saturated_into());
            T::Currency::set_lock(
                BRIDGE_LOCK_ID,
                &who,
                stake,
                frame_support::traits::WithdrawReasons::all(),
            );

            // Decode supported chains
            let mut chains = Vec::new();
            for idx in &supported_chain_indices {
                let chain = Self::decode_chain(*idx).ok_or(Error::<T>::UnsupportedChain)?;
                chains.push(chain);
            }
            let supported_chains: BoundedVec<BridgeChain, ConstU32<64>> = chains
                .try_into()
                .map_err(|_| Error::<T>::InvalidConfiguration)?;

            let pq_key: BoundedVec<u8, ConstU32<2592>> = pq_public_key
                .try_into()
                .map_err(|_| Error::<T>::InvalidPQSignature)?;

            let validator = BridgeValidator {
                account: who.clone(),
                pq_public_key: pq_key,
                supported_chains: supported_chains.clone(),
                stake: stake.saturated_into::<u128>(),
                reliability_score: 100,
                signatures_count: 0,
                failed_signatures: 0,
            };

            BridgeValidators::<T>::insert(&who, validator);

            let chain_bytes: Vec<u8> = supported_chains
                .iter()
                .map(|c| Self::encode_chain(c))
                .collect();
            Self::deposit_event(Event::BridgeValidatorRegistered {
                validator: who,
                supported_chains: chain_bytes,
            });

            Ok(())
        }

        /// Remove a bridge validator (governance only, H-37 fix)
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::update_config())]
        pub fn remove_bridge_validator(
            origin: OriginFor<T>,
            validator_account: T::AccountId,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            ensure!(
                BridgeValidators::<T>::contains_key(&validator_account),
                Error::<T>::ValidatorNotRegistered
            );

            // Remove lock
            T::Currency::remove_lock(BRIDGE_LOCK_ID, &validator_account);

            BridgeValidators::<T>::remove(&validator_account);

            Ok(())
        }

        /// Withdraw liquidity from a bridge pool (H-35 fix)
        ///
        /// Only the pool manager can withdraw. Unreserves funds and updates pool.
        /// If the entire liquidity is withdrawn the pool is deactivated.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::create_liquidity_pool())]
        pub fn withdraw_liquidity(
            origin: OriginFor<T>,
            pool_id: u32,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut pool = LiquidityPools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

            // Only the pool manager may withdraw
            ensure!(pool.manager == who, Error::<T>::UnauthorizedOperation);

            let amount_u128: u128 = amount.saturated_into();
            ensure!(
                pool.belizechain_liquidity >= amount_u128,
                Error::<T>::InsufficientLiquidity
            );

            // Unreserve funds back to manager
            T::Currency::unreserve(&who, amount);

            pool.belizechain_liquidity = pool.belizechain_liquidity.saturating_sub(amount_u128);

            // Deactivate if fully drained
            if pool.belizechain_liquidity == 0 {
                pool.active = false;
            }

            LiquidityPools::<T>::insert(pool_id, pool);

            Self::deposit_event(Event::AssetsUnlocked {
                account: who,
                amount: amount_u128,
                asset: 0,        // DALLA
                source_chain: 0, // Internal pool withdrawal
            });

            Ok(())
        }

        /// P0-1: Oracle operator attests that a burn event was verified on the source chain.
        ///
        /// Each registered oracle operator independently confirms they observed the burn
        /// transaction on the external chain. Validators may only sign a `BurnAndUnlock`
        /// transaction via `provide_pq_signature` once `MinOracleConfirmations` have been
        /// collected, closing the CONS-003 gap (validator-only attestation without proof).
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::confirm_burn_proof())]
        pub fn confirm_burn_proof(origin: OriginFor<T>, tx_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered oracle operators may confirm burns
            ensure!(
                T::OracleCheck::is_oracle_operator(&who),
                Error::<T>::NotOracleOperator
            );

            // Sanctions check
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            let bridge_tx =
                Self::bridge_transactions(tx_id).ok_or(Error::<T>::TransactionNotFound)?;

            // Only BurnAndUnlock transactions require burn proof
            ensure!(
                matches!(bridge_tx.operation, BridgeOperation::BurnAndUnlock { .. }),
                Error::<T>::NotBurnTransaction
            );

            // Transaction must be in a signable state
            ensure!(
                matches!(
                    bridge_tx.status,
                    BridgeStatus::Initiated | BridgeStatus::AwaitingSignatures
                ),
                Error::<T>::AlreadyExecuted
            );

            // Prevent duplicate confirmation from same oracle
            ensure!(
                !BurnConfirmations::<T>::get(tx_id, &who),
                Error::<T>::AlreadyConfirmedBurn
            );

            // Record confirmation
            BurnConfirmations::<T>::insert(tx_id, &who, true);
            let new_count = BurnConfirmationCount::<T>::mutate(tx_id, |c| {
                *c = c.saturating_add(1);
                *c
            });

            Self::deposit_event(Event::BurnProofConfirmed {
                tx_id,
                oracle: who,
                confirmations: new_count,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// CONS-027: Derives the escrow account ID from the pallet's PalletId.
        fn escrow_account() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        // P0-5: Block-number-tagged rate limit — no unbounded clear() needed.
        // Stale entries from old blocks are naturally treated as count=0.
        fn check_bridge_rate_limit(who: &T::AccountId) -> frame_support::dispatch::DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (stored_block, stored_count) = BridgeCallsThisBlock::<T>::get(who);
            let count = if stored_block == current_block {
                stored_count.saturating_add(1)
            } else {
                1u32
            };
            ensure!(
                count <= T::MaxBridgePerBlock::get(),
                Error::<T>::RateLimitExceeded
            );
            BridgeCallsThisBlock::<T>::insert(who, (current_block, count));
            Ok(())
        }

        pub fn decode_chain(index: u8) -> Option<BridgeChain> {
            use BridgeChain::*;
            Some(match index as u16 {
                0 => Bitcoin,
                1 => Ethereum,
                2 => Solana,
                3 => BinanceSmartChain,
                4 => Tron,
                5 => Ripple,
                6 => Cardano,
                7 => Dogecoin,
                8 => Polygon,
                9 => Litecoin,
                10 => Polkadot,
                11 => Avalanche,
                12 => CosmosHub,
                13 => Ton,
                14 => InternetComputer,
                15 => Near,
                16 => Stellar,
                17 => Algorand,
                18 => Tezos,
                19 => EOS,
                20 => Hedera,
                21 => Fantom,
                22 => Aptos,
                23 => Sui,
                24 => Kava,
                25 => Celo,
                26 => Harmony,
                27 => Cronos,
                28 => Thorchain,
                29 => Gnosis,
                30 => ArbitrumOne,
                31 => Optimism,
                32 => Base,
                33 => ZkSyncEra,
                34 => Linea,
                35 => Scroll,
                36 => Mantle,
                37 => PolygonZkEvm,
                38 => Metis,
                39 => Boba,
                40 => Zora,
                41 => Moonbeam,
                42 => Moonriver,
                43 => Kusama,
                44 => OKTC,
                45 => Waves,
                46 => Qtum,
                47 => BitTorrentChain,
                48 => ICON,
                49 => VeChain,
                50 => XCM,
                _ => return None,
            })
        }

        pub fn decode_asset(index: u8) -> Option<BridgeAsset> {
            match index {
                0 => Some(BridgeAsset::DALLA),
                1 => Some(BridgeAsset::BBZD),
                _ => None,
            }
        }

        pub fn encode_chain(chain: &BridgeChain) -> u8 {
            use BridgeChain::*;
            match chain {
                Bitcoin => 0,
                Ethereum => 1,
                Solana => 2,
                BinanceSmartChain => 3,
                Tron => 4,
                Ripple => 5,
                Cardano => 6,
                Dogecoin => 7,
                Polygon => 8,
                Litecoin => 9,
                Polkadot => 10,
                Avalanche => 11,
                CosmosHub => 12,
                Ton => 13,
                InternetComputer => 14,
                Near => 15,
                Stellar => 16,
                Algorand => 17,
                Tezos => 18,
                EOS => 19,
                Hedera => 20,
                Fantom => 21,
                Aptos => 22,
                Sui => 23,
                Kava => 24,
                Celo => 25,
                Harmony => 26,
                Cronos => 27,
                Thorchain => 28,
                Gnosis => 29,
                ArbitrumOne => 30,
                Optimism => 31,
                Base => 32,
                ZkSyncEra => 33,
                Linea => 34,
                Scroll => 35,
                Mantle => 36,
                PolygonZkEvm => 37,
                Metis => 38,
                Boba => 39,
                Zora => 40,
                Moonbeam => 41,
                Moonriver => 42,
                Kusama => 43,
                OKTC => 44,
                Waves => 45,
                Qtum => 46,
                BitTorrentChain => 47,
                ICON => 48,
                VeChain => 49,
                XCM => 50,
            }
        }

        pub fn encode_asset(asset: &BridgeAsset) -> u8 {
            match asset {
                BridgeAsset::DALLA => 0,
                BridgeAsset::BBZD => 1,
            }
        }
        /// Get bridge transaction by ID
        pub fn get_bridge_transaction(
            tx_id: u32,
        ) -> Option<BridgeTransaction<T::AccountId, BlockNumberFor<T>>> {
            Self::bridge_transactions(tx_id)
        }

        /// Get total locked assets for a chain and asset
        pub fn get_total_locked(chain: &BridgeChain, asset: &BridgeAsset) -> u128 {
            Self::total_locked_assets(chain, asset)
        }

        /// Check if chain is supported
        pub fn is_chain_supported(chain: &BridgeChain) -> bool {
            Self::chain_configurations(chain).is_some()
        }
    }
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn initiate_bridge() -> Weight;
    fn provide_signature() -> Weight;
    fn create_liquidity_pool() -> Weight;
    fn process_unlock() -> Weight;
    fn send_message() -> Weight;
    fn update_config() -> Weight;
    fn confirm_burn_proof() -> Weight;
}

impl WeightInfo for () {
    fn initiate_bridge() -> Weight {
        Weight::from_parts(25_000_000, 512).saturating_add(Weight::from_parts(0, 3500))
    }
    fn provide_signature() -> Weight {
        Weight::from_parts(15_000_000, 512).saturating_add(Weight::from_parts(0, 2000))
    }
    fn create_liquidity_pool() -> Weight {
        Weight::from_parts(20_000_000, 512).saturating_add(Weight::from_parts(0, 3000))
    }
    fn process_unlock() -> Weight {
        Weight::from_parts(18_000_000, 512).saturating_add(Weight::from_parts(0, 2500))
    }
    fn send_message() -> Weight {
        Weight::from_parts(12_000_000, 512).saturating_add(Weight::from_parts(0, 2000))
    }
    fn update_config() -> Weight {
        Weight::from_parts(8_000_000, 512).saturating_add(Weight::from_parts(0, 1000))
    }
    fn confirm_burn_proof() -> Weight {
        Weight::from_parts(15_000_000, 512).saturating_add(Weight::from_parts(0, 2000))
    }
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
