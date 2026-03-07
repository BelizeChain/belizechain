//! BNS Pallet Tests

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_eq!(TotalDomains::<Test>::get(), 1);
    });
}

#[test]
fn register_domain_fails_duplicate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(Bns::register_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 0), Error::<Test>::DomainAlreadyExists);
    });
}

#[test]
fn sanctioned_account_cannot_register() {
    new_test_ext().execute_with(|| {
        assert_noop!(Bns::register_domain(RuntimeOrigin::signed(666), b"blocked".to_vec(), 0), Error::<Test>::AccountSanctioned);
    });
}

#[test]
fn verified_domain_requires_kyc() {
    new_test_ext().execute_with(|| {
        assert_noop!(Bns::register_domain(RuntimeOrigin::signed(1), b"verified".to_vec(), 3), Error::<Test>::VerifiedKycRequired);
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(100), b"verified".to_vec(), 3));
    });
}

#[test]
fn transfer_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::transfer_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 2));
    });
}

#[test]
fn set_resolution_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::set_resolution(RuntimeOrigin::signed(1), b"mysite".to_vec(), Some(1), Some([0u8; 32]), vec![]));
    });
}

#[test]
fn list_and_buy_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 100_000_000_000_000, None, 1000));
        assert_ok!(Bns::buy_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 100_000_000_000_000));
    });
}

#[test]
fn activate_hosting_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 1, [0u8; 32], false));
    });
}

#[test]
fn renew_hosting_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 1, [0u8; 32], false));
        assert_ok!(Bns::renew_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 3));
    });
}

#[test]
fn register_external_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mybns".to_vec(), 0));
        assert_ok!(Bns::register_external_domain(RuntimeOrigin::signed(1), b"example.com".to_vec(), b"mybns".to_vec(), 2));
    });
}

#[test]
fn unlist_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 1_000_000_000_000_000u128, None, 100_000u64));
        // Verify listing exists
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"mysite".to_vec().try_into().unwrap();
        assert!(DomainListings::<Test>::contains_key(&domain));
        assert_ok!(Bns::unlist_domain(RuntimeOrigin::signed(1), b"mysite".to_vec()));
        assert!(!DomainListings::<Test>::contains_key(&domain));
    });
}

#[test]
fn deactivate_hosting_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"myhost".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"myhost".to_vec(), 0, [0u8; 32], false));
        assert_ok!(Bns::deactivate_hosting(RuntimeOrigin::signed(1), b"myhost".to_vec()));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"myhost".to_vec().try_into().unwrap();
        assert!(HostedWebsites::<Test>::get(&domain).is_none());
    });
}

#[test]
fn update_hosting_content_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mypage".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mypage".to_vec(), 0, [0u8; 32], false));
        let new_hash = [0xABu8; 32];
        assert_ok!(Bns::update_hosting_content(
            RuntimeOrigin::signed(1),
            b"mypage".to_vec(),
            new_hash,
            b"Updated content".to_vec(),
            1024u64,
        ));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"mypage".to_vec().try_into().unwrap();
        let hosting = HostedWebsites::<Test>::get(&domain).unwrap();
        assert_eq!(hosting.content_hash, new_hash);
    });
}

#[test]
fn verify_external_domain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mybns2".to_vec(), 0));
        assert_ok!(Bns::register_external_domain(
            RuntimeOrigin::signed(1),
            b"example2.com".to_vec(),
            b"mybns2".to_vec(),
            0, // Free tier
        ));
        // Verification attempt is recorded but not auto-approved
        assert_ok!(Bns::verify_external_domain(
            RuntimeOrigin::signed(1),
            b"example2.com".to_vec(),
        ));
        let external: BoundedVec<u8, ConstU32<128>> = b"example2.com".to_vec().try_into().unwrap();
        let verification = DomainVerification::<Test>::get(&external).unwrap();
        assert_eq!(verification.attempts, 1);
        assert!(!verification.verified); // Not auto-verified
    });
}

#[test]
fn create_subdomain_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"parent".to_vec(), 0));
        assert_ok!(Bns::create_subdomain(
            RuntimeOrigin::signed(1),
            b"parent".to_vec(),
            b"blog".to_vec(),
            None, // same owner
        ));
        // Full domain = "blog.parent"
        let full: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"blog.parent".to_vec().try_into().unwrap();
        assert!(DomainRegistry::<Test>::contains_key(&full));
    });
}

#[test]
fn create_subdomain_delegated_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"myorg".to_vec(), 0));
        assert_ok!(Bns::create_subdomain(
            RuntimeOrigin::signed(1),
            b"myorg".to_vec(),
            b"shop".to_vec(),
            Some(2u64), // delegate to account 2
        ));
        let full: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"shop.myorg".to_vec().try_into().unwrap();
        let record = DomainRegistry::<Test>::get(&full).unwrap();
        assert_eq!(record.owner, 2u64);
    });
}

#[test]
fn rollback_content_works() {
    new_test_ext().execute_with(|| {
        let original_hash = [0x01u8; 32];
        let v1_hash       = [0x02u8; 32];
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"myrollback".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"myrollback".to_vec(), 0, original_hash, false));
        // Update to v1 — saves original_hash as version 0 in history
        assert_ok!(Bns::update_hosting_content(
            RuntimeOrigin::signed(1), b"myrollback".to_vec(), v1_hash, b"v1".to_vec(), 512,
        ));
        // Rollback to version 0 (original_hash)
        assert_ok!(Bns::rollback_content(RuntimeOrigin::signed(1), b"myrollback".to_vec(), 0));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"myrollback".to_vec().try_into().unwrap();
        let hosting = HostedWebsites::<Test>::get(&domain).unwrap();
        assert_eq!(hosting.content_hash, original_hash);
    });
}

#[test]
fn update_ssl_certificate_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"secure".to_vec(), 0));
        let cert_hash = [0xCCu8; 32];
        assert_ok!(Bns::update_ssl_certificate(
            RuntimeOrigin::signed(1),
            b"secure".to_vec(),
            cert_hash,
            b"1234567890ABCDEF".to_vec(),  // serial_number
            b"Let's Encrypt".to_vec(),      // issuer
            10_000u64, // expires_at block
        ));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"secure".to_vec().try_into().unwrap();
        let ssl = SSLCertificates::<Test>::get(&domain).unwrap();
        assert_eq!(ssl.cert_hash, cert_hash);
    });
}

// ============================================================================
// ERROR PATH TESTS — domain registration
// ============================================================================

#[test]
fn register_domain_too_short_fails() {
    new_test_ext().execute_with(|| {
        // MinDomainLength in mock = 3
        assert_noop!(
            Bns::register_domain(RuntimeOrigin::signed(1), b"ab".to_vec(), 0),
            Error::<Test>::DomainTooShort
        );
    });
}

#[test]
fn register_domain_too_long_fails() {
    new_test_ext().execute_with(|| {
        // MaxDomainLength in mock = 64
        let long_name = vec![b'a'; 65];
        assert_noop!(
            Bns::register_domain(RuntimeOrigin::signed(1), long_name, 0),
            Error::<Test>::DomainTooLong
        );
    });
}

#[test]
fn register_domain_invalid_chars_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::register_domain(RuntimeOrigin::signed(1), b"my site!".to_vec(), 0),
            Error::<Test>::InvalidDomainCharacters
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — transfer_domain
// ============================================================================

#[test]
fn transfer_domain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::transfer_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 3),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn transfer_domain_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::transfer_domain(RuntimeOrigin::signed(1), b"nosuch".to_vec(), 2),
            Error::<Test>::DomainNotFound
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — set_resolution
// ============================================================================

#[test]
fn set_resolution_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::set_resolution(RuntimeOrigin::signed(2), b"mysite".to_vec(), Some(1), Some([0u8; 32]), vec![]),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn set_resolution_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::set_resolution(RuntimeOrigin::signed(1), b"nosuch".to_vec(), Some(1), Some([0u8; 32]), vec![]),
            Error::<Test>::DomainNotFound
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — buy_domain
// ============================================================================

#[test]
fn buy_domain_not_listed_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::buy_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 100),
            Error::<Test>::NotListedForSale
        );
    });
}

#[test]
fn buy_domain_own_domain_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 100_000_000_000_000, None, 1000));
        assert_noop!(
            Bns::buy_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 100_000_000_000_000),
            Error::<Test>::CannotBuyOwnDomain
        );
    });
}

#[test]
fn buy_domain_bid_too_low_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 100_000_000_000_000, None, 1000));
        assert_noop!(
            Bns::buy_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 50_000_000_000_000),
            Error::<Test>::BidTooLow
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — list_domain
// ============================================================================

#[test]
fn list_domain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::list_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), 100, None, 1000),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn list_domain_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::list_domain(RuntimeOrigin::signed(1), b"nosuch".to_vec(), 100, None, 1000),
            Error::<Test>::DomainNotFound
        );
    });
}

#[test]
fn unlist_domain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 100, None, 1000));
        assert_noop!(
            Bns::unlist_domain(RuntimeOrigin::signed(2), b"mysite".to_vec()),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn unlist_domain_not_listed_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::unlist_domain(RuntimeOrigin::signed(1), b"mysite".to_vec()),
            Error::<Test>::NotListedForSale
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — hosting
// ============================================================================

#[test]
fn activate_hosting_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::activate_hosting(RuntimeOrigin::signed(2), b"mysite".to_vec(), 0, [0u8; 32], false),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn activate_hosting_domain_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::activate_hosting(RuntimeOrigin::signed(1), b"nosuch".to_vec(), 0, [0u8; 32], false),
            Error::<Test>::DomainNotFound
        );
    });
}

#[test]
fn activate_hosting_already_active_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0, [0u8; 32], false));
        assert_noop!(
            Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0, [0u8; 32], false),
            Error::<Test>::HostingAlreadyActive
        );
    });
}

#[test]
fn deactivate_hosting_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0, [0u8; 32], false));
        assert_noop!(
            Bns::deactivate_hosting(RuntimeOrigin::signed(2), b"mysite".to_vec()),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn deactivate_hosting_not_active_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::deactivate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec()),
            Error::<Test>::HostingNotActive
        );
    });
}

#[test]
fn renew_hosting_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0, [0u8; 32], false));
        assert_noop!(
            Bns::renew_hosting(RuntimeOrigin::signed(2), b"mysite".to_vec(), 3),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn renew_hosting_not_active_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_noop!(
            Bns::renew_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 3),
            Error::<Test>::HostingNotActive
        );
    });
}

#[test]
fn update_hosting_content_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0, [0u8; 32], false));
        assert_noop!(
            Bns::update_hosting_content(RuntimeOrigin::signed(2), b"mysite".to_vec(), [0xABu8; 32], b"new".to_vec(), 512),
            Error::<Test>::NotDomainOwner
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — subdomains
// ============================================================================

#[test]
fn create_subdomain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"parent".to_vec(), 0));
        assert_noop!(
            Bns::create_subdomain(RuntimeOrigin::signed(2), b"parent".to_vec(), b"sub".to_vec(), None),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn create_subdomain_parent_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Bns::create_subdomain(RuntimeOrigin::signed(1), b"nosuch".to_vec(), b"sub".to_vec(), None),
            Error::<Test>::DomainNotFound
        );
    });
}

#[test]
fn create_subdomain_already_exists_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"parent".to_vec(), 0));
        assert_ok!(Bns::create_subdomain(RuntimeOrigin::signed(1), b"parent".to_vec(), b"sub".to_vec(), None));
        assert_noop!(
            Bns::create_subdomain(RuntimeOrigin::signed(1), b"parent".to_vec(), b"sub".to_vec(), None),
            Error::<Test>::DomainAlreadyExists
        );
    });
}

// ============================================================================
// ERROR PATH TESTS — external domain / SSL
// ============================================================================

#[test]
fn register_external_domain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mybns".to_vec(), 0));
        assert_noop!(
            Bns::register_external_domain(RuntimeOrigin::signed(2), b"example.com".to_vec(), b"mybns".to_vec(), 0),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn update_ssl_certificate_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"secure".to_vec(), 0));
        assert_noop!(
            Bns::update_ssl_certificate(RuntimeOrigin::signed(2), b"secure".to_vec(), [0u8; 32], b"serial".to_vec(), b"issuer".to_vec(), 10_000),
            Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn rollback_content_not_owner_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"myrollback".to_vec(), 0));
        assert_ok!(Bns::activate_hosting(RuntimeOrigin::signed(1), b"myrollback".to_vec(), 0, [0x01u8; 32], false));
        assert_ok!(Bns::update_hosting_content(RuntimeOrigin::signed(1), b"myrollback".to_vec(), [0x02u8; 32], b"v1".to_vec(), 512));
        assert_noop!(
            Bns::rollback_content(RuntimeOrigin::signed(2), b"myrollback".to_vec(), 0),
            Error::<Test>::NotDomainOwner
        );
    });
}

// ============================================================================
// EVENT EMISSION TESTS
// ============================================================================

#[test]
fn register_domain_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"mysite".to_vec().try_into().unwrap();
        System::assert_has_event(
            Event::<Test>::DomainRegistered {
                domain,
                owner: 1,
                price: 100_000_000_000_000,
                tier: 0,
            }.into()
        );
    });
}

#[test]
fn transfer_domain_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::transfer_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 2));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"mysite".to_vec().try_into().unwrap();
        System::assert_has_event(
            Event::<Test>::DomainTransferred {
                domain,
                from: 1,
                to: 2,
            }.into()
        );
    });
}

#[test]
fn buy_domain_emits_event() {
    new_test_ext().execute_with(|| {
        let price = 100_000_000_000_000u128;
        assert_ok!(Bns::register_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), 0));
        assert_ok!(Bns::list_domain(RuntimeOrigin::signed(1), b"mysite".to_vec(), price, None, 1000));
        assert_ok!(Bns::buy_domain(RuntimeOrigin::signed(2), b"mysite".to_vec(), price));
        let domain: BoundedVec<u8, <Test as Config>::MaxDomainLength> = b"mysite".to_vec().try_into().unwrap();
        let expected_fee = price * 5 / 100;
        System::assert_has_event(
            Event::<Test>::DomainSold {
                domain,
                seller: 1,
                buyer: 2,
                price,
                marketplace_fee: expected_fee,
            }.into()
        );
    });
}