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

#[test]
fn remove_surveyor_works() {
    new_test_ext().execute_with(|| {
        // Register BOB as surveyor first
        assert_ok!(LandLedger::register_surveyor(
            RuntimeOrigin::root(),
            BOB,
        ));
        assert!(LandLedger::government_surveyors(BOB));

        // Government removes surveyor
        assert_ok!(LandLedger::remove_surveyor(
            RuntimeOrigin::root(),
            BOB,
        ));

        // BOB is no longer a surveyor
        assert!(!LandLedger::government_surveyors(BOB));
    });
}

#[test]
fn remove_surveyor_not_found_fails() {
    new_test_ext().execute_with(|| {
        // BOB was never registered
        assert_noop!(
            LandLedger::remove_surveyor(RuntimeOrigin::root(), BOB),
            Error::<Test>::SurveyorNotFound
        );
    });
}

#[test]
fn remove_surveyor_requires_government_origin() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));

        assert_noop!(
            LandLedger::remove_surveyor(RuntimeOrigin::signed(ALICE), BOB),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// Extended Property Type Tests
// ============================================================================

#[test]
fn register_commercial_property() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Commercial building"),
            test_coordinates(0, 0),
            500,
            1, // Commercial
            1_000_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Commercial);
        assert!(!prop.is_tourism_property);
        assert!(!prop.environmental_clearance);
    });
}

#[test]
fn register_agricultural_property() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Farm land"),
            test_coordinates(0, 0),
            10000,
            2, // Agricultural
            300_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Agricultural);
        assert!(!prop.is_tourism_property);
        assert!(!prop.environmental_clearance);
    });
}

#[test]
fn register_industrial_property() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Factory zone"),
            test_coordinates(0, 0),
            2000,
            3, // Industrial
            800_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Industrial);
    });
}

#[test]
fn register_government_property() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Government office"),
            test_coordinates(0, 0),
            300,
            5, // Government
            2_000_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Government);
        assert!(!prop.environmental_clearance);
    });
}

#[test]
fn register_protected_property_sets_environmental_clearance() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Conservation area"),
            test_coordinates(0, 0),
            50000,
            6, // Protected
            100_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Protected);
        // Protected type auto-sets environmental_clearance = true
        assert!(prop.environmental_clearance);
        assert!(!prop.is_tourism_property);
    });
}

#[test]
fn register_undeveloped_property() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Raw land"),
            test_coordinates(0, 0),
            8000,
            7, // Undeveloped
            50_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Undeveloped);
        assert!(!prop.environmental_clearance);
    });
}

#[test]
fn invalid_property_type_defaults_to_residential() {
    new_test_ext().execute_with(|| {
        // property_type_index >= 8 defaults to Residential
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Unknown type"),
            test_coordinates(0, 0),
            100,
            99, // Invalid index
            100_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert_eq!(prop.property_type, PropertyType::Residential);
    });
}

// ============================================================================
// Extended Transfer Type Tests
// ============================================================================

/// Helper to create a pre-verified property for transfer tests
fn setup_verified_property(owner: u64, property_id: u32) {
    let property = PropertyRecord {
        property_id,
        owner,
        title_number: test_title(property_id).try_into().unwrap(),
        description: test_description("Transfer test").try_into().unwrap(),
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
    Properties::<Test>::insert(property_id, property);
    PropertyOwners::<Test>::mutate(owner, |props| {
        let _ = props.try_push(property_id);
    });
}

#[test]
fn transfer_property_gift_type() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            0, // Gift - zero price
            1, // Gift type
        ));
        let prop = LandLedger::properties(1).unwrap();
        assert_eq!(prop.owner, EVE);
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::Gift);
        assert_eq!(transfer.transfer_price, 0);
        assert_eq!(transfer.tax_paid, 0);
    });
}

#[test]
fn transfer_property_inheritance_type() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            200_000,
            2, // Inheritance
        ));
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::Inheritance);
    });
}

#[test]
fn transfer_property_government_acquisition_type() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            1_000_000,
            3, // GovernmentAcquisition
        ));
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::GovernmentAcquisition);
    });
}

#[test]
fn transfer_property_foreclosure_type() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            300_000,
            4, // Foreclosure
        ));
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::Foreclosure);
    });
}

#[test]
fn transfer_property_court_order_type() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            0,
            5, // CourtOrder
        ));
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::CourtOrder);
    });
}

#[test]
fn invalid_transfer_type_defaults_to_sale() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            100_000,
            99, // Invalid - defaults to Sale
        ));
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.transfer_type, TransferType::Sale);
    });
}

// ============================================================================
// Transfer Edge Case Tests
// ============================================================================

#[test]
fn transfer_property_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                999, // Nonexistent
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::PropertyNotFound
        );
    });
}

#[test]
fn transfer_property_buyer_sanctioned() {
    new_test_ext().execute_with(|| {
        // Property 1, owner ALICE (oracle verifies props 1-100 for accounts 1-3)
        setup_verified_property(ALICE, 1);
        // Transfer to SANCTIONED (account 666) - buyer is sanctioned
        assert_noop!(
            LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                1,
                SANCTIONED,
                100_000,
                0,
            ),
            Error::<Test>::AccountSanctioned
        );
    });
}

#[test]
fn transfer_property_zero_price_no_tax() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        let alice_before = Balances::free_balance(ALICE);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            0, // Zero price
            0,
        ));
        // No tax should be deducted
        assert_eq!(Balances::free_balance(ALICE), alice_before);
        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.tax_paid, 0);
    });
}

#[test]
fn transfer_property_updates_ownership_lists() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        // Verify ownership before
        assert_eq!(LandLedger::get_owned_properties(&ALICE), vec![1]);
        assert_eq!(LandLedger::get_owned_properties(&EVE), Vec::<u32>::new());

        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            100_000,
            0,
        ));

        // Verify ownership after: ALICE loses property, EVE gains it
        assert_eq!(LandLedger::get_owned_properties(&ALICE), Vec::<u32>::new());
        assert_eq!(LandLedger::get_owned_properties(&EVE), vec![1]);
    });
}

#[test]
fn sequential_transfers_chain() {
    new_test_ext().execute_with(|| {
        // Property 1 owned by ALICE, transferred A→B→C
        // We need oracle-verified properties. Oracle verifies 1-100 for accounts 1-3.
        // ALICE=1, BOB=2, CHARLIE=3 all have KYC L3.
        // But we need EVE(10) to have verified property too. Let's use direct setup.
        setup_verified_property(ALICE, 1);

        // A → B (BOB has KYC L3, account 2)
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            BOB,
            500_000,
            0,
        ));
        assert_eq!(LandLedger::properties(1).unwrap().owner, BOB);
        assert_eq!(LandLedger::next_transfer_id(), 1);

        // B → C (CHARLIE has KYC L3, account 3)
        // Oracle verifies property 1 for account 2 (BOB)
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(BOB),
            1,
            CHARLIE,
            600_000,
            0,
        ));
        assert_eq!(LandLedger::properties(1).unwrap().owner, CHARLIE);
        assert_eq!(LandLedger::next_transfer_id(), 2);

        // Verify full chain
        assert_eq!(LandLedger::get_owned_properties(&ALICE), Vec::<u32>::new());
        assert_eq!(LandLedger::get_owned_properties(&BOB), Vec::<u32>::new());
        assert_eq!(LandLedger::get_owned_properties(&CHARLIE), vec![1]);

        // Both transfer records exist
        let t0 = LandLedger::transfer_records(0).unwrap();
        assert_eq!(t0.from_owner, ALICE);
        assert_eq!(t0.to_owner, BOB);
        let t1 = LandLedger::transfer_records(1).unwrap();
        assert_eq!(t1.from_owner, BOB);
        assert_eq!(t1.to_owner, CHARLIE);
    });
}

#[test]
fn transfer_tax_computed_correctly_for_large_price() {
    new_test_ext().execute_with(|| {
        setup_verified_property(ALICE, 1);
        let price: u128 = 10_000_000; // 10M
        let expected_tax = price * 250 / 10000; // 2.5% = 250,000

        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            price,
            0,
        ));

        let transfer = LandLedger::transfer_records(0).unwrap();
        assert_eq!(transfer.tax_paid, expected_tax);

        let registry_balance = Balances::free_balance(LandLedger::account_id());
        assert_eq!(registry_balance, expected_tax);
    });
}

// ============================================================================
// Coordinate Boundary Tests
// ============================================================================

#[test]
fn register_property_at_min_latitude_boundary() {
    new_test_ext().execute_with(|| {
        // Exactly at min latitude (15_000_000)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("South boundary"),
            (15_000_000, -88_000_000),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_at_max_latitude_boundary() {
    new_test_ext().execute_with(|| {
        // Exactly at max latitude (19_000_000)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("North boundary"),
            (19_000_000, -88_000_000),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_at_min_longitude_boundary() {
    new_test_ext().execute_with(|| {
        // Exactly at min longitude (-90_000_000)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("West boundary"),
            (17_000_000, -90_000_000),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_at_max_longitude_boundary() {
    new_test_ext().execute_with(|| {
        // Exactly at max longitude (-87_000_000)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("East boundary"),
            (17_000_000, -87_000_000),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_just_outside_latitude_boundaries() {
    new_test_ext().execute_with(|| {
        // One below min latitude
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(1),
                test_description("Too far south"),
                (14_999_999, -88_000_000),
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );
        // One above max latitude
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(2),
                test_description("Too far north"),
                (19_000_001, -88_000_000),
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );
    });
}

#[test]
fn register_property_just_outside_longitude_boundaries() {
    new_test_ext().execute_with(|| {
        // One below min longitude
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(1),
                test_description("Too far west"),
                (17_000_000, -90_000_001),
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );
        // One above max longitude
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(2),
                test_description("Too far east"),
                (17_000_000, -86_999_999),
                100,
                0,
                100_000,
            ),
            Error::<Test>::InvalidCoordinates
        );
    });
}

// ============================================================================
// Title & Description Edge Cases
// ============================================================================

#[test]
fn register_property_title_too_long_fails() {
    new_test_ext().execute_with(|| {
        // Title bounded to 64 bytes — exceeding that triggers DescriptionTooLong (reused error)
        let long_title = "X".repeat(65).into_bytes();
        assert_noop!(
            LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                long_title,
                test_description("Property"),
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
fn register_property_title_at_max_length_succeeds() {
    new_test_ext().execute_with(|| {
        // Exactly 64 bytes should work
        let title_64 = "A".repeat(64).into_bytes();
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            title_64,
            test_description("Prop"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_description_at_max_length_succeeds() {
    new_test_ext().execute_with(|| {
        // Exactly 256 bytes should work
        let desc_256 = "B".repeat(256).into_bytes();
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            desc_256,
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert!(LandLedger::properties(0).is_some());
    });
}

#[test]
fn register_property_empty_title_and_description() {
    new_test_ext().execute_with(|| {
        // Empty title and description should be valid (no minimum enforced)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            Vec::new(),
            Vec::new(),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert!(prop.title_number.is_empty());
        assert!(prop.description.is_empty());
    });
}

// ============================================================================
// Verification Edge Cases
// ============================================================================

#[test]
fn double_verification_is_idempotent() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        // Verify once
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 0));
        assert!(LandLedger::properties(0).unwrap().government_verified);
        // Verify again — no error
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 0));
        assert!(LandLedger::properties(0).unwrap().government_verified);
    });
}

// ============================================================================
// Survey Edge Cases
// ============================================================================

#[test]
fn survey_property_without_coordinate_update() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        let original_coords = LandLedger::properties(0).unwrap().coordinates;

        // Survey with None coordinates — should keep original
        assert_ok!(LandLedger::survey_property(
            RuntimeOrigin::signed(BOB),
            0,
            200,
            None,
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert!(prop.surveyed);
        assert_eq!(prop.area_sqm, 200);
        assert_eq!(prop.coordinates, original_coords);
    });
}

#[test]
fn survey_property_not_found() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_noop!(
            LandLedger::survey_property(
                RuntimeOrigin::signed(BOB),
                999, // Nonexistent
                200,
                None,
            ),
            Error::<Test>::PropertyNotFound
        );
    });
}

#[test]
fn double_survey_updates_values() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // First survey
        assert_ok!(LandLedger::survey_property(
            RuntimeOrigin::signed(BOB),
            0,
            200,
            Some(test_coordinates(10, 10)),
        ));
        assert_eq!(LandLedger::properties(0).unwrap().area_sqm, 200);

        // Second survey — updates again
        assert_ok!(LandLedger::survey_property(
            RuntimeOrigin::signed(BOB),
            0,
            250,
            Some(test_coordinates(20, 20)),
        ));
        let prop = LandLedger::properties(0).unwrap();
        assert!(prop.surveyed);
        assert_eq!(prop.area_sqm, 250);
        assert_eq!(prop.coordinates, test_coordinates(20, 20));
    });
}

#[test]
fn removed_surveyor_cannot_survey() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // Remove BOB as surveyor
        assert_ok!(LandLedger::remove_surveyor(RuntimeOrigin::root(), BOB));

        // BOB can no longer survey
        assert_noop!(
            LandLedger::survey_property(
                RuntimeOrigin::signed(BOB),
                0,
                200,
                None,
            ),
            Error::<Test>::NotAuthorizedSurveyor
        );
    });
}

// ============================================================================
// Surveyor Registration Edge Cases
// ============================================================================

#[test]
fn re_register_surveyor_is_idempotent() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert!(LandLedger::government_surveyors(BOB));
        // Register again — no error
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert!(LandLedger::government_surveyors(BOB));
    });
}

// ============================================================================
// PropertyByTitle Storage Tests
// ============================================================================

#[test]
fn property_by_title_lookup_works() {
    new_test_ext().execute_with(|| {
        let title = test_title(42);
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            title.clone(),
            test_description("Titled property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        // Compute hash the same way pallet does
        let title_bounded: BoundedVec<u8, ConstU32<64>> = title.try_into().unwrap();
        let title_hash = sp_io::hashing::blake2_256(&title_bounded);
        assert_eq!(LandLedger::property_by_title(title_hash), Some(0));
    });
}

#[test]
fn multiple_properties_have_distinct_title_hashes() {
    new_test_ext().execute_with(|| {
        for i in 1..=5 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Prop"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100,
                0,
                100_000,
            ));
        }
        assert_eq!(LandLedger::next_property_id(), 5);

        // Each title maps to a different property ID
        for i in 1..=5u32 {
            let title_bounded: BoundedVec<u8, ConstU32<64>> = test_title(i).try_into().unwrap();
            let hash = sp_io::hashing::blake2_256(&title_bounded);
            assert_eq!(LandLedger::property_by_title(hash), Some(i - 1));
        }
    });
}

// ============================================================================
// Event Emission Tests
// ============================================================================

#[test]
fn register_property_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let title = test_title(1);
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            title.clone(),
            test_description("Event test"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        System::assert_has_event(
            Event::PropertyRegistered {
                property_id: 0,
                owner: ALICE,
                title_number: title,
            }.into()
        );
    });
}

#[test]
fn transfer_property_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        setup_verified_property(ALICE, 1);
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            200_000,
            0,
        ));
        System::assert_has_event(
            Event::PropertyTransferred {
                property_id: 1,
                from_owner: ALICE,
                to_owner: EVE,
                transfer_price: 200_000,
                transfer_id: 0,
            }.into()
        );
    });
}

#[test]
fn verify_property_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 0));
        System::assert_has_event(
            Event::PropertyVerified {
                property_id: 0,
                verifier: LandLedger::account_id(),
            }.into()
        );
    });
}

#[test]
fn survey_property_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert_ok!(LandLedger::survey_property(
            RuntimeOrigin::signed(BOB),
            0,
            300,
            None,
        ));
        System::assert_has_event(
            Event::PropertySurveyed {
                property_id: 0,
                surveyor: BOB,
                area_sqm: 300,
            }.into()
        );
    });
}

#[test]
fn register_surveyor_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        System::assert_has_event(
            Event::SurveyorRegistered {
                surveyor: BOB,
            }.into()
        );
    });
}

#[test]
fn remove_surveyor_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(LandLedger::register_surveyor(RuntimeOrigin::root(), BOB));
        assert_ok!(LandLedger::remove_surveyor(RuntimeOrigin::root(), BOB));
        System::assert_has_event(
            Event::SurveyorRemoved {
                surveyor: BOB,
            }.into()
        );
    });
}

// ============================================================================
// Temporal Anchoring Tests
// ============================================================================

#[test]
fn registration_creates_temporal_anchor() {
    new_test_ext().execute_with(|| {
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Anchored property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        // PropertyAnchorChain should have an entry for property 0
        let anchor_hash = PropertyAnchorChain::<Test>::get(0);
        assert!(anchor_hash.is_some(), "Anchor hash should exist after registration");

        // The anchor itself should exist in LandAnchors
        let anchor = LandAnchors::<Test>::get(anchor_hash.unwrap());
        assert!(anchor.is_some(), "Temporal anchor record should exist");
        let anchor = anchor.unwrap();
        assert_eq!(anchor.version, 0); // Genesis anchor
        assert!(anchor.previous_hash.is_none()); // No predecessor
        assert_eq!(anchor.anchor_type, AnchorType::LandTitle);
    });
}

#[test]
fn transfer_updates_temporal_anchor_chain() {
    new_test_ext().execute_with(|| {
        // Start from ID=1 so oracle verifies (verifies 1-100 for accounts 1-3)
        NextPropertyId::<Test>::put(1);
        // Register property (creates genesis anchor)
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        let genesis_hash = PropertyAnchorChain::<Test>::get(1).unwrap();

        // Verify and transfer
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 1));
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            100_000,
            0,
        ));

        // Anchor chain should be updated
        let new_hash = PropertyAnchorChain::<Test>::get(1).unwrap();
        assert_ne!(new_hash, genesis_hash, "Anchor hash should change after transfer");

        // New anchor links back to genesis
        let new_anchor = LandAnchors::<Test>::get(new_hash).unwrap();
        assert_eq!(new_anchor.version, 1);
        assert_eq!(new_anchor.previous_hash, Some(genesis_hash));
    });
}

#[test]
fn anchor_chain_structure_is_consistent() {
    new_test_ext().execute_with(|| {
        // Start from ID=1 so oracle verifies
        NextPropertyId::<Test>::put(1);
        // Register property
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Chain test"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 1));

        // Transfer creates second anchor in chain
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            50_000,
            0,
        ));

        let latest_hash = PropertyAnchorChain::<Test>::get(1).unwrap();
        let latest = LandAnchors::<Test>::get(latest_hash).unwrap();

        // Verify structural integrity: version incremented, links back
        assert_eq!(latest.version, 1);
        assert!(latest.previous_hash.is_some());

        // Follow previous link to genesis
        let genesis = LandAnchors::<Test>::get(latest.previous_hash.unwrap()).unwrap();
        assert_eq!(genesis.version, 0);
        assert!(genesis.previous_hash.is_none());
        assert_eq!(genesis.anchor_type, AnchorType::LandTitle);
    });
}

#[test]
fn anchor_chain_grows_with_multiple_transfers() {
    new_test_ext().execute_with(|| {
        // Start from ID=1 so oracle verifies (1-100 for accounts 1-3)
        NextPropertyId::<Test>::put(1);
        // Register and verify
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Multi-transfer"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 1));

        let hash0 = PropertyAnchorChain::<Test>::get(1).unwrap();

        // Transfer A → B
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            BOB,
            100_000,
            0,
        ));
        let hash1 = PropertyAnchorChain::<Test>::get(1).unwrap();
        assert_ne!(hash0, hash1);

        // Transfer B → C
        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(BOB),
            1,
            CHARLIE,
            200_000,
            0,
        ));
        let hash2 = PropertyAnchorChain::<Test>::get(1).unwrap();
        assert_ne!(hash1, hash2);

        // Verify chain: version should be 2 at head
        let anchor2 = LandAnchors::<Test>::get(hash2).unwrap();
        assert_eq!(anchor2.version, 2);
        assert_eq!(anchor2.previous_hash, Some(hash1));
    });
}

// ============================================================================
// Zoning Tests
// ============================================================================

#[test]
fn property_defaults_to_mixed_use_zoning() {
    new_test_ext().execute_with(|| {
        // ZoningMap is empty in tests, so default should be MixedUse
        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Property"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));
        assert_eq!(LandLedger::properties(0).unwrap().zoning, ZoningType::MixedUse);
    });
}

#[test]
fn zoning_map_overrides_default() {
    new_test_ext().execute_with(|| {
        // Set up zoning for specific coordinates
        let coords = test_coordinates(500, 500);
        ZoningMap::<Test>::insert(coords.0, coords.1, ZoningType::TourismDevelopment);

        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Tourism zone"),
            coords,
            100,
            4, // Tourism
            100_000,
        ));
        assert_eq!(LandLedger::properties(0).unwrap().zoning, ZoningType::TourismDevelopment);
    });
}

// ============================================================================
// Multiple Properties Per Account Tests
// ============================================================================

#[test]
fn account_can_own_many_properties() {
    new_test_ext().execute_with(|| {
        for i in 1..=10u32 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Multi"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100,
                0,
                100_000,
            ));
        }
        let owned = LandLedger::get_owned_properties(&ALICE);
        assert_eq!(owned.len(), 10);
        for i in 0..10u32 {
            assert!(owned.contains(&i));
        }
    });
}

#[test]
fn transfer_one_of_many_leaves_rest() {
    new_test_ext().execute_with(|| {
        // Register 3 properties
        for i in 1..=3u32 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Prop"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100,
                0,
                100_000,
            ));
        }

        // Verify all via gov and manually set verified on the storage-inserted one
        // Register via extrinsic so we use the proper PropertyOwners path.
        // Verify property 1 (id=1)
        assert_ok!(LandLedger::verify_property(RuntimeOrigin::root(), 1));

        // Set oracle-compatible property_id to 1 so oracle verifies
        // Properties 0,1,2 were created. Transfer prop 1 which has oracle-verified id 1.
        // Oracle verifies property_id 1-100 for accounts 1-3 (ALICE=1).

        // Mark property 1 as government_verified
        Properties::<Test>::mutate(1, |p| {
            if let Some(ref mut prop) = p {
                prop.government_verified = true;
            }
        });

        assert_ok!(LandLedger::transfer_property(
            RuntimeOrigin::signed(ALICE),
            1,
            EVE,
            100_000,
            0,
        ));

        // ALICE still owns 0 and 2
        let alice_owned = LandLedger::get_owned_properties(&ALICE);
        assert_eq!(alice_owned.len(), 2);
        assert!(alice_owned.contains(&0));
        assert!(alice_owned.contains(&2));
        assert!(!alice_owned.contains(&1));

        // EVE now owns just 1
        assert_eq!(LandLedger::get_owned_properties(&EVE), vec![1]);
    });
}

// ============================================================================
// NextPropertyId / NextTransferId Counter Tests
// ============================================================================

#[test]
fn next_property_id_increments_correctly() {
    new_test_ext().execute_with(|| {
        assert_eq!(LandLedger::next_property_id(), 0);
        for i in 1..=5u32 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Prop"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100,
                0,
                100_000,
            ));
            assert_eq!(LandLedger::next_property_id(), i);
        }
    });
}

#[test]
fn next_transfer_id_increments_correctly() {
    new_test_ext().execute_with(|| {
        assert_eq!(LandLedger::next_transfer_id(), 0);
        // Set up 3 properties and transfer them sequentially
        for i in 1..=3u32 {
            setup_verified_property(ALICE, i);
        }

        for i in 1..=3u32 {
            assert_ok!(LandLedger::transfer_property(
                RuntimeOrigin::signed(ALICE),
                i,
                EVE,
                100_000,
                0,
            ));
            assert_eq!(LandLedger::next_transfer_id(), i);
        }
    });
}

// ============================================================================
// Registration Deposit Tests
// ============================================================================

#[test]
fn registration_reserves_exact_deposit() {
    new_test_ext().execute_with(|| {
        let before = Balances::free_balance(ALICE);
        let reserved_before = Balances::reserved_balance(ALICE);

        assert_ok!(LandLedger::register_property(
            RuntimeOrigin::signed(ALICE),
            test_title(1),
            test_description("Prop"),
            test_coordinates(0, 0),
            100,
            0,
            100_000,
        ));

        assert_eq!(Balances::free_balance(ALICE), before - 10_000); // RegistrationDeposit
        assert_eq!(Balances::reserved_balance(ALICE), reserved_before + 10_000);
    });
}

#[test]
fn multiple_registrations_reserve_deposits_cumulatively() {
    new_test_ext().execute_with(|| {
        let before = Balances::free_balance(ALICE);
        for i in 1..=3u32 {
            assert_ok!(LandLedger::register_property(
                RuntimeOrigin::signed(ALICE),
                test_title(i),
                test_description("Prop"),
                test_coordinates(i as i64 * 100, i as i64 * 100),
                100,
                0,
                100_000,
            ));
        }
        assert_eq!(Balances::free_balance(ALICE), before - 3 * 10_000);
        assert_eq!(Balances::reserved_balance(ALICE), 3 * 10_000);
    });
}
