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
