//! Benchmarks for pallet-belize-bns (v2 API)
//!
//! Covers 13 WeightInfo functions:
//!   register_domain, set_resolution, transfer_domain,
//!   list_domain, buy_domain, unlist_domain,
//!   activate_hosting, renew_hosting, deactivate_hosting,
//!   register_external_domain, verify_external_domain,
//!   update_hosting_content, create_subdomain

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_std::vec::Vec;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Create a funded account for benchmarks
fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, 0);
    let amount: BalanceOf<T> = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
    T::Currency::make_free_balance_be(&caller, amount);
    caller
}

/// Build a valid .bz domain name as Vec<u8>, e.g. "bench0.bz"
fn make_domain_name(seed: u8) -> Vec<u8> {
    let mut name = b"benchdomain".to_vec();
    name.push(b'0' + (seed % 10));
    name.extend_from_slice(b".bz");
    name
}

/// Build an external domain name, e.g. "extern0.com"
fn make_external_domain(seed: u8) -> Vec<u8> {
    let mut name = b"externsite".to_vec();
    name.push(b'0' + (seed % 10));
    name.extend_from_slice(b".com");
    name
}

/// Directly insert a DomainRecord into storage, bypassing KYC and fee collection.
fn insert_domain<T: Config>(owner: &T::AccountId, domain_name: &[u8]) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let domain: BoundedVec<u8, T::MaxDomainLength> =
        BoundedVec::try_from(domain_name.to_vec()).expect("domain within bounds");

    let record = DomainRecord {
        owner: owner.clone(),
        original_owner: owner.clone(),
        registered_at: current_block,
        purchase_price: 100u128,
        tier: DomainTier::Standard,
        locked_until: None,
        transfer_count: 0,
    };

    DomainRegistry::<T>::insert(&domain, record);

    // Add to account's domain list
    let mut owned = AccountDomains::<T>::get(owner);
    let _ = owned.try_push(domain);
    AccountDomains::<T>::insert(owner, owned);

    TotalDomains::<T>::mutate(|n| *n = n.saturating_add(1));
}

/// Insert a DomainListing for a domain already in the registry.
fn insert_listing<T: Config>(seller: &T::AccountId, domain_name: &[u8], price: u128) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let domain: BoundedVec<u8, T::MaxDomainLength> =
        BoundedVec::try_from(domain_name.to_vec()).expect("domain within bounds");

    let expiry_val: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0) + 100_000u64;
    let expiry: BlockNumberFor<T> = expiry_val.saturated_into();

    let listing = DomainListing {
        seller: seller.clone(),
        price,
        listed_at: current_block,
        expires_at: expiry,
        min_offer: None,
    };

    DomainListings::<T>::insert(&domain, listing);
}

/// Insert a HostingInfo record for an owned domain
fn insert_hosting<T: Config>(subscriber: &T::AccountId, domain_name: &[u8]) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let domain: BoundedVec<u8, T::MaxDomainLength> =
        BoundedVec::try_from(domain_name.to_vec()).expect("domain within bounds");

    let expiry_val: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0) + 1_000_000u64;
    let expiry: BlockNumberFor<T> = expiry_val.saturated_into();

    let hosting = HostingInfo {
        subscriber: subscriber.clone(),
        tier: HostingTier::Basic,
        content_hash: [1u8; 32],
        activated_at: current_block,
        last_payment_at: current_block,
        expires_at: expiry,
        data_size: 1024,
        monthly_fee: 10_000_000_000_000u128, // 10 DALLA - matches calculate_hosting_fee(Basic)
        auto_renew: false,
    };

    HostedWebsites::<T>::insert(&domain, hosting);
}

/// Insert an ExternalDomainInfo record
fn insert_external_domain<T: Config>(
    owner: &T::AccountId,
    external_name: &[u8],
    bns_domain_name: &[u8],
) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let ext_domain: BoundedVec<u8, ConstU32<128>> =
        BoundedVec::try_from(external_name.to_vec()).expect("ext domain within bounds");
    let linked: BoundedVec<u8, T::MaxDomainLength> =
        BoundedVec::try_from(bns_domain_name.to_vec()).expect("bns domain within bounds");

    let expiry_val: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0) + 500_000u64;
    let expiry: BlockNumberFor<T> = expiry_val.saturated_into();

    // Generate a deterministic verification token
    let token = {
        let mut buf = [0u8; 32];
        let data = (owner, external_name).encode();
        let len = core::cmp::min(data.len(), 32);
        buf[..len].copy_from_slice(&data[..len]);
        buf
    };

    let info = ExternalDomainInfo {
        owner: owner.clone(),
        linked_bns_domain: linked,
        tier: HostingTier::Basic,
        verification_token: token,
        verified: false,
        registered_at: current_block,
        monthly_fee: 10u128,
        expires_at: expiry,
    };

    ExternalDomains::<T>::insert(&ext_domain, info);

    // Also insert verification status (verify_external_domain reads this)
    let verification = VerificationStatus {
        token,
        attempts: 0u8,
        last_attempt: current_block,
        verified: false,
    };
    DomainVerification::<T>::insert(&ext_domain, verification);
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// register_domain is KYC-gated via `T::Identity::can_register_domain()`.
    /// The runtime's BnsIdentityProvider checks Identity + Oracle pallets.
    /// For benchmarks we bypass by directly inserting the domain (see other benchmarks)
    /// and only benchmark register_domain when the mock allows permissionless registration.
    ///
    /// If the runtime implements a `runtime-benchmarks` feature bypass in
    /// BnsIdentityProvider (always returning true), this benchmark will work.
    /// Otherwise, it may fail due to KYC requirements, in which case you should
    /// add the bypass or use the weight from the hand-estimated value.
    #[benchmark]
    fn register_domain() {
        let caller = funded_account::<T>("registrant", 0);
        let domain_name = make_domain_name(0);

        #[extrinsic_call]
        register_domain(
            RawOrigin::Signed(caller),
            domain_name,
            0u8, // tier: Standard
        );
    }

    #[benchmark]
    fn set_resolution() {
        let caller = funded_account::<T>("owner", 0);
        let domain_name = make_domain_name(1);
        insert_domain::<T>(&caller, &domain_name);

        let metadata = b"benchmark metadata".to_vec();

        #[extrinsic_call]
        set_resolution(
            RawOrigin::Signed(caller),
            domain_name,
            None,            // wallet_address
            Some([2u8; 32]), // content_hash
            metadata,
        );
    }

    #[benchmark]
    fn transfer_domain() {
        let owner = funded_account::<T>("owner", 0);
        let new_owner: T::AccountId = account("newowner", 1, 0);
        let domain_name = make_domain_name(2);
        insert_domain::<T>(&owner, &domain_name);

        #[extrinsic_call]
        transfer_domain(RawOrigin::Signed(owner), domain_name, new_owner);
    }

    #[benchmark]
    fn list_domain() {
        let seller = funded_account::<T>("seller", 0);
        let domain_name = make_domain_name(3);
        insert_domain::<T>(&seller, &domain_name);

        let price: u128 = 500_000_000_000u128;
        let duration: BlockNumberFor<T> = 100_000u32.into();

        #[extrinsic_call]
        list_domain(
            RawOrigin::Signed(seller),
            domain_name,
            price,
            None, // min_offer
            duration,
        );
    }

    #[benchmark]
    fn buy_domain() {
        let seller = funded_account::<T>("seller", 0);
        let buyer = funded_account::<T>("buyer", 1);
        let domain_name = make_domain_name(4);
        insert_domain::<T>(&seller, &domain_name);

        let price: u128 = 100_000_000_000u128;
        insert_listing::<T>(&seller, &domain_name, price);

        #[extrinsic_call]
        buy_domain(
            RawOrigin::Signed(buyer),
            domain_name,
            price, // offer_price
        );
    }

    #[benchmark]
    fn unlist_domain() {
        let seller = funded_account::<T>("seller", 0);
        let domain_name = make_domain_name(5);
        insert_domain::<T>(&seller, &domain_name);

        let price: u128 = 200_000_000_000u128;
        insert_listing::<T>(&seller, &domain_name, price);

        #[extrinsic_call]
        unlist_domain(RawOrigin::Signed(seller), domain_name);
    }

    #[benchmark]
    fn activate_hosting() {
        let owner = funded_account::<T>("owner", 0);
        let domain_name = make_domain_name(6);
        insert_domain::<T>(&owner, &domain_name);

        #[extrinsic_call]
        activate_hosting(
            RawOrigin::Signed(owner),
            domain_name,
            1u8,       // tier: Basic
            [3u8; 32], // content_hash
            false,     // auto_renew
        );
    }

    #[benchmark]
    fn renew_hosting() {
        let subscriber = funded_account::<T>("subscriber", 0);
        let domain_name = make_domain_name(7);
        insert_domain::<T>(&subscriber, &domain_name);
        insert_hosting::<T>(&subscriber, &domain_name);

        #[extrinsic_call]
        renew_hosting(
            RawOrigin::Signed(subscriber),
            domain_name,
            3u32, // months
        );
    }

    #[benchmark]
    fn deactivate_hosting() {
        let subscriber = funded_account::<T>("subscriber", 0);
        let domain_name = make_domain_name(8);
        insert_domain::<T>(&subscriber, &domain_name);
        insert_hosting::<T>(&subscriber, &domain_name);

        #[extrinsic_call]
        deactivate_hosting(RawOrigin::Signed(subscriber), domain_name);
    }

    #[benchmark]
    fn register_external_domain() {
        let owner = funded_account::<T>("owner", 0);
        let bns_domain = make_domain_name(9);
        insert_domain::<T>(&owner, &bns_domain);

        let external_domain = make_external_domain(0);

        #[extrinsic_call]
        register_external_domain(
            RawOrigin::Signed(owner),
            external_domain,
            bns_domain,
            1u8, // tier: Basic
        );
    }

    #[benchmark]
    fn verify_external_domain() {
        let owner = funded_account::<T>("owner", 0);
        let bns_domain = make_domain_name(9);
        insert_domain::<T>(&owner, &bns_domain);

        let external_domain = make_external_domain(1);
        insert_external_domain::<T>(&owner, &external_domain, &bns_domain);

        #[extrinsic_call]
        verify_external_domain(RawOrigin::Signed(owner), external_domain);
    }

    #[benchmark]
    fn update_hosting_content() {
        let subscriber = funded_account::<T>("subscriber", 0);
        let domain_name = make_domain_name(9);
        insert_domain::<T>(&subscriber, &domain_name);
        insert_hosting::<T>(&subscriber, &domain_name);

        let description = b"benchmark update".to_vec();

        #[extrinsic_call]
        update_hosting_content(
            RawOrigin::Signed(subscriber),
            domain_name,
            [4u8; 32], // new_content_hash
            description,
            2048u64, // size_bytes
        );
    }

    #[benchmark]
    fn create_subdomain() {
        let owner = funded_account::<T>("owner", 0);
        let parent_domain = make_domain_name(9);
        insert_domain::<T>(&owner, &parent_domain);

        let subdomain = b"app".to_vec();

        #[extrinsic_call]
        create_subdomain(
            RawOrigin::Signed(owner),
            parent_domain,
            subdomain,
            None, // delegate_to
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
