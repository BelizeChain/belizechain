use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// Property Registration Tests
// ============================================================================

#[test]
fn register_property_works() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(ALICE);

        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Beachfront residential lot"),
            test_coordinates(0, 0),
            1000, // 1000 sqm
            0, // Residential
            500_000, // 500K bBZD assessed value
        ));

        // Verify property was created
        let property = LandLedger::properties(0).unwrap();
        assert_eq!(property.property_id, 0);
        assert_eq!(property.owner, ALICE);
        assert_eq!(property.area_sqm, 1000);
        assert_eq!(property.assessed_value, 500_000);
        assert!(!property.government_verified);
        assert!(!property.surveyed);

        // Verify deposit was reserved
        assert_eq!(
            Balances::free_balance(ALICE),
            initial_balance - RegistrationDeposit::get()
        );

        // Verify property ownership tracking
        let owned = LandLedger::get_owned_properties(&ALICE);
        assert_eq!(owned, vec![0]);

        // Verify NextPropertyId incremented
        assert_eq!(LandLedger::next_property_id(), 1);
    });
}

#[test]
fn register_property_requires_sufficient_balance() {
    new_test_ext().execute_with(|| {
        // Create account with insufficient balance
        let poor_account = 999;
        Balances::make_free_balance_be(&poor_account, 1000); // Less than RegistrationDeposit

        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(poor_account),
                test_title(1),
                test_description("Property"),
                test_coordinates(0, 0),
                100,
                0,
                100_000,
            ),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn register_property_validates_coordinates() {
    new_test_ext().execute_with(|| {
        // Invalid latitude (too far north - outside Belize)
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(1),
                test_description("Property"),
                (20_000_000, -88_000_000), // Latitude too high
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );

        // Invalid longitude (too far east - outside Belize)
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(2),
                test_description("Property"),
                (17_000_000, -85_000_000), // Longitude out of range
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );
    });
}

#[test]
fn register_property_prevents_duplicate_titles() {
    new_test_ext().execute_with(|| {
        // Register first property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("First property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // Try to register with same title number
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(BOB),
                test_title(1), // Same title
                test_description("Second property"),
                test_coordinates(100, 100),
                200,
                0,
                200_000,
            ),
            Error::<Test>::PropertyAlreadyExists
        );
    });
}

#[test]
fn register_property_validates_description_length() {
    new_test_ext().execute_with(|| {
        // Description too long (> 256 characters)
        let long_description = "x".repeat(300);

        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(1),
                long_description.into_bytes(),
                test_coordinates(0, 0),
                100,
                0,
                100_000,
            ),
            Error::<Test>::DescriptionTooLong
        );
    });
}

#[test]
fn register_property_handles_different_types() {
    new_test_ext().execute_with(|| {
        // Residential
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Residential"),
            test_coordinates(0, 0),
            100,
            0, // Residential
            100_000,
        ));

        // Tourism
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(2),
            test_description("Tourism resort"),
            test_coordinates(100, 100),
            5000,
            4, // Tourism
            2_000_000,
        ));

        let residential = LandLedger::properties(0).unwrap();
        assert_eq!(residential.property_type, PropertyType::Residential);
        assert!(!residential.is_tourism_property);

        let tourism = LandLedger::properties(1).unwrap();
        assert_eq!(tourism.property_type, PropertyType::Tourism);
        assert!(tourism.is_tourism_property);
    });
}

// ============================================================================
// Property Transfer Tests
// ============================================================================

#[test]
fn transfer_property_works() {
    new_test_ext().execute_with(|| {
        // Setup: Create property ID 1 owned by ALICE (oracle verified for ALICE)
        let property = PropertyRecord {
            property_id: 1,
            owner: ALICE,
            title_number: test_title(1).try_into().unwrap(),
            description: test_description("Property for sale").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 1000,
            property_type: PropertyType::Residential,
            assessed_value: 500_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: true, // Pre-verified
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(1, property);

        // Transfer to EVE (who has Level 2 KYC)
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1, // Property ID
            EVE,
            500_000, // Transfer price
            0, // Sale
        ));

        // Verify ownership changed
        let property = LandLedger::properties(1).unwrap();
        assert_eq!(property.owner, EVE);
        assert!(property.last_transferred.is_some());

        // Verify transfer tax was collected (2.5% of 500_000 = 12_500)
        let expected_tax = 500_000 * 250 / 10000;
        let land_registry_balance = Balances::free_balance(LandLedger::account_id());
        assert_eq!(land_registry_balance, expected_tax);

        // Verify transfer record created
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.from_owner, ALICE);
        assert_eq!(transfer.to_owner, EVE);
        assert_eq!(transfer.transfer_price, 500_000);
        assert_eq!(transfer.tax_paid, expected_tax);

        // Verify NextTransferId incremented
        assert_eq!(LandLedger::next_transfer_id(), 1);
    });
}

#[test]
fn transfer_property_requires_ownership() {
    new_test_ext().execute_with(|| {
        // Setup: Create property ID 1 owned by ALICE (oracle verified)
        let property = PropertyRecord {
            property_id: 1,
            owner: ALICE,
            title_number: test_title(1).try_into().unwrap(),
            description: test_description("Property").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 100,
            property_type: PropertyType::Residential,
            assessed_value: 100_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: true,
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(1, property);

        // BOB tries to transfer ALICE's property
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(BOB),
                1,
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn transfer_property_requires_oracle_verification() {
    new_test_ext().execute_with(|| {
        // Setup: Create property ID 200 owned by ALICE
        // Oracle only verifies properties 1-100 for ALICE, so 200 will fail verification
        let property = PropertyRecord {
            property_id: 200,
            owner: ALICE,
            title_number: test_title(200).try_into().unwrap(),
            description: test_description("Property").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 100,
            property_type: PropertyType::Residential,
            assessed_value: 100_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: true,
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(200, property);

        // Transfer should fail because Oracle doesn't verify property 200 for ALICE
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                200,
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::OwnershipVerificationFailed
        );
    });
}

#[test]
fn transfer_property_checks_sanctions() {
    new_test_ext().execute_with(|| {
        // Register property ID 101 which oracle verifies for SANCTIONED account
        // We need to manipulate storage directly to test this scenario
        let property = PropertyRecord {
            property_id: 101,
            owner: SANCTIONED,
            title_number: test_title(101).try_into().unwrap(),
            description: test_description("Sanctioned property").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 100,
            property_type: PropertyType::Residential,
            assessed_value: 100_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: true, // Pre-verified
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(101, property);

        // SANCTIONED tries to transfer
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(SANCTIONED),
                101,
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::AccountSanctioned
        );
    });
}

#[test]
fn transfer_property_requires_buyer_kyc() {
    new_test_ext().execute_with(|| {
        // Setup: Create property ID 1 owned by ALICE (oracle verified)
        let property = PropertyRecord {
            property_id: 1,
            owner: ALICE,
            title_number: test_title(1).try_into().unwrap(),
            description: test_description("Property").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 100,
            property_type: PropertyType::Residential,
            assessed_value: 100_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: true,
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(1, property);

        // Transfer to LOW_KYC (Level 1 - insufficient)
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                1,
                LOW_KYC,
                100_000,
                0,
            ),
            Error::<Test>::BuyerKycInsufficient
        );

        // Transfer to NO_KYC (no KYC data)
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                1,
                NO_KYC,
                100_000,
                0,
            ),
            Error::<Test>::BuyerKycInsufficient
        );
    });
}

#[test]
fn transfer_property_requires_government_verification() {
    new_test_ext().execute_with(|| {
        // Setup: Create property ID 1 owned by ALICE (oracle verified but not government verified)
        let property = PropertyRecord {
            property_id: 1,
            owner: ALICE,
            title_number: test_title(1).try_into().unwrap(),
            description: test_description("Property").try_into().unwrap(),
            coordinates: test_coordinates(0, 0),
            area_sqm: 100,
            property_type: PropertyType::Residential,
            assessed_value: 100_000,
            registered_at: 1,
            last_transferred: None,
            government_verified: false, // NOT verified
            surveyed: false,
            environmental_clearance: false,
            is_tourism_property: false,
            zoning: ZoningType::UrbanResidential,
            encumbrances: BoundedVec::default(),
        };
        Properties::<Test>::insert(1, property);

        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                1,
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::PropertyNotVerified
        );
    });
}

// ============================================================================
// Government Verification Tests
// ============================================================================

#[test]
fn verify_property_works() {
    new_test_ext().execute_with(|| {
        // Register property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // Initially not verified
        let property = LandLedger::properties(0).unwrap();
        assert!(!property.government_verified);

        // Government verifies
        assert_ok!(LandLedger::verify_property(
            RuntimeOrigin::root(),
            0,
        ));

        // Now verified
        let property = LandLedger::properties(0).unwrap();
        assert!(property.government_verified);
    });
}

#[test]
fn verify_property_requires_government_origin() {
    new_test_ext().execute_with(|| {
        // Register property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // Regular user cannot verify
        assert_noop!(
            LandLedger::verify_property(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn verify_property_fails_for_nonexistent() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LandLedger::verify_property(
                RuntimeOrigin::root(),
                999, // Doesn't exist
            ),
            Error::<Test>::PropertyNotFound
        );
    });
}

// ============================================================================
// Survey Tests
// ============================================================================

#[test]
fn survey_property_works() {
    new_test_ext().execute_with(|| {
        // Register surveyor
        assert_ok!(LandLedger::register_surveyor(
            RuntimeOrigin::root(),
            BOB,
        ));

        // Register property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100, // Initial area estimate
            0,
            100_000,
        ));

        // Initially not surveyed
        let property = LandLedger::properties(0).unwrap();
        assert!(!property.surveyed);
        assert_eq!(property.area_sqm, 100);

        // BOB surveys the property
        assert_ok!(LandLedger::survey_property(
            RuntimeOrigin::signed(BOB),
            0,
            150, // Actual measured area
            Some(test_coordinates(10, 10)), // Updated coordinates
        ));

        // Now surveyed with updated area
        let property = LandLedger::properties(0).unwrap();
        assert!(property.surveyed);
        assert_eq!(property.area_sqm, 150);
        assert_eq!(property.coordinates, test_coordinates(10, 10));
    });
}

#[test]
fn survey_property_requires_authorized_surveyor() {
    new_test_ext().execute_with(|| {
        // Register property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // ALICE (not a surveyor) tries to survey
        assert_noop!(
            LandLedger::survey_property(
                RuntimeOrigin::signed(ALICE),
                0,
                150,
                None,
            ),
            Error::<Test>::NotAuthorizedSurveyor
        );
    });
}

// ============================================================================
// Surveyor Registration Tests
// ============================================================================

#[test]
fn register_surveyor_works() {
    new_test_ext().execute_with(|| {
        // Initially not a surveyor
        assert!(!LandLedger::government_surveyors(BOB));

        // Government registers BOB as surveyor
        assert_ok!(LandLedger::register_surveyor(
            RuntimeOrigin::root(),
            BOB,
        ));

        // Now registered
        assert!(LandLedger::government_surveyors(BOB));
    });
}

#[test]
fn register_surveyor_requires_government_origin() {
    new_test_ext().execute_with(|| {
        // Regular user cannot register surveyor
        assert_noop!(
            LandLedger::register_surveyor(
                RuntimeOrigin::signed(ALICE),
                BOB,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// Helper Function Tests
// ============================================================================

#[test]
fn get_owned_properties_works() {
    new_test_ext().execute_with(|| {
        // ALICE registers multiple properties
        for i in 1..=3 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Property"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100 * i,
                0,
                100_000 * i as u128,
            ));
        }

        let owned = LandLedger::get_owned_properties(&ALICE);
        assert_eq!(owned.len(), 3);
        assert_eq!(owned, vec![0, 1, 2]);

        // BOB has no properties
        assert_eq!(LandLedger::get_owned_properties(&BOB), Vec::<u32>::new());
    });
}

#[test]
fn account_id_generation_works() {
    new_test_ext().execute_with(|| {
        let account_id = LandLedger::account_id();
        // Should be deterministic based on PalletId
        assert!(account_id != 0);
    });
}
