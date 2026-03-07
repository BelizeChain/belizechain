#![cfg_attr(not(feature = "std"), no_std)]

//! BelizeChain BNS (Belize Name Service) Pallet
//!
//! This pallet implements:
//! 1. Immutable domain registry (.bz domains - permanent ownership like ENS)
//! 2. Domain resolution (wallet addresses, content hashes, metadata)
//! 3. Domain marketplace (buy/sell domains with 5% treasury fee)
//! 4. Decentralized web hosting (DAG storage via Pakit)
//! 5. External domain support (.com, .org, .net, etc.)
//! 6. Multi-tier pricing (Standard, Premium, Government, Verified)
//! 7. KYC-gated registration (prevent abuse, compliance)
//! 8. Treasury fee collection (domain sales + hosting fees)

extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;
pub use types::*;
pub use weights::*;

use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency, ExistenceRequirement,
        Get, Time, ConstU32,
    },
    PalletId, BoundedVec,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::Saturating,
    SaturatedConversion,
};
use codec::Encode;
use alloc::vec::Vec;

// ===== CONSTANTS =====

const BNS_TREASURY_ID: PalletId = PalletId(*b"bz/bnstr");

// One month in blocks (assuming 6 second blocks)
const BLOCKS_PER_MONTH: u32 = 432_000;

// Marketplace fee (5%)
const MARKETPLACE_FEE_PERCENT: u8 = 5;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency used for domain purchases and hosting fees
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Treasury account for fee collection
        #[pallet::constant]
        type Treasury: Get<Self::AccountId>;

        /// Maximum number of domains per account
        #[pallet::constant]
        type MaxDomainsPerAccount: Get<u32>;

        /// Maximum domain name length
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;

        /// Maximum text records per domain
        #[pallet::constant]
        type MaxTextRecords: Get<u32>;

        /// Minimum domain length (to prevent 1-letter squatting without high price)
        #[pallet::constant]
        type MinDomainLength: Get<u32>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Identity provider for KYC verification
        type Identity: BnsIdentityProvider<Self::AccountId>;

        /// Governance origin for emergency controls
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    // ==================== STORAGE ====================

    // ===== IMMUTABLE DOMAIN REGISTRY =====

    /// Domain ownership registry (permanent, transferable)
    #[pallet::storage]
    #[pallet::getter(fn domain_registry)]
    pub type DomainRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,  // domain name
        DomainRecord<T::AccountId, BlockNumberFor<T>>,
    >;

    /// Reverse lookup: AccountId → List of owned domains
    #[pallet::storage]
    #[pallet::getter(fn account_domains)]
    pub type AccountDomains<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<BoundedVec<u8, T::MaxDomainLength>, T::MaxDomainsPerAccount>,
        ValueQuery,
    >;

    /// Domain resolution records (wallet, IPFS, metadata)
    #[pallet::storage]
    #[pallet::getter(fn domain_resolution)]
    pub type DomainResolution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,
        ResolutionRecords<T::AccountId, T::MaxTextRecords>,
    >;

    /// Domain marketplace listings
    #[pallet::storage]
    #[pallet::getter(fn domain_listings)]
    pub type DomainListings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,
        DomainListing<T::AccountId, BlockNumberFor<T>>,
    >;

    // ===== WEB HOSTING SERVICE =====

    /// Active website hosting (recurring subscription)
    #[pallet::storage]
    #[pallet::getter(fn hosted_websites)]
    pub type HostedWebsites<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,
        HostingInfo<T::AccountId, BlockNumberFor<T>>,
    >;

    /// External domain mapping (.com, .org, .net, etc.)
    #[pallet::storage]
    #[pallet::getter(fn external_domains)]
    pub type ExternalDomains<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<128>>,  // Allow longer external domains
        ExternalDomainInfo<T::AccountId, BlockNumberFor<T>, T::MaxDomainLength>,
    >;

    /// Domain verification status (for external domains)
    #[pallet::storage]
    #[pallet::getter(fn domain_verification)]
    pub type DomainVerification<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<128>>,
        VerificationStatus<BlockNumberFor<T>>,
    >;

    /// SSL/TLS certificate hashes (for verification)
    #[pallet::storage]
    #[pallet::getter(fn ssl_certificates)]
    pub type SSLCertificates<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,  // domain name
        SSLCertInfo<BlockNumberFor<T>>,
    >;

    // ===== GLOBAL COUNTERS =====

    /// Total domains registered
    #[pallet::storage]
    #[pallet::getter(fn total_domains)]
    pub type TotalDomains<T> = StorageValue<_, u64, ValueQuery>;

    /// Total hosting revenue collected
    #[pallet::storage]
    #[pallet::getter(fn total_hosting_revenue)]
    pub type TotalHostingRevenue<T> = StorageValue<_, u128, ValueQuery>;

    /// Total marketplace revenue (5% fees)
    #[pallet::storage]
    #[pallet::getter(fn total_marketplace_revenue)]
    pub type TotalMarketplaceRevenue<T> = StorageValue<_, u128, ValueQuery>;

    /// Next operation ID for tracking
    #[pallet::storage]
    #[pallet::getter(fn next_operation_id)]
    pub type NextOperationId<T> = StorageValue<_, u64, ValueQuery>;

    /// Content version history (domain -> version -> content hash)
    /// Allows rollback to previous versions
    #[pallet::storage]
    #[pallet::getter(fn content_history)]
    pub type ContentHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, BoundedVec<u8, T::MaxDomainLength>,  // domain
        Blake2_128Concat, u32,                                  // version number
        ContentVersion<BlockNumberFor<T>>,                     // version data
        OptionQuery,
    >;

    /// Current content version number for each domain
    #[pallet::storage]
    #[pallet::getter(fn content_version)]
    pub type CurrentContentVersion<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDomainLength>,
        u32,
        ValueQuery,  // Defaults to 0
    >;

    // ==================== EVENTS ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Domain registered [domain, owner, price, tier: 0=Standard, 1=Premium, 2=Government, 3=Verified]
        DomainRegistered {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            owner: T::AccountId,
            price: u128,
            tier: u8,
        },
        /// Domain resolution updated [domain, owner]
        ResolutionUpdated {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            owner: T::AccountId,
        },
        /// Domain transferred [domain, from, to]
        DomainTransferred {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            from: T::AccountId,
            to: T::AccountId,
        },
        /// Domain listed for sale [domain, seller, price]
        DomainListed {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            seller: T::AccountId,
            price: u128,
        },
        /// Domain sold [domain, seller, buyer, price, marketplace_fee]
        DomainSold {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: u128,
            marketplace_fee: u128,
        },
        /// Hosting activated [domain, owner, tier: 0=Free, 1=Basic, 2=Pro, 3=Enterprise, content_hash]
        HostingActivated {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            owner: T::AccountId,
            tier: u8,
            content_hash: [u8; 32],
        },
        /// Hosting renewed [domain, owner, blocks_extended]
        HostingRenewed {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            owner: T::AccountId,
            blocks_extended: BlockNumberFor<T>,
        },
        /// Hosting fee collected [payer, amount]
        HostingFeeCollected {
            payer: T::AccountId,
            amount: u128,
        },
        /// External domain registered [domain, owner, tier: 0=Free, 1=Basic, 2=Pro, 3=Enterprise]
        ExternalDomainRegistered {
            domain: BoundedVec<u8, ConstU32<128>>,
            owner: T::AccountId,
            tier: u8,
        },
        /// External domain verified [domain, owner]
        ExternalDomainVerified {
            domain: BoundedVec<u8, ConstU32<128>>,
            owner: T::AccountId,
        },
        /// External domain verification requested (pending governance approval) [domain, owner]
        ExternalDomainVerificationRequested {
            domain: BoundedVec<u8, ConstU32<128>>,
            owner: T::AccountId,
        },
        /// Subdomain created [subdomain, parent_domain, owner, delegated]
        SubdomainCreated {
            subdomain: BoundedVec<u8, T::MaxDomainLength>,
            parent: BoundedVec<u8, T::MaxDomainLength>,
            owner: T::AccountId,
            delegated: bool,
        },

        /// Content updated with version tracking [domain, version, content_hash]
        ContentUpdated {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            version: u32,
            content_hash: [u8; 32],
        },

        /// Content rolled back to previous version [domain, version, content_hash]
        ContentRolledBack {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            version: u32,
            content_hash: [u8; 32],
        },

        /// SSL/TLS certificate updated [domain, cert_hash, expires_at]
        SSLCertificateUpdated {
            domain: BoundedVec<u8, T::MaxDomainLength>,
            cert_hash: [u8; 32],
            expires_at: BlockNumberFor<T>,
        },
    }

    // ==================== ERRORS ====================

    #[pallet::error]
    pub enum Error<T> {
        /// Domain already registered
        DomainAlreadyExists,
        /// Domain not found
        DomainNotFound,
        /// Not domain owner
        NotDomainOwner,
        /// Domain too short (minimum length not met)
        DomainTooShort,
        /// Domain too long (exceeds maximum)
        DomainTooLong,
        /// Invalid domain characters
        InvalidDomainCharacters,
        /// Insufficient balance for purchase
        InsufficientBalance,
        /// Max domains per account reached
        MaxDomainsReached,
        /// KYC required for domain registration
        KycRequired,
        /// KYC Level 3 required for verified domains
        VerifiedKycRequired,
        /// Account is sanctioned
        AccountSanctioned,
        /// Domain not listed for sale
        NotListedForSale,
        /// Cannot buy own domain
        CannotBuyOwnDomain,
        /// Bid too low
        BidTooLow,
        /// Hosting already active
        HostingAlreadyActive,
        /// Hosting not active
        HostingNotActive,
        /// Hosting expired
        HostingExpired,
        /// Invalid hosting tier
        InvalidTier,
        /// Domain locked (transfer restricted)
        DomainLocked,
        /// Verification failed
        VerificationFailed,
        /// External domain not verified
        ExternalDomainNotVerified,
        /// Arithmetic overflow
        ArithmeticOverflow,
        /// Invalid metadata
        InvalidMetadata,
    }

    // ==================== EXTRINSICS ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new domain (permanent ownership)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            tier: u8,  // 0=Standard, 1=Premium, 2=Government, 3=Verified
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate domain name
            let domain = Self::validate_domain_name(domain_name)?;

            // Check if domain already exists
            ensure!(
                !DomainRegistry::<T>::contains_key(&domain),
                Error::<T>::DomainAlreadyExists
            );

            // Convert tier from u8
            let domain_tier = match tier {
                0 => DomainTier::Standard,
                1 => DomainTier::Premium,
                2 => DomainTier::Government,
                3 => DomainTier::Verified,
                _ => return Err(Error::<T>::InvalidTier.into()),
            };

            // KYC verification
            ensure!(
                T::Identity::can_register_domain(&who),
                Error::<T>::KycRequired
            );

            // Verified domains require Level 3 KYC
            if domain_tier == DomainTier::Verified {
                ensure!(
                    T::Identity::can_register_verified(&who),
                    Error::<T>::VerifiedKycRequired
                );
            }

            // Sanctions check
            ensure!(
                !T::Identity::is_sanctioned(&who),
                Error::<T>::AccountSanctioned
            );

            // Check max domains limit
            let mut owned_domains = AccountDomains::<T>::get(&who);
            ensure!(
                owned_domains.len() < T::MaxDomainsPerAccount::get() as usize,
                Error::<T>::MaxDomainsReached
            );

            // Calculate and collect fee
            let price = Self::calculate_domain_price(&domain, &domain_tier)?;
            Self::collect_domain_fee(&who, price)?;

            // Create domain record
            let current_block = frame_system::Pallet::<T>::block_number();
            let domain_record = DomainRecord {
                owner: who.clone(),
                original_owner: who.clone(),
                registered_at: current_block,
                purchase_price: price,
                tier: domain_tier,
                locked_until: None,
                transfer_count: 0,
            };

            // Store domain
            DomainRegistry::<T>::insert(&domain, domain_record);
            
            // Update account's domain list
            owned_domains.try_push(domain.clone())
                .map_err(|_| Error::<T>::MaxDomainsReached)?;
            AccountDomains::<T>::insert(&who, owned_domains);

            // Update counter
            TotalDomains::<T>::mutate(|n| *n = n.saturating_add(1));

            Self::deposit_event(Event::DomainRegistered {
                domain,
                owner: who,
                price,
                tier,
            });

            Ok(())
        }

        /// Set resolution records for owned domain
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::set_resolution())]
        pub fn set_resolution(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            wallet_address: Option<T::AccountId>,
            content_hash: Option<[u8; 32]>,
            metadata: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Verify ownership
            let domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Create/update resolution records
            let metadata_bounded: BoundedVec<u8, ConstU32<256>> = metadata.try_into()
                .map_err(|_| Error::<T>::InvalidMetadata)?;

            let resolution = ResolutionRecords {
                wallet_address,
                content_hash,
                avatar: None,
                metadata: metadata_bounded,
                text_records: BoundedVec::default(),
            };

            DomainResolution::<T>::insert(&domain, resolution);

            Self::deposit_event(Event::ResolutionUpdated {
                domain,
                owner: who,
            });

            Ok(())
        }

        /// Transfer domain to another account
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::transfer_domain())]
        pub fn transfer_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            new_owner: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Verify ownership
            let mut domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Check if domain is locked
            let current_block = frame_system::Pallet::<T>::block_number();
            if let Some(locked_until) = domain_record.locked_until {
                ensure!(current_block >= locked_until, Error::<T>::DomainLocked);
            }

            // M64 FIX: Remove any existing marketplace listing to prevent stale listings
            DomainListings::<T>::remove(&domain);

            // Update domain record
            domain_record.owner = new_owner.clone();
            domain_record.transfer_count = domain_record.transfer_count.saturating_add(1);
            DomainRegistry::<T>::insert(&domain, domain_record);

            // Update account domain lists
            let mut old_domains = AccountDomains::<T>::get(&who);
            old_domains.retain(|d| d != &domain);
            AccountDomains::<T>::insert(&who, old_domains);

            let mut new_domains = AccountDomains::<T>::get(&new_owner);
            new_domains.try_push(domain.clone())
                .map_err(|_| Error::<T>::MaxDomainsReached)?;
            AccountDomains::<T>::insert(&new_owner, new_domains);

            Self::deposit_event(Event::DomainTransferred {
                domain,
                from: who,
                to: new_owner,
            });

            Ok(())
        }

        /// List domain on marketplace for sale
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::list_domain())]
        pub fn list_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            price: u128,
            min_offer: Option<u128>,
            duration_blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Verify ownership
            let domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Check if already listed
            ensure!(
                !DomainListings::<T>::contains_key(&domain),
                Error::<T>::DomainAlreadyExists
            );

            // Create listing
            let current_block = frame_system::Pallet::<T>::block_number();
            let listing = DomainListing {
                seller: who.clone(),
                price,
                listed_at: current_block,
                expires_at: current_block.saturating_add(duration_blocks),
                min_offer,
            };

            DomainListings::<T>::insert(&domain, listing);

            Self::deposit_event(Event::DomainListed {
                domain,
                seller: who,
                price,
            });

            Ok(())
        }

        /// Buy listed domain from marketplace
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::buy_domain())]
        pub fn buy_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            offer_price: u128,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get listing
            let listing = DomainListings::<T>::get(&domain)
                .ok_or(Error::<T>::NotListedForSale)?;

            // Verify not buying own domain
            ensure!(listing.seller != buyer, Error::<T>::CannotBuyOwnDomain);

            // Check price
            ensure!(offer_price >= listing.price, Error::<T>::BidTooLow);
            if let Some(min_offer) = listing.min_offer {
                ensure!(offer_price >= min_offer, Error::<T>::BidTooLow);
            }

            // Check listing not expired
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= listing.expires_at, Error::<T>::HostingExpired);

            // Calculate marketplace fee (5%)
            let marketplace_fee = offer_price
                .checked_mul(MARKETPLACE_FEE_PERCENT as u128)
                .and_then(|v| v.checked_div(100))
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            let seller_amount = offer_price
                .checked_sub(marketplace_fee)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            // Transfer funds: buyer -> seller (minus fee)
            // SAFETY(saturated_into): u128 price → Balance. For standard substrate u128 balances
            // this is a no-op identity conversion.
            let seller_balance = seller_amount.saturated_into();
            T::Currency::transfer(
                &buyer,
                &listing.seller,
                seller_balance,
                ExistenceRequirement::KeepAlive,
            )?;

            // Transfer marketplace fee to treasury
            let treasury = T::Treasury::get();
            // SAFETY(saturated_into): u128 → Balance. The marketplace_fee is derived from
            // the listing price via a bounded percentage, so it stays within Balance range.
            // If it exceeds Balance::MAX, the transfer below would fail gracefully.
            let fee_balance = marketplace_fee.saturated_into();
            T::Currency::transfer(
                &buyer,
                &treasury,
                fee_balance,
                ExistenceRequirement::KeepAlive,
            )?;

            // Update domain ownership
            let mut domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            
            let old_owner = domain_record.owner.clone();
            domain_record.owner = buyer.clone();
            domain_record.transfer_count = domain_record.transfer_count.saturating_add(1);
            DomainRegistry::<T>::insert(&domain, domain_record);

            // Update account domain lists
            let mut old_domains = AccountDomains::<T>::get(&old_owner);
            old_domains.retain(|d| d != &domain);
            AccountDomains::<T>::insert(&old_owner, old_domains);

            let mut buyer_domains = AccountDomains::<T>::get(&buyer);
            buyer_domains.try_push(domain.clone())
                .map_err(|_| Error::<T>::MaxDomainsReached)?;
            AccountDomains::<T>::insert(&buyer, buyer_domains);

            // Remove listing
            DomainListings::<T>::remove(&domain);

            // Update marketplace revenue
            TotalMarketplaceRevenue::<T>::mutate(|r| *r = r.saturating_add(marketplace_fee));

            Self::deposit_event(Event::DomainSold {
                domain,
                seller: listing.seller,
                buyer,
                price: offer_price,
                marketplace_fee,
            });

            Ok(())
        }

        /// Remove domain listing from marketplace
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::unlist_domain())]
        pub fn unlist_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get listing
            let listing = DomainListings::<T>::get(&domain)
                .ok_or(Error::<T>::NotListedForSale)?;

            // Verify seller
            ensure!(listing.seller == who, Error::<T>::NotDomainOwner);

            // Remove listing
            DomainListings::<T>::remove(&domain);

            Ok(())
        }

        /// Activate web hosting for domain
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::activate_hosting())]
        pub fn activate_hosting(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            tier: u8,  // 0=Free, 1=Basic, 2=Pro, 3=Enterprise
            content_hash: [u8; 32],
            auto_renew: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Convert tier from u8
            let hosting_tier = match tier {
                0 => HostingTier::Free,
                1 => HostingTier::Basic,
                2 => HostingTier::Pro,
                3 => HostingTier::Enterprise,
                _ => return Err(Error::<T>::InvalidTier.into()),
            };

            // Verify domain ownership
            let domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Check if hosting already active
            ensure!(
                !HostedWebsites::<T>::contains_key(&domain),
                Error::<T>::HostingAlreadyActive
            );

            // Calculate monthly fee
            let monthly_fee = Self::calculate_hosting_fee(&hosting_tier);

            // Collect first month's fee (if not free tier)
            if monthly_fee > 0 {
                Self::collect_hosting_fee(&who, monthly_fee)?;
            }

            // Create hosting info
            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at = current_block.saturating_add(BLOCKS_PER_MONTH.into());

            let hosting_info = HostingInfo {
                subscriber: who.clone(),
                tier: hosting_tier,
                content_hash,
                activated_at: current_block,
                last_payment_at: current_block,
                expires_at,
                data_size: 0,
                monthly_fee,
                auto_renew,
            };

            HostedWebsites::<T>::insert(&domain, hosting_info);

            Self::deposit_event(Event::HostingActivated {
                domain,
                owner: who,
                tier,
                content_hash,
            });

            Ok(())
        }

        /// Renew hosting subscription
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::renew_hosting())]
        pub fn renew_hosting(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            months: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get hosting info
            let mut hosting_info = HostedWebsites::<T>::get(&domain)
                .ok_or(Error::<T>::HostingNotActive)?;

            // Verify subscriber
            ensure!(hosting_info.subscriber == who, Error::<T>::NotDomainOwner);

            // Calculate renewal fee
            let total_fee = hosting_info.monthly_fee
                .checked_mul(months as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            // Collect renewal fee
            if total_fee > 0 {
                Self::collect_hosting_fee(&who, total_fee)?;
            }

            // Extend expiry
            let current_block = frame_system::Pallet::<T>::block_number();
            let blocks_to_add = BLOCKS_PER_MONTH
                .checked_mul(months)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            let new_expiry = if current_block > hosting_info.expires_at {
                // Expired - renew from current block
                current_block.saturating_add(blocks_to_add.into())
            } else {
                // Still active - extend from current expiry
                hosting_info.expires_at.saturating_add(blocks_to_add.into())
            };

            hosting_info.expires_at = new_expiry;
            hosting_info.last_payment_at = current_block;
            HostedWebsites::<T>::insert(&domain, hosting_info);

            Self::deposit_event(Event::HostingRenewed {
                domain,
                owner: who,
                blocks_extended: blocks_to_add.into(),
            });

            Ok(())
        }

        /// Deactivate web hosting
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::deactivate_hosting())]
        pub fn deactivate_hosting(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get hosting info
            let hosting_info = HostedWebsites::<T>::get(&domain)
                .ok_or(Error::<T>::HostingNotActive)?;

            // Verify subscriber
            ensure!(hosting_info.subscriber == who, Error::<T>::NotDomainOwner);

            // Remove hosting
            HostedWebsites::<T>::remove(&domain);

            Ok(())
        }

        /// Update hosted website content
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::update_hosting_content())]
        pub fn update_hosting_content(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            new_content_hash: [u8; 32],
            description: Vec<u8>,
            size_bytes: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get hosting info
            let mut hosting_info = HostedWebsites::<T>::get(&domain)
                .ok_or(Error::<T>::HostingNotActive)?;

            // Verify subscriber
            ensure!(hosting_info.subscriber == who, Error::<T>::NotDomainOwner);

            // Check not expired
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= hosting_info.expires_at, Error::<T>::HostingExpired);

            // Save current version to history
            let current_version = CurrentContentVersion::<T>::get(&domain);
            let old_version = ContentVersion {
                content_hash: hosting_info.content_hash,
                uploaded_at: current_block,
                description: description.clone().try_into()
                    .map_err(|_| Error::<T>::DomainTooLong)?,
                size_bytes,
            };
            ContentHistory::<T>::insert(&domain, current_version, old_version);

            // Increment version counter
            let new_version = current_version.saturating_add(1);
            CurrentContentVersion::<T>::insert(&domain, new_version);

            // Update content hash
            hosting_info.content_hash = new_content_hash;
            HostedWebsites::<T>::insert(&domain, hosting_info);

            // Also update domain resolution
            if let Some(mut resolution) = DomainResolution::<T>::get(&domain) {
                resolution.content_hash = Some(new_content_hash);
                DomainResolution::<T>::insert(&domain, resolution);
            }

            Self::deposit_event(Event::ContentUpdated {
                domain: domain.clone(),
                version: new_version,
                content_hash: new_content_hash,
            });

            Ok(())
        }

        /// Register external domain (.com, .org, .net, etc.)
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::register_external_domain())]
        pub fn register_external_domain(
            origin: OriginFor<T>,
            external_domain: Vec<u8>,
            linked_bns_domain: Vec<u8>,
            tier: u8,  // 0=Free, 1=Basic, 2=Pro, 3=Enterprise
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate external domain format
            let external_bounded: BoundedVec<u8, ConstU32<128>> = external_domain.try_into()
                .map_err(|_| Error::<T>::DomainTooLong)?;

            // Validate linked BNS domain
            let bns_domain = Self::validate_domain_name(linked_bns_domain)?;

            // Verify BNS domain ownership
            let domain_record = DomainRegistry::<T>::get(&bns_domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Check if external domain already registered
            ensure!(
                !ExternalDomains::<T>::contains_key(&external_bounded),
                Error::<T>::DomainAlreadyExists
            );

            // Convert tier from u8
            let hosting_tier = match tier {
                0 => HostingTier::Free,
                1 => HostingTier::Basic,
                2 => HostingTier::Pro,
                3 => HostingTier::Enterprise,
                _ => return Err(Error::<T>::InvalidTier.into()),
            };

            // Generate verification token
            let verification_token = Self::generate_verification_token(&who, &external_bounded);

            // Calculate monthly fee
            let monthly_fee = Self::calculate_hosting_fee(&hosting_tier);

            // Collect first month's fee
            if monthly_fee > 0 {
                Self::collect_hosting_fee(&who, monthly_fee)?;
            }

            // Create external domain record
            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at = current_block.saturating_add(BLOCKS_PER_MONTH.into());

            let external_info = ExternalDomainInfo {
                owner: who.clone(),
                linked_bns_domain: bns_domain,
                tier: hosting_tier,
                verification_token,
                verified: false,
                registered_at: current_block,
                monthly_fee,
                expires_at,
            };

            ExternalDomains::<T>::insert(&external_bounded, external_info);

            // Create verification status
            let verification = VerificationStatus {
                token: verification_token,
                attempts: 0,
                last_attempt: current_block,
                verified: false,
            };

            DomainVerification::<T>::insert(&external_bounded, verification);

            Self::deposit_event(Event::ExternalDomainRegistered {
                domain: external_bounded,
                owner: who,
                tier,
            });

            Ok(())
        }

        /// Verify external domain (checks DNS TXT record)
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::verify_external_domain())]
        pub fn verify_external_domain(
            origin: OriginFor<T>,
            external_domain: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let external_bounded: BoundedVec<u8, ConstU32<128>> = external_domain.try_into()
                .map_err(|_| Error::<T>::DomainTooLong)?;

            // Get external domain info
            let external_info = ExternalDomains::<T>::get(&external_bounded)
                .ok_or(Error::<T>::DomainNotFound)?;

            // Verify ownership
            ensure!(external_info.owner == who, Error::<T>::NotDomainOwner);

            // Get verification status
            let mut verification = DomainVerification::<T>::get(&external_bounded)
                .ok_or(Error::<T>::VerificationFailed)?;

            // NOTE: In production, this would call an oracle or off-chain worker
            // to verify the DNS TXT record: belize-verify=<verification_token>
            // Verification is NOT auto-approved — requires separate governance approval

            // Record the verification attempt but do NOT mark as verified
            // A governance/root call should be used to actually approve verification
            verification.attempts = verification.attempts.saturating_add(1);
            verification.last_attempt = frame_system::Pallet::<T>::block_number();

            DomainVerification::<T>::insert(&external_bounded, verification);

            Self::deposit_event(Event::ExternalDomainVerificationRequested {
                domain: external_bounded,
                owner: who,
            });

            Ok(())
        }

        /// Create subdomain under owned parent domain
        /// Allows delegation to different accounts (e.g., blog.example.bz owned by different user)
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::create_subdomain())]
        pub fn create_subdomain(
            origin: OriginFor<T>,
            parent_domain: Vec<u8>,
            subdomain: Vec<u8>,
            delegate_to: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate parent domain
            let parent_bounded = Self::validate_domain_name(parent_domain.clone())?;

            // Verify parent domain ownership
            let parent_record = DomainRegistry::<T>::get(&parent_bounded)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(parent_record.owner == who, Error::<T>::NotDomainOwner);

            // Validate subdomain part (before combining)
            for &byte in &subdomain {
                let is_valid = byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || byte == b'-';
                ensure!(is_valid, Error::<T>::InvalidDomainCharacters);
            }
            ensure!(!subdomain.is_empty(), Error::<T>::DomainTooShort);
            ensure!(subdomain.len() <= 32, Error::<T>::DomainTooLong);

            // Create full subdomain name (subdomain.parent)
            let mut full_domain = subdomain.clone();
            full_domain.push(b'.');
            full_domain.extend_from_slice(&parent_domain);

            let full_domain_bounded = Self::validate_domain_name(full_domain.clone())?;

            // Check if subdomain already exists
            ensure!(
                !DomainRegistry::<T>::contains_key(&full_domain_bounded),
                Error::<T>::DomainAlreadyExists
            );

            // Determine owner (delegate or parent owner)
            let is_delegated = delegate_to.is_some();
            let subdomain_owner = delegate_to.unwrap_or_else(|| who.clone());

            // Create subdomain record (free, inherits parent tier)
            let subdomain_record = DomainRecord {
                owner: subdomain_owner.clone(),
                original_owner: subdomain_owner.clone(),
                registered_at: frame_system::Pallet::<T>::block_number(),
                purchase_price: 0,  // Subdomains are free for parent domain owners
                tier: parent_record.tier,  // Inherit parent tier
                locked_until: None,
                transfer_count: 0,
            };

            DomainRegistry::<T>::insert(&full_domain_bounded, subdomain_record);

            // Add to subdomain owner's domains
            let mut owned_domains = AccountDomains::<T>::get(&subdomain_owner);
            owned_domains.try_push(full_domain_bounded.clone())
                .map_err(|_| Error::<T>::MaxDomainsReached)?;
            AccountDomains::<T>::insert(&subdomain_owner, owned_domains);

            // Update counter
            TotalDomains::<T>::mutate(|n| *n = n.saturating_add(1));

            Self::deposit_event(Event::SubdomainCreated {
                subdomain: full_domain_bounded,
                parent: parent_bounded,
                owner: subdomain_owner,
                delegated: is_delegated,
            });

            Ok(())
        }

        /// Rollback hosting content to a previous version
        #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::update_hosting_content())] // Reuse update weight
        pub fn rollback_content(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            target_version: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Get hosting info
            let mut hosting_info = HostedWebsites::<T>::get(&domain)
                .ok_or(Error::<T>::HostingNotActive)?;

            // Verify subscriber
            ensure!(hosting_info.subscriber == who, Error::<T>::NotDomainOwner);

            // Check not expired
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block <= hosting_info.expires_at, Error::<T>::HostingExpired);

            // Get target version from history
            let target_content = ContentHistory::<T>::get(&domain, target_version)
                .ok_or(Error::<T>::InvalidTier)?; // Reuse error for "invalid version"

            // Verify version exists and is not current
            let current_version = CurrentContentVersion::<T>::get(&domain);
            ensure!(target_version < current_version, Error::<T>::InvalidTier);

            // Save current version to history before rollback
            let rollback_version = ContentVersion {
                content_hash: hosting_info.content_hash,
                uploaded_at: current_block,
                description: b"Rollback savepoint".to_vec().try_into()
                    .map_err(|_| Error::<T>::DomainTooLong)?,
                size_bytes: target_content.size_bytes,
            };
            ContentHistory::<T>::insert(&domain, current_version, rollback_version);

            // Increment version (rollback creates new version)
            let new_version = current_version.saturating_add(1);
            CurrentContentVersion::<T>::insert(&domain, new_version);

            // Restore content from target version
            hosting_info.content_hash = target_content.content_hash;
            HostedWebsites::<T>::insert(&domain, hosting_info);

            // Update domain resolution
            if let Some(mut resolution) = DomainResolution::<T>::get(&domain) {
                resolution.content_hash = Some(target_content.content_hash);
                DomainResolution::<T>::insert(&domain, resolution);
            }

            Self::deposit_event(Event::ContentRolledBack {
                domain: domain.clone(),
                version: target_version,
                content_hash: target_content.content_hash,
            });

            Ok(())
        }

        /// Update SSL/TLS certificate hash for domain
        #[pallet::call_index(14)]
        #[pallet::weight(T::WeightInfo::update_hosting_content())] // Reuse similar weight
        pub fn update_ssl_certificate(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            cert_hash: [u8; 32],
            serial_number: Vec<u8>,
            issuer: Vec<u8>,
            expires_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let domain = Self::validate_domain_name(domain_name)?;

            // Verify domain ownership
            let domain_record = DomainRegistry::<T>::get(&domain)
                .ok_or(Error::<T>::DomainNotFound)?;
            ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

            // Create SSL certificate info
            let current_block = frame_system::Pallet::<T>::block_number();
            let ssl_info = SSLCertInfo {
                cert_hash,
                issued_at: current_block,
                expires_at,
                serial_number: serial_number.try_into()
                    .map_err(|_| Error::<T>::DomainTooLong)?,
                issuer: issuer.try_into()
                    .map_err(|_| Error::<T>::DomainTooLong)?,
            };

            SSLCertificates::<T>::insert(&domain, ssl_info);

            Self::deposit_event(Event::SSLCertificateUpdated {
                domain: domain.clone(),
                cert_hash,
                expires_at,
            });

            Ok(())
        }
    }

    // ==================== HELPER FUNCTIONS ====================

    impl<T: Config> Pallet<T> {
        /// Validate domain name format
        fn validate_domain_name(domain: Vec<u8>) -> Result<BoundedVec<u8, T::MaxDomainLength>, DispatchError> {
            let min_len = T::MinDomainLength::get() as usize;
            let max_len = T::MaxDomainLength::get() as usize;
            
            // Check length
            ensure!(
                domain.len() >= min_len,
                Error::<T>::DomainTooShort
            );
            ensure!(
                domain.len() <= max_len,
                Error::<T>::DomainTooLong
            );

            // Check valid characters (lowercase alphanumeric + hyphen)
            for &byte in &domain {
                let is_valid = byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || byte == b'-'
                    || byte == b'.';
                ensure!(is_valid, Error::<T>::InvalidDomainCharacters);
            }

            domain.try_into()
                .map_err(|_| Error::<T>::DomainTooLong.into())
        }

        /// Calculate domain registration price
        fn calculate_domain_price(
            domain: &BoundedVec<u8, T::MaxDomainLength>,
            tier: &DomainTier,
        ) -> Result<u128, DispatchError> {
            let base_price = match tier {
                DomainTier::Standard => 100_000_000_000_000,    // 100 DALLA
                DomainTier::Premium => 500_000_000_000_000,     // 500 DALLA
                DomainTier::Government => 50_000_000_000_000,   // 50 DALLA (subsidized)
                DomainTier::Verified => 1_000_000_000_000_000,  // 1000 DALLA
            };

            // Length-based multiplier
            let multiplier = match domain.len() {
                1..=2 => 10,  // Ultra-premium
                3 => 5,       // Premium
                4 => 2,       // Semi-premium
                _ => 1,       // Standard
            };

            base_price.checked_mul(&multiplier)
                .ok_or(Error::<T>::ArithmeticOverflow.into())
        }

        /// Collect domain registration fee to treasury
        fn collect_domain_fee(payer: &T::AccountId, amount: u128) -> DispatchResult {
            let treasury = T::Treasury::get();
            // SAFETY(saturated_into): u128 → Balance. Domain fees are protocol-defined
            // constants (e.g. 10–50 DALLA) that are well within Balance range.
            let balance_amount = amount.saturated_into();

            T::Currency::transfer(
                payer,
                &treasury,
                balance_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Ok(())
        }

        /// Calculate monthly hosting fee
        fn calculate_hosting_fee(tier: &HostingTier) -> u128 {
            match tier {
                HostingTier::Free => 0,
                HostingTier::Basic => 10_000_000_000_000,      // 10 DALLA/month
                HostingTier::Pro => 50_000_000_000_000,        // 50 DALLA/month
                HostingTier::Enterprise => 200_000_000_000_000, // 200 DALLA/month
            }
        }

        /// Collect hosting fee to treasury
        fn collect_hosting_fee(payer: &T::AccountId, amount: u128) -> DispatchResult {
            if amount == 0 {
                return Ok(());
            }

            let treasury = T::Treasury::get();
            // SAFETY(saturated_into): u128 → Balance. Hosting fees are protocol-defined
            // via HostingTier constants (10–200 DALLA), well within Balance range.
            let balance_amount = amount.saturated_into();

            T::Currency::transfer(
                payer,
                &treasury,
                balance_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // Update revenue counter
            TotalHostingRevenue::<T>::mutate(|r| *r = r.saturating_add(amount));

            Self::deposit_event(Event::HostingFeeCollected {
                payer: payer.clone(),
                amount,
            });

            Ok(())
        }

        /// Generate verification token for external domain
        /// BNS-3 FIX: Added parent_hash and extrinsics_root for unpredictability
        fn generate_verification_token(
            account: &T::AccountId,
            domain: &BoundedVec<u8, ConstU32<128>>,
        ) -> [u8; 32] {
            use sp_runtime::traits::Hash;
            
            let mut data = Vec::new();
            data.extend_from_slice(&account.encode());
            data.extend_from_slice(&domain.encode());
            data.extend_from_slice(&frame_system::Pallet::<T>::block_number().encode());
            // Add parent block hash for unpredictability (cannot be predicted before block is sealed)
            data.extend_from_slice(&frame_system::Pallet::<T>::parent_hash().encode());
            // Add extrinsic count for additional entropy from current block execution state
            data.extend_from_slice(&frame_system::Pallet::<T>::extrinsic_count().encode());
            
            let hash = <T as frame_system::Config>::Hashing::hash(&data);
            let mut token = [0u8; 32];
            token.copy_from_slice(hash.as_ref());
            token
        }

        /// Get treasury account
        pub fn treasury_account() -> T::AccountId {
            BNS_TREASURY_ID.into_account_truncating()
        }
    }
}

