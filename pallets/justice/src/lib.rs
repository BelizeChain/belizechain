#![cfg_attr(not(feature = "std"), no_std)]

//! # BelizeChain Justice Pallet
//!
//! Implements the restorative-justice layer for the BelizeChain protocol.
//! Before a punitive slash is executed or a ban is issued, this pallet provides:
//!
//! 1. **Dispute gateway** — any account can open a dispute against an alleged offender.
//!    A cooling-off period is enforced for first offenses; the slash amount is escrowed.
//! 2. **Mediation** — governance-appointed mediators issue a binding ruling.
//! 3. **Appeals** — the accused can lodge one appeal with counter-evidence.
//! 4. **Rehabilitation tracking** — once the cooling-off period ends and no new offenses
//!    occur, an account can be declared rehabilitated and reintegrated.
//!
//! ## Slash Escrow Flow
//!
//! ```text
//! External pallet detects offense
//!        │
//!        ├─ first offense? ─────► open_dispute() ─► escrow slash in SlashPendingJusticeReview
//!        │                              │
//!        │                        mediator_ruling() ─► Upheld: execute slash
//!        │                                           └─► Dismissed/Mediated: refund
//!        │
//!        └─ repeat offense? ──────────► bypass justice (external pallet slashes directly)
//! ```

pub use pallet::*;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Get},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use crate::weights::WeightInfo;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ── Dispute severity ──────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DisputeSeverity {
        /// Minor infractions — compulsory cooling-off only, no escrow.
        Minor,
        /// Moderate — partial escrow.
        Moderate,
        /// Severe — full escrow. Mediator required before resolution.
        Severe,
    }

    impl DisputeSeverity {
        pub fn from_u8(v: u8) -> Option<Self> {
            match v {
                0 => Some(Self::Minor),
                1 => Some(Self::Moderate),
                2 => Some(Self::Severe),
                _ => None,
            }
        }
    }

    // ── Dispute status ────────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DisputeStatus {
        /// Freshly opened; awaiting mediator assignment.
        Pending,
        /// Mediator assigned; ruling in progress.
        UnderReview,
        /// Mediator issued ruling; optionally appealed.
        Ruled,
        /// Accused appealed the ruling.
        Appealed,
        /// Final — no further changes.
        Closed,
    }

    // ── Dispute resolution ────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DisputeResolution {
        /// Allegation dismissed; no penalty. Escrowed slash is refunded.
        Dismissed,
        /// Allegation upheld; full escrowed slash is executed.
        Upheld,
        /// Mediated settlement — slash_bps (basis points, 0–10_000) of escrow executed.
        Mediated { slash_bps: u32 },
    }

    // ── Rehabilitation status ─────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum RehabStatus {
        /// No active justice proceedings.
        Clean,
        /// Cooling-off period is active.
        InCoolingOff,
        /// Cooling-off completed; formal rehabilitation programme ongoing.
        InRehabilitation,
        /// Fully reinstated.
        Reinstated,
    }

    impl Default for RehabStatus {
        fn default() -> Self { Self::Clean }
    }

    // ── DisputeRecord ─────────────────────────────────────────────────────────
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DisputeRecord<T: Config> {
        /// Account that filed the dispute (pays the bond).
        pub disputant: T::AccountId,
        /// Account accused of the offense.
        pub target: T::AccountId,
        /// Blake2-256 hash of off-chain evidence.
        pub evidence_hash: [u8; 32],
        pub severity: DisputeSeverity,
        pub opened_at: BlockNumberFor<T>,
        pub status: DisputeStatus,
        pub resolution: Option<DisputeResolution>,
        /// Bond paid by disputant; refunded on Dismissed, forfeited on frivolous.
        pub bond: BalanceOf<T>,
        /// Hash of counter-evidence supplied in an appeal.
        pub appeal_evidence: Option<[u8; 32]>,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency used for bonds and escrowed slashes.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Origin that can approve/remove mediators.
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin that can issue mediator rulings (must be an approved mediator).
        /// Checked at both origin level and storage level (MediatorList).
        type MediatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Bond required to open a dispute — prevents frivolous filings.
        #[pallet::constant]
        type OpenDisputeBond: Get<BalanceOf<Self>>;

        /// Blocks before a cooling-off account may enter rehabilitation.
        #[pallet::constant]
        type CoolingOffPeriod: Get<BlockNumberFor<Self>>;

        /// Maximum number of governance-appointed mediators.
        #[pallet::constant]
        type MaxMediators: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    // ── Storage ───────────────────────────────────────────────────────────────

    /// Monotonically increasing dispute counter.
    #[pallet::storage]
    #[pallet::getter(fn dispute_counter)]
    pub type DisputeCounter<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// All open and closed disputes.
    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u32,
        DisputeRecord<T>,
        OptionQuery,
    >;

    /// Block at which a cooling-off period ends for an account.
    #[pallet::storage]
    #[pallet::getter(fn cooling_off_end)]
    pub type CoolingOffEnd<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// Rehabilitation status per account.
    #[pallet::storage]
    #[pallet::getter(fn rehab_status)]
    pub type RehabilitationStatus<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        RehabStatus,
        ValueQuery,
    >;

    /// Slash amounts held in escrow pending justice review.
    /// Set by external pallets (via the `escrow_slash` public function) before
    /// routing to a dispute.
    #[pallet::storage]
    #[pallet::getter(fn slash_pending)]
    pub type SlashPendingJusticeReview<T: Config> = StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// Governance-appointed mediators.
    #[pallet::storage]
    #[pallet::getter(fn mediator_list)]
    pub type MediatorList<T: Config> = StorageValue<
        _,
        BoundedVec<T::AccountId, T::MaxMediators>,
        ValueQuery,
    >;

    // ── Events ────────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A dispute was opened against `target`.
        DisputeOpened { dispute_id: u32, disputant: T::AccountId, target: T::AccountId },
        /// A mediator issued a ruling.
        MediatorRulingIssued { dispute_id: u32, resolution: DisputeResolution },
        /// The accused appealed a ruling.
        RulingAppealed { dispute_id: u32, by: T::AccountId },
        /// An account completed rehabilitation and was reinstated.
        AccountReinstated { account: T::AccountId },
        /// A slash was escrowed by an external pallet for justice review.
        SlashEscrowed { account: T::AccountId, amount: BalanceOf<T> },
        /// An escrowed slash was executed after an Upheld ruling.
        SlashExecuted { account: T::AccountId, amount: BalanceOf<T> },
        /// An escrowed slash was refunded after a Dismissed ruling.
        SlashRefunded { account: T::AccountId, amount: BalanceOf<T> },
        /// Mediator added.
        MediatorAdded { mediator: T::AccountId },
        /// Mediator removed.
        MediatorRemoved { mediator: T::AccountId },
    }

    // ── Errors ────────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// No dispute found with this ID.
        DisputeNotFound,
        /// Dispute is not in the correct state for this action.
        InvalidDisputeStatus,
        /// Caller is not an approved mediator.
        NotApprovedMediator,
        /// Dispute has already been appealed.
        AlreadyAppealed,
        /// Account is not in the correct rehabilitation state.
        InvalidRehabState,
        /// Cooling-off period has not yet ended.
        CoolingOffActive,
        /// Invalid severity code.
        InvalidSeverity,
        /// Mediator list is full.
        MediatorListFull,
        /// Mediator not found in list.
        MediatorNotFound,
        /// Caller must be the target of the dispute to appeal.
        NotDisputeTarget,
        /// slash_bps exceeds maximum of 10_000 (100%).
        SlashBpsExceedsMaximum,
    }

    // ── Extrinsics ────────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Open a dispute against `target`.
        ///
        /// The caller pays `OpenDisputeBond` which is reserved.  The bond is
        /// refunded if the dispute is Dismissed, and slashed if the mediator
        /// determines the dispute was frivolous.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::open_dispute())]
        pub fn open_dispute(
            origin: OriginFor<T>,
            target: T::AccountId,
            evidence_hash: [u8; 32],
            severity: u8,
        ) -> DispatchResult {
            let disputant = ensure_signed(origin)?;

            let severity = DisputeSeverity::from_u8(severity)
                .ok_or(Error::<T>::InvalidSeverity)?;

            // Reserve the dispute bond
            T::Currency::reserve(&disputant, T::OpenDisputeBond::get())?;

            let dispute_id = DisputeCounter::<T>::mutate(|c| { *c = c.saturating_add(1); *c });
            let current_block = frame_system::Pallet::<T>::block_number();

            // Start cooling-off for the target
            let cooling_end = current_block.saturating_add(T::CoolingOffPeriod::get());
            CoolingOffEnd::<T>::insert(&target, cooling_end);
            RehabilitationStatus::<T>::insert(&target, RehabStatus::InCoolingOff);

            let record = DisputeRecord::<T> {
                disputant: disputant.clone(),
                target: target.clone(),
                evidence_hash,
                severity,
                opened_at: current_block,
                status: DisputeStatus::Pending,
                resolution: None,
                bond: T::OpenDisputeBond::get(),
                appeal_evidence: None,
            };

            Disputes::<T>::insert(dispute_id, record);

            Self::deposit_event(Event::DisputeOpened { dispute_id, disputant, target });

            Ok(())
        }

        /// Issue a mediator ruling on an open dispute.
        ///
        /// The caller must be both in `MediatorList` and satisfy `MediatorOrigin`.
        /// `resolution`: 0 = Dismissed, 1 = Upheld, 2 = Mediated(slash_bps as leading arg)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::mediator_ruling())]
        pub fn mediator_ruling(
            origin: OriginFor<T>,
            dispute_id: u32,
            resolution_code: u8, // 0=Dismissed, 1=Upheld, 2=Mediated
            slash_bps: u32,       // only used when resolution_code == 2
        ) -> DispatchResult {
            T::MediatorOrigin::ensure_origin(origin.clone())?;
            let mediator = ensure_signed(origin)?;

            // Verify caller is in approved mediator list
            let list = MediatorList::<T>::get();
            ensure!(list.contains(&mediator), Error::<T>::NotApprovedMediator);

            let mut record = Disputes::<T>::get(dispute_id)
                .ok_or(Error::<T>::DisputeNotFound)?;

            ensure!(
                matches!(record.status, DisputeStatus::Pending | DisputeStatus::UnderReview),
                Error::<T>::InvalidDisputeStatus
            );

            let resolution = match resolution_code {
                0 => DisputeResolution::Dismissed,
                1 => DisputeResolution::Upheld,
                2 => {
                    ensure!(slash_bps <= 10_000, Error::<T>::SlashBpsExceedsMaximum);
                    DisputeResolution::Mediated { slash_bps }
                }
                _ => return Err(Error::<T>::InvalidDisputeStatus.into()),
            };

            // Execute or refund escrowed slash
            if let Some(escrowed) = SlashPendingJusticeReview::<T>::take(&record.target) {
                match &resolution {
                    DisputeResolution::Dismissed => {
                        // Unreserve back to target
                        T::Currency::unreserve(&record.target, escrowed);
                        Self::deposit_event(Event::SlashRefunded {
                            account: record.target.clone(), amount: escrowed,
                        });
                        // Update rehab status
                        RehabilitationStatus::<T>::insert(&record.target, RehabStatus::InRehabilitation);
                    }
                    DisputeResolution::Upheld => {
                        // Slash the full escrowed amount by unreserving and burning/transferring
                        T::Currency::unreserve(&record.target, escrowed);
                        let _ = T::Currency::slash(&record.target, escrowed);
                        Self::deposit_event(Event::SlashExecuted {
                            account: record.target.clone(), amount: escrowed,
                        });
                        RehabilitationStatus::<T>::insert(&record.target, RehabStatus::InCoolingOff);
                    }
                    DisputeResolution::Mediated { slash_bps } => {
                        let slash_amount = escrowed.saturating_mul((*slash_bps as u32).into())
                            / 10_000u32.into();
                        let refund = escrowed.saturating_sub(slash_amount);
                        T::Currency::unreserve(&record.target, escrowed);
                        if slash_amount > BalanceOf::<T>::default() {
                            let _ = T::Currency::slash(&record.target, slash_amount);
                            Self::deposit_event(Event::SlashExecuted {
                                account: record.target.clone(), amount: slash_amount,
                            });
                        }
                        if refund > BalanceOf::<T>::default() {
                            Self::deposit_event(Event::SlashRefunded {
                                account: record.target.clone(), amount: refund,
                            });
                        }
                        RehabilitationStatus::<T>::insert(&record.target, RehabStatus::InRehabilitation);
                    }
                }
            }

            // Refund disputant bond on Dismissed (they were right)
            if matches!(resolution, DisputeResolution::Dismissed) {
                T::Currency::unreserve(&record.disputant, record.bond);
            }

            record.resolution = Some(resolution.clone());
            record.status = DisputeStatus::Ruled;
            Disputes::<T>::insert(dispute_id, record);

            Self::deposit_event(Event::MediatorRulingIssued { dispute_id, resolution });

            Ok(())
        }

        /// Appeal a mediator ruling.
        ///
        /// Only the dispute target may appeal, and only once.  Provides
        /// counter-evidence hash.  Governance decides the final outcome.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::appeal_ruling())]
        pub fn appeal_ruling(
            origin: OriginFor<T>,
            dispute_id: u32,
            counter_evidence_hash: [u8; 32],
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let mut record = Disputes::<T>::get(dispute_id)
                .ok_or(Error::<T>::DisputeNotFound)?;

            ensure!(caller == record.target, Error::<T>::NotDisputeTarget);
            ensure!(
                matches!(record.status, DisputeStatus::Ruled),
                Error::<T>::InvalidDisputeStatus
            );
            ensure!(record.appeal_evidence.is_none(), Error::<T>::AlreadyAppealed);

            record.appeal_evidence = Some(counter_evidence_hash);
            record.status = DisputeStatus::Appealed;
            Disputes::<T>::insert(dispute_id, record);

            Self::deposit_event(Event::RulingAppealed { dispute_id, by: caller });

            Ok(())
        }

        /// Declare an account rehabilitated and reinstated.
        ///
        /// Callable by `GovernanceOrigin` or a mediator.  The account must be
        /// in `InRehabilitation` and its cooling-off period must have elapsed.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::complete_rehabilitation())]
        pub fn complete_rehabilitation(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            // Either governance or a mediator may reinstate
            let _ = T::GovernanceOrigin::ensure_origin(origin.clone())
                .map(|_| ())
                .or_else(|_| T::MediatorOrigin::ensure_origin(origin).map(|_| ()))?;

            let status = RehabilitationStatus::<T>::get(&account);
            ensure!(
                matches!(status, RehabStatus::InRehabilitation),
                Error::<T>::InvalidRehabState
            );

            // Cooling-off must have elapsed
            if let Some(end) = CoolingOffEnd::<T>::get(&account) {
                let now = frame_system::Pallet::<T>::block_number();
                ensure!(now >= end, Error::<T>::CoolingOffActive);
            }

            RehabilitationStatus::<T>::insert(&account, RehabStatus::Reinstated);
            CoolingOffEnd::<T>::remove(&account);

            Self::deposit_event(Event::AccountReinstated { account });

            Ok(())
        }

        /// Add a mediator (governance only).
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::add_mediator())]
        pub fn add_mediator(
            origin: OriginFor<T>,
            mediator: T::AccountId,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            MediatorList::<T>::try_mutate(|list| {
                list.try_push(mediator.clone())
                    .map_err(|_| Error::<T>::MediatorListFull)
            })?;

            Self::deposit_event(Event::MediatorAdded { mediator });
            Ok(())
        }

        /// Remove a mediator (governance only).
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::remove_mediator())]
        pub fn remove_mediator(
            origin: OriginFor<T>,
            mediator: T::AccountId,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            MediatorList::<T>::try_mutate(|list| {
                let pos = list.iter().position(|a| a == &mediator)
                    .ok_or(Error::<T>::MediatorNotFound)?;
                list.remove(pos);
                Ok::<(), Error<T>>(())
            })?;

            Self::deposit_event(Event::MediatorRemoved { mediator });
            Ok(())
        }
    }

    // ── Public API for external pallets ──────────────────────────────────────

    impl<T: Config> Pallet<T> {
        /// Escrow a slash amount for justice review.
        ///
        /// Called by an external pallet (e.g., staking) when it detects a
        /// first offense and wants to route through the justice pallet before
        /// executing the slash.  The amount is reserved from the offender's
        /// free balance.
        ///
        /// Returns `Ok(())` if the slash was successfully escrowed, or an
        /// error if the account cannot cover the reservation.
        pub fn escrow_slash(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            T::Currency::reserve(account, amount)?;
            SlashPendingJusticeReview::<T>::insert(account, amount);
            Self::deposit_event(Event::SlashEscrowed { account: account.clone(), amount });
            Ok(())
        }

        /// Returns true if the account has an escrowed slash awaiting review.
        pub fn has_pending_review(account: &T::AccountId) -> bool {
            SlashPendingJusticeReview::<T>::contains_key(account)
        }

        /// Returns the current rehabilitation status of an account.
        pub fn rehabilitation_status(account: &T::AccountId) -> RehabStatus {
            RehabilitationStatus::<T>::get(account)
        }
    }
}
