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
//! 11. Tourism payment incentives (3-8% cashback)
//!
//! ## bBZD Fiat-Backed Model (USDC-style):
//! - Central Bank holds BZD reserves in traditional bank account
//! - User deposits BZ$100 → Central Bank mints 100 bBZD on-chain
//! - User redeems 100 bBZD → Chain burns bBZD → Central Bank transfers BZ$100 off-chain
//! - DALLA is NOT used as collateral (separate token for gas/fees/governance)

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, ExistenceRequirement,
        Get, UnixTime, ConstU32,
    },
    PalletId, BoundedVec,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::Saturating,
    Permill, RuntimeDebug,
};
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::weights::{Weight, constants::RocksDbWeight};

// ===== CONSTANTS =====

const TREASURY_ID: PalletId = PalletId(*b"bz/trsry");

/// Blocks per year (assuming 6 second block time)
/// 60 seconds/min * 60 min/hour * 24 hours/day * 365.25 days/year / 6 seconds/block
const BLOCKS_PER_YEAR: u32 = 5_256_000;

/// Annual inflation rate (2%)
const ANNUAL_INFLATION_RATE: Permill = Permill::from_percent(2);

// ===== WEIGHT INFO TRAIT =====

/// Weight functions needed for the Economy pallet
pub trait WeightInfo {
    fn issue_bbzd() -> Weight;  // Used for mint_bbzd
    fn redeem_bbzd() -> Weight;
    fn process_redemption() -> Weight;
    fn set_minter_authorization() -> Weight;
    fn update_reserves() -> Weight;
    fn pay_tourism_incentive() -> Weight; 
    fn send_remittance() -> Weight;
    fn update_inflation() -> Weight;
    fn update_peg_rate() -> Weight;
    fn emergency_shutdown() -> Weight;
    fn burn_dalla() -> Weight;
    fn governance_burn() -> Weight;
}

/// Default weight implementation
pub struct SubstrateWeight<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn issue_bbzd() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))  // CentralBankReserves, TotalBbzdSupply, AuthorizedMinters
            .saturating_add(RocksDbWeight::get().writes(2))  // BBZDBalances, TotalBbzdSupply
    }
    
    fn redeem_bbzd() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // BBZDBalances, TotalBbzdSupply
            .saturating_add(RocksDbWeight::get().writes(4))  // BBZDBalances, TotalBbzdSupply, RedemptionRequests, NextRedemptionId
    }
    
    fn process_redemption() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // AuthorizedMinters, RedemptionRequests
            .saturating_add(RocksDbWeight::get().writes(1))  // RedemptionRequests
    }
    
    fn set_minter_authorization() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(0))
            .saturating_add(RocksDbWeight::get().writes(1))  // AuthorizedMinters
    }
    
    fn update_reserves() -> Weight {
        Weight::from_parts(18_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))  // CentralBankReserves, TotalBbzdSupply
            .saturating_add(RocksDbWeight::get().writes(1))  // CentralBankReserves
    }
    
    fn pay_tourism_incentive() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    
    fn send_remittance() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    
    fn update_inflation() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    
    fn update_peg_rate() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    
    fn emergency_shutdown() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
    
    fn burn_dalla() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
    
    fn governance_burn() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}

// ===== TYPE DEFINITIONS =====

/// Multi-signature treasury operation
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MultiSigOperation<AccountId, Balance, BlockNumber> {
    /// Operation ID
    pub operation_id: u32,
    /// Operation type
    pub operation_type: TreasuryOperationType,
    /// Amount involved
    pub amount: Balance,
    /// Destination account (if applicable)
    pub destination: Option<AccountId>,
    /// Required signatures
    pub required_signatures: u32,
    /// Current signatures
    pub signatures: BoundedVec<AccountId, ConstU32<20>>,
    /// Block when operation was created
    pub created_at: BlockNumber,
    /// Block when operation expires
    pub expires_at: BlockNumber,
    /// Operation status
    pub status: OperationStatus,
    /// Operation metadata
    pub metadata: BoundedVec<u8, ConstU32<512>>,
}

/// Types of treasury operations requiring multi-signature
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TreasuryOperationType {
    /// Government treasury withdrawal
    GovernmentWithdrawal,
    /// Ministry budget allocation
    MinistryAllocation,
    /// Emergency fund access
    EmergencyFunding,
    /// Economic stimulus distribution
    StimulusDistribution,
    /// Monetary policy change
    MonetaryPolicyChange,
    /// Cross-border payment authorization
    CrossBorderPayment,
    /// Treasury reserve management
    ReserveManagement,
}

/// Tourism spending categories for different incentive rates
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

/// Account types for transaction limits
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum AccountType {
    /// Regular citizen account
    #[default]
    Citizen,
    /// Business account
    Business,
    /// Government account (unlimited but requires multi-sig)
    Government,
    /// Tourism account (higher limits for visitors)
    Tourism,
    /// Ministry account (2-of-3 signatures)
    Ministry,
    /// Emergency account (3-of-5 signatures with time delays)
    Emergency,
}

/// Operation execution status
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OperationStatus {
    /// Pending signatures
    Pending,
    /// Ready for execution
    ReadyToExecute,
    /// Successfully executed
    Executed,
    /// Operation expired
    Expired,
    /// Operation cancelled
    Cancelled,
    /// Operation failed during execution
    Failed,
}

/// Economic metrics for monitoring
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EconomicMetrics {
    /// Total DALLA in circulation
    pub total_dalla_supply: u128,
    /// Total bBZD in circulation
    pub total_bbzd_supply: u128,
    /// Current exchange rate (DALLA per BZD * 1e12)
    pub exchange_rate: u128,
    /// Economic stability score (0-100)
    pub stability_score: u8,
    /// Last update block
    pub last_updated: u32,
}

impl Default for EconomicMetrics {
    fn default() -> Self {
        Self {
            total_dalla_supply: 10_000_000_000_000u128, // 10M DALLA (6 decimals)
            total_bbzd_supply: 0u128,
            exchange_rate: 1_000_000u128, // 1:1 peg initially (scaled to 1e6)
            stability_score: 100u8,
            last_updated: 0u32,
        }
    }
}

/// Redemption status for tracking off-chain settlement
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RedemptionStatus {
    /// Awaiting Central Bank processing
    Pending,
    /// Off-chain BZD transfer completed
    Processed,
    /// Redemption cancelled (rare - requires governance)
    Cancelled,
}

/// Redemption request for off-chain settlement
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    pub type TotalSupply<T: Config> = StorageValue<
        _,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn economic_metrics)]
    /// Current economic metrics for Belize
    pub type EconomicMetricsStorage<T: Config> = StorageValue<_, EconomicMetrics, ValueQuery>;

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
    #[pallet::getter(fn authorized_minters)]
    /// Authorized Central Bank accounts that can mint bBZD
    pub type AuthorizedMinters<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_redemption_id)]
    /// Next redemption ID for tracking off-chain settlements
    pub type NextRedemptionId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn redemption_requests)]
    /// Pending redemption requests for Central Bank processing
    pub type RedemptionRequests<T: Config> = StorageMap<_, Blake2_128Concat, u64, RedemptionRequest<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn pending_redemption_ids)]
    /// Convenience index of currently pending redemption IDs for simple UI/tests access
    /// Bounded to prevent unbounded growth; governance should ensure timely processing.
    pub type PendingRedemptionIds<T: Config> = StorageValue<_, BoundedVec<u64, ConstU32<10000>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bbzd_balances)]
    /// bBZD stablecoin balances
    pub type BBZDBalances<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_payment_id)]
    /// Next tourism payment ID
    pub type NextPaymentId<T: Config> = StorageValue<_, u32, ValueQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn last_inflation_block)]
    /// Last block number when inflation was applied
    pub type LastInflationBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::from_parts(5_000_000, 0);
            
            // Check if a year has passed since last inflation
            let last_inflation = LastInflationBlock::<T>::get();
            
            // Convert block numbers to u64 for comparison
            let current_block: u64 = TryInto::<u64>::try_into(n).unwrap_or(0);
            let last_block: u64 = TryInto::<u64>::try_into(last_inflation).unwrap_or(0);
            let blocks_passed = current_block.saturating_sub(last_block);
            
            // Apply annual inflation if BLOCKS_PER_YEAR have passed
            if blocks_passed >= BLOCKS_PER_YEAR as u64 {
                let current_supply = TotalSupply::<T>::get();
                let max_supply = T::MaxSupply::get();
                
                // Calculate inflation amount (2% of current supply)
            let inflation_amount = ANNUAL_INFLATION_RATE * current_supply;
            
            // Check if adding inflation would exceed max supply
            let new_supply = current_supply.saturating_add(inflation_amount);
            
            if new_supply <= max_supply {
                    // Mint to treasury
                    let treasury = T::Treasury::get();
                    
                    // Deposit inflation to treasury
                    let _ = T::Currency::deposit_creating(&treasury, inflation_amount);
                    
                    // Update total supply
                    TotalSupply::<T>::put(new_supply);
                    
                    // Update last inflation block
                    LastInflationBlock::<T>::put(n);
                    
                    // Emit event
                    Self::deposit_event(Event::AnnualInflationApplied {
                        amount: inflation_amount,
                        new_supply,
                    });
                    
                    weight = weight.saturating_add(T::WeightInfo::update_inflation());
                }
                // If max supply would be exceeded, don't apply inflation (hard cap reached)
            }
            
            weight
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
        /// Exchange rate not available from Oracle
        ExchangeRateUnavailable,
        /// Merchant not verified by Oracle
        MerchantNotVerified,
        /// KYC verification required (minimum level not met)
        KycVerificationRequired,
        /// Account is sanctioned and cannot perform this operation
        SanctionedEntity,
        /// Amount must be greater than zero
        AmountMustBeNonZero,
        /// Redemption request not found
        RedemptionNotFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// bBZD minted by Central Bank (after off-chain BZD deposit verified)
        BbzdMinted {
            minter: T::AccountId,  // Central Bank account
            recipient: T::AccountId,
            amount: u128,
            deposit_reference: BoundedVec<u8, ConstU32<64>>,  // Off-chain bank transaction ID
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
            total_supply: u128,  // Must be <= new_reserves (1:1 backing)
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
        /// Cross-border remittance sent
        RemittanceSent {
            from: T::AccountId,
            to: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
            corridor: BoundedVec<u8, ConstU32<8>>,
        },
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
            
            // Ensure non-zero amount
            ensure!(amount > 0, Error::<T>::AmountMustBeNonZero);
            
            // ONLY authorized Central Bank accounts can mint
            ensure!(
                Self::authorized_minters(&minter),
                Error::<T>::UnauthorizedMinter
            );

            // Compliance: recipient must not be sanctioned and must have at least Basic KYC (level 1)
            ensure!(!T::Oracle::is_sanctioned(&recipient), Error::<T>::SanctionedEntity);
            ensure!(T::Oracle::meets_kyc_requirement(&recipient, 1), Error::<T>::KycVerificationRequired);
            
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
            ensure!(!T::Oracle::is_sanctioned(&who), Error::<T>::SanctionedEntity);
            ensure!(T::Oracle::meets_kyc_requirement(&who, 1), Error::<T>::KycVerificationRequired);
            
            // Verify user has sufficient bBZD balance
            let balance = Self::bbzd_balances(&who);
            ensure!(
                balance >= amount,
                Error::<T>::InsufficientBbzdBalance
            );
            
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
            NextRedemptionId::<T>::put(redemption_id.saturating_add(1));

            // Track pending redemption ID for easy discovery by UIs/tests
            PendingRedemptionIds::<T>::try_mutate(|ids| {
                ids.try_push(redemption_id)
                    .map_err(|_| Error::<T>::MaxSupplyReached)
            })?;
            
            // Update total supply
            let new_supply = TotalBbzdSupply::<T>::get();
            
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
        pub fn process_redemption(
            origin: OriginFor<T>,
            redemption_id: u64,
        ) -> DispatchResult {
            let processor = ensure_signed(origin)?;
            
            // Only Central Bank can mark redemptions as processed
            ensure!(
                Self::authorized_minters(&processor),
                Error::<T>::UnauthorizedMinter
            );
            
            // Get redemption request
            let mut request = Self::redemption_requests(redemption_id)
                .ok_or(Error::<T>::InvalidRedemptionId)?;
            
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
            let transfer_ref: BoundedVec<u8, ConstU32<64>> = transfer_ref_bytes
                .try_into()
                .unwrap_or_default();
            
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
        pub fn update_reserves(
            origin: OriginFor<T>,
            new_reserves: u128,
        ) -> DispatchResult {
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
        }        /// Process tourism payment with incentives (DALLA-based, NOT bBZD)
        ///
        /// Tourism incentive payments use DALLA tokens (5-8% rewards based on category).
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
                1 => TourismCategory::Dining,          // Oracle FoodBeverage
                2 => TourismCategory::Tours,            // Oracle TourOperator
                3 => TourismCategory::Transportation,
                4 => TourismCategory::Shopping,         // Oracle Retail
                5 => TourismCategory::Cultural,         // Oracle Other
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
            T::Currency::transfer(&treasury, &tourist, incentive_amount, ExistenceRequirement::KeepAlive)?;

            Self::deposit_event(Event::TourismIncentivePaid {
                tourist,
                vendor,
                amount,
                incentive: incentive_amount,
            });

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
            ensure!(
                amount > Zero::zero(),
                Error::<T>::AmountMustBeNonZero
            );
            
            // Verify balance
            ensure!(
                T::Currency::free_balance(&who) >= amount,
                Error::<T>::InsufficientBalance
            );
            
        // Slash tokens (effectively burning them)
        let _imbalance = T::Currency::slash(&who, amount);
        
        // Update total supply
        TotalSupply::<T>::mutate(|supply| {
            *supply = supply.saturating_sub(amount);
        });
        
        let new_supply = TotalSupply::<T>::get();            Self::deposit_event(Event::DallaBurned {
                who,
                amount,
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
        let _imbalance = T::Currency::slash(&treasury, amount);
        
        // Update total supply
        TotalSupply::<T>::mutate(|supply| {
            *supply = supply.saturating_sub(amount);
        });
        
        let new_supply = TotalSupply::<T>::get();            Self::deposit_event(Event::GovernanceBurn {
                amount,
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

        /// Get tourism incentive rate for category
        pub fn get_tourism_incentive_rate(category: &TourismCategory) -> u32 {
            match category {
                TourismCategory::Accommodation => 500, // 5%
                TourismCategory::Dining => 300,        // 3%
                TourismCategory::Tours => 700,         // 7%
                TourismCategory::Transportation => 200, // 2%
                TourismCategory::Shopping => 400,      // 4%
                TourismCategory::Cultural => 800,      // 8%
            }
        }
    }
}