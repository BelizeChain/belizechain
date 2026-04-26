//! Hand-estimated weights for pallet-belize-belizex
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_belizex using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: TradingPairs (r:1 w:1), NextPairId (r:1 w:1)
    fn create_pair() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: TradingPairs (r:1 w:1), LiquidityProviders (r:1 w:1),
    ///          Currency::transfer (r:2 w:2)
    fn add_liquidity() -> Weight {
        Weight::from_parts(45_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: TradingPairs (r:1 w:1), Orderbook (r:1 w:1),
    ///          Currency::transfer (r:2 w:2), TradeHistory (r:0 w:1)
    fn execute_trade() -> Weight {
        Weight::from_parts(55_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    /// Storage: TourismTraders (r:0 w:1)
    fn register_tourism_trader() -> Weight {
        Weight::from_parts(15_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: TradingPairs (r:1 w:0), Orderbook (r:1 w:1),
    ///          Currency::reserve (r:1 w:1)
    fn place_order() -> Weight {
        Weight::from_parts(35_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: OrderBook (r:1 w:1), ExchangePaused (r:1 w:0)
    fn cancel_order() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ExchangePaused (r:0 w:1)
    fn pause() -> Weight {
        Weight::from_parts(5_000_000, 512).saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ExchangePaused (r:0 w:1)
    fn resume() -> Weight {
        Weight::from_parts(5_000_000, 512).saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: TradingPairs (r:1 w:1)
    fn set_pair_status() -> Weight {
        Weight::from_parts(12_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: TradingPairs (r:1 w:1), LPBalances (r:1 w:1),
    ///          LiquidityProviders (r:1 w:1), Currency::transfer (r:2 w:2)
    fn remove_liquidity() -> Weight {
        Weight::from_parts(50_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }
}
