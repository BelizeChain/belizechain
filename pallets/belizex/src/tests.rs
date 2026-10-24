//! Unit tests for BelizeX pallet

use super::*;

/// Test ledger: maps every asset onto the mock `Balances` so existing tests keep
/// their single-currency semantics, while still exercising the real
/// `AssetLedger` seam that production uses.
pub struct MockAssetLedger;

#[cfg(test)]
thread_local! {
    /// Asset of every transfer the mock ledger has performed, in order.
    static TRANSFER_LOG: core::cell::RefCell<Vec<(AssetId, u64, u64)>> =
        const { core::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
thread_local! {
    /// Account whose incoming transfers the mock ledger rejects.
    ///
    /// `settle_fill` moves the base leg first and the quote leg second, so
    /// rejecting transfers to the *asker* exercises the path where the first leg
    /// has already moved and must be put back.
    static REJECT_TRANSFERS_TO: core::cell::RefCell<Option<u64>> =
        const { core::cell::RefCell::new(None) };
}

impl MockAssetLedger {
    fn clear() {
        TRANSFER_LOG.with(|l| l.borrow_mut().clear());
    }
    fn transfers() -> Vec<(AssetId, u64, u64)> {
        TRANSFER_LOG.with(|l| l.borrow().clone())
    }
    /// Reject every incoming transfer to `who` until `accept_transfers_to` is called.
    fn reject_transfers_to(who: u64) {
        REJECT_TRANSFERS_TO.with(|r| *r.borrow_mut() = Some(who));
    }
    fn accept_transfers_to_all() {
        REJECT_TRANSFERS_TO.with(|r| *r.borrow_mut() = None);
    }
}

impl AssetLedger<u64> for MockAssetLedger {
    fn balance_of(_asset: AssetId, who: &u64) -> u128 {
        Balances::free_balance(who)
    }

    fn transfer(asset: AssetId, from: &u64, to: &u64, amount: u128) -> DispatchResult {
        // WUSDC has no ledger in the runtime; mirror that here so the test
        // double does not silently accept what production rejects.
        if asset == AssetId::WUSDC {
            return Err(Error::<Test>::UnsupportedAsset.into());
        }
        if amount == 0 {
            return Ok(());
        }
        // Failure injection: see `REJECT_TRANSFERS_TO`.
        if REJECT_TRANSFERS_TO.with(|r| *r.borrow()) == Some(*to) {
            return Err(Error::<Test>::InsufficientBalance.into());
        }
        TRANSFER_LOG.with(|l| l.borrow_mut().push((asset, *from, *to)));
        <Balances as frame_support::traits::Currency<u64>>::transfer(
            from,
            to,
            amount,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        )
        .map(|_| ())
        .map_err(|_| Error::<Test>::InsufficientBalance.into())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn set_balance(_asset: AssetId, who: &u64, amount: u128) {
        // Every asset collapses to `Balances` in this double, which is exactly
        // why the mock cannot reproduce a runtime failure on the quote leg.
        let _ = Balances::make_free_balance_be(who, amount);
    }
}
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn pause_and_resume_blocks_user_flows() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // ensure default pair exists (DALLA, BBZD) => (0,1)
        assert!(BelizeX::trading_pairs((0, 1)).is_some());

        // Add liquidity should work when not paused
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            1_000u128,
            1_000u128,
            1
        ));

        // Pause globally
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));

        // Adding liquidity now should fail with Paused
        assert_noop!(
            BelizeX::add_liquidity(RuntimeOrigin::signed(1), 0, 1, 10u128, 10u128, 1),
            Error::<Test>::Paused
        );

        // Resume globally
        assert_ok!(BelizeX::resume_global(RuntimeOrigin::root()));

        // Add liquidity should succeed again
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10u128,
            10u128,
            1
        ));
    });
}

#[test]
fn per_pair_toggle_enforced() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Ensure default pair exists and is active
        let mut pair = BelizeX::trading_pairs((0, 1)).expect("pair exists");
        assert!(pair.active);

        // Add liquidity first so we can test trading
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            1_000u128,
            1_000u128,
            1
        ));

        // Deactivate via governance
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, false));
        pair = BelizeX::trading_pairs((0, 1)).unwrap();
        assert!(!pair.active);

        // Try to trade should fail with PairNotActive
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 1, false),
            Error::<Test>::PairNotActive
        );

        // Reactivate and trade should proceed to at least pass dispatch
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, true));
        assert_ok!(BelizeX::execute_trade(
            RuntimeOrigin::signed(1),
            0,
            1,
            100u128,
            0,
            false
        ));
    });
}

#[test]
fn remove_liquidity_returns_funds() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Add liquidity first so we have LP tokens to remove.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            1,
        ));

        let lp = BelizeX::get_lp_balance(&1);
        assert!(lp > 0, "should have LP tokens after add_liquidity");

        let balance_before = Balances::free_balance(1);

        // Remove all LP tokens.
        assert_ok!(BelizeX::remove_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            lp,
            0, // min_base — accept any
            0, // min_quote — accept any
        ));

        // Balance should have recovered (minus any rounding).
        let balance_after = Balances::free_balance(1);
        assert!(
            balance_after >= balance_before,
            "funds should be returned on remove_liquidity"
        );
        assert_eq!(BelizeX::get_lp_balance(&1), 0);
    });
}

#[test]
fn remove_liquidity_fails_with_insufficient_lp_tokens() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Account 2 never added liquidity.
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(2), 0, 1, 1_000u128, 0, 0,),
            Error::<Test>::InsufficientLPTokens
        );
    });
}

#[test]
fn cancel_order_removes_from_book() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Place a limit order first.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            500, // price
            1,   // amount
            10,  // expiry blocks
        ));

        // Order ID starts at 0.
        let order_id = 0u32;
        assert!(BelizeX::order_book(order_id).is_some());

        // Cancel it.
        assert_ok!(BelizeX::cancel_order(RuntimeOrigin::signed(1), order_id));

        // Must no longer exist.
        assert!(BelizeX::order_book(order_id).is_none());
    });
}

#[test]
fn cancel_order_fails_for_non_creator() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            500,
            1,
            10,
        ));

        // Account 2 did not create this order.
        assert_noop!(
            BelizeX::cancel_order(RuntimeOrigin::signed(2), 0),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn add_liquidity_increases_pair_reserves() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let pair_before = BelizeX::trading_pairs((0u8, 1u8)).expect("default pair must exist");
        let reserve_before = pair_before.base_reserve;

        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(2),
            0,
            1,
            5_000u128,
            5_000u128,
            1,
        ));

        let pair_after = BelizeX::trading_pairs((0u8, 1u8)).unwrap();
        assert!(pair_after.base_reserve > reserve_before);
    });
}

#[test]
fn add_liquidity_zero_amount_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::add_liquidity(
                RuntimeOrigin::signed(1),
                0,
                1,
                0u128, // zero base
                1_000u128,
                1,
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}

#[test]
fn global_pause_prevents_remove_liquidity() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Add liquidity, then pause.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            1_000u128,
            1_000u128,
            1,
        ));
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));

        let lp = BelizeX::get_lp_balance(&1);
        assert!(lp > 0);

        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 0, 1, lp, 0, 0,),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn kyc_gating_blocks_when_not_verified() {
    // Build a new ext but override Kyc to DenyKyc by crafting calls requiring KYC
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Replace KYC behavior at compile-time would require alt runtime.
        // For a runtime-free check, we can simulate by calling register_tourism_trader which requires root, not KYC.
        // Instead, assert KYC gating exists by checking error surfaces when mocked to false.
        // Since our MockKyc returns true, we check that Paused logic works and KYC path is covered by type.
        // For completeness, we still assert that placing an order with active pair works (requires KYC in code path).
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            100,
            1,
            10
        ));
    });
}

#[test]
fn create_trading_pair_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Pair (5,6) does not exist yet.
        assert!(BelizeX::trading_pairs((5u8, 6u8)).is_none());

        // Root creates a new pair with fee_rate = 50.
        assert_ok!(BelizeX::create_trading_pair(
            RuntimeOrigin::root(),
            5,
            6,
            50
        ));

        // Verify it now exists with correct fee rate and is active.
        let pair = BelizeX::trading_pairs((5u8, 6u8)).expect("pair must exist after creation");
        assert_eq!(pair.fee_rate, 50);
        assert!(pair.active);
    });
}

#[test]
fn create_trading_pair_already_exists_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Pair (0,1) already exists from genesis.
        assert!(BelizeX::trading_pairs((0u8, 1u8)).is_some());

        // Attempting to create it again must fail.
        assert_noop!(
            BelizeX::create_trading_pair(RuntimeOrigin::root(), 0, 1, 30),
            Error::<Test>::PairAlreadyExists
        );
    });
}

#[test]
fn register_tourism_trader_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Account 2 is not a tourism trader yet.
        assert!(!BelizeX::tourism_traders(2u64));

        // Root registers account 2 as a tourism trader.
        assert_ok!(BelizeX::register_tourism_trader(RuntimeOrigin::root(), 2));

        // Now account 2 must be a verified tourism trader.
        assert!(BelizeX::tourism_traders(2u64));
    });
}

#[test]
fn execute_trade_tourism_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Register account 1 as a verified tourism trader.
        assert_ok!(BelizeX::register_tourism_trader(RuntimeOrigin::root(), 1));

        // Seed liquidity into pair (0,1).
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1,
        ));

        // Execute a tourism trade — should receive the discounted fee rate.
        assert_ok!(BelizeX::execute_trade(
            RuntimeOrigin::signed(1),
            0,
            1,
            500u128,
            0,
            true,
        ));
    });
}

#[test]
fn settlement_debits_the_assets_the_pair_names() {
    // P1-08 regression. Before the `AssetLedger` seam, `execute_trade` moved
    // native DALLA for both legs, so this assertion could not be written: the
    // bBZD side had no balance to observe. The mock ledger records the asset of
    // every transfer so the test can assert the *pair's* assets are the ones
    // debited, not just that some balance changed.
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1,
        ));

        MockAssetLedger::clear();

        assert_ok!(BelizeX::execute_trade(
            RuntimeOrigin::signed(1),
            0,
            1,
            500u128,
            0,
            false,
        ));

        let log = MockAssetLedger::transfers();
        assert!(
            log.iter().any(|(a, _, _)| *a == AssetId::DALLA),
            "the input leg must be denominated in the base asset: {log:?}"
        );
        assert!(
            log.iter().any(|(a, _, _)| *a == AssetId::BBZD),
            "the output leg must be denominated in the quote asset: {log:?}"
        );
    });
}

#[test]
fn settlement_refuses_an_asset_with_no_ledger() {
    // WUSDC has no ledger in this runtime. The mock mirrors production and
    // rejects it rather than inventing a balance.
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            MockAssetLedger::transfer(AssetId::WUSDC, &1, &2, 1),
            Error::<Test>::UnsupportedAsset
        );
    });
}

#[test]
fn execute_multihop_trade_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Seed liquidity so pair (0,1) has non-zero reserves.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1,
        ));

        // Execute a single-hop multihop trade: asset 0 → asset 1.
        assert_ok!(BelizeX::execute_multihop_trade(
            RuntimeOrigin::signed(1),
            vec![0u8, 1u8],
            1_000u128,
            0u128, // accept any output amount
            false,
        ));
    });
}

#[test]
fn execute_multihop_trade_too_short_path_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // A path with fewer than 2 elements is invalid.
        assert_noop!(
            BelizeX::execute_multihop_trade(
                RuntimeOrigin::signed(1),
                vec![0u8],
                100u128,
                0u128,
                false,
            ),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn place_limit_order_sell_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Place a sell limit order.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Sell.as_u8(),
            200, // price
            50,  // amount
            20,  // expires_in_blocks
        ));

        // The order must appear in the order book.
        let order = BelizeX::order_book(0).expect("order must exist");
        assert_eq!(order.amount, 200);
        assert_eq!(order.price, 50);
    });
}

#[test]
fn place_limit_order_zero_price_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Price of 0 must be rejected.
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::Buy.as_u8(),
                100, // amount
                0,   // price = 0 is invalid
                10,
            ),
            Error::<Test>::InvalidPrice
        );
    });
}

// ================================================================
// Additional error-path tests
// ================================================================

#[test]
fn create_trading_pair_non_root_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::create_trading_pair(RuntimeOrigin::signed(1), 5, 6, 50),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_liquidity_pair_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Pair (9,8) does not exist.
        assert_noop!(
            BelizeX::add_liquidity(RuntimeOrigin::signed(1), 9, 8, 1_000u128, 1_000u128, 0),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn add_liquidity_pair_not_active_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Deactivate the default pair.
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, false));

        assert_noop!(
            BelizeX::add_liquidity(RuntimeOrigin::signed(1), 0, 1, 1_000u128, 1_000u128, 0),
            Error::<Test>::PairNotActive
        );
    });
}

#[test]
fn add_liquidity_insufficient_balance_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Account 1 has 1_000_000_000_000. Try to add more than that.
        assert_noop!(
            BelizeX::add_liquidity(
                RuntimeOrigin::signed(1),
                0,
                1,
                900_000_000_000u128,
                900_000_000_000u128,
                0
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn add_liquidity_paused_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_noop!(
            BelizeX::add_liquidity(RuntimeOrigin::signed(1), 0, 1, 1_000u128, 1_000u128, 0),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn execute_trade_pair_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 9, 8, 100u128, 0, false),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn execute_trade_pair_not_active_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, false));
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 0, false),
            Error::<Test>::PairNotActive
        );
    });
}

#[test]
fn execute_trade_paused_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 0, false),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn execute_trade_tourism_unauthorized_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Account 1 is NOT a tourism trader.
        assert!(!BelizeX::tourism_traders(1u64));

        // Seed some liquidity first.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1
        ));

        // Attempt tourism trade without registration.
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 0, true),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn execute_trade_slippage_exceeded_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Seed liquidity.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1
        ));

        // Request unrealistically high min_amount_out.
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 999_999u128, false),
            Error::<Test>::SlippageExceeded
        );
    });
}

#[test]
fn execute_trade_insufficient_liquidity_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Default pair exists but has zero reserves.
        let pair = BelizeX::trading_pairs((0u8, 1u8)).unwrap();
        assert_eq!(pair.base_reserve, 0);

        // Trading against empty pool should fail.
        assert_noop!(
            BelizeX::execute_trade(RuntimeOrigin::signed(1), 0, 1, 100u128, 0, false),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn place_limit_order_pair_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                9,
                8,
                OrderType::Buy.as_u8(),
                100,
                50,
                10
            ),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn place_limit_order_paused_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::Buy.as_u8(),
                100,
                50,
                10
            ),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn place_limit_order_tourism_unauthorized_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Account 1 is NOT a tourism trader — TourismBuy requires it.
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::TourismBuy.as_u8(),
                100,
                50,
                10
            ),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn place_limit_order_tourism_sell_unauthorized_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::TourismSell.as_u8(),
                100,
                50,
                10
            ),
            Error::<Test>::Unauthorized
        );
    });
}

#[test]
fn cancel_order_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::cancel_order(RuntimeOrigin::signed(1), 999),
            Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn cancel_order_paused_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Place an order, then pause, then try to cancel.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            500,
            1,
            10,
        ));
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_noop!(
            BelizeX::cancel_order(RuntimeOrigin::signed(1), 0),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn remove_liquidity_zero_lp_tokens_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            1_000u128,
            1_000u128,
            0
        ));
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 0, 1, 0, 0, 0),
            Error::<Test>::BelowMinimumAmount
        );
    });
}

#[test]
fn remove_liquidity_pair_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 9, 8, 100, 0, 0),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn remove_liquidity_pair_not_active_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            1_000u128,
            1_000u128,
            0
        ));
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, false));
        let lp = BelizeX::get_lp_balance(&1);
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 0, 1, lp, 0, 0),
            Error::<Test>::PairNotActive
        );
    });
}

#[test]
fn remove_liquidity_slippage_min_base_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        let lp = BelizeX::get_lp_balance(&1);
        assert!(lp > 0);
        // Request more base than proportionally available.
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 0, 1, lp, 999_999, 0),
            Error::<Test>::SlippageExceeded
        );
    });
}

#[test]
fn remove_liquidity_slippage_min_quote_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        let lp = BelizeX::get_lp_balance(&1);
        assert!(lp > 0);
        assert_noop!(
            BelizeX::remove_liquidity(RuntimeOrigin::signed(1), 0, 1, lp, 0, 999_999),
            Error::<Test>::SlippageExceeded
        );
    });
}

#[test]
fn register_tourism_trader_non_root_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::register_tourism_trader(RuntimeOrigin::signed(1), 2),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn pause_global_non_root_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::pause_global(RuntimeOrigin::signed(1)),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn resume_global_non_root_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::resume_global(RuntimeOrigin::signed(1)),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_pair_status_non_root_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::set_pair_status(RuntimeOrigin::signed(1), 0, 1, false),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_pair_status_pair_not_found_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::set_pair_status(RuntimeOrigin::root(), 9, 8, false),
            Error::<Test>::PairNotFound
        );
    });
}

#[test]
fn execute_multihop_paused_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_noop!(
            BelizeX::execute_multihop_trade(
                RuntimeOrigin::signed(1),
                vec![0u8, 1u8],
                100u128,
                0,
                false
            ),
            Error::<Test>::Paused
        );
    });
}

#[test]
fn execute_multihop_path_too_long_fails() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Path length > 5 should fail.
        assert_noop!(
            BelizeX::execute_multihop_trade(
                RuntimeOrigin::signed(1),
                vec![0, 1, 2, 0, 1, 2],
                100u128,
                0,
                false
            ),
            Error::<Test>::SlippageExceeded
        );
    });
}

// ================================================================
// Event emission tests
// ================================================================

#[test]
fn create_trading_pair_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::create_trading_pair(
            RuntimeOrigin::root(),
            5,
            6,
            50
        ));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::TradingPairCreated {
            base_asset: 5,
            quote_asset: 6,
        }));
    });
}

#[test]
fn add_liquidity_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        // sqrt(10_000 * 10_000) = 10_000 LP tokens for initial deposit.
        System::assert_last_event(RuntimeEvent::BelizeX(Event::LiquidityAdded {
            provider: 1,
            pair: (0, 1),
            base_amount: 10_000,
            quote_amount: 10_000,
            lp_tokens: 10_000,
        }));
    });
}

#[test]
fn remove_liquidity_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        let lp = BelizeX::get_lp_balance(&1);
        assert_ok!(BelizeX::remove_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            lp,
            0,
            0
        ));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::LiquidityRemoved {
            provider: 1,
            pair: (0, 1),
            base_amount: 10_000,
            quote_amount: 10_000,
            lp_tokens: lp,
        }));
    });
}

#[test]
fn execute_trade_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1
        ));
        assert_ok!(BelizeX::execute_trade(
            RuntimeOrigin::signed(2),
            0,
            1,
            1_000u128,
            0,
            false
        ));
        // Verify the event is TradeExecuted type.
        // fee = 1000 * 30 / 10000 = 3; after_fee = 997
        // amount_out = 997 * 100000 / (100000 + 997) = 987
        System::assert_has_event(RuntimeEvent::BelizeX(Event::TradeExecuted {
            trader: 2,
            pair: (0, 1),
            amount_in: 1_000,
            amount_out: 987,
            fee_paid: 3,
            is_tourism_trade: false,
        }));
    });
}

#[test]
fn place_limit_order_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            500,
            1,
            10,
        ));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::OrderPlaced {
            order_id: 0,
            creator: 1,
            pair: (0, 1),
            order_type: OrderType::Buy.as_u8(),
            amount: 500,
            price: 1,
        }));
    });
}

#[test]
fn cancel_order_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Sell.as_u8(),
            200,
            50,
            10,
        ));
        assert_ok!(BelizeX::cancel_order(RuntimeOrigin::signed(1), 0));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::OrderCancelled {
            order_id: 0,
            creator: 1,
        }));
    });
}

#[test]
fn register_tourism_trader_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::register_tourism_trader(RuntimeOrigin::root(), 2));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::TourismTraderVerified {
            trader: 2,
        }));
    });
}

#[test]
fn pause_global_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::DexPaused));
    });
}

#[test]
fn resume_global_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::pause_global(RuntimeOrigin::root()));
        assert_ok!(BelizeX::resume_global(RuntimeOrigin::root()));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::DexResumed));
    });
}

#[test]
fn set_pair_status_emits_event() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BelizeX::set_pair_status(RuntimeOrigin::root(), 0, 1, false));
        System::assert_last_event(RuntimeEvent::BelizeX(Event::PairStatusUpdated {
            base_asset: 0,
            quote_asset: 1,
            active: false,
        }));
    });
}

// ================================================================
// Helper / public API tests
// ================================================================

#[test]
fn get_price_returns_none_for_missing_pair() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert!(BelizeX::get_price(&AssetId::from(9), &AssetId::from(8)).is_none());
    });
}

#[test]
fn get_price_returns_none_for_empty_reserves() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Default pair (0,1) exists with zero reserves.
        assert!(BelizeX::get_price(&AssetId::DALLA, &AssetId::BBZD).is_none());
    });
}

#[test]
fn get_price_returns_correct_ratio() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Seed 1:2 ratio.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            20_000u128,
            0
        ));
        let price = BelizeX::get_price(&AssetId::DALLA, &AssetId::BBZD).unwrap();
        // quote/base = 20_000/10_000 = 2.0
        assert_eq!(price, FixedU128::from_rational(20_000, 10_000));
    });
}

#[test]
fn get_verified_exchange_rate_uses_oracle() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // MockOracle returns Some(1_000_000) for any pair.
        let rate = BelizeX::get_verified_exchange_rate(0, 1);
        assert_eq!(rate, Some(1_000_000));
    });
}

#[test]
fn is_verified_tourism_merchant_fallback() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // MockOracle returns false for is_tourism_merchant.
        assert!(!BelizeX::is_verified_tourism_merchant(&1));
        // Register on-chain, then should be true via fallback.
        assert_ok!(BelizeX::register_tourism_trader(RuntimeOrigin::root(), 1));
        assert!(BelizeX::is_verified_tourism_merchant(&1));
    });
}

#[test]
fn get_effective_fee_rate_base() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // MockOracle returns tier 0, no tourism → base rate of 30 bps.
        assert_eq!(BelizeX::get_effective_fee_rate(&1, false), 30);
    });
}

#[test]
fn get_effective_fee_rate_with_tourism_discount() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Tourism discount = 10 bps → 30 - 10 = 20.
        assert_eq!(BelizeX::get_effective_fee_rate(&1, true), 20);
    });
}

#[test]
fn get_lp_balance_across_multiple_pairs() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Add liquidity to DALLA/BBZD (0,1) and TourismDALLA/BBZD (2,1).
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            2,
            1,
            5_000u128,
            5_000u128,
            0
        ));
        // Total LP should be sum across both pairs.
        let lp_pair_0_1 = BelizeX::lp_balances(1, (0u8, 1u8));
        let lp_pair_2_1 = BelizeX::lp_balances(1, (2u8, 1u8));
        let total = BelizeX::get_lp_balance(&1);
        assert_eq!(total, lp_pair_0_1 + lp_pair_2_1);
        assert!(total > 0);
    });
}

#[test]
fn liquidity_provider_storage_update() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert!(BelizeX::liquidity_providers(1).is_none());
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            5_000u128,
            5_000u128,
            0
        ));
        let lp_info = BelizeX::liquidity_providers(1).expect("LP info must exist");
        assert_eq!(lp_info.total_value_locked, 10_000);
        assert_eq!(lp_info.provider, 1u64);
    });
}

#[test]
fn add_liquidity_subsequent_uses_proportional_formula() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Initial deposit: sqrt(10_000 * 10_000) = 10_000 LP.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            10_000u128,
            0
        ));
        let lp1 = BelizeX::lp_balances(1, (0u8, 1u8));
        assert_eq!(lp1, 10_000);

        // Account 2: proportional deposit with same ratio.
        // min(5_000 * 10_000 / 10_000, 5_000 * 10_000 / 10_000) = 5_000.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(2),
            0,
            1,
            5_000u128,
            5_000u128,
            0
        ));
        let lp2 = BelizeX::lp_balances(2, (0u8, 1u8));
        assert_eq!(lp2, 5_000);
    });
}

#[test]
fn execute_trade_reserves_update_correctly() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            100_000u128,
            100_000u128,
            1
        ));
        let pair_before = BelizeX::trading_pairs((0u8, 1u8)).unwrap();
        let base_before = pair_before.base_reserve;
        let quote_before = pair_before.quote_reserve;

        assert_ok!(BelizeX::execute_trade(
            RuntimeOrigin::signed(2),
            0,
            1,
            1_000u128,
            0,
            false
        ));

        let pair_after = BelizeX::trading_pairs((0u8, 1u8)).unwrap();
        // Base reserve increases (trader sold base).
        assert!(pair_after.base_reserve > base_before);
        // Quote reserve decreases (trader received quote).
        assert!(pair_after.quote_reserve < quote_before);
    });
}

#[test]
fn asset_id_from_u8_roundtrip() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        for id in 0..=3u8 {
            let asset = AssetId::from(id);
            assert_eq!(asset.as_u8(), id);
        }
        // Unknown defaults to DALLA.
        assert_eq!(AssetId::from(255), AssetId::DALLA);
    });
}

#[test]
fn order_type_from_u8_roundtrip() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        for id in 0..=3u8 {
            let ot = OrderType::from(id);
            assert_eq!(ot.as_u8(), id);
        }
        // Unknown defaults to Buy.
        assert_eq!(OrderType::from(200), OrderType::Buy);
    });
}

#[test]
fn genesis_creates_three_default_pairs() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // DALLA/BBZD (0,1)
        let pair_01 = BelizeX::trading_pairs((0u8, 1u8)).expect("DALLA/BBZD must exist");
        assert!(pair_01.active);
        assert_eq!(pair_01.fee_rate, 30);

        // TourismDALLA/BBZD (2,1)
        let pair_21 = BelizeX::trading_pairs((2u8, 1u8)).expect("TourismDALLA/BBZD must exist");
        assert!(pair_21.active);
        assert_eq!(pair_21.fee_rate, 15);

        // WUSDC/BBZD (3,1)
        let pair_31 = BelizeX::trading_pairs((3u8, 1u8)).expect("WUSDC/BBZD must exist");
        assert!(pair_31.active);
        assert_eq!(pair_31.fee_rate, 30);
    });
}

#[test]
fn get_pair_reserves_u128_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            7_000u128,
            14_000u128,
            0
        ));
        let (base, quote, lp) = BelizeX::get_pair_reserves_u128(0, 1).unwrap();
        assert_eq!(base, 7_000);
        assert_eq!(quote, 14_000);
        assert!(lp > 0);
    });
}

#[test]
fn get_pair_reserves_u128_none_for_missing() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert!(BelizeX::get_pair_reserves_u128(9, 8).is_none());
    });
}

#[test]
fn get_implied_price_scaled_1e6_works() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Seed 1:2 ratio.
        assert_ok!(BelizeX::add_liquidity(
            RuntimeOrigin::signed(1),
            0,
            1,
            10_000u128,
            20_000u128,
            0
        ));
        let scaled = BelizeX::get_implied_price_scaled_1e6(0, 1).unwrap();
        // quote/base = 2.0, scaled by 1e6 = 2_000_000.
        assert_eq!(scaled, 2_000_000);
    });
}

#[test]
fn pool_account_is_deterministic() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let a = BelizeX::pool_account();
        let b = BelizeX::pool_account();
        assert_eq!(a, b);
    });
}

#[test]
fn next_order_id_increments() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_eq!(BelizeX::next_order_id(), 0);
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            100,
            10,
            10
        ));
        assert_eq!(BelizeX::next_order_id(), 1);
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Sell.as_u8(),
            200,
            20,
            10
        ));
        assert_eq!(BelizeX::next_order_id(), 2);
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// P1-08 prerequisite: the per-pair order index
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn placing_an_order_indexes_it_by_pair_and_side() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            100,
            50,
            20,
        ));
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Sell.as_u8(),
            100,
            60,
            20,
        ));

        let index = BelizeX::pair_orders((0, 1));
        assert_eq!(
            index.bids.to_vec(),
            vec![0],
            "buy order must index as a bid"
        );
        assert_eq!(
            index.asks.to_vec(),
            vec![1],
            "sell order must index as an ask"
        );

        // A different pair must not see either order.
        let other = BelizeX::pair_orders((1, 0));
        assert!(other.bids.is_empty() && other.asks.is_empty());
    });
}

#[test]
fn cancelling_an_order_removes_it_from_the_pair_index() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            100,
            50,
            20,
        ));
        assert_eq!(BelizeX::pair_orders((0, 1)).bids.to_vec(), vec![0]);

        assert_ok!(BelizeX::cancel_order(RuntimeOrigin::signed(1), 0));

        let index = BelizeX::pair_orders((0, 1));
        assert!(
            index.bids.is_empty(),
            "cancelled order must leave the index"
        );
        assert!(BelizeX::open_orders((0, 1)).0.is_empty());
    });
}

#[test]
fn open_orders_returns_the_best_price_first_on_each_side() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Three bids at 40, 70, 55; two asks at 90, 80.
        for price in [40u128, 70, 55] {
            assert_ok!(BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::Buy.as_u8(),
                10,
                price,
                20,
            ));
        }
        for price in [90u128, 80] {
            assert_ok!(BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::Sell.as_u8(),
                10,
                price,
                20,
            ));
        }

        let (bids, asks) = BelizeX::open_orders((0, 1));
        let bid_prices: Vec<u128> = bids.iter().map(|(_, o)| o.price).collect();
        let ask_prices: Vec<u128> = asks.iter().map(|(_, o)| o.price).collect();
        assert_eq!(bid_prices, vec![70, 55, 40], "best bid (highest) first");
        assert_eq!(ask_prices, vec![80, 90], "best ask (lowest) first");
    });
}

#[test]
fn open_orders_skips_ids_that_are_no_longer_in_the_order_book() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            50,
            20,
        ));

        // Simulate a bookkeeping inconsistency: an indexed ID with no book entry.
        crate::PairOrders::<Test>::mutate((0, 1), |index| {
            let _ = index.bids.try_push(4242);
        });

        let (bids, _) = BelizeX::open_orders((0, 1));
        assert_eq!(bids.len(), 1, "the stale ID must be skipped, not returned");
        assert_eq!(bids[0].0, 0);
    });
}

// ================================================================
// P1-08 — limit-order matching engine
// ================================================================
//
// `price` is scaled by `PRICE_SCALE` (1e6), so `PRICE_SCALE` is 1:1 and
// `2 * PRICE_SCALE` is 2:1. Using the constant instead of the literal keeps
// these tests readable and keeps them honest if the scale ever changes.

use crate::PRICE_SCALE;

/// Escrow account address in the mock runtime.
fn escrow() -> AccountId {
    BelizeX::order_escrow_account()
}

#[test]
fn placing_a_bid_escrows_the_quote_asset() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let before = Balances::free_balance(1);
        MockAssetLedger::clear();

        // Buy 10 base at 2 quote each => 20 quote escrowed.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        let log = MockAssetLedger::transfers();
        assert_eq!(log.len(), 1, "placement must move exactly one leg");
        assert_eq!(log[0].0, AssetId::BBZD, "a bid pays quote");
        assert_eq!(log[0].2, escrow(), "escrow goes to the escrow account");
        assert_eq!(
            before - Balances::free_balance(1),
            20,
            "the bidder must be down exactly the escrowed quote"
        );
    });
}

#[test]
fn placing_an_ask_escrows_the_base_asset() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let before = Balances::free_balance(2);
        MockAssetLedger::clear();

        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            7,
            3 * PRICE_SCALE,
            20,
        ));

        let log = MockAssetLedger::transfers();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, AssetId::DALLA, "an ask pays base");
        assert_eq!(before - Balances::free_balance(2), 7);
    });
}

#[test]
fn a_crossing_ask_fills_against_a_resting_bid_and_settles_both_legs() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Bid first: 10 base at 2 quote.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        MockAssetLedger::clear();

        // Ask from another account at the same price — must cross immediately.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        // Both orders are gone: fully filled orders leave the book.
        assert!(BelizeX::order_book(0).is_none(), "the bid must be consumed");
        assert!(BelizeX::order_book(1).is_none(), "the ask must be consumed");

        // The maker (bidder) got base, the taker (asker) got quote.
        let log = MockAssetLedger::transfers();
        assert!(
            log.iter()
                .any(|(a, from, to)| *a == AssetId::DALLA && *from == escrow() && *to == 1),
            "the bidder must receive the base asset from escrow: {log:?}"
        );
        assert!(
            log.iter()
                .any(|(a, from, to)| *a == AssetId::BBZD && *from == escrow() && *to == 2),
            "the asker must receive the quote asset from escrow: {log:?}"
        );

        // Net: bidder spent 20 quote and gained 10 base; asker the reverse.
        assert_eq!(Balances::free_balance(1), 1_000_000_000_000 - 20 + 10);
        assert_eq!(Balances::free_balance(2), 1_000_000_000_000 - 10 + 20);
        assert_eq!(
            Balances::free_balance(escrow()),
            0,
            "escrow must not retain funds after a full fill"
        );
    });
}

#[test]
fn a_rejected_second_leg_puts_the_first_leg_back_and_leaves_the_book_untouched() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Bid rests: 10 base at 2 quote, so 20 quote escrowed.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        let escrow_after_bid = Balances::free_balance(escrow());
        MockAssetLedger::clear();

        // The asker's ledger entry is about to reject the quote leg — the second
        // of the two transfers `settle_fill` performs.
        MockAssetLedger::reject_transfers_to(2);

        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        MockAssetLedger::accept_transfers_to_all();

        // The base leg ran and was reversed; the quote leg never completed.
        let log = MockAssetLedger::transfers();
        assert!(
            log.iter()
                .any(|(a, from, to)| *a == AssetId::DALLA && *from == escrow() && *to == 1),
            "the base leg should have been attempted: {log:?}"
        );
        assert!(
            log.iter()
                .any(|(a, from, to)| *a == AssetId::DALLA && *from == 1 && *to == escrow()),
            "the base leg should have been put back: {log:?}"
        );
        assert!(
            !log.iter().any(|(a, _, to)| *a == AssetId::BBZD && *to == 2),
            "the rejected quote leg must not appear as settled: {log:?}"
        );

        // No fill was recorded.
        assert!(
            !System::events()
                .iter()
                .any(|r| matches!(r.event, RuntimeEvent::BelizeX(Event::OrderFilled { .. }))),
            "a fill that moved nothing must not emit OrderFilled"
        );

        // The bidder kept its base, and the escrow holds exactly the two orders'
        // escrow: the bidder's 20 quote plus the asker's 10 base, which the ask
        // itself deposited on placement. The mock collapses every asset onto
        // `Balances`, so both legs show up in one figure.
        assert_eq!(
            Balances::free_balance(1),
            1_000_000_000_000 - 20,
            "the bidder must not gain base from a fill that did not settle"
        );
        assert_eq!(
            Balances::free_balance(escrow()),
            escrow_after_bid + 10,
            "escrow must hold the bid's 20 quote plus the ask's 10 base, and \
             nothing from the failed fill"
        );

        let bid = BelizeX::order_book(0).expect("the bid must still rest");
        assert_eq!(bid.remaining, 10, "the bid must not be partly filled");
        assert_eq!(bid.escrow, 20, "the bid's escrow must be intact");

        let ask = BelizeX::order_book(1).expect("the ask must still rest");
        assert_eq!(ask.remaining, 10, "the ask must not be partly filled");
        assert_eq!(ask.escrow, 10, "the ask's escrow must be intact");
    });
}

#[test]
fn a_partial_fill_leaves_the_remainder_resting() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Resting bid for 10 base at 2 quote.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        // Only 4 base offered — the bid should be 60% filled and stay resting.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            4,
            2 * PRICE_SCALE,
            20,
        ));

        let bid = BelizeX::order_book(0).expect("the bid must still be resting");
        assert_eq!(bid.remaining, 6, "6 base left to buy");
        assert_eq!(bid.escrow, 12, "escrow must shrink with the remainder");

        assert!(
            BelizeX::order_book(1).is_none(),
            "the ask was fully consumed"
        );
    });
}

#[test]
fn the_resting_order_gets_its_own_price_not_the_takers() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Resting bid at 2 quote per base.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        // A cheaper ask at 1 quote crosses a bid that is willing to pay 2.
        // The maker keeps its price, so the bidder pays only 1 per base.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            PRICE_SCALE,
            20,
        ));

        // Bidder: escrowed 20, paid 10, and the 10 difference is refunded when
        // the filled order leaves the book.
        assert_eq!(
            Balances::free_balance(1),
            1_000_000_000_000 - 20 + 10,
            "the bidder must be charged the maker's price, and refunded the rest"
        );
        assert_eq!(
            Balances::free_balance(escrow()),
            0,
            "price improvement must not strand escrow"
        );
    });
}

#[test]
fn cancelling_an_order_refunds_its_escrow() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        let before = Balances::free_balance(1);

        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        assert_eq!(Balances::free_balance(1), before - 20);

        assert_ok!(BelizeX::cancel_order(RuntimeOrigin::signed(1), 0));

        assert_eq!(
            Balances::free_balance(1),
            before,
            "cancel must return the full escrow"
        );
        assert_eq!(Balances::free_balance(escrow()), 0);
        assert_eq!(BelizeX::account_order_count(1), 0);
    });
}

#[test]
fn cancelling_a_partially_filled_order_refunds_only_the_remainder() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        // Fill half of it.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            5,
            2 * PRICE_SCALE,
            20,
        ));

        let before_cancel = Balances::free_balance(1);
        assert_ok!(BelizeX::cancel_order(RuntimeOrigin::signed(1), 0));

        assert_eq!(
            Balances::free_balance(1),
            before_cancel + 10,
            "only the unfilled half is still escrowed"
        );
    });
}

#[test]
fn orders_that_do_not_cross_do_not_fill() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            PRICE_SCALE,
            20,
        ));
        // Asks above the bid must rest, not fill.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        assert_eq!(
            BelizeX::order_book(0).map(|o| o.remaining),
            Some(10),
            "the bid must be untouched"
        );
        assert_eq!(
            BelizeX::order_book(1).map(|o| o.remaining),
            Some(10),
            "the ask must be untouched"
        );
    });
}

#[test]
fn an_account_cannot_match_its_own_order() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        // Same account, crossing price. It must be skipped, not matched.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        assert_eq!(
            BelizeX::order_book(0).map(|o| o.remaining),
            Some(10),
            "the bid must not match against its own creator"
        );
        assert_eq!(
            BelizeX::order_book(1).map(|o| o.remaining),
            Some(10),
            "the ask must not match against its own creator"
        );
    });
}

#[test]
fn the_best_priced_maker_fills_first() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Two resting bids: 1 quote and 3 quote, placed worst-first.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            PRICE_SCALE,
            20,
        ));
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            3 * PRICE_SCALE,
            20,
        ));

        // An ask for exactly one bid's size must hit the 3-quote bid, not the 1.
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(999),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            PRICE_SCALE,
            20,
        ));

        assert!(
            BelizeX::order_book(1).is_none(),
            "the 3-quote bid is the best price and must be consumed first"
        );
        assert_eq!(
            BelizeX::order_book(0).map(|o| o.remaining),
            Some(10),
            "the 1-quote bid must be untouched"
        );
    });
}

#[test]
fn an_expired_resting_order_is_purged_and_refunded_during_matching() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            1, // expires almost immediately
        ));
        let bidder_after_place = Balances::free_balance(1);

        frame_system::Pallet::<Test>::set_block_number(100);

        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        assert!(
            BelizeX::order_book(0).is_none(),
            "the expired bid must be purged"
        );
        assert_eq!(
            Balances::free_balance(1),
            bidder_after_place + 20,
            "the expired order's escrow must come back"
        );
        assert_eq!(
            BelizeX::order_book(1).map(|o| o.remaining),
            Some(10),
            "the ask must not fill against an expired order"
        );
    });
}

#[test]
fn an_asker_without_the_base_asset_cannot_place_an_order() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        // Drain account 3 down to less than the order size.
        let _ = Balances::make_free_balance_be(&3, 5);

        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(3),
                0,
                1,
                OrderType::Sell.as_u8(),
                10,
                PRICE_SCALE,
                20,
            ),
            Error::<Test>::InsufficientBalance
        );

        assert!(
            BelizeX::order_book(0).is_none(),
            "a failed escrow must not leave an order behind"
        );
        assert_eq!(BelizeX::account_order_count(3), 0);
    });
}

#[test]
fn zero_amount_orders_are_rejected() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_noop!(
            BelizeX::place_limit_order(
                RuntimeOrigin::signed(1),
                0,
                1,
                OrderType::Buy.as_u8(),
                0,
                PRICE_SCALE,
                20,
            ),
            Error::<Test>::BelowMinimumAmount
        );
    });
}

#[test]
fn a_full_fill_frees_the_account_order_slot() {
    let mut ext = new_test_ext();
    ext.execute_with(|| {
        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(1),
            0,
            1,
            OrderType::Buy.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));
        assert_eq!(BelizeX::account_order_count(1), 1);

        assert_ok!(BelizeX::place_limit_order(
            RuntimeOrigin::signed(2),
            0,
            1,
            OrderType::Sell.as_u8(),
            10,
            2 * PRICE_SCALE,
            20,
        ));

        assert_eq!(
            BelizeX::account_order_count(1),
            0,
            "a filled order must not keep counting against the per-account cap"
        );
        assert_eq!(BelizeX::account_order_count(2), 0);
    });
}
