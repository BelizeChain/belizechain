//! Runtime API surface for `pallet-belize-oracle`.
//!
//! `PriceFeeds` is a map keyed by `CurrencyPair`, so "what is every rate right
//! now" is a full iteration only the runtime can do without leaking storage
//! layout to every client.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use pallet_belize_oracle::types::{CurrencyPair, PriceFeedData};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// Read-only access to oracle state.
    pub trait OracleApi<BlockNumber>
    where
        BlockNumber: Codec,
    {
        /// Every aggregated price feed, in unspecified order.
        ///
        /// A pair absent here has no rate — `BelizeXOracle` rejects trades
        /// against it rather than assuming a default, and a client should
        /// present the same answer.
        fn price_feeds() -> Vec<PriceFeedData<BlockNumber>>;

        /// Administrator-set fallback rates as `(pair, price, last_update)`,
        /// scaled by 1e6.
        ///
        /// Distinct from `price_feeds`: these bypass the operator consensus path
        /// and are what the DEX reads when no aggregated feed exists.
        fn manual_exchange_rates() -> Vec<(CurrencyPair, u128, BlockNumber)>;
    }
}
