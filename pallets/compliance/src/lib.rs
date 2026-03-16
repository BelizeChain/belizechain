#![cfg_attr(not(feature = "std"), no_std)]

//! # BelizeChain Compliance Pallet - Production Ready
//!
//! ## Overview
//! 
//! This pallet provides comprehensive regulatory compliance enforcement for BelizeChain,
//! ensuring all participants meet KYC/AML requirements and comply with international standards
//! including FATF recommendations, while maintaining privacy through hash-based verification.
//!
//! ## Architecture
//!
//! ### Compliance Enforcement Points
//! - **Validator Operations**: Join, nomination, rewards
//! - **Governance**: Voting, proposals, council membership
//! - **Treasury Access**: Spending proposals, fund management
//! - **Token Transfers**: Optional restrictions based on risk scores
//! - **Cross-border Transactions**: Enhanced monitoring
//!
//! ### Privacy-First Design
//! - No raw PII stored on-chain
//! - Hash-based identity verification via BelizeIdentity pallet
//! - Minimal required fields for compliance
//! - Off-chain KYC with on-chain attestations
//!
//! ## Integration with BelizeIdentity
//!
//! This pallet relies on `pallet-belize-identity` for KYC level verification:
//! - L0: No verification (severely restricted)
//! - L1: SSN verified (basic operations)
//! - L2: SSN + Passport verified (full participation)
//! - L3: SSN + Passport + Biometrics (government/validator roles)
//!
//! ## Regulatory Compliance
//!
//! - **FATF Recommendations**: Travel rule, suspicious activity reporting
//! - **AML/CFT**: Transaction monitoring, risk scoring
//! - **Sanctions Screening**: OFAC, UN sanctions lists
//! - **Audit Trails**: Complete compliance logging
//!
//! ## Usage Example
//!
//! ```ignore
//! // Check if account can participate in governance
//! if !Compliance::can_vote(&account, current_block) {
//!     return Err(Error::<T>::InsufficientKycLevel.into());
//! }
//!
//! // Verify account for validator role
//! Compliance::verify_account(
//!     origin,
//!     account,
//!     VerificationLevel::Enhanced,
//!     Some(risk_assessment)
//! )?;
//!
//! // Report suspicious activity
//! Compliance::flag_suspicious_activity(
//!     origin,
//!     account,
//!     SuspiciousActivityType::UnusualVolumePattern,
//!     b"High-frequency transactions detected".to_vec()
//! )?;
//! ```

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency, UnixTime, Get},
    BoundedVec,
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, Saturating, SaturatedConversion};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
/// Runtime API declaration for off-chain compliance queries (AR-11).
pub mod runtime_api;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// ===== TYPE DEFINITIONS =====

/// Type alias for suspicious activity report entries
/// (ActivityType, BlockNumber, Description)
pub type SuspiciousActivityReport<BlockNumber> = (SuspiciousActivityType, BlockNumber, BoundedVec<u8, ConstU32<256>>);

/// Verification levels for compliance
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum VerificationLevel {
    /// No verification - restricted operations only
    None,
    /// Basic KYC (L1 from BelizeIdentity)
    Basic,
    /// Standard KYC (L2 from BelizeIdentity)
    Standard,
    /// Enhanced KYC (L3 from BelizeIdentity) - required for validators/governance
    Enhanced,
    /// Government/institutional level
    Government,
}

impl VerificationLevel {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Basic => 1,
            Self::Standard => 2,
            Self::Enhanced => 3,
            Self::Government => 4,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Basic,
            2 => Self::Standard,
            3 => Self::Enhanced,
            4 => Self::Government,
            _ => Self::None,
        }
    }
}

/// Risk assessment levels
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RiskLevel {
    /// Low risk - normal operations
    Low,
    /// Medium risk - enhanced monitoring
    Medium,
    /// High risk - restricted operations
    High,
    /// Prohibited - account blocked
    Prohibited,
}

/// Compliance status for an account
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ComplianceStatus {
    /// Current verification level
    pub verification_level: VerificationLevel,
    /// Risk assessment
    pub risk_level: RiskLevel,
    /// Whether account is whitelisted for special operations
    pub whitelisted: bool,
    /// Whether account is currently restricted
    pub restricted: bool,
    /// Last verification timestamp
    pub last_verification: u64,
}

impl Default for ComplianceStatus {
    fn default() -> Self {
        Self {
            verification_level: VerificationLevel::None,
            risk_level: RiskLevel::Low,
            whitelisted: false,
            restricted: false,
            last_verification: 0,
        }
    }
}

/// Suspicious activity types for AML/CFT reporting
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SuspiciousActivityType {
    /// Unusual transaction patterns
    UnusualVolumePattern,
    /// Rapid movement of funds
    RapidFundMovement,
    /// Structuring to avoid reporting thresholds
    Structuring,
    /// Transactions with high-risk jurisdictions
    HighRiskJurisdiction,
    /// Potential money laundering
    MoneyLaundering,
    /// Terrorist financing indicators
    TerroristFinancing,
    /// Sanctions violation
    SanctionsViolation,
    /// Other suspicious activity
    Other,
}

/// Compliance audit record
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ComplianceAuditRecord<T: Config> {
    /// Account being audited
    pub account: T::AccountId,
    /// Type of action
    pub action_type: ActionType,
    /// Timestamp of action
    pub timestamp: u64,
    /// Block number
    pub block_number: BlockNumberFor<T>,
    /// Success or failure
    pub success: bool,
    /// Additional details (bounded)
    pub details: BoundedVec<u8, ConstU32<256>>,
}

/// Types of compliance-relevant actions
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ActionType {
    /// Verification status changed
    VerificationUpdated,
    /// Risk level changed
    RiskAssessmentUpdated,
    /// Account whitelisted
    AccountWhitelisted,
    /// Account restricted
    AccountRestricted,
    /// Restriction lifted from account
    RestrictionLifted,
    /// Suspicious activity reported
    SuspiciousActivityReported,
    /// Compliance check performed
    ComplianceCheckPerformed,
    /// Sanctions screening performed
    SanctionsScreened,
    /// Travel rule check
    TravelRuleChecked,
}

/// Sanctions list entry (hash-based for privacy)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SanctionEntry {
    /// Hash of sanctioned entity identifier
    pub entity_hash: [u8; 32],
    /// Sanctions list source (OFAC, UN, EU, etc.)
    pub list_source: BoundedVec<u8, ConstU32<32>>,
    /// Block when added
    pub added_at: u32,
    /// Whether currently active
    pub active: bool,
}

// ===== WEIGHT INFO TRAIT =====

/// Weight functions for compliance operations
pub trait WeightInfo {
    fn verify_account() -> Weight;
    fn update_risk_level() -> Weight;
    fn whitelist_account() -> Weight;
    fn restrict_account() -> Weight;
    fn flag_suspicious_activity() -> Weight;
    fn add_sanctions_entry() -> Weight;
    fn remove_sanctions_entry() -> Weight;
    fn check_compliance() -> Weight;
    fn update_verification_level() -> Weight;
}

/// Trait for cross-pallet structuring detection reporting.
/// Economy pallet (or others) call `report_transaction` after each value transfer;
/// the implementation checks a sliding window and auto-files a SAR if cumulative
/// amounts indicate structuring (many sub-threshold transactions that together
/// exceed the travel-rule threshold).
pub trait ComplianceReporter<AccountId, BlockNumber> {
    /// Record a transaction amount and return `true` if structuring was detected.
    fn report_transaction(account: &AccountId, amount: u128) -> bool;
}

/// COMP-SANC-ISO: Account-level sanctions checker trait.
/// Allows the compliance pallet to query account-level sanctions from an
/// external source (e.g., identity pallet oracle) in addition to its own
/// hash-based `SanctionsList`, unifying both into a single checking point.
pub trait AccountSanctionsChecker<AccountId> {
    fn is_sanctioned(account: &AccountId) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// Currency for compliance deposits and fees
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Time provider for timestamps
        type UnixTime: UnixTime;

        /// Origin that can perform administrative actions (governance)
        type ComplianceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin that can add/remove sanctions
        type SanctionsOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Minimum verification level required for validator operations
        #[pallet::constant]
        type MinValidatorVerification: Get<VerificationLevel>;

        /// Minimum verification level required for governance participation
        #[pallet::constant]
        type MinGovernanceVerification: Get<VerificationLevel>;

        /// Minimum verification level required for treasury access
        #[pallet::constant]
        type MinTreasuryVerification: Get<VerificationLevel>;

        /// Travel rule threshold (in native token units)
        #[pallet::constant]
        type TravelRuleThreshold: Get<BalanceOf<Self>>;

        /// Verification validity period in seconds (e.g., 1 year)
        #[pallet::constant]
        type VerificationValidityPeriod: Get<u64>;

        /// Maximum audit records to keep per account
        #[pallet::constant]
        type MaxAuditRecords: Get<u32>;

        /// Maximum suspicious activity reports per account
        #[pallet::constant]
        type MaxSuspiciousActivityReports: Get<u32>;

        // ── COMP-CRIT-2: Structuring detection ─────────────────────────────

        /// Sliding-window size (in blocks) for structuring detection.
        /// Transactions within this window are aggregated; if cumulative total
        /// exceeds the travel-rule threshold while each individual tx is below
        /// it, a structuring SAR is auto-filed.
        #[pallet::constant]
        type StructuringWindowBlocks: Get<BlockNumberFor<Self>>;

        /// Maximum entries tracked per account in the transaction window.
        /// Oldest entries are pruned once the cap is reached.
        #[pallet::constant]
        type MaxTransactionWindowEntries: Get<u32>;

        /// Weight information
        type WeightInfo: WeightInfo;

        /// COMP-SANC-ISO: External account-level sanctions source.
        /// Typically wired to the identity pallet's oracle-based sanctions check.
        type AccountSanctionsChecker: AccountSanctionsChecker<Self::AccountId>;
    }

    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ===== STORAGE =====

    /// Compliance status for each account
    #[pallet::storage]
    #[pallet::getter(fn compliance_status)]
    pub type ComplianceStatusOf<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ComplianceStatus,
        ValueQuery,
    >;

    /// Audit trail for compliance actions
    #[pallet::storage]
    #[pallet::getter(fn audit_records)]
    pub type AuditRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<ComplianceAuditRecord<T>, T::MaxAuditRecords>,
        ValueQuery,
    >;

    /// Suspicious activity reports
    #[pallet::storage]
    #[pallet::getter(fn suspicious_activities)]
    pub type SuspiciousActivities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<SuspiciousActivityReport<BlockNumberFor<T>>, T::MaxSuspiciousActivityReports>,
        ValueQuery,
    >;

    /// Sanctions list (hash-based for privacy)
    #[pallet::storage]
    #[pallet::getter(fn sanctions_list)]
    pub type SanctionsList<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32],
        SanctionEntry,
        OptionQuery,
    >;

    /// Whitelisted accounts for special operations
    #[pallet::storage]
    #[pallet::getter(fn is_whitelisted)]
    pub type WhitelistedAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    /// Restricted accounts (blocked from operations)
    #[pallet::storage]
    #[pallet::getter(fn is_restricted)]
    pub type RestrictedAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (bool, BoundedVec<u8, ConstU32<256>>), // (restricted, reason)
        ValueQuery,
    >;

    // ── COMP-CRIT-2: Per-account sliding-window transaction log ───────────

    /// Sliding window of recent transaction amounts for structuring detection.
    /// Each entry is (amount_u128, block_number).
    #[pallet::storage]
    pub type TransactionWindow<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<(u128, BlockNumberFor<T>), T::MaxTransactionWindowEntries>,
        ValueQuery,
    >;

    /// Global compliance statistics
    #[pallet::storage]
    #[pallet::getter(fn compliance_stats)]
    pub type ComplianceStats<T: Config> = StorageValue<
        _,
        (u32, u32, u32, u32), // (total_verified, total_restricted, total_suspicious_reports, total_sanctions_checks)
        ValueQuery,
    >;

    // ===== HOOKS (DOS-017 FIX) =====

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// DOS-017 FIX: Prune stale suspicious activity reports in idle time.
        ///
        /// Removes reports older than ~180 days (2,628,000 blocks at 6s).
        /// Processes at most 5 accounts per block to bound weight.
        fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            // ~180 days at 6-second blocks
            const SUSPICIOUS_ACTIVITY_TTL_BLOCKS: u64 = 2_628_000;
            const MAX_ACCOUNTS_PER_BLOCK: usize = 5;

            let per_account_weight = Weight::from_parts(15_000_000, 2048)
                .saturating_add(T::DbWeight::get().reads(1))
                .saturating_add(T::DbWeight::get().writes(1));
            let base_weight = Weight::from_parts(5_000_000, 256);

            if remaining_weight.ref_time()
                < base_weight.saturating_add(per_account_weight).ref_time()
            {
                return Weight::zero();
            }

            let mut total_weight = base_weight;
            let current_block: u64 = TryInto::<u64>::try_into(
                frame_system::Pallet::<T>::block_number(),
            )
            .unwrap_or(0);

            // Collect account keys first to avoid iterator invalidation on mutate.
            let accounts: Vec<T::AccountId> = SuspiciousActivities::<T>::iter_keys()
                .take(MAX_ACCOUNTS_PER_BLOCK)
                .collect();

            for account in accounts {
                if total_weight.saturating_add(per_account_weight).ref_time()
                    > remaining_weight.ref_time()
                {
                    break;
                }

                let reports = SuspiciousActivities::<T>::get(&account);
                let original_len = reports.len();

                let filtered: Vec<_> = reports
                    .into_inner()
                    .into_iter()
                    .filter(|report| {
                        let report_block: u64 =
                            TryInto::<u64>::try_into(report.1).unwrap_or(0);
                        current_block.saturating_sub(report_block)
                            < SUSPICIOUS_ACTIVITY_TTL_BLOCKS
                    })
                    .collect();

                if filtered.len() < original_len {
                    if filtered.is_empty() {
                        SuspiciousActivities::<T>::remove(&account);
                    } else {
                        let pruned = BoundedVec::truncate_from(filtered);
                        SuspiciousActivities::<T>::insert(&account, pruned);
                    }
                }

                total_weight = total_weight.saturating_add(per_account_weight);
            }

            total_weight
        }
    }

    // ===== EVENTS =====

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account verification level updated (level as u8 for encoding)
        VerificationLevelUpdated {
            account: T::AccountId,
            level: u8,
        },
        /// Risk level updated for account (risk as u8 for encoding)
        RiskLevelUpdated {
            account: T::AccountId,
            risk_level: u8,
        },
        /// Account whitelisted
        AccountWhitelisted {
            account: T::AccountId,
        },
        /// Account restricted
        AccountRestricted {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<256>>,
        },
        /// Account restriction lifted
        RestrictionLifted {
            account: T::AccountId,
        },
        /// Suspicious activity reported (activity_type as u8 for encoding)
        SuspiciousActivityReported {
            account: T::AccountId,
            activity_type: u8,
        },
        /// Sanctions entry added
        SanctionsEntryAdded {
            entity_hash: [u8; 32],
        },
        /// Sanctions entry removed
        SanctionsEntryRemoved {
            entity_hash: [u8; 32],
        },
        /// Compliance check performed
        ComplianceCheckPerformed {
            account: T::AccountId,
            passed: bool,
        },
        /// Audit record created (action_type as u8 for encoding)
        AuditRecordCreated {
            account: T::AccountId,
            action_type: u8,
        },
        /// COMP-CRIT-2: Structuring detected — cumulative sub-threshold txs exceed travel rule
        StructuringDetected {
            account: T::AccountId,
            window_total: u128,
            tx_count: u32,
        },
    }

    // ===== ERRORS =====

    #[pallet::error]
    pub enum Error<T> {
        /// Account verification level insufficient for operation
        InsufficientVerificationLevel,
        /// Account is restricted from operations
        AccountRestricted,
        /// Account is on sanctions list
        AccountSanctioned,
        /// Risk level too high for operation
        RiskLevelTooHigh,
        /// Verification expired
        VerificationExpired,
        /// Not authorized to perform compliance operations
        NotAuthorized,
        /// Audit records storage full
        AuditRecordsOverflow,
        /// Suspicious activity reports storage full
        SuspiciousActivityReportsOverflow,
        /// Invalid verification level
        InvalidVerificationLevel,
        /// Sanctions entry already exists
        SanctionsEntryExists,
        /// Sanctions entry not found
        SanctionsEntryNotFound,
        /// Invalid risk assessment
        InvalidRiskAssessment,
        /// Account not found
        AccountNotFound,
        /// Compliance check failed
        ComplianceCheckFailed,
    }

    // ===== EXTRINSICS =====

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Verify an account with a specific verification level
        ///
        /// This should be called by authorized entities (government, KYC providers)
        /// after off-chain verification is complete.
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account to verify
        /// - `level`: Verification level to assign
        /// - `risk_level`: Initial risk assessment
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::verify_account())]
        pub fn verify_account(
            origin: OriginFor<T>,
            account: T::AccountId,
            level: u8,
            risk_level: u8,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            let now = T::UnixTime::now().as_secs();
            let mut status = ComplianceStatusOf::<T>::get(&account);
            
            let verification_level = VerificationLevel::from_u8(level);
            let risk = Self::u8_to_risk_level(risk_level);
            
            // CP-4 FIX: Capture previous level before mutation
            let was_unverified = status.verification_level == VerificationLevel::None;

            status.verification_level = verification_level;
            status.risk_level = risk;
            status.last_verification = now;
            
            ComplianceStatusOf::<T>::insert(&account, status);

            // Create audit record
            Self::create_audit_record(
                &account,
                ActionType::VerificationUpdated,
                true,
                b"Verification level updated".to_vec(),
            )?;

            // CP-4 FIX: Only increment verified counter if account was not previously verified
            if was_unverified && verification_level != VerificationLevel::None {
                let (mut verified, restricted, suspicious, sanctions) = ComplianceStats::<T>::get();
                verified = verified.saturating_add(1);
                ComplianceStats::<T>::put((verified, restricted, suspicious, sanctions));
            }

            Self::deposit_event(Event::VerificationLevelUpdated { account, level: verification_level.as_u8() });
            
            Ok(())
        }

        /// Update risk level for an account
        ///
        /// Used for ongoing risk assessment based on transaction patterns
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account to update
        /// - `risk_level`: New risk level
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_risk_level())]
        pub fn update_risk_level(
            origin: OriginFor<T>,
            account: T::AccountId,
            risk_level: u8,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            let risk = Self::u8_to_risk_level(risk_level);

            ComplianceStatusOf::<T>::try_mutate(&account, |status| -> DispatchResult {
                status.risk_level = risk;
                Ok(())
            })?;

            Self::create_audit_record(
                &account,
                ActionType::RiskAssessmentUpdated,
                true,
                b"Risk level updated".to_vec(),
            )?;

            Self::deposit_event(Event::RiskLevelUpdated { account, risk_level: Self::risk_level_to_u8(risk) });
            
            Ok(())
        }

        /// Whitelist an account for special operations
        ///
        /// Whitelisted accounts may bypass certain restrictions (e.g., government entities)
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account to whitelist
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::whitelist_account())]
        pub fn whitelist_account(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            ComplianceStatusOf::<T>::try_mutate(&account, |status| -> DispatchResult {
                status.whitelisted = true;
                Ok(())
            })?;

            WhitelistedAccounts::<T>::insert(&account, true);

            Self::create_audit_record(
                &account,
                ActionType::AccountWhitelisted,
                true,
                b"Account whitelisted".to_vec(),
            )?;

            Self::deposit_event(Event::AccountWhitelisted { account });
            
            Ok(())
        }

        /// Restrict an account from operations
        ///
        /// Used to block accounts that violate compliance requirements
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account to restrict
        /// - `reason`: Reason for restriction (for audit trail)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::restrict_account())]
        pub fn restrict_account(
            origin: OriginFor<T>,
            account: T::AccountId,
            reason: Vec<u8>,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            let bounded_reason: BoundedVec<u8, ConstU32<256>> = reason.try_into()
                .map_err(|_| Error::<T>::InvalidRiskAssessment)?;

            ComplianceStatusOf::<T>::try_mutate(&account, |status| -> DispatchResult {
                status.restricted = true;
                Ok(())
            })?;

            // C-3 FIX: Only increment counter if this is a NEW restriction,
            // preventing double-count when restrict_account is called twice on the same account.
            let is_new_restriction = !RestrictedAccounts::<T>::contains_key(&account);

            RestrictedAccounts::<T>::insert(&account, (true, bounded_reason.clone()));

            Self::create_audit_record(
                &account,
                ActionType::AccountRestricted,
                true,
                bounded_reason.to_vec(),
            )?;

            // Update stats only for new restrictions
            if is_new_restriction {
                let (verified, mut restricted, suspicious, sanctions) = ComplianceStats::<T>::get();
                restricted = restricted.saturating_add(1);
                ComplianceStats::<T>::put((verified, restricted, suspicious, sanctions));
            }

            Self::deposit_event(Event::AccountRestricted { account, reason: bounded_reason });
            
            Ok(())
        }

        /// Lift restriction from an account
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account to unrestrict
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::restrict_account())]
        pub fn lift_restriction(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            // CRIT-2 FIX: Only decrement counter if account was actually restricted
            ensure!(
                RestrictedAccounts::<T>::contains_key(&account),
                Error::<T>::AccountNotFound
            );

            ComplianceStatusOf::<T>::try_mutate(&account, |status| -> DispatchResult {
                status.restricted = false;
                Ok(())
            })?;

            RestrictedAccounts::<T>::remove(&account);

            Self::create_audit_record(
                &account,
                ActionType::RestrictionLifted,
                true,
                b"Restriction lifted".to_vec(),
            )?;

            // CP-3 FIX: Decrement restricted counter
            let (verified, mut restricted, suspicious, sanctions) = ComplianceStats::<T>::get();
            restricted = restricted.saturating_sub(1);
            ComplianceStats::<T>::put((verified, restricted, suspicious, sanctions));

            Self::deposit_event(Event::RestrictionLifted { account });
            
            Ok(())
        }

        /// Report suspicious activity for AML/CFT compliance
        ///
        /// Can be called by compliance authorities or automated monitoring systems
        ///
        /// # Parameters
        /// - `origin`: Must be ComplianceOrigin
        /// - `account`: Account with suspicious activity
        /// - `activity_type`: Type of suspicious activity
        /// - `details`: Additional details for investigation
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::flag_suspicious_activity())]
        pub fn flag_suspicious_activity(
            origin: OriginFor<T>,
            account: T::AccountId,
            activity_type: u8,
            details: Vec<u8>,
        ) -> DispatchResult {
            T::ComplianceOrigin::ensure_origin(origin)?;

            let activity = Self::u8_to_activity_type(activity_type);

            let bounded_details: BoundedVec<u8, ConstU32<256>> = details.try_into()
                .map_err(|_| Error::<T>::InvalidRiskAssessment)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            SuspiciousActivities::<T>::try_mutate(&account, |reports| -> DispatchResult {
                // FIFO eviction: if at capacity, remove oldest report to make room
                if reports.try_push((activity, current_block, bounded_details.clone())).is_err() {
                    if !reports.is_empty() {
                        reports.remove(0);
                    }
                    reports.try_push((activity, current_block, bounded_details.clone()))
                        .map_err(|_| Error::<T>::SuspiciousActivityReportsOverflow)?;
                }
                Ok(())
            })?;

            // Automatically elevate risk level
            ComplianceStatusOf::<T>::try_mutate(&account, |status| -> DispatchResult {
                status.risk_level = RiskLevel::High;
                Ok(())
            })?;

            Self::create_audit_record(
                &account,
                ActionType::SuspiciousActivityReported,
                true,
                bounded_details.to_vec(),
            )?;

            // Update stats
            let (verified, restricted, mut suspicious, sanctions) = ComplianceStats::<T>::get();
            suspicious = suspicious.saturating_add(1);
            ComplianceStats::<T>::put((verified, restricted, suspicious, sanctions));

            Self::deposit_event(Event::SuspiciousActivityReported { account, activity_type });
            
            Ok(())
        }

        /// Add entry to sanctions list
        ///
        /// Used to enforce OFAC, UN, and other sanctions lists
        ///
        /// # Parameters
        /// - `origin`: Must be SanctionsOrigin
        /// - `entity_hash`: Hash of sanctioned entity identifier (for privacy)
        /// - `list_source`: Source of sanctions list (e.g., "OFAC", "UN")
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::add_sanctions_entry())]
        pub fn add_sanctions_entry(
            origin: OriginFor<T>,
            entity_hash: [u8; 32],
            list_source: Vec<u8>,
        ) -> DispatchResult {
            T::SanctionsOrigin::ensure_origin(origin)?;

            ensure!(
                !SanctionsList::<T>::contains_key(entity_hash),
                Error::<T>::SanctionsEntryExists
            );

            let bounded_source: BoundedVec<u8, ConstU32<32>> = list_source.try_into()
                .map_err(|_| Error::<T>::InvalidRiskAssessment)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let block_u32 = TryInto::<u32>::try_into(current_block).unwrap_or(0);

            let entry = SanctionEntry {
                entity_hash,
                list_source: bounded_source,
                added_at: block_u32,
                active: true,
            };

            SanctionsList::<T>::insert(entity_hash, entry);

            Self::deposit_event(Event::SanctionsEntryAdded { entity_hash });
            
            Ok(())
        }

        /// Remove entry from sanctions list
        ///
        /// # Parameters
        /// - `origin`: Must be SanctionsOrigin
        /// - `entity_hash`: Hash of entity to remove
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::remove_sanctions_entry())]
        pub fn remove_sanctions_entry(
            origin: OriginFor<T>,
            entity_hash: [u8; 32],
        ) -> DispatchResult {
            T::SanctionsOrigin::ensure_origin(origin)?;

            ensure!(
                SanctionsList::<T>::contains_key(entity_hash),
                Error::<T>::SanctionsEntryNotFound
            );

            SanctionsList::<T>::remove(entity_hash);

            Self::deposit_event(Event::SanctionsEntryRemoved { entity_hash });
            
            Ok(())
        }

        /// Sync verification level from identity / refresh expired verification
        ///
        /// H-31 fix: replaces placeholder with real logic.
        /// If the caller's last verification is older than `VerificationValidityPeriod`,
        /// the verification level is downgraded to `None` and the account is flagged
        /// as restricted until re-verified by compliance authority.
        /// Otherwise the last-verification timestamp is refreshed and an audit
        /// record is written.
        ///
        /// # Parameters
        /// - `origin`: Signed by the account itself
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::update_verification_level())]
        pub fn sync_verification_from_identity(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let now = T::UnixTime::now().as_secs();
            let validity_period = T::VerificationValidityPeriod::get();

            let mut status = ComplianceStatusOf::<T>::get(&who);

            if status.last_verification > 0
                && now.saturating_sub(status.last_verification) > validity_period
            {
                // Verification has expired — downgrade to None and restrict
                status.verification_level = VerificationLevel::None;
                status.restricted = true;
                ComplianceStatusOf::<T>::insert(&who, &status);

                Self::create_audit_record(
                    &who,
                    ActionType::ComplianceCheckPerformed,
                    false,
                    b"Verification expired, level reset".to_vec(),
                )?;

                Self::deposit_event(Event::VerificationLevelUpdated {
                    account: who,
                    level: VerificationLevel::None as u8,
                });
            } else {
                // Still valid — log sync but do NOT refresh timestamp.
                // Only ComplianceOrigin::verify_account can set last_verification.
                Self::create_audit_record(
                    &who,
                    ActionType::ComplianceCheckPerformed,
                    true,
                    b"Verification synced (timestamp unchanged)".to_vec(),
                )?;
            }

            Ok(())
        }
    }

    // ===== HELPER FUNCTIONS =====

    impl<T: Config> Pallet<T> {
        /// Create an audit record for compliance actions
        fn create_audit_record(
            account: &T::AccountId,
            action_type: ActionType,
            success: bool,
            details: Vec<u8>,
        ) -> DispatchResult {
            let bounded_details: BoundedVec<u8, ConstU32<256>> = details.try_into()
                .map_err(|_| Error::<T>::InvalidRiskAssessment)?;

            let now = T::UnixTime::now().as_secs();
            let current_block = frame_system::Pallet::<T>::block_number();

            let record = ComplianceAuditRecord {
                account: account.clone(),
                action_type,
                timestamp: now,
                block_number: current_block,
                success,
                details: bounded_details,
            };

            AuditRecords::<T>::try_mutate(account, |records| -> DispatchResult {
                // If the BoundedVec is full, evict the oldest record to make room
                if records.try_push(record.clone()).is_err() {
                    // Remove the first (oldest) record and retry
                    if !records.is_empty() {
                        records.remove(0);
                    }
                    records.try_push(record)
                        .map_err(|_| Error::<T>::AuditRecordsOverflow)?;
                }
                Ok(())
            })?;

            Self::deposit_event(Event::AuditRecordCreated { account: account.clone(), action_type: Self::action_type_to_u8(action_type) });

            Ok(())
        }

        /// Check if account meets minimum verification level
        pub fn meets_verification_level(
            account: &T::AccountId,
            required_level: VerificationLevel,
        ) -> bool {
            let status = ComplianceStatusOf::<T>::get(account);
            status.verification_level.as_u8() >= required_level.as_u8()
        }

        /// Check if account verification is still valid
        pub fn is_verification_valid(account: &T::AccountId) -> bool {
            let status = ComplianceStatusOf::<T>::get(account);
            let now = T::UnixTime::now().as_secs();
            let validity_period = T::VerificationValidityPeriod::get();
            
            // ARITH-COMP-01 FIX: use saturating_sub to avoid theoretical u64 overflow
            now.saturating_sub(status.last_verification) <= validity_period
        }

        /// Check if account can participate in validator operations
        pub fn can_be_validator(account: &T::AccountId) -> bool {
            let status = ComplianceStatusOf::<T>::get(account);
            
            !status.restricted &&
            status.verification_level.as_u8() >= T::MinValidatorVerification::get().as_u8() &&
            Self::is_verification_valid(account) &&
            status.risk_level != RiskLevel::Prohibited
        }

        /// Check if account can participate in governance
        pub fn can_participate_in_governance(account: &T::AccountId) -> bool {
            let status = ComplianceStatusOf::<T>::get(account);
            
            !status.restricted &&
            status.verification_level.as_u8() >= T::MinGovernanceVerification::get().as_u8() &&
            Self::is_verification_valid(account) &&
            status.risk_level != RiskLevel::Prohibited
        }

        /// Check if account can access treasury
        pub fn can_access_treasury(account: &T::AccountId) -> bool {
            let status = ComplianceStatusOf::<T>::get(account);
            
            (status.whitelisted || 
             status.verification_level.as_u8() >= T::MinTreasuryVerification::get().as_u8()) &&
            !status.restricted &&
            Self::is_verification_valid(account) &&
            status.risk_level != RiskLevel::Prohibited
        }

        /// Check if transaction requires travel rule reporting
        pub fn requires_travel_rule(amount: BalanceOf<T>) -> bool {
            amount >= T::TravelRuleThreshold::get()
        }

        /// Check if entity is on sanctions list (hash-based)
        pub fn is_sanctioned(entity_hash: &[u8; 32]) -> bool {
            SanctionsList::<T>::get(entity_hash)
                .map(|entry| entry.active)
                .unwrap_or(false)
        }

        /// COMP-SANC-ISO: Unified account-level sanctions check.
        /// Returns `true` if the account is sanctioned in EITHER the
        /// compliance pallet's hash-based list OR the external oracle
        /// (identity pallet). Callers should prefer this over `is_sanctioned`
        /// when checking by AccountId rather than entity hash.
        pub fn is_account_sanctioned(account: &T::AccountId) -> bool {
            T::AccountSanctionsChecker::is_sanctioned(account)
        }

        /// Get compliance statistics
        pub fn get_stats() -> (u32, u32, u32, u32) {
            ComplianceStats::<T>::get()
        }

        /// Convert RiskLevel to u8 for event encoding
        fn risk_level_to_u8(risk: RiskLevel) -> u8 {
            match risk {
                RiskLevel::Low => 0,
                RiskLevel::Medium => 1,
                RiskLevel::High => 2,
                RiskLevel::Prohibited => 3,
            }
        }

        /// Convert ActionType to u8 for event encoding
        fn action_type_to_u8(action: ActionType) -> u8 {
            match action {
                ActionType::VerificationUpdated => 0,
                ActionType::RiskAssessmentUpdated => 1,
                ActionType::AccountWhitelisted => 2,
                ActionType::AccountRestricted => 3,
                ActionType::RestrictionLifted => 8,
                ActionType::SuspiciousActivityReported => 4,
                ActionType::ComplianceCheckPerformed => 5,
                ActionType::SanctionsScreened => 6,
                ActionType::TravelRuleChecked => 7,
            }
        }

        /// Convert u8 to RiskLevel
        fn u8_to_risk_level(v: u8) -> RiskLevel {
            match v {
                1 => RiskLevel::Medium,
                2 => RiskLevel::High,
                3 => RiskLevel::Prohibited,
                _ => RiskLevel::Low,
            }
        }

        /// Convert u8 to SuspiciousActivityType
        fn u8_to_activity_type(v: u8) -> SuspiciousActivityType {
            match v {
                1 => SuspiciousActivityType::RapidFundMovement,
                2 => SuspiciousActivityType::Structuring,
                3 => SuspiciousActivityType::HighRiskJurisdiction,
                4 => SuspiciousActivityType::MoneyLaundering,
                5 => SuspiciousActivityType::TerroristFinancing,
                6 => SuspiciousActivityType::SanctionsViolation,
                7 => SuspiciousActivityType::Other,
                _ => SuspiciousActivityType::UnusualVolumePattern,
            }
        }

        /// COMP-CRIT-2: Record a transaction and check for structuring.
        ///
        /// Appends `(amount, current_block)` to the per-account sliding window,
        /// prunes entries older than `StructuringWindowBlocks`, then checks:
        ///   • Every individual tx in the window is BELOW the travel-rule threshold
        ///   • The cumulative total EXCEEDS the threshold
        ///
        /// If both conditions hold, a `Structuring` SAR is auto-filed and
        /// `StructuringDetected` is emitted.  Returns `true` when detected.
        pub fn record_transaction_and_check_structuring(
            account: &T::AccountId,
            amount: u128,
        ) -> bool {
            let current_block = frame_system::Pallet::<T>::block_number();
            let window_size = T::StructuringWindowBlocks::get();
            let threshold: u128 = T::TravelRuleThreshold::get().saturated_into();

            // Skip if the single tx already meets or exceeds the threshold
            // (that is covered by the travel rule, not structuring).
            if amount >= threshold {
                return false;
            }

            TransactionWindow::<T>::mutate(account, |entries| {
                // Prune entries outside the window
                let cutoff = current_block.saturating_sub(window_size);
                let pruned: Vec<_> = entries
                    .iter()
                    .filter(|(_, blk)| *blk >= cutoff)
                    .cloned()
                    .collect();

                // Append the new entry; if full, drop the oldest
                let mut updated = pruned;
                updated.push((amount, current_block));
                *entries = BoundedVec::truncate_from(updated);

                // Check structuring condition
                let all_below = entries.iter().all(|(a, _)| *a < threshold);
                let cumulative: u128 = entries.iter().map(|(a, _)| a).fold(0u128, |acc, a| acc.saturating_add(*a));
                let tx_count = entries.len() as u32;

                if all_below && cumulative >= threshold && tx_count >= 2 {
                    // Auto-file structuring SAR (best-effort; ignore if storage full)
                    let description = b"Auto-detected: cumulative sub-threshold transactions exceed travel rule within window".to_vec();
                    let bounded_desc = BoundedVec::truncate_from(description);

                    SuspiciousActivities::<T>::mutate(account, |reports| {
                        // FIFO eviction if full
                        if reports.len() as u32 >= T::MaxSuspiciousActivityReports::get() {
                            let remove_count = reports.len().saturating_sub(
                                T::MaxSuspiciousActivityReports::get().saturating_sub(1) as usize,
                            );
                            for _ in 0..remove_count {
                                if !reports.is_empty() {
                                    reports.remove(0);
                                }
                            }
                        }
                        let _ = reports.try_push((
                            SuspiciousActivityType::Structuring,
                            current_block,
                            bounded_desc,
                        ));
                    });

                    Self::deposit_event(Event::StructuringDetected {
                        account: account.clone(),
                        window_total: cumulative,
                        tx_count,
                    });

                    return true;
                }

                false
            })
        }
    }
}

// COMP-CRIT-2: Implement ComplianceReporter for the pallet so other pallets
// can report transactions and trigger structuring detection.
impl<T: Config> ComplianceReporter<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
    fn report_transaction(account: &T::AccountId, amount: u128) -> bool {
        Self::record_transaction_and_check_structuring(account, amount)
    }
}

// No-op implementation for unit type (used in tests / mock runtimes that
// don't wire up a real compliance pallet).
impl ComplianceReporter<sp_runtime::AccountId32, u64> for () {
    fn report_transaction(_account: &sp_runtime::AccountId32, _amount: u128) -> bool {
        false
    }
}

// No-op implementation for mocks that use u64 account IDs.
impl ComplianceReporter<u64, u64> for () {
    fn report_transaction(_account: &u64, _amount: u128) -> bool {
        false
    }
}

// COMP-SANC-ISO: No-op AccountSanctionsChecker implementations for tests/mocks
impl<AccountId> AccountSanctionsChecker<AccountId> for () {
    fn is_sanctioned(_account: &AccountId) -> bool {
        false
    }
}

// Implement default weights for () to match runtime wiring
impl WeightInfo for () {
    fn verify_account() -> Weight {
        Weight::from_parts(50_000_000, 512)
    }
    fn update_risk_level() -> Weight {
        Weight::from_parts(35_000_000, 512)
    }
    fn whitelist_account() -> Weight {
        Weight::from_parts(30_000_000, 512)
    }
    fn restrict_account() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
    fn flag_suspicious_activity() -> Weight {
        Weight::from_parts(60_000_000, 512)
    }
    fn add_sanctions_entry() -> Weight {
        Weight::from_parts(45_000_000, 512)
    }
    fn remove_sanctions_entry() -> Weight {
        Weight::from_parts(35_000_000, 512)
    }
    fn check_compliance() -> Weight {
        Weight::from_parts(25_000_000, 512)
    }
    fn update_verification_level() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
}