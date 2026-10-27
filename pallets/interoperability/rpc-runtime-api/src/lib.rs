//! Runtime API surface for `pallet-belize-interoperability`.
//!
//! Both queries exist because their storage is unreadable from a client without
//! a full scan: `ChainConfigurations` is a `StorageMap` keyed by `BridgeChain`,
//! and there is no key-resolvable index for "all bridge transactions".

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use pallet_belize_interoperability::{BridgeChain, BridgeTransaction, ChainConfig};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// Read-only access to bridge state.
    pub trait BridgeApi<AccountId, BlockNumber>
    where
        AccountId: Codec,
        BlockNumber: Codec,
    {
        /// Every configured bridge chain with its settings, in unspecified
        /// order.
        ///
        /// This is the authoritative "which chains can this node bridge to"
        /// answer — a chain absent here cannot be bridged to, regardless of what
        /// any client-side list says.
        fn chain_configurations() -> Vec<(BridgeChain, ChainConfig)>;

        /// Every bridge transaction record, in unspecified order.
        ///
        /// Unbounded on a long-lived chain. Callers should treat this as an
        /// administrative/debugging view; a production client should read
        /// `BridgeTransactions` by id from storage instead.
        fn bridge_transactions() -> Vec<BridgeTransaction<AccountId, BlockNumber>>;
    }
}
