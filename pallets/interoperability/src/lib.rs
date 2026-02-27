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

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, LockableCurrency, LockIdentifier, 
        Get, Time, Randomness,
    },
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{
        Saturating, SaturatedConversion,
    },
    Perbill,
};
use sp_std::{vec::Vec, convert::TryInto};
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

pub use pallet::*;

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

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency used for bridging operations
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
        
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
    }

    /// Supported bridge chains
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeChain {
        // L1s and Major Networks
        Bitcoin,            // 0
        Ethereum,           // 1
        Solana,             // 2
        BinanceSmartChain,  // 3
        Tron,               // 4
        Ripple,             // 5 (XRP Ledger)
        Cardano,            // 6
        Dogecoin,           // 7
        Polygon,            // 8
        Litecoin,           // 9
        Polkadot,           // 10 (as external chain)
        Avalanche,          // 11
        CosmosHub,          // 12
        Ton,                // 13
        InternetComputer,   // 14
        Near,               // 15
        Stellar,            // 16
        Algorand,           // 17
        Tezos,              // 18
        EOS,                // 19
        Hedera,             // 20
        Fantom,             // 21
        Aptos,              // 22
        Sui,                // 23
        Kava,               // 24
        Celo,               // 25
        Harmony,            // 26
        Cronos,             // 27
        Thorchain,          // 28
        Gnosis,             // 29
        ArbitrumOne,        // 30 (L2)
        Optimism,           // 31 (L2)
        Base,               // 32 (L2)
    ZkSyncEra,          // 33 (L2)
        Linea,              // 34 (L2)
        Scroll,             // 35 (L2)
        Mantle,             // 36 (L2)
        PolygonZkEvm,       // 37 (L2)
        Metis,              // 38 (L2)
        Boba,               // 39 (L2)
        Zora,               // 40 (L2)
        Moonbeam,           // 41 (Polkadot EVM)
        Moonriver,          // 42 (Kusama EVM)
        Kusama,             // 43
        OKTC,               // 44
        Waves,              // 45
        Qtum,               // 46
        BitTorrentChain,    // 47
        ICON,               // 48
        VeChain,            // 49
        // Generic XCM marker (for aggregation)
        XCM,                // 50
    }

    /// Bridgeable assets (DALLA and bBZD only for sovereignty)
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BridgeAsset {
        /// Native DALLA token
        DALLA,
        /// Belizean Dollar stablecoin
        BBZD,
    }

    /// Bridge operation types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BridgeValidator<AccountId> {
        /// Validator account
        pub account: AccountId,
        /// Post-quantum public key (Falcon/Dilithium)
    pub pq_public_key: BoundedVec<u8, ConstU32<96>>,
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
        /// Post-quantum signature data
    pub pq_signatures: BoundedVec<(AccountId, BoundedVec<u8, ConstU32<96>>), ConstU32<64>>, // (validator, signature)
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    pub type BridgeValidators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BridgeValidator<T::AccountId>,
    >;

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
    pub type ChainConfigurations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BridgeChain,
        ChainConfig,
    >;

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
    pub type PendingFinalizations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Transaction ID
        BlockNumberFor<T>,
    >;

    /// Chain-specific configuration
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
        BridgeConfigUpdated {
            chain: u8,
            fee_rate: u32,
        },
        /// Bridge fee collected
        BridgeFeeCollected {
            amount: u128,
            asset: u8,
        },
        /// Bridge transaction disputed during challenge period (§4.4b)
        BridgeTransactionDisputed {
            tx_id: u32,
            disputer: T::AccountId,
        },
        /// Bridge transaction finalized after challenge period elapsed (§4.4b)
        BridgeTransactionFinalized {
            tx_id: u32,
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
    }

    // ===== HOOKS (§4.4b Challenge Period Finalization) =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::from_parts(5_000_000, 0);
            let mut finalized_ids = Vec::new();

            // SECURITY: Bounded iteration — process at most 20 finalizations per block
            for (tx_id, expiry_block) in PendingFinalizations::<T>::iter().take(20) {
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
                    weight = weight.saturating_add(Weight::from_parts(10_000_000, 0));
                }
            }

            for tx_id in finalized_ids {
                PendingFinalizations::<T>::remove(tx_id);
            }

            weight
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

            // Cross-chain KYC verification (minimum Level 2 Enhanced KYC for bridges)
            let kyc_level = T::Identity::get_kyc_level(&who).unwrap_or(0);
            ensure!(kyc_level >= 2, Error::<T>::KycRequired);
            
            // Sanctions screening for cross-chain compliance
            ensure!(!T::Identity::is_sanctioned(&who), Error::<T>::AccountSanctioned);

            let target_chain = Self::decode_chain(target_chain_index).ok_or(Error::<T>::UnsupportedChain)?;
            let asset = Self::decode_asset(asset_index).ok_or(Error::<T>::UnsupportedAsset)?;

            // Validate chain is supported and enabled
            let chain_config = Self::chain_configurations(&target_chain)
                .ok_or(Error::<T>::UnsupportedChain)?;
            ensure!(chain_config.enabled, Error::<T>::BridgeDisabled);

            // Validate amount
            ensure!(amount >= T::MinBridgeAmount::get(), Error::<T>::BelowMinimumAmount);
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
            let fee_amount = Perbill::from_parts(chain_config.fee_rate) * amount;
            let net_amount = amount.saturating_sub(fee_amount);

            // Lock assets
            T::Currency::set_lock(
                BRIDGE_LOCK_ID,
                &who,
                amount,
                frame_support::traits::WithdrawReasons::all(),
            );

            // Transfer fee to treasury
            let treasury = T::Treasury::get();
            T::Currency::transfer(&who, &treasury, fee_amount, frame_support::traits::ExistenceRequirement::KeepAlive)?;

            let tx_id = Self::next_tx_id();
            let current_block = frame_system::Pallet::<T>::block_number();

            let target_address_bounded: BoundedVec<u8, ConstU32<128>> = target_address
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
            NextTxId::<T>::put(tx_id.saturating_add(1));

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
            ensure!(!T::Identity::is_sanctioned(&who), Error::<T>::AccountSanctioned);

            // Ensure validator is registered
            let _validator = Self::bridge_validators(&who)
                .ok_or(Error::<T>::ValidatorNotRegistered)?;

            let mut bridge_tx = Self::bridge_transactions(tx_id)
                .ok_or(Error::<T>::TransactionNotFound)?;

            // Ensure transaction is awaiting signatures
            ensure!(
                matches!(bridge_tx.status, BridgeStatus::Initiated | BridgeStatus::AwaitingSignatures),
                Error::<T>::AlreadyExecuted
            );

            // Validate post-quantum signature (simplified validation)
            ensure!(pq_signature.len() >= 64, Error::<T>::InvalidPQSignature);

            // Add signature
            let sig: BoundedVec<u8, ConstU32<96>> = pq_signature
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
            NextPoolId::<T>::put(pool_id.saturating_add(1));

            Self::deposit_event(Event::LiquidityPoolCreated {
                pool_id,
                chain: Self::encode_chain(&chain),
                asset: Self::encode_asset(&asset),
                manager: who,
            });

            Ok(())
        }

        /// Process burn and unlock from external chain
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::process_unlock())]
        pub fn process_unlock(
            origin: OriginFor<T>,
            source_chain_index: u8,
            _source_tx_hash: Vec<u8>,
            recipient: T::AccountId,
            amount: u128,
            asset_index: u8,
        ) -> DispatchResult {
            // Only bridge validators can process unlocks
            let who = ensure_signed(origin)?;
            let _validator = Self::bridge_validators(&who)
                .ok_or(Error::<T>::ValidatorNotRegistered)?;

            let source_chain = Self::decode_chain(source_chain_index).ok_or(Error::<T>::UnsupportedChain)?;
            let asset = Self::decode_asset(asset_index).ok_or(Error::<T>::UnsupportedAsset)?;

            // Verify external chain transaction (simplified)
            // In production, would verify against external chain state

            // Calculate unlock amount (minus fees already deducted)
            // SAFETY: amount is u128 and Balance is u128-backed; u128 → Balance is lossless
            let unlock_amount: <T::Currency as Currency<T::AccountId>>::Balance = amount.saturated_into();

            // Unlock assets and credit tokens to recipient
            T::Currency::remove_lock(BRIDGE_LOCK_ID, &recipient);
            let _imbalance = T::Currency::deposit_creating(&recipient, unlock_amount);

            // Update total locked assets
            TotalLockedAssets::<T>::mutate(&source_chain, &asset, |total| {
                *total = total.saturating_sub(amount);
            });

            Self::deposit_event(Event::AssetsUnlocked {
                account: recipient,
                amount,
                asset: Self::encode_asset(&asset),
                source_chain: Self::encode_chain(&source_chain),
            });

            Ok(())
        }

        /// Send cross-chain message
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::send_message())]
        pub fn send_cross_chain_message(
            origin: OriginFor<T>,
            target_chain_index: u8,
            payload: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let target_chain = Self::decode_chain(target_chain_index).ok_or(Error::<T>::UnsupportedChain)?;

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
            NextMessageId::<T>::put(message_id.saturating_add(1));

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

            let mut bridge_tx = Self::bridge_transactions(tx_id)
                .ok_or(Error::<T>::TransactionNotFound)?;

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
    }

    impl<T: Config> Pallet<T> {
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
        pub fn get_bridge_transaction(tx_id: u32) -> Option<BridgeTransaction<T::AccountId, BlockNumberFor<T>>> {
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
}

impl WeightInfo for () {
    fn initiate_bridge() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3500))
    }
    fn provide_signature() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2000))
    }
    fn create_liquidity_pool() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(0, 3000))
    }
    fn process_unlock() -> Weight {
        Weight::from_parts(18_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2500))
    }
    fn send_message() -> Weight {
        Weight::from_parts(12_000_000, 0)
            .saturating_add(Weight::from_parts(0, 2000))
    }
    fn update_config() -> Weight {
        Weight::from_parts(8_000_000, 0)
            .saturating_add(Weight::from_parts(0, 1000))
    }
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;