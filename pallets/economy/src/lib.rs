#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain Economy Pallet - National Economic System with Commercial Security
//!
//! This pallet manages:
//! 1. DALLA token (native token with 6 decimals, max supply 501B) - Used for gas, fees, governance, DEX, AI/quantum payments
//! 2. bBZD stablecoin (1:1 Belize Dollar peg, USDC-style fiat-backed) - Central Bank mints on BZD deposit, burns on redemption
//! 3. Multi-signature treasury management with controlled inflation (2% annual)
//! 4. Token burning mechanisms (user-initiated and governance-controlled)
//! 5. Automatic annual inflation minted to treasury
//! 6. Transaction limits and velocity monitoring
//! 7. Comprehensive audit trails for compliance
//! 8. Rate limiting to prevent DoS attacks
//! 9. Emergency shutdown mechanisms
//! 10. Cross-border remittance optimization
//! 11. Tourism payment incentives (2-8% cashback)
//!
//! ## bBZD Fiat-Backed Model (USDC-style):
//! - Central Bank holds BZD reserves in traditional bank account
//! - User deposits BZ$100 → Central Bank mints 100 bBZD on-chain
//! - User redeems 100 bBZD → Chain burns bBZD → Central Bank transfers BZ$100 off-chain
//! - DALLA is NOT used as collateral (separate token for gas/fees/governance)

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::weights::{constants::RocksDbWeight, Weight};
use frame_support::{
    pallet_prelude::*,
    sp_runtime::traits::AccountIdConversion,
    traits::{ConstU32, Currency, ExistenceRequirement, Get, ReservableCurrency, UnixTime},
    BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Saturating, Zero},
    Debug, Permill, SaturatedConversion,
};

// ===== CONSTANTS =====

const TREASURY_ID: PalletId = PalletId(*b"py/trsry");

/// Blocks per year (assuming 6 second block time)
/// 60 seconds/min * 60 min/hour * 24 hours/day * 365.25 days/year / 6 seconds/block
const BLOCKS_PER_YEAR: u32 = 5_256_000;

/// Annual inflation rate (2%)
const ANNUAL_INFLATION_RATE: Permill = Permill::from_percent(2);

// ===== WEIGHT INFO TRAIT =====

/// Weight functions needed for the Economy pallet
pub trait WeightInfo {
    fn issue_bbzd() -> Weight; // Used for mint_bbzd
    fn redeem_bbzd() -> Weight;
    fn process_redemption() -> Weight;
    fn set_minter_authorization() -> Weight;
    fn update_reserves() -> Weight;
    fn pay_tourism_incentive() -> Weight;
    fn update_inflation() -> Weight;
    fn burn_dalla() -> Weight;
    fn governance_burn() -> Weight;
    fn transfer_bbzd() -> Weight;
}

/// Default weight implementation
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn issue_bbzd() -> Weight {
        Weight::from_parts(25_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3)) // CentralBankReserves, TotalBbzdSupply, AuthorizedMinters
            .saturating_add(RocksDbWeight::get().writes(2)) // BBZDBalances, TotalBbzdSupply
    }

    fn redeem_bbzd() -> Weight {
        Weight::from_parts(30_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2)) // BBZDBalances, TotalBbzdSupply
            .saturating_add(RocksDbWeight::get().writes(4)) // BBZDBalances, TotalBbzdSupply, RedemptionRequests, NextRedemptionId
    }

    fn process_redemption() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2)) // AuthorizedMinters, RedemptionRequests
            .saturating_add(RocksDbWeight::get().writes(1)) // RedemptionRequests
    }

    fn set_minter_authorization() -> Weight {
        Weight::from_parts(15_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(0))
            .saturating_add(RocksDbWeight::get().writes(1)) // AuthorizedMinters
    }

    fn update_reserves() -> Weight {
        Weight::from_parts(18_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(2)) // CentralBankReserves, TotalBbzdSupply
            .saturating_add(RocksDbWeight::get().writes(1)) // CentralBankReserves
    }

    fn pay_tourism_incentive() -> Weight {
        Weight::from_parts(50_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// DOS-012 FIX: Weight covers full inflation path — 3× deposit_creating,
    /// 2× cumulative mutate, TotalSupply sync, LastInflationBlock update.
    fn update_inflation() -> Weight {
        Weight::from_parts(60_000_000, 4096)
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(7))
    }

    fn burn_dalla() -> Weight {
        Weight::from_parts(30_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn governance_burn() -> Weight {
        Weight::from_parts(35_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn transfer_bbzd() -> Weight {
        // 2 reads (BBZDBalances sender + receiver), 2 writes (mutate both),
        // plus Oracle reads for sanctions/KYC checks on both parties
        Weight::from_parts(30_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}

// ===== TYPE DEFINITIONS =====

// E-7 FIX: Removed dead code types (MultiSigOperation, TreasuryOperationType,
// AccountType, OperationStatus). These were unused scaffolding for future multi-sig
// treasury operations. Will be re-introduced when multi-sig is implemented.

/// Tourism spending categories for different incentive rates
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum TourismCategory {
    /// Accommodation (hotels, resorts)
    Accommodation,
    /// Dining and beverages
    Dining,
    /// Tours and activities
    Tours,
    /// Transportation
    Transportation,
    /// Shopping and crafts
    Shopping,
    /// Cultural experiences
    Cultural,
}

// E-7 FIX: AccountType and OperationStatus removed (dead code).
// AccountType was scaffolding for transaction limit tiers.
// OperationStatus tracked multi-sig operation lifecycle.
// EconomicMetrics struct removed (E-7): write-only, never read on-chain.

/// Redemption status for tracking off-chain settlement
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum RedemptionStatus {
    /// Awaiting Central Bank processing
    Pending,
    /// Off-chain BZD transfer completed
    Processed,
    /// Redemption cancelled (rare - requires governance)
    Cancelled,
}

/// Redemption request for off-chain settlement
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct RedemptionRequest<AccountId> {
    /// User requesting redemption
    pub user: AccountId,
    /// Amount of bBZD to redeem
    pub amount: u128,
    /// User's bank account for off-chain transfer (UTF-8 encoded)
    pub bank_account: BoundedVec<u8, ConstU32<64>>,
    /// Redemption ID for tracking
    pub redemption_id: u64,
    /// Processing status
    pub status: RedemptionStatus,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::Imbalance;
    use pallet_belize_compliance::ComplianceReporter as _;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Native currency (DALLA token)
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Treasury account ID
        type Treasury: Get<Self::AccountId>;

        /// Unix time provider
        type UnixTime: UnixTime;

        /// WeightInfo trait for operation weights
        type WeightInfo: WeightInfo;

        /// Maximum DALLA supply (501B DALLA with 6 decimals = 501_000_000_000 * 10^6)
        #[pallet::constant]
        type MaxSupply: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Governance origin for economic policy changes
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Oracle for merchant verification only (NOT used for bBZD peg)
        type Oracle: OracleProvider<Self::AccountId>;

        // ── AR-15: Rate limiting ──────────────────────────────────────────────
        /// Maximum times a single account may call `mint_bbzd` per block.
        /// Prevents DoS flooding of the Central Bank mint extrinsic.
        #[pallet::constant]
        type MaxMintPerBlock: Get<u32>;

        // ── Phase 2C: Progressive public-goods routing ────────────────────────

        /// Account that accumulates the public-goods share of annual inflation.
        /// Distinct from the main treasury so funds can be governed separately.
        type PublicGoodsTreasury: Get<Self::AccountId>;

        /// Percentage of annual inflation (0–100) redirected to `PublicGoodsTreasury`.
        /// The remainder goes to the main `Treasury`.  Set to 0 to disable.
        #[pallet::constant]
        type PublicGoodsRoutingPercent: Get<u8>;

        // ── Phase 5B: Wellbeing treasury routing ──────────────────────────────

        /// Account that accumulates the wellbeing sub-share carved out of the
        /// public-goods allocation each year.  Funded via governance
        /// `WellbeingFunding` proposals and used for mental-health / digital-
        /// wellness programmes.
        type WellbeingTreasury: Get<Self::AccountId>;

        /// Percentage (0–100) of the *public-goods* portion of annual inflation
        /// that is further redirected to `WellbeingTreasury`.
        /// Example: PublicGoodsRoutingPercent=20, WellbeingFundPercent=10
        ///   → 10 % of the PG allocation (= 2 % of total inflation) goes to
        ///     wellbeing, the rest stays in the PG treasury.
        /// Set to 0 to disable.
        #[pallet::constant]
        type WellbeingFundPercent: Get<u8>;

        // ── COMP-CRIT-1: Travel rule enforcement ─────────────────────────────

        /// FATF travel rule threshold (in base units, e.g. 100 DALLA = 100_000_000_000).
        /// Transactions at or above this amount require Enhanced KYC (L2).
        #[pallet::constant]
        type TravelRuleThreshold: Get<u128>;

        /// COMP-CRIT-2: Structuring detection reporter.
        /// Called after each value transfer to feed the sliding-window detector.
        type ComplianceReporter: pallet_belize_compliance::ComplianceReporter<
            Self::AccountId,
            BlockNumberFor<Self>,
        >;
    }

    /// Oracle provider trait for merchant verification only
    pub trait OracleProvider<AccountId> {
        /// Verify merchant is registered for tourism category
        fn is_merchant_verified(merchant: &AccountId, category: u8) -> bool;
        /// Check if account meets a minimum KYC level
        fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool;
        /// Check if account is sanctioned
        fn is_sanctioned(account: &AccountId) -> bool;
    }

    #[pallet::storage]
    #[pallet::getter(fn total_supply)]
    /// Total DALLA supply in circulation
    pub type TotalSupply<T: Config> =
        StorageValue<_, <T::Currency as Currency<T::AccountId>>::Balance, ValueQuery>;

    // EconomicMetricsStorage removed (E-7): write-only, never read on-chain.

    #[pallet::storage]
    #[pallet::getter(fn central_bank_reserves)]
    /// Total BZD reserves held by Central Bank (off-chain) - for audit transparency
    /// This MUST equal or exceed TotalBbzdSupply at all times (1:1 backing)
    pub type CentralBankReserves<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_bbzd_supply)]
    /// Total bBZD supply in circulation (must be backed 1:1 by CentralBankReserves)
    pub type TotalBbzdSupply<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn minting_halted)]
    /// Set by `on_initialize` when bBZD supply exceeds reserves (defense-in-depth halt flag)
    pub type MintingHalted<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn authorized_minters)]
    /// Authorized Central Bank accounts that can mint bBZD
    pub type AuthorizedMinters<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_redemption_id)]
    /// Next redemption ID for tracking off-chain settlements
    pub type NextRedemptionId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn redemption_requests)]
    /// Pending redemption requests for Central Bank processing
    pub type RedemptionRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, RedemptionRequest<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn pending_redemption_ids)]
    /// Convenience index of currently pending redemption IDs for simple UI/tests access
    /// Bounded to prevent unbounded growth; governance should ensure timely processing.
    pub type PendingRedemptionIds<T: Config> =
        StorageValue<_, BoundedVec<u64, ConstU32<10000>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bbzd_balances)]
    /// bBZD stablecoin balances
    pub type BBZDBalances<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_payment_id)]
    /// Next tourism payment ID
    pub type NextPaymentId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_inflation_block)]
    /// Last block number when inflation was applied
    pub type LastInflationBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    // AR-15: Per-account per-block call counter for mint_bbzd rate limiting.
    // Key: (AccountId, last_block_number). Cleared lazily each new block.
    #[pallet::storage]
    /// Rate limit: number of mint_bbzd calls by account in the current block.
    pub type MintCallsThisBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    // Tracks the block number when MintCallsThisBlock was last reset.
    #[pallet::storage]
    pub type LastMintRateLimitBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    // CumulativePublicGoodsInflation removed (E-7): write-only accumulator, never read on-chain.
    // CumulativeWellbeingInflation removed (E-7): write-only accumulator, never read on-chain.

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::from_parts(5_000_000, 512);

            // E-1: Runtime bBZD invariant check — detect under-collateralization
            let bbzd_supply = TotalBbzdSupply::<T>::get();
            let reserves = CentralBankReserves::<T>::get();
            if bbzd_supply > reserves {
                MintingHalted::<T>::put(true);
                Self::deposit_event(Event::BbzdInvariantViolation {
                    total_supply: bbzd_supply,
                    reserves,
                });
                // Defensive: log but don't halt the chain
                frame_support::defensive!("bBZD invariant violated: supply > reserves");
            } else if MintingHalted::<T>::get() {
                // Reserves restored — clear the halt flag
                MintingHalted::<T>::put(false);
            }
            weight = weight.saturating_add(Weight::from_parts(2_000_000, 64));

            // Check if a year has passed since last inflation
            let last_inflation = LastInflationBlock::<T>::get();

            // Convert block numbers to u64 for comparison (W-8: safe saturated conversion)
            let current_block: u64 = n.saturated_into();
            let last_block: u64 = last_inflation.saturated_into();
            let blocks_passed = current_block.saturating_sub(last_block);

            // Apply annual inflation if BLOCKS_PER_YEAR have passed
            if blocks_passed >= BLOCKS_PER_YEAR as u64 {
                let current_supply = T::Currency::total_issuance();
                let max_supply = T::MaxSupply::get();

                // Calculate inflation amount (2% of current supply)
                let inflation_amount = ANNUAL_INFLATION_RATE * current_supply;

                // Check if adding inflation would exceed max supply
                let new_supply = current_supply.saturating_add(inflation_amount);

                if new_supply <= max_supply {
                    let treasury = T::Treasury::get();

                    // ── Phase 2C: Progressive public-goods routing ────────────
                    // Route `PublicGoodsRoutingPercent`% of inflation to the
                    // public-goods treasury; remainder goes to main treasury.
                    let routing_percent: u128 =
                        T::PublicGoodsRoutingPercent::get().min(100) as u128;
                    let total_u128: u128 = inflation_amount.saturated_into::<u128>();
                    let pg_u128: u128 = total_u128.saturating_mul(routing_percent) / 100;
                    let main_u128: u128 = total_u128.saturating_sub(pg_u128);

                    // ── Phase 5B: Carve wellbeing sub-portion from PG allocation ──
                    let wb_percent: u128 = T::WellbeingFundPercent::get().min(100) as u128;
                    let wb_u128: u128 = pg_u128.saturating_mul(wb_percent) / 100;
                    let pg_net_u128: u128 = pg_u128.saturating_sub(wb_u128);

                    let pg_amount: <T::Currency as Currency<T::AccountId>>::Balance =
                        pg_net_u128.saturated_into();
                    let wb_amount: <T::Currency as Currency<T::AccountId>>::Balance =
                        wb_u128.saturated_into();
                    let main_amount: <T::Currency as Currency<T::AccountId>>::Balance =
                        main_u128.saturated_into();

                    if pg_amount > Zero::zero() {
                        let pg_treasury = T::PublicGoodsTreasury::get();
                        let _ = T::Currency::deposit_creating(&pg_treasury, pg_amount);
                    }

                    if wb_amount > Zero::zero() {
                        let wb_treasury = T::WellbeingTreasury::get();
                        let _ = T::Currency::deposit_creating(&wb_treasury, wb_amount);
                    }

                    if main_amount > Zero::zero() {
                        let _ = T::Currency::deposit_creating(&treasury, main_amount);
                    }

                    // Sync TotalSupply to authoritative TotalIssuance (H-20)
                    let new_supply = T::Currency::total_issuance();
                    TotalSupply::<T>::put(new_supply);

                    // W-1: Emit actual minted amount (sum of deposits) not theoretical
                    let actually_minted = new_supply.saturating_sub(current_supply);

                    // Update last inflation block
                    LastInflationBlock::<T>::put(n);

                    // Emit event
                    Self::deposit_event(Event::AnnualInflationApplied {
                        amount: actually_minted,
                        new_supply,
                    });

                    weight = weight.saturating_add(T::WeightInfo::update_inflation());
                }
                // If max supply would be exceeded, don't apply inflation (hard cap reached)
            }

            weight
        }

        /// Verify critical economic invariants at runtime.
        ///
        /// Invariants:
        /// 1. TotalBbzdSupply ≤ CentralBankReserves (1:1 backing)
        /// 2. TotalBbzdSupply ≤ MaxSupply (hard cap)
        fn integrity_test() {
            let supply = TotalBbzdSupply::<T>::get();
            let reserves = CentralBankReserves::<T>::get();
            assert!(
                supply <= reserves,
                "INVARIANT VIOLATION: TotalBbzdSupply ({}) > CentralBankReserves ({})",
                supply,
                reserves,
            );
            // MaxSupply is a Balance; convert to u128 for comparison.
            let max_u128: u128 = TotalSupply::<T>::get().try_into().unwrap_or(u128::MAX);
            assert!(
                supply <= max_u128,
                "INVARIANT VIOLATION: TotalBbzdSupply ({}) > MaxSupply ({})",
                supply,
                max_u128,
            );
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient balance for operation
        InsufficientBalance,
        /// Unauthorized minter (not Central Bank)
        UnauthorizedMinter,
        /// Invalid redemption ID
        InvalidRedemptionId,
        /// Redemption already processed
        RedemptionAlreadyProcessed,
        /// Insufficient bBZD balance for redemption
        InsufficientBbzdBalance,
        /// Reserve backing insufficient (audit failure)
        InsufficientReserves,
        /// Tourism incentive calculation failed
        TourismIncentiveError,
        /// Remittance corridor not configured
        RemittanceCorridorNotFound,
        /// Invalid tourism category ID provided
        InvalidTourismCategory,
        /// Maximum supply cap reached
        MaxSupplyReached,
        /// Cannot burn more than available supply
        InsufficientSupplyToBurn,
        /// Pending redemption queue is full
        RedemptionQueueFull,
        /// AR-15: Account has exceeded the maximum mint_bbzd calls permitted per block.
        RateLimitExceeded,
        /// Exchange rate not available from Oracle
        ExchangeRateUnavailable,
        /// Merchant not verified by Oracle
        MerchantNotVerified,
        /// KYC verification required (minimum level not met)
        KycVerificationRequired,
        /// Account is sanctioned and cannot perform this operation
        SanctionedEntity,
        /// Minting halted: bBZD supply exceeds reserves (under-collateralization detected)
        MintingHaltedUndercollateralized,
        /// Amount must be greater than zero
        AmountMustBeNonZero,
        /// Redemption request not found
        RedemptionNotFound,
        /// COMP-CRIT-1: Travel rule requires Enhanced KYC (L2) for large transactions
        TravelRuleEnhancedKycRequired,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// bBZD minted by Central Bank (after off-chain BZD deposit verified)
        BbzdMinted {
            minter: T::AccountId, // Central Bank account
            recipient: T::AccountId,
            amount: u128,
            deposit_reference: BoundedVec<u8, ConstU32<64>>, // Off-chain bank transaction ID
            new_total_supply: u128,
        },
        /// bBZD redeemed (burned on-chain, triggers off-chain BZD transfer)
        BbzdRedeemed {
            user: T::AccountId,
            amount: u128,
            bank_account: BoundedVec<u8, ConstU32<64>>,
            redemption_id: u64,
            new_total_supply: u128,
        },
        /// Redemption processed by Central Bank (off-chain transfer completed)
        RedemptionProcessed {
            redemption_id: u64,
            amount: u128,
            bank_transfer_reference: BoundedVec<u8, ConstU32<64>>,
        },
        /// Central Bank reserves updated (for audit transparency)
        ReservesUpdated {
            old_reserves: u128,
            new_reserves: u128,
            total_supply: u128, // Must be <= new_reserves (1:1 backing)
        },
        /// Authorized minter added/removed
        MinterAuthorization {
            minter: T::AccountId,
            authorized: bool,
        },
        /// Tourism incentive paid
        TourismIncentivePaid {
            tourist: T::AccountId,
            vendor: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            incentive: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// COMP-CRIT-1: Travel rule triggered — large transaction logged for audit
        TravelRuleTriggered {
            from: T::AccountId,
            to: T::AccountId,
            amount: u128,
        },
        /// bBZD peer-to-peer transfer (W-5)
        BbzdTransferred {
            from: T::AccountId,
            to: T::AccountId,
            amount: u128,
        },
        // RemittanceSent removed (E-7): orphaned, never emitted.
        /// Annual inflation applied to treasury
        AnnualInflationApplied {
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            new_supply: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// DALLA tokens burned (user-initiated)
        DallaBurned {
            who: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            new_supply: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// DALLA tokens burned by governance
        GovernanceBurn {
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            new_supply: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// E-1: bBZD invariant violation detected at runtime
        BbzdInvariantViolation { total_supply: u128, reserves: u128 },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint bBZD stablecoin (Central Bank only - USDC-style fiat-backed)
        ///
        /// This extrinsic mints new bBZD tokens ONLY when the Central Bank has received
        /// an off-chain BZD deposit into their traditional bank account. The flow is:
        ///
        /// 1. User deposits 100 BZD into Central Bank's fiat account
        /// 2. Central Bank verifies deposit via traditional banking system
        /// 3. Central Bank calls this extrinsic with deposit reference number
        /// 4. bBZD is minted 1:1 (100 BZD → 100 bBZD)
        /// 5. Central Bank reserves increase on-chain (audit transparency)
        ///
        /// **CRITICAL**: NO DALLA COLLATERAL is involved. This is pure fiat-backing.
        ///
        /// # Parameters
        /// - `origin`: Must be an authorized Central Bank minter account
        /// - `recipient`: User account receiving the bBZD
        /// - `amount`: Amount of bBZD to mint (1:1 with off-chain BZD deposit)
        /// - `deposit_reference`: Off-chain bank deposit reference (audit trail)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::issue_bbzd())]
        pub fn mint_bbzd(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            amount: u128,
            deposit_reference: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let minter = ensure_signed(origin)?;
            Self::check_mint_rate_limit(&minter)?;

            // C-ECON-1: Block minting while under-collateralized
            ensure!(
                !MintingHalted::<T>::get(),
                Error::<T>::MintingHaltedUndercollateralized
            );

            // Ensure non-zero amount
            ensure!(amount > 0, Error::<T>::AmountMustBeNonZero);

            // ONLY authorized Central Bank accounts can mint
            ensure!(
                Self::authorized_minters(&minter),
                Error::<T>::UnauthorizedMinter
            );

            // Compliance: recipient must not be sanctioned and must have at least Basic KYC (level 1)
            ensure!(
                !T::Oracle::is_sanctioned(&recipient),
                Error::<T>::SanctionedEntity
            );
            ensure!(
                T::Oracle::meets_kyc_requirement(&recipient, 1),
                Error::<T>::KycVerificationRequired
            );

            // COMP-CRIT-1: Travel rule — large mints require Enhanced KYC (L2)
            let threshold = T::TravelRuleThreshold::get();
            if amount >= threshold {
                ensure!(
                    T::Oracle::meets_kyc_requirement(&recipient, 2),
                    Error::<T>::TravelRuleEnhancedKycRequired
                );
                Self::deposit_event(Event::TravelRuleTriggered {
                    from: minter.clone(),
                    to: recipient.clone(),
                    amount,
                });
            }

            // Verify sufficient off-chain reserves exist
            // Total supply MUST NEVER exceed Central Bank's BZD holdings
            let current_supply = Self::total_bbzd_supply();
            let current_reserves = Self::central_bank_reserves();
            let new_supply = current_supply.saturating_add(amount);

            ensure!(
                new_supply <= current_reserves,
                Error::<T>::InsufficientReserves
            );

            // Mint bBZD to recipient (pure mint, no collateral locked)
            BBZDBalances::<T>::mutate(&recipient, |balance| {
                *balance = balance.saturating_add(amount);
            });

            // Update total supply tracking
            TotalBbzdSupply::<T>::put(new_supply);

            // COMP-CRIT-2: Report transaction to structuring detector
            T::ComplianceReporter::report_transaction(&recipient, amount);

            // Emit event with deposit reference for off-chain audit trail
            Self::deposit_event(Event::BbzdMinted {
                minter,
                recipient,
                amount,
                deposit_reference,
                new_total_supply: new_supply,
            });

            Ok(())
        }

        /// Redeem bBZD for off-chain BZD transfer (user-initiated burn)
        ///
        /// This extrinsic allows users to burn their bBZD and receive BZD in their
        /// traditional bank account. The flow is:
        ///
        /// 1. User calls this extrinsic with 100 bBZD + bank account details
        /// 2. bBZD is burned immediately (removed from circulation)
        /// 3. Redemption request queued for Central Bank processing
        /// 4. Central Bank transfers 100 BZD to user's bank account off-chain
        /// 5. Central Bank calls `process_redemption` to mark settlement complete
        ///
        /// **CRITICAL**: Burn happens FIRST, settlement happens off-chain after.
        ///
        /// # Parameters
        /// - `origin`: User requesting redemption
        /// - `amount`: Amount of bBZD to burn
        /// - `bank_account`: User's traditional bank account (BIC/IBAN or local format)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::redeem_bbzd())]
        pub fn redeem_bbzd(
            origin: OriginFor<T>,
            amount: u128,
            bank_account: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure non-zero amount
            ensure!(amount > 0, Error::<T>::AmountMustBeNonZero);

            // Compliance: redeemer must not be sanctioned and must have at least Basic KYC (level 1)
            ensure!(
                !T::Oracle::is_sanctioned(&who),
                Error::<T>::SanctionedEntity
            );
            ensure!(
                T::Oracle::meets_kyc_requirement(&who, 1),
                Error::<T>::KycVerificationRequired
            );

            // COMP-CRIT-1: Travel rule — large redemptions require Enhanced KYC (L2)
            let threshold = T::TravelRuleThreshold::get();
            if amount >= threshold {
                ensure!(
                    T::Oracle::meets_kyc_requirement(&who, 2),
                    Error::<T>::TravelRuleEnhancedKycRequired
                );
            }

            // Verify user has sufficient bBZD balance
            let balance = Self::bbzd_balances(&who);
            ensure!(balance >= amount, Error::<T>::InsufficientBbzdBalance);

            // Burn bBZD immediately (remove from circulation)
            BBZDBalances::<T>::mutate(&who, |balance| {
                *balance = balance.saturating_sub(amount);
            });

            // Update total supply
            TotalBbzdSupply::<T>::mutate(|supply| {
                *supply = supply.saturating_sub(amount);
            });

            // Create redemption request for Central Bank processing
            let redemption_id = Self::next_redemption_id();
            let request = RedemptionRequest {
                user: who.clone(),
                amount,
                bank_account: bank_account.clone(),
                redemption_id,
                status: RedemptionStatus::Pending,
            };

            RedemptionRequests::<T>::insert(redemption_id, request);
            // W-4: Use checked_add to prevent ID collision at u64::MAX
            let next_id = redemption_id
                .checked_add(1)
                .ok_or(Error::<T>::RedemptionQueueFull)?;
            NextRedemptionId::<T>::put(next_id);

            // Track pending redemption ID for easy discovery by UIs/tests
            PendingRedemptionIds::<T>::try_mutate(|ids| {
                ids.try_push(redemption_id)
                    .map_err(|_| Error::<T>::RedemptionQueueFull)
            })?;

            // Update total supply
            let new_supply = TotalBbzdSupply::<T>::get();

            // COMP-CRIT-2: Report transaction to structuring detector
            T::ComplianceReporter::report_transaction(&who, amount);

            // Emit event for Central Bank to process off-chain settlement
            Self::deposit_event(Event::BbzdRedeemed {
                user: who,
                amount,
                bank_account,
                redemption_id,
                new_total_supply: new_supply,
            });

            Ok(())
        }

        /// Process redemption settlement (Central Bank confirms off-chain transfer)
        ///
        /// Called by Central Bank after successfully transferring BZD to user's
        /// traditional bank account. Updates on-chain records for audit transparency.
        ///
        /// # Parameters
        /// - `origin`: Must be authorized Central Bank minter
        /// - `redemption_id`: ID of the redemption request
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::process_redemption())]
        pub fn process_redemption(origin: OriginFor<T>, redemption_id: u64) -> DispatchResult {
            let processor = ensure_signed(origin)?;

            // Only Central Bank can mark redemptions as processed
            ensure!(
                Self::authorized_minters(&processor),
                Error::<T>::UnauthorizedMinter
            );

            // Get redemption request
            let mut request =
                Self::redemption_requests(redemption_id).ok_or(Error::<T>::InvalidRedemptionId)?;

            // Verify not already processed
            ensure!(
                matches!(request.status, RedemptionStatus::Pending),
                Error::<T>::RedemptionAlreadyProcessed
            );

            // Mark as processed
            request.status = RedemptionStatus::Processed;
            RedemptionRequests::<T>::insert(redemption_id, request.clone());

            // Remove from pending list if present
            PendingRedemptionIds::<T>::mutate(|ids| {
                if let Some(pos) = ids.iter().position(|id| *id == redemption_id) {
                    ids.swap_remove(pos);
                }
            });

            // Emit confirmation event with redemption_id as bank transfer reference
            // (Central Bank would provide actual bank transaction ID off-chain)
            // Using simple numeric conversion without to_string (no_std compat)
            let id_bytes = redemption_id.to_le_bytes();
            let mut transfer_ref_bytes = b"REDEMPTION_".to_vec();
            transfer_ref_bytes.extend_from_slice(&id_bytes);
            let transfer_ref: BoundedVec<u8, ConstU32<64>> =
                transfer_ref_bytes.try_into().unwrap_or_default();

            Self::deposit_event(Event::RedemptionProcessed {
                redemption_id,
                amount: request.amount,
                bank_transfer_reference: transfer_ref,
            });

            Ok(())
        }

        /// Set minter authorization (governance-controlled Central Bank management)
        ///
        /// Adds or removes accounts authorized to mint bBZD. Only governance can
        /// manage Central Bank minter accounts for security.
        ///
        /// # Parameters
        /// - `origin`: Must be governance
        /// - `account`: Account to authorize/deauthorize
        /// - `authorized`: true = grant minting authority, false = revoke
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::set_minter_authorization())]
        pub fn set_minter_authorization(
            origin: OriginFor<T>,
            account: T::AccountId,
            authorized: bool,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            AuthorizedMinters::<T>::insert(&account, authorized);

            Self::deposit_event(Event::MinterAuthorization {
                minter: account,
                authorized,
            });

            Ok(())
        }

        /// Update Central Bank reserves (audit transparency)
        ///
        /// Governance periodically updates the on-chain record of how much BZD
        /// the Central Bank holds in traditional bank accounts. This provides
        /// public transparency and ensures total supply ≤ reserves.
        ///
        /// # Parameters
        /// - `origin`: Must be governance
        /// - `new_reserves`: Updated BZD reserve amount (in bBZD decimals)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_reserves())]
        pub fn update_reserves(origin: OriginFor<T>, new_reserves: u128) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let old_reserves = Self::central_bank_reserves();
            let total_supply = Self::total_bbzd_supply();

            // Reserves can never be less than circulating supply
            // (would mean unbacked bBZD in circulation)
            ensure!(
                new_reserves >= total_supply,
                Error::<T>::InsufficientReserves
            );

            CentralBankReserves::<T>::put(new_reserves);

            Self::deposit_event(Event::ReservesUpdated {
                old_reserves,
                new_reserves,
                total_supply,
            });

            Ok(())
        }
        /// Process tourism payment with incentives (DALLA-based, NOT bBZD)
        ///
        /// Tourism incentive payments use DALLA tokens (2-8% rewards based on category).
        /// This does NOT involve bBZD - tourists pay vendors in DALLA and receive
        /// DALLA incentives from the treasury.
        ///
        /// # Parameters
        /// - `origin`: Tourist making payment
        /// - `vendor`: Verified merchant account
        /// - `amount`: Payment amount in DALLA
        /// - `category_id`: Tourism category (0=Accommodation, 1=Dining, 2=Tours, etc.)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::pay_tourism_incentive())]
        pub fn process_tourism_payment(
            origin: OriginFor<T>,
            vendor: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            category_id: u8,
        ) -> DispatchResult {
            let tourist = ensure_signed(origin)?;

            // COMP-CRIT-1: Compliance checks (previously missing entirely)
            ensure!(
                !T::Oracle::is_sanctioned(&tourist),
                Error::<T>::SanctionedEntity
            );
            ensure!(
                T::Oracle::meets_kyc_requirement(&tourist, 1),
                Error::<T>::KycVerificationRequired
            );

            // Travel rule — large payments require Enhanced KYC (L2)
            let amount_u128: u128 = amount.saturated_into();
            let threshold = T::TravelRuleThreshold::get();
            if amount_u128 >= threshold {
                ensure!(
                    T::Oracle::meets_kyc_requirement(&tourist, 2),
                    Error::<T>::TravelRuleEnhancedKycRequired
                );
                Self::deposit_event(Event::TravelRuleTriggered {
                    from: tourist.clone(),
                    to: vendor.clone(),
                    amount: amount_u128,
                });
            }

            // Verify vendor is registered and verified for this category via Oracle first
            // Oracle uses: 0=Accommodation, 1=FoodBeverage, 2=TourOperator, 3=Transportation, 4=Retail, 5=Other
            ensure!(
                T::Oracle::is_merchant_verified(&vendor, category_id),
                Error::<T>::MerchantNotVerified
            );

            // Convert category_id to TourismCategory for incentive calculation
            // Map Oracle categories to Economy categories
            let category = match category_id {
                0 => TourismCategory::Accommodation,
                1 => TourismCategory::Dining, // Oracle FoodBeverage
                2 => TourismCategory::Tours,  // Oracle TourOperator
                3 => TourismCategory::Transportation,
                4 => TourismCategory::Shopping, // Oracle Retail
                5 => TourismCategory::Cultural, // Oracle Other
                _ => return Err(Error::<T>::InvalidTourismCategory.into()),
            };

            // Ensure sufficient DALLA balance
            ensure!(
                T::Currency::free_balance(&tourist) >= amount,
                Error::<T>::InsufficientBalance
            );

            // Calculate tourism incentive based on category (DALLA rewards)
            let incentive_rate = Self::get_tourism_incentive_rate(&category);
            let incentive_amount = Permill::from_parts(incentive_rate) * amount;

            // Transfer DALLA payment to vendor
            T::Currency::transfer(&tourist, &vendor, amount, ExistenceRequirement::KeepAlive)?;

            // Pay DALLA incentive from treasury
            let treasury = T::Treasury::get();
            T::Currency::transfer(
                &treasury,
                &tourist,
                incentive_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // COMP-CRIT-2: Report transaction to structuring detector
            T::ComplianceReporter::report_transaction(&tourist, amount_u128);

            Self::deposit_event(Event::TourismIncentivePaid {
                tourist,
                vendor,
                amount,
                incentive: incentive_amount,
            });

            Ok(())
        }

        /// Transfer bBZD stablecoin between accounts (W-5: peer-to-peer transfer)
        ///
        /// Enables bBZD holders to send stablecoin to other verified accounts.
        /// Both sender and receiver must pass KYC/sanctions checks.
        /// Travel rule applies for amounts at or above the threshold.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::transfer_bbzd())]
        pub fn transfer_bbzd(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            ensure!(amount > 0, Error::<T>::AmountMustBeNonZero);
            ensure!(from != to, Error::<T>::InsufficientBbzdBalance); // no self-transfer

            // KYC/sanctions checks on both parties
            ensure!(
                !T::Oracle::is_sanctioned(&from),
                Error::<T>::SanctionedEntity
            );
            ensure!(!T::Oracle::is_sanctioned(&to), Error::<T>::SanctionedEntity);
            ensure!(
                T::Oracle::meets_kyc_requirement(&from, 1),
                Error::<T>::KycVerificationRequired
            );
            ensure!(
                T::Oracle::meets_kyc_requirement(&to, 1),
                Error::<T>::KycVerificationRequired
            );

            // Travel rule for large transfers
            let threshold = T::TravelRuleThreshold::get();
            if amount >= threshold {
                ensure!(
                    T::Oracle::meets_kyc_requirement(&from, 2),
                    Error::<T>::TravelRuleEnhancedKycRequired
                );
                Self::deposit_event(Event::TravelRuleTriggered {
                    from: from.clone(),
                    to: to.clone(),
                    amount,
                });
            }

            let sender_balance = BBZDBalances::<T>::get(&from);
            ensure!(
                sender_balance >= amount,
                Error::<T>::InsufficientBbzdBalance
            );

            BBZDBalances::<T>::mutate(&from, |b| *b = b.saturating_sub(amount));
            BBZDBalances::<T>::mutate(&to, |b| *b = b.saturating_add(amount));

            // Report to compliance structuring detector
            T::ComplianceReporter::report_transaction(&from, amount);

            Self::deposit_event(Event::BbzdTransferred { from, to, amount });

            Ok(())
        }

        /// Burn DALLA tokens voluntarily (user-initiated deflationary mechanism)
        ///
        /// Allows any user to permanently destroy their DALLA tokens, reducing the total supply.
        /// This creates deflationary pressure and helps stabilize the economy. NOT related to bBZD.
        ///
        /// # Parameters
        /// - `origin`: Signed origin of the account burning tokens
        /// - `amount`: Amount of DALLA to burn (must not exceed user's balance)
        ///
        /// # Errors
        /// - `InsufficientBalance`: User doesn't have enough DALLA to burn
        ///
        /// # Events
        /// - `DallaBurned { who, amount, new_supply }`: Emitted when tokens are successfully burned
        ///
        /// # Example
        /// ```ignore
        /// // Burn 1000 DALLA (1000 * 10^6 with 6 decimals)
        /// Economy::burn_dalla(Origin::signed(user), 1_000_000_000)?;
        /// ```
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::burn_dalla())]
        pub fn burn_dalla(
            origin: OriginFor<T>,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure non-zero amount
            ensure!(amount > Zero::zero(), Error::<T>::AmountMustBeNonZero);

            // Verify balance
            ensure!(
                T::Currency::free_balance(&who) >= amount,
                Error::<T>::InsufficientBalance
            );

            // Slash tokens (effectively burning them)
            // slash() returns (NegativeImbalance, remaining) — remaining is the amount that could NOT be slashed
            let (imbalance, remaining) = T::Currency::slash(&who, amount);
            let actually_burned = imbalance.peek();

            // Drop imbalance — this decrements TotalIssuance
            drop(imbalance);

            // Sync TotalSupply to authoritative TotalIssuance (fixes H-20 desync & H-21 double-decrement)
            TotalSupply::<T>::put(T::Currency::total_issuance());

            // If slash was incomplete, ensure caller is aware
            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            let new_supply = T::Currency::total_issuance();
            Self::deposit_event(Event::DallaBurned {
                who,
                amount: actually_burned,
                new_supply,
            });

            Ok(())
        }

        /// Governance-controlled burn from treasury (monetary policy tool)
        ///
        /// Allows governance to burn DALLA tokens directly from the treasury account.
        /// This is a critical economic policy tool for controlling inflation, managing supply,
        /// and responding to macroeconomic conditions. Only callable by governance origin
        /// (e.g., elected councils, root, or multi-signature governance).
        ///
        /// # Parameters
        /// - `origin`: Governance origin (must pass GovernanceOrigin check)
        /// - `amount`: Amount of DALLA to burn from treasury (must not exceed treasury balance)
        ///
        /// # Errors
        /// - `BadOrigin`: Caller is not authorized governance origin
        /// - `InsufficientBalance`: Treasury doesn't have enough DALLA to burn
        ///
        /// # Events
        /// - `GovernanceBurn { amount, new_supply }`: Emitted when tokens are successfully burned
        ///
        /// # Example
        /// ```ignore
        /// // Governance burns 1M DALLA from treasury (1M * 10^6 with 6 decimals)
        /// Economy::governance_burn(Origin::from(GovernanceOrigin), 1_000_000_000_000)?;
        /// ```
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::governance_burn())]
        pub fn governance_burn(
            origin: OriginFor<T>,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            // Only governance can call
            T::GovernanceOrigin::ensure_origin(origin)?;

            let treasury = T::Treasury::get();

            // Verify treasury balance
            ensure!(
                T::Currency::free_balance(&treasury) >= amount,
                Error::<T>::InsufficientBalance
            );

            // Slash from treasury
            let (imbalance, remaining) = T::Currency::slash(&treasury, amount);
            let actually_burned = imbalance.peek();

            // Drop imbalance — decrements TotalIssuance
            drop(imbalance);

            // Sync TotalSupply to authoritative TotalIssuance (fixes H-20/H-21)
            TotalSupply::<T>::put(T::Currency::total_issuance());

            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            let new_supply = T::Currency::total_issuance();
            Self::deposit_event(Event::GovernanceBurn {
                amount: actually_burned,
                new_supply,
            });

            Ok(())
        }
    }

    // ===== HELPER FUNCTIONS =====

    impl<T: Config> Pallet<T> {
        /// Get treasury account
        pub fn treasury_account() -> T::AccountId {
            TREASURY_ID.into_account_truncating()
        }

        /// Check per-account mint rate limit for the current block.
        /// W-2: Uses lazy staleness instead of unbounded clear(u32::MAX).
        fn check_mint_rate_limit(who: &T::AccountId) -> frame_support::dispatch::DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let last_block = LastMintRateLimitBlock::<T>::get();
            // If new block, advance generation marker; stale entries are logically zero
            let stale = current_block != last_block;
            if stale {
                LastMintRateLimitBlock::<T>::put(current_block);
            }
            let prev = if stale {
                0u32
            } else {
                MintCallsThisBlock::<T>::get(who)
            };
            let count = prev.saturating_add(1);
            ensure!(
                count <= T::MaxMintPerBlock::get(),
                Error::<T>::RateLimitExceeded
            );
            MintCallsThisBlock::<T>::insert(who, count);
            Ok(())
        }

        /// Get tourism incentive rate for category (Permill parts: 1_000_000 = 100%)
        pub fn get_tourism_incentive_rate(category: &TourismCategory) -> u32 {
            match category {
                TourismCategory::Accommodation => 50_000,  // 5%
                TourismCategory::Dining => 30_000,         // 3%
                TourismCategory::Tours => 70_000,          // 7%
                TourismCategory::Transportation => 20_000, // 2%
                TourismCategory::Shopping => 40_000,       // 4%
                TourismCategory::Cultural => 80_000,       // 8%
            }
        }
    }
}
