//! Tests for the oracle pallet

use crate::{mock::*, Error, Event, types::*};
use frame_support::{assert_noop, assert_ok, BoundedVec, traits::ConstU32};

// Test accounts
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DAVE: u64 = 4;

// ================================
// Operator Management Tests
// ================================

#[test]
fn add_operator_works() {
    new_test_ext().execute_with(|| {
        // Alice and Bob are already operators from genesis
        assert!(Oracle::oracle_operators(ALICE));
        assert!(Oracle::oracle_operators(BOB));

        // Add Charlie as operator
        assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), CHARLIE));

        assert!(Oracle::oracle_operators(CHARLIE));

        // Check event
        System::assert_last_event(Event::OperatorAdded { operator: CHARLIE }.into());
    });
}

#[test]
fn add_duplicate_operator_fails() {
    new_test_ext().execute_with(|| {
        // Alice is already an operator
        assert_noop!(
            Oracle::add_operator(RuntimeOrigin::root(), ALICE),
            Error::<Test>::OperatorAlreadyExists
        );
    });
}

#[test]
fn remove_operator_works() {
    new_test_ext().execute_with(|| {
        assert!(Oracle::oracle_operators(BOB));

        // Remove Bob
        assert_ok!(Oracle::remove_operator(RuntimeOrigin::root(), BOB));

        assert!(!Oracle::oracle_operators(BOB));

        // Check event
        System::assert_last_event(Event::OperatorRemoved { operator: BOB }.into());
    });
}

#[test]
fn remove_non_existent_operator_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Oracle::remove_operator(RuntimeOrigin::root(), CHARLIE),
            Error::<Test>::OperatorNotFound
        );
    });
}

// ================================
// Price Feed Tests
// ================================
// NOTE: Tests use BZD/USD pairs for convenience, but in production
// bBZD stablecoin is pegged 1:1 to BZD (Belize Dollar) governed by
// the central bank through the Economy pallet. The oracle provides merchant verification.

#[test]
fn submit_price_works() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);
        let price = 500000u128; // 0.50 USD per BZD (test value)

        // Alice submits price
        assert_ok!(Oracle::submit_price(
            RuntimeOrigin::signed(ALICE),
            pair.base.to_u8(),
            pair.quote.to_u8(),
            price
        ));

        // Check event
        System::assert_has_event(Event::PriceSubmitted {
            operator: ALICE,
            base_currency: pair.base.to_u8(),
            quote_currency: pair.quote.to_u8(),
            price,
        }.into());
    });
}

#[test]
fn unauthorized_price_submission_fails() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);

        // Charlie is not an operator
        assert_noop!(
            Oracle::submit_price(RuntimeOrigin::signed(CHARLIE), pair.base.to_u8(), pair.quote.to_u8(), 500000),
            Error::<Test>::NotAuthorizedOperator
        );
    });
}

#[test]
fn zero_price_submission_fails() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);

        assert_noop!(
            Oracle::submit_price(RuntimeOrigin::signed(ALICE), pair.base.to_u8(), pair.quote.to_u8(), 0),
            Error::<Test>::InvalidPrice
        );
    });
}

#[test]
fn price_aggregation_works() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);

        // Alice submits 0.50
        assert_ok!(Oracle::submit_price(
            RuntimeOrigin::signed(ALICE),
            pair.base.to_u8(),
            pair.quote.to_u8(),
            500000
        ));

        // Bob submits 0.51
        assert_ok!(Oracle::submit_price(
            RuntimeOrigin::signed(BOB),
            pair.base.to_u8(),
            pair.quote.to_u8(),
            510000
        ));

        // Check aggregated feed (median should be 505000)
        let feed = Oracle::price_feeds(pair).unwrap();
        assert_eq!(feed.price, 505000);
        assert_eq!(feed.num_submissions, 2);
    });
}

// NOTE: Commented out because MinConsensusOperators is set to 1 for testing
// #[test]
// fn insufficient_consensus_fails() {
//     new_test_ext().execute_with(|| {
//         // Remove Bob so we only have 1 operator
//         assert_ok!(Oracle::remove_operator(RuntimeOrigin::root(), BOB));
//         let pair = CurrencyPair::new(Currency::EUR, Currency::USD);
//         // Alice submits price (only 1 operator, need 2)
//         assert_noop!(
//             Oracle::submit_price(RuntimeOrigin::signed(ALICE), pair.base.to_u8(), pair.quote.to_u8(), 1_100_000),
//             Error::<Test>::InsufficientConsensus
//         );
//     });
// }

#[test]
fn get_exchange_rate_works() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);

        // Submit prices
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(ALICE), pair.base.to_u8(), pair.quote.to_u8(), 500000));
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(BOB), pair.base.to_u8(), pair.quote.to_u8(), 500000));

        // Get rate
        let rate = Oracle::get_exchange_rate(pair);
        assert_eq!(rate, Some(500000));
    });
}

#[test]
fn stale_price_returns_none() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::BZD, Currency::USD);

        // Submit price at block 1
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(ALICE), pair.base.to_u8(), pair.quote.to_u8(), 500000));
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(BOB), pair.base.to_u8(), pair.quote.to_u8(), 500000));

        // Advance past staleness limit (100 blocks)
        System::set_block_number(150);

        // Rate should be None (stale)
        let rate = Oracle::get_exchange_rate(pair);
        assert_eq!(rate, None);
    });
}

#[test]
fn stale_price_emits_event() {
    new_test_ext().execute_with(|| {
        let pair = CurrencyPair::new(Currency::USD, Currency::BZD);

        // Set manual exchange rate via admin (1 USD = 2.0 BZD)
        assert_ok!(Oracle::update_exchange_rate(RuntimeOrigin::root(), pair.base.to_u8(), pair.quote.to_u8(), 2_000_000));

        // Capture current block and advance past staleness threshold
        // In tests, MaxDataStaleness is 100 blocks (see mock.rs)
        System::set_block_number(150);

        // Calling get_exchange_rate should emit ExchangeRateStale and return None
        let rate = Oracle::get_exchange_rate(pair);
        assert_eq!(rate, None);

        System::assert_has_event(Event::ExchangeRateStale {
            base_currency: pair.base.to_u8(),
            quote_currency: pair.quote.to_u8(),
        }.into());
    });
}

// ================================
// Merchant Verification Tests
// ================================

#[test]
fn verify_merchant_works() {
    new_test_ext().execute_with(|| {
        let merchant = DAVE;
        let category = MerchantCategory::Accommodation;
        let cert: BoundedVec<u8, ConstU32<128>> = b"TB-12345".to_vec().try_into().unwrap();
        let license: BoundedVec<u8, ConstU32<128>> = b"BL-67890".to_vec().try_into().unwrap();
        let location = Some((17_250_000, -88_750_000)); // Belize coordinates

        assert_ok!(Oracle::verify_merchant(
            RuntimeOrigin::signed(ALICE),
            merchant,
            category.to_u8(),
            cert.clone(),
            license.clone(),
            location
        ));

        // Check merchant info
        let info = Oracle::merchant_categories(merchant).unwrap();
        assert_eq!(info.category, category);
        assert_eq!(info.certification, cert);
        assert_eq!(info.license, license);
        assert_eq!(info.location, location);

        // Check event
        System::assert_last_event(Event::MerchantVerified {
            merchant,
            category: category.to_u8(),
        }.into());
    });
}

#[test]
fn get_merchant_category_works() {
    new_test_ext().execute_with(|| {
        let merchant = DAVE;
        let category = MerchantCategory::TourOperator;
        let cert: BoundedVec<u8, ConstU32<128>> = b"TB-12345".to_vec().try_into().unwrap();
        let license: BoundedVec<u8, ConstU32<128>> = b"BL-67890".to_vec().try_into().unwrap();

        assert_ok!(Oracle::verify_merchant(
            RuntimeOrigin::signed(ALICE),
            merchant,
            category.to_u8(),
            cert,
            license,
            None
        ));

        // Get category
        assert_eq!(Oracle::get_merchant_category(&merchant), Some(category));
    });
}

#[test]
fn get_merchant_reward_rate_works() {
    new_test_ext().execute_with(|| {
        let merchant = DAVE;
        let cert: BoundedVec<u8, ConstU32<128>> = b"TB-12345".to_vec().try_into().unwrap();
        let license: BoundedVec<u8, ConstU32<128>> = b"BL-67890".to_vec().try_into().unwrap();

        // Verify hotel (8% reward)
        assert_ok!(Oracle::verify_merchant(
            RuntimeOrigin::signed(ALICE),
            merchant,
            MerchantCategory::Accommodation.to_u8(),
            cert,
            license,
            None
        ));

        // Get reward rate
        assert_eq!(Oracle::get_merchant_reward_rate(&merchant), Some(8));
    });
}

#[test]
fn expired_merchant_returns_none() {
    new_test_ext().execute_with(|| {
        let merchant = DAVE;
        let cert: BoundedVec<u8, ConstU32<128>> = b"TB-12345".to_vec().try_into().unwrap();
        let license: BoundedVec<u8, ConstU32<128>> = b"BL-67890".to_vec().try_into().unwrap();

        assert_ok!(Oracle::verify_merchant(
            RuntimeOrigin::signed(ALICE),
            merchant,
            MerchantCategory::Retail.to_u8(),
            cert,
            license,
            None
        ));

        // Merchant verified at block 1, expires at 1 + 5_256_000 = 5_256_001
        // Advance past expiry (1 year = 5,256,000 blocks)
        System::set_block_number(5_256_002);

        // Should return None (expired)
        assert_eq!(Oracle::get_merchant_category(&merchant), None);
    });
}

// NOTE: Certification validation not implemented yet
// #[test]
// fn invalid_certification_fails() {
//     new_test_ext().execute_with(|| {
//         let merchant = DAVE;
//         let empty_cert: BoundedVec<u8, ConstU32<128>> = vec![].try_into().unwrap();
//         let license: BoundedVec<u8, ConstU32<128>> = b"BL-67890".to_vec().try_into().unwrap();
//         assert_noop!(
//             Oracle::verify_merchant(
//                 RuntimeOrigin::root(),
//                 merchant,
//                 MerchantCategory::Accommodation.to_u8(),
//                 empty_cert,
//                 license,
//                 None
//             ),
//             Error::<Test>::InvalidCertification
//         );
//     });
// }

// ================================
// Sanctions Tests
// ================================

#[test]
fn add_sanction_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let source = SanctionSource::OFAC;
        let reason: BoundedVec<u8, ConstU32<256>> = b"Terrorism financing".to_vec().try_into().unwrap();

        assert_ok!(Oracle::add_sanctioned_entity(
            RuntimeOrigin::root(),
            account,
            source.to_u8(),
            reason.clone(),
            None
        ));

        // Check sanction info
        let info = Oracle::sanctioned_entities(account).unwrap();
        assert_eq!(info.source, source);
        assert_eq!(info.reason, reason);
        assert!(info.active);

        // Check event
        System::assert_last_event(Event::SanctionAdded {
            account,
            source: source.to_u8(),
        }.into());
    });
}

#[test]
fn is_sanctioned_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let reason: BoundedVec<u8, ConstU32<256>> = b"Money laundering".to_vec().try_into().unwrap();

        // Not sanctioned initially
        assert!(!Oracle::is_sanctioned(&account));

        // Add sanction
        assert_ok!(Oracle::add_sanctioned_entity(
            RuntimeOrigin::root(),
            account,
            SanctionSource::UN.to_u8(),
            reason,
            None
        ));

        // Now sanctioned
        assert!(Oracle::is_sanctioned(&account));
    });
}

#[test]
fn remove_sanction_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let reason: BoundedVec<u8, ConstU32<256>> = b"Removed from list".to_vec().try_into().unwrap();

        // Add sanction
        assert_ok!(Oracle::add_sanctioned_entity(
            RuntimeOrigin::root(),
            account,
            SanctionSource::FSC.to_u8(),
            reason,
            None
        ));

        assert!(Oracle::is_sanctioned(&account));

        // Remove sanction
        assert_ok!(Oracle::remove_sanction(
            RuntimeOrigin::root(),
            account
        ));

        assert!(!Oracle::is_sanctioned(&account));

        // Check event
        System::assert_last_event(Event::SanctionRemoved { account }.into());
    });
}

#[test]
fn expired_sanction_not_active() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let reason: BoundedVec<u8, ConstU32<256>> = b"Temporary sanction".to_vec().try_into().unwrap();
        let expiry = 100u64; // Expires at block 100

        // Add sanction with expiry
        assert_ok!(Oracle::add_sanctioned_entity(
            RuntimeOrigin::root(),
            account,
            SanctionSource::EU.to_u8(),
            reason,
            Some(expiry)
        ));

        // Should be sanctioned at block 50
        System::set_block_number(50);
        assert!(Oracle::is_sanctioned(&account));

        // Should NOT be sanctioned at block 101 (expired)
        System::set_block_number(101);
        assert!(!Oracle::is_sanctioned(&account));
    });
}

// ================================
// Integration Tests
// ================================

#[test]
fn full_price_feed_workflow() {
    new_test_ext().execute_with(|| {
        // Setup: Add Charlie as third operator for better consensus
        assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), CHARLIE));

        let pair = CurrencyPair::new(Currency::BZD, Currency::EUR);

        // Three operators submit prices
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(ALICE), pair.base.to_u8(), pair.quote.to_u8(), 450000)); // 0.45 EUR
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(BOB), pair.base.to_u8(), pair.quote.to_u8(), 460000));   // 0.46 EUR
        assert_ok!(Oracle::submit_price(RuntimeOrigin::signed(CHARLIE), pair.base.to_u8(), pair.quote.to_u8(), 455000)); // 0.455 EUR

        // Check aggregated feed (median = 455000)
        let feed = Oracle::price_feeds(pair).unwrap();
        assert_eq!(feed.price, 455000);
        assert_eq!(feed.num_submissions, 3);

        // Check variance calculation
        // max=460000, min=450000, median=455000
        // variance = (460000-450000)*100/455000 = 2.19% (rounded to 219 basis points)
        assert!(feed.variance <= 250); // 2.5% tolerance for test
    });
}

#[test]
fn merchant_verification_and_reward_calculation() {
    new_test_ext().execute_with(|| {
        let hotel = DAVE;
        let cert: BoundedVec<u8, ConstU32<128>> = b"TB-HOTEL-001".to_vec().try_into().unwrap();
        let license: BoundedVec<u8, ConstU32<128>> = b"BL-HOTEL-001".to_vec().try_into().unwrap();

        // Verify hotel merchant (8% reward)
        assert_ok!(Oracle::verify_merchant(
            RuntimeOrigin::signed(ALICE),
            hotel,
            MerchantCategory::Accommodation.to_u8(),
            cert,
            license,
            Some((17_250_000, -88_750_000))
        ));

        // Economy pallet would use this to calculate rewards
        let reward_rate = Oracle::get_merchant_reward_rate(&hotel).unwrap();
        assert_eq!(reward_rate, 8);

        // For a 1000 DALLA payment, reward would be 80 DALLA
        let payment = 1000;
        let reward = (payment * reward_rate as u64) / 100;
        assert_eq!(reward, 80);
    });
}

// ===== IDENTITY VERIFICATION ORACLE TESTS =====

#[test]
fn verify_identity_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let kyc_level = KycLevel::Enhanced;
        let id_hash = [1u8; 32];
        let provider: BoundedVec<u8, ConstU32<64>> = b"Onfido".to_vec().try_into().unwrap();

        // Two oracle votes needed to reach quorum and finalize
        verify_identity_with_quorum(
            account,
            kyc_level.to_u8(),
            id_hash,
            provider,
            true,  // biometric_verified
            true,  // address_verified
        );

        // Check identity is stored
        let identity_info = Oracle::identity_verifications(account).unwrap();
        assert_eq!(identity_info.kyc_level, KycLevel::Enhanced);
        assert!(identity_info.biometric_verified);
        assert!(identity_info.address_verified);
    });
}

#[test]
fn get_kyc_level_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let kyc_level = KycLevel::Basic;
        let id_hash = [2u8; 32];
        let provider: BoundedVec<u8, ConstU32<64>> = b"Jumio".to_vec().try_into().unwrap();

        // Two oracle votes needed to finalize
        verify_identity_with_quorum(
            account,
            kyc_level.to_u8(),
            id_hash,
            provider,
            false,
            true,
        );

        // Check helper function
        assert_eq!(Oracle::get_kyc_level(&account), Some(KycLevel::Basic));
    });
}

#[test]
fn kyc_transaction_limits_work() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let kyc_level = KycLevel::Enhanced;
        let id_hash = [3u8; 32];
        let provider: BoundedVec<u8, ConstU32<64>> = b"Gov-Belize".to_vec().try_into().unwrap();

        // Two oracle votes needed to finalize
        verify_identity_with_quorum(
            account,
            kyc_level.to_u8(),
            id_hash,
            provider,
            true,
            true,
        );

        // Check transaction limits
        let tx_limit = Oracle::get_transaction_limit(&account);
        assert_eq!(tx_limit, 100_000 * 1_000_000_000_000); // 100,000 DALLA

        let daily_limit = Oracle::get_daily_limit(&account);
        assert_eq!(daily_limit, 250_000 * 1_000_000_000_000); // 250,000 DALLA/day
    });
}

#[test]
fn meets_kyc_requirement_works() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let kyc_level = KycLevel::Enhanced;
        let id_hash = [4u8; 32];
        let provider: BoundedVec<u8, ConstU32<64>> = b"Onfido".to_vec().try_into().unwrap();

        // Two oracle votes needed to finalize
        verify_identity_with_quorum(
            account,
            kyc_level.to_u8(),
            id_hash,
            provider,
            true,
            true,
        );

        // Should meet Basic and Enhanced requirements
        assert!(Oracle::meets_kyc_requirement(&account, KycLevel::None));
        assert!(Oracle::meets_kyc_requirement(&account, KycLevel::Basic));
        assert!(Oracle::meets_kyc_requirement(&account, KycLevel::Enhanced));
        
        // Should NOT meet Full requirement
        assert!(!Oracle::meets_kyc_requirement(&account, KycLevel::Full));
    });
}

#[test]
fn expired_identity_returns_none() {
    new_test_ext().execute_with(|| {
        let account = DAVE;
        let kyc_level = KycLevel::Basic;
        let id_hash = [5u8; 32];
        let provider: BoundedVec<u8, ConstU32<64>> = b"Jumio".to_vec().try_into().unwrap();

        // Verify identity
        assert_ok!(Oracle::verify_identity(
            RuntimeOrigin::signed(ALICE),
            account,
            kyc_level.to_u8(),
            id_hash,
            provider,
            false,
            true,
        ));

        // Identity verified at block 1, expires at 1 + 5_256_000 = 5_256_001
        // Advance past expiry
        System::set_block_number(5_256_002);

        // Should return None (expired)
        assert_eq!(Oracle::get_kyc_level(&account), None);
    });
}

// ===== LAND REGISTRY ORACLE TESTS =====

#[test]
fn register_land_works() {
    new_test_ext().execute_with(|| {
        let property_id: PropertyId = [1u8; 32];
        let owner = DAVE;
        let valuation = 500_000 * 1_000_000; // 500,000 bBZD

        assert_ok!(Oracle::register_land(
            RuntimeOrigin::signed(ALICE),
            property_id,
            owner,
            valuation,
            false, // no encumbrances
            0,     // no co-owners
        ));

        // Check land registry
        let land_info = Oracle::land_registry_data(property_id).unwrap();
        assert_eq!(land_info.owner, owner);
        assert_eq!(land_info.valuation, valuation);
        assert!(!land_info.has_encumbrances);
        assert!(land_info.verified);
    });
}

#[test]
fn verify_land_owner_works() {
    new_test_ext().execute_with(|| {
        let property_id: PropertyId = [2u8; 32];
        let owner = DAVE;
        let valuation = 1_000_000 * 1_000_000; // 1M bBZD

        // Register property
        assert_ok!(Oracle::register_land(
            RuntimeOrigin::signed(ALICE),
            property_id,
            owner,
            valuation,
            true, // has encumbrances
            2,    // 2 co-owners
        ));

        // Verify ownership
        assert!(Oracle::verify_land_owner(property_id, &owner));
        assert!(!Oracle::verify_land_owner(property_id, &BOB));
    });
}

#[test]
fn get_land_ownership_works() {
    new_test_ext().execute_with(|| {
        let property_id: PropertyId = [3u8; 32];
        let owner = CHARLIE;
        let valuation = 250_000 * 1_000_000;

        // Register property
        assert_ok!(Oracle::register_land(
            RuntimeOrigin::signed(ALICE),
            property_id,
            owner,
            valuation,
            false,
            1, // 1 co-owner
        ));

        // Get land ownership info
        let land_info = Oracle::get_land_ownership(property_id).unwrap();
        assert_eq!(land_info.owner, owner);
        assert_eq!(land_info.co_owner_count, 1);
        assert!(!land_info.has_encumbrances);
    });
}

#[test]
fn land_registry_with_encumbrances() {
    new_test_ext().execute_with(|| {
        let property_id: PropertyId = [4u8; 32];
        let owner = BOB;
        let valuation = 750_000 * 1_000_000;

        // Register property with mortgage
        assert_ok!(Oracle::register_land(
            RuntimeOrigin::signed(ALICE),
            property_id,
            owner,
            valuation,
            true, // has mortgage encumbrance
            0,
        ));

        let land_info = Oracle::land_registry_data(property_id).unwrap();
        assert!(land_info.has_encumbrances);
        
        // Economy pallet could check this before allowing certain transactions
        if land_info.has_encumbrances {
            // Additional verification required
        }
    });
}

// ============================================================================
// EDGE CASE TESTS - Boundary Validation (3 tests)
// ============================================================================

#[test]
fn duplicate_operator_rejected() {
    new_test_ext().execute_with(|| {
        // ALICE is already an operator from genesis
        assert!(Oracle::oracle_operators(ALICE));
        
        assert_noop!(
            Oracle::add_operator(RuntimeOrigin::root(), ALICE),
            Error::<Test>::OperatorAlreadyExists
        );
    });
}

#[test]
fn remove_nonexistent_operator_edge_case() {
    new_test_ext().execute_with(|| {
        // CHARLIE was never added as operator
        assert!(!Oracle::oracle_operators(CHARLIE));
        
        assert_noop!(
            Oracle::remove_operator(RuntimeOrigin::root(), CHARLIE),
            Error::<Test>::OperatorNotFound
        );
    });
}

#[test]
fn land_valuation_zero_allowed() {
    new_test_ext().execute_with(|| {
        let property_id: PropertyId = [250u8; 32];
        let owner = DAVE;
        
        // Zero valuation (unassessed land)
        assert_ok!(Oracle::register_land(
            RuntimeOrigin::signed(ALICE),
            property_id,
            owner,
            0, // Zero valuation
            false,
            0
        ));
        
        let land_info = Oracle::land_registry_data(property_id).unwrap();
        assert_eq!(land_info.valuation, 0);
        assert_eq!(land_info.owner, owner);
    });
}

// ================================
// IoT Device Tests
// ================================

#[test]
fn register_iot_device_works() {
    new_test_ext().execute_with(|| {
        let device_id = [1u8; 32];
        assert_ok!(Oracle::register_iot_device(
            RuntimeOrigin::signed(ALICE),
            device_id,
            2, // IoTSensor
            Some((174500, -176000)), // Belmopan coords (scaled)
        ));
        let device = Oracle::get_iot_device(device_id).unwrap();
        assert_eq!(device.owner, ALICE);
        assert!(!device.verified);
        assert_eq!(device.data_submissions, 0);
    });
}

#[test]
fn register_iot_device_duplicate_fails() {
    new_test_ext().execute_with(|| {
        let device_id = [2u8; 32];
        assert_ok!(Oracle::register_iot_device(RuntimeOrigin::signed(ALICE), device_id, 0, None));
        assert_noop!(
            Oracle::register_iot_device(RuntimeOrigin::signed(BOB), device_id, 0, None),
            Error::<Test>::DeviceAlreadyRegistered
        );
    });
}

#[test]
fn submit_iot_data_works() {
    new_test_ext().execute_with(|| {
        let device_id = [3u8; 32];
        // Register the device
        assert_ok!(Oracle::register_iot_device(RuntimeOrigin::signed(ALICE), device_id, 2, None));

        let data: BoundedVec<u8, ConstU32<256>> = vec![1u8; 32].try_into().unwrap();
        let data_hash = [0xABu8; 32];

        assert_ok!(Oracle::submit_iot_data(
            RuntimeOrigin::signed(ALICE),
            device_id,
            0,    // feed_type_index: IotTemperature or similar
            None, // no domain
            data,
            data_hash,
            None, // no location override
            90,   // accuracy
            85,   // timeliness
            95,   // completeness
            88,   // consistency
            80,   // provenance
        ));

        let device = Oracle::get_iot_device(device_id).unwrap();
        assert_eq!(device.data_submissions, 1);

        // Stats should exist for ALICE after submission
        let stats = Oracle::get_operator_stats(&ALICE);
        assert!(stats.is_some());
    });
}

#[test]
fn submit_iot_data_wrong_owner_fails() {
    new_test_ext().execute_with(|| {
        let device_id = [4u8; 32];
        assert_ok!(Oracle::register_iot_device(RuntimeOrigin::signed(ALICE), device_id, 2, None));

        let data: BoundedVec<u8, ConstU32<256>> = vec![1u8; 8].try_into().unwrap();
        assert_noop!(
            Oracle::submit_iot_data(
                RuntimeOrigin::signed(BOB), // BOB is not ALICE's device owner
                device_id,
                0, None, data, [0u8; 32], None,
                90, 85, 95, 88, 80,
            ),
            Error::<Test>::NotDeviceOwner
        );
    });
}

#[test]
fn verify_iot_device_works() {
    new_test_ext().execute_with(|| {
        let device_id = [5u8; 32];
        assert_ok!(Oracle::register_iot_device(RuntimeOrigin::signed(CHARLIE), device_id, 3, None));
        assert!(!Oracle::is_device_verified(device_id));

        // ALICE is an oracle operator (from genesis) and can verify
        assert_ok!(Oracle::verify_iot_device(RuntimeOrigin::signed(ALICE), device_id));
        assert!(Oracle::is_device_verified(device_id));
    });
}

#[test]
fn verify_iot_device_non_operator_fails() {
    new_test_ext().execute_with(|| {
        let device_id = [6u8; 32];
        assert_ok!(Oracle::register_iot_device(RuntimeOrigin::signed(CHARLIE), device_id, 3, None));
        // CHARLIE is not an operator
        assert_noop!(
            Oracle::verify_iot_device(RuntimeOrigin::signed(CHARLIE), device_id),
            Error::<Test>::NotAuthorizedOperator
        );
    });
}

#[test]
fn claim_oracle_rewards_no_stats_fails() {
    new_test_ext().execute_with(|| {
        // CHARLIE has never submitted oracle data
        assert_noop!(
            Oracle::claim_oracle_rewards(RuntimeOrigin::signed(CHARLIE)),
            Error::<Test>::NoStatsFound
        );
    });
}

#[test]
fn claim_oracle_rewards_works() {
    new_test_ext().execute_with(|| {
        // Directly populate stats for ALICE with enough submissions to earn rewards.
        //
        // Reward formula: base * volume_factor * quality_factor * domain_multiplier * uptime_factor / 10000
        // With total=2000, agritech=2000, quality=1000, uptime=10000:
        //   volume=2, quality=10, domain=300000/(2000*100)=1, uptime=10
        //   reward = 100_000_000_000_000 * 2 * 10 * 1 * 10 / 10000 = 2_000_000_000_000
        crate::OracleOperatorStatsMap::<Test>::insert(ALICE, crate::types::OracleOperatorStats {
            total_submissions:    2_000,
            avg_quality_score:    1_000,
            agritech_submissions: 2_000,
            marine_submissions:   0,
            education_submissions:0,
            tech_submissions:     0,
            general_submissions:  0,
            uptime_percentage:    10_000,
            last_active:          1u64,
            total_rewards:        0,
        });

        // Fund treasury (account 999) so the transfer succeeds.
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            999u64,
            3_000_000_000_000u64,
        ));

        let alice_before = Balances::free_balance(ALICE);
        assert_ok!(Oracle::claim_oracle_rewards(RuntimeOrigin::signed(ALICE)));

        // Balance should have increased by the reward amount
        assert!(Balances::free_balance(ALICE) > alice_before);
    });
}
