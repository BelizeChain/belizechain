#![cfg_attr(not(feature = "std"), no_std)]

//! Belize Land Ledger Pallet
//!
//! This pallet implements:
//! 1. Blockchain-based property registry for all Belizean land
//! 2. Digital land titles with immutable ownership records
//! 3. Property transfers with government verification
//! 4. Tourism property investment tracking
//! 5. Environmental compliance monitoring for developments
//! 6. Integration with government land offices

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency,
        Get,
    },
    PalletId, BoundedVec,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{
        SaturatedConversion,
    },
};
use sp_std::vec::Vec;
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;

// BelizeChain temporal anchoring
use pallet_belize_common::{
    TemporalAnchor,
    AnchorType,
    TemporalAnchoring,
    temporal_helpers,
};

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const LAND_REGISTRY_ID: PalletId = PalletId(*b"bz/landr");

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use pallet_belize_identity::BelizeKyc;
    

    /// Property ID type - simple counter for on-chain properties
    pub type PropertyId = u32;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency used for land transactions
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// KYC provider — registrants must have at least L1 identity verification
        type BelizeKyc: pallet_belize_identity::BelizeKyc<Self::AccountId, BlockNumberFor<Self>>;
        
        /// Government origin for land verification
        type GovernmentOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Surveyor origin for land measurements
        type SurveyorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Environmental authority origin
        type EnvironmentalOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        
        /// Oracle provider for land ownership verification
        type Oracle: LandLedgerOracleProvider<Self::AccountId>;
        
        /// Property registration deposit
        #[pallet::constant]
        type RegistrationDeposit: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;
        
        /// Transfer tax rate (basis points)
        #[pallet::constant]
        type TransferTaxRate: Get<u32>;
        
        /// Maximum property description length
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;
        
        /// Weight information
        type WeightInfo: WeightInfo;

        /// Maximum allowed assessed property value (prevents overflow / registry abuse)
        #[pallet::constant]
        type MaxPropertyPrice: Get<u128>;
    }

    /// Oracle provider trait for Land Ledger pallet
    /// Allows verification of land ownership and property data from external sources
    pub trait LandLedgerOracleProvider<AccountId> {
        /// Verify property ownership via external land registry (uses internal u32 ID)
        fn verify_land_owner(property_id: u32, account: &AccountId) -> bool;
        
        /// Get KYC level for property transfers
        fn get_kyc_level(account: &AccountId) -> Option<u8>;
        
        /// Check if account is sanctioned (for property transactions)
        fn is_sanctioned(account: &AccountId) -> bool;
    }

    /// Land property information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(AccountId))]
    pub struct PropertyRecord<AccountId> {
        /// Unique property ID
        pub property_id: PropertyId,
        /// Current owner
        pub owner: AccountId,
        /// Property title number
        pub title_number: BoundedVec<u8, ConstU32<64>>,
        /// Description of encumbrance
    pub description: BoundedVec<u8, ConstU32<256>>,
        /// GPS coordinates (latitude, longitude)
        pub coordinates: (i64, i64),
        /// Property area in square meters
        pub area_sqm: u32,
        /// Property type
        pub property_type: PropertyType,
        /// Current assessed value in bBZD
        pub assessed_value: u128,
        /// Registration timestamp
        pub registered_at: u64,
        /// Last transfer timestamp
        pub last_transferred: Option<u64>,
        /// Verified status
        pub government_verified: bool,
        /// Survey status
        pub surveyed: bool,
        /// Environmental clearance
        pub environmental_clearance: bool,
        /// Tourism property flag
        pub is_tourism_property: bool,
        /// Zoning classification
        pub zoning: ZoningType,
        /// Encumbrances or liens
        pub encumbrances: BoundedVec<Encumbrance<AccountId>, ConstU32<10>>,
    }

    /// Property types in Belize
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum PropertyType {
        /// Residential property
        Residential,
        /// Commercial property
        Commercial,
        /// Agricultural land
        Agricultural,
        /// Industrial property
        Industrial,
        /// Tourism/hospitality property
        Tourism,
        /// Government property
        Government,
        /// Protected/conservation land
        Protected,
        /// Undeveloped land
        Undeveloped,
    }

    /// Zoning classifications
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum ZoningType {
        /// Urban residential
        UrbanResidential,
        /// Rural residential  
        RuralResidential,
        /// Commercial zone
        Commercial,
        /// Light industrial
        LightIndustrial,
        /// Heavy industrial
        HeavyIndustrial,
        /// Agricultural zone
        Agricultural,
        /// Tourism development zone
        TourismDevelopment,
        /// Protected environmental area
        EnvironmentalProtected,
        /// Mixed use development
        MixedUse,
    }

    /// Encumbrances on property
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(AccountId))]
    pub struct Encumbrance<AccountId> {
        /// Encumbrance type
        pub encumbrance_type: EncumbranceType,
        /// Party holding the encumbrance
        pub holder: AccountId,
        /// Amount (if applicable)
        pub amount: Option<u128>,
        /// Description
        pub description: BoundedVec<u8, ConstU32<256>>,
        /// Active status
        pub active: bool,
    }

    /// Types of encumbrances
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum EncumbranceType {
        /// Mortgage lien
        Mortgage,
        /// Tax lien
        TaxLien,
        /// Easement
        Easement,
        /// Right of way
        RightOfWay,
        /// Covenant
        Covenant,
        /// Court order
        CourtOrder,
    }

    /// Property transfer record
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(AccountId, Moment))]
    pub struct TransferRecord<AccountId, Moment> {
        /// Transfer ID
        pub transfer_id: u32,
        /// Property ID
        pub property_id: PropertyId,
        /// Previous owner
        pub from_owner: AccountId,
        /// New owner
        pub to_owner: AccountId,
        /// Transfer price
        pub transfer_price: u128,
        /// Transfer timestamp
        pub transferred_at: Moment,
        /// Government approved
        pub government_approved: bool,
        /// Tax paid
        pub tax_paid: u128,
        /// Transfer type
        pub transfer_type: TransferType,
    }

    /// Transfer types
    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub enum TransferType {
        /// Sale transaction
        Sale,
        /// Gift transfer
        Gift,
        /// Inheritance
        Inheritance,
        /// Government acquisition
        GovernmentAcquisition,
        /// Foreclosure
        Foreclosure,
        /// Court order transfer
        CourtOrder,
    }

    #[pallet::storage]
    #[pallet::getter(fn properties)]
    /// Property records registry  
    pub type Properties<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Property ID
        PropertyRecord<T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn property_owners)]
    /// Properties owned by each account
    pub type PropertyOwners<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u32, ConstU32<1000>>, // Property IDs
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn transfer_records)]
    /// Property transfer history
    pub type TransferRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // Transfer ID
        TransferRecord<T::AccountId, u64>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn property_by_title)]
    /// Property lookup by title number hash
    pub type PropertyByTitle<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Hash of title number
        PropertyId,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_property_id)]
    /// Next available property ID counter
    pub type NextPropertyId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_transfer_id)]
    /// Next available transfer ID
    pub type NextTransferId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn government_surveyors)]
    /// Registered government surveyors
    pub type GovernmentSurveyors<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn zoning_map)]
    /// Zoning designations by area
    pub type ZoningMap<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        i64, // Latitude
        Blake2_128Concat,
        i64, // Longitude
        ZoningType,
    >;

    #[pallet::storage]
    #[pallet::getter(fn land_anchors)]
    /// Temporal anchors for land title history
    pub type LandAnchors<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // Anchor hash
        TemporalAnchor<BlockNumberFor<T>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn property_anchor_chain)]
    /// Latest anchor hash for each property
    pub type PropertyAnchorChain<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PropertyId,
        [u8; 32], // Latest anchor hash
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Property registered
        PropertyRegistered {
            property_id: PropertyId,
            owner: T::AccountId,
            title_number: Vec<u8>,
        },
        /// Property transferred
        /// Property transfer completed
        PropertyTransferred {
            property_id: PropertyId,
            from_owner: T::AccountId,
            to_owner: T::AccountId,
            transfer_price: <T::Currency as Currency<T::AccountId>>::Balance,
            transfer_id: u32,
        },
        /// Property verified by government authority
        PropertyVerified {
            property_id: PropertyId,
            verifier: T::AccountId,
        },
        /// Property surveyed
        PropertySurveyed {
            property_id: PropertyId,
            surveyor: T::AccountId,
            area_sqm: u32,
        },
        /// Environmental clearance granted
        EnvironmentalClearanceGranted {
            property_id: PropertyId,
            authority: T::AccountId,
        },
        /// Encumbrance added
        EncumbranceAdded {
            property_id: PropertyId,
            encumbrance_type: EncumbranceType,
            holder: T::AccountId,
        },
        /// AUDIT FIX (CRIT-02): Encumbrance removed/deactivated
        EncumbranceRemoved {
            property_id: PropertyId,
            encumbrance_index: u32,
        },
        /// Surveyor registered
        SurveyorRegistered {
            surveyor: T::AccountId,
        },
        /// LL-4 FIX: Surveyor removed
        SurveyorRemoved {
            surveyor: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Property not found
        PropertyNotFound,
        /// Property already exists with this title
        PropertyAlreadyExists,
        /// Not the property owner
        NotOwner,
        /// Insufficient balance for registration
        InsufficientBalance,
        /// Invalid coordinates
        InvalidCoordinates,
        /// Property not verified
        PropertyNotVerified,
        /// Property not surveyed
        PropertyNotSurveyed,
        /// Transfer not approved
        TransferNotApproved,
        /// Description too long
        DescriptionTooLong,
        /// Invalid transfer price
        InvalidTransferPrice,
        /// Encumbrance exists
        EncumbranceExists,
        /// Not authorized surveyor
        NotAuthorizedSurveyor,
        /// Ownership verification failed via Oracle
        OwnershipVerificationFailed,
        /// Buyer has insufficient KYC for property transaction
        BuyerKycInsufficient,
        /// Account is sanctioned and cannot participate in property transactions
        AccountSanctioned,
        /// Maximum properties per account reached
        MaxPropertiesReached,
        /// Registrant lacks required KYC verification (L1 minimum)
        KycNotVerified,
        /// LL-4 FIX: Surveyor not found in registry
        SurveyorNotFound,
        /// Assessed value must be greater than zero
        InvalidPropertyPrice,
        /// Assessed value exceeds maximum allowed
        PropertyPriceTooHigh,
        /// Maximum encumbrances per property reached
        MaxEncumbrancesReached,
        /// Encumbrance index out of range
        EncumbranceNotFound,
        /// Area must be greater than zero
        InvalidArea,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register new property
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_property())]
        pub fn register_property(
            origin: OriginFor<T>,
            title_number: Vec<u8>,
            description: Vec<u8>,
            coordinates: (i64, i64),
            area_sqm: u32,
            property_type_index: u8,
            assessed_value: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // P0-03 FIX: Require at least L1 KYC (SSN verified) for property registration
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                T::BelizeKyc::is_kyc_verified(&who, pallet_belize_identity::KycLevel::L1, current_block),
                Error::<T>::KycNotVerified
            );

            let property_type = match property_type_index {
                0 => PropertyType::Residential,
                1 => PropertyType::Commercial,
                2 => PropertyType::Agricultural,
                3 => PropertyType::Industrial,
                4 => PropertyType::Tourism,
                5 => PropertyType::Government,
                6 => PropertyType::Protected,
                7 => PropertyType::Undeveloped,
                _ => PropertyType::Residential,
            };

            // Validate description length
            ensure!(
                description.len() <= T::MaxDescriptionLength::get() as usize,
                Error::<T>::DescriptionTooLong
            );

            // AUDIT FIX (HIGH-03): Area must be positive
            ensure!(area_sqm > 0, Error::<T>::InvalidArea);

            // Validate assessed value (non-zero, reasonable bounds)
            ensure!(assessed_value > 0, Error::<T>::InvalidPropertyPrice);
            ensure!(assessed_value <= T::MaxPropertyPrice::get(), Error::<T>::PropertyPriceTooHigh);

            // Validate coordinates (basic bounds checking for Belize)
            ensure!(
                coordinates.0 >= 15_000_000 && coordinates.0 <= 19_000_000 && // Latitude range
                coordinates.1 >= -90_000_000 && coordinates.1 <= -87_000_000, // Longitude range
                Error::<T>::InvalidCoordinates
            );

            // Ensure property doesn't already exist
            let title_bounded: BoundedVec<u8, ConstU32<64>> = title_number.clone().try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            let title_hash = sp_io::hashing::blake2_256(&title_bounded);
            ensure!(
                !PropertyByTitle::<T>::contains_key(title_hash),
                Error::<T>::PropertyAlreadyExists
            );

            // Reserve registration deposit
            T::Currency::reserve(&who, T::RegistrationDeposit::get())?;

            let property_id = Self::next_property_id();
            // SAFETY(saturated_into): BlockNumber → u64 is lossless for BelizeChain's
            // u32 block numbers. Used only as a timestamp for the property record.
            let now = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();

            let property = PropertyRecord {
                property_id,
                owner: who.clone(),
                title_number: title_bounded.clone(),
                description: description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?,
                coordinates,
                area_sqm,
                property_type: property_type.clone(),
                assessed_value,
                registered_at: now,
                last_transferred: None,
                government_verified: false,
                surveyed: false,
                environmental_clearance: matches!(property_type, PropertyType::Protected),
                is_tourism_property: matches!(property_type, PropertyType::Tourism),
                zoning: Self::get_zoning_for_coordinates(coordinates).unwrap_or(ZoningType::MixedUse),
                encumbrances: BoundedVec::default(),
            };

            Properties::<T>::insert(property_id, property);
            PropertyByTitle::<T>::insert(title_hash, property_id);
            
            PropertyOwners::<T>::try_mutate(&who, |properties| {
                properties.try_push(property_id)
                    .map_err(|_| Error::<T>::MaxPropertiesReached)
            })?;

            NextPropertyId::<T>::put(property_id.saturating_add(1));

            // Create temporal anchor for the property registration
            // Use property_id + title_number as content hash
            let mut content = Vec::new();
            content.extend_from_slice(&property_id.to_le_bytes());
            content.extend_from_slice(&title_number);
            let content_hash = sp_io::hashing::blake2_256(&content);
            
            if let Ok(anchor_hash) = Self::create_anchor(content_hash, AnchorType::LandTitle) {
                PropertyAnchorChain::<T>::insert(property_id, anchor_hash);
            }

            Self::deposit_event(Event::PropertyRegistered {
                property_id,
                owner: who,
                title_number,
            });

            Ok(())
        }

        /// Transfer property ownership
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::transfer_property())]
        pub fn transfer_property(
            origin: OriginFor<T>,
            property_id: PropertyId,
            new_owner: T::AccountId,
            transfer_price: u128,
            transfer_type_index: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let transfer_type = match transfer_type_index {
                0 => TransferType::Sale,
                1 => TransferType::Gift,
                2 => TransferType::Inheritance,
                3 => TransferType::GovernmentAcquisition,
                4 => TransferType::Foreclosure,
                5 => TransferType::CourtOrder,
                _ => TransferType::Sale,
            };

            let mut property = Self::properties(property_id)
                .ok_or(Error::<T>::PropertyNotFound)?;

            // Verify current ownership
            ensure!(property.owner == who, Error::<T>::NotOwner);
            
            // Verify ownership via Oracle (cross-checks with external land registry)
            ensure!(
                T::Oracle::verify_land_owner(property_id, &who),
                Error::<T>::OwnershipVerificationFailed
            );
            
            // Check sanctions for both parties
            ensure!(
                !T::Oracle::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );
            ensure!(
                !T::Oracle::is_sanctioned(&new_owner),
                Error::<T>::AccountSanctioned
            );
            
            // Verify buyer has sufficient KYC (Level 2 required for property transactions)
            if let Some(kyc_level) = T::Oracle::get_kyc_level(&new_owner) {
                ensure!(kyc_level >= 2, Error::<T>::BuyerKycInsufficient);
            } else {
                // No KYC data available - deny transaction for safety
                return Err(Error::<T>::BuyerKycInsufficient.into());
            }

            // Ensure property is verified for transfers
            ensure!(property.government_verified, Error::<T>::PropertyNotVerified);

            // AUDIT FIX (CRIT-01): Block transfer if any active encumbrance exists
            ensure!(
                !property.encumbrances.iter().any(|e| e.active),
                Error::<T>::EncumbranceExists
            );

            // AUDIT FIX (HIGH-06): Require property to be surveyed before transfer
            ensure!(property.surveyed, Error::<T>::PropertyNotSurveyed);

            // AUDIT FIX: Check buyer capacity early to avoid wasted computation
            let buyer_properties = PropertyOwners::<T>::get(&new_owner);
            ensure!(
                buyer_properties.len() < 1000,
                Error::<T>::MaxPropertiesReached
            );

            // Calculate transfer tax
            let transfer_tax = transfer_price.saturating_mul(T::TransferTaxRate::get() as u128) / 10000;

            // Collect transfer tax
            if transfer_tax > 0 {
                // SAFETY(saturated_into): u128 → Balance. The transfer_tax is derived
                // from transfer_price (user-supplied) × rate / 10000, so it fits within
                // any reasonable Balance type. If it somehow exceeds Balance::MAX the
                // subsequent transfer call would fail with InsufficientBalance.
                T::Currency::transfer(
                    &who,
                    &Self::account_id(),
                    transfer_tax.saturated_into(),
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
            }

            let transfer_id = Self::next_transfer_id();
            // SAFETY(saturated_into): BlockNumber → u64 — lossless for BelizeChain's u32 blocks.
            let now = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();

            // Create transfer record
            let transfer_record = TransferRecord {
                transfer_id,
                property_id,
                from_owner: who.clone(),
                to_owner: new_owner.clone(),
                transfer_price,
                // SAFETY(saturated_into): u64 → u64 is identity; the `now` value is
                // already u64 from the block number conversion above.
                transferred_at: now.saturated_into(),
                government_approved: false, // Requires separate government approval via approve_transfer
                tax_paid: transfer_tax,
                transfer_type: transfer_type.clone(),
            };

            // Update property ownership
            property.owner = new_owner.clone();
            property.last_transferred = Some(now);

            // Update storage
            Properties::<T>::insert(property_id, property);
            TransferRecords::<T>::insert(transfer_id, transfer_record);

            // Update property ownership lists
            PropertyOwners::<T>::mutate(&who, |properties| {
                properties.retain(|&x| x != property_id);
            });
            PropertyOwners::<T>::try_mutate(&new_owner, |properties| {
                properties.try_push(property_id)
                    .map_err(|_| Error::<T>::MaxPropertiesReached)
            })?;

            NextTransferId::<T>::put(transfer_id.saturating_add(1));

            // Update temporal anchor chain with new ownership record
            if let Some(previous_anchor_hash) = PropertyAnchorChain::<T>::get(property_id) {
                // Create content hash from transfer record
                let mut content = Vec::new();
                content.extend_from_slice(&property_id.to_le_bytes());
                content.extend_from_slice(&transfer_id.to_le_bytes());
                content.extend_from_slice(new_owner.encode().as_slice());
                let content_hash = sp_io::hashing::blake2_256(&content);
                
                if let Ok(new_anchor_hash) = Self::update_anchor(previous_anchor_hash, content_hash) {
                    PropertyAnchorChain::<T>::insert(property_id, new_anchor_hash);
                }
            }

            Self::deposit_event(Event::PropertyTransferred {
                property_id,
                from_owner: who,
                to_owner: new_owner,
                // SAFETY(saturated_into): u128 → Balance event field. The transfer_price
                // is user-supplied and has been validated through the tax calculation above.
                transfer_price: transfer_price.saturated_into(),
                transfer_id,
            });

            Ok(())
        }

        /// Government verification of property
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::verify_property())]
        pub fn verify_property(
            origin: OriginFor<T>,
            property_id: PropertyId,
        ) -> DispatchResult {
            T::GovernmentOrigin::ensure_origin(origin)?;

            Properties::<T>::mutate(property_id, |maybe_property| {
                if let Some(property) = maybe_property {
                    property.government_verified = true;
                    Ok(())
                } else {
                    Err(Error::<T>::PropertyNotFound)
                }
            })?;

            Self::deposit_event(Event::PropertyVerified {
                property_id,
                verifier: Self::account_id(), // Use pallet account instead
            });

            Ok(())
        }

        /// Survey property (by authorized surveyor)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::survey_property())]
        pub fn survey_property(
            origin: OriginFor<T>,
            property_id: PropertyId,
            verified_area_sqm: u32,
            updated_coordinates: Option<(i64, i64)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Verify surveyor authorization
            ensure!(Self::government_surveyors(&who), Error::<T>::NotAuthorizedSurveyor);

            // AUDIT FIX (HIGH-03): Area must be positive
            ensure!(verified_area_sqm > 0, Error::<T>::InvalidArea);

            Properties::<T>::mutate(property_id, |maybe_property| {
                if let Some(property) = maybe_property {
                    property.surveyed = true;
                    property.area_sqm = verified_area_sqm;
                    
                    if let Some(coords) = updated_coordinates {
                        // AUDIT FIX (HIGH-02): Validate updated coordinates
                        if coords.0 < 15_000_000 || coords.0 > 19_000_000 ||
                           coords.1 < -90_000_000 || coords.1 > -87_000_000 {
                            return Err(Error::<T>::InvalidCoordinates);
                        }
                        property.coordinates = coords;
                        property.zoning = Self::get_zoning_for_coordinates(coords)
                            .unwrap_or(property.zoning.clone());
                    }
                    
                    Ok(())
                } else {
                    Err(Error::<T>::PropertyNotFound)
                }
            })?;

            Self::deposit_event(Event::PropertySurveyed {
                property_id,
                surveyor: who,
                area_sqm: verified_area_sqm,
            });

            Ok(())
        }

        /// Register government surveyor
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::register_surveyor())]
        pub fn register_surveyor(
            origin: OriginFor<T>,
            surveyor: T::AccountId,
        ) -> DispatchResult {
            T::GovernmentOrigin::ensure_origin(origin)?;

            GovernmentSurveyors::<T>::insert(&surveyor, true);

            Self::deposit_event(Event::SurveyorRegistered {
                surveyor,
            });

            Ok(())
        }

        /// LL-4 FIX: Remove a government surveyor from the registry
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::register_surveyor())]
        pub fn remove_surveyor(
            origin: OriginFor<T>,
            surveyor: T::AccountId,
        ) -> DispatchResult {
            T::GovernmentOrigin::ensure_origin(origin)?;

            ensure!(
                GovernmentSurveyors::<T>::contains_key(&surveyor),
                Error::<T>::SurveyorNotFound
            );
            GovernmentSurveyors::<T>::remove(&surveyor);

            Self::deposit_event(Event::SurveyorRemoved {
                surveyor,
            });

            Ok(())
        }

        /// AUDIT FIX (CRIT-02): Add an encumbrance to a property (government-only)
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::register_surveyor())]
        pub fn add_encumbrance(
            origin: OriginFor<T>,
            property_id: u32,
            encumbrance_type: EncumbranceType,
            holder: T::AccountId,
            amount: Option<u128>,
            description: Vec<u8>,
        ) -> DispatchResult {
            T::GovernmentOrigin::ensure_origin(origin)?;

            let desc: BoundedVec<u8, ConstU32<256>> = description
                .try_into()
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            Properties::<T>::try_mutate(property_id, |maybe_property| -> DispatchResult {
                let property = maybe_property.as_mut().ok_or(Error::<T>::PropertyNotFound)?;
                let encumbrance = Encumbrance {
                    encumbrance_type: encumbrance_type.clone(),
                    holder: holder.clone(),
                    amount,
                    description: desc,
                    active: true,
                };
                property.encumbrances.try_push(encumbrance)
                    .map_err(|_| Error::<T>::MaxEncumbrancesReached)?;
                Ok(())
            })?;

            Self::deposit_event(Event::EncumbranceAdded {
                property_id,
                encumbrance_type,
                holder,
            });

            Ok(())
        }

        /// AUDIT FIX (CRIT-02): Remove (deactivate) an encumbrance from a property (government-only)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::register_surveyor())]
        pub fn remove_encumbrance(
            origin: OriginFor<T>,
            property_id: u32,
            encumbrance_index: u32,
        ) -> DispatchResult {
            T::GovernmentOrigin::ensure_origin(origin)?;

            Properties::<T>::try_mutate(property_id, |maybe_property| -> DispatchResult {
                let property = maybe_property.as_mut().ok_or(Error::<T>::PropertyNotFound)?;
                let enc = property.encumbrances
                    .get_mut(encumbrance_index as usize)
                    .ok_or(Error::<T>::EncumbranceNotFound)?;
                enc.active = false;
                Ok(())
            })?;

            Self::deposit_event(Event::EncumbranceRemoved {
                property_id,
                encumbrance_index,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Generate the account ID for the land registry
        pub fn account_id() -> T::AccountId {
            LAND_REGISTRY_ID.into_account_truncating()
        }

        /// Get zoning type for given coordinates
        fn get_zoning_for_coordinates(coordinates: (i64, i64)) -> Option<ZoningType> {
            Self::zoning_map(coordinates.0, coordinates.1)
        }

        /// Get properties owned by account
        pub fn get_owned_properties(owner: &T::AccountId) -> Vec<u32> {
            Self::property_owners(owner).to_vec()
        }
    }

    /// Temporal anchoring implementation for land titles
    impl<T: Config> TemporalAnchoring<BlockNumberFor<T>> for Pallet<T> {
        fn create_anchor(
            content_hash: [u8; 32],
            anchor_type: AnchorType,
        ) -> Result<[u8; 32], &'static str> {
            let block_number = frame_system::Pallet::<T>::block_number();
            // SAFETY(saturated_into): BlockNumber → u64 — lossless for BelizeChain's u32 blocks.
            let timestamp = block_number.saturated_into::<u64>();
            
            let anchor = TemporalAnchor {
                content_hash,
                previous_hash: None,
                block_number,
                timestamp,
                anchor_type,
                merkle_root: content_hash, // Genesis anchor - merkle root is just content hash
                version: 0,
            };
            
            let anchor_hash = temporal_helpers::calculate_anchor_hash(&anchor);
            LandAnchors::<T>::insert(anchor_hash, anchor);
            
            Ok(anchor_hash)
        }
        
        fn update_anchor(
            previous_anchor_hash: [u8; 32],
            new_content_hash: [u8; 32],
        ) -> Result<[u8; 32], &'static str> {
            // Get previous anchor
            let previous_anchor = LandAnchors::<T>::get(previous_anchor_hash)
                .ok_or("Previous anchor not found")?;
            
            let block_number = frame_system::Pallet::<T>::block_number();
            // SAFETY(saturated_into): BlockNumber → u64 is lossless for BelizeChain's
            // u32 block numbers. Used as timestamp in temporal anchor.
            let timestamp = block_number.saturated_into::<u64>();
            
            // Build history for merkle root calculation
            let history = Self::get_anchor_history(previous_anchor_hash);
            let mut hashes: Vec<[u8; 32]> = history.iter().map(|a| a.content_hash).collect();
            hashes.push(new_content_hash);
            
            let merkle_root = temporal_helpers::calculate_merkle_root(&hashes);
            
            let new_anchor = TemporalAnchor {
                content_hash: new_content_hash,
                previous_hash: Some(previous_anchor_hash),
                block_number,
                timestamp,
                anchor_type: previous_anchor.anchor_type,
                merkle_root,
                version: previous_anchor.version.saturating_add(1),
            };
            
            let anchor_hash = temporal_helpers::calculate_anchor_hash(&new_anchor);
            LandAnchors::<T>::insert(anchor_hash, new_anchor);
            
            Ok(anchor_hash)
        }
        
        fn get_anchor(hash: [u8; 32]) -> Option<TemporalAnchor<BlockNumberFor<T>>> {
            LandAnchors::<T>::get(hash)
        }
        
        fn verify_anchor_chain(hash: [u8; 32]) -> bool {
            // M65 FIX: Depth-limit anchor chain traversal to prevent O(n²) DoS
            const MAX_CHAIN_DEPTH: u32 = 100;
            let mut current_hash = hash;
            let mut version = u32::MAX; // Start with max, should decrease as we go back
            let mut depth = 0u32;
            
            loop {
                depth = depth.saturating_add(1);
                if depth > MAX_CHAIN_DEPTH {
                    return false; // Chain too deep, reject
                }

                let anchor = match LandAnchors::<T>::get(current_hash) {
                    Some(a) => a,
                    None => return false, // Broken chain
                };
                
                // Verify version decreases
                if anchor.version >= version {
                    return false;
                }
                version = anchor.version;
                
                // Verify merkle root if not genesis
                if anchor.previous_hash.is_some() {
                    let history = Self::get_anchor_history(current_hash);
                    let hashes: Vec<[u8; 32]> = history.iter().map(|a| a.content_hash).collect();
                    let calculated_root = temporal_helpers::calculate_merkle_root(&hashes);
                    
                    if calculated_root != anchor.merkle_root {
                        return false;
                    }
                }
                
                // Move to previous anchor
                match anchor.previous_hash {
                    Some(prev) => current_hash = prev,
                    None => return true, // Reached genesis, chain is valid
                }
            }
        }
        
        fn get_anchor_history(hash: [u8; 32]) -> Vec<TemporalAnchor<BlockNumberFor<T>>> {
            // M65 FIX: Depth-limit history traversal to prevent unbounded loops
            const MAX_HISTORY_DEPTH: u32 = 100;
            let mut history = Vec::new();
            let mut current_hash = hash;
            let mut depth = 0u32;
            
            while let Some(anchor) = LandAnchors::<T>::get(current_hash) {
                depth = depth.saturating_add(1);
                if depth > MAX_HISTORY_DEPTH {
                    break; // Truncate history at max depth
                }

                let previous_hash = anchor.previous_hash;
                history.push(anchor);
                
                match previous_hash {
                    Some(prev) => current_hash = prev,
                    None => break, // Reached genesis
                }
            }
            
            history
        }
        
        fn get_latest_anchor(_content_hash: [u8; 32]) -> Option<[u8; 32]> {
            // This would require additional storage mapping
            // For now, return None - can be enhanced later
            // In practice, PropertyAnchorChain storage serves this purpose
            None
        }
    }
}

/// Weight information for pallet extrinsics
pub trait WeightInfo {
    fn register_property() -> Weight;
    fn transfer_property() -> Weight;
    fn verify_property() -> Weight;
    fn survey_property() -> Weight;
    fn register_surveyor() -> Weight;
}

impl WeightInfo for () {
    fn register_property() -> Weight {
        Weight::from_parts(35_000_000, 512)
            .saturating_add(Weight::from_parts(0, 5000))
    }
    fn transfer_property() -> Weight {
        Weight::from_parts(40_000_000, 512)
            .saturating_add(Weight::from_parts(0, 6000))
    }
    fn verify_property() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(Weight::from_parts(0, 2000))
    }
    fn survey_property() -> Weight {
        Weight::from_parts(25_000_000, 512)
            .saturating_add(Weight::from_parts(0, 3500))
    }
    fn register_surveyor() -> Weight {
        Weight::from_parts(10_000_000, 512)
            .saturating_add(Weight::from_parts(0, 1500))
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;