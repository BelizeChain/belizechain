use crate::{mock::*, Error, Event, *};
use frame_support::{assert_noop, assert_ok, traits::Currency};

// ============================================================================
// REGISTRATION TESTS
// ============================================================================

#[test]
fn register_identity_works() {
    new_test_ext().execute_with(|| {
        let alice_balance_before = Balances::free_balance(ALICE);
        
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice Citizen")
        ));
        
        // Verify fee charged
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before - 100);
        
        // Verify identity created
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        assert_eq!(id, 1000); // start_identity_id from genesis
        
        // Verify record
        let record = BelizeIdentity::identities(id).unwrap();
        assert_eq!(record.name, test_name("Alice Citizen"));
        assert_eq!(record.owner, ALICE);
        assert_eq!(record.accounts.len(), 1);
        assert_eq!(record.accounts[0], ALICE);
        assert_eq!(record.did_doc_cid, None);
        
        // Verify NextIdentityId incremented
        assert_eq!(BelizeIdentity::next_identity_id(), 1001);
        
        // Verify event
        System::assert_last_event(Event::IdentityRegistered { identity: 1000, owner: ALICE }.into());
    });
}

#[test]
fn register_identity_fails_if_already_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Try registering again
        assert_noop!(
            BelizeIdentity::register_identity(RuntimeOrigin::signed(ALICE), test_name("Alice 2")),
            Error::<Test>::IdentityExists
        );
    });
}

#[test]
fn register_identity_fails_when_paused() {
    new_test_ext().execute_with(|| {
        // Pause the pallet
        assert_ok!(BelizeIdentity::set_pause(RuntimeOrigin::root(), true));
        
        assert_noop!(
            BelizeIdentity::register_identity(RuntimeOrigin::signed(ALICE), test_name("Alice")),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn register_identity_charges_fee() {
    new_test_ext().execute_with(|| {
        let alice_balance = Balances::free_balance(ALICE);
        let vault_balance = Balances::free_balance(BelizeIdentity::account_id());
        
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Fee = 100 (from genesis)
        assert_eq!(Balances::free_balance(ALICE), alice_balance - 100);
        assert_eq!(Balances::free_balance(BelizeIdentity::account_id()), vault_balance + 100);
    });
}

// ============================================================================
// ACCOUNT LINKING TESTS
// ============================================================================

#[test]
fn link_account_works() {
    new_test_ext().execute_with(|| {
        // Register identity for Alice
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        // Link Bob's account to Alice's identity
        assert_ok!(BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), BOB));
        
        // Verify both accounts map to same identity
        assert_eq!(BelizeIdentity::identity_of(BOB), Some(id));
        
        // Verify identity record updated
        let record = BelizeIdentity::identities(id).unwrap();
        assert_eq!(record.accounts.len(), 2);
        assert!(record.accounts.contains(&ALICE));
        assert!(record.accounts.contains(&BOB));
        
        // Verify event
        System::assert_last_event(Event::AccountLinked { identity: id, account: BOB }.into());
    });
}

#[test]
fn link_account_fails_without_identity() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), BOB),
            Error::<Test>::IdentityNotFound
        );
    });
}

#[test]
fn link_account_fails_if_target_has_identity() {
    new_test_ext().execute_with(|| {
        // Both have identities
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(BOB),
            test_name("Bob")
        ));
        
        // Cannot link
        assert_noop!(
            BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), BOB),
            Error::<Test>::IdentityExists
        );
    });
}

#[test]
fn link_account_respects_max_accounts() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Link up to max (MaxAccountsPerIdentity = 5)
        assert_ok!(BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), BOB));
        assert_ok!(BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), CHARLIE));
        assert_ok!(BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), DAVE));
        assert_ok!(BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), EVE));
        
        // Now at 5 accounts (ALICE + 4 linked)
        // Try linking 6th account
        assert_noop!(
            BelizeIdentity::link_account(RuntimeOrigin::signed(ALICE), 6),
            Error::<Test>::TooManyAccounts
        );
    });
}

// ============================================================================
// DID DOCUMENT TESTS
// ============================================================================

#[test]
fn update_did_doc_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        let cid = test_anchor(1);
        assert_ok!(BelizeIdentity::update_did_doc(RuntimeOrigin::signed(ALICE), cid.clone()));
        
        // Verify updated
        let record = BelizeIdentity::identities(id).unwrap();
        assert_eq!(record.did_doc_cid, Some(cid));
        
        System::assert_last_event(Event::DidDocUpdated { identity: id }.into());
    });
}

#[test]
fn update_did_doc_fails_without_identity() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeIdentity::update_did_doc(RuntimeOrigin::signed(ALICE), test_anchor(1)),
            Error::<Test>::IdentityNotFound
        );
    });
}

#[test]
fn did_of_returns_correct_format() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        let did = BelizeIdentity::did_of(&ALICE).unwrap();
        let did_str = String::from_utf8(did.into_inner()).unwrap();
        assert_eq!(did_str, "did:belize:1000");
    });
}

// ============================================================================
// ISSUER MANAGEMENT TESTS
// ============================================================================

#[test]
fn add_issuer_works() {
    new_test_ext().execute_with(|| {
        let new_issuer = 20;
        
        // Fund new issuer
        Balances::make_free_balance_be(&new_issuer, 10_000_000);
        
        // First deposit bond
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(new_issuer),
            AttributeType::Ssn as u8
        ));
        
        // Admin adds issuer
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            new_issuer
        ));
        
        // Verify added to SSN issuers
        let issuers = BelizeIdentity::ssn_issuers();
        assert!(issuers.contains(&new_issuer));
        
        System::assert_last_event(Event::IssuerAdded {
            attr: AttributeType::Ssn as u8,
            issuer: new_issuer,
        }.into());
    });
}

#[test]
fn add_issuer_fails_without_bond() {
    new_test_ext().execute_with(|| {
        let new_issuer = 20;
        
        // Try adding without bond
        assert_noop!(
            BelizeIdentity::add_issuer(RuntimeOrigin::root(), AttributeType::Ssn as u8, new_issuer),
            Error::<Test>::BondInsufficient
        );
    });
}

#[test]
fn remove_issuer_works() {
    new_test_ext().execute_with(|| {
        // SSN_ISSUER is initialized in genesis
        assert_ok!(BelizeIdentity::remove_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            SSN_ISSUER
        ));
        
        let issuers = BelizeIdentity::ssn_issuers();
        assert!(!issuers.contains(&SSN_ISSUER));
    });
}

#[test]
fn set_standard_version_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::set_standard_version(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            2
        ));
        
        assert_eq!(BelizeIdentity::ssn_standard_version(), 2);
    });
}

#[test]
fn set_operation_fee_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::set_operation_fee(RuntimeOrigin::root(), 500));
        
        assert_eq!(BelizeIdentity::operation_fee(), 500);
        
        // Verify new fee applies
        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        assert_eq!(Balances::free_balance(ALICE), balance_before - 500);
    });
}

// ============================================================================
// SSN ATTESTATION TESTS
// ============================================================================

#[test]
fn issue_ssn_works() {
    new_test_ext().execute_with(|| {
        // Setup: Alice registers identity
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        // Issue SSN attestation
        let hash = test_hash(1);
        let anchor = test_anchor(1);
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            hash,
            anchor.clone(),
            true
        ));
        
        // Verify attestation stored
        let att = BelizeIdentity::ssn_attestation(id).unwrap();
        assert_eq!(att.attr_type, AttributeType::Ssn);
        assert_eq!(att.issuer, SSN_ISSUER);
        assert_eq!(att.hash, hash);
        assert_eq!(att.anchor, anchor);
        assert!(att.format_ok);
        assert_eq!(att.status, AttestationStatus::Active);
        assert_eq!(att.standard_version, 1);
        
        // Verify validity windows
        assert_eq!(att.issued_at, 1);
        assert_eq!(att.valid_until, 101); // 1 + 100
        assert_eq!(att.grace_until, 121); // 101 + 20
        
        // Verify history
        let history = BelizeIdentity::attribute_history(id, AttributeType::Ssn);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].action, HistoryAction::Issued);
        
        System::assert_last_event(Event::Attested {
            identity: id,
            attr: AttributeType::Ssn as u8,
            issuer: SSN_ISSUER,
        }.into());
    });
}

#[test]
fn issue_ssn_fails_without_authorization() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Bob is not an authorized SSN issuer
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(BOB),
                ALICE,
                test_hash(1),
                test_anchor(1),
                true
            ),
            Error::<Test>::NotAuthorizedIssuer
        );
    });
}

#[test]
fn issue_ssn_fails_if_issuer_flagged() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Flag the issuer
        assert_ok!(BelizeIdentity::flag_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            SSN_ISSUER,
            true
        ));
        
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                ALICE,
                test_hash(1),
                test_anchor(1),
                true
            ),
            Error::<Test>::IssuerFlagged
        );
    });
}

#[test]
fn issue_ssn_prevents_duplicate_hash() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(BOB),
            test_name("Bob")
        ));
        
        let hash = test_hash(1);
        
        // Issue to Alice
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            hash,
            test_anchor(1),
            true
        ));
        
        // Try issuing same hash to Bob
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                BOB,
                hash,
                test_anchor(2),
                true
            ),
            Error::<Test>::HashAlreadyTaken
        );
    });
}

#[test]
fn issue_ssn_respects_rate_limit() {
    new_test_ext().execute_with(|| {
        // Create 11 identities
        for i in 0..11 {
            let account = 200 + i;
            Balances::make_free_balance_be(&account, 1_000_000);
            assert_ok!(BelizeIdentity::register_identity(
                RuntimeOrigin::signed(account),
                test_name("User")
            ));
        }
        
        // Issue 10 SSNs (rate limit from genesis)
        for i in 0..10 {
            assert_ok!(BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                200 + i,
                test_hash(i as u8),
                test_anchor(i as u8),
                true
            ));
        }
        
        // 11th should fail due to rate limit
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                210,
                test_hash(10),
                test_anchor(10),
                true
            ),
            Error::<Test>::IssuerRateLimitExceeded
        );
        
        // Advance past rate window (50 blocks)
        run_to_block(52);
        
        // Now it should work
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            210,
            test_hash(10),
            test_anchor(10),
            true
        ));
    });
}

// ============================================================================
// PASSPORT ATTESTATION TESTS
// ============================================================================

#[test]
fn issue_passport_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        let hash = test_hash(2);
        let anchor = test_anchor(2);
        assert_ok!(BelizeIdentity::issue_passport(
            RuntimeOrigin::signed(PASSPORT_ISSUER),
            ALICE,
            hash,
            anchor.clone(),
            true
        ));
        
        let att = BelizeIdentity::passport_attestation(id).unwrap();
        assert_eq!(att.attr_type, AttributeType::Passport);
        assert_eq!(att.issuer, PASSPORT_ISSUER);
        assert_eq!(att.hash, hash);
    });
}

#[test]
fn issue_passport_prevents_duplicate_hash() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(BOB),
            test_name("Bob")
        ));
        
        let hash = test_hash(2);
        
        assert_ok!(BelizeIdentity::issue_passport(
            RuntimeOrigin::signed(PASSPORT_ISSUER),
            ALICE,
            hash,
            test_anchor(2),
            true
        ));
        
        assert_noop!(
            BelizeIdentity::issue_passport(
                RuntimeOrigin::signed(PASSPORT_ISSUER),
                BOB,
                hash,
                test_anchor(3),
                true
            ),
            Error::<Test>::HashAlreadyTaken
        );
    });
}

// ============================================================================
// BIOMETRICS ATTESTATION TESTS
// ============================================================================

#[test]
fn issue_biometrics_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        let anchor = test_anchor(3);
        assert_ok!(BelizeIdentity::issue_biometrics(
            RuntimeOrigin::signed(BIO_ISSUER),
            ALICE,
            anchor.clone()
        ));
        
        let att = BelizeIdentity::biometric_attestation(id).unwrap();
        assert_eq!(att.attr_type, AttributeType::Biometrics);
        assert_eq!(att.issuer, BIO_ISSUER);
        assert_eq!(att.anchor, anchor);
    });
}

// ============================================================================
// REVOCATION AND SUSPENSION TESTS
// ============================================================================

#[test]
fn revoke_attestation_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        // Issue SSN
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        // Revoke it
        assert_ok!(BelizeIdentity::revoke(
            RuntimeOrigin::root(),
            ALICE,
            AttributeType::Ssn as u8
        ));
        
        // Verify status changed
        let att = BelizeIdentity::ssn_attestation(id).unwrap();
        assert_eq!(att.status, AttestationStatus::Revoked);
        
        // Verify history
        let history = BelizeIdentity::attribute_history(id, AttributeType::Ssn);
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].action, HistoryAction::Revoked);
    });
}

#[test]
fn suspend_attestation_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        let id = BelizeIdentity::identity_of(ALICE).unwrap();
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        assert_ok!(BelizeIdentity::suspend(
            RuntimeOrigin::root(),
            ALICE,
            AttributeType::Ssn as u8
        ));
        
        let att = BelizeIdentity::ssn_attestation(id).unwrap();
        assert_eq!(att.status, AttestationStatus::Suspended);
    });
}

#[test]
fn revoke_fails_without_attestation() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_noop!(
            BelizeIdentity::revoke(RuntimeOrigin::root(), ALICE, AttributeType::Ssn as u8),
            Error::<Test>::NoAttestation
        );
    });
}

// ============================================================================
// KYC LEVEL TESTS
// ============================================================================

#[test]
fn kyc_level_l1_requires_ssn() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // No SSN yet
        assert!(!BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L1, 1));
        
        // Issue SSN
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        // Now L1 verified
        assert!(BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L1, 1));
    });
}

#[test]
fn kyc_level_l2_requires_ssn_and_passport() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Just SSN is not enough for L2
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        assert!(!BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L2, 1));
        
        // Add passport
        assert_ok!(BelizeIdentity::issue_passport(
            RuntimeOrigin::signed(PASSPORT_ISSUER),
            ALICE,
            test_hash(2),
            test_anchor(2),
            true
        ));
        
        // Now L2 verified
        assert!(BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L2, 1));
    });
}

#[test]
fn kyc_level_l3_requires_all_three() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        assert_ok!(BelizeIdentity::issue_passport(
            RuntimeOrigin::signed(PASSPORT_ISSUER),
            ALICE,
            test_hash(2),
            test_anchor(2),
            true
        ));
        
        // Not L3 yet
        assert!(!BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L3, 1));
        
        // Add biometrics
        assert_ok!(BelizeIdentity::issue_biometrics(
            RuntimeOrigin::signed(BIO_ISSUER),
            ALICE,
            test_anchor(3)
        ));
        
        // Now L3 verified
        assert!(BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L3, 1));
    });
}

#[test]
fn kyc_expires_after_validity_blocks() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        // Valid at block 1
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 1), KycState::Valid);
        
        // Valid at block 100
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 100), KycState::Valid);
        
        // Valid at block 101 (exactly at valid_until)
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 101), KycState::Valid);
        
        // Grace at block 102
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 102), KycState::Grace);
        
        // Grace until block 121
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 121), KycState::Grace);
        
        // Invalid at block 122
        assert_eq!(BelizeIdentity::kyc_state(&ALICE, KycLevel::L1, 122), KycState::Invalid);
    });
}

#[test]
fn kyc_invalid_when_revoked() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        assert!(BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L1, 1));
        
        // Revoke
        assert_ok!(BelizeIdentity::revoke(
            RuntimeOrigin::root(),
            ALICE,
            AttributeType::Ssn as u8
        ));
        
        assert!(!BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L1, 1));
    });
}

#[test]
fn kyc_invalid_when_suspended() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        assert_ok!(BelizeIdentity::suspend(
            RuntimeOrigin::root(),
            ALICE,
            AttributeType::Ssn as u8
        ));
        
        assert!(!BelizeIdentity::is_kyc_verified(&ALICE, KycLevel::L1, 1));
    });
}

// ============================================================================
// ISSUER BOND TESTS
// ============================================================================

#[test]
fn issuer_deposit_bond_works() {
    new_test_ext().execute_with(|| {
        let new_issuer = 30;
        Balances::make_free_balance_be(&new_issuer, 5_000_000);
        
        let balance_before = Balances::free_balance(new_issuer);
        
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(new_issuer),
            AttributeType::Ssn as u8
        ));
        
        // Verify bond transferred
        assert_eq!(Balances::free_balance(new_issuer), balance_before - 1_000_000);
        
        // Verify bond recorded
        assert_eq!(
            BelizeIdentity::issuer_bond(AttributeType::Ssn, new_issuer),
            1_000_000
        );
    });
}

#[test]
fn issuer_withdraw_bond_works() {
    new_test_ext().execute_with(|| {
        let new_issuer = 30;
        Balances::make_free_balance_be(&new_issuer, 5_000_000);
        
        // Deposit bond
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(new_issuer),
            AttributeType::Ssn as u8
        ));
        
        let balance_after_deposit = Balances::free_balance(new_issuer);
        
        // Withdraw (not authorized, so can withdraw)
        assert_ok!(BelizeIdentity::issuer_withdraw_bond(
            RuntimeOrigin::signed(new_issuer),
            AttributeType::Ssn as u8
        ));
        
        // Verify bond returned
        assert_eq!(Balances::free_balance(new_issuer), balance_after_deposit + 1_000_000);
        
        // Verify bond removed
        assert_eq!(BelizeIdentity::issuer_bond(AttributeType::Ssn, new_issuer), 0);
    });
}

#[test]
fn issuer_cannot_withdraw_while_authorized() {
    new_test_ext().execute_with(|| {
        // SSN_ISSUER is authorized from genesis and has bond
        Balances::make_free_balance_be(&SSN_ISSUER, 10_000_000);
        
        // Ensure bond exists
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(SSN_ISSUER),
            AttributeType::Ssn as u8
        ));
        
        // Try to withdraw
        assert_noop!(
            BelizeIdentity::issuer_withdraw_bond(
                RuntimeOrigin::signed(SSN_ISSUER),
                AttributeType::Ssn as u8
            ),
            Error::<Test>::CannotWithdrawWhileAuthorized
        );
    });
}

#[test]
fn slash_issuer_bond_works() {
    new_test_ext().execute_with(|| {
        let new_issuer = 30;
        Balances::make_free_balance_be(&new_issuer, 5_000_000);
        
        // Deposit bond
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(new_issuer),
            AttributeType::Ssn as u8
        ));
        
        let treasury_before = Balances::free_balance(TREASURY);
        
        // Slash 500K
        assert_ok!(BelizeIdentity::slash_issuer_bond(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            new_issuer,
            500_000
        ));
        
        // Verify bond reduced
        assert_eq!(
            BelizeIdentity::issuer_bond(AttributeType::Ssn, new_issuer),
            500_000
        );
        
        // Verify treasury received funds
        assert_eq!(Balances::free_balance(TREASURY), treasury_before + 500_000);
    });
}

#[test]
fn report_bad_attestation_flags_and_slashes() {
    new_test_ext().execute_with(|| {
        let bad_issuer = 30;
        Balances::make_free_balance_be(&bad_issuer, 5_000_000);
        
        // Deposit bond
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(bad_issuer),
            AttributeType::Ssn as u8
        ));
        
        // Report bad attestation
        assert_ok!(BelizeIdentity::report_bad_attestation(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            bad_issuer,
            true, // flag
            300_000 // slash amount
        ));
        
        // Verify flagged
        assert!(BelizeIdentity::is_issuer_flagged(AttributeType::Ssn, bad_issuer));
        
        // Verify slashed
        assert_eq!(
            BelizeIdentity::issuer_bond(AttributeType::Ssn, bad_issuer),
            700_000
        );
    });
}

// ============================================================================
// ORACLE INTEGRATION TESTS
// ============================================================================

#[test]
fn oracle_verified_account_meets_kyc_requirement() {
    new_test_ext().execute_with(|| {
        // ORACLE_VERIFIED (100) is oracle-verified for level <= 2 in MockOracle
        assert!(BelizeIdentity::meets_kyc_requirement_level(&ORACLE_VERIFIED, 1));
        assert!(BelizeIdentity::meets_kyc_requirement_level(&ORACLE_VERIFIED, 2));
        assert!(!BelizeIdentity::meets_kyc_requirement_level(&ORACLE_VERIFIED, 3));
    });
}

#[test]
fn sanctioned_account_detected() {
    new_test_ext().execute_with(|| {
        // SANCTIONED (666) is marked as sanctioned in MockOracle
        assert!(BelizeIdentity::is_account_sanctioned(&SANCTIONED));
        assert!(!BelizeIdentity::is_account_sanctioned(&ALICE));
    });
}

// ============================================================================
// PAUSE/RESUME TESTS
// ============================================================================

#[test]
fn pause_and_resume_works() {
    new_test_ext().execute_with(|| {
        // Pause
        assert_ok!(BelizeIdentity::set_pause(RuntimeOrigin::root(), true));
        assert!(BelizeIdentity::paused());
        
        // Operations should fail
        assert_noop!(
            BelizeIdentity::register_identity(RuntimeOrigin::signed(ALICE), test_name("Alice")),
            Error::<Test>::Paused
        );
        
        // Resume
        assert_ok!(BelizeIdentity::set_pause(RuntimeOrigin::root(), false));
        assert!(!BelizeIdentity::paused());
        
        // Operations should work
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
    });
}

// ============================================================================
// RATE LIMIT CONFIGURATION TESTS
// ============================================================================

#[test]
fn set_rate_limits_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::set_rate_limits(
            RuntimeOrigin::root(),
            100, // window
            20,  // SSN limit
            10,  // Passport limit
            5    // Biometrics limit
        ));
        
        // Verifying the call succeeded is sufficient
    });
}

// ============================================================================
// Edge Case Tests Added for Hardening (10 tests)
// ============================================================================

#[test]
fn register_identity_accepts_max_length_name() {
    new_test_ext().execute_with(|| {
        let max_name = "A".repeat(64); // MaxNameLen = 64
        let name_vec = test_name(&max_name);
        
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            name_vec
        ));
        
        let id = IdentityOf::<Test>::get(ALICE).unwrap();
        let identity = Identities::<Test>::get(id).unwrap();
        assert_eq!(identity.name.len(), 64);
    });
}

#[test]
fn link_account_fails_when_max_accounts_reached() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // MaxAccounts = 5 in mock.rs, Alice already has 1 account (herself)
        // Link 4 more accounts (2-5) to reach the limit
        for i in 2u64..=5u64 {
            assert_ok!(BelizeIdentity::link_account(
                RuntimeOrigin::signed(ALICE),  // Alice links new accounts
                i  // The new account to link
            ));
        }
        
        // 6th account should fail (total would be 6)
        assert_noop!(
            BelizeIdentity::link_account(
                RuntimeOrigin::signed(ALICE),
                6u64
            ),
            Error::<Test>::TooManyAccounts
        );
    });
}

#[test]
fn concurrent_registrations_maintain_unique_ids() {
    new_test_ext().execute_with(|| {
        let mut ids = Vec::new();
        
        // Register identities using pre-funded accounts (ALICE=1, BOB=2, etc.)
        // Test with just 5 accounts since they're pre-funded in mock
        let accounts = [ALICE, BOB, CHARLIE, DAVE, EVE];
        
        for (idx, account) in accounts.iter().enumerate() {
            let name_str = format!("User{}", idx);
            assert_ok!(BelizeIdentity::register_identity(
                RuntimeOrigin::signed(*account),
                test_name(&name_str)
            ));
            let id = IdentityOf::<Test>::get(*account).unwrap();
            ids.push(id);
        }
        
        // Verify all IDs are unique
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique_ids.len(), 5);
    });
}

#[test]
fn identity_registration_with_zero_fee() {
    new_test_ext().execute_with(|| {
        // Set registration fee to 0
        assert_ok!(BelizeIdentity::set_operation_fee(
            RuntimeOrigin::root(),
            0
        ));
        
        let balance_before = Balances::free_balance(ALICE);
        
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        let balance_after = Balances::free_balance(ALICE);
        assert_eq!(balance_before, balance_after); // No fee charged
    });
}

#[test]
fn pause_and_resume_works_correctly() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        // Add issuer with bond deposit
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(SSN_ISSUER),
            AttributeType::Ssn as u8
        ));
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            SSN_ISSUER
        ));
        
        // Pause the pallet
        assert_ok!(BelizeIdentity::set_pause(RuntimeOrigin::root(), true));
        
        // Attestation operations should fail when paused
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                ALICE,
                test_hash(1),
                test_anchor(1),
                true
            ),
            Error::<Test>::Paused
        );
        
        // Resume
        assert_ok!(BelizeIdentity::set_pause(RuntimeOrigin::root(), false));
        
        // Operations should work again
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            test_hash(1),
            test_anchor(1),
            true
        ));
    });
}

#[test]
fn zero_bond_allows_free_issuer_bond_deposit() {
    new_test_ext().execute_with(|| {
        // Set issuer bond to 0
        assert_ok!(BelizeIdentity::set_issuer_bond_amount(
            RuntimeOrigin::root(),
            0
        ));
        
        let issuer = 5u64;
        let balance_before = Balances::free_balance(issuer);
        
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            issuer
        ));
        
        let balance_after = Balances::free_balance(issuer);
        assert_eq!(balance_before, balance_after); // No bond charged
    });
}

#[test]
fn issuer_bond_withdrawal_works() {
    new_test_ext().execute_with(|| {
        let issuer = 5u64;
        let bond_amount = 1000u128;
        
        // Set and deposit bond
        assert_ok!(BelizeIdentity::set_issuer_bond_amount(
            RuntimeOrigin::root(),
            bond_amount
        ));
        
        // Deposit the bond
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(issuer),
            AttributeType::Ssn as u8
        ));
        
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            issuer
        ));
        
        let balance_after_bond = Balances::free_balance(issuer);
        
        // Remove issuer - bond should be returned
        assert_ok!(BelizeIdentity::remove_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            issuer
        ));
        
        // Withdraw the bond
        assert_ok!(BelizeIdentity::issuer_withdraw_bond(
            RuntimeOrigin::signed(issuer),
            AttributeType::Ssn as u8
        ));
        
        let balance_final = Balances::free_balance(issuer);
        assert_eq!(balance_final, balance_after_bond + bond_amount);
    });
}

#[test]
fn rate_limits_prevent_spam() {
    new_test_ext().execute_with(|| {
        // Set aggressive rate limits
        assert_ok!(BelizeIdentity::set_rate_limits(
            RuntimeOrigin::root(),
            10,  // window_blocks = 10
            2,   // max_per_window_ssn = 2
            2,   // passport
            2    // biometric
        ));
        
        // Deposit bond first
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(SSN_ISSUER),
            AttributeType::Ssn as u8
        ));
        
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            SSN_ISSUER
        ));
        
        // Register 3 identities
        for i in 1u64..=3u64 {
            let name_str = format!("User{}", i);
            assert_ok!(BelizeIdentity::register_identity(
                RuntimeOrigin::signed(i),
                test_name(&name_str)
            ));
        }
        
        // Issue 2 attestations (within limit)
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            1u64,
            test_hash(1),
            test_anchor(1),
            true
        ));
        
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            2u64,
            test_hash(2),
            test_anchor(2),
            true
        ));
        
        // 3rd attestation should fail (rate limit exceeded)
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                3u64,
                test_hash(3),
                test_anchor(3),
                true
            ),
            Error::<Test>::IssuerRateLimitExceeded
        );
    });
}

#[test]
fn ssn_duplicate_prevention_works() {
    new_test_ext().execute_with(|| {
        // Deposit bond first
        assert_ok!(BelizeIdentity::issuer_deposit_bond(
            RuntimeOrigin::signed(SSN_ISSUER),
            AttributeType::Ssn as u8
        ));
        
        // Add issuer
        assert_ok!(BelizeIdentity::add_issuer(
            RuntimeOrigin::root(),
            AttributeType::Ssn as u8,
            SSN_ISSUER
        ));
        
        // Register 2 identities
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(BOB),
            test_name("Bob")
        ));
        
        let ssn_hash = test_hash(99);
        
        // Issue SSN to Alice
        assert_ok!(BelizeIdentity::issue_ssn(
            RuntimeOrigin::signed(SSN_ISSUER),
            ALICE,
            ssn_hash,
            test_anchor(1),
            true
        ));
        
        // Try to issue same SSN hash to Bob (should fail)
        assert_noop!(
            BelizeIdentity::issue_ssn(
                RuntimeOrigin::signed(SSN_ISSUER),
                BOB,
                ssn_hash,
                test_anchor(2),
                true
            ),
            Error::<Test>::HashAlreadyTaken
        );
    });
}

#[test]
fn did_document_update_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeIdentity::register_identity(
            RuntimeOrigin::signed(ALICE),
            test_name("Alice")
        ));
        
        let alice_id = IdentityOf::<Test>::get(ALICE).unwrap();
        let did_doc = test_anchor(5);
        
        assert_ok!(BelizeIdentity::update_did_doc(
            RuntimeOrigin::signed(ALICE),
            did_doc.clone()
        ));
        
        let identity = Identities::<Test>::get(alice_id).unwrap();
        assert_eq!(identity.did_doc_cid, Some(did_doc));
    });
}

