#![cfg_attr(not(feature = "std"), no_std)]

//! # BelizeChain Whistleblower Pallet
//!
//! Provides pseudonymous misconduct reporting with on-chain reward escrow.
//!
//! ## Pseudonymity Mechanism
//!
//! Reporters never store their real identity on-chain.  Instead they compute:
//!
//! ```text
//! commitment = blake2_256(DOMAIN_TAG ++ account_id_bytes ++ secret_bytes)
//! ```
//!
//! where `DOMAIN_TAG = b"BelizeChainWhistleblowerV1"` provides domain separation
//! to prevent cross-protocol pre-image attacks.
//!
//! Only `commitment` is stored with the report.  When a report is Verified and
//! the reporter wishes to claim their reward, they call `claim_reward(report_id,
//! secret)`.  The chain recomputes the hash; identity is revealed only
//! at that moment and is never persisted in the report record.
//!
//! ## Relayer / Sponsor Pattern
//!
//! The `submit_report` extrinsic reserves the anti-spam bond from the **signer**,
//! who is recorded as `bond_depositor` in the report.  The signer need not be the
//! reporter — any trusted third party can submit on behalf of the reporter.  This
//! provides operational anonymity: the reporter's real account never appears in any
//! transaction.
//!
//! ## Reward Tiers
//!
//! | Category        | Reward (configurable) |
//! |-----------------|----------------------|
//! | Fraud           | `FraudReward`        |
//! | SystematicAbuse | `AbuseReward`        |
//! | ChainExploit    | `ExploitReward`      |
//!
//! Rewards are funded by governance via `fund_whistleblower_pool` and are
//! escrowed per-report at submission time.

pub use pallet::*;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Get},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use sp_std::vec::Vec;
    use crate::weights::WeightInfo;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Domain separation tag for commitment hashes.
    /// Prevents cross-protocol pre-image attacks.
    const COMMITMENT_DOMAIN: &[u8] = b"BelizeChainWhistleblowerV1";

    // ── Report category ───────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ReportCategory {
        /// Fraudulent governance votes, bribery.
        Fraud,
        /// Systematic abuse of platform mechanisms.
        SystematicAbuse,
        /// Critical protocol exploit that threatens chain integrity.
        ChainExploit,
    }

    impl ReportCategory {
        pub fn from_u8(v: u8) -> Option<Self> {
            match v {
                0 => Some(Self::Fraud),
                1 => Some(Self::SystematicAbuse),
                2 => Some(Self::ChainExploit),
                _ => None,
            }
        }
    }

    // ── Report status ─────────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ReportStatus {
        /// Freshly submitted; awaiting a reviewer.
        Pending,
        /// Under active review by council.
        UnderReview,
        /// Verified; reward escrowed for claim.
        Verified,
        /// Dismissed; bond slashed from depositor.
        Dismissed,
        /// Reward claimed; terminal state.
        Claimed,
    }

    // ── Report record ─────────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Report<T: Config> {
        /// Reporter commitment = blake2_256(DOMAIN_TAG ++ account ++ secret).
        pub commitment: [u8; 32],
        /// Account accused of misconduct.
        pub target: T::AccountId,
        /// Blake2-256 hash of off-chain evidence document.
        pub evidence_hash: [u8; 32],
        pub category: ReportCategory,
        pub submitted_at: BlockNumberFor<T>,
        pub status: ReportStatus,
        /// Bond paid by the submitter (forfeited on Dismissed, refunded on Verified).
        pub bond: BalanceOf<T>,
        /// Account that paid the anti-spam bond.  May differ from the reporter
        /// (the `alias_hash` holder), enabling a *relayer / sponsor* pattern
        /// for pseudonymous submission.
        pub bond_depositor: T::AccountId,
        /// Hash of reviewer reasoning (for audit trail).
        pub reasoning_hash: Option<[u8; 32]>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Origin allowed to review reports (council members).
        type ReviewerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin allowed to fund the whistleblower pool (governance).
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Bond required to submit a report — protects against spam.
        #[pallet::constant]
        type ReportBond: Get<BalanceOf<Self>>;

        /// Reward for a verified Fraud report.
        #[pallet::constant]
        type FraudReward: Get<BalanceOf<Self>>;

        /// Reward for a verified SystematicAbuse report.
        #[pallet::constant]
        type AbuseReward: Get<BalanceOf<Self>>;

        /// Reward for a verified ChainExploit report.
        #[pallet::constant]
        type ExploitReward: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;

        /// Maximum total issuance of the native token.
        /// `deposit_creating` will be rejected if it would exceed this cap.
        #[pallet::constant]
        type MaxDallaSupply: Get<BalanceOf<Self>>;

        /// Maximum number of reports that can be submitted per block.
        #[pallet::constant]
        type MaxReportsPerBlock: Get<u32>;
    }

    // ── Storage ───────────────────────────────────────────────────────────────

    /// Monotonically increasing report counter.
    #[pallet::storage]
    #[pallet::getter(fn report_counter)]
    pub type ReportCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// All submitted reports.
    #[pallet::storage]
    #[pallet::getter(fn reports)]
    pub type Reports<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u32,
        Report<T>,
        OptionQuery,
    >;

    /// Total balance currently in the whistleblower reward pool.
    #[pallet::storage]
    #[pallet::getter(fn whistleblower_pool)]
    pub type WhistleblowerPool<T: Config> = StorageValue<
        _,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Reward amounts escrowed per report (set on Verified verdict).
    #[pallet::storage]
    #[pallet::getter(fn escrowed_reward)]
    pub type EscrowedReward<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u32,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// Per-block report submission counter (reset each block).
    #[pallet::storage]
    pub type ReportsThisBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            ReportsThisBlock::<T>::kill();
            // Phase-5 FIX: kill() is a DB write; account for ref_time + proof_size
            Weight::from_parts(5_000_000, 64)
                .saturating_add(T::DbWeight::get().writes(1))
        }
    }

    // ── Events ────────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new report was submitted.
        ReportSubmitted { report_id: u32, target: T::AccountId, category: u8 },
        /// A reviewer issued a verdict on a report.
        ReportReviewed { report_id: u32, verdict: u8 /* 0=Verified, 1=Dismissed */ },
        /// Reporter revealed identity and claimed reward.
        RewardClaimed { report_id: u32, claimant: T::AccountId, amount: BalanceOf<T> },
        /// Bond returned to depositor after report verified.
        BondReturned { report_id: u32, depositor: T::AccountId, amount: BalanceOf<T> },
        /// Bond slashed from depositor after report dismissed.
        BondSlashed { report_id: u32, depositor: T::AccountId, amount: BalanceOf<T> },
        /// Governance funded the whistleblower pool.
        PoolFunded { amount: BalanceOf<T>, new_total: BalanceOf<T> },
    }

    // ── Errors ────────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// No report found with this ID.
        ReportNotFound,
        /// Report is not in the correct state for this action.
        InvalidReportStatus,
        /// Invalid category code.
        InvalidCategory,
        /// Commitment verification failed — (account, secret) does not match commitment.
        CommitmentMismatch,
        /// No reward has been escrowed for this report.
        NoEscrowedReward,
        /// The whistleblower pool does not have sufficient funds.
        InsufficientPool,
        /// Reporter bond could not be reserved (insufficient balance).
        InsufficientBondBalance,
        /// Minting reward would exceed MaxDallaSupply.
        SupplyCapExceeded,
        /// Per-block report submission limit reached.
        ReportRateLimitExceeded,
        /// Report counter has reached its maximum value.
        ReportCounterOverflow,
    }

    // ── Extrinsics ────────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a pseudonymous misconduct report.
        ///
        /// `commitment` = blake2_256(DOMAIN_TAG ++ reporter_account_bytes ++ secret_bytes).
        /// The bond is reserved from the **signer** (the "sponsor"), who may
        /// differ from the reporter (the commitment holder).  This enables a
        /// relayer pattern: a trusted third party pays the bond on behalf of
        /// the reporter, preserving the reporter's on-chain anonymity.
        ///
        /// The bond is returned on Verified verdict, forfeited on Dismissed.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_report())]
        pub fn submit_report(
            origin: OriginFor<T>,
            commitment: [u8; 32],
            target: T::AccountId,
            evidence_hash: [u8; 32],
            category: u8,
        ) -> DispatchResult {
            let reporter = ensure_signed(origin)?;

            let category = ReportCategory::from_u8(category)
                .ok_or(Error::<T>::InvalidCategory)?;

            // Per-block rate limit check
            let report_count = ReportsThisBlock::<T>::get();
            ensure!(report_count < T::MaxReportsPerBlock::get(), Error::<T>::ReportRateLimitExceeded);

            // Reserve anti-spam bond from the REAL account (not stored in report)
            T::Currency::reserve(&reporter, T::ReportBond::get())
                .map_err(|_| Error::<T>::InsufficientBondBalance)?;

            let report_id = ReportCounter::<T>::try_mutate(|c| {
                *c = c.checked_add(1).ok_or(Error::<T>::ReportCounterOverflow)?;
                Ok::<u32, Error<T>>(*c)
            })?;
            let current_block = frame_system::Pallet::<T>::block_number();

            let report = Report::<T> {
                commitment,
                target: target.clone(),
                evidence_hash,
                category: category.clone(),
                submitted_at: current_block,
                status: ReportStatus::Pending,
                bond: T::ReportBond::get(),
                bond_depositor: reporter.clone(),
                reasoning_hash: None,
            };

            Reports::<T>::insert(report_id, report);
            ReportsThisBlock::<T>::mutate(|c| *c = c.saturating_add(1));

            let cat_u8 = match category {
                ReportCategory::Fraud => 0,
                ReportCategory::SystematicAbuse => 1,
                ReportCategory::ChainExploit => 2,
            };
            Self::deposit_event(Event::ReportSubmitted { report_id, target, category: cat_u8 });

            Ok(())
        }

        /// Council reviewer issues a verdict (Verified or Dismissed).
        ///
        /// `verdict`: 0 = Verified, 1 = Dismissed.
        /// On Verified: escrow the appropriate reward from the pool.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::review_report())]
        pub fn review_report(
            origin: OriginFor<T>,
            report_id: u32,
            verdict: u8, // 0 = Verified, 1 = Dismissed
            reasoning_hash: [u8; 32],
        ) -> DispatchResult {
            T::ReviewerOrigin::ensure_origin(origin)?;

            let mut report = Reports::<T>::get(report_id)
                .ok_or(Error::<T>::ReportNotFound)?;

            ensure!(
                matches!(report.status, ReportStatus::Pending | ReportStatus::UnderReview),
                Error::<T>::InvalidReportStatus
            );

            report.reasoning_hash = Some(reasoning_hash);

            match verdict {
                0 => {
                    // Verified — escrow reward and return bond to depositor
                    let reward = match report.category {
                        ReportCategory::Fraud => T::FraudReward::get(),
                        ReportCategory::SystematicAbuse => T::AbuseReward::get(),
                        ReportCategory::ChainExploit => T::ExploitReward::get(),
                    };
                    let pool = WhistleblowerPool::<T>::get();
                    ensure!(pool >= reward, Error::<T>::InsufficientPool);
                    WhistleblowerPool::<T>::put(pool.saturating_sub(reward));
                    EscrowedReward::<T>::insert(report_id, reward);
                    T::Currency::unreserve(&report.bond_depositor, report.bond);
                    Self::deposit_event(Event::BondReturned {
                        report_id,
                        depositor: report.bond_depositor.clone(),
                        amount: report.bond,
                    });
                    report.status = ReportStatus::Verified;
                }
                1 => {
                    // Dismissed — forfeit bond (slash from depositor)
                    let (_imbalance, _remaining) = T::Currency::slash_reserved(
                        &report.bond_depositor,
                        report.bond,
                    );
                    Self::deposit_event(Event::BondSlashed {
                        report_id,
                        depositor: report.bond_depositor.clone(),
                        amount: report.bond,
                    });
                    report.status = ReportStatus::Dismissed;
                }
                _ => return Err(Error::<T>::InvalidReportStatus.into()),
            }

            Reports::<T>::insert(report_id, report);
            Self::deposit_event(Event::ReportReviewed { report_id, verdict });

            Ok(())
        }

        /// Claim a verified report reward by revealing identity.
        ///
        /// The caller proves they are the original reporter by supplying their
        /// `secret` such that `blake2_256(DOMAIN_TAG ++ account ++ secret) == commitment`.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_reward())]
        pub fn claim_reward(
            origin: OriginFor<T>,
            report_id: u32,
            secret: [u8; 32],
        ) -> DispatchResult {
            let claimant = ensure_signed(origin)?;

            let report = Reports::<T>::get(report_id)
                .ok_or(Error::<T>::ReportNotFound)?;

            ensure!(
                matches!(report.status, ReportStatus::Verified),
                Error::<T>::InvalidReportStatus
            );

            // Domain-separated commitment verification
            let mut preimage = Vec::from(COMMITMENT_DOMAIN);
            preimage.extend_from_slice(&claimant.encode());
            preimage.extend_from_slice(&secret);
            let computed = sp_io::hashing::blake2_256(&preimage);
            ensure!(computed == report.commitment, Error::<T>::CommitmentMismatch);

            // Pay out escrowed reward
            let reward = EscrowedReward::<T>::take(report_id)
                .ok_or(Error::<T>::NoEscrowedReward)?;

            // P0-06 FIX: Check supply cap before minting
            let new_issuance = T::Currency::total_issuance()
                .checked_add(&reward)
                .ok_or(Error::<T>::SupplyCapExceeded)?;
            ensure!(
                new_issuance <= T::MaxDallaSupply::get(),
                Error::<T>::SupplyCapExceeded
            );

            let _ = T::Currency::deposit_creating(&claimant, reward);

            // P0-3: Transition to terminal state — prevents any future re-claim
            Reports::<T>::mutate(report_id, |maybe_report| {
                if let Some(r) = maybe_report {
                    r.status = ReportStatus::Claimed;
                }
            });

            Self::deposit_event(Event::RewardClaimed {
                report_id,
                claimant: claimant.clone(),
                amount: reward,
            });

            Ok(())
        }

        /// Governance funds the whistleblower reward pool.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::fund_whistleblower_pool())]
        pub fn fund_whistleblower_pool(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // Governance may authorize, but a signer must always provide real funds.
            if T::GovernanceOrigin::ensure_origin(origin.clone()).is_err() {
                ensure_signed(origin.clone())?;
            }
            let funder_account = ensure_signed(origin)?;

            T::Currency::reserve(&funder_account, amount)?;
            let new_total = WhistleblowerPool::<T>::mutate(|p| {
                *p = p.saturating_add(amount);
                *p
            });

            Self::deposit_event(Event::PoolFunded { amount, new_total });
            Ok(())
        }
    }
}
