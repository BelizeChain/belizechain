#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

//! # BelizeChain Oracle Pallet
//!
//! Provides external data integration for price feeds, merchant verification,
//! sanctions compliance, and exchange rate management.

pub use pallet::*;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// BLOCKS_PER_YEAR removed — use T::ExpiryDurationBlocks instead (C-5 fix).

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::types::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Currency as CurrencyTrait;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{SaturatedConversion, Saturating};
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        /// Origin that can add/remove oracle operators
        type OracleAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // ── AR-7: Currency for oracle reward payouts ─────────────────────────
        /// The currency used to pay oracle operators.
        ///
        /// Rewards are transferred from the treasury account; operators must have
        /// accumulated at least 1 DALLA-equivalent in stats before claiming.
        type Currency: frame_support::traits::Currency<Self::AccountId>;

        /// Treasury account that funds oracle reward payouts (AR-7).
        ///
        /// In BelizeChain this is the main treasury, derived from `TreasuryPalletId`.
        type TreasuryAccount: frame_support::traits::Get<Self::AccountId>;

        /// Maximum number of oracle operators
        #[pallet::constant]
        type MaxOperators: Get<u32>;

        /// Maximum staleness for oracle data (in blocks)
        #[pallet::constant]
        type MaxDataStaleness: Get<BlockNumberFor<Self>>;

        /// Minimum number of operators required for consensus
        #[pallet::constant]
        type MinConsensusOperators: Get<u32>;

        /// Minimum distinct oracle operators that must submit matching KYC data
        /// before it is committed on-chain (Phase 3A — oracle plurality).
        /// Separate from MinConsensusOperators so identity requirements can be stricter.
        #[pallet::constant]
        type MinOracleAgreement: Get<u32>;

        /// Duration (blocks) of governance cooldown applied when a behavior flag
        /// reaches consensus (Phase 5A).  Default: 14,400 blocks ≈ 24 hours.
        #[pallet::constant]
        type BehaviorCooldownBlocks: Get<BlockNumberFor<Self>>;

        /// Duration (blocks) for KYC/merchant verification expiry.
        /// Default: 5,256,000 blocks ≈ 1 year at 6-second block time.
        #[pallet::constant]
        type ExpiryDurationBlocks: Get<BlockNumberFor<Self>>;

        /// Maximum allowed price deviation (basis points) from existing feed.
        /// Submissions deviating more than this from the current median are rejected.
        /// Default: 5000 = 50%.
        #[pallet::constant]
        type MaxPriceDeviation: Get<u32>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    // ===== STORAGE =====

    /// Authorized oracle operators
    #[pallet::storage]
    #[pallet::getter(fn oracle_operators)]
    pub type OracleOperators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    /// Tracked count of oracle operators (avoids O(n) iter().count())
    #[pallet::storage]
    #[pallet::getter(fn operator_count)]
    pub type OperatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Price submissions by operator for each currency pair
    #[pallet::storage]
    #[pallet::getter(fn price_submissions)]
    pub type PriceSubmissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CurrencyPair,
        Blake2_128Concat,
        T::AccountId,
        (u128, BlockNumberFor<T>), // (price, block_number)
        OptionQuery,
    >;

    /// Aggregated price feed data
    #[pallet::storage]
    #[pallet::getter(fn price_feeds)]
    pub type PriceFeeds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        CurrencyPair,
        PriceFeedData<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Manually set exchange rates by admin (fallback for DEX quoting)
    /// Stored as (price, last_update_block) with 6 decimal precision (1e6 scale)
    #[pallet::storage]
    #[pallet::getter(fn manual_exchange_rates)]
    pub type ManualExchangeRates<T: Config> =
        StorageMap<_, Blake2_128Concat, CurrencyPair, (u128, BlockNumberFor<T>), OptionQuery>;

    /// Verified merchants for tourism incentives
    #[pallet::storage]
    #[pallet::getter(fn merchant_categories)]
    pub type MerchantCategories<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        MerchantInfo<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Sanctioned entities
    #[pallet::storage]
    #[pallet::getter(fn sanctioned_entities)]
    pub type SanctionedEntities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SanctionInfo<BlockNumberFor<T>>, OptionQuery>;

    /// Identity verification data (KYC levels)
    #[pallet::storage]
    #[pallet::getter(fn identity_verifications)]
    pub type IdentityVerifications<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        IdentityInfo<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Land registry ownership data
    #[pallet::storage]
    #[pallet::getter(fn land_registry_data)]
    pub type LandRegistryData<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PropertyId,
        LandOwnershipInfo<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    // ===== IoT DATA ORACLE STORAGE =====

    /// Registered IoT devices
    #[pallet::storage]
    #[pallet::getter(fn iot_devices)]
    pub type IoTDevices<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // device_id
        IoTDevice<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Oracle operator contribution statistics
    #[pallet::storage]
    #[pallet::getter(fn oracle_operator_stats)]
    pub type OracleOperatorStatsMap<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        OracleOperatorStats<BlockNumberFor<T>>,
        OptionQuery,
    >;

    // ── Phase 3A: KYC oracle plurality ──────────────────────────────────────────

    /// Staging area: KYC submissions awaiting consensus.
    /// Outer key: subject account.  Inner key: oracle operator.
    /// Value: (kyc_level_u8, id_hash) — the specific attestation the oracle is vouching for.
    /// Cleared once MinOracleAgreement is reached or a dispute is resolved.
    #[pallet::storage]
    pub type PendingKycSubmissions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // subject
        Blake2_128Concat,
        T::AccountId, // oracle operator
        (u8, [u8; 32]),
        OptionQuery,
    >;

    /// Leading KYC vote state per subject: (kyc_level, id_hash, agreement_count).
    /// When agreement_count >= T::MinOracleAgreement, data is finalized and
    /// written to IdentityVerifications.
    #[pallet::storage]
    pub type KycLeadingVote<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (u8, [u8; 32], u32), OptionQuery>;

    /// Dispute flag — set when two oracle operators submit conflicting KYC data
    /// for the same subject.  Admin must call `resolve_kyc_dispute` before fresh
    /// votes are accepted for that subject.
    #[pallet::storage]
    pub type OracleDisputeFlag<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // ===== PHASE 5A: BEHAVIOR FLAG STORAGE =====

    /// Active behavior flags per account.
    /// Each flag is set only when `MinOracleAgreement` oracle operators agree.
    /// 0 = CompulsiveActivity, 1 = RewardLoopDetected, 2 = AutomatedActingPattern
    #[pallet::storage]
    pub type BehaviorFlags<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u8, // BehaviorFlag variant as u8
        OptionQuery,
    >;

    /// Block number at which an account's behavior-flag cooldown expires.
    /// After this block the account resumes normal governance participation.
    #[pallet::storage]
    pub type BehaviorFlagCooldown<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, OptionQuery>;

    /// Staged behavior-flag votes from oracle operators before consensus is reached.
    /// DoubleMap: (account, oracle_operator) → flag_type_u8
    #[pallet::storage]
    pub type PendingBehaviorFlags<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        u8,
        OptionQuery,
    >;

    // ===== GENESIS CONFIGURATION =

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub operators: Vec<T::AccountId>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                operators: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for operator in &self.operators {
                OracleOperators::<T>::insert(operator, true);
            }
        }
    }

    // ===== EVENTS =====

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Oracle operator added
        OperatorAdded { operator: T::AccountId },
        /// Oracle operator removed
        OperatorRemoved { operator: T::AccountId },
        /// Price submitted by operator
        PriceSubmitted {
            operator: T::AccountId,
            base_currency: u8,
            quote_currency: u8,
            price: u128,
        },
        /// Price feed updated (aggregated)
        PriceFeedUpdated {
            base_currency: u8,
            quote_currency: u8,
            price: u128,
            num_submissions: u32,
        },
        /// Merchant verified
        MerchantVerified {
            merchant: T::AccountId,
            category: u8,
        },
        /// Merchant verification expired
        MerchantExpired { merchant: T::AccountId },
        /// Sanctioned entity added
        SanctionAdded { account: T::AccountId, source: u8 },
        /// Sanction removed
        SanctionRemoved { account: T::AccountId },
        /// Identity verified
        IdentityVerified {
            account: T::AccountId,
            kyc_level: u8,
        },
        /// Identity verification expired
        IdentityExpired { account: T::AccountId },
        /// Land registry entry added
        LandRegistryUpdated {
            property_id: PropertyId,
            owner: T::AccountId,
        },
        // IoT Data Oracle Events
        /// IoT device registered
        IoTDeviceRegistered {
            device_id: [u8; 32],
            owner: T::AccountId,
        },
        /// IoT data submitted
        IoTDataSubmitted {
            device_id: [u8; 32],
            operator: T::AccountId,
            feed_type_index: u8,
            quality_score: u16,
        },
        /// IoT device verified by oracle
        IoTDeviceVerified {
            device_id: [u8; 32],
            operator: T::AccountId,
        },
        /// Oracle rewards claimed
        OracleRewardsClaimed {
            operator: T::AccountId,
            amount: u128,
        },
        /// Manual exchange rate updated by admin
        ExchangeRateUpdated {
            base_currency: u8,
            quote_currency: u8,
            price: u128,
        },
        /// Exchange rate is stale beyond configured max staleness
        ExchangeRateStale {
            base_currency: u8,
            quote_currency: u8,
        },
        /// KYC vote staged by oracle operator; waiting for consensus (Phase 3A).
        KycVoteStaged {
            subject: T::AccountId,
            oracle: T::AccountId,
            kyc_level: u8,
            votes_so_far: u32,
        },
        /// KYC oracle plurality reached; identity data committed on-chain (Phase 3A).
        KycConsensusReached {
            subject: T::AccountId,
            kyc_level: u8,
            oracle_count: u32,
        },
        /// Conflicting KYC submissions detected; data frozen pending admin resolution (Phase 3A).
        KycDisputeFlagged { subject: T::AccountId },
        /// KYC dispute resolved by admin; fresh oracle submissions now accepted (Phase 3A).
        KycDisputeResolved { subject: T::AccountId },

        // ── Phase 5A: Behavior flags ──────────────────────────────────────────
        /// Oracle operator staged a behavior-flag vote; waiting for consensus (Phase 5A).
        BehaviorFlagVoteStaged {
            account: T::AccountId,
            oracle: T::AccountId,
            flag_type: u8,
            votes_so_far: u32,
        },
        /// Oracle plurality agreed on a behavior flag — governance cooldown applied (Phase 5A).
        /// flag_type: 0=CompulsiveActivity, 1=RewardLoopDetected, 2=AutomatedActingPattern
        BehaviorFlagged {
            account: T::AccountId,
            flag_type: u8,
            cooldown_until: BlockNumberFor<T>,
        },
        /// Behavior flag cleared by admin after manual review (Phase 5A).
        BehaviorFlagCleared { account: T::AccountId },
    }

    // ===== ERRORS =====

    #[pallet::error]
    pub enum Error<T> {
        /// Too many operators
        TooManyOperators,
        /// Operator already exists
        OperatorAlreadyExists,
        /// Operator not found
        OperatorNotFound,
        /// Not authorized as oracle operator
        NotAuthorizedOperator,
        /// Invalid price value
        InvalidPrice,
        /// Stale data (too old)
        StaleData,
        /// Merchant already verified
        MerchantAlreadyVerified,
        /// Merchant not found
        MerchantNotFound,
        /// Already sanctioned
        AlreadySanctioned,
        /// Not sanctioned
        NotSanctioned,
        /// Price feed not found
        PriceFeedNotFound,
        /// Invalid certification format
        InvalidCertification,
        /// Insufficient consensus (not enough operators)
        InsufficientConsensus,
        /// High price variance (suspicious data)
        HighVariance,
        /// Identity already verified
        IdentityAlreadyVerified,
        /// Identity verification expired
        IdentityVerificationExpired,
        /// Property already registered
        PropertyAlreadyRegistered,
        /// Property not found
        PropertyNotFound,
        /// Invalid KYC level
        InvalidKycLevel,
        /// KYC verification required
        KycVerificationRequired,
        // IoT Data Oracle Errors
        /// Device already registered
        DeviceAlreadyRegistered,
        /// Device not found
        DeviceNotFound,
        /// Not the device owner
        NotDeviceOwner,
        /// Invalid feed type
        InvalidFeedType,
        /// Invalid domain
        InvalidDomain,
        /// No stats found for operator
        NoStatsFound,
        /// No rewards available
        NoRewardsAvailable,
        /// Treasury account does not hold enough balance to pay the reward
        InsufficientTreasuryBalance,
        /// A KYC dispute is active for this subject; admin must call resolve_kyc_dispute first (Phase 3A).
        KycDisputeActive,
        /// This oracle already submitted identical KYC data for this subject (Phase 3A).
        AlreadyVotedSameKyc,
        /// No active KYC dispute to resolve (Phase 3A).
        NoKycDisputeActive,
        /// This oracle already submitted a behavior flag vote for this account (Phase 5A).
        AlreadyVotedBehaviorFlag,
        /// Behavior flag type is invalid (must be 0, 1, or 2) (Phase 5A).
        InvalidBehaviorFlagType,
        /// No active behavior flag to clear (Phase 5A).
        NoBehaviorFlagActive,
        /// Quality metric value exceeds maximum (must be 0-100).
        InvalidQualityMetric,
        /// Submitted price deviates too far from current feed median.
        PriceDeviationTooHigh,
        /// KYC id_hash must not be all-zeros.
        InvalidIdHash,
    }

    // ===== CALL FUNCTIONS (EXTRINSICS) =====

    // ===== EXTRINSICS =====

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add authorized oracle operator
        #[pallet::weight(T::WeightInfo::add_operator())]
        #[pallet::call_index(0)]
        pub fn add_operator(origin: OriginFor<T>, operator: T::AccountId) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(
                !OracleOperators::<T>::contains_key(&operator),
                Error::<T>::OperatorAlreadyExists
            );

            // O-2 FIX: Enforce MaxOperators cap using O(1) counter
            let count = OperatorCount::<T>::get();
            ensure!(count < T::MaxOperators::get(), Error::<T>::TooManyOperators);

            OracleOperators::<T>::insert(&operator, true);
            OperatorCount::<T>::put(count.saturating_add(1));

            Self::deposit_event(Event::OperatorAdded { operator });

            Ok(())
        }

        /// Remove oracle operator
        #[pallet::weight(T::WeightInfo::remove_operator())]
        #[pallet::call_index(1)]
        pub fn remove_operator(origin: OriginFor<T>, operator: T::AccountId) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(
                OracleOperators::<T>::contains_key(&operator),
                Error::<T>::OperatorNotFound
            );

            OracleOperators::<T>::remove(&operator);
            OperatorCount::<T>::mutate(|c| *c = c.saturating_sub(1));

            Self::deposit_event(Event::OperatorRemoved { operator });

            Ok(())
        }

        /// Submit price feed data (using u8 for currency to avoid DecodeWithMemTracking)
        // DOS-009 FIX: Operational dispatch — oracle price feeds must get
        // through during block congestion.
        #[pallet::weight((T::WeightInfo::submit_price(), DispatchClass::Operational))]
        #[pallet::call_index(2)]
        pub fn submit_price(
            origin: OriginFor<T>,
            base_currency: u8,
            quote_currency: u8,
            price: u128,
        ) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            ensure!(
                OracleOperators::<T>::get(&operator),
                Error::<T>::NotAuthorizedOperator
            );

            ensure!(price > 0, Error::<T>::InvalidPrice);

            // C-3 FIX: Reject submissions that deviate too far from existing feed
            let base = Self::u8_to_currency(base_currency)?;
            let quote = Self::u8_to_currency(quote_currency)?;
            let pair = CurrencyPair { base, quote };

            if let Some(feed) = PriceFeeds::<T>::get(pair) {
                if feed.price > 0 {
                    let max_deviation = T::MaxPriceDeviation::get() as u128;
                    let deviation = if price > feed.price {
                        (price - feed.price).saturating_mul(10000) / feed.price
                    } else {
                        (feed.price - price).saturating_mul(10000) / feed.price
                    };
                    ensure!(
                        deviation <= max_deviation,
                        Error::<T>::PriceDeviationTooHigh
                    );
                }
            }

            let current_block = frame_system::Pallet::<T>::block_number();

            // Store the submission
            PriceSubmissions::<T>::insert(pair, &operator, (price, current_block));

            // Aggregate all submissions
            Self::aggregate_price_feed(pair)?;

            Self::deposit_event(Event::PriceSubmitted {
                operator,
                base_currency,
                quote_currency,
                price,
            });

            Ok(())
        }

        /// Verify merchant for tourism incentives (using u8 for category)
        #[pallet::weight(T::WeightInfo::verify_merchant())]
        #[pallet::call_index(3)]
        pub fn verify_merchant(
            origin: OriginFor<T>,
            merchant: T::AccountId,
            category: u8,
            certification: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
            license: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
            location: Option<(i32, i32)>,
        ) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            ensure!(
                OracleOperators::<T>::get(&operator),
                Error::<T>::NotAuthorizedOperator
            );

            // Convert u8 to MerchantCategory
            let category = Self::u8_to_merchant_category(category)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let expiry_block = current_block.saturating_add(T::ExpiryDurationBlocks::get());

            let merchant_info = MerchantInfo {
                merchant: merchant.clone(),
                category,
                certification,
                license,
                location,
                verified_at: current_block,
                expires_at: expiry_block,
            };

            MerchantCategories::<T>::insert(&merchant, merchant_info);

            Self::deposit_event(Event::MerchantVerified {
                merchant,
                category: category.to_u8(),
            });

            Ok(())
        }

        /// Add sanctioned entity (using u8 for source)
        #[pallet::weight(T::WeightInfo::add_sanction())]
        #[pallet::call_index(4)]
        pub fn add_sanctioned_entity(
            origin: OriginFor<T>,
            account: T::AccountId,
            source: u8,
            reason: BoundedVec<u8, ConstU32<MAX_REASON_LEN>>,
            expires_at: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            // Convert u8 to SanctionSource
            let source = Self::u8_to_sanction_source(source)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            let sanction_info = SanctionInfo {
                source,
                added_at: current_block,
                reason,
                active: true,
                expires_at,
            };

            SanctionedEntities::<T>::insert(&account, sanction_info);

            Self::deposit_event(Event::SanctionAdded {
                account,
                source: source.to_u8(),
            });

            Ok(())
        }

        /// Remove sanction
        #[pallet::weight(T::WeightInfo::remove_sanction())]
        #[pallet::call_index(5)]
        pub fn remove_sanction(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(
                SanctionedEntities::<T>::contains_key(&account),
                Error::<T>::NotSanctioned
            );

            SanctionedEntities::<T>::remove(&account);

            Self::deposit_event(Event::SanctionRemoved { account });

            Ok(())
        }

        /// Verify identity with KYC level
        #[pallet::weight(T::WeightInfo::verify_identity())]
        #[pallet::call_index(6)]
        pub fn verify_identity(
            origin: OriginFor<T>,
            account: T::AccountId,
            kyc_level: u8,
            id_hash: [u8; 32],
            provider: BoundedVec<u8, ConstU32<64>>,
            biometric_verified: bool,
            address_verified: bool,
        ) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            // Verify caller is an authorized oracle operator
            ensure!(
                OracleOperators::<T>::get(&operator),
                Error::<T>::NotAuthorizedOperator
            );

            // Phase 3A: reject if a dispute is active for this subject
            ensure!(
                !OracleDisputeFlag::<T>::get(&account),
                Error::<T>::KycDisputeActive
            );

            // C-2 FIX: Validate id_hash is non-zero and kyc_level is in valid range
            ensure!(id_hash != [0u8; 32], Error::<T>::InvalidIdHash);
            Self::u8_to_kyc_level(kyc_level)?;

            // Reject duplicate vote from same oracle for same (kyc_level, id_hash)
            let key = (kyc_level, id_hash);
            if let Some(prev) = PendingKycSubmissions::<T>::get(&account, &operator) {
                ensure!(prev != key, Error::<T>::AlreadyVotedSameKyc);
                // Oracle is changing its vote — remove old submission before re-staging
                PendingKycSubmissions::<T>::remove(&account, &operator);
            }

            // Stage the vote
            PendingKycSubmissions::<T>::insert(&account, &operator, key);

            // Update the leading-vote tally
            let min_agreement = T::MinOracleAgreement::get();
            let (new_count, finalize) = KycLeadingVote::<T>::mutate_exists(&account, |entry| {
                match entry {
                    None => {
                        // First submission — becomes the leader
                        *entry = Some((kyc_level, id_hash, 1u32));
                        (1u32, false)
                    }
                    Some((lkl, lhash, count)) if *lkl == kyc_level && *lhash == id_hash => {
                        // Matches leader — increment
                        *count = count.saturating_add(1);
                        let c = *count;
                        (c, c >= min_agreement)
                    }
                    Some(_) => {
                        // Conflicts with leading vote — flag dispute
                        (0u32, false)
                    }
                }
            });

            // Detect dispute: mutate_exists returned 0 meaning conflict branch was hit
            if new_count == 0 {
                OracleDisputeFlag::<T>::insert(&account, true);
                Self::deposit_event(Event::KycDisputeFlagged { subject: account });
                return Ok(());
            }

            Self::deposit_event(Event::KycVoteStaged {
                subject: account.clone(),
                oracle: operator,
                kyc_level,
                votes_so_far: new_count,
            });

            // Finalize when quorum is reached
            if finalize {
                let resolved_level = Self::u8_to_kyc_level(kyc_level)?;
                let current_block = frame_system::Pallet::<T>::block_number();
                let expiry_block = current_block.saturating_add(T::ExpiryDurationBlocks::get());

                let identity_info = IdentityInfo {
                    account: account.clone(),
                    kyc_level: resolved_level,
                    id_hash,
                    provider,
                    verified_at: current_block,
                    expires_at: expiry_block,
                    biometric_verified,
                    address_verified,
                };

                IdentityVerifications::<T>::insert(&account, identity_info);

                // Clear staging storage
                let _ = PendingKycSubmissions::<T>::clear_prefix(&account, u32::MAX, None);
                KycLeadingVote::<T>::remove(&account);

                Self::deposit_event(Event::KycConsensusReached {
                    subject: account.clone(),
                    kyc_level,
                    oracle_count: new_count,
                });
                Self::deposit_event(Event::IdentityVerified { account, kyc_level });
            }

            Ok(())
        }

        /// Register land ownership
        #[pallet::weight(T::WeightInfo::register_land())]
        #[pallet::call_index(7)]
        pub fn register_land(
            origin: OriginFor<T>,
            property_id: PropertyId,
            owner: T::AccountId,
            valuation: u128,
            has_encumbrances: bool,
            co_owner_count: u8,
        ) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            // Verify caller is an authorized oracle operator
            ensure!(
                OracleOperators::<T>::get(&operator),
                Error::<T>::NotAuthorizedOperator
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // O-4 FIX: Prevent overwriting existing property records
            ensure!(
                !LandRegistryData::<T>::contains_key(property_id),
                Error::<T>::PropertyAlreadyRegistered
            );

            let land_info = LandOwnershipInfo {
                property_id,
                owner: owner.clone(),
                valuation,
                last_transfer: current_block,
                has_encumbrances,
                co_owner_count,
                verified: true,
                verified_at: current_block,
            };

            LandRegistryData::<T>::insert(property_id, land_info);

            Self::deposit_event(Event::LandRegistryUpdated { property_id, owner });

            Ok(())
        }

        // ===== IoT DATA ORACLE EXTRINSICS (Phase 1: Nawal Expansion) =====

        /// Register an IoT device for data submission
        /// device_type_index: 0=Drone, 1=PhoneSensor, 2=IoTSensor, 3=WeatherStation,
        ///                    4=AgriculturalSensor, 5=MarineBuoy, 6=Camera, 7=Other
        #[pallet::weight(T::WeightInfo::register_iot_device())]
        #[pallet::call_index(8)]
        pub fn register_iot_device(
            origin: OriginFor<T>,
            device_id: [u8; 32],
            device_type_index: u8,
            location: Option<(i32, i32)>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(
                !IoTDevices::<T>::contains_key(device_id),
                Error::<T>::DeviceAlreadyRegistered
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // Convert index to DeviceType (simplified, default specs)
            let device_type = match device_type_index {
                0 => DeviceType::Drone(DroneSpec {
                    model: BoundedVec::default(),
                    camera_resolution: 0,
                    multispectral: false,
                    max_altitude: 0,
                    flight_time: 0,
                }),
                1 => DeviceType::PhoneSensor(PhoneSpec {
                    os: BoundedVec::default(),
                    has_camera: true,
                    has_gps: true,
                    has_accelerometer: true,
                }),
                2 => DeviceType::IoTSensor(SensorSpec {
                    sensor_type: SensorType::Temperature,
                    accuracy: 0,
                    sampling_rate: 0,
                }),
                3 => DeviceType::WeatherStation,
                4 => DeviceType::AgriculturalSensor,
                5 => DeviceType::MarineBuoy,
                6 => DeviceType::Camera,
                7 => DeviceType::Other,
                _ => return Err(Error::<T>::InvalidFeedType.into()),
            };

            let device = IoTDevice {
                device_id,
                device_type,
                owner: owner.clone(),
                location,
                registered_at: current_block,
                last_active: current_block,
                reputation_score: 5000, // Start at 50%
                data_submissions: 0,
                verified: false,
            };

            IoTDevices::<T>::insert(device_id, device);

            Self::deposit_event(Event::IoTDeviceRegistered { device_id, owner });

            Ok(())
        }

        /// Submit IoT data feed
        #[pallet::weight(T::WeightInfo::submit_iot_data())]
        #[pallet::call_index(9)]
        pub fn submit_iot_data(
            origin: OriginFor<T>,
            device_id: [u8; 32],
            feed_type_index: u8,
            domain_index: Option<u8>,
            data: BoundedVec<u8, ConstU32<MAX_DATA_LEN>>,
            data_hash: [u8; 32],
            location: Option<(i32, i32)>,
            accuracy: u8,
            timeliness: u8,
            completeness: u8,
            consistency: u8,
            provenance: u8,
        ) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            // Verify device exists and belongs to operator
            let mut device = IoTDevices::<T>::get(device_id).ok_or(Error::<T>::DeviceNotFound)?;

            ensure!(device.owner == operator, Error::<T>::NotDeviceOwner);

            // Quality metrics are documented as 0-100 scale; reject out-of-range values
            ensure!(
                accuracy <= 100
                    && timeliness <= 100
                    && completeness <= 100
                    && consistency <= 100
                    && provenance <= 100,
                Error::<T>::InvalidQualityMetric
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // Convert indices to enums
            let feed_type = Self::u8_to_feed_type(feed_type_index)?;
            let domain = if let Some(idx) = domain_index {
                Some(Self::u8_to_domain(idx)?)
            } else {
                None
            };

            // Construct quality metrics from individual params
            let quality_metrics = DataQualityMetrics {
                accuracy,
                timeliness,
                completeness,
                consistency,
                provenance,
            };

            // Create submission record (NOTE: not stored currently, could be added to storage later)
            let _submission = IoTDataSubmission {
                device_id,
                operator: operator.clone(),
                feed_type,
                domain,
                data,
                data_hash,
                timestamp: current_block,
                quality: quality_metrics,
                location,
            };

            // Update device stats
            device.last_active = current_block;
            device.data_submissions = device.data_submissions.saturating_add(1);

            // Update reputation based on quality score
            let quality_score = quality_metrics.calculate_score();
            Self::update_device_reputation(&mut device, quality_score);

            IoTDevices::<T>::insert(device_id, device);

            // Update operator stats
            Self::update_operator_stats(&operator, domain, quality_score, current_block)?;

            Self::deposit_event(Event::IoTDataSubmitted {
                device_id,
                operator,
                feed_type_index,
                quality_score,
            });

            Ok(())
        }

        /// Verify IoT device (oracle operator function)
        #[pallet::weight(T::WeightInfo::verify_iot_device())]
        #[pallet::call_index(10)]
        pub fn verify_iot_device(origin: OriginFor<T>, device_id: [u8; 32]) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            ensure!(
                OracleOperators::<T>::get(&operator),
                Error::<T>::NotAuthorizedOperator
            );

            let mut device = IoTDevices::<T>::get(device_id).ok_or(Error::<T>::DeviceNotFound)?;

            device.verified = true;
            IoTDevices::<T>::insert(device_id, device);

            Self::deposit_event(Event::IoTDeviceVerified {
                device_id,
                operator,
            });

            Ok(())
        }

        /// Claim oracle rewards for data contributions (AR-7)
        ///
        /// Calculates the operator's earned DALLA from quality, volume, and uptime
        /// scores, then transfers the amount from the treasury.  On success the
        /// per-operator statistics counters are reset so the same work cannot be
        /// double-claimed.
        ///
        /// # Errors
        ///
        /// - `NoStatsFound` — caller has never submitted oracle data.
        /// - `NoRewardsAvailable` — calculated reward rounds to zero.
        /// - `InsufficientTreasuryBalance` — treasury cannot cover the reward.
        #[pallet::weight(T::WeightInfo::claim_oracle_rewards())]
        #[pallet::call_index(11)]
        pub fn claim_oracle_rewards(origin: OriginFor<T>) -> DispatchResult {
            let operator = ensure_signed(origin)?;

            let stats =
                OracleOperatorStatsMap::<T>::get(&operator).ok_or(Error::<T>::NoStatsFound)?;

            // Calculate rewards based on contributions
            let reward_u128 = Self::calculate_oracle_reward(&stats);
            ensure!(reward_u128 > 0, Error::<T>::NoRewardsAvailable);

            // ── AR-7: Execute the actual DALLA transfer ───────────────────────
            //
            // Convert u128 reward to the Currency::Balance type.
            // `saturated_into` is safe here: Balance is also u128-backed in
            // BelizeChain so there is no information loss.
            let reward: <T::Currency as frame_support::traits::Currency<T::AccountId>>::Balance =
                reward_u128.saturated_into();

            let treasury = T::TreasuryAccount::get();
            let treasury_balance = T::Currency::free_balance(&treasury);

            ensure!(
                treasury_balance >= reward,
                Error::<T>::InsufficientTreasuryBalance
            );

            T::Currency::transfer(
                &treasury,
                &operator,
                reward,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // Reset stats to prevent double-claiming the same work.
            // Active status and last_active timestamp are preserved so the
            // operator can continue accumulating rewards immediately.
            OracleOperatorStatsMap::<T>::mutate(&operator, |maybe_stats| {
                if let Some(s) = maybe_stats {
                    s.total_submissions = 0;
                    s.agritech_submissions = 0;
                    s.marine_submissions = 0;
                    s.education_submissions = 0;
                    s.tech_submissions = 0;
                    s.general_submissions = 0;
                    s.avg_quality_score = 0;
                    s.uptime_percentage = 0;
                }
            });

            Self::deposit_event(Event::OracleRewardsClaimed {
                operator,
                amount: reward_u128,
            });

            Ok(())
        }

        /// Update manual exchange rate (admin origin)
        /// Price uses 6 decimal precision (scaled by 1e6)
        #[pallet::weight(T::WeightInfo::update_exchange_rate())]
        #[pallet::call_index(12)]
        pub fn update_exchange_rate(
            origin: OriginFor<T>,
            base_currency: u8,
            quote_currency: u8,
            price: u128,
        ) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(price > 0, Error::<T>::InvalidPrice);

            let base = Self::u8_to_currency(base_currency)?;
            let quote = Self::u8_to_currency(quote_currency)?;
            let pair = CurrencyPair { base, quote };

            let current_block = frame_system::Pallet::<T>::block_number();
            ManualExchangeRates::<T>::insert(pair, (price, current_block));

            Self::deposit_event(Event::ExchangeRateUpdated {
                base_currency,
                quote_currency,
                price,
            });

            Ok(())
        }

        /// Resolve a KYC oracle dispute for the given subject account (Phase 3A).
        ///
        /// Clears the dispute flag and all pending KYC votes so fresh oracle
        /// submissions can restart the consensus process.  Only callable by
        /// `OracleAdminOrigin` (governance council).
        #[pallet::weight(T::WeightInfo::resolve_kyc_dispute())]
        #[pallet::call_index(13)]
        pub fn resolve_kyc_dispute(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(
                OracleDisputeFlag::<T>::get(&account),
                Error::<T>::NoKycDisputeActive
            );

            OracleDisputeFlag::<T>::remove(&account);
            KycLeadingVote::<T>::remove(&account);
            let _ = PendingKycSubmissions::<T>::clear_prefix(&account, u32::MAX, None);

            Self::deposit_event(Event::KycDisputeResolved { subject: account });

            Ok(())
        }

        // ===== PHASE 5A: BEHAVIOR FLAG EXTRINSICS =====

        /// Submit a behavior-flag vote for an account via oracle consensus.
        ///
        /// Each oracle operator submits their assessment.  Once `MinOracleAgreement`
        /// operators agree on the same flag type, the flag becomes active and a
        /// governance-cooldown of `BehaviorCooldownBlocks` is applied.
        ///
        /// Flag types (flag_type_u8):
        ///   0 = CompulsiveActivity    — >50 governance actions in 24 h
        ///   1 = RewardLoopDetected    — variable-reward exploitation pattern
        ///   2 = AutomatedActingPattern — bot-like distinct from human variance
        ///
        /// ## Safety
        /// Multi-oracle consensus (MinOracleAgreement) prevents unilateral Nawal suppression.
        #[pallet::weight(Weight::from_parts(15_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2)))]
        #[pallet::call_index(14)]
        pub fn submit_behavior_flag(
            origin: OriginFor<T>,
            account: T::AccountId,
            flag_type_u8: u8,
        ) -> DispatchResult {
            let oracle = ensure_signed(origin)?;

            // Only registered oracle operators may flag
            ensure!(
                OracleOperators::<T>::contains_key(&oracle),
                Error::<T>::NotAuthorizedOperator
            );

            // Validate flag type (0-2 are valid)
            ensure!(flag_type_u8 <= 2, Error::<T>::InvalidBehaviorFlagType);

            // Prevent duplicate vote from same operator
            ensure!(
                !PendingBehaviorFlags::<T>::contains_key(&account, &oracle),
                Error::<T>::AlreadyVotedBehaviorFlag
            );

            // Record this operator's vote
            PendingBehaviorFlags::<T>::insert(&account, &oracle, flag_type_u8);

            // Count votes matching this flag type (bounded by MaxOperators)
            let max_ops = T::MaxOperators::get() as usize;
            let agreeing_votes: u32 = PendingBehaviorFlags::<T>::iter_prefix(&account)
                .take(max_ops)
                .filter(|(_, f)| *f == flag_type_u8)
                .count() as u32;

            let threshold = T::MinOracleAgreement::get();

            Self::deposit_event(Event::BehaviorFlagVoteStaged {
                account: account.clone(),
                oracle,
                flag_type: flag_type_u8,
                votes_so_far: agreeing_votes,
            });

            if agreeing_votes >= threshold {
                // Consensus reached — activate flag
                BehaviorFlags::<T>::insert(&account, flag_type_u8);

                // Set cooldown end block (~24 hours at 6 s/block = 14,400 blocks)
                let cooldown_end = frame_system::Pallet::<T>::block_number()
                    .saturating_add(T::BehaviorCooldownBlocks::get());
                BehaviorFlagCooldown::<T>::insert(&account, cooldown_end);

                // Clear pending votes
                let _ = PendingBehaviorFlags::<T>::clear_prefix(&account, u32::MAX, None);

                Self::deposit_event(Event::BehaviorFlagged {
                    account,
                    flag_type: flag_type_u8,
                    cooldown_until: cooldown_end,
                });
            }

            Ok(())
        }

        /// Clear a behavior flag after manual admin review.
        ///
        /// Callable only by `OracleAdminOrigin`.  Used when Nawal oracle assessment
        /// is disputed or when sufficient time and rehabilitation has occurred.
        #[pallet::weight(Weight::from_parts(10_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2)))]
        #[pallet::call_index(15)]
        pub fn clear_behavior_flag(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            T::OracleAdminOrigin::ensure_origin(origin)?;

            ensure!(
                BehaviorFlags::<T>::contains_key(&account),
                Error::<T>::NoBehaviorFlagActive
            );

            BehaviorFlags::<T>::remove(&account);
            BehaviorFlagCooldown::<T>::remove(&account);
            let _ = PendingBehaviorFlags::<T>::clear_prefix(&account, u32::MAX, None);

            Self::deposit_event(Event::BehaviorFlagCleared { account });

            Ok(())
        }
    }

    // ===== INTERNAL FUNCTIONS =====

    impl<T: Config> Pallet<T> {
        /// Convert u8 to Currency
        fn u8_to_currency(value: u8) -> Result<Currency, DispatchError> {
            match value {
                0 => Ok(Currency::BZD),
                1 => Ok(Currency::USD),
                2 => Ok(Currency::EUR),
                3 => Ok(Currency::CAD),
                4 => Ok(Currency::MXN),
                _ => Err(Error::<T>::InvalidPrice.into()),
            }
        }

        /// Convert u8 to MerchantCategory
        fn u8_to_merchant_category(value: u8) -> Result<MerchantCategory, DispatchError> {
            match value {
                0 => Ok(MerchantCategory::Accommodation),
                1 => Ok(MerchantCategory::FoodBeverage),
                2 => Ok(MerchantCategory::TourOperator),
                3 => Ok(MerchantCategory::Transportation),
                4 => Ok(MerchantCategory::Retail),
                5 => Ok(MerchantCategory::Other),
                _ => Err(Error::<T>::InvalidCertification.into()),
            }
        }

        /// Convert u8 to SanctionSource
        fn u8_to_sanction_source(value: u8) -> Result<SanctionSource, DispatchError> {
            match value {
                0 => Ok(SanctionSource::OFAC),
                1 => Ok(SanctionSource::UN),
                2 => Ok(SanctionSource::EU),
                3 => Ok(SanctionSource::FSC),
                4 => Ok(SanctionSource::Other),
                _ => Err(Error::<T>::InvalidCertification.into()),
            }
        }

        /// Convert u8 to KycLevel
        fn u8_to_kyc_level(value: u8) -> Result<KycLevel, DispatchError> {
            match value {
                0 => Ok(KycLevel::None),
                1 => Ok(KycLevel::Basic),
                2 => Ok(KycLevel::Enhanced),
                3 => Ok(KycLevel::Full),
                _ => Err(Error::<T>::InvalidKycLevel.into()),
            }
        }

        /// Aggregate price submissions from multiple operators
        fn aggregate_price_feed(pair: CurrencyPair) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let max_staleness = T::MaxDataStaleness::get();

            // Collect all recent submissions
            let mut submissions: Vec<u128> = Vec::new();

            // SECURITY: Bound iteration to MaxOperators to prevent DoS (H-29)
            let max_ops = T::MaxOperators::get() as usize;
            for (operator, _) in OracleOperators::<T>::iter().take(max_ops) {
                if let Some((price, block)) = PriceSubmissions::<T>::get(pair, &operator) {
                    // Only include fresh data
                    if current_block.saturating_sub(block) <= max_staleness {
                        submissions.push(price);
                    }
                }
            }

            // Require minimum consensus
            ensure!(
                submissions.len() as u32 >= T::MinConsensusOperators::get(),
                Error::<T>::InsufficientConsensus
            );

            // Calculate median price
            submissions.sort_unstable();
            let median_price = if submissions.len().is_multiple_of(2) {
                let mid = submissions.len() / 2;
                let a = submissions[mid - 1];
                let b = submissions[mid];
                // Overflow-safe average: a/2 + b/2 + (a%2 + b%2)/2
                a / 2 + b / 2 + (a % 2 + b % 2) / 2
            } else {
                submissions[submissions.len() / 2]
            };

            // Calculate variance (max - min as percentage of median, in basis points)
            let min_price = submissions[0];
            let max_price = submissions[submissions.len() - 1];
            let variance = if median_price > 0 {
                (max_price - min_price).saturating_mul(10000) / median_price
            } else {
                0u128
            };

            let feed_data = PriceFeedData {
                pair,
                price: median_price,
                num_submissions: submissions.len() as u32,
                last_update: current_block,
                variance: variance.min(u32::MAX as u128) as u32,
            };

            PriceFeeds::<T>::insert(pair, feed_data);

            Self::deposit_event(Event::PriceFeedUpdated {
                base_currency: pair.base.to_u8(),
                quote_currency: pair.quote.to_u8(),
                price: median_price,
                num_submissions: submissions.len() as u32,
            });

            Ok(())
        }

        /// Check if account is sanctioned
        pub fn is_sanctioned(account: &T::AccountId) -> bool {
            if let Some(sanction) = SanctionedEntities::<T>::get(account) {
                if !sanction.active {
                    return false;
                }

                // Check expiry
                if let Some(expires_at) = sanction.expires_at {
                    let current_block = frame_system::Pallet::<T>::block_number();
                    if current_block > expires_at {
                        return false;
                    }
                }

                return true;
            }
            false
        }

        // ── Phase 5A: Behavior flag public API ──────────────────────────────

        /// Returns `true` if `account` has an active behavior flag and its
        /// cooldown period has not yet expired.  Used by governance and economy
        /// pallets as an addiction-loop circuit breaker.
        pub fn has_active_behavior_flag(account: &T::AccountId) -> bool {
            if !BehaviorFlags::<T>::contains_key(account) {
                return false;
            }
            // If cooldown has expired, treat the flag as lapsed
            if let Some(cooldown_end) = BehaviorFlagCooldown::<T>::get(account) {
                let now = frame_system::Pallet::<T>::block_number();
                return now < cooldown_end;
            }
            false
        }

        /// Returns the block at which the behavior-flag cooldown ends, if active.
        pub fn behavior_cooldown_end(account: &T::AccountId) -> Option<BlockNumberFor<T>> {
            let end = BehaviorFlagCooldown::<T>::get(account)?;
            let now = frame_system::Pallet::<T>::block_number();
            if now < end {
                Some(end)
            } else {
                None
            }
        }

        /// Get current exchange rate for currency pair
        /// O-4 FIX: Corrected brace alignment for stale rate diagnostics
        pub fn get_exchange_rate(pair: CurrencyPair) -> Option<u128> {
            let current_block = frame_system::Pallet::<T>::block_number();
            let max_staleness = T::MaxDataStaleness::get();

            // Prefer manual override if fresh
            if let Some((price, last_update)) = ManualExchangeRates::<T>::get(pair) {
                if current_block.saturating_sub(last_update) <= max_staleness {
                    return Some(price);
                } else {
                    // Emit diagnostics for stale manual rate
                    Self::deposit_event(Event::ExchangeRateStale {
                        base_currency: pair.base.to_u8(),
                        quote_currency: pair.quote.to_u8(),
                    });
                }
            }

            // Fall back to aggregated price feed
            if let Some(feed) = PriceFeeds::<T>::get(pair) {
                if current_block.saturating_sub(feed.last_update) <= max_staleness {
                    return Some(feed.price);
                } else {
                    // Emit diagnostics for stale aggregated feed
                    Self::deposit_event(Event::ExchangeRateStale {
                        base_currency: pair.base.to_u8(),
                        quote_currency: pair.quote.to_u8(),
                    });
                }
            }
            None
        }

        /// Get merchant category
        pub fn get_merchant_category(merchant: &T::AccountId) -> Option<MerchantCategory> {
            if let Some(info) = MerchantCategories::<T>::get(merchant) {
                let current_block = frame_system::Pallet::<T>::block_number();

                // Check if verification is still valid
                if current_block <= info.expires_at {
                    return Some(info.category);
                }
            }
            None
        }

        /// Get merchant reward rate
        pub fn get_merchant_reward_rate(merchant: &T::AccountId) -> Option<u8> {
            Self::get_merchant_category(merchant).map(|cat| cat.reward_percentage())
        }

        /// Get KYC verification level
        pub fn get_kyc_level(account: &T::AccountId) -> Option<KycLevel> {
            if let Some(info) = IdentityVerifications::<T>::get(account) {
                let current_block = frame_system::Pallet::<T>::block_number();

                // Check if verification is still valid
                if current_block <= info.expires_at {
                    return Some(info.kyc_level);
                }
            }
            None
        }

        /// Check if account meets minimum KYC level requirement
        pub fn meets_kyc_requirement(account: &T::AccountId, required_level: KycLevel) -> bool {
            if let Some(actual_level) = Self::get_kyc_level(account) {
                return actual_level.to_u8() >= required_level.to_u8();
            }
            false
        }

        /// Get transaction limit for account based on KYC level
        pub fn get_transaction_limit(account: &T::AccountId) -> u128 {
            Self::get_kyc_level(account)
                .map(|level| level.transaction_limit())
                .unwrap_or(KycLevel::None.transaction_limit())
        }

        /// Get daily limit for account based on KYC level
        pub fn get_daily_limit(account: &T::AccountId) -> u128 {
            Self::get_kyc_level(account)
                .map(|level| level.daily_limit())
                .unwrap_or(KycLevel::None.daily_limit())
        }

        /// Get land ownership info
        pub fn get_land_ownership(
            property_id: PropertyId,
        ) -> Option<LandOwnershipInfo<T::AccountId, BlockNumberFor<T>>> {
            LandRegistryData::<T>::get(property_id)
        }

        /// Verify land ownership for account
        pub fn verify_land_owner(property_id: PropertyId, account: &T::AccountId) -> bool {
            if let Some(info) = LandRegistryData::<T>::get(property_id) {
                return info.owner == *account;
            }
            false
        }

        // ===== IoT HELPER FUNCTIONS =====

        /// Convert u8 to ModelDomain
        fn u8_to_domain(value: u8) -> Result<ModelDomain, DispatchError> {
            match value {
                0 => Ok(ModelDomain::General),
                1 => Ok(ModelDomain::AgriTech),
                2 => Ok(ModelDomain::Marine),
                3 => Ok(ModelDomain::Education),
                4 => Ok(ModelDomain::Tech),
                _ => Err(Error::<T>::InvalidDomain.into()),
            }
        }

        /// Convert u8 to DataFeedType (simplified mapping)
        fn u8_to_feed_type(value: u8) -> Result<DataFeedType, DispatchError> {
            match value {
                0 => Ok(DataFeedType::PriceFeed),
                1 => Ok(DataFeedType::MerchantVerification),
                2 => Ok(DataFeedType::SanctionsCheck),
                3 => Ok(DataFeedType::LandRegistry),
                4 => Ok(DataFeedType::IdentityVerification),
                5 => Ok(DataFeedType::WeatherData),
                6 => Ok(DataFeedType::GPSTracking),
                7 => Ok(DataFeedType::CameraFeed),
                // Domain-specific feeds require additional domain parameter
                10 => Ok(DataFeedType::DroneImagery(ModelDomain::General)),
                11 => Ok(DataFeedType::SensorReading(SensorType::Temperature)),
                12 => Ok(DataFeedType::PhoneCollection(CollectionType::Photo)),
                _ => Err(Error::<T>::InvalidFeedType.into()),
            }
        }

        /// Update device reputation based on quality score
        fn update_device_reputation(
            device: &mut IoTDevice<T::AccountId, BlockNumberFor<T>>,
            quality_score: u16,
        ) {
            // Exponential moving average: new_rep = 0.9 * old_rep + 0.1 * quality
            let old_rep = device.reputation_score as u32;
            let new_quality = quality_score as u32;
            let new_rep = (old_rep * 9 + new_quality) / 10;
            device.reputation_score = new_rep.min(10000) as u16;
        }

        /// Update operator stats with new submission
        fn update_operator_stats(
            operator: &T::AccountId,
            domain: Option<ModelDomain>,
            quality_score: u16,
            current_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            OracleOperatorStatsMap::<T>::mutate(operator, |maybe_stats| {
                let mut stats = maybe_stats.take().unwrap_or(OracleOperatorStats {
                    total_submissions: 0,
                    avg_quality_score: 0,
                    agritech_submissions: 0,
                    marine_submissions: 0,
                    education_submissions: 0,
                    tech_submissions: 0,
                    general_submissions: 0,
                    uptime_percentage: 10000, // Start at 100%
                    last_active: current_block,
                    total_rewards: 0,
                });

                // Update submission count
                stats.total_submissions = stats.total_submissions.saturating_add(1);

                // Update average quality score (moving average)
                let old_avg = stats.avg_quality_score as u32;
                let new_score = quality_score as u32;
                stats.avg_quality_score =
                    ((old_avg * (stats.total_submissions.saturating_sub(1)) as u32 + new_score)
                        / stats.total_submissions as u32) as u16;

                // Update domain-specific counts
                if let Some(d) = domain {
                    match d {
                        ModelDomain::AgriTech => {
                            stats.agritech_submissions =
                                stats.agritech_submissions.saturating_add(1)
                        }
                        ModelDomain::Marine => {
                            stats.marine_submissions = stats.marine_submissions.saturating_add(1)
                        }
                        ModelDomain::Education => {
                            stats.education_submissions =
                                stats.education_submissions.saturating_add(1)
                        }
                        ModelDomain::Tech => {
                            stats.tech_submissions = stats.tech_submissions.saturating_add(1)
                        }
                        ModelDomain::General => {
                            stats.general_submissions = stats.general_submissions.saturating_add(1)
                        }
                    }
                }

                stats.last_active = current_block;
                *maybe_stats = Some(stats);
            });

            Ok(())
        }

        /// Calculate oracle reward based on stats
        fn calculate_oracle_reward(stats: &OracleOperatorStats<BlockNumberFor<T>>) -> u128 {
            // Base reward per 1000 submissions
            let base_reward: u128 = 100_000_000_000_000; // 100 DALLA

            // Calculate weighted domain multiplier
            let total_domain_submissions = stats.agritech_submissions as u128
                * ModelDomain::AgriTech.reward_multiplier() as u128
                + stats.marine_submissions as u128
                    * ModelDomain::Marine.reward_multiplier() as u128
                + stats.education_submissions as u128
                    * ModelDomain::Education.reward_multiplier() as u128
                + stats.tech_submissions as u128 * ModelDomain::Tech.reward_multiplier() as u128
                + stats.general_submissions as u128
                    * ModelDomain::General.reward_multiplier() as u128;

            let total_submissions = stats.total_submissions.max(1) as u128;

            // C-4 FIX: Multiply all numerators first, divide once at the end to avoid
            // cascading integer truncation that zeroed rewards for <1000 submissions.
            //
            // numerator = base_reward * total_submissions * total_domain_submissions
            //             * avg_quality_score * uptime_percentage
            // denominator = 1000 (volume) * (total_submissions * 100) (domain)
            //             * 100 (quality) * 1000 (uptime) * 10000 (final scale)
            //           = total_submissions * 10_000_000_000_000
            //
            // Use u128 — max practical numerator is bounded by realistic stats values.
            let numerator = base_reward
                .saturating_mul(total_submissions)
                .saturating_mul(total_domain_submissions)
                .saturating_mul(stats.avg_quality_score as u128)
                .saturating_mul(stats.uptime_percentage as u128);

            let denominator = total_submissions.saturating_mul(10_000_000_000_000u128);

            if denominator == 0 {
                return 0;
            }

            numerator / denominator
        }

        /// Get IoT device info
        pub fn get_iot_device(
            device_id: [u8; 32],
        ) -> Option<IoTDevice<T::AccountId, BlockNumberFor<T>>> {
            IoTDevices::<T>::get(device_id)
        }

        /// Get oracle operator stats
        pub fn get_operator_stats(
            operator: &T::AccountId,
        ) -> Option<OracleOperatorStats<BlockNumberFor<T>>> {
            OracleOperatorStatsMap::<T>::get(operator)
        }

        /// Check if device is verified
        pub fn is_device_verified(device_id: [u8; 32]) -> bool {
            IoTDevices::<T>::get(device_id)
                .map(|d| d.verified)
                .unwrap_or(false)
        }
    }
}
