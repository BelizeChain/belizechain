#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeID - Sovereign identity pallet for Belize
//!
//! Goals:
//! - Treat Passport and SSN as first-class, standards-based identifiers (ICAO Doc 9303 + SSB 9-digit)
//! - Privacy-first: on-chain stores salted hashes and credential anchors, never plaintext PII
//!   **Note**: Current privacy model uses salted blake2_256 hashes for PII. To verify identity,
//!   the verifier needs the plaintext value to hash-and-compare. True ZK selective disclosure
//!   (prove "I have KYC L2+" without revealing underlying data) requires ZK circuits and is
//!   roadmapped for 2028 via sp-arkworks integration.
//! - Issuer attestations: Immigration (Passport), SSB (SSN), Biometrics issuers; governance controls
//! - KYC levels: L1 (SSN), L2 (Passport + SSN), L3 (Biometrics + all prior)
//! - Annual validity + 6-month grace; revocation/suspension by governance/councils; emergency pause
//! - Multi-account identities and minimal DID export: did:belize:<identity>

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, EnsureOrigin},
    BoundedVec, PalletId,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::Zero;

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_std::prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Currency used for BelizeID fees
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// PalletId to derive the pallet's sovereign account
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Treasury account for slashed bonds
        type Treasury: Get<Self::AccountId>;

        /// Origin that can perform admin actions (governance)
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin that can revoke/suspend attributes (governance or councils)
        type RevokeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Oracle provider for external KYC data verification
        type Oracle: IdentityOracleProvider<Self::AccountId>;

        /// Maximum accounts that may link to a single identity
        #[pallet::constant]
        type MaxAccountsPerIdentity: Get<u32>;

        /// Max length for names
        #[pallet::constant]
        type MaxNameLen: Get<u32>;

        /// Max length for credential anchors (pakit CIDs)
        #[pallet::constant]
        type MaxAnchorLen: Get<u32>;

        /// Max issuers per attribute type
        #[pallet::constant]
        type MaxIssuerCount: Get<u32>;

        /// Number of blocks for KYC validity window (annual)
        #[pallet::constant]
    type KycValidityBlocks: Get<BlockNumberFor<Self>>;

        /// Number of blocks for grace window after validity
        #[pallet::constant]
    type KycGraceBlocks: Get<BlockNumberFor<Self>>;

        /// Max number of history events to keep per attribute per identity
        #[pallet::constant]
        type MaxHistoryLen: Get<u32>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    /// Oracle provider trait for Identity pallet
    /// Allows Identity pallet to query Oracle for KYC verification and sanctions
    pub trait IdentityOracleProvider<AccountId> {
        /// Get KYC level for an account from Oracle
        fn get_kyc_level(account: &AccountId) -> Option<u8>;
        
        /// Check if account meets minimum KYC requirement
        fn meets_kyc_requirement(account: &AccountId, required_level: u8) -> bool;
        
        /// Check if account is sanctioned (OFAC/UN lists)
        fn is_sanctioned(account: &AccountId) -> bool;
    }

    /// Balance type alias
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// IdentityId internal type
    pub type IdentityId = u128;

    /// DID representation (bounded)
    pub type DidOf<T> = BoundedVec<u8, <T as Config>::MaxAnchorLen>;

    /// Attribute types supported by BelizeID
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AttributeType {
        Ssn,
        Passport,
        Biometrics,
    }

    impl AttributeType {
        pub fn as_u8(self) -> u8 {
            match self { AttributeType::Ssn => 0, AttributeType::Passport => 1, AttributeType::Biometrics => 2 }
        }
    }

    impl From<u8> for AttributeType {
        fn from(v: u8) -> Self {
            match v { 0 => AttributeType::Ssn, 1 => AttributeType::Passport, _ => AttributeType::Biometrics }
        }
    }

    /// Attestation status
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AttestationStatus {
        Active,
        Suspended,
        Revoked,
    }

    /// KYC levels
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum KycLevel {
        L0,
        L1,
        L2,
        L3,
    }

    /// KYC overall state including grace
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum KycState {
        Valid,
        Grace,
        Invalid,
    }

    /// Identity core record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct IdentityRecord<T: Config> {
        /// Display name (not PII-critical but bounded)
        pub name: BoundedVec<u8, <T as Config>::MaxNameLen>,
        /// Primary owner account
        pub owner: T::AccountId,
        /// Linked accounts
        pub accounts: BoundedVec<T::AccountId, <T as Config>::MaxAccountsPerIdentity>,
        /// Optional DID Document anchor CID (pakit)
        pub did_doc_cid: Option<BoundedVec<u8, <T as Config>::MaxAnchorLen>>,
    }

    /// Attestation structure
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Attestation<T: Config> {
        pub attr_type: AttributeType,
        pub issuer: T::AccountId,
        /// Version of the standard used to issue
        pub standard_version: u32,
        /// Salted hash of normalized attribute value (blake2_256(salt ++ plaintext))
        pub hash: H256,
        /// DEPRECATED: Salt removed (P0-01 fix). Hash must be computed off-chain with
        /// a strong KDF (Argon2id) and secret salt that never touches the chain.
        /// Kept as Option for storage compatibility; always None for new attestations.
        pub salt: Option<BoundedVec<u8, ConstU32<64>>>,
        /// Issuer asserts that format complies with standard
        pub format_ok: bool,
        /// When issued
    pub issued_at: BlockNumberFor<T>,
        /// Validity window end
    pub valid_until: BlockNumberFor<T>,
        /// Grace window end
    pub grace_until: BlockNumberFor<T>,
        /// Off-chain credential anchor (pakit CID)
        pub anchor: BoundedVec<u8, <T as Config>::MaxAnchorLen>,
        /// Current status (Active, Suspended, Revoked)
        pub status: AttestationStatus,
    }

    /// History action
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum HistoryAction { Issued, Suspended, Revoked, Updated }

    /// History event
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct HistoryEvent<T: Config> {
        pub when: BlockNumberFor<T>,
        pub action: HistoryAction,
    }

    #[pallet::storage]
    #[pallet::getter(fn next_identity_id)]
    /// Auto-incrementing identity id
    pub type NextIdentityId<T: Config> = StorageValue<_, IdentityId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn identity_of)]
    /// Map account to identity id
    pub type IdentityOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, IdentityId>;

    #[pallet::storage]
    #[pallet::getter(fn identities)]
    /// IdentityId to record
    pub type Identities<T: Config> = StorageMap<_, Blake2_128Concat, IdentityId, IdentityRecord<T>>;

    // Attribute attestations per identity
    #[pallet::storage]
    #[pallet::getter(fn ssn_attestation)]
    pub type SsnAttestations<T: Config> = StorageMap<_, Blake2_128Concat, IdentityId, Attestation<T>>;

    #[pallet::storage]
    #[pallet::getter(fn passport_attestation)]
    pub type PassportAttestations<T: Config> = StorageMap<_, Blake2_128Concat, IdentityId, Attestation<T>>;

    #[pallet::storage]
    #[pallet::getter(fn biometric_attestation)]
    pub type BiometricAttestations<T: Config> = StorageMap<_, Blake2_128Concat, IdentityId, Attestation<T>>;

    // Reverse indexes to prevent duplicates and enable O(1) checks
    #[pallet::storage]
    pub type SsnHashIndex<T: Config> = StorageMap<_, Blake2_128Concat, H256, IdentityId>;
    #[pallet::storage]
    pub type PassportHashIndex<T: Config> = StorageMap<_, Blake2_128Concat, H256, IdentityId>;

    // Per-issuer flags
    #[pallet::storage]
    #[pallet::getter(fn is_issuer_flagged)]
    pub type FlaggedIssuers<T: Config> = StorageDoubleMap<_, Blake2_128Concat, AttributeType, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // Per-issuer bonds per attribute
    #[pallet::storage]
    #[pallet::getter(fn issuer_bond)]
    pub type IssuerBonds<T: Config> = StorageDoubleMap<_, Blake2_128Concat, AttributeType, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    // Governance-configurable bond amount required (can be 0)
    #[pallet::storage]
    #[pallet::getter(fn issuer_bond_amount)]
    pub type IssuerBondAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // Rate limiting config: window size and max per attribute per window
    #[pallet::storage]
    #[pallet::getter(fn rate_window_blocks)]
    pub type RateWindowBlocks<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn rate_max_per_window_ssn)]
    pub type RateMaxPerWindowSsn<T: Config> = StorageValue<_, u32, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn rate_max_per_window_passport)]
    pub type RateMaxPerWindowPassport<T: Config> = StorageValue<_, u32, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn rate_max_per_window_biometrics)]
    pub type RateMaxPerWindowBiometrics<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Rate counter per issuer per attribute
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct RateCounter<T: Config> {
        pub window_start: BlockNumberFor<T>,
        pub count: u32,
    }

    #[pallet::storage]
    #[pallet::getter(fn rate_counter)]
    pub type IssuerRate<T: Config> = StorageDoubleMap<_, Blake2_128Concat, AttributeType, Blake2_128Concat, T::AccountId, RateCounter<T>>;

    // Attribute history: per identity and attr
    #[pallet::storage]
    #[pallet::getter(fn attribute_history)]
    pub type AttributeHistory<T: Config> = StorageDoubleMap<_, Blake2_128Concat, IdentityId, Blake2_128Concat, AttributeType, BoundedVec<HistoryEvent<T>, <T as Config>::MaxHistoryLen>, ValueQuery>;

    // Issuer registries (governance-managed)
    #[pallet::storage]
    #[pallet::getter(fn ssn_issuers)]
    pub type SsnIssuers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxIssuerCount>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn passport_issuers)]
    pub type PassportIssuers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxIssuerCount>, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn biometric_issuers)]
    pub type BiometricIssuers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxIssuerCount>, ValueQuery>;

    // Standards (versioned)
    #[pallet::storage]
    #[pallet::getter(fn ssn_standard_version)]
    pub type SsnStandardVersion<T: Config> = StorageValue<_, u32, ValueQuery>;
    #[pallet::storage]
    #[pallet::getter(fn passport_standard_version)]
    pub type PassportStandardVersion<T: Config> = StorageValue<_, u32, ValueQuery>;

    // Fees and controls
    #[pallet::storage]
    #[pallet::getter(fn operation_fee)]
    pub type OperationFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn paused)]
    pub type GlobalPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub operation_fee: Option<BalanceOf<T>>,
        pub ssn_standard_version: u32,
        pub passport_standard_version: u32,
        pub initial_ssn_issuers: Vec<T::AccountId>,
        pub initial_passport_issuers: Vec<T::AccountId>,
        pub initial_biometric_issuers: Vec<T::AccountId>,
        pub paused: bool,
        pub start_identity_id: IdentityId,
        // Optional issuer bond amount requirement
        pub issuer_bond_amount: Option<BalanceOf<T>>,
        // Optional initial rate limiting
        pub rate_window_blocks: Option<BlockNumberFor<T>>,
        pub rate_limit_ssn: Option<u32>,
        pub rate_limit_passport: Option<u32>,
        pub rate_limit_biometrics: Option<u32>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                operation_fee: None,
                ssn_standard_version: 1,
                passport_standard_version: 1,
                initial_ssn_issuers: vec![],
                initial_passport_issuers: vec![],
                initial_biometric_issuers: vec![],
                paused: false,
                start_identity_id: 1,
                issuer_bond_amount: None,
                rate_window_blocks: None,
                rate_limit_ssn: None,
                rate_limit_passport: None,
                rate_limit_biometrics: None,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(fee) = &self.operation_fee { OperationFee::<T>::put(*fee); }
            SsnStandardVersion::<T>::put(self.ssn_standard_version);
            PassportStandardVersion::<T>::put(self.passport_standard_version);
            if let Some(b) = &self.issuer_bond_amount { IssuerBondAmount::<T>::put(*b); }
            if let Some(w) = &self.rate_window_blocks { RateWindowBlocks::<T>::put(*w); }
            if let Some(v) = &self.rate_limit_ssn { RateMaxPerWindowSsn::<T>::put(*v); }
            if let Some(v) = &self.rate_limit_passport { RateMaxPerWindowPassport::<T>::put(*v); }
            if let Some(v) = &self.rate_limit_biometrics { RateMaxPerWindowBiometrics::<T>::put(*v); }
            let mut ssn = BoundedVec::default();
            for i in &self.initial_ssn_issuers { let _ = ssn.try_push(i.clone()); }
            SsnIssuers::<T>::put(ssn);
            let mut pass = BoundedVec::default();
            for i in &self.initial_passport_issuers { let _ = pass.try_push(i.clone()); }
            PassportIssuers::<T>::put(pass);
            let mut bio = BoundedVec::default();
            for i in &self.initial_biometric_issuers { let _ = bio.try_push(i.clone()); }
            BiometricIssuers::<T>::put(bio);
            GlobalPaused::<T>::put(self.paused);
            NextIdentityId::<T>::put(self.start_identity_id);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Identity created
        IdentityRegistered { identity: IdentityId, owner: T::AccountId },
        /// Account linked to identity
        AccountLinked { identity: IdentityId, account: T::AccountId },
        /// DID document anchor updated
        DidDocUpdated { identity: IdentityId },
        /// Issuers updated
        IssuerAdded { attr: u8, issuer: T::AccountId },
        IssuerRemoved { attr: u8, issuer: T::AccountId },
        /// Standards version changes
        StandardVersionUpdated { attr: u8, version: u32 },
        /// Fees updated
        OperationFeeUpdated { fee: BalanceOf<T> },
        /// Global pause/resume
        Paused,
        Resumed,
        /// Attestations lifecycle
        Attested { identity: IdentityId, attr: u8, issuer: T::AccountId },
        Suspended { identity: IdentityId, attr: u8 },
        Revoked { identity: IdentityId, attr: u8 },
        /// Issuer flagged/unflagged
        IssuerFlagged { attr: u8, issuer: T::AccountId, flagged: bool },
        /// Issuer bond deposit/withdraw
        IssuerBondDeposited { attr: u8, issuer: T::AccountId, amount: BalanceOf<T> },
        IssuerBondWithdrawn { attr: u8, issuer: T::AccountId, amount: BalanceOf<T> },
        /// Issuer bond slashed
        IssuerBondSlashed { attr: u8, issuer: T::AccountId, amount: BalanceOf<T> },
        /// Rate limit config updated
        RateLimitUpdated { window: BlockNumberFor<T>, ssn: u32, passport: u32, biometrics: u32 },
        /// Issuer bond requirement updated
        IssuerBondAmountUpdated { amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Identity already exists for this account
        IdentityExists,
        /// Identity not found
        IdentityNotFound,
        /// Global paused
        Paused,
        /// Too many linked accounts
        TooManyAccounts,
        /// Not authorized issuer
        NotAuthorizedIssuer,
        /// Hash already linked to another identity
        HashAlreadyTaken,
        /// No attestation exists
        NoAttestation,
        /// Already attested
        AlreadyAttested,
        /// Admin origin required
        NotAdmin,
        /// Issuer flagged and cannot issue
        IssuerFlagged,
        /// Issuer rate limit exceeded
        IssuerRateLimitExceeded,
        /// Bond operations
        BondNotFound,
        BondInsufficient,
        CannotWithdrawWhileAuthorized,
        /// Provided salt is fewer than 32 bytes; minimum entropy not met.
        SaltTooShort,
    }

    impl<T: Config> Pallet<T> {
        /// Pallet sovereign account for fee collection
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        fn ensure_not_paused() -> DispatchResult {
            ensure!(!GlobalPaused::<T>::get(), Error::<T>::Paused);
            Ok(())
        }

        fn is_authorized_issuer(attr: AttributeType, who: &T::AccountId) -> bool {
            match attr {
                AttributeType::Ssn => SsnIssuers::<T>::get().into_inner().contains(who),
                AttributeType::Passport => PassportIssuers::<T>::get().into_inner().contains(who),
                AttributeType::Biometrics => BiometricIssuers::<T>::get().into_inner().contains(who),
            }
        }

        fn charge_fee(who: &T::AccountId) -> DispatchResult {
            let fee = OperationFee::<T>::get();
            if !fee.is_zero() {
                T::Currency::transfer(who, &Self::account_id(), fee, ExistenceRequirement::KeepAlive)?;
            }
            Ok(())
        }

        fn now() -> BlockNumberFor<T> { <frame_system::Pallet<T>>::block_number() }

        fn validity_windows() -> (BlockNumberFor<T>, BlockNumberFor<T>) {
            (T::KycValidityBlocks::get(), T::KycGraceBlocks::get())
        }

        /// Helper to add block numbers safely
        fn add_blocks(a: BlockNumberFor<T>, b: BlockNumberFor<T>) -> BlockNumberFor<T> {
            let a_u64: u64 = TryInto::<u64>::try_into(a).unwrap_or(0);
            let b_u64: u64 = TryInto::<u64>::try_into(b).unwrap_or(0);
            let sum = a_u64.saturating_add(b_u64);
            sum.try_into().unwrap_or(a)
        }

        /// Helper to subtract block numbers safely
        fn sub_blocks(a: BlockNumberFor<T>, b: BlockNumberFor<T>) -> BlockNumberFor<T> {
            let a_u64: u64 = TryInto::<u64>::try_into(a).unwrap_or(0);
            let b_u64: u64 = TryInto::<u64>::try_into(b).unwrap_or(0);
            let diff = a_u64.saturating_sub(b_u64);
            diff.try_into().unwrap_or(Zero::zero())
        }

        fn append_history(id: IdentityId, attr: AttributeType, action: HistoryAction) {
            let now = Self::now();
            AttributeHistory::<T>::mutate(id, attr, |hist| {
                let evt = HistoryEvent::<T> { when: now, action };
                if hist.try_push(evt.clone()).is_err() {
                    // remove oldest: O(n) but bounded small size
                    if !hist.is_empty() {
                        let _ = hist.remove(0);
                        let _ = hist.try_push(evt);
                    }
                }
            });
        }

        fn max_per_window(attr: AttributeType) -> u32 {
            match attr {
                AttributeType::Ssn => RateMaxPerWindowSsn::<T>::get(),
                AttributeType::Passport => RateMaxPerWindowPassport::<T>::get(),
                AttributeType::Biometrics => RateMaxPerWindowBiometrics::<T>::get(),
            }
        }

        fn check_and_bump_rate(attr: AttributeType, issuer: &T::AccountId) -> DispatchResult {
            let now = Self::now();
            let window = RateWindowBlocks::<T>::get();
            let max = Self::max_per_window(attr);
            if max == 0 || window == Zero::zero() { return Ok(()); }
            let mut rc = IssuerRate::<T>::get(attr, issuer.clone()).unwrap_or(RateCounter { window_start: now, count: 0 });
            if Self::sub_blocks(now, rc.window_start) > window {
                rc.window_start = now;
                rc.count = 0;
            }
            if rc.count >= max { return Err(Error::<T>::IssuerRateLimitExceeded.into()); }
            rc.count = rc.count.saturating_add(1);
            IssuerRate::<T>::insert(attr, issuer, rc);
            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new identity for the caller, paying the configured fee
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_identity())]
        pub fn register_identity(origin: OriginFor<T>, name: BoundedVec<u8, T::MaxNameLen>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_not_paused()?;

            ensure!(!IdentityOf::<T>::contains_key(&who), Error::<T>::IdentityExists);

            // charge fee
            Self::charge_fee(&who)?;

            let id = NextIdentityId::<T>::get();
            let mut accounts: BoundedVec<T::AccountId, T::MaxAccountsPerIdentity> = BoundedVec::default();
            accounts.try_push(who.clone()).map_err(|_| Error::<T>::TooManyAccounts)?;

            let record = IdentityRecord::<T> { name: name.clone(), owner: who.clone(), accounts, did_doc_cid: None };
            Identities::<T>::insert(id, record);
            IdentityOf::<T>::insert(&who, id);
            NextIdentityId::<T>::put(id.saturating_add(1));

            Self::deposit_event(Event::IdentityRegistered { identity: id, owner: who });
            Ok(())
        }

        /// Link another account to the caller's identity
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::link_account())]
        pub fn link_account(origin: OriginFor<T>, new_account: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_not_paused()?;
            let id = IdentityOf::<T>::get(&who).ok_or(Error::<T>::IdentityNotFound)?;
            ensure!(!IdentityOf::<T>::contains_key(&new_account), Error::<T>::IdentityExists);

            Identities::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let rec = maybe.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                // M50 FIX: Only primary account owner (first account) can link new accounts
                // This prevents unauthorized delegation chains from compromised linked accounts
                ensure!(!rec.accounts.is_empty() && rec.accounts[0] == who, Error::<T>::IdentityNotFound);
                rec.accounts.try_push(new_account.clone()).map_err(|_| Error::<T>::TooManyAccounts)?;
                Ok(())
            })?;
            IdentityOf::<T>::insert(&new_account, id);
            Self::deposit_event(Event::AccountLinked { identity: id, account: new_account });
            Ok(())
        }

        /// Update DID Document anchor (pakit CID)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_did())]
        pub fn update_did_doc(origin: OriginFor<T>, cid: BoundedVec<u8, T::MaxAnchorLen>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let id = IdentityOf::<T>::get(&who).ok_or(Error::<T>::IdentityNotFound)?;
            Identities::<T>::try_mutate(id, |maybe| -> DispatchResult {
                let rec = maybe.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                // I-7 FIX: Only primary account (first linked) can update DID doc
                ensure!(
                    !rec.accounts.is_empty() && rec.accounts[0] == who,
                    Error::<T>::IdentityNotFound
                );
                rec.did_doc_cid = Some(cid.clone());
                Ok(())
            })?;
            Self::deposit_event(Event::DidDocUpdated { identity: id });
            Ok(())
        }

        /// Admin: add an issuer for a given attribute type
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn add_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            let attr_e = AttributeType::from(attr);
            // Ensure bond requirement met
            let required = IssuerBondAmount::<T>::get();
            if !required.is_zero() {
                ensure!(IssuerBonds::<T>::get(attr_e, issuer.clone()) >= required, Error::<T>::BondInsufficient);
            }
            // I-4 FIX: Prevent duplicate issuer registration
            match attr_e {
                AttributeType::Ssn => {
                    SsnIssuers::<T>::try_mutate(|v| {
                        ensure!(!v.contains(&issuer), Error::<T>::AlreadyAttested);
                        v.try_push(issuer.clone()).map_err(|_| Error::<T>::TooManyAccounts)
                    })?;
                }
                AttributeType::Passport => {
                    PassportIssuers::<T>::try_mutate(|v| {
                        ensure!(!v.contains(&issuer), Error::<T>::AlreadyAttested);
                        v.try_push(issuer.clone()).map_err(|_| Error::<T>::TooManyAccounts)
                    })?;
                }
                AttributeType::Biometrics => {
                    BiometricIssuers::<T>::try_mutate(|v| {
                        ensure!(!v.contains(&issuer), Error::<T>::AlreadyAttested);
                        v.try_push(issuer.clone()).map_err(|_| Error::<T>::TooManyAccounts)
                    })?;
                }
            }
            Self::deposit_event(Event::IssuerAdded { attr, issuer });
            Ok(())
        }

        /// Admin: remove an issuer
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn remove_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            let attr_e = AttributeType::from(attr);
            let removed = match attr_e {
                AttributeType::Ssn => {
                    SsnIssuers::<T>::mutate(|v| { if let Some(pos) = v.iter().position(|x| x == &issuer) { v.swap_remove(pos); true } else { false } })
                }
                AttributeType::Passport => {
                    PassportIssuers::<T>::mutate(|v| { if let Some(pos) = v.iter().position(|x| x == &issuer) { v.swap_remove(pos); true } else { false } })
                }
                AttributeType::Biometrics => {
                    BiometricIssuers::<T>::mutate(|v| { if let Some(pos) = v.iter().position(|x| x == &issuer) { v.swap_remove(pos); true } else { false } })
                }
            };
            ensure!(removed, Error::<T>::NotAuthorizedIssuer);
            Self::deposit_event(Event::IssuerRemoved { attr, issuer });
            Ok(())
        }

        /// Admin: update standards version
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn set_standard_version(origin: OriginFor<T>, attr: u8, version: u32) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            // I-5 FIX: Return error for Biometrics instead of silent no-op
            match AttributeType::from(attr) {
                AttributeType::Ssn => SsnStandardVersion::<T>::put(version),
                AttributeType::Passport => PassportStandardVersion::<T>::put(version),
                AttributeType::Biometrics => {
                    return Err(Error::<T>::NoAttestation.into()); // No BiometricsStandardVersion storage exists
                },
            }
            Self::deposit_event(Event::StandardVersionUpdated { attr, version });
            Ok(())
        }

        /// Admin: set operation fee (10 DALLA default via genesis)
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn set_operation_fee(origin: OriginFor<T>, fee: BalanceOf<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            OperationFee::<T>::put(fee);
            Self::deposit_event(Event::OperationFeeUpdated { fee });
            Ok(())
        }

        /// Admin: set issuer bond requirement (0 disables)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn set_issuer_bond_amount(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            IssuerBondAmount::<T>::put(amount);
            Self::deposit_event(Event::IssuerBondAmountUpdated { amount });
            Ok(())
        }

        /// Admin: set rate limit window and per-attr thresholds (0 disables)
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn set_rate_limits(
            origin: OriginFor<T>,
            window: BlockNumberFor<T>,
            ssn: u32,
            passport: u32,
            biometrics: u32,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            RateWindowBlocks::<T>::put(window);
            RateMaxPerWindowSsn::<T>::put(ssn);
            RateMaxPerWindowPassport::<T>::put(passport);
            RateMaxPerWindowBiometrics::<T>::put(biometrics);
            Self::deposit_event(Event::RateLimitUpdated { window, ssn, passport, biometrics });
            Ok(())
        }

        /// Admin: pause/resume the pallet
    #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn set_pause(origin: OriginFor<T>, paused: bool) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            GlobalPaused::<T>::put(paused);
            if paused { Self::deposit_event(Event::Paused); } else { Self::deposit_event(Event::Resumed); }
            Ok(())
        }

        /// Issuer: issue SSN attestation.
        /// The `hash` must be computed OFF-CHAIN using Argon2id(secret_salt || ssn).
        /// The salt must NEVER be submitted on-chain (P0-01: SSN brute-force prevention).
    #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::issue_attestation())]
        pub fn issue_ssn(
            origin: OriginFor<T>,
            target: T::AccountId,
            hash: H256,
            anchor: BoundedVec<u8, T::MaxAnchorLen>,
            format_ok: bool,
        ) -> DispatchResult {
            let issuer = ensure_signed(origin)?;
            ensure!(Self::is_authorized_issuer(AttributeType::Ssn, &issuer), Error::<T>::NotAuthorizedIssuer);
            ensure!(!FlaggedIssuers::<T>::get(AttributeType::Ssn, issuer.clone()), Error::<T>::IssuerFlagged);
            Self::ensure_not_paused()?;
            Self::check_and_bump_rate(AttributeType::Ssn, &issuer)?;
            let id = IdentityOf::<T>::get(&target).ok_or(Error::<T>::IdentityNotFound)?;
            ensure!(SsnHashIndex::<T>::get(hash).map(|x| x == id).unwrap_or(true), Error::<T>::HashAlreadyTaken);

            let now = Self::now();
            let (valid, grace) = Self::validity_windows();
            let att = Attestation::<T> {
                attr_type: AttributeType::Ssn,
                issuer: issuer.clone(),
                standard_version: SsnStandardVersion::<T>::get(),
                hash,
                salt: None,
                format_ok,
                issued_at: now,
                valid_until: Self::add_blocks(now, valid),
                grace_until: Self::add_blocks(Self::add_blocks(now, valid), grace),
                anchor,
                status: AttestationStatus::Active,
            };
            // Remove stale hash index entry if SSN was previously issued
            if let Some(old_att) = SsnAttestations::<T>::get(id) {
                if old_att.hash != hash {
                    SsnHashIndex::<T>::remove(old_att.hash);
                }
            }
            SsnAttestations::<T>::insert(id, att);
            SsnHashIndex::<T>::insert(hash, id);
            Self::append_history(id, AttributeType::Ssn, HistoryAction::Issued);
            Self::deposit_event(Event::Attested { identity: id, attr: AttributeType::Ssn as u8, issuer });
            Ok(())
        }

        /// Issuer: issue Passport attestation
    #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::issue_attestation())]
        pub fn issue_passport(
            origin: OriginFor<T>,
            target: T::AccountId,
            hash: H256,
            anchor: BoundedVec<u8, T::MaxAnchorLen>,
            format_ok: bool,
        ) -> DispatchResult {
            let issuer = ensure_signed(origin)?;
            ensure!(Self::is_authorized_issuer(AttributeType::Passport, &issuer), Error::<T>::NotAuthorizedIssuer);
            ensure!(!FlaggedIssuers::<T>::get(AttributeType::Passport, issuer.clone()), Error::<T>::IssuerFlagged);
            Self::ensure_not_paused()?;
            Self::check_and_bump_rate(AttributeType::Passport, &issuer)?;
            let id = IdentityOf::<T>::get(&target).ok_or(Error::<T>::IdentityNotFound)?;
            ensure!(PassportHashIndex::<T>::get(hash).map(|x| x == id).unwrap_or(true), Error::<T>::HashAlreadyTaken);

            let now = Self::now();
            let (valid, grace) = Self::validity_windows();
            let att = Attestation::<T> {
                attr_type: AttributeType::Passport,
                issuer: issuer.clone(),
                standard_version: PassportStandardVersion::<T>::get(),
                hash,
                salt: None,
                format_ok,
                issued_at: now,
                valid_until: Self::add_blocks(now, valid),
                grace_until: Self::add_blocks(Self::add_blocks(now, valid), grace),
                anchor,
                status: AttestationStatus::Active,
            };
            // Remove stale hash index entry if passport was previously issued
            if let Some(old_att) = PassportAttestations::<T>::get(id) {
                if old_att.hash != hash {
                    PassportHashIndex::<T>::remove(old_att.hash);
                }
            }
            PassportAttestations::<T>::insert(id, att);
            PassportHashIndex::<T>::insert(hash, id);
            Self::append_history(id, AttributeType::Passport, HistoryAction::Issued);
            Self::deposit_event(Event::Attested { identity: id, attr: AttributeType::Passport as u8, issuer });
            Ok(())
        }

        /// Issuer: issue Biometrics attestation (optional issuer set)
    #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::issue_attestation())]
        pub fn issue_biometrics(
            origin: OriginFor<T>,
            target: T::AccountId,
            anchor: BoundedVec<u8, T::MaxAnchorLen>,
        ) -> DispatchResult {
            let issuer = ensure_signed(origin)?;
            ensure!(Self::is_authorized_issuer(AttributeType::Biometrics, &issuer), Error::<T>::NotAuthorizedIssuer);
            ensure!(!FlaggedIssuers::<T>::get(AttributeType::Biometrics, issuer.clone()), Error::<T>::IssuerFlagged);
            Self::ensure_not_paused()?;
            Self::check_and_bump_rate(AttributeType::Biometrics, &issuer)?;
            let id = IdentityOf::<T>::get(&target).ok_or(Error::<T>::IdentityNotFound)?;
            let now = Self::now();
            let (valid, grace) = Self::validity_windows();
            let att = Attestation::<T> {
                attr_type: AttributeType::Biometrics,
                issuer: issuer.clone(),
                standard_version: 1,
                hash: H256::zero(),
                salt: None,
                format_ok: true,
                issued_at: now,
                valid_until: Self::add_blocks(now, valid),
                grace_until: Self::add_blocks(Self::add_blocks(now, valid), grace),
                anchor,
                status: AttestationStatus::Active,
            };
            BiometricAttestations::<T>::insert(id, att);
            Self::append_history(id, AttributeType::Biometrics, HistoryAction::Issued);
            Self::deposit_event(Event::Attested { identity: id, attr: AttributeType::Biometrics as u8, issuer });
            Ok(())
        }

        /// Revoke an attribute attestation (governance or councils)
    #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::revoke())]
        pub fn revoke(origin: OriginFor<T>, account: T::AccountId, attr: u8) -> DispatchResult {
            T::RevokeOrigin::ensure_origin(origin)?;
            let id = IdentityOf::<T>::get(&account).ok_or(Error::<T>::IdentityNotFound)?;
            match AttributeType::from(attr) {
                AttributeType::Ssn => {
                    SsnAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> {
                        let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?;
                        att.status = AttestationStatus::Revoked;
                        // Release the hash index so the credential can be re-issued
                        SsnHashIndex::<T>::remove(att.hash);
                        Ok(())
                    })?;
                }
                AttributeType::Passport => {
                    PassportAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> {
                        let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?;
                        att.status = AttestationStatus::Revoked;
                        PassportHashIndex::<T>::remove(att.hash);
                        Ok(())
                    })?;
                }
                AttributeType::Biometrics => {
                    BiometricAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> {
                        let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?;
                        att.status = AttestationStatus::Revoked;
                        Ok(())
                    })?;
                }
            }
            Self::append_history(id, AttributeType::from(attr), HistoryAction::Revoked);
            Self::deposit_event(Event::Revoked { identity: id, attr });
            Ok(())
        }

        /// Suspend an attribute (temporarily disable)
    #[pallet::call_index(14)]
        #[pallet::weight(T::WeightInfo::revoke())]
        pub fn suspend(origin: OriginFor<T>, account: T::AccountId, attr: u8) -> DispatchResult {
            T::RevokeOrigin::ensure_origin(origin)?;
            let id = IdentityOf::<T>::get(&account).ok_or(Error::<T>::IdentityNotFound)?;
            match AttributeType::from(attr) {
                AttributeType::Ssn => SsnAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> { let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?; att.status = AttestationStatus::Suspended; SsnHashIndex::<T>::remove(att.hash); Ok(()) })?,
                AttributeType::Passport => PassportAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> { let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?; att.status = AttestationStatus::Suspended; PassportHashIndex::<T>::remove(att.hash); Ok(()) })?,
                AttributeType::Biometrics => BiometricAttestations::<T>::try_mutate(id, |a| -> Result<(), DispatchError> { let att = a.as_mut().ok_or(Error::<T>::NoAttestation)?; att.status = AttestationStatus::Suspended; Ok(()) })?,
            }
            Self::append_history(id, AttributeType::from(attr), HistoryAction::Suspended);
            Self::deposit_event(Event::Suspended { identity: id, attr });
            Ok(())
        }

        /// Issuer self-bond deposit for a specific attribute (must be >= configured bond)
    #[pallet::call_index(15)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn issuer_deposit_bond(origin: OriginFor<T>, attr: u8) -> DispatchResult {
            let issuer = ensure_signed(origin)?;
            let attr_e = AttributeType::from(attr);
            let required = IssuerBondAmount::<T>::get();
            if required.is_zero() { return Ok(()); }
            let current = IssuerBonds::<T>::get(attr_e, issuer.clone());
            if current >= required { return Ok(()); }
            let diff = required - current;
            T::Currency::transfer(&issuer, &Self::account_id(), diff, ExistenceRequirement::KeepAlive)?;
            IssuerBonds::<T>::insert(attr_e, issuer.clone(), required);
            Self::deposit_event(Event::IssuerBondDeposited { attr, issuer, amount: required });
            Ok(())
        }

        /// Issuer withdraws bond (must not be currently authorized or flagged)
    #[pallet::call_index(16)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn issuer_withdraw_bond(origin: OriginFor<T>, attr: u8) -> DispatchResult {
            let issuer = ensure_signed(origin)?;
            let attr_e = AttributeType::from(attr);
            // cannot be currently authorized
            ensure!(!Self::is_authorized_issuer(attr_e, &issuer), Error::<T>::CannotWithdrawWhileAuthorized);
            ensure!(!FlaggedIssuers::<T>::get(attr_e, issuer.clone()), Error::<T>::IssuerFlagged);
            let amount = IssuerBonds::<T>::get(attr_e, issuer.clone());
            ensure!(!amount.is_zero(), Error::<T>::BondNotFound);
            T::Currency::transfer(&Self::account_id(), &issuer, amount, ExistenceRequirement::KeepAlive)?;
            IssuerBonds::<T>::remove(attr_e, issuer.clone());
            Self::deposit_event(Event::IssuerBondWithdrawn { attr, issuer, amount });
            Ok(())
        }

        /// Admin: flag/unflag issuer (cannot issue when flagged)
    #[pallet::call_index(17)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn flag_issuer(origin: OriginFor<T>, attr: u8, issuer: T::AccountId, flagged: bool) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            FlaggedIssuers::<T>::insert(AttributeType::from(attr), issuer.clone(), flagged);
            Self::deposit_event(Event::IssuerFlagged { attr, issuer, flagged });
            Ok(())
        }

        /// Admin: slash issuer bond, funds transferred to Treasury
    #[pallet::call_index(18)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn slash_issuer_bond(origin: OriginFor<T>, attr: u8, issuer: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            let attr_e = AttributeType::from(attr);
            let current = IssuerBonds::<T>::get(attr_e, issuer.clone());
            ensure!(current >= amount, Error::<T>::BondInsufficient);
            let new_bal = current - amount;
            IssuerBonds::<T>::insert(attr_e, issuer.clone(), new_bal);
            T::Currency::transfer(&Self::account_id(), &T::Treasury::get(), amount, ExistenceRequirement::KeepAlive)?;
            Self::deposit_event(Event::IssuerBondSlashed { attr, issuer, amount });
            Ok(())
        }

        /// Admin: convenience action to flag an issuer for bad attestation and optionally slash
        #[pallet::call_index(19)]
        #[pallet::weight(T::WeightInfo::admin_simple())]
        pub fn report_bad_attestation(
            origin: OriginFor<T>,
            attr: u8,
            issuer: T::AccountId,
            flag: bool,
            slash_amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            // flag/unflag
            FlaggedIssuers::<T>::insert(AttributeType::from(attr), issuer.clone(), flag);
            Self::deposit_event(Event::IssuerFlagged { attr, issuer: issuer.clone(), flagged: flag });
            // optional slash
            if !slash_amount.is_zero() {
                let attr_e = AttributeType::from(attr);
                let current = IssuerBonds::<T>::get(attr_e, issuer.clone());
                ensure!(current >= slash_amount, Error::<T>::BondInsufficient);
                let new_bal = current - slash_amount;
                IssuerBonds::<T>::insert(attr_e, issuer.clone(), new_bal);
                T::Currency::transfer(&Self::account_id(), &T::Treasury::get(), slash_amount, ExistenceRequirement::KeepAlive)?;
                Self::deposit_event(Event::IssuerBondSlashed { attr, issuer, amount: slash_amount });
            }
            Ok(())
        }
    }

    // Helper trait for KYC checks to be used by other pallets (e.g., Compliance, DEX)
    pub trait BelizeKyc<AccountId, BlockNumber> {
        fn is_kyc_verified(who: &AccountId, level: KycLevel, now: BlockNumber) -> bool;
        /// Grace-aware KYC check: returns true for both Valid and Grace states.
        /// Consumer pallets that tolerate a grace period should call this instead
        /// of `is_kyc_verified` to avoid hard-rejecting users during attestation renewal.
        fn is_kyc_verified_or_grace(who: &AccountId, level: KycLevel, now: BlockNumber) -> bool {
            // Default: fall back to strict check (backward compatible)
            Self::is_kyc_verified(who, level, now)
        }
    }

    impl<T: Config> BelizeKyc<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
        fn is_kyc_verified(who: &T::AccountId, level: KycLevel, now: BlockNumberFor<T>) -> bool {
            let Some(id) = IdentityOf::<T>::get(who) else { return false };

            let is_valid = |att: &Attestation<T>| -> bool {
                if att.status != AttestationStatus::Active { return false; }
                now <= att.valid_until
            };

            match level {
                KycLevel::L0 => true,
                KycLevel::L1 => SsnAttestations::<T>::get(id).as_ref().map(&is_valid).unwrap_or(false),
                KycLevel::L2 => {
                    let ssn_ok = SsnAttestations::<T>::get(id).as_ref().map(&is_valid).unwrap_or(false);
                    let pass_ok = PassportAttestations::<T>::get(id).as_ref().map(&is_valid).unwrap_or(false);
                    ssn_ok && pass_ok
                }
                KycLevel::L3 => {
                    let l2 = {
                        let ssn_ok = SsnAttestations::<T>::get(id).as_ref().map(&is_valid).unwrap_or(false);
                        let pass_ok = PassportAttestations::<T>::get(id).as_ref().map(&is_valid).unwrap_or(false);
                        ssn_ok && pass_ok
                    };
                    let bio_ok = BiometricAttestations::<T>::get(id).as_ref().map(is_valid).unwrap_or(false);
                    l2 && bio_ok
                }
            }
        }

        fn is_kyc_verified_or_grace(who: &T::AccountId, level: KycLevel, now: BlockNumberFor<T>) -> bool {
            matches!(Self::kyc_state(who, level, now), KycState::Valid | KycState::Grace)
        }
    }

    impl<T: Config> Pallet<T> {
        /// Check KYC level via on-chain attestations first, Oracle as supplement
        /// M51 FIX: On-chain attestations take priority over Oracle to prevent
        /// compromised oracle from overriding verified on-chain KYC state
        pub fn get_verified_kyc_level(who: &T::AccountId) -> Option<u8> {
            // Check on-chain attestations first (immutable, verifiable)
            let now = frame_system::Pallet::<T>::block_number();
            if Self::kyc_state(who, KycLevel::L3, now) == KycState::Valid {
                return Some(3);
            }
            if Self::kyc_state(who, KycLevel::L2, now) == KycState::Valid {
                return Some(2);
            }
            if Self::kyc_state(who, KycLevel::L1, now) == KycState::Valid {
                return Some(1);
            }
            
            // Fallback to Oracle for external verification (supplementary)
            if let Some(oracle_level) = T::Oracle::get_kyc_level(who) {
                return Some(oracle_level);
            }
            
            Some(0) // L0 - no verification
        }
        
        /// Check if account meets KYC requirement (for cross-pallet use)
        /// M51 FIX: On-chain verification takes priority over Oracle
        pub fn meets_kyc_requirement_level(who: &T::AccountId, required_level: u8) -> bool {
            // Check on-chain verification first
            let now = frame_system::Pallet::<T>::block_number();
            let kyc_level = match required_level {
                0 => KycLevel::L0,
                1 => KycLevel::L1,
                2 => KycLevel::L2,
                _ => KycLevel::L3,
            };
            if matches!(Self::kyc_state(who, kyc_level, now), KycState::Valid | KycState::Grace) {
                return true;
            }
            
            // Fallback to Oracle as supplementary verification
            if T::Oracle::meets_kyc_requirement(who, required_level) {
                return true;
            }
            
            false
        }
        
        /// Check if account is sanctioned (via Oracle).
        /// Resolves identity and checks ALL linked accounts — a sanctioned
        /// linked account taints the entire identity (MaxAccountsPerIdentity=5).
        pub fn is_account_sanctioned(who: &T::AccountId) -> bool {
            if T::Oracle::is_sanctioned(who) {
                return true;
            }
            if let Some(id) = IdentityOf::<T>::get(who) {
                if let Some(rec) = Identities::<T>::get(id) {
                    return rec.accounts.iter().any(T::Oracle::is_sanctioned);
                }
            }
            false
        }

        /// Return KYC state including grace handling (Valid, Grace, Invalid)
        pub fn kyc_state(who: &T::AccountId, level: KycLevel, now: BlockNumberFor<T>) -> KycState {
            let Some(id) = IdentityOf::<T>::get(who) else { return KycState::Invalid };
            let state_of = |att: &Attestation<T>| -> KycState {
                if att.status != AttestationStatus::Active { return KycState::Invalid; }
                if now <= att.valid_until { return KycState::Valid; }
                if now <= att.grace_until { return KycState::Grace; }
                KycState::Invalid
            };
            let combine = |a: KycState, b: KycState| -> KycState {
                // both must be Valid to be Valid; any Grace without Invalid -> Grace
                match (a, b) {
                    (KycState::Invalid, _) | (_, KycState::Invalid) => KycState::Invalid,
                    (KycState::Valid, KycState::Valid) => KycState::Valid,
                    _ => KycState::Grace,
                }
            };
            match level {
                KycLevel::L0 => KycState::Valid,
                KycLevel::L1 => SsnAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid),
                KycLevel::L2 => {
                    let ssn = SsnAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid);
                    let pass = PassportAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid);
                    combine(ssn, pass)
                }
                KycLevel::L3 => {
                    let l2 = {
                        let ssn = SsnAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid);
                        let pass = PassportAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid);
                        combine(ssn, pass)
                    };
                    let bio = BiometricAttestations::<T>::get(id).as_ref().map(state_of).unwrap_or(KycState::Invalid);
                    combine(l2, bio)
                }
            }
        }

        /// Return did:belize:<identity_id> for an account
        pub fn did_of(who: &T::AccountId) -> Option<DidOf<T>> {
            let id = IdentityOf::<T>::get(who)?;
            let mut bytes: sp_std::vec::Vec<u8> = b"did:belize:".to_vec();
            // Convert u128 to decimal string
            let mut n = id;
            let mut digits: sp_std::vec::Vec<u8> = vec![];
            if n == 0 { digits.push(b'0'); }
            while n > 0 { digits.push(b'0' + (n % 10) as u8); n /= 10; }
            digits.reverse();
            bytes.extend_from_slice(&digits);
            BoundedVec::try_from(bytes).ok()
        }
    }
}

/// Weight information
pub trait WeightInfo {
    fn register_identity() -> Weight;
    fn link_account() -> Weight;
    fn update_did() -> Weight;
    fn admin_simple() -> Weight;
    fn issue_attestation() -> Weight;
    fn revoke() -> Weight;
}

impl WeightInfo for () {
    fn register_identity() -> frame_support::weights::Weight {
        // reads: NextIdentityId, writes: NextIdentityId, Identities, IdentityOf
        Weight::from_parts(25_000_000, 512)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(1))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(3))
    }
    fn link_account() -> frame_support::weights::Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(2))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(2))
    }
    fn update_did() -> frame_support::weights::Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(1))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(1))
    }
    fn admin_simple() -> frame_support::weights::Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(1))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(1))
    }
    fn issue_attestation() -> frame_support::weights::Weight {
        Weight::from_parts(40_000_000, 1536)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(3))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(3))
    }
    fn revoke() -> frame_support::weights::Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().reads(1))
            .saturating_add(frame_support::weights::constants::RocksDbWeight::get().writes(1))
    }
}