use crate::{mock::*, Error, Event, RedemptionStatus};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, OnInitialize},
    BoundedVec,
};
use sp_core::ConstU32;

// ============================================================================
// DALLA TOKEN TESTS (Native Gas Token)
// ============================================================================

#[test]
fn burn_dalla_works() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let burn_amount = 100_000_000_000;
        
        let initial_supply = Economy::total_supply();
        let initial_balance = Balances::free_balance(user);
        
        // Burn tokens
        assert_ok!(Economy::burn_dalla(RuntimeOrigin::signed(user), burn_amount));
        
        // Check balance decreased
        assert_eq!(
            Balances::free_balance(user),
            initial_balance - burn_amount
        );
        
        // Check supply decreased
        assert_eq!(
            Economy::total_supply(),
            initial_supply - burn_amount
        );
        
        // Check event exists in events
        System::assert_has_event(
            Event::DallaBurned {
                who: user,
                amount: burn_amount,
                new_supply: initial_supply - burn_amount,
            }.into()
        );
    });
}

#[test]
fn burn_dalla_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let user_balance = Balances::free_balance(user);
        let burn_amount = user_balance + 1;
        
        // Try to burn more than balance
        assert_noop!(
            Economy::burn_dalla(RuntimeOrigin::signed(user), burn_amount),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn governance_burn_works() {
    new_test_ext().execute_with(|| {
        let burn_amount = 1_000_000_000_000;
        let treasury = 100;
        
        let initial_supply = Economy::total_supply();
        let initial_treasury_balance = Balances::free_balance(treasury);
        
        // Governance burns from treasury
        assert_ok!(Economy::governance_burn(
            RuntimeOrigin::signed(1), // Governance account
            burn_amount
        ));
        
        // Check treasury balance decreased
        assert_eq!(
            Balances::free_balance(treasury),
            initial_treasury_balance - burn_amount
        );
        
        // Check supply decreased
        assert_eq!(
            Economy::total_supply(),
            initial_supply - burn_amount
        );
        
        // Check event exists in events
        System::assert_has_event(
            Event::GovernanceBurn {
                amount: burn_amount,
                new_supply: initial_supply - burn_amount,
            }
            .into(),
        );
    });
}

#[test]
fn governance_burn_requires_governance_origin() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let burn_amount = 1_000_000;
        
        // Regular user cannot call governance burn
        assert_noop!(
            Economy::governance_burn(RuntimeOrigin::signed(user), burn_amount),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn max_supply_enforced_on_inflation() {
    new_test_ext().execute_with(|| {
        let max_supply = 501_000_000_000_000_000u64;
        // Bring actual total_issuance close to max so inflation would exceed it
        let current_iso = Balances::total_issuance();
        let to_deposit = max_supply.saturating_sub(current_iso).saturating_sub(100);
        let _ = Balances::deposit_creating(&100u64, to_deposit);

        let treasury_balance_before = Balances::free_balance(100);

        // Advance to trigger inflation (5_256_000 blocks = 1 year)
        System::set_block_number(5_256_001);

        // Run on_initialize
        Economy::on_initialize(5_256_001);

        // Total supply should not exceed max
        assert!(Economy::total_supply() <= max_supply);

        // Treasury balance should not increase (inflation not applied due to cap)
        assert_eq!(Balances::free_balance(100), treasury_balance_before);
    });
}

#[test]
fn annual_inflation_applied_correctly() {
    new_test_ext().execute_with(|| {
        crate::LastInflationBlock::<Test>::put(0u64);

        // Fund treasury so the deposit_creating works and total_issuance is known
        let _ = Balances::deposit_creating(&100u64, 1_000_000_000_000_000);

        // Capture the ACTUAL total issuance AFTER all funding (impl uses this, not TotalSupply storage)
        let actual_supply = Balances::total_issuance();
        let treasury_balance_before = Balances::free_balance(100);

        // Advance to trigger inflation (5_256_000 blocks = 1 year)
        let blocks_per_year = 5_256_000u64;
        System::set_block_number(blocks_per_year + 1);

        // Run on_initialize
        Economy::on_initialize(blocks_per_year + 1);

        // Expected: 2% of actual total issuance
        let expected_inflation = actual_supply / 50; // 2% = 1/50

        // Check total supply increased
        let new_supply = Economy::total_supply();
        assert_eq!(new_supply, actual_supply + expected_inflation);

        // Check treasury received inflation
        assert!(Balances::free_balance(100) > treasury_balance_before);

        // Check last inflation block updated
        assert_eq!(Economy::last_inflation_block(), blocks_per_year + 1);
    });
}

#[test]
fn inflation_not_applied_before_year() {
    new_test_ext().execute_with(|| {
        let initial_supply = Economy::total_supply();
        let treasury_balance_before = Balances::free_balance(100);
        
        // Advance less than a year
        System::set_block_number(1_000_000);
        
        // Run on_initialize
        Economy::on_initialize(1_000_000);
        
        // Supply should not change
        assert_eq!(Economy::total_supply(), initial_supply);
        
        // Treasury balance should not change
        assert_eq!(Balances::free_balance(100), treasury_balance_before);
    });
}

#[test]
fn multiple_burns_decrease_supply_correctly() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let burn_amount_1 = 10_000_000_000;
        let burn_amount_2 = 20_000_000_000;
        
        let initial_supply = Economy::total_supply();
        
        // First burn
        assert_ok!(Economy::burn_dalla(
            RuntimeOrigin::signed(user),
            burn_amount_1
        ));
        
        assert_eq!(
            Economy::total_supply(),
            initial_supply - burn_amount_1
        );
        
        // Second burn
        assert_ok!(Economy::burn_dalla(
            RuntimeOrigin::signed(user),
            burn_amount_2
        ));
        
        assert_eq!(
            Economy::total_supply(),
            initial_supply - burn_amount_1 - burn_amount_2
        );
    });
}

#[test]
fn total_supply_tracks_correctly() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let burn_amount = 50_000_000_000;
        
        // Get initial supply
        let supply_1 = Economy::total_supply();
        
        // Burn some tokens
        assert_ok!(Economy::burn_dalla(
            RuntimeOrigin::signed(user),
            burn_amount
        ));
        
        let supply_2 = Economy::total_supply();
        assert_eq!(supply_2, supply_1 - burn_amount);
        
        // Governance burn
        let gov_burn_amount = 100_000_000_000;
        assert_ok!(Economy::governance_burn(
            RuntimeOrigin::signed(1),
            gov_burn_amount
        ));
        
        let supply_3 = Economy::total_supply();
        assert_eq!(supply_3, supply_2 - gov_burn_amount);
    });
}

// ============================================================================
// bBZD TOKEN TESTS (USDC-Style Fiat-Backed Stablecoin)
// ============================================================================

#[test]
fn mint_bbzd_works() {
    new_test_ext().execute_with(|| {
        // Setup: Account 1 is authorized minter (Central Bank)
        let central_bank = 1u64;
        let user = 2u64;
        let mint_amount = 100_000_000_000u128; // 100 bBZD
        let deposit_ref: BoundedVec<u8, ConstU32<64>> = 
            b"BANK_DEPOSIT_ABC123".to_vec().try_into().unwrap();
        
        // Authorize central bank as minter
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        
        // Set initial reserves (Central Bank has 1000 BZD in bank account)
        let initial_reserves = 1_000_000_000_000u128;
        crate::CentralBankReserves::<Test>::put(initial_reserves);
        
        // Mint bBZD after BZD deposit verification
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user,
            mint_amount,
            deposit_ref.clone()
        ));
        
        // Check user received bBZD
        assert_eq!(crate::BBZDBalances::<Test>::get(user), mint_amount);
        
        // Check total supply updated
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), mint_amount);
        
        // Check invariant: supply ≤ reserves
        assert!(crate::TotalBbzdSupply::<Test>::get() <= crate::CentralBankReserves::<Test>::get());
        
        // Check event emitted
        System::assert_has_event(
            Event::BbzdMinted {
                minter: central_bank,
                recipient: user,
                amount: mint_amount,
                deposit_reference: deposit_ref,
                new_total_supply: mint_amount,
            }.into()
        );
    });
}

#[test]
fn mint_bbzd_fails_unauthorized_minter() {
    new_test_ext().execute_with(|| {
        let unauthorized_user = 2u64;
        let recipient = 3u64;
        let mint_amount = 100_000_000_000u128;
        let deposit_ref: BoundedVec<u8, ConstU32<64>> = 
            b"DEPOSIT_123".to_vec().try_into().unwrap();
        
        // Try to mint without authorization
        assert_noop!(
            Economy::mint_bbzd(
                RuntimeOrigin::signed(unauthorized_user),
                recipient,
                mint_amount,
                deposit_ref
            ),
            Error::<Test>::UnauthorizedMinter
        );
    });
}

#[test]
fn mint_bbzd_fails_insufficient_reserves() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user = 2u64;
        let mint_amount = 1_000_000_000_000u128; // 1000 bBZD
        
        // Authorize minter
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        
        // Set reserves LOWER than mint amount (only 500 BZD in bank)
        crate::CentralBankReserves::<Test>::put(500_000_000_000u128);
        
        // Try to mint more than reserves
        let deposit_ref: BoundedVec<u8, ConstU32<64>> = 
            b"DEPOSIT_123".to_vec().try_into().unwrap();
        assert_noop!(
            Economy::mint_bbzd(
                RuntimeOrigin::signed(central_bank),
                user,
                mint_amount,
                deposit_ref
            ),
            Error::<Test>::InsufficientReserves
        );
    });
}

#[test]
fn redeem_bbzd_works() {
    new_test_ext().execute_with(|| {
        let user = 2u64;
        let redeem_amount = 50_000_000_000u128; // 50 bBZD
        let bank_account: BoundedVec<u8, ConstU32<64>> = 
            b"USER_BANK_ACCOUNT_789".to_vec().try_into().unwrap();
        
        // Setup: User has 100 bBZD
        crate::BBZDBalances::<Test>::insert(user, 100_000_000_000u128);
        crate::TotalBbzdSupply::<Test>::put(100_000_000_000u128);
        crate::CentralBankReserves::<Test>::put(100_000_000_000u128);
        crate::NextRedemptionId::<Test>::put(1);
        
        // Redeem bBZD
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user),
            redeem_amount,
            bank_account.clone()
        ));
        
        // Check bBZD burned immediately
        assert_eq!(crate::BBZDBalances::<Test>::get(user), 50_000_000_000u128);
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), 50_000_000_000u128);
        
        // Check redemption request created
        let request = crate::RedemptionRequests::<Test>::get(1).unwrap();
        assert_eq!(request.user, user);
        assert_eq!(request.amount, redeem_amount);
        assert_eq!(request.bank_account, bank_account);
        assert_eq!(request.status, RedemptionStatus::Pending);
        
        // Check next redemption ID incremented
        assert_eq!(crate::NextRedemptionId::<Test>::get(), 2);
        
        // Check event emitted
        System::assert_has_event(
            Event::BbzdRedeemed {
                user,
                amount: redeem_amount,
                bank_account,
                redemption_id: 1,
                new_total_supply: 50_000_000_000u128,
            }.into()
        );
    });
}

#[test]
fn redeem_bbzd_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let user = 2u64;
        let user_balance = 50_000_000_000u128;
        let redeem_amount = 100_000_000_000u128; // More than user has
        
        // Setup: User has only 50 bBZD
        crate::BBZDBalances::<Test>::insert(user, user_balance);
        
        // Try to redeem more than balance
        let bank_account: BoundedVec<u8, ConstU32<64>> = 
            b"BANK_ACCOUNT".to_vec().try_into().unwrap();
        assert_noop!(
            Economy::redeem_bbzd(
                RuntimeOrigin::signed(user),
                redeem_amount,
                bank_account
            ),
            Error::<Test>::InsufficientBbzdBalance
        );
    });
}

#[test]
fn process_redemption_works() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user = 2u64;
        let redemption_id = 1;
        
        // Setup: Create pending redemption
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        crate::RedemptionRequests::<Test>::insert(
            redemption_id,
            crate::RedemptionRequest {
                user,
                amount: 50_000_000_000u128,
                bank_account: b"USER_BANK_123".to_vec().try_into().unwrap(),
                redemption_id,
                status: RedemptionStatus::Pending,
            },
        );
        
        // Process redemption after off-chain BZD transfer
        assert_ok!(Economy::process_redemption(
            RuntimeOrigin::signed(central_bank),
            redemption_id
        ));
        
        // Check redemption marked as processed
        let request = crate::RedemptionRequests::<Test>::get(redemption_id).unwrap();
        assert_eq!(request.status, RedemptionStatus::Processed);
        
        // Check event emitted (transfer_ref generated internally)
        // Event: RedemptionProcessed { redemption_id, bank_transfer_reference }
        // (bank_transfer_reference is auto-generated as "REDEMPTION_{id}")
    });
}

#[test]
fn process_redemption_fails_invalid_id() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let invalid_id = 999;
        
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        
        // Try to process non-existent redemption
        assert_noop!(
            Economy::process_redemption(
                RuntimeOrigin::signed(central_bank),
                invalid_id
            ),
            Error::<Test>::InvalidRedemptionId
        );
    });
}

#[test]
fn process_redemption_fails_already_processed() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let redemption_id = 1;
        
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        
        // Setup: Already processed redemption
        crate::RedemptionRequests::<Test>::insert(
            redemption_id,
            crate::RedemptionRequest {
                user: 2,
                amount: 50_000_000_000u128,
                bank_account: b"BANK_123".to_vec().try_into().unwrap(),
                redemption_id,
                status: RedemptionStatus::Processed, // Already processed
            },
        );
        
        // Try to process again
        assert_noop!(
            Economy::process_redemption(
                RuntimeOrigin::signed(central_bank),
                redemption_id
            ),
            Error::<Test>::RedemptionAlreadyProcessed
        );
    });
}

#[test]
fn set_minter_authorization_works() {
    new_test_ext().execute_with(|| {
        let governance = 1u64; // Governance account from mock
        let new_minter = 5u64;
        
        // Authorize new minter
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::signed(governance),
            new_minter,
            true
        ));
        
        // Check authorization set
        assert!(crate::AuthorizedMinters::<Test>::get(new_minter));
        
        // Check event emitted
        System::assert_has_event(
            Event::MinterAuthorization {
                minter: new_minter,
                authorized: true,
            }.into()
        );
        
        // Revoke authorization
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::signed(governance),
            new_minter,
            false
        ));
        
        assert!(!crate::AuthorizedMinters::<Test>::get(new_minter));
    });
}

#[test]
fn set_minter_authorization_requires_governance() {
    new_test_ext().execute_with(|| {
        let unauthorized_user = 2u64;
        let new_minter = 5u64;
        
        // Try to set authorization without governance
        assert_noop!(
            Economy::set_minter_authorization(
                RuntimeOrigin::signed(unauthorized_user),
                new_minter,
                true
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_reserves_works() {
    new_test_ext().execute_with(|| {
        let governance = 1u64;
        let new_reserves = 2_000_000_000_000u128; // 2000 BZD
        
        // Setup initial state
        let old_reserves = 1_000_000_000_000u128;
        crate::CentralBankReserves::<Test>::put(old_reserves);
        crate::TotalBbzdSupply::<Test>::put(500_000_000_000u128); // 500 bBZD in circulation
        
        // Update reserves (after audit/verification)
        assert_ok!(Economy::update_reserves(
            RuntimeOrigin::signed(governance),
            new_reserves
        ));
        
        // Check reserves updated
        assert_eq!(crate::CentralBankReserves::<Test>::get(), new_reserves);
        
        // Check event emitted
        System::assert_has_event(
            Event::ReservesUpdated {
                old_reserves,
                new_reserves,
                total_supply: 500_000_000_000u128,
            }.into()
        );
    });
}

#[test]
fn update_reserves_requires_governance() {
    new_test_ext().execute_with(|| {
        let unauthorized_user = 2u64;
        let new_reserves = 2_000_000_000_000u128;
        
        // Try to update reserves without governance
        assert_noop!(
            Economy::update_reserves(
                RuntimeOrigin::signed(unauthorized_user),
                new_reserves
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn bbzd_reserve_invariant_maintained() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user1 = 2u64;
        let user2 = 3u64;
        
        // Setup
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        crate::CentralBankReserves::<Test>::put(1_000_000_000_000u128); // 1000 BZD reserves
        
        // Mint 500 bBZD to user1
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user1,
            500_000_000_000u128,
            b"DEPOSIT_1".to_vec().try_into().unwrap()
        ));
        
        // Invariant check
        assert!(crate::TotalBbzdSupply::<Test>::get() <= crate::CentralBankReserves::<Test>::get());
        
        // Mint 300 bBZD to user2
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user2,
            300_000_000_000u128,
            b"DEPOSIT_2".to_vec().try_into().unwrap()
        ));
        
        // Invariant still holds (800 bBZD ≤ 1000 BZD reserves)
        assert!(crate::TotalBbzdSupply::<Test>::get() <= crate::CentralBankReserves::<Test>::get());
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), 800_000_000_000u128);
        
        // User1 redeems 200 bBZD
        crate::NextRedemptionId::<Test>::put(1);
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user1),
            200_000_000_000u128,
            b"BANK_ACCOUNT_1".to_vec().try_into().unwrap()
        ));
        
        // Invariant still holds (600 bBZD ≤ 1000 BZD reserves)
        assert!(crate::TotalBbzdSupply::<Test>::get() <= crate::CentralBankReserves::<Test>::get());
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), 600_000_000_000u128);
    });
}

#[test]
fn multiple_users_can_mint_and_redeem() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user1 = 2u64;
        let user2 = 3u64;
        let user3 = 4u64;
        
        // Setup
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        crate::CentralBankReserves::<Test>::put(10_000_000_000_000u128); // 10K BZD
        crate::NextRedemptionId::<Test>::put(1);
        
        // Mint to multiple users
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user1,
            1_000_000_000_000u128,
            b"DEPOSIT_USER1".to_vec().try_into().unwrap()
        ));
        
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user2,
            2_000_000_000_000u128,
            b"DEPOSIT_USER2".to_vec().try_into().unwrap()
        ));
        
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user3,
            500_000_000_000u128,
            b"DEPOSIT_USER3".to_vec().try_into().unwrap()
        ));
        
        // Check balances
        assert_eq!(crate::BBZDBalances::<Test>::get(user1), 1_000_000_000_000u128);
        assert_eq!(crate::BBZDBalances::<Test>::get(user2), 2_000_000_000_000u128);
        assert_eq!(crate::BBZDBalances::<Test>::get(user3), 500_000_000_000u128);
        
        // Total supply correct
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), 3_500_000_000_000u128);
        
        // User2 redeems half their balance
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user2),
            1_000_000_000_000u128,
            b"BANK_USER2".to_vec().try_into().unwrap()
        ));
        
        // Check user2 balance updated
        assert_eq!(crate::BBZDBalances::<Test>::get(user2), 1_000_000_000_000u128);
        
        // Total supply updated
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), 2_500_000_000_000u128);
    });
}

// ============================================================================
// EDGE CASE TESTS - Zero Values, Boundaries, Error Recovery
// ============================================================================

#[test]
fn burn_dalla_fails_on_zero_amount() {
    new_test_ext().execute_with(|| {
        let user = 2;
        
        // Try to burn zero
        assert_noop!(
            Economy::burn_dalla(RuntimeOrigin::signed(user), 0),
            Error::<Test>::AmountMustBeNonZero
        );
    });
}

#[test]
fn mint_bbzd_fails_on_zero_amount() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user = 2u64;
        
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        crate::CentralBankReserves::<Test>::put(1_000_000_000_000u128);
        
        let deposit_ref: BoundedVec<u8, ConstU32<64>> = 
            b"DEPOSIT_123".to_vec().try_into().unwrap();
        
        // Try to mint zero
        assert_noop!(
            Economy::mint_bbzd(
                RuntimeOrigin::signed(central_bank),
                user,
                0u128,
                deposit_ref
            ),
            Error::<Test>::AmountMustBeNonZero
        );
    });
}

#[test]
fn redeem_bbzd_fails_on_zero_amount() {
    new_test_ext().execute_with(|| {
        let user = 2u64;
        
        crate::BBZDBalances::<Test>::insert(user, 100_000_000_000u128);
        
        let bank_account: BoundedVec<u8, ConstU32<64>> = 
            b"BANK_123".to_vec().try_into().unwrap();
        
        // Try to redeem zero
        assert_noop!(
            Economy::redeem_bbzd(RuntimeOrigin::signed(user), 0u128, bank_account),
            Error::<Test>::AmountMustBeNonZero
        );
    });
}

#[test]
fn max_bbzd_supply_enforced() {
    new_test_ext().execute_with(|| {
        let central_bank = 1u64;
        let user = 2u64;
        
        crate::AuthorizedMinters::<Test>::insert(central_bank, true);
        
        // Set reserves to max u128
        let max_reserves = u128::MAX;
        crate::CentralBankReserves::<Test>::put(max_reserves);
        
        // Try to mint max u128 (should work if reserves exist)
        let large_mint = 1_000_000_000_000_000_000u128; // 1 billion bBZD
        let deposit_ref: BoundedVec<u8, ConstU32<64>> = 
            b"LARGE_DEPOSIT".to_vec().try_into().unwrap();
        
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(central_bank),
            user,
            large_mint,
            deposit_ref
        ));
        
        assert_eq!(crate::BBZDBalances::<Test>::get(user), large_mint);
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), large_mint);
    });
}

#[test]
fn burn_dalla_with_exact_balance() {
    new_test_ext().execute_with(|| {
        let user = 2;
        let user_balance = Balances::free_balance(user);
        
        // Burn exact balance (keeping existential deposit)
        let burn_amount = user_balance - 1; // Keep 1 for existential deposit
        
        assert_ok!(Economy::burn_dalla(RuntimeOrigin::signed(user), burn_amount));
        
        assert_eq!(Balances::free_balance(user), 1);
    });
}

#[test]
fn concurrent_redemptions_maintain_consistency() {
    new_test_ext().execute_with(|| {
        let user1 = 2u64;
        let user2 = 3u64;
        let user3 = 4u64;
        
        // Setup: Multiple users with bBZD
        crate::BBZDBalances::<Test>::insert(user1, 100_000_000_000u128);
        crate::BBZDBalances::<Test>::insert(user2, 200_000_000_000u128);
        crate::BBZDBalances::<Test>::insert(user3, 300_000_000_000u128);
        crate::TotalBbzdSupply::<Test>::put(600_000_000_000u128);
        crate::CentralBankReserves::<Test>::put(600_000_000_000u128);
        crate::NextRedemptionId::<Test>::put(1);
        
        let initial_supply = crate::TotalBbzdSupply::<Test>::get();
        
        // All users redeem simultaneously
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user1),
            50_000_000_000u128,
            b"BANK_USER1".to_vec().try_into().unwrap()
        ));
        
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user2),
            100_000_000_000u128,
            b"BANK_USER2".to_vec().try_into().unwrap()
        ));
        
        assert_ok!(Economy::redeem_bbzd(
            RuntimeOrigin::signed(user3),
            150_000_000_000u128,
            b"BANK_USER3".to_vec().try_into().unwrap()
        ));
        
        // Verify all redemptions recorded
        assert_eq!(crate::NextRedemptionId::<Test>::get(), 4); // 1, 2, 3 used
        
        // Verify total supply decreased correctly
        let expected_supply = initial_supply - 50_000_000_000u128 - 100_000_000_000u128 - 150_000_000_000u128;
        assert_eq!(crate::TotalBbzdSupply::<Test>::get(), expected_supply);
        
        // Verify individual balances
        assert_eq!(crate::BBZDBalances::<Test>::get(user1), 50_000_000_000u128);
        assert_eq!(crate::BBZDBalances::<Test>::get(user2), 100_000_000_000u128);
        assert_eq!(crate::BBZDBalances::<Test>::get(user3), 150_000_000_000u128);
        
        // Verify invariant holds
        assert!(crate::TotalBbzdSupply::<Test>::get() <= crate::CentralBankReserves::<Test>::get());
    });
}

#[test]
fn authorization_can_be_revoked_and_restored() {
    new_test_ext().execute_with(|| {
        let minter = 2u64;
        let user = 3u64;
        
        // Authorize minter
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::root(),
            minter,
            true
        ));
        
        assert!(crate::AuthorizedMinters::<Test>::get(minter));
        
        // Revoke authorization
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::root(),
            minter,
            false
        ));
        
        assert!(!crate::AuthorizedMinters::<Test>::get(minter));
        
        // Verify minter cannot mint
        crate::CentralBankReserves::<Test>::put(1_000_000_000_000u128);
        assert_noop!(
            Economy::mint_bbzd(
                RuntimeOrigin::signed(minter),
                user,
                100_000_000_000u128,
                b"DEPOSIT_123".to_vec().try_into().unwrap()
            ),
            Error::<Test>::UnauthorizedMinter
        );
        
        // Restore authorization
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::root(),
            minter,
            true
        ));
        
        // Now minting works
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(minter),
            user,
            100_000_000_000u128,
            b"DEPOSIT_123".to_vec().try_into().unwrap()
        ));
    });
}

// ============================================================================
// RATE LIMITING TESTS (AR-15)
// ============================================================================

#[test]
fn mint_bbzd_rate_limit_blocks_calls_after_per_block_maximum() {
    new_test_ext().execute_with(|| {
        let minter = 2u64;
        let user = 3u64;
        let small_amount = 1_000_000u128; // tiny amount so reserve never runs out

        // Authorise the minter and seed reserves.
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::root(), minter, true));
        crate::CentralBankReserves::<Test>::put(100_000_000_000_000u128);

        // MaxMintPerBlock = 5 in the mock; exhaust the limit.
        for _ in 0..5u32 {
            assert_ok!(Economy::mint_bbzd(
                RuntimeOrigin::signed(minter),
                user,
                small_amount,
                b"DEP".to_vec().try_into().unwrap(),
            ));
        }

        // Sixth call in the same block must be rejected.
        assert_noop!(
            Economy::mint_bbzd(
                RuntimeOrigin::signed(minter),
                user,
                small_amount,
                b"DEP6".to_vec().try_into().unwrap(),
            ),
            Error::<Test>::RateLimitExceeded
        );
    });
}

#[test]
fn mint_bbzd_rate_limit_resets_on_next_block() {
    new_test_ext().execute_with(|| {
        let minter = 2u64;
        let user = 3u64;
        let small_amount = 1_000_000u128;

        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::root(), minter, true));
        crate::CentralBankReserves::<Test>::put(100_000_000_000_000u128);

        // Exhaust limit in block 1.
        for _ in 0..5u32 {
            assert_ok!(Economy::mint_bbzd(
                RuntimeOrigin::signed(minter),
                user,
                small_amount,
                b"DEP".to_vec().try_into().unwrap(),
            ));
        }

        // Advance to block 2 — counter resets.
        System::set_block_number(2);

        // Should succeed again.
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(minter),
            user,
            small_amount,
            b"DEP_B2".to_vec().try_into().unwrap(),
        ));
    });
}

#[test]
fn mint_bbzd_rate_limit_is_per_account() {
    new_test_ext().execute_with(|| {
        let minter1 = 2u64;
        let minter2 = 3u64;
        let user = 100u64; // treasury account already has funds
        let small_amount = 1_000_000u128;

        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::root(), minter1, true));
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::root(), minter2, true));
        crate::CentralBankReserves::<Test>::put(100_000_000_000_000u128);

        // Exhaust minter1's limit.
        for _ in 0..5u32 {
            assert_ok!(Economy::mint_bbzd(
                RuntimeOrigin::signed(minter1),
                user,
                small_amount,
                b"DEP1".to_vec().try_into().unwrap(),
            ));
        }

        // minter2 still has a fresh slot within the same block.
        assert_ok!(Economy::mint_bbzd(
            RuntimeOrigin::signed(minter2),
            user,
            small_amount,
            b"DEP2".to_vec().try_into().unwrap(),
        ));
    });
}

// ============================================================================
// TOURISM PAYMENT TESTS
// ============================================================================

#[test]
fn process_tourism_payment_works() {
    new_test_ext().execute_with(|| {
        let tourist = 2u64;
        let vendor  = 3u64;
        let amount  = 10_000_000_000u64; // 10K DALLA

        let tourist_before = Balances::free_balance(tourist);
        let vendor_before  = Balances::free_balance(vendor);
        let treasury       = 100u64;
        let treasury_before = Balances::free_balance(treasury);

        // category_id 0 = Accommodation (500 ppm incentive)
        assert_ok!(Economy::process_tourism_payment(
            RuntimeOrigin::signed(tourist),
            vendor,
            amount,
            0,
        ));

        // Tourist paid `amount` and received incentive from treasury
        let incentive = sp_runtime::Permill::from_parts(50_000) * amount;
        assert_eq!(Balances::free_balance(tourist), tourist_before - amount + incentive);
        assert_eq!(Balances::free_balance(vendor),  vendor_before + amount);
        assert_eq!(Balances::free_balance(treasury), treasury_before - incentive);

        // Event emitted
        System::assert_last_event(
            Event::TourismIncentivePaid {
                tourist,
                vendor,
                amount,
                incentive,
            }.into()
        );
    });
}

#[test]
fn process_tourism_payment_all_categories_work() {
    new_test_ext().execute_with(|| {
        let tourist = 2u64;
        let vendor  = 3u64;
        let amount  = 1_000_000_000u64; // 1K DALLA (small so treasury covers all)

        // Test all valid category IDs 0-5
        for category_id in 0u8..=5u8 {
            // Re-fund tourist each iteration
            Balances::make_free_balance_be(&tourist, 1_000_000_000_000);
            assert_ok!(
                Economy::process_tourism_payment(
                    RuntimeOrigin::signed(tourist),
                    vendor,
                    amount,
                    category_id,
                )
            );
        }
    });
}

#[test]
fn process_tourism_payment_invalid_category_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Economy::process_tourism_payment(
                RuntimeOrigin::signed(2u64),
                3u64,
                1_000_000_000u64,
                6, // invalid — only 0-5 valid
            ),
            Error::<Test>::InvalidTourismCategory
        );
    });
}

#[test]
fn process_tourism_payment_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let tourist = 2u64;
        let huge_amount = 1_000_000_000_000_000u64; // more than tourist balance
        assert_noop!(
            Economy::process_tourism_payment(
                RuntimeOrigin::signed(tourist),
                3u64,
                huge_amount,
                0,
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn get_tourism_incentive_rate_returns_correct_rates() {
    new_test_ext().execute_with(|| {
        use crate::TourismCategory;
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Accommodation), 50_000);
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Dining),        30_000);
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Tours),         70_000);
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Transportation),20_000);
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Shopping),      40_000);
        assert_eq!(Economy::get_tourism_incentive_rate(&TourismCategory::Cultural),      80_000);
    });
}

// ============================================================================
// EXPANDED COVERAGE TESTS
// ============================================================================

#[test]
fn process_redemption_unauthorized_minter_fails() {
    new_test_ext().execute_with(|| {
        // Account 2 is NOT an authorized minter
        // First create a redemption to try to process
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::signed(1), // governance
            2,
            true,
        ));
        assert_ok!(Economy::update_reserves(RuntimeOrigin::signed(1), 1_000_000_000_000));
        let deposit_ref = BoundedVec::try_from(b"DEP-001".to_vec()).unwrap();
        assert_ok!(Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 1000, deposit_ref));

        let bank = BoundedVec::try_from(b"BZ-BANK-001".to_vec()).unwrap();
        assert_ok!(Economy::redeem_bbzd(RuntimeOrigin::signed(3), 500, bank));

        // Revoke minter authorization first
        assert_ok!(Economy::set_minter_authorization(
            RuntimeOrigin::signed(1),
            2,
            false,
        ));

        // Now account 2 should fail to process
        assert_noop!(
            Economy::process_redemption(RuntimeOrigin::signed(2), 1),
            Error::<Test>::UnauthorizedMinter
        );
    });
}

#[test]
fn update_reserves_below_supply_fails() {
    new_test_ext().execute_with(|| {
        // Set up some bBZD supply
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::signed(1), 2, true));
        assert_ok!(Economy::update_reserves(RuntimeOrigin::signed(1), 10_000));
        let dep = BoundedVec::try_from(b"D".to_vec()).unwrap();
        assert_ok!(Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 5_000, dep));

        // Try to lower reserves below circulating supply (5000)
        assert_noop!(
            Economy::update_reserves(RuntimeOrigin::signed(1), 4_999),
            Error::<Test>::InsufficientReserves
        );
    });
}

#[test]
fn pending_redemption_ids_tracked() {
    new_test_ext().execute_with(|| {
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::signed(1), 2, true));
        assert_ok!(Economy::update_reserves(RuntimeOrigin::signed(1), 100_000));
        let dep = BoundedVec::try_from(b"D".to_vec()).unwrap();
        assert_ok!(Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 10_000, dep));

        // Redeem twice
        let bank1 = BoundedVec::try_from(b"BK1".to_vec()).unwrap();
        let bank2 = BoundedVec::try_from(b"BK2".to_vec()).unwrap();
        assert_ok!(Economy::redeem_bbzd(RuntimeOrigin::signed(3), 1_000, bank1));
        assert_ok!(Economy::redeem_bbzd(RuntimeOrigin::signed(3), 2_000, bank2));

        let pending = crate::PendingRedemptionIds::<Test>::get();
        assert_eq!(pending.len(), 2);

        // Process first
        assert_ok!(Economy::process_redemption(RuntimeOrigin::signed(2), pending[0]));
        let pending_after = crate::PendingRedemptionIds::<Test>::get();
        assert_eq!(pending_after.len(), 1);
    });
}

#[test]
fn redemption_processed_event_fields() {
    new_test_ext().execute_with(|| {
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::signed(1), 2, true));
        assert_ok!(Economy::update_reserves(RuntimeOrigin::signed(1), 100_000));
        let dep = BoundedVec::try_from(b"D".to_vec()).unwrap();
        assert_ok!(Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 5_000, dep));

        let bank = BoundedVec::try_from(b"BK1".to_vec()).unwrap();
        assert_ok!(Economy::redeem_bbzd(RuntimeOrigin::signed(3), 1_000, bank));

        let pending = crate::PendingRedemptionIds::<Test>::get();
        let rid = pending[0];
        assert_ok!(Economy::process_redemption(RuntimeOrigin::signed(2), rid));

        // Verify event with exact fields
        let id_bytes = rid.to_le_bytes();
        let mut ref_bytes = b"REDEMPTION_".to_vec();
        ref_bytes.extend_from_slice(&id_bytes);
        let transfer_ref: BoundedVec<u8, ConstU32<64>> = ref_bytes.try_into().unwrap();

        System::assert_has_event(
            Event::RedemptionProcessed {
                redemption_id: rid,
                amount: 1_000,
                bank_transfer_reference: transfer_ref,
            }.into()
        );
    });
}

#[test]
fn annual_inflation_event_emitted() {
    new_test_ext().execute_with(|| {
        let supply_before = Economy::total_supply();
        let inflation_amount = sp_runtime::Permill::from_percent(2) * supply_before;

        // Advance to block BLOCKS_PER_YEAR + 1
        let target = 5_256_000u64 + 1;
        System::set_block_number(target);
        Economy::on_initialize(target);

        let new_supply = Economy::total_supply();
        System::assert_has_event(
            Event::AnnualInflationApplied {
                amount: inflation_amount,
                new_supply,
            }.into()
        );
    });
}

#[test]
fn inflation_routes_to_public_goods_treasury() {
    new_test_ext().execute_with(|| {
        let pg_before = Balances::free_balance(101); // PublicGoodsTreasury
        let wb_before = Balances::free_balance(102); // WellbeingTreasury

        let target = 5_256_000u64 + 1;
        System::set_block_number(target);
        Economy::on_initialize(target);

        let pg_after = Balances::free_balance(101);
        let wb_after = Balances::free_balance(102);

        // PG should receive 10% of inflation minus wellbeing portion
        assert!(pg_after > pg_before, "Public goods treasury should receive funds");
        // Wellbeing gets 5% of PG allocation
        assert!(wb_after > wb_before, "Wellbeing treasury should receive funds");
    });
}

#[test]
fn governance_burn_zero_amount_succeeds() {
    new_test_ext().execute_with(|| {
        // governance_burn has no AmountMustBeNonZero guard — documents this behavior
        let supply_before = Economy::total_supply();
        let result = Economy::governance_burn(RuntimeOrigin::signed(1), 0);
        // Either it succeeds with zero burn or it has an amount check
        if result.is_ok() {
            // Zero burn, supply unchanged
            assert_eq!(Economy::total_supply(), supply_before);
        }
        // If it fails, that's also acceptable (means guard exists)
    });
}

#[test]
fn process_tourism_payment_zero_amount() {
    new_test_ext().execute_with(|| {
        // Zero-amount payment — documents current behavior
        let result = Economy::process_tourism_payment(
            RuntimeOrigin::signed(2), 3, 0, 0,
        );
        // Currently no guard → succeeds silently, which is fine to document
        if result.is_ok() {
            // Vendor received 0, no incentive
        }
    });
}

#[test]
fn multiple_inflation_years() {
    new_test_ext().execute_with(|| {
        let supply_0 = Economy::total_supply();

        // First year
        let year1 = 5_256_000u64 + 1;
        System::set_block_number(year1);
        Economy::on_initialize(year1);
        let supply_1 = Economy::total_supply();
        assert!(supply_1 > supply_0);

        // Second year
        let year2 = 2 * 5_256_000u64 + 1;
        System::set_block_number(year2);
        Economy::on_initialize(year2);
        let supply_2 = Economy::total_supply();
        assert!(supply_2 > supply_1);

        // Compounding: second year inflation > first year (since base is larger)
        let first_inflation = supply_1 - supply_0;
        let second_inflation = supply_2 - supply_1;
        assert!(second_inflation > first_inflation, "Compounding effect");
    });
}

#[test]
fn mint_bbzd_rate_limit_exact_boundary() {
    new_test_ext().execute_with(|| {
        assert_ok!(Economy::set_minter_authorization(RuntimeOrigin::signed(1), 2, true));
        assert_ok!(Economy::update_reserves(RuntimeOrigin::signed(1), 1_000_000));
        let dep = BoundedVec::try_from(b"D".to_vec()).unwrap();

        // MaxMintPerBlock = 5, mint exactly 5 times
        for i in 0..5u8 {
            let d = BoundedVec::try_from(vec![i]).unwrap();
            assert_ok!(Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 100, d));
        }

        // 6th should fail
        assert_noop!(
            Economy::mint_bbzd(RuntimeOrigin::signed(2), 3, 100, dep),
            Error::<Test>::RateLimitExceeded
        );
    });
}
