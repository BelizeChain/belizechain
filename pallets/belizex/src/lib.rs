#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeX Decentralized Exchange Pallet
//!
//! This pallet implements:
//! 1. Automated Market Maker (AMM) for DALLA/bBZD pairs
//! 2. Liquidity provision with LP tokens
//! 3. Tourism-optimized trading with reduced fees
//! 4. Cross-border remittance trading pairs
//! 5. Integration with Belize economic incentives

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, LockableCurrency, Get, Randomness,
        ExistenceRequirement,
    },
    weights::{Weight, constants::RocksDbWeight},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{
        Saturating, SaturatedConversion, IntegerSquareRoot, AccountIdConversion,
    },
    RuntimeDebug, FixedU128, FixedPointNumber,
};

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
// use sp_std::collections::btree_map::BTreeMap; // No per-pair LP token map stored currently

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// LIQUIDITY_LOCK_ID removed: custody model now uses transfer-to-pool instead of locks

/// Minimal trait to check KYC status of an account.
pub trait KycCheck<AccountId> {
    fn is_kyc_ok(who: &AccountId) -> bool;
}

/// Trait for Oracle integration - provides real-time market data and merchant verification
pub trait BelizeXOracleProvider<AccountId> {
    /// Get real-time exchange rate for a crypto trading pair (returns rate * 10^6 for precision)
    fn get_crypto_exchange_rate(base_asset: u8, quote_asset: u8) -> Option<u128>;
    
    /// Verify if account is a registered tourism merchant (eligible for fee discounts)
    fn is_tourism_merchant(account: &AccountId) -> bool;
    
    /// Get trading volume tier for an account (0-3, higher tier = lower fees)
    fn get_trading_volume_tier(account: &AccountId) -> u8;
}

// test modules are not included in this workspace configuration


#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for DEX operations
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;

        /// X-10 NOTE: Randomness source reserved for future LP token generation.
        /// Currently unused — required by Config trait for forward compatibility.
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// Tourism origin for fee reductions
        type TourismOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// Governance origin for listing new trading pairs
    type PairListingOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Treasury account for DEX fees
        type Treasury: Get<Self::AccountId>;

        /// Trading fee rate (in basis points)
        #[pallet::constant]
        type TradingFeeRate: Get<u32>;

        /// Tourism discount rate (in basis points)
        #[pallet::constant]
        type TourismDiscountRate: Get<u32>;

        /// Minimum liquidity provision amount
        #[pallet::constant]
        type MinLiquidityAmount: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// KYC checker implementation
        type Kyc: KycCheck<Self::AccountId>;

        /// Portion of fee (in bps) that goes to Treasury; remainder benefits LPs
        #[pallet::constant]
        type ProtocolFeeToTreasuryBps: Get<u32>;

        /// Oracle provider for real-time market data and merchant verification
        type Oracle: BelizeXOracleProvider<Self::AccountId>;

        /// Maximum allowed deviation from Oracle rate for BBZD trades (in basis points)
        /// Used as a safety guard to prevent extreme price manipulation when trading bBZD.
        #[pallet::constant]
        type MaxOracleDeviationBps: Get<u32>;

        /// Pallet ID used to derive the DEX pool escrow account
        #[pallet::constant]
        type DexPalletId: Get<PalletId>;
    }

    // X-9 FIX: Removed dead LiquidityPosition struct (unused scaffolding).
    // Active LP tracking uses LiquidityProvider and LPBalances storage.

    /// Trading order information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    pub struct TradingPair {
        /// Base asset identifier
        pub base_asset: AssetId,
        /// Quote asset identifier  
        pub quote_asset: AssetId,
        /// Total base asset in pool
        pub base_reserve: u128,
        /// Total quote asset in pool
        pub quote_reserve: u128,
        /// Total LP tokens issued
        pub total_lp_tokens: u128,
        /// Trading fee rate for this pair
        pub fee_rate: u32,
        /// Pair active status
        pub active: bool,
    }

    /// Asset identifiers in BelizeChain
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Ord, PartialOrd)]
    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    pub enum AssetId {
        /// Native DALLA token
        DALLA,
        /// Belizean Dollar stablecoin
        BBZD,
        /// Tourism DALLA (with special benefits)
        TourismDALLA,
        /// Wrapped USDC (USD-pegged)
        WUSDC,
    }

    impl AssetId {
        /// Convert AssetId to u8
        pub fn as_u8(&self) -> u8 {
            match self {
                AssetId::DALLA => 0,
                AssetId::BBZD => 1,
                AssetId::TourismDALLA => 2,
                AssetId::WUSDC => 3,
            }
        }
    }

    impl From<u8> for AssetId {
        fn from(value: u8) -> Self {
            match value {
                0 => AssetId::DALLA,
                1 => AssetId::BBZD,
                2 => AssetId::TourismDALLA,
                3 => AssetId::WUSDC,
                _ => AssetId::DALLA, // Default
            }
        }
    }

    /// Liquidity provider information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct LiquidityProvider<AccountId> {
        /// Account providing liquidity
        pub provider: AccountId,
        /// Total value locked
        pub total_value_locked: u128,
        /// Rewards earned
        pub rewards_earned: u128,
        /// Tourism provider status
        pub is_tourism_provider: bool,
    }

    /// Order book entry for limit orders
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct OrderEntry<AccountId, BlockNumber> {
        /// Order ID
        pub order_id: u32,
        /// Order creator
        pub creator: AccountId,
        /// Trading pair
        pub pair: (AssetId, AssetId),
        /// Order type (buy/sell)
        pub order_type: OrderType,
        /// Amount to trade
        pub amount: u128,
        /// Price per unit
        pub price: u128,
        /// Remaining amount
        pub remaining: u128,
        /// Block when order expires
        pub expires_at: BlockNumber,
        /// Tourism order flag
        pub is_tourism_order: bool,
    }

    /// Order types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    pub enum OrderType {
        /// Buy order
        Buy,
        /// Sell order
        Sell,
        /// Tourism-specific order with benefits
        TourismBuy,
        /// Tourism-specific sell order
        TourismSell,
    }

    impl OrderType {
        /// Convert OrderType to u8
        pub fn as_u8(&self) -> u8 {
            match self {
                OrderType::Buy => 0,
                OrderType::Sell => 1,
                OrderType::TourismBuy => 2,
                OrderType::TourismSell => 3,
            }
        }
    }

    impl From<u8> for OrderType {
        fn from(value: u8) -> Self {
            match value {
                0 => OrderType::Buy,
                1 => OrderType::Sell,
                2 => OrderType::TourismBuy,
                3 => OrderType::TourismSell,
                _ => OrderType::Buy, // Default
            }
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn trading_pairs)]
    /// Trading pairs available on BelizeX
    pub type TradingPairs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (u8, u8),
        TradingPair,
    >;

    #[pallet::storage]
    #[pallet::getter(fn liquidity_providers)]
    /// Liquidity provider information
    pub type LiquidityProviders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        LiquidityProvider<T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn order_book)]
    /// Order book for limit orders
    pub type OrderBook<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Order ID
        OrderEntry<T::AccountId, BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    /// Next available order ID
    pub type NextOrderId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn daily_volume)]
    /// Daily trading volume per pair
    pub type DailyVolume<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (u8, u8),
        u128,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn tourism_traders)]
    /// Tourism-verified traders with benefits
    pub type TourismTraders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn global_paused)]
    /// Global pause flag for DEX operations
    pub type GlobalPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    // Dev-only helpers to auto-seed minimal liquidity on startup
    #[pallet::storage]
    #[pallet::getter(fn dev_seed_enabled)]
    pub type DevSeedEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn dev_seed_done)]
    pub type DevSeedDone<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// X-1 FIX: Per-user LP token balances per trading pair.
    /// Maps (AccountId, (base_asset, quote_asset)) → LP token amount.
    #[pallet::storage]
    #[pallet::getter(fn lp_balances)]
    pub type LPBalances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        (u8, u8), // pair key
        u128,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// Initial trading pairs - simplified to avoid serde issues
        pub enable_default_pairs: bool,
        /// Initial liquidity providers - simplified
        pub initial_providers: Vec<T::AccountId>,
        /// Dev flag: seed minimal WUSDC/BBZD liquidity at startup
        pub dev_seed_default_liquidity: bool,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                enable_default_pairs: true,
                initial_providers: vec![],
                dev_seed_default_liquidity: true,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if self.enable_default_pairs {
                // Initialize default trading pairs
                let dalla_bbzd_pair = TradingPair {
                    base_asset: AssetId::DALLA,
                    quote_asset: AssetId::BBZD,
                    base_reserve: 0,
                    quote_reserve: 0,
                    total_lp_tokens: 0,
                    fee_rate: 30, // 0.3%
                    active: true,
                };
                TradingPairs::<T>::insert((AssetId::DALLA.as_u8(), AssetId::BBZD.as_u8()), dalla_bbzd_pair);

                let tourism_bbzd_pair = TradingPair {
                    base_asset: AssetId::TourismDALLA,
                    quote_asset: AssetId::BBZD,
                    base_reserve: 0,
                    quote_reserve: 0,
                    total_lp_tokens: 0,
                    fee_rate: 15, // 0.15% for tourism
                    active: true,
                };
                TradingPairs::<T>::insert((AssetId::TourismDALLA.as_u8(), AssetId::BBZD.as_u8()), tourism_bbzd_pair);

                // Default WUSDC/BBZD pair for USDC→bBZD swaps
                let wusdc_bbzd_pair = TradingPair {
                    base_asset: AssetId::WUSDC,
                    quote_asset: AssetId::BBZD,
                    base_reserve: 0,
                    quote_reserve: 0,
                    total_lp_tokens: 0,
                    fee_rate: 30, // 0.3%
                    active: true,
                };
                TradingPairs::<T>::insert((AssetId::WUSDC.as_u8(), AssetId::BBZD.as_u8()), wusdc_bbzd_pair);
            }

            // Enable or disable dev auto-seed via genesis
            DevSeedEnabled::<T>::put(self.dev_seed_default_liquidity);
            DevSeedDone::<T>::put(false);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // AR-10: One-time dev seed moved from on_initialize to on_idle.
        // The seed is non-critical at block-start; it can safely run whenever
        // remaining weight allows rather than consuming mandatory block budget.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }

        fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            // One-time dev seed for WUSDC/BBZD liquidity
            if !Self::dev_seed_enabled() || Self::dev_seed_done() {
                return Weight::zero();
            }
            // Require at least 50M ref_time to safely execute.
            if remaining_weight.ref_time() < 50_000_000 {
                return Weight::zero();
            }

            let pair_key: (u8, u8) = (AssetId::WUSDC.as_u8(), AssetId::BBZD.as_u8());
            if let Some(mut pair) = TradingPairs::<T>::get(pair_key) {
                // Minimal reserves aligned with integration tests (~2.0 price)
                let base_amount: u128 = 10_000_000_000_000;
                let quote_amount: u128 = 20_000_000_000_000;
                let lp_tokens: u128 = (base_amount.saturating_add(quote_amount)).integer_sqrt();

                pair.base_reserve = pair.base_reserve.saturating_add(base_amount);
                pair.quote_reserve = pair.quote_reserve.saturating_add(quote_amount);
                pair.total_lp_tokens = pair.total_lp_tokens.saturating_add(lp_tokens);
                TradingPairs::<T>::insert(pair_key, &pair);

                let treasury = T::Treasury::get();
                LiquidityProviders::<T>::mutate(&treasury, |maybe_provider| {
                    match maybe_provider {
                        Some(provider) => {
                            provider.total_value_locked = provider
                                .total_value_locked
                                .saturating_add(base_amount.saturating_add(quote_amount));
                        },
                        None => {
                            *maybe_provider = Some(LiquidityProvider {
                                provider: treasury.clone(),
                                total_value_locked: base_amount.saturating_add(quote_amount),
                                rewards_earned: 0,
                                is_tourism_provider: false,
                            });
                        }
                    }
                });

                // Emit event for transparency
                Self::deposit_event(Event::LiquidityAdded {
                    provider: treasury,
                    pair: pair_key,
                    base_amount,
                    quote_amount,
                    lp_tokens,
                });
            }

            DevSeedDone::<T>::put(true);
            Weight::zero()
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Trading pair created
        TradingPairCreated {
            base_asset: u8, // AssetId as u8
            quote_asset: u8, // AssetId as u8
        },
        /// Liquidity added to pair
        LiquidityAdded {
            provider: T::AccountId,
            pair: (u8, u8), // (AssetId, AssetId) as (u8, u8)
            base_amount: u128,
            quote_amount: u128,
            lp_tokens: u128,
        },
        /// Oracle guard rejected a trade due to excessive deviation
        OracleGuardRejected {
            pair: (u8, u8),
            implied_rate: u128, // scaled 1e6
            oracle_rate: u128,  // scaled 1e6
            deviation_bps: u128,
            max_allowed_bps: u32,
        },
        /// Oracle rate unavailable or stale for guarded pairs
        OracleRateUnavailable {
            pair: (u8, u8),
        },
        /// Liquidity removed from pair
        LiquidityRemoved {
            provider: T::AccountId,
            pair: (u8, u8), // (AssetId, AssetId) as (u8, u8)
            base_amount: u128,
            quote_amount: u128,
            lp_tokens: u128,
        },
        /// Trade executed
        TradeExecuted {
            trader: T::AccountId,
            pair: (u8, u8), // (AssetId, AssetId) as (u8, u8)
            amount_in: u128,
            amount_out: u128,
            fee_paid: u128,
            is_tourism_trade: bool,
        },
        /// Limit order placed
        OrderPlaced {
            order_id: u32,
            creator: T::AccountId,
            pair: (u8, u8), // (AssetId, AssetId) as (u8, u8)
            order_type: u8, // OrderType as u8
            amount: u128,
            price: u128,
        },
        /// Order executed
        OrderExecuted {
            order_id: u32,
            executor: T::AccountId,
            amount: u128,
        },
        /// Order cancelled by creator
        OrderCancelled {
            order_id: u32,
            creator: T::AccountId,
        },
        /// Tourism trader verified
        TourismTraderVerified {
            trader: T::AccountId,
        },
        /// Global DEX paused
        DexPaused,
        /// Global DEX resumed
        DexResumed,
        /// Trading pair status updated
        PairStatusUpdated {
            base_asset: u8,
            quote_asset: u8,
            active: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Trading pair not found
        PairNotFound,
        /// Trading pair already exists
        PairAlreadyExists,
        /// Insufficient balance for trade
        InsufficientBalance,
        /// Insufficient liquidity in pair
        InsufficientLiquidity,
        /// Amount below minimum threshold
        BelowMinimumAmount,
        /// Order not found
        OrderNotFound,
        /// Order expired
        OrderExpired,
        /// Invalid price
        InvalidPrice,
        /// Slippage tolerance exceeded
        SlippageExceeded,
        /// Trading pair not active
        PairNotActive,
        /// Unauthorized operation
        Unauthorized,
        /// KYC required for this operation
        KycRequired,
        /// Operation is paused
        Paused,
        /// Oracle rate unavailable or stale
        OracleRateUnavailable,
        /// Insufficient LP tokens for withdrawal
        InsufficientLPTokens,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create new trading pair
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_pair())]
        pub fn create_trading_pair(
            origin: OriginFor<T>,
            base_asset: u8, // AssetId as u8
            quote_asset: u8, // AssetId as u8
            fee_rate: u32,
        ) -> DispatchResult {
            // Only governance may list new pairs
            T::PairListingOrigin::ensure_origin(origin)?;

            let pair_key: (u8,u8) = (base_asset, quote_asset);
            ensure!(!TradingPairs::<T>::contains_key(pair_key), Error::<T>::PairAlreadyExists);

            let trading_pair = TradingPair {
                base_asset: AssetId::from(base_asset),
                quote_asset: AssetId::from(quote_asset),
                fee_rate,
                base_reserve: 0u128,
                quote_reserve: 0u128,
                total_lp_tokens: 0u128,
                active: true,
            };

            TradingPairs::<T>::insert(pair_key, trading_pair);

                Self::deposit_event(Event::TradingPairCreated { base_asset, quote_asset });
                Ok(())
        }

        /// Add liquidity to trading pair
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            base_asset: u8, // AssetId as u8
            quote_asset: u8, // AssetId as u8
            base_amount: <T::Currency as Currency<T::AccountId>>::Balance,
            quote_amount: <T::Currency as Currency<T::AccountId>>::Balance,
            min_lp_tokens: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require KYC for liquidity provision
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            let pair_key: (u8,u8) = (base_asset, quote_asset);
            let mut pair = Self::trading_pairs(pair_key)
                .ok_or(Error::<T>::PairNotFound)?;

            ensure!(pair.active, Error::<T>::PairNotActive);

            // Ensure sufficient balance
            let total_amount = base_amount.saturating_add(quote_amount);
            ensure!(
                T::Currency::free_balance(&who) >= total_amount,
                Error::<T>::InsufficientBalance
            );

            // SAFETY(saturated_into): Balance → u128 is lossless for standard substrate balances
            // (u128 max > any practical token supply). If a custom Balance type exceeds u128,
            // values would saturate at u128::MAX — acceptable since LP math already uses
            // saturating arithmetic.
            let base_amount_u128: u128 = base_amount.saturated_into();
            let quote_amount_u128: u128 = quote_amount.saturated_into();

            // Calculate LP tokens to mint
            // M47 FIX: Use proportional formula for existing pools, sqrt only for initial
            let lp_tokens = if pair.total_lp_tokens == 0 {
                // Initial deposit: sqrt(base * quote) — standard constant-product AMM
                base_amount_u128.saturating_mul(quote_amount_u128).integer_sqrt()
            } else {
                // Subsequent deposits: min(base/base_reserve, quote/quote_reserve) * total_lp
                // This preserves pool ratio and prevents LP value dilution
                let lp_from_base = base_amount_u128
                    .saturating_mul(pair.total_lp_tokens)
                    / pair.base_reserve.max(1);
                let lp_from_quote = quote_amount_u128
                    .saturating_mul(pair.total_lp_tokens)
                    / pair.quote_reserve.max(1);
                lp_from_base.min(lp_from_quote)
            };
            
            let min_lp_tokens_u128: u128 = min_lp_tokens;
            ensure!(lp_tokens >= min_lp_tokens_u128, Error::<T>::SlippageExceeded);

            // Transfer funds to the DEX pool escrow account
            let pool = Self::pool_account();
            T::Currency::transfer(
                &who,
                &pool,
                base_amount.saturating_add(quote_amount),
                ExistenceRequirement::KeepAlive,
            )?;

            // Update pair reserves
            pair.base_reserve = pair.base_reserve.saturating_add(base_amount_u128);
            pair.quote_reserve = pair.quote_reserve.saturating_add(quote_amount_u128);
            pair.total_lp_tokens = pair.total_lp_tokens.saturating_add(lp_tokens);

            TradingPairs::<T>::insert(pair_key, &pair);

            // X-1 FIX: Track LP tokens per user per pair
            LPBalances::<T>::mutate(&who, pair_key, |balance| {
                *balance = balance.saturating_add(lp_tokens);
            });

            // Update liquidity provider info
            LiquidityProviders::<T>::mutate(&who, |maybe_provider| {
                match maybe_provider {
                    Some(provider) => {
                        provider.total_value_locked = provider
                            .total_value_locked
                            .saturating_add(base_amount_u128.saturating_add(quote_amount_u128));
                    },
                    None => {
                        *maybe_provider = Some(LiquidityProvider {
                            provider: who.clone(),
                            total_value_locked: base_amount_u128.saturating_add(quote_amount_u128),
                            rewards_earned: 0,
                            is_tourism_provider: Self::tourism_traders(&who),
                        });
                    }
                }
            });

            Self::deposit_event(Event::LiquidityAdded {
                provider: who.clone(),
                pair: (base_asset, quote_asset),
                base_amount: base_amount_u128,
                quote_amount: quote_amount_u128,
                lp_tokens,
            });            Ok(())
        }

        /// Execute market trade
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::execute_trade())]
        pub fn execute_trade(
            origin: OriginFor<T>,
            base_asset: u8, // AssetId as u8
            quote_asset: u8, // AssetId as u8
            amount_in: <T::Currency as Currency<T::AccountId>>::Balance,
            min_amount_out: u128,
            is_tourism_trade: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require KYC for trading
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            let pair_key: (u8,u8) = (base_asset, quote_asset);
            let mut pair = Self::trading_pairs(pair_key)
                .ok_or(Error::<T>::PairNotFound)?;

            ensure!(pair.active, Error::<T>::PairNotActive);

            // Verify tourism trade authorization
            if is_tourism_trade {
                ensure!(Self::tourism_traders(&who), Error::<T>::Unauthorized);
            }

            // SAFETY(saturated_into): Balance → u128 is lossless; substrate balances fit within u128.
            let amount_in_u128: u128 = amount_in.saturated_into();
            
            // Calculate dynamic fee with Oracle-enhanced volume tiers and tourism verification
            let is_verified_tourism = Self::is_verified_tourism_merchant(&who);
            let effective_fee_rate = Self::get_effective_fee_rate(&who, is_verified_tourism && is_tourism_trade);
            
            let fee = amount_in_u128.saturating_mul(effective_fee_rate as u128) / 10_000;
            let treasury_fee_rate = core::cmp::min(T::ProtocolFeeToTreasuryBps::get(), effective_fee_rate);
            let treasury_fee = amount_in_u128.saturating_mul(treasury_fee_rate as u128) / 10_000;
            let lp_fee = fee.saturating_sub(treasury_fee);
            // Amount used in swap formula (after total fee)
            let amount_after_fee = amount_in_u128.saturating_sub(fee);

            // Calculate output amount using constant product formula
            let amount_out = Self::get_amount_out(
                amount_after_fee,
                pair.base_reserve,
                pair.quote_reserve,
            )?;

            // Oracle-based slippage guard for WUSDC/BBZD trades only
            // Compare implied rate vs Oracle-verified USD/BZD rate (both scaled by 1e6 precision)
            if base_asset == AssetId::WUSDC.as_u8() && quote_asset == AssetId::BBZD.as_u8() {
                // Require a fresh Oracle rate; if missing/stale, reject
                if let Some(oracle_rate) = T::Oracle::get_crypto_exchange_rate(base_asset, quote_asset) {
                    let implied_rate = amount_out.saturating_mul(1_000_000) / amount_in_u128;
                    if oracle_rate > 0 {
                        let diff = implied_rate.abs_diff(oracle_rate);
                        let deviation_bps = diff.saturating_mul(10_000) / oracle_rate;
                        let max_allowed = T::MaxOracleDeviationBps::get() as u128;
                        if deviation_bps > max_allowed {
                            Self::deposit_event(Event::OracleGuardRejected {
                                pair: (base_asset, quote_asset),
                                implied_rate,
                                oracle_rate,
                                deviation_bps,
                                max_allowed_bps: T::MaxOracleDeviationBps::get(),
                            });
                            return Err(Error::<T>::SlippageExceeded.into());
                        }
                    }
                } else {
                    // No Oracle rate available (likely stale or not set) → reject trade
                    Self::deposit_event(Event::OracleRateUnavailable { pair: (base_asset, quote_asset) });
                    return Err(Error::<T>::OracleRateUnavailable.into());
                }
            }

            ensure!(amount_out >= min_amount_out, Error::<T>::SlippageExceeded);

            // Transfer input amount from trader to DEX pool
            let pool = Self::pool_account();
            T::Currency::transfer(
                &who,
                &pool,
                // SAFETY(saturated_into): Balance → Balance identity; saturation is a no-op.
                amount_in.saturated_into(),
                ExistenceRequirement::KeepAlive,
            )?;

            // Transfer output amount from DEX pool to trader
            // SAFETY(saturated_into): u128 → Balance; output is bounded by pool reserve which fits in Balance.
            let amount_out_balance: <T::Currency as Currency<T::AccountId>>::Balance =
                amount_out.saturated_into();
            T::Currency::transfer(
                &pool,
                &who,
                amount_out_balance,
                ExistenceRequirement::AllowDeath,
            )?;

            // Update reserves: LP fee is retained in pool
            pair.base_reserve = pair.base_reserve
                .saturating_add(amount_after_fee)
                .saturating_add(lp_fee);
            pair.quote_reserve = pair.quote_reserve.saturating_sub(amount_out);

            TradingPairs::<T>::insert(pair_key, pair);

            // Update daily volume
            DailyVolume::<T>::mutate(pair_key, |volume| {
                *volume = volume.saturating_add(amount_in_u128);
            });

            // Transfer treasury portion of fee from pool to treasury
            let treasury = T::Treasury::get();
            T::Currency::transfer(
                &pool, 
                &treasury, 
                // SAFETY(saturated_into): u128 → Balance; treasury fee ≤ amount_in which originated as Balance.
                treasury_fee.saturated_into(),
                ExistenceRequirement::AllowDeath,
            )?;

            Self::deposit_event(Event::TradeExecuted {
                trader: who,
                pair: (base_asset, quote_asset),
                amount_in: amount_in_u128,
                amount_out,
                fee_paid: fee,
                is_tourism_trade,
            });

            Ok(())
        }

        /// Register as tourism trader
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::register_tourism_trader())]
        pub fn register_tourism_trader(
            origin: OriginFor<T>,
            trader: T::AccountId,
        ) -> DispatchResult {
            T::TourismOrigin::ensure_origin(origin)?;

            TourismTraders::<T>::insert(&trader, true);

            Self::deposit_event(Event::TourismTraderVerified {
                trader,
            });

            Ok(())
        }

        /// Place limit order
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::place_order())]
        pub fn place_limit_order(
            origin: OriginFor<T>,
            base_asset: u8, // AssetId as u8
            quote_asset: u8, // AssetId as u8
            order_type: u8, // OrderType as u8
            amount: u128,
            price: u128,
            expires_in_blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require KYC for order placement
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            let base_asset_id = AssetId::from(base_asset);
            let quote_asset_id = AssetId::from(quote_asset);
            let order_type_enum = OrderType::from(order_type);
            let pair_key: (u8,u8) = (base_asset, quote_asset);
            ensure!(TradingPairs::<T>::contains_key(pair_key), Error::<T>::PairNotFound);

            ensure!(price > 0, Error::<T>::InvalidPrice);

            let order_id = Self::next_order_id();
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let expires_in_u64: u64 = TryInto::<u64>::try_into(expires_in_blocks).unwrap_or(0);
            let expires_u64 = current_u64.saturating_add(expires_in_u64);
            // SAFETY(saturated_into): u64 → BlockNumber. If the computed expiry exceeds
            // BlockNumber::MAX it saturates rather than wrapping — order simply expires
            // at the maximum possible block.
            let expires_at: BlockNumberFor<T> = expires_u64.saturated_into();

            let is_tourism_order = matches!(order_type_enum, OrderType::TourismBuy | OrderType::TourismSell);
            if is_tourism_order {
                ensure!(Self::tourism_traders(&who), Error::<T>::Unauthorized);
            }

            let order = OrderEntry {
                order_id,
                creator: who.clone(),
                pair: (base_asset_id.clone(), quote_asset_id.clone()),
                order_type: order_type_enum.clone(),
                amount,
                price,
                remaining: amount,
                expires_at,
                is_tourism_order,
            };

            OrderBook::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id.saturating_add(1));

            Self::deposit_event(Event::OrderPlaced {
                order_id,
                creator: who,
                pair: (base_asset_id.as_u8(), quote_asset_id.as_u8()),
                order_type: order_type_enum.as_u8(),
                amount,
                price,
            });

            Ok(())
        }

        /// Execute a multihop market trade along a path of AssetIds (as u8)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::execute_trade())]
        pub fn execute_multihop_trade(
            origin: OriginFor<T>,
            path: Vec<u8>,
            amount_in: <T::Currency as Currency<T::AccountId>>::Balance,
            min_amount_out: u128,
            is_tourism_trade: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require KYC for multihop trade
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            ensure!(path.len() >= 2, Error::<T>::PairNotFound);
            ensure!(path.len() <= 5, Error::<T>::SlippageExceeded); // simple path limit

            // SAFETY(saturated_into): Balance → u128 is lossless; substrate balances fit within u128.
            let mut amount: u128 = amount_in.saturated_into();
            let mut total_treasury_fee: u128 = 0;

            // X-2 FIX: Transfer input tokens from trader to DEX pool before executing hops
            let pool = Self::pool_account();
            T::Currency::transfer(
                &who,
                &pool,
                amount_in.saturated_into(),
                ExistenceRequirement::KeepAlive,
            )?;

            for w in path.windows(2) {
                let a = w[0];
                let b = w[1];
                let pair_key = (a, b);
                let mut pair = Self::trading_pairs(pair_key).ok_or(Error::<T>::PairNotFound)?;
                ensure!(pair.active, Error::<T>::PairNotActive);

                let base_fee_rate = if is_tourism_trade {
                    pair.fee_rate.saturating_sub(T::TourismDiscountRate::get())
                } else {
                    pair.fee_rate
                };
                let effective_fee_rate = base_fee_rate;
                let fee = amount.saturating_mul(effective_fee_rate as u128) / 10_000;
                let treasury_fee_rate = core::cmp::min(T::ProtocolFeeToTreasuryBps::get(), effective_fee_rate);
                let treasury_fee = amount.saturating_mul(treasury_fee_rate as u128) / 10_000;
                let lp_fee = fee.saturating_sub(treasury_fee);

                let amount_after_fee = amount.saturating_sub(fee);

                // For direction: assume (a,b) means a is base_in, b is quote_out
                let amount_out = Self::get_amount_out(amount_after_fee, pair.base_reserve, pair.quote_reserve)?;

                // Oracle-based guard for WUSDC->BBZD hop
                if a == AssetId::WUSDC.as_u8() && b == AssetId::BBZD.as_u8() {
                    if let Some(oracle_rate) = Self::get_verified_exchange_rate(a, b) {
                        if amount > 0 {
                            let implied_rate = amount_out.saturating_mul(1_000_000) / amount;
                            let diff = implied_rate.abs_diff(oracle_rate);
                            if oracle_rate > 0 {
                                let deviation_bps = diff.saturating_mul(10_000) / oracle_rate;
                                ensure!(deviation_bps <= T::MaxOracleDeviationBps::get() as u128, Error::<T>::SlippageExceeded);
                            }
                        }
                    }
                }

                // Update reserves for this hop
                pair.base_reserve = pair.base_reserve
                    .saturating_add(amount_after_fee)
                    .saturating_add(lp_fee);
                pair.quote_reserve = pair.quote_reserve.saturating_sub(amount_out);
                TradingPairs::<T>::insert(pair_key, &pair);

                total_treasury_fee = total_treasury_fee.saturating_add(treasury_fee);
                amount = amount_out;
            }

            ensure!(amount >= min_amount_out, Error::<T>::SlippageExceeded);

            // X-2 FIX: Transfer final output tokens from DEX pool to trader
            let amount_out_balance: <T::Currency as Currency<T::AccountId>>::Balance =
                amount.saturated_into();
            T::Currency::transfer(
                &pool,
                &who,
                amount_out_balance,
                ExistenceRequirement::AllowDeath,
            )?;

            // Transfer aggregated treasury fee from pool to treasury
            if total_treasury_fee > 0 {
                let treasury = T::Treasury::get();
                T::Currency::transfer(
                    &pool,
                    &treasury,
                    // SAFETY(saturated_into): u128 → Balance; total fee ≤ amount_in which originated as Balance.
                    total_treasury_fee.saturated_into(),
                    frame_support::traits::ExistenceRequirement::AllowDeath,
                )?;
            }

            // Emit final event with last pair of path for reference
            let last_pair = (path[path.len()-2], path[path.len()-1]);
            Self::deposit_event(Event::TradeExecuted {
                trader: who,
                pair: last_pair,
                // SAFETY(saturated_into): Balance → u128 is lossless; substrate balances fit within u128.
                amount_in: amount_in.saturated_into(),
                amount_out: amount,
                fee_paid: total_treasury_fee, // only treasury fee reported here
                is_tourism_trade,
            });

            Ok(())
        }

        /// Pause all DEX operations (governance only)
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::pause())]
        pub fn pause_global(origin: OriginFor<T>) -> DispatchResult {
            T::PairListingOrigin::ensure_origin(origin)?;
            GlobalPaused::<T>::put(true);
            Self::deposit_event(Event::DexPaused);
            Ok(())
        }

        /// Resume all DEX operations (governance only)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::resume())]
        pub fn resume_global(origin: OriginFor<T>) -> DispatchResult {
            T::PairListingOrigin::ensure_origin(origin)?;
            GlobalPaused::<T>::put(false);
            Self::deposit_event(Event::DexResumed);
            Ok(())
        }

        /// Set pair active status (governance only)
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::set_pair_status())]
        pub fn set_pair_status(
            origin: OriginFor<T>,
            base_asset: u8,
            quote_asset: u8,
            active: bool,
        ) -> DispatchResult {
            T::PairListingOrigin::ensure_origin(origin)?;
            let key = (base_asset, quote_asset);
            TradingPairs::<T>::try_mutate(key, |maybe_pair| -> DispatchResult {
                let pair = maybe_pair.as_mut().ok_or(Error::<T>::PairNotFound)?;
                pair.active = active;
                Ok(())
            })?;
            Self::deposit_event(Event::PairStatusUpdated { base_asset, quote_asset, active });
            Ok(())
        }

        /// Remove liquidity from a trading pair (X-1 FIX)
        ///
        /// Burns `lp_tokens` from the caller's LP balance and returns proportional
        /// base and quote reserves from the pool.
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::add_liquidity())] // similar weight profile
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            base_asset: u8,
            quote_asset: u8,
            lp_tokens: u128,
            min_base_amount: u128,
            min_quote_amount: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Require KYC
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            ensure!(lp_tokens > 0, Error::<T>::BelowMinimumAmount);

            let pair_key: (u8, u8) = (base_asset, quote_asset);
            let mut pair = Self::trading_pairs(pair_key)
                .ok_or(Error::<T>::PairNotFound)?;

            ensure!(pair.active, Error::<T>::PairNotActive);

            // Check LP balance
            let user_lp = LPBalances::<T>::get(&who, pair_key);
            ensure!(user_lp >= lp_tokens, Error::<T>::InsufficientLPTokens);
            ensure!(pair.total_lp_tokens > 0, Error::<T>::InsufficientLiquidity);

            // Calculate proportional share of reserves
            let base_amount = lp_tokens
                .saturating_mul(pair.base_reserve)
                / pair.total_lp_tokens;
            let quote_amount = lp_tokens
                .saturating_mul(pair.quote_reserve)
                / pair.total_lp_tokens;

            // Slippage protection
            ensure!(base_amount >= min_base_amount, Error::<T>::SlippageExceeded);
            ensure!(quote_amount >= min_quote_amount, Error::<T>::SlippageExceeded);

            // Transfer funds from pool back to provider
            let pool = Self::pool_account();
            let total_return: <T::Currency as Currency<T::AccountId>>::Balance =
                base_amount.saturating_add(quote_amount).saturated_into();
            T::Currency::transfer(
                &pool,
                &who,
                total_return,
                ExistenceRequirement::AllowDeath,
            )?;

            // Update pair reserves and total LP tokens
            pair.base_reserve = pair.base_reserve.saturating_sub(base_amount);
            pair.quote_reserve = pair.quote_reserve.saturating_sub(quote_amount);
            pair.total_lp_tokens = pair.total_lp_tokens.saturating_sub(lp_tokens);
            TradingPairs::<T>::insert(pair_key, &pair);

            // Burn LP tokens from user
            LPBalances::<T>::mutate(&who, pair_key, |balance| {
                *balance = balance.saturating_sub(lp_tokens);
            });

            // Update liquidity provider info
            LiquidityProviders::<T>::mutate(&who, |maybe_provider| {
                if let Some(provider) = maybe_provider.as_mut() {
                    provider.total_value_locked = provider
                        .total_value_locked
                        .saturating_sub(base_amount.saturating_add(quote_amount));
                }
            });

            Self::deposit_event(Event::LiquidityRemoved {
                provider: who,
                pair: (base_asset, quote_asset),
                base_amount,
                quote_amount,
                lp_tokens,
            });

            Ok(())
        }

        /// Cancel an open limit order (H-24 fix: creator-only cancellation)
        ///
        /// Only the original creator may cancel. Removes the order from the
        /// orderbook and emits `OrderCancelled`.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::cancel_order())]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            let order = OrderBook::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;

            // Only the creator may cancel their own order
            ensure!(order.creator == who, Error::<T>::Unauthorized);

            // Remove from orderbook
            OrderBook::<T>::remove(order_id);

            Self::deposit_event(Event::OrderCancelled {
                order_id,
                creator: who,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Derive the DEX pool escrow account from PalletId
        pub fn pool_account() -> T::AccountId {
            T::DexPalletId::get().into_account_truncating()
        }

        /// Calculate output amount for trade using constant product formula
        fn get_amount_out(
            amount_in: u128,
            reserve_in: u128,
            reserve_out: u128,
        ) -> Result<u128, DispatchError> {
            ensure!(amount_in > 0, Error::<T>::BelowMinimumAmount);
            ensure!(reserve_in > 0 && reserve_out > 0, Error::<T>::InsufficientLiquidity);

            let numerator = amount_in.saturating_mul(reserve_out);
            let denominator = reserve_in.saturating_add(amount_in);
            
            Ok(numerator / denominator)
        }

        /// Get current price for a trading pair
        pub fn get_price(base_asset: &AssetId, quote_asset: &AssetId) -> Option<FixedU128> {
            let pair_key: (u8,u8) = (base_asset.as_u8(), quote_asset.as_u8());
            let pair = Self::trading_pairs(pair_key)?;
            
            if pair.base_reserve == 0 {
                return None;
            }
            
            Some(FixedU128::from_rational(pair.quote_reserve, pair.base_reserve))
        }

        /// Get Oracle-verified exchange rate for crypto pairs (with on-chain fallback)
        pub fn get_verified_exchange_rate(base_asset: u8, quote_asset: u8) -> Option<u128> {
            // Try Oracle first for real-time rates
            if let Some(rate) = T::Oracle::get_crypto_exchange_rate(base_asset, quote_asset) {
                return Some(rate);
            }
            
            // Fallback to on-chain AMM pricing
            let pair_key: (u8, u8) = (base_asset, quote_asset);
            if let Some(pair) = Self::trading_pairs(pair_key) {
                if pair.base_reserve > 0 {
                    // Return rate scaled to 10^6 precision (matching Oracle format)
                    return Some(pair.quote_reserve.saturating_mul(1_000_000) / pair.base_reserve);
                }
            }
            
            None
        }

        /// Check if trader is tourism merchant (Oracle primary, on-chain fallback)
        pub fn is_verified_tourism_merchant(account: &T::AccountId) -> bool {
            // Check Oracle first for real-time merchant verification
            if T::Oracle::is_tourism_merchant(account) {
                return true;
            }
            
            // Fallback to on-chain registration
            Self::tourism_traders(account)
        }

        /// X-8 FIX: Get total LP balance across all pairs for an account
        pub fn get_lp_balance(account: &T::AccountId) -> u128 {
            LPBalances::<T>::iter_prefix(account)
                .fold(0u128, |acc, (_, bal)| acc.saturating_add(bal))
        }

        /// Get effective trading fee with volume tier discount
        pub fn get_effective_fee_rate(account: &T::AccountId, is_tourism: bool) -> u32 {
            let base_fee = T::TradingFeeRate::get(); // e.g., 30 bps (0.3%)
            
            // Apply volume tier discount (0-3 tiers, each tier = 2 bps discount)
            let volume_tier = T::Oracle::get_trading_volume_tier(account);
            let volume_discount = volume_tier.min(3).saturating_mul(2); // Max 6 bps discount
            
            // Apply tourism discount if eligible
            let tourism_discount = if is_tourism {
                T::TourismDiscountRate::get() // e.g., 5 bps (0.05%)
            } else {
                0
            };
            
            // Total fee = base - volume_discount - tourism_discount (min 10 bps floor)
            base_fee
                .saturating_sub(volume_discount as u32)
                .saturating_sub(tourism_discount)
                .max(10)
        }

                /// Public helper: get pair reserves and LP supply as u128
                pub fn get_pair_reserves_u128(base_asset: u8, quote_asset: u8) -> Option<(u128, u128, u128)> {
                    let pair_key = (base_asset, quote_asset);
                    let pair = Self::trading_pairs(pair_key)?;
                    Some((pair.base_reserve, pair.quote_reserve, pair.total_lp_tokens))
                }

                /// Public helper: implied price scaled by 1e6 (quote/base)
                pub fn get_implied_price_scaled_1e6(base_asset: u8, quote_asset: u8) -> Option<u128> {
                    let base = AssetId::from(base_asset);
                    let quote = AssetId::from(quote_asset);
                    let price = Self::get_price(&base, &quote)?;
                    // Convert FixedU128 to scaled u128 with 1e6 precision
                    // price = quote/base; scaled = floor(price * 1e6)
                    let scaled = price.saturating_mul_int(1_000_000u128);
                    Some(scaled)
                }
    }
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn create_pair() -> Weight;
    fn add_liquidity() -> Weight;
    fn execute_trade() -> Weight;
    fn register_tourism_trader() -> Weight;
    fn place_order() -> Weight;
    fn cancel_order() -> Weight;
    fn pause() -> Weight;
    fn resume() -> Weight;
    fn set_pair_status() -> Weight;
    fn remove_liquidity() -> Weight;
}

impl WeightInfo for () {
    fn create_pair() -> Weight {
        Weight::from_parts(15_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn add_liquidity() -> Weight {
        Weight::from_parts(30_000_000, 3584)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn execute_trade() -> Weight {
        Weight::from_parts(25_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn register_tourism_trader() -> Weight {
        Weight::from_parts(10_000_000, 1536)
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn place_order() -> Weight {
        Weight::from_parts(20_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn cancel_order() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn pause() -> Weight {
        Weight::from_parts(5_000_000, 512)
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn resume() -> Weight {
        Weight::from_parts(5_000_000, 512)
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn set_pair_status() -> Weight {
        Weight::from_parts(10_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn remove_liquidity() -> Weight {
        Weight::from_parts(45_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }
}

// Test modules
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;