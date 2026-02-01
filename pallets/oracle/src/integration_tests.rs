/// Oracle Integration Test - Cross-Pallet Usage Examples
///
/// This file demonstrates how other pallets can use the Oracle pallet's
/// data feeds and verification services.

#[cfg(test)]
mod oracle_integration_tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};
    use sp_runtime::DispatchError;

    // Example 1: Economy Pallet Using Oracle Exchange Rate
    #[test]
    fn economy_can_get_exchange_rate_from_oracle() {
        new_test_ext().execute_with(|| {
            // Setup: Add oracle operators
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(1)));
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(2)));

            // Operators submit BZD/BZD exchange rate (1:1 peg)
            assert_ok!(Oracle::submit_price(
                RuntimeOrigin::signed(account(1)),
                0, // BZD
                1, // BZD
                100, // 1.00 with 2 decimals (1:1 peg)
            ));
            
            assert_ok!(Oracle::submit_price(
                RuntimeOrigin::signed(account(2)),
                0, // BZD
                1, // BZD  
                100, // 1.00 with 2 decimals (1:1 peg)
            ));

            // Economy pallet can now use the exchange rate
            let exchange_rate = Oracle::get_exchange_rate();
            assert_eq!(exchange_rate, Some(100));

            // Example: Calculate bBZD issuance amount (BZD-pegged 1:1)
            let dalla_amount = 1000u128; // 1000 DALLA
            let bbzd_amount = dalla_amount * exchange_rate.unwrap() / 100;
            assert_eq!(bbzd_amount, 1000); // 1000 bBZD (1:1 BZD/BZD peg)
        });
    }

    // Example 2: Identity Pallet Using KYC Verification
    #[test]
    fn identity_can_check_kyc_limits() {
        new_test_ext().execute_with(|| {
            // Setup: Add oracle operator
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(1)));

            let user = account(100);
            
            // User starts with no KYC
            let kyc_level = Oracle::get_kyc_level(&user);
            assert_eq!(kyc_level, None);
            
            // Default transaction limit for unverified
            let limit = Oracle::get_transaction_limit(&user);
            assert_eq!(limit, 1_000_000_000); // 1,000 DALLA (12 decimals)

            // Operator verifies user with Basic KYC
            assert_ok!(Oracle::verify_identity(
                RuntimeOrigin::signed(account(1)),
                user.clone(),
                1, // Basic KYC
                [1u8; 32], // ID hash
                bounded_vec![1, 2, 3], // Provider
                false, // No biometric
                None, // No address verification
            ));

            // Identity pallet can now check upgraded limit
            let new_limit = Oracle::get_transaction_limit(&user);
            assert_eq!(new_limit, 10_000_000_000); // 10,000 DALLA

            // Example: Validate transaction amount
            let transaction_amount = 5_000_000_000; // 5,000 DALLA
            assert!(transaction_amount <= new_limit);
        });
    }

    // Example 3: Community Pallet Checking Sanctions
    #[test]
    fn community_can_check_sanctions_before_payout() {
        new_test_ext().execute_with(|| {
            // Setup: Add oracle operator
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(1)));

            let bad_actor = account(666);
            let good_citizen = account(100);

            // Community pallet checks sanctions before treasury disbursement
            assert!(!Oracle::is_sanctioned(&good_citizen));
            
            // Operator adds sanction
            assert_ok!(Oracle::add_sanction(
                RuntimeOrigin::signed(account(1)),
                bad_actor.clone(),
                bounded_vec![b'O', b'F', b'A', b'C'], // OFAC
                None, // Permanent
            ));

            // Community pallet can now detect sanctioned account
            assert!(Oracle::is_sanctioned(&bad_actor));

            // Example: Reject payout to sanctioned account
            // if Oracle::is_sanctioned(&recipient) {
            //     return Err(Error::<T>::RecipientSanctioned.into());
            // }
        });
    }

    // Example 4: Land Ledger Using Ownership Verification
    #[test]
    fn land_ledger_can_verify_property_ownership() {
        new_test_ext().execute_with(|| {
            // Setup: Add oracle operator
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(1)));

            let property_id = [1u8; 32];
            let owner = account(100);

            // Operator registers property
            assert_ok!(Oracle::register_land(
                RuntimeOrigin::signed(account(1)),
                property_id,
                owner.clone(),
                500_000_000_000, // 500,000 BZD valuation
                false, // No encumbrances
                None, // No co-owners
            ));

            // Land Ledger can verify ownership before tokenization
            assert!(Oracle::verify_land_owner(&property_id, &owner));
            
            let land_info = Oracle::get_land_info(&property_id).unwrap();
            assert_eq!(land_info.owner, owner);
            assert_eq!(land_info.valuation, 500_000_000_000);
            
            // Check for liens before allowing tokenization
            assert!(!Oracle::has_encumbrances(&property_id));

            // Example: Allow property tokenization
            // if !Oracle::verify_land_owner(&property_id, &claimant) {
            //     return Err(Error::<T>::NotPropertyOwner.into());
            // }
            // if Oracle::has_encumbrances(&property_id) {
            //     return Err(Error::<T>::PropertyHasLiens.into());
            // }
        });
    }

    // Example 5: Tourism Merchant Verification
    #[test]
    fn economy_can_verify_tourism_merchants() {
        new_test_ext().execute_with(|| {
            // Setup: Add oracle operator
            assert_ok!(Oracle::add_operator(RuntimeOrigin::root(), account(1)));

            let hotel = account(200);
            let restaurant = account(201);
            let tour_operator = account(202);

            // Operators verify different merchant categories
            assert_ok!(Oracle::verify_merchant(
                RuntimeOrigin::signed(account(1)),
                hotel.clone(),
                1, // Accommodation
                100, // 100 blocks validity
            ));

            assert_ok!(Oracle::verify_merchant(
                RuntimeOrigin::signed(account(1)),
                restaurant.clone(),
                2, // Food & Beverage
                100,
            ));

            assert_ok!(Oracle::verify_merchant(
                RuntimeOrigin::signed(account(1)),
                tour_operator.clone(),
                3, // Tours & Activities
                100,
            ));

            // Economy pallet can check merchant categories
            assert!(Oracle::is_merchant_verified(&hotel, 1));
            assert!(Oracle::is_merchant_verified(&restaurant, 2));
            assert!(Oracle::is_merchant_verified(&tour_operator, 3));

            // Example: Calculate tourism incentives by category
            let base_amount = 1000u128;
            let hotel_incentive = base_amount * 8 / 100; // 8% for accommodation
            let restaurant_incentive = base_amount * 5 / 100; // 5% for F&B
            let tour_incentive = base_amount * 7 / 100; // 7% for tours

            assert_eq!(hotel_incentive, 80);
            assert_eq!(restaurant_incentive, 50);
            assert_eq!(tour_incentive, 70);
        });
    }

    // Helper functions
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .unwrap()
            .into()
    }

    fn account(id: u64) -> AccountId {
        [id as u8; 32].into()
    }

    fn bounded_vec<T: Clone>(data: Vec<T>) -> BoundedVec<T, ConstU32<64>> {
        BoundedVec::try_from(data).unwrap()
    }
}
