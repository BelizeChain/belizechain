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
    traits::{Currency, Get, LockableCurrency, Randomness, ReservableCurrency},
    weights::{constants::RocksDbWeight, Weight},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{AccountIdConversion, IntegerSquareRoot, SaturatedConversion, Saturating},
    FixedPointNumber, FixedU128,
};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_std::vec::Vec;
// use sp_std::collections::btree_map::BTreeMap; // No per-pair LP token map stored currently

pub use pallet::*;

/// Fixed-point scale for a limit order's `price`.
///
/// `price` is *quote units per base unit*, multiplied by this constant, so a
/// price of 1.5 quote-per-base is stored as `1_500_000`. The same 1e6 precision
/// is already the convention in this pallet for oracle rates (see the
/// `implied_rate * 1_000_000` guard in `execute_trade`), so no new convention is
/// introduced.
pub const PRICE_SCALE: u128 = 1_000_000;

/// One side of a pair's order book: `(order_id, order)` pairs, best price first.
///
/// Named so `open_orders`'s signature stays readable and does not trip
/// `clippy::type_complexity` under CI's `-D warnings`.
pub type OrderList<T> = Vec<(
    u32,
    OrderEntry<
        <T as frame_system::Config>::AccountId,
        frame_system::pallet_prelude::BlockNumberFor<T>,
    >,
)>;

/// Per-asset ledger behind the `AssetId` labels (P1-08 root cause).
///
/// Before this trait, `belizex` settled **every** asset through `T::Currency`,
/// i.e. the native DALLA balance. `AssetId` was a label with no ledger behind
/// it, so a "DALLA/bBZD" trade moved DALLA twice and `remove_liquidity` paid
/// `base_amount + quote_amount` as one native transfer. bBZD lives in
/// `pallet-belize-economy`'s `BBZDBalances`, which this pallet had no dependency
/// on and could not reach.
///
/// The runtime implements this by delegating DALLA to `Balances` and bBZD to the
/// economy pallet, so each asset settles against its real ledger.
///
/// `TourismDALLA` is a fee/discount flag rather than a separate balance and maps
/// to the DALLA ledger. `WUSDC` has no ledger in this runtime; implementations
/// must reject it rather than inventing a balance.
pub trait AssetLedger<AccountId> {
    /// Spendable balance of `asset` held by `who`.
    fn balance_of(asset: AssetId, who: &AccountId) -> u128;

    /// Move `amount` of `asset` from `from` to `to`.
    ///
    /// Must fail rather than create value. Implementations are responsible for
    /// whatever per-asset checks they need (sanctions, existence requirements);
    /// the pallet has already checked KYC and pause state before calling.
    fn transfer(asset: AssetId, from: &AccountId, to: &AccountId, amount: u128) -> DispatchResult;

    /// Benchmark-only: set `who`'s balance of `asset` to `amount`.
    ///
    /// Needed because the assets in a pair do not share a ledger — a
    /// DALLA/bBZD benchmark must fund `Balances` *and*
    /// `pallet_belize_economy::BBZDBalances`, and this pallet deliberately has
    /// no dependency on `pallet-belize-economy`. Without this hook a benchmark
    /// can only ever fund the base leg, so `add_liquidity`, `execute_trade`
    /// and `remove_liquidity` fail on the quote leg against the real runtime
    /// while passing against any mock that maps every asset to one balance.
    #[cfg(feature = "runtime-benchmarks")]
    fn set_balance(asset: AssetId, who: &AccountId, amount: u128);
}

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Storage migrations (P1-08 escrow purge).
pub mod migrations;

// LIQUIDITY_LOCK_ID removed: custody model now uses transfer-to-pool instead of locks

/// Minimal trait to check KYC status of an account.
pub trait KycCheck<AccountId> {
    fn is_kyc_ok(who: &AccountId) -> bool;
    /// AUDIT FIX (CW-1): Standalone sanctions check for operations that don't
    /// require full KYC (e.g. governance-registered tourism traders).
    fn is_sanctioned(who: &AccountId) -> bool;
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

    /// On-chain storage version for this pallet.
    ///
    /// Bump this and register a migration in the runtime's `Migrations` tuple
    /// whenever this pallet's storage layout changes.
    ///
    /// v1 — `OrderEntry` gained `escrow`. Orders placed before v1 hold no
    /// escrowed funds, so they must be purged rather than matched; the
    /// migration does exactly that (the book was empty on the live chain, so
    /// nothing was lost).
    pub const STORAGE_VERSION: frame_support::traits::StorageVersion =
        frame_support::traits::StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// The currency used for DEX operations
        type Currency: Currency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + LockableCurrency<Self::AccountId>;

        /// Per-asset ledger used to settle trades and liquidity (P1-08).
        ///
        /// `Currency` above remains the DALLA ledger and is still used for
        /// deposits and LP accounting. Settlement goes through this trait so
        /// bBZD moves against its real balance instead of being simulated as
        /// native currency.
        type Assets: AssetLedger<Self::AccountId>;

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

        /// Benchmark-only: write real KYC state for `account` so the KYC gate
        /// runs its production path instead of a shim that answers "verified"
        /// without touching storage.
        #[cfg(feature = "runtime-benchmarks")]
        fn seed_bench_kyc(_account: &Self::AccountId, _level: u8) {}

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

        /// AUDIT FIX: Maximum number of open limit orders per account.
        /// Prevents a single account from flooding the order book.
        #[pallet::constant]
        type MaxOrdersPerAccount: Get<u32>;

        /// Maximum open orders of one side in a single pair.
        ///
        /// Bounds the per-pair order index so a future matching loop can iterate
        /// a bounded list. Also the cap that makes the index's `BoundedVec`
        /// infallible to size.
        #[pallet::constant]
        type MaxOrdersPerPair: Get<u32>;

        /// Pallet ID used to derive the DEX pool escrow account
        #[pallet::constant]
        type DexPalletId: Get<PalletId>;

        /// Pallet ID used to derive the limit-order escrow account.
        ///
        /// Every resting order's funds live here, not in the pool account: the
        /// pool balance is part of the AMM price curve and would be corrupted by
        /// escrow sitting in it.
        #[pallet::constant]
        type OrderEscrowPalletId: Get<PalletId>;
    }

    // X-9 FIX: Removed dead LiquidityPosition struct (unused scaffolding).
    // Active LP tracking uses LiquidityProvider and LPBalances storage.

    /// Trading order information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
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
    #[derive(
        Encode, Decode, Copy, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Ord, PartialOrd,
    )]
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct OrderEntry<AccountId, BlockNumber> {
        /// Order ID
        pub order_id: u32,
        /// Order creator
        pub creator: AccountId,
        /// Trading pair
        pub pair: (AssetId, AssetId),
        /// Order type (buy/sell)
        pub order_type: OrderType,
        /// Amount to trade, in base units
        pub amount: u128,
        /// Price per base unit in quote units, scaled by `PRICE_SCALE`
        pub price: u128,
        /// Remaining amount, in base units
        pub remaining: u128,
        /// Funds still held for this order.
        ///
        /// A bid escrows the quote asset (it is buying base); an ask escrows the
        /// base asset. Decremented as the order fills, and whatever is left is
        /// refunded when the order leaves the book — so the escrow account never
        /// accumulates stranded dust.
        pub escrow: u128,
        /// Block when order expires
        pub expires_at: BlockNumber,
        /// Tourism order flag
        pub is_tourism_order: bool,
    }

    /// Open order IDs for one pair, split by side.
    ///
    /// Bounded by `T::MaxOrdersPerPair` per side — the bound is the config item,
    /// not a copy of it. `ValueQuery` on the pallet storage means an absent pair
    /// reads as two empty vectors, so no `Option` handling is needed at call
    /// sites.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    // `MaxSize` only parameterises the `BoundedVec` bounds; deriving `TypeInfo`
    // would otherwise demand `MaxSize: TypeInfo`, which `ConstU32` does not
    // promise. Skipping it keeps the encoded shape identical.
    #[scale_info(skip_type_params(MaxSize))]
    pub struct PairOrderIndex<MaxSize: Get<u32>> {
        /// Buy-side order IDs (`Buy`, `TourismBuy`)
        pub bids: BoundedVec<u32, MaxSize>,
        /// Sell-side order IDs (`Sell`, `TourismSell`)
        pub asks: BoundedVec<u32, MaxSize>,
    }

    // Written out because `derive(Default)` would infer `MaxSize: Default`, which
    // `ConstU32` does not implement. An absent pair must read as two empty books.
    impl<MaxSize: Get<u32>> Default for PairOrderIndex<MaxSize> {
        fn default() -> Self {
            Self {
                bids: BoundedVec::new(),
                asks: BoundedVec::new(),
            }
        }
    }

    /// The amounts for one fill, computed before any state is committed.
    ///
    /// Planning and applying are separate steps so a ledger rejection can abort a
    /// fill with the book untouched. Decrementing `remaining`/`escrow` first and
    /// moving funds second would leave both orders claiming a fill that never
    /// happened, with the funds stranded in the escrow account.
    struct FillAmounts {
        /// Base moved from the asker to the bidder.
        base_amount: u128,
        /// Quote moved from the bidder to the asker.
        quote_amount: u128,
        /// What the taker gives up, in the asset it escrowed.
        taker_pays: u128,
        /// What the maker gives up, in the asset it escrowed.
        maker_pays: u128,
    }

    /// Order types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
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

    impl<AccountId, BlockNumber> OrderEntry<AccountId, BlockNumber> {
        /// A bid buys base with quote; an ask sells base for quote.
        pub fn is_bid(&self) -> bool {
            matches!(self.order_type, OrderType::Buy | OrderType::TourismBuy)
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
    pub type TradingPairs<T: Config> = StorageMap<_, Blake2_128Concat, (u8, u8), TradingPair>;

    #[pallet::storage]
    #[pallet::getter(fn liquidity_providers)]
    /// Liquidity provider information
    pub type LiquidityProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, LiquidityProvider<T::AccountId>>;

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
    #[pallet::getter(fn pair_orders)]
    /// Open order IDs per pair, split by side (P1-08 prerequisite).
    ///
    /// `OrderBook` is keyed by `order_id` alone, so finding the orders for a pair
    /// meant iterating every order on the chain — O(all orders) per lookup, a DoS
    /// vector for anything that runs inside a block. This index makes it one
    /// read per pair.
    ///
    /// Deliberately NOT sorted by price: maintaining price order on-chain means
    /// shifting the `BoundedVec` on every insert, which is O(n) writes per
    /// placement. Consumers sort the (bounded) list in memory instead.
    ///
    /// Stale entries are possible if an order is removed without touching the
    /// index; readers MUST treat a missing `OrderBook` entry as "skip", which
    /// `open_orders` below does.
    pub type PairOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (u8, u8), // (base_asset, quote_asset)
        PairOrderIndex<T::MaxOrdersPerPair>,
        ValueQuery,
    >;

    /// AUDIT FIX: Per-account open order counter to enforce MaxOrdersPerAccount.
    #[pallet::storage]
    #[pallet::getter(fn account_order_count)]
    pub type AccountOrderCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    // DailyVolume removed (E-7): write-only accumulator, never read on-chain.

    #[pallet::storage]
    #[pallet::getter(fn tourism_traders)]
    /// Tourism-verified traders with benefits
    pub type TourismTraders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

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
                TradingPairs::<T>::insert(
                    (AssetId::DALLA.as_u8(), AssetId::BBZD.as_u8()),
                    dalla_bbzd_pair,
                );

                let tourism_bbzd_pair = TradingPair {
                    base_asset: AssetId::TourismDALLA,
                    quote_asset: AssetId::BBZD,
                    base_reserve: 0,
                    quote_reserve: 0,
                    total_lp_tokens: 0,
                    fee_rate: 15, // 0.15% for tourism
                    active: true,
                };
                TradingPairs::<T>::insert(
                    (AssetId::TourismDALLA.as_u8(), AssetId::BBZD.as_u8()),
                    tourism_bbzd_pair,
                );

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
                TradingPairs::<T>::insert(
                    (AssetId::WUSDC.as_u8(), AssetId::BBZD.as_u8()),
                    wusdc_bbzd_pair,
                );
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
                LiquidityProviders::<T>::mutate(&treasury, |maybe_provider| match maybe_provider {
                    Some(provider) => {
                        provider.total_value_locked = provider
                            .total_value_locked
                            .saturating_add(base_amount.saturating_add(quote_amount));
                    }
                    None => {
                        *maybe_provider = Some(LiquidityProvider {
                            provider: treasury.clone(),
                            total_value_locked: base_amount.saturating_add(quote_amount),
                            rewards_earned: 0,
                            is_tourism_provider: false,
                        });
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
            // DOS-008 FIX: Return actual consumed weight instead of zero.
            // Operations: TradingPairs read+write, LiquidityProviders mutate, DevSeedDone write, event deposit.
            T::DbWeight::get()
                .reads(2)
                .saturating_add(T::DbWeight::get().writes(3))
                .saturating_add(Weight::from_parts(10_000_000, 1024))
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Trading pair created
        TradingPairCreated {
            base_asset: u8,  // AssetId as u8
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
        OracleRateUnavailable { pair: (u8, u8) },
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
        /// Order cancelled by creator
        OrderCancelled {
            order_id: u32,
            creator: T::AccountId,
        },
        /// A resting order was matched against an incoming order.
        ///
        /// `price` is the **maker's** price — the resting order gets the price it
        /// asked for, which is what price-time priority means.
        OrderFilled {
            maker_order_id: u32,
            taker_order_id: u32,
            maker: T::AccountId,
            taker: T::AccountId,
            base_amount: u128,
            quote_amount: u128,
            price: u128,
        },
        /// An expired resting order was purged and its escrow refunded.
        OrderExpired {
            order_id: u32,
            creator: T::AccountId,
        },
        /// Tourism trader verified
        TourismTraderVerified { trader: T::AccountId },
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
        /// Multi-hop path was empty, so no input or output asset is defined
        InvalidPath,
        /// This asset has no ledger in the runtime (e.g. WUSDC today)
        UnsupportedAsset,
        /// The pair's open-order book for this side is full
        TooManyOrdersForPair,
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
        /// Arithmetic overflow in financial calculation
        ArithmeticOverflow,
        /// Order ID space exhausted
        OrderIdOverflow,
        /// AUDIT FIX: Account has too many open orders
        TooManyOrders,
        /// AUDIT FIX: Cannot trade with yourself
        SelfTradeNotAllowed,
        /// AUDIT FIX (CW-1): Account is sanctioned
        AccountSanctioned,
        /// (P1-08) The order's escrow is smaller than the amount it would pay.
        ///
        /// Defensive: escrow is computed and decremented by the same helper, so
        /// this can only fire if storage was written by something other than
        /// this pallet. Failing closed beats paying out funds that were never
        /// held.
        EscrowUnderflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create new trading pair
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_pair())]
        pub fn create_trading_pair(
            origin: OriginFor<T>,
            base_asset: u8,  // AssetId as u8
            quote_asset: u8, // AssetId as u8
            fee_rate: u32,
        ) -> DispatchResult {
            // Only governance may list new pairs
            T::PairListingOrigin::ensure_origin(origin)?;

            let pair_key: (u8, u8) = (base_asset, quote_asset);
            ensure!(
                !TradingPairs::<T>::contains_key(pair_key),
                Error::<T>::PairAlreadyExists
            );

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

            Self::deposit_event(Event::TradingPairCreated {
                base_asset,
                quote_asset,
            });
            Ok(())
        }

        /// Add liquidity to trading pair
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::add_liquidity())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            base_asset: u8,  // AssetId as u8
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

            let pair_key: (u8, u8) = (base_asset, quote_asset);
            let mut pair = Self::trading_pairs(pair_key).ok_or(Error::<T>::PairNotFound)?;

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
                let product = U256::from(base_amount_u128)
                    .checked_mul(U256::from(quote_amount_u128))
                    .ok_or(Error::<T>::ArithmeticOverflow)?;
                let lp_u256 = product.integer_sqrt();
                let lp: u128 = lp_u256
                    .try_into()
                    .map_err(|_| Error::<T>::ArithmeticOverflow)?;
                lp
            } else {
                // Subsequent deposits: min(base/base_reserve, quote/quote_reserve) * total_lp
                // This preserves pool ratio and prevents LP value dilution
                let lp_from_base: u128 = U256::from(base_amount_u128)
                    .checked_mul(U256::from(pair.total_lp_tokens))
                    .ok_or(Error::<T>::ArithmeticOverflow)?
                    .checked_div(U256::from(pair.base_reserve.max(1)))
                    .ok_or(Error::<T>::ArithmeticOverflow)?
                    .try_into()
                    .map_err(|_| Error::<T>::ArithmeticOverflow)?;
                let lp_from_quote: u128 = U256::from(quote_amount_u128)
                    .checked_mul(U256::from(pair.total_lp_tokens))
                    .ok_or(Error::<T>::ArithmeticOverflow)?
                    .checked_div(U256::from(pair.quote_reserve.max(1)))
                    .ok_or(Error::<T>::ArithmeticOverflow)?
                    .try_into()
                    .map_err(|_| Error::<T>::ArithmeticOverflow)?;
                lp_from_base.min(lp_from_quote)
            };

            let min_lp_tokens_u128: u128 = min_lp_tokens;
            ensure!(
                lp_tokens >= min_lp_tokens_u128,
                Error::<T>::SlippageExceeded
            );

            // Transfer funds to the DEX pool escrow account.
            //
            // P1-08: these were previously summed into a single native transfer,
            // which only balanced because every asset was DALLA. Each side now
            // moves against its own ledger.
            let pool = Self::pool_account();
            let base_id = AssetId::from(base_asset);
            let quote_id = AssetId::from(quote_asset);
            T::Assets::transfer(base_id, &who, &pool, base_amount.saturated_into())?;
            T::Assets::transfer(quote_id, &who, &pool, quote_amount.saturated_into())?;

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
            LiquidityProviders::<T>::mutate(&who, |maybe_provider| match maybe_provider {
                Some(provider) => {
                    provider.total_value_locked = provider
                        .total_value_locked
                        .saturating_add(base_amount_u128.saturating_add(quote_amount_u128));
                }
                None => {
                    *maybe_provider = Some(LiquidityProvider {
                        provider: who.clone(),
                        total_value_locked: base_amount_u128.saturating_add(quote_amount_u128),
                        rewards_earned: 0,
                        is_tourism_provider: Self::tourism_traders(&who),
                    });
                }
            });

            Self::deposit_event(Event::LiquidityAdded {
                provider: who.clone(),
                pair: (base_asset, quote_asset),
                base_amount: base_amount_u128,
                quote_amount: quote_amount_u128,
                lp_tokens,
            });
            Ok(())
        }

        /// Execute market trade
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::execute_trade())]
        pub fn execute_trade(
            origin: OriginFor<T>,
            base_asset: u8,  // AssetId as u8
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

            let pair_key: (u8, u8) = (base_asset, quote_asset);
            let mut pair = Self::trading_pairs(pair_key).ok_or(Error::<T>::PairNotFound)?;

            ensure!(pair.active, Error::<T>::PairNotActive);

            // Verify tourism trade authorization
            if is_tourism_trade {
                ensure!(Self::tourism_traders(&who), Error::<T>::Unauthorized);
            }

            // SAFETY(saturated_into): Balance → u128 is lossless; substrate balances fit within u128.
            let amount_in_u128: u128 = amount_in.saturated_into();

            // Calculate dynamic fee with Oracle-enhanced volume tiers and tourism verification
            let is_verified_tourism = Self::is_verified_tourism_merchant(&who);
            let effective_fee_rate =
                Self::get_effective_fee_rate(&who, is_verified_tourism && is_tourism_trade);

            let fee = amount_in_u128.saturating_mul(effective_fee_rate as u128) / 10_000;
            let treasury_fee_rate =
                core::cmp::min(T::ProtocolFeeToTreasuryBps::get(), effective_fee_rate);
            let treasury_fee = amount_in_u128.saturating_mul(treasury_fee_rate as u128) / 10_000;
            let lp_fee = fee.saturating_sub(treasury_fee);
            // Amount used in swap formula (after total fee)
            let amount_after_fee = amount_in_u128.saturating_sub(fee);

            // Calculate output amount using constant product formula
            let amount_out =
                Self::get_amount_out(amount_after_fee, pair.base_reserve, pair.quote_reserve)?;

            // Oracle-based slippage guard for WUSDC/BBZD trades only
            // Compare implied rate vs Oracle-verified USD/BZD rate (both scaled by 1e6 precision)
            if base_asset == AssetId::WUSDC.as_u8() && quote_asset == AssetId::BBZD.as_u8() {
                // Require a fresh Oracle rate; if missing/stale, reject
                if let Some(oracle_rate) =
                    T::Oracle::get_crypto_exchange_rate(base_asset, quote_asset)
                {
                    let implied_rate = amount_out
                        .saturating_mul(1_000_000)
                        .checked_div(amount_in_u128)
                        .unwrap_or(0);
                    if oracle_rate > 0 {
                        let diff = implied_rate.abs_diff(oracle_rate);
                        let deviation_bps = diff
                            .saturating_mul(10_000)
                            .checked_div(oracle_rate)
                            .unwrap_or(0);
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
                    Self::deposit_event(Event::OracleRateUnavailable {
                        pair: (base_asset, quote_asset),
                    });
                    return Err(Error::<T>::OracleRateUnavailable.into());
                }
            }

            ensure!(amount_out >= min_amount_out, Error::<T>::SlippageExceeded);

            // P1-08: settle each leg in its own asset. `amount_in` is the base
            // asset (it is priced against `base_reserve`) and `amount_out` is the
            // quote asset.
            let pool = Self::pool_account();
            let base_id = AssetId::from(base_asset);
            let quote_id = AssetId::from(quote_asset);
            T::Assets::transfer(base_id, &who, &pool, amount_in_u128)?;
            T::Assets::transfer(quote_id, &pool, &who, amount_out)?;

            // Update reserves: LP fee is retained in pool
            pair.base_reserve = pair
                .base_reserve
                .saturating_add(amount_after_fee)
                .saturating_add(lp_fee);
            pair.quote_reserve = pair.quote_reserve.saturating_sub(amount_out);

            TradingPairs::<T>::insert(pair_key, pair);

            // Transfer treasury portion of fee from pool to treasury. The fee is
            // a fraction of `amount_in`, so it is denominated in the base asset.
            let treasury = T::Treasury::get();
            T::Assets::transfer(base_id, &pool, &treasury, treasury_fee)?;

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

            // AUDIT FIX (CW-1): Cannot register a sanctioned account as tourism trader
            ensure!(
                !T::Kyc::is_sanctioned(&trader),
                Error::<T>::AccountSanctioned
            );

            TourismTraders::<T>::insert(&trader, true);

            Self::deposit_event(Event::TourismTraderVerified { trader });

            Ok(())
        }

        /// Place a limit order and match it against the opposite side of the book.
        ///
        /// **Escrow.** Funds move at placement, not at fill:
        ///
        /// - a **bid** (`Buy`, `TourismBuy`) buys base, so it escrows
        ///   `ceil(amount * price / PRICE_SCALE)` of the **quote** asset;
        /// - an **ask** (`Sell`, `TourismSell`) sells base, so it escrows
        ///   `amount` of the **base** asset.
        ///
        /// Escrow sits in a dedicated account derived from
        /// `T::OrderEscrowPalletId` — never the pool account, whose balance is
        /// part of the AMM price curve. Whatever escrow remains when the order
        /// leaves the book (cancelled, expired, or filled) is refunded to the
        /// creator, so rounding dust cannot accumulate.
        ///
        /// **Matching.** After the order is indexed it walks the opposite side of
        /// `PairOrders`, best price first, and crosses every order whose limit is
        /// met. The **resting** order gets its own price; a bidder that crosses at
        /// a lower ask keeps the difference until its order leaves the book.
        /// Partial fills decrement `remaining` on both sides. Orders from the same
        /// account are skipped rather than matched, so a maker and taker can never
        /// be the same account.
        ///
        /// The walk is bounded by `T::MaxOrdersPerPair`, which is also the maximum
        /// number of fills one placement can produce (each fill either exhausts the
        /// maker or the taker, and exhausting the taker ends the loop).
        ///
        /// Market trades (`execute_trade`) still swap against the AMM pool only and
        /// do not cross the book — that is a separate change with its own weight.
        ///
        /// **Weight.** `match_orders` is measured, but it measures the *fill*
        /// shape. A walk iteration is either a fill or the purge of an expired
        /// order, and `open_orders` returns expired orders so the matcher purges
        /// them — a walk can be entirely purges. Neither shape dominates: a fill
        /// costs more time (two settlement legs) while a purge costs more writes
        /// (`remove_order` plus a refund). Both are therefore charged per
        /// iteration. The purge term lives here rather than in `weights.rs`
        /// because that file is regenerated by `scripts/bench_weights.sh` and a
        /// hand-added term there would be silently lost on the next run.
        #[pallet::call_index(4)]
        #[pallet::weight(
            T::WeightInfo::place_order()
                .saturating_add(T::WeightInfo::match_orders(T::MaxOrdersPerPair::get()))
                .saturating_add(
                    T::WeightInfo::cancel_order()
                        .saturating_mul(T::MaxOrdersPerPair::get().into())
                )
        )]
        pub fn place_limit_order(
            origin: OriginFor<T>,
            base_asset: u8,  // AssetId as u8
            quote_asset: u8, // AssetId as u8
            order_type: u8,  // OrderType as u8
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
            let pair_key: (u8, u8) = (base_asset, quote_asset);
            ensure!(
                TradingPairs::<T>::contains_key(pair_key),
                Error::<T>::PairNotFound
            );

            ensure!(price > 0, Error::<T>::InvalidPrice);
            ensure!(amount > 0, Error::<T>::BelowMinimumAmount);

            // AUDIT FIX: Enforce per-account order limit
            let current_count = AccountOrderCount::<T>::get(&who);
            ensure!(
                current_count < T::MaxOrdersPerAccount::get(),
                Error::<T>::TooManyOrders
            );

            let order_id = Self::next_order_id();
            ensure!(order_id < u32::MAX, Error::<T>::OrderIdOverflow);
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0);
            let expires_in_u64: u64 = TryInto::<u64>::try_into(expires_in_blocks).unwrap_or(0);
            let expires_u64 = current_u64.saturating_add(expires_in_u64);
            // SAFETY(saturated_into): u64 → BlockNumber. If the computed expiry exceeds
            // BlockNumber::MAX it saturates rather than wrapping — order simply expires
            // at the maximum possible block.
            let expires_at: BlockNumberFor<T> = expires_u64.saturated_into();

            let is_tourism_order = matches!(
                order_type_enum,
                OrderType::TourismBuy | OrderType::TourismSell
            );
            if is_tourism_order {
                ensure!(Self::tourism_traders(&who), Error::<T>::Unauthorized);
            }

            let is_bid = matches!(order_type_enum, OrderType::Buy | OrderType::TourismBuy);

            // Escrow before the order exists in storage: a failed transfer must not
            // leave a resting order that owes funds it never received.
            let (escrow_asset, escrow) = if is_bid {
                // Round up, so the order can always pay its own limit price.
                (quote_asset_id, Self::quote_for(amount, price, true)?)
            } else {
                (base_asset_id, amount)
            };
            let escrow_account = Self::order_escrow_account();
            T::Assets::transfer(escrow_asset, &who, &escrow_account, escrow)?;

            let order = OrderEntry {
                order_id,
                creator: who.clone(),
                pair: (base_asset_id, quote_asset_id),
                order_type: order_type_enum.clone(),
                amount,
                price,
                remaining: amount,
                escrow,
                expires_at,
                is_tourism_order,
            };

            OrderBook::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id.saturating_add(1));
            AccountOrderCount::<T>::mutate(&who, |c| *c = c.saturating_add(1));

            // Index by pair and side so the matching pass reads one entry instead
            // of scanning `OrderBook`. Fails closed at the per-pair cap rather
            // than leaving an order in `OrderBook` that no index can reach.
            PairOrders::<T>::try_mutate(pair_key, |index| -> DispatchResult {
                let side = if is_bid {
                    &mut index.bids
                } else {
                    &mut index.asks
                };
                side.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrdersForPair.into())
            })?;

            Self::deposit_event(Event::OrderPlaced {
                order_id,
                creator: who,
                pair: (base_asset_id.as_u8(), quote_asset_id.as_u8()),
                order_type: order_type_enum.as_u8(),
                amount,
                price,
            });

            Self::match_order(pair_key, order_id);

            Ok(())
        }

        /// Execute a multihop market trade along a path of AssetIds (as u8)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::execute_trade().saturating_mul(path.len().saturating_sub(1) as u64))]
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

            // X-2 FIX: Transfer input tokens from trader to DEX pool before
            // executing hops. P1-08: denominated in the first asset of the path.
            let pool = Self::pool_account();
            let first_asset = AssetId::from(*path.first().ok_or(Error::<T>::InvalidPath)?);
            T::Assets::transfer(first_asset, &who, &pool, amount_in.saturated_into())?;

            for w in path.windows(2) {
                let a = w[0];
                let b = w[1];
                let pair_key = (a, b);
                let mut pair = Self::trading_pairs(pair_key).ok_or(Error::<T>::PairNotFound)?;
                ensure!(pair.active, Error::<T>::PairNotActive);

                // H-3 FIX: Use get_effective_fee_rate() with .max(10) floor,
                // matching execute_trade fee logic (volume tiers + tourism + min floor)
                let is_verified_tourism = Self::is_verified_tourism_merchant(&who);
                let effective_fee_rate =
                    Self::get_effective_fee_rate(&who, is_verified_tourism && is_tourism_trade);
                let fee = amount.saturating_mul(effective_fee_rate as u128) / 10_000;
                let treasury_fee_rate =
                    core::cmp::min(T::ProtocolFeeToTreasuryBps::get(), effective_fee_rate);
                let treasury_fee = amount.saturating_mul(treasury_fee_rate as u128) / 10_000;
                let lp_fee = fee.saturating_sub(treasury_fee);

                let amount_after_fee = amount.saturating_sub(fee);

                // For direction: assume (a,b) means a is base_in, b is quote_out
                let amount_out =
                    Self::get_amount_out(amount_after_fee, pair.base_reserve, pair.quote_reserve)?;

                // H-4 FIX: Oracle guard for WUSDC->BBZD hop — use hard-reject
                // matching execute_trade logic (reject if Oracle unavailable)
                if a == AssetId::WUSDC.as_u8() && b == AssetId::BBZD.as_u8() {
                    if let Some(oracle_rate) = T::Oracle::get_crypto_exchange_rate(a, b) {
                        if amount > 0 {
                            let implied_rate = amount_out
                                .saturating_mul(1_000_000)
                                .checked_div(amount)
                                .unwrap_or(0);
                            let diff = implied_rate.abs_diff(oracle_rate);
                            if oracle_rate > 0 {
                                let deviation_bps = diff
                                    .saturating_mul(10_000)
                                    .checked_div(oracle_rate)
                                    .unwrap_or(0);
                                let max_allowed = T::MaxOracleDeviationBps::get() as u128;
                                if deviation_bps > max_allowed {
                                    Self::deposit_event(Event::OracleGuardRejected {
                                        pair: (a, b),
                                        implied_rate,
                                        oracle_rate,
                                        deviation_bps,
                                        max_allowed_bps: T::MaxOracleDeviationBps::get(),
                                    });
                                    return Err(Error::<T>::SlippageExceeded.into());
                                }
                            }
                        }
                    } else {
                        // No Oracle rate available → hard-reject (same as execute_trade)
                        Self::deposit_event(Event::OracleRateUnavailable { pair: (a, b) });
                        return Err(Error::<T>::OracleRateUnavailable.into());
                    }
                }

                // Update reserves for this hop
                pair.base_reserve = pair
                    .base_reserve
                    .saturating_add(amount_after_fee)
                    .saturating_add(lp_fee);
                pair.quote_reserve = pair.quote_reserve.saturating_sub(amount_out);
                TradingPairs::<T>::insert(pair_key, &pair);

                total_treasury_fee = total_treasury_fee.saturating_add(treasury_fee);
                amount = amount_out;
            }

            ensure!(amount >= min_amount_out, Error::<T>::SlippageExceeded);

            // X-2 FIX: Transfer final output tokens from DEX pool to trader
            // P1-08: output is denominated in the last asset of the path.
            let last_asset = AssetId::from(*path.last().ok_or(Error::<T>::InvalidPath)?);
            T::Assets::transfer(last_asset, &pool, &who, amount)?;

            // Transfer aggregated treasury fee from pool to treasury. Fees were
            // taken from the input leg, so they are in the first asset.
            if total_treasury_fee > 0 {
                let treasury = T::Treasury::get();
                T::Assets::transfer(first_asset, &pool, &treasury, total_treasury_fee)?;
            }

            // Emit final event with last pair of path for reference
            let last_pair = (path[path.len() - 2], path[path.len() - 1]);
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
            Self::deposit_event(Event::PairStatusUpdated {
                base_asset,
                quote_asset,
                active,
            });
            Ok(())
        }

        /// Remove liquidity from a trading pair (X-1 FIX)
        ///
        /// Burns `lp_tokens` from the caller's LP balance and returns proportional
        /// base and quote reserves from the pool.
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::remove_liquidity())]
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
            let mut pair = Self::trading_pairs(pair_key).ok_or(Error::<T>::PairNotFound)?;

            ensure!(pair.active, Error::<T>::PairNotActive);

            // Check LP balance
            let user_lp = LPBalances::<T>::get(&who, pair_key);
            ensure!(user_lp >= lp_tokens, Error::<T>::InsufficientLPTokens);
            ensure!(pair.total_lp_tokens > 0, Error::<T>::InsufficientLiquidity);

            // Calculate proportional share of reserves
            let base_amount: u128 = U256::from(lp_tokens)
                .checked_mul(U256::from(pair.base_reserve))
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(U256::from(pair.total_lp_tokens))
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;
            let quote_amount: u128 = U256::from(lp_tokens)
                .checked_mul(U256::from(pair.quote_reserve))
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(U256::from(pair.total_lp_tokens))
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;

            // Slippage protection
            ensure!(base_amount >= min_base_amount, Error::<T>::SlippageExceeded);
            ensure!(
                quote_amount >= min_quote_amount,
                Error::<T>::SlippageExceeded
            );

            // Transfer funds from pool back to provider.
            //
            // P1-08: this used to pay `base_amount + quote_amount` as a single
            // native transfer, which was only meaningful when both sides were
            // DALLA. Each side is now paid in its own asset.
            let pool = Self::pool_account();
            let base_id = AssetId::from(base_asset);
            let quote_id = AssetId::from(quote_asset);
            T::Assets::transfer(base_id, &pool, &who, base_amount)?;
            T::Assets::transfer(quote_id, &pool, &who, quote_amount)?;

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
        pub fn cancel_order(origin: OriginFor<T>, order_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // AUDIT FIX (CW-1): Sanctions + KYC check for order cancellation
            ensure!(T::Kyc::is_kyc_ok(&who), Error::<T>::KycRequired);

            // Check global pause
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);

            let order = OrderBook::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            // Only the creator may cancel their own order
            ensure!(order.creator == who, Error::<T>::Unauthorized);

            // Remove from orderbook
            OrderBook::<T>::remove(order_id);
            AccountOrderCount::<T>::mutate(&who, |c| *c = c.saturating_sub(1));

            // P1-08: hand back whatever the order still holds. Refund happens
            // after the removal so a refund failure cannot leave a cancelled
            // order that is still matchable.
            Self::refund_escrow(&order)?;

            // Drop it from the pair index too so the index does not accumulate
            // dead IDs. Best-effort by design - a leftover ID is skipped by
            // `open_orders`, which re-reads `OrderBook`.
            Self::deindex_order(&order);

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

        /// Derive the limit-order escrow account from PalletId.
        ///
        /// Deliberately separate from `pool_account`: reserves in the pool feed
        /// the AMM curve, so escrow held there would be priced into every swap.
        pub fn order_escrow_account() -> T::AccountId {
            T::OrderEscrowPalletId::get().into_account_truncating()
        }

        /// Quote amount for `base_amount` at `price`, rounding up.
        ///
        /// `price` is scaled by `PRICE_SCALE`. Rounding **up** is what makes a bid's
        /// escrow sufficient for its own limit price; rounding down would leave the
        /// order a wei short of payable and it could never fill completely.
        fn quote_for(
            base_amount: u128,
            price: u128,
            round_up: bool,
        ) -> Result<u128, DispatchError> {
            let numerator = U256::from(base_amount)
                .checked_mul(U256::from(price))
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            let scale = U256::from(PRICE_SCALE);
            let mut result = numerator / scale;
            if round_up && !(numerator % scale).is_zero() {
                result = result.saturating_add(U256::one());
            }
            result
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow.into())
        }

        /// Which asset an order escrows, and how much of it one base unit costs.
        ///
        /// A bid holds quote and pays quote; an ask holds base and receives quote.
        fn escrow_asset_of(order: &OrderEntry<T::AccountId, BlockNumberFor<T>>) -> AssetId {
            if order.is_bid() {
                order.pair.1
            } else {
                order.pair.0
            }
        }

        /// Return an order's remaining escrow to its creator and zero the entry.
        fn refund_escrow(order: &OrderEntry<T::AccountId, BlockNumberFor<T>>) -> DispatchResult {
            if order.escrow == 0 {
                return Ok(());
            }
            T::Assets::transfer(
                Self::escrow_asset_of(order),
                &Self::order_escrow_account(),
                &order.creator,
                order.escrow,
            )
        }

        /// Match `taker_id` against the opposite side of its pair.
        ///
        /// Price-time priority: the opposite side is sorted best-price-first by
        /// `open_orders`, and equal prices fall back to the lower order id, which
        /// is insertion order. The walk stops when the taker is exhausted, the
        /// best opposing price no longer crosses, or the per-pair bound is reached.
        ///
        /// Deliberately non-fatal: a maker that cannot settle (escrow underflow,
        /// ledger rejection) is skipped and left resting rather than failing the
        /// whole placement, because the taker's order has already been accepted and
        /// escrowed. Nothing is half-applied — each fill moves both legs or neither.
        ///
        /// `pub(crate)` only so `benchmarking::match_orders` can measure the walk
        /// directly; it is not part of the pallet's callable surface.
        pub(crate) fn match_order(pair: (u8, u8), taker_id: u32) {
            let Some(mut taker) = OrderBook::<T>::get(taker_id) else {
                return;
            };
            if taker.remaining == 0 {
                return;
            }

            let current_block = frame_system::Pallet::<T>::block_number();
            let (bids, asks) = Self::open_orders(pair);
            let makers = if taker.is_bid() { asks } else { bids };

            for (maker_id, _) in makers {
                if maker_id == taker_id {
                    continue;
                }
                if taker.remaining == 0 {
                    break;
                }
                // Re-read: an earlier iteration in this same loop may have moved
                // this maker, and `open_orders` returned a snapshot.
                let Some(mut maker) = OrderBook::<T>::get(maker_id) else {
                    continue;
                };
                if maker.remaining == 0 || maker.creator == taker.creator {
                    // Self-trade is skipped, not rejected: the taker can still
                    // cross the next maker. `SelfTradeNotAllowed` stays reserved
                    // for a direct maker/taker call.
                    continue;
                }
                if maker.expires_at <= current_block {
                    Self::purge_expired(&maker);
                    continue;
                }
                if !Self::crosses(&taker, &maker) {
                    // Sorted by price, so the first non-crossing price ends the walk.
                    break;
                }

                let Some(plan) = Self::plan_fill(&taker, &maker) else {
                    // Taker escrow or the maker's remainder went inconsistent;
                    // leave the maker resting and stop rather than looping.
                    break;
                };
                if !Self::settle_fill(&taker, &maker, &plan) {
                    // The ledger rejected a leg and `settle_fill` put the other
                    // back, so both orders still hold their escrow. Leave the
                    // maker resting and stop: retrying it would be rejected the
                    // same way.
                    break;
                }
                // Funds have moved, so it is now safe to record the fill.
                Self::apply_fill(&mut taker, &mut maker, &plan);
                if maker.remaining == 0 {
                    Self::remove_order(&maker);
                } else {
                    // Moved, not cloned: `maker` is rebound from storage at the
                    // top of every iteration, so nothing reads it afterwards.
                    OrderBook::<T>::insert(maker_id, maker);
                }
            }

            // A fully-filled taker leaves the book, which refunds any escrow
            // rounding dust back to its creator.
            if taker.remaining == 0 {
                Self::remove_order(&taker);
            } else {
                OrderBook::<T>::insert(taker_id, taker);
            }
        }

        /// True when `taker`'s limit accepts `maker`'s price.
        ///
        /// Prices are compared directly because both are scaled by `PRICE_SCALE` and
        /// both orders name the same (base, quote) pair — a bid at or above the ask
        /// crosses.
        fn crosses(
            taker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
            maker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
        ) -> bool {
            if taker.is_bid() {
                taker.price >= maker.price
            } else {
                taker.price <= maker.price
            }
        }

        /// Work out the amounts for one fill without touching either order.
        ///
        /// Returns `None` if either side would have to pay more than it holds, or
        /// if the quote cannot be computed. The caller treats that as "stop".
        fn plan_fill(
            taker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
            maker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
        ) -> Option<FillAmounts> {
            let base_amount = taker.remaining.min(maker.remaining);
            if base_amount == 0 {
                return None;
            }
            // The maker's price wins — that is what resting liquidity buys.
            //
            // Rounded DOWN, unlike the escrow at placement. A bid escrows
            // `ceil(amount * price / PRICE_SCALE)` for its whole size, but a fill
            // charges for one slice at a time, and the sum of per-slice ceilings
            // can exceed that single ceiling: ten 1-unit fills at price 1.5 need
            // 20 quote against an escrow of 15. The order then fails the escrow
            // check below on every later walk and can never fill again.
            //
            // `sum(floor(a_i * p / S)) <= floor(A * p / S) <= ceil(A * p / S)`,
            // so rounding down is always covered by the escrow, and it strands at
            // most one base unit of dust per fill, refunded when the order leaves
            // the book.
            let quote_amount = Self::quote_for(base_amount, maker.price, false).ok()?;

            // Both legs must be covered by the escrow each side actually holds.
            let taker_pays = if taker.is_bid() {
                quote_amount
            } else {
                base_amount
            };
            let maker_pays = if maker.is_bid() {
                quote_amount
            } else {
                base_amount
            };
            if taker_pays > taker.escrow || maker_pays > maker.escrow {
                return None;
            }

            Some(FillAmounts {
                base_amount,
                quote_amount,
                taker_pays,
                maker_pays,
            })
        }

        /// Record a settled fill against both orders.
        ///
        /// Only called once the transfers have moved, so `remaining` and `escrow`
        /// can never claim a fill that did not happen.
        fn apply_fill(
            taker: &mut OrderEntry<T::AccountId, BlockNumberFor<T>>,
            maker: &mut OrderEntry<T::AccountId, BlockNumberFor<T>>,
            plan: &FillAmounts,
        ) {
            taker.remaining = taker.remaining.saturating_sub(plan.base_amount);
            taker.escrow = taker.escrow.saturating_sub(plan.taker_pays);
            maker.remaining = maker.remaining.saturating_sub(plan.base_amount);
            maker.escrow = maker.escrow.saturating_sub(plan.maker_pays);
        }

        /// Move both legs of a planned fill between escrow and the two parties.
        ///
        /// Everything routes through the escrow account so the pallet never mints
        /// or burns: the bidder's quote goes to the asker, the asker's base goes to
        /// the bidder.
        ///
        /// Returns `false` if either leg was rejected, having first put back
        /// whatever the other leg moved. `plan_fill` proves the escrow
        /// *bookkeeping* covers the fill, but not that the ledger will accept it —
        /// the adapter re-checks sanctions on bBZD, and a party can be sanctioned
        /// between placement and fill. Callers must not apply the fill when this
        /// returns `false`, and must stop rather than retry: the same maker would
        /// be rejected identically.
        fn settle_fill(
            taker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
            maker: &OrderEntry<T::AccountId, BlockNumberFor<T>>,
            plan: &FillAmounts,
        ) -> bool {
            let (bid, ask) = if taker.is_bid() {
                (taker, maker)
            } else {
                (maker, taker)
            };
            let escrow = Self::order_escrow_account();
            let (base_asset, quote_asset) = (bid.pair.0, bid.pair.1);

            let base_leg = T::Assets::transfer(base_asset, &escrow, &bid.creator, plan.base_amount);
            if base_leg.is_err() {
                return false;
            }
            let quote_leg =
                T::Assets::transfer(quote_asset, &escrow, &ask.creator, plan.quote_amount);
            if quote_leg.is_err() {
                // Undo the base leg so the escrow still backs both orders.
                let _ = T::Assets::transfer(base_asset, &bid.creator, &escrow, plan.base_amount);
                return false;
            }

            Self::deposit_event(Event::OrderFilled {
                maker_order_id: maker.order_id,
                taker_order_id: taker.order_id,
                maker: maker.creator.clone(),
                taker: taker.creator.clone(),
                base_amount: plan.base_amount,
                quote_amount: plan.quote_amount,
                price: maker.price,
            });
            true
        }

        /// Remove an order from the book and index, refunding its escrow.
        fn remove_order(order: &OrderEntry<T::AccountId, BlockNumberFor<T>>) {
            OrderBook::<T>::remove(order.order_id);
            AccountOrderCount::<T>::mutate(&order.creator, |c| *c = c.saturating_sub(1));
            Self::deindex_order(order);
            let _ = Self::refund_escrow(order);
        }

        /// Purge one expired order, refunding its escrow.
        fn purge_expired(order: &OrderEntry<T::AccountId, BlockNumberFor<T>>) {
            Self::remove_order(order);
            Self::deposit_event(Event::OrderExpired {
                order_id: order.order_id,
                creator: order.creator.clone(),
            });
        }

        /// Remove `order` from the pair index (P1-08 prerequisite).
        ///
        /// Best-effort: callers may hold an `OrderBook` entry whose index entry
        /// was already dropped, and `open_orders` re-reads `OrderBook` so a dead
        /// ID is skipped rather than trusted. Not returning an error keeps
        /// `cancel_order` from failing on a bookkeeping inconsistency.
        fn deindex_order(order: &OrderEntry<T::AccountId, BlockNumberFor<T>>) {
            let pair_key = (order.pair.0.as_u8(), order.pair.1.as_u8());
            let is_bid = matches!(order.order_type, OrderType::Buy | OrderType::TourismBuy);
            PairOrders::<T>::mutate(pair_key, |index| {
                let side = if is_bid {
                    &mut index.bids
                } else {
                    &mut index.asks
                };
                side.retain(|id| *id != order.order_id);
            });
        }

        /// Open orders for a pair, cheapest-to-sell first on the ask side and
        /// highest-price-first on the bid side.
        ///
        /// Reads ONE storage entry for the pair, then one per candidate — the
        /// point of the index. The returned list is bounded by
        /// `T::MaxOrdersPerPair x 2` before any per-order read happens.
        ///
        /// Stale IDs (an `OrderBook` entry removed without de-indexing) are
        /// skipped. Expired orders are returned as-is: expiry handling belongs to
        /// the caller, because a matching pass wants to skip them while a UI
        /// query may want to show them.
        pub fn open_orders(pair: (u8, u8)) -> (OrderList<T>, OrderList<T>) {
            let index = PairOrders::<T>::get(pair);
            let load = |ids: &BoundedVec<u32, T::MaxOrdersPerPair>| -> OrderList<T> {
                ids.iter()
                    .filter_map(|id| OrderBook::<T>::get(id).map(|o| (*id, o)))
                    .collect()
            };
            let mut bids = load(&index.bids);
            let mut asks = load(&index.asks);
            // Best bid first, best ask first. Consumers take the head and compare.
            bids.sort_by(|a, b| b.1.price.cmp(&a.1.price).then(a.0.cmp(&b.0)));
            asks.sort_by(|a, b| a.1.price.cmp(&b.1.price).then(a.0.cmp(&b.0)));
            (bids, asks)
        }

        /// Calculate output amount for trade using constant product formula
        fn get_amount_out(
            amount_in: u128,
            reserve_in: u128,
            reserve_out: u128,
        ) -> Result<u128, DispatchError> {
            ensure!(amount_in > 0, Error::<T>::BelowMinimumAmount);
            ensure!(
                reserve_in > 0 && reserve_out > 0,
                Error::<T>::InsufficientLiquidity
            );

            let numerator = U256::from(amount_in)
                .checked_mul(U256::from(reserve_out))
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            let denominator = U256::from(reserve_in).saturating_add(U256::from(amount_in));
            let result: u128 = (numerator / denominator)
                .try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;
            Ok(result)
        }

        /// Get current price for a trading pair
        pub fn get_price(base_asset: &AssetId, quote_asset: &AssetId) -> Option<FixedU128> {
            let pair_key: (u8, u8) = (base_asset.as_u8(), quote_asset.as_u8());
            let pair = Self::trading_pairs(pair_key)?;

            if pair.base_reserve == 0 {
                return None;
            }

            Some(FixedU128::from_rational(
                pair.quote_reserve,
                pair.base_reserve,
            ))
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
                    return Some(
                        pair.quote_reserve
                            .saturating_mul(1_000_000)
                            .checked_div(pair.base_reserve)
                            .unwrap_or(0),
                    );
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
        pub fn get_pair_reserves_u128(
            base_asset: u8,
            quote_asset: u8,
        ) -> Option<(u128, u128, u128)> {
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
    /// Cost of walking up to `fills` resting orders during one placement.
    ///
    /// `fills` is the caller-supplied bound (`T::MaxOrdersPerPair`), because the
    /// matching loop's cost is data-dependent and `#[pallet::weight]` cannot
    /// express that. Each fill reads and writes two `OrderBook` entries plus the
    /// two ledger legs.
    fn match_orders(orders: u32) -> Weight;
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
        Weight::from_parts(10_000_000, 1536).saturating_add(RocksDbWeight::get().writes(1))
    }
    fn place_order() -> Weight {
        Weight::from_parts(20_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    fn match_orders(orders: u32) -> Weight {
        // An iteration is either a fill or a purge of an expired order, and a walk
        // can be entirely one or the other. Charging both is a strict upper bound;
        // see the note on the same function in `weights.rs`.
        let per_order = Self::cancel_order().saturating_add(Self::execute_trade());
        per_order.saturating_mul(orders as u64).saturating_add(
            RocksDbWeight::get().reads(1_u64.saturating_add(2_u64.saturating_mul(orders as u64))),
        )
    }
    fn cancel_order() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn pause() -> Weight {
        Weight::from_parts(5_000_000, 512).saturating_add(RocksDbWeight::get().writes(1))
    }
    fn resume() -> Weight {
        Weight::from_parts(5_000_000, 512).saturating_add(RocksDbWeight::get().writes(1))
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
