//! Benchmarking for pallet-belize-exchange (BelizeX)
//!
//! Uses `frame_benchmarking::v2` API.
//! PairListingOrigin and TourismOrigin = EnsureRoot in runtime → `RawOrigin::Root`.
//! KYC-gated calls (add_liquidity, execute_trade, place_limit_order) need
//! KYC level; benchmark mode should auto-pass or we rely on the default
//! identity provider returning a sufficient level.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;

const SEED: u32 = 0;

/// Helper: create a trading pair via Root (PairListingOrigin = Root).
fn setup_trading_pair<T: Config>(base: u8, quote: u8) {
    let pair = TradingPair {
        base_asset: AssetId::from(base),
        quote_asset: AssetId::from(quote),
        base_reserve: 0u128,
        quote_reserve: 0u128,
        total_lp_tokens: 0u128,
        fee_rate: 30, // 0.3%
        active: true,
    };
    TradingPairs::<T>::insert((base, quote), pair);
}

/// Helper: create a trading pair with reserves for trade/order benchmarks.
fn setup_trading_pair_with_liquidity<T: Config>(base: u8, quote: u8) {
    let pair = TradingPair {
        base_asset: AssetId::from(base),
        quote_asset: AssetId::from(quote),
        base_reserve: 1_000_000_000_000_000u128,  // 1M
        quote_reserve: 1_000_000_000_000_000u128, // 1M
        total_lp_tokens: 1_000_000_000_000_000u128,
        fee_rate: 30,
        active: true,
    };
    TradingPairs::<T>::insert((base, quote), pair);
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ───────────────────────────────────────────
    // 1. create_pair → create_trading_pair
    //    WeightInfo fn: create_pair()
    //    Origin: PairListingOrigin = Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn create_pair() {
        // Use (0,2) = DALLA/TourismDALLA which is NOT created by genesis
        // (genesis creates (0,1), (2,1), (3,1) via enable_default_pairs)
        #[extrinsic_call]
        create_trading_pair(RawOrigin::Root, 0u8, 2u8, 30u32);
    }

    // ───────────────────────────────────────────
    // 2. add_liquidity — signed, needs existing pair + KYC
    // ───────────────────────────────────────────
    #[benchmark]
    fn add_liquidity() {
        setup_trading_pair::<T>(0, 1);
        let caller: T::AccountId = whitelisted_caller();
        let deposit = 100_000_000_000_000u128; // 100k
        T::Currency::make_free_balance_be(&caller, (deposit * 10).saturated_into());

        #[extrinsic_call]
        add_liquidity(
            RawOrigin::Signed(caller),
            0u8, // base: DALLA
            1u8, // quote: BBZD
            deposit.saturated_into(),
            deposit.saturated_into(),
            0u128, // min_lp_tokens
        );
    }

    // ───────────────────────────────────────────
    // 3. execute_trade — signed, needs pair with reserves + KYC
    // ───────────────────────────────────────────
    #[benchmark]
    fn execute_trade() {
        setup_trading_pair_with_liquidity::<T>(0, 1);
        let caller: T::AccountId = whitelisted_caller();
        let trade_amount = 1_000_000_000_000u128; // 1k
        T::Currency::make_free_balance_be(&caller, (trade_amount * 10).saturated_into());

        #[extrinsic_call]
        execute_trade(
            RawOrigin::Signed(caller),
            0u8, // base: DALLA
            1u8, // quote: BBZD
            trade_amount.saturated_into(),
            0u128, // min_amount_out
            false, // is_tourism_trade
        );
    }

    // ───────────────────────────────────────────
    // 4. register_tourism_trader — Root (TourismOrigin)
    // ───────────────────────────────────────────
    #[benchmark]
    fn register_tourism_trader() {
        let trader: T::AccountId = account("trader", 0, SEED);

        #[extrinsic_call]
        register_tourism_trader(RawOrigin::Root, trader);
    }

    // ───────────────────────────────────────────
    // 5. place_order → place_limit_order
    //    WeightInfo fn: place_order()
    //    Extrinsic name: place_limit_order
    // ───────────────────────────────────────────
    #[benchmark]
    fn place_order() {
        setup_trading_pair_with_liquidity::<T>(0, 1);
        let caller: T::AccountId = whitelisted_caller();
        let order_amount = 500_000_000_000u128;
        T::Currency::make_free_balance_be(&caller, (order_amount * 10).saturated_into());

        #[extrinsic_call]
        place_limit_order(
            RawOrigin::Signed(caller),
            0u8, // base: DALLA
            1u8, // quote: BBZD
            0u8, // order_type: 0 = Buy
            order_amount,
            1_000_000_000_000u128, // price
            1000u32.into(),        // expires_in_blocks
        );
    }

    // ───────────────────────────────────────────
    // 6. pause → pause_global
    //    WeightInfo fn: pause()
    //    Origin: PairListingOrigin = Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn pause() {
        #[extrinsic_call]
        pause_global(RawOrigin::Root);
    }

    // ───────────────────────────────────────────
    // 7. resume → resume_global
    //    WeightInfo fn: resume()
    //    Origin: PairListingOrigin = Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn resume() {
        // Pause first so resume does work
        GlobalPaused::<T>::put(true);

        #[extrinsic_call]
        resume_global(RawOrigin::Root);
    }

    // ───────────────────────────────────────────
    // 8. set_pair_status — Root (PairListingOrigin)
    // ───────────────────────────────────────────
    #[benchmark]
    fn set_pair_status() {
        setup_trading_pair::<T>(0, 1);

        #[extrinsic_call]
        set_pair_status(RawOrigin::Root, 0u8, 1u8, false);
    }

    // ───────────────────────────────────────────
    // 9. remove_liquidity — signed, needs existing pair + LP tokens
    // ───────────────────────────────────────────
    #[benchmark]
    fn remove_liquidity() {
        setup_trading_pair_with_liquidity::<T>(0, 1);
        let caller: T::AccountId = whitelisted_caller();
        let lp_amount = 100_000_000_000u128;

        // Give the caller LP tokens
        LPBalances::<T>::insert(&caller, (0u8, 1u8), lp_amount);

        // Fund the pool to cover the withdrawal
        let pool = Pallet::<T>::pool_account();
        T::Currency::make_free_balance_be(&pool, (lp_amount * 100).saturated_into());
        T::Currency::make_free_balance_be(&caller, (lp_amount * 10).saturated_into());

        #[extrinsic_call]
        remove_liquidity(
            RawOrigin::Signed(caller),
            0u8, // base: DALLA
            1u8, // quote: BBZD
            lp_amount,
            0u128, // min_base_amount
            0u128, // min_quote_amount
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
