//! Benchmarking for pallet-belize-exchange (BelizeX)
//!
//! Uses `frame_benchmarking::v2` API.
//! PairListingOrigin and TourismOrigin = EnsureRoot in runtime → `RawOrigin::Root`.
//! KYC-gated calls (add_liquidity, execute_trade, place_limit_order) need
//! KYC level; benchmark mode should auto-pass or we rely on the default
//! identity provider returning a sufficient level.

use super::*;
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};

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

/// Helper: fund `who` with `amount` of `asset` through the ledger seam.
///
/// The two assets in a pair live in different ledgers in the real runtime
/// (`Balances` vs `pallet_belize_economy::BBZDBalances`), so funding only the
/// native balance leaves the quote leg empty and the extrinsic fails with
/// `InsufficientBalance`.
fn fund<T: Config>(asset: AssetId, who: &T::AccountId, amount: u128) {
    T::Assets::set_balance(asset, who, amount);
}

/// `whitelisted_caller()` with real KYC L1 state written.
///
/// Trading and liquidity provision are KYC-gated. Seeding the storage the gate
/// actually reads keeps it on the measured path; a permissive shim would not.
fn kyc_caller<T: Config>() -> T::AccountId {
    let caller: T::AccountId = whitelisted_caller();
    T::seed_bench_kyc(&caller, 1);
    caller
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
        let caller = kyc_caller::<T>();
        let deposit = 100_000_000_000_000u128; // 100k
        fund::<T>(AssetId::DALLA, &caller, deposit * 10);
        fund::<T>(AssetId::BBZD, &caller, deposit * 10);

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
        let caller = kyc_caller::<T>();
        let trade_amount = 1_000_000_000_000u128; // 1k
        fund::<T>(AssetId::DALLA, &caller, trade_amount * 10);
        // The pool pays the trader out of its quote reserve, so it must hold
        // bBZD — the reserves in `TradingPairs` are bookkeeping, not custody.
        fund::<T>(
            AssetId::BBZD,
            &Pallet::<T>::pool_account(),
            trade_amount * 10,
        );

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
        let caller = kyc_caller::<T>();
        let order_amount = 500_000_000_000u128;
        // A bid escrows the **quote** asset (bBZD), not the base, so funding
        // only the native balance would leave the escrow short in the runtime
        // even though a single-ledger mock would let it through.
        fund::<T>(AssetId::BBZD, &caller, order_amount * 10);

        #[extrinsic_call]
        place_limit_order(
            RawOrigin::Signed(caller),
            0u8, // base: DALLA
            1u8, // quote: BBZD
            0u8, // order_type: 0 = Buy
            order_amount,
            // Price is scaled by `PRICE_SCALE`, so 2 * PRICE_SCALE is "2 bBZD
            // per DALLA". A raw 1e12 would mean 1e6 per unit and the escrow
            // (1e17 quote) would dwarf any balance this benchmark can fund.
            2 * PRICE_SCALE,
            1000u32.into(), // expires_in_blocks
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
        let caller = kyc_caller::<T>();
        let lp_amount = 100_000_000_000u128;

        // Give the caller LP tokens
        LPBalances::<T>::insert(&caller, (0u8, 1u8), lp_amount);

        // Fund the pool to cover the withdrawal — both legs, since the pool
        // refunds base *and* quote.
        let pool = Pallet::<T>::pool_account();
        fund::<T>(AssetId::DALLA, &pool, lp_amount * 100);
        fund::<T>(AssetId::BBZD, &pool, lp_amount * 100);
        fund::<T>(AssetId::DALLA, &caller, lp_amount * 10);
        fund::<T>(AssetId::BBZD, &caller, lp_amount * 10);

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

    // ───────────────────────────────────────────
    // 10. issue_wusdc — Root (PairListingOrigin)
    // ───────────────────────────────────────────
    #[benchmark]
    fn issue_wusdc() {
        let target: T::AccountId = account("wusdc_holder", 0, 0);
        let amount = 1_000_000_000_000u128;

        #[extrinsic_call]
        issue_wusdc(RawOrigin::Root, target.clone(), amount);

        assert_eq!(WusdcBalances::<T>::get(&target), amount);
    }

    // ───────────────────────────────────────────
    // 11. burn_wusdc — signed, burns the caller's own balance
    // ───────────────────────────────────────────
    #[benchmark]
    fn burn_wusdc() {
        let caller: T::AccountId = whitelisted_caller();
        let amount = 1_000_000_000_000u128;
        WusdcBalances::<T>::insert(&caller, amount);

        #[extrinsic_call]
        burn_wusdc(RawOrigin::Signed(caller.clone()), amount);

        assert_eq!(WusdcBalances::<T>::get(&caller), 0);
    }

    // ───────────────────────────────────────
    // 10. cancel_order → cancel_order
    //     WeightInfo fn: cancel_order()
    //     Creator-only, so the order is seeded straight into the orderbook
    //     rather than placed through the trading path.
    // ───────────────────────────────────────────
    #[benchmark]
    fn cancel_order() {
        let caller = kyc_caller::<T>();
        let order_id: u32 = 0;
        let escrowed: u128 = 500_000_000_000u128;
        // The refund moves whichever asset the order escrowed — quote (bBZD)
        // for a bid — so the escrow account must hold it in the real runtime's
        // ledger, not just the native balance.
        fund::<T>(
            AssetId::BBZD,
            &Pallet::<T>::order_escrow_account(),
            escrowed,
        );
        OrderBook::<T>::insert(
            order_id,
            OrderEntry::<T::AccountId, BlockNumberFor<T>> {
                order_id,
                creator: caller.clone(),
                pair: (AssetId::from(0u8), AssetId::from(1u8)),
                order_type: OrderType::Buy,
                amount: 500_000_000_000u128,
                price: 2 * PRICE_SCALE,
                remaining: 500_000_000_000u128,
                escrow: escrowed,
                expires_at: 1_000u32.into(),
                is_tourism_order: false,
            },
        );
        AccountOrderCount::<T>::insert(&caller, 1u32);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id);
    }

    // ───────────────────────────────────────
    // 11. match_orders → the fill loop inside `place_limit_order`
    //     WeightInfo fn: match_orders(fills)
    //     Not a dispatchable: this measures the bounded book walk directly, so
    //     `place_limit_order`'s weight is `place_order() + match_orders(N)`.
    // ───────────────────────────────────────
    #[benchmark]
    fn match_orders(n: Linear<1, { T::MaxOrdersPerPair::get() }>) {
        setup_trading_pair_with_liquidity::<T>(0, 1);

        let escrow_account = Pallet::<T>::order_escrow_account();
        // Settlement pays the bidder base (DALLA) and the asker quote (bBZD)
        // out of escrow, so both ledgers must be funded or `settle_fill` bails
        // and the walk is measured without its settlement cost.
        fund::<T>(
            AssetId::DALLA,
            &escrow_account,
            n as u128 * 1_000_000_000u128,
        );
        fund::<T>(
            AssetId::BBZD,
            &escrow_account,
            n as u128 * 2_000_000_000u128,
        );

        // `n` resting asks, all at the same price so the taker must consume the
        // whole level — the worst case for the walk.
        let maker: T::AccountId = account("maker", 0, SEED);
        for i in 0..n {
            let order_id = i;
            OrderBook::<T>::insert(
                order_id,
                OrderEntry::<T::AccountId, BlockNumberFor<T>> {
                    order_id,
                    creator: maker.clone(),
                    pair: (AssetId::from(0u8), AssetId::from(1u8)),
                    order_type: OrderType::Sell,
                    amount: 1_000_000_000u128,
                    price: PRICE_SCALE,
                    remaining: 1_000_000_000u128,
                    escrow: 1_000_000_000u128,
                    expires_at: 1_000_000u32.into(),
                    is_tourism_order: false,
                },
            );
            PairOrders::<T>::mutate((0u8, 1u8), |book| {
                let _ = book.asks.try_push(order_id);
            });
        }

        // One bid big enough to cross every ask, seeded directly so this
        // benchmark measures the walk rather than the placement path.
        let taker: T::AccountId = whitelisted_caller();
        let taker_id: u32 = n;
        OrderBook::<T>::insert(
            taker_id,
            OrderEntry::<T::AccountId, BlockNumberFor<T>> {
                order_id: taker_id,
                creator: taker.clone(),
                pair: (AssetId::from(0u8), AssetId::from(1u8)),
                order_type: OrderType::Buy,
                amount: n as u128 * 1_000_000_000u128,
                price: 2 * PRICE_SCALE,
                remaining: n as u128 * 1_000_000_000u128,
                escrow: n as u128 * 2_000_000_000u128,
                expires_at: 1_000_000u32.into(),
                is_tourism_order: false,
            },
        );

        #[block]
        {
            Pallet::<T>::match_order((0u8, 1u8), taker_id);
        }
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
