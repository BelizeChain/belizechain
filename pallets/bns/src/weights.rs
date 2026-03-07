//! BNS Weight Calculations

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

/// Weight information for BNS extrinsics
pub trait WeightInfo {
    fn register_domain() -> Weight;
    fn set_resolution() -> Weight;
    fn transfer_domain() -> Weight;
    fn list_domain() -> Weight;
    fn buy_domain() -> Weight;
    fn unlist_domain() -> Weight;
    fn activate_hosting() -> Weight;
    fn renew_hosting() -> Weight;
    fn deactivate_hosting() -> Weight;
    fn register_external_domain() -> Weight;
    fn verify_external_domain() -> Weight;
    fn update_hosting_content() -> Weight;
    fn create_subdomain() -> Weight;
}

/// Substrate default weights implementation
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Weight for registering a new domain
    /// Reads: DomainRegistry, AccountDomains, Identity verification (3 reads)
    /// Writes: DomainRegistry, AccountDomains, TotalDomains (3 writes)
    fn register_domain() -> Weight {
        Weight::from_parts(50_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Weight for setting domain resolution records
    /// Reads: DomainRegistry (1 read)
    /// Writes: DomainResolution (1 write)
    fn set_resolution() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Weight for transferring domain ownership
    /// Reads: DomainRegistry, AccountDomains (2 reads)
    /// Writes: DomainRegistry, AccountDomains (2 writes)
    fn transfer_domain() -> Weight {
        Weight::from_parts(40_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for listing domain on marketplace
    /// Reads: DomainRegistry (1 read)
    /// Writes: DomainListings (1 write)
    fn list_domain() -> Weight {
        Weight::from_parts(25_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Weight for buying listed domain
    /// Reads: DomainRegistry, DomainListings, AccountDomains (3 reads)
    /// Writes: DomainRegistry, DomainListings, AccountDomains, TotalMarketplaceRevenue (4 writes)
    fn buy_domain() -> Weight {
        Weight::from_parts(60_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Weight for unlisting domain from marketplace
    /// Reads: DomainListings (1 read)
    /// Writes: DomainListings (1 write)
    fn unlist_domain() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Weight for activating web hosting
    /// Reads: DomainRegistry, HostedWebsites (2 reads)
    /// Writes: HostedWebsites (1 write)
    fn activate_hosting() -> Weight {
        Weight::from_parts(45_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Weight for renewing hosting subscription
    /// Reads: HostedWebsites (1 read)
    /// Writes: HostedWebsites, TotalHostingRevenue (2 writes)
    fn renew_hosting() -> Weight {
        Weight::from_parts(35_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for deactivating hosting
    /// Reads: HostedWebsites (1 read)
    /// Writes: HostedWebsites (1 write)
    fn deactivate_hosting() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Weight for registering external domain
    /// Reads: DomainRegistry (1 read)
    /// Writes: ExternalDomains, DomainVerification (2 writes)
    fn register_external_domain() -> Weight {
        Weight::from_parts(40_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for verifying external domain (DNS check)
    /// Reads: ExternalDomains, DomainVerification (2 reads)
    /// Writes: ExternalDomains, DomainVerification (2 writes)
    fn verify_external_domain() -> Weight {
        Weight::from_parts(50_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for updating hosted website content
    /// Weight for updating hosting content with versioning
    /// Reads: HostedWebsites, ContentHistory, CurrentContentVersion (3 reads)
    /// Writes: HostedWebsites, DomainResolution, ContentHistory, CurrentContentVersion (4 writes)
    fn update_hosting_content() -> Weight {
        Weight::from_parts(45_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Weight for creating subdomain
    /// Reads: DomainRegistry (parent), DomainRegistry (subdomain check) (2 reads)
    /// Writes: DomainRegistry, DomainResolution, AccountDomains, TotalDomains (4 writes)
    fn create_subdomain() -> Weight {
        Weight::from_parts(50_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}
