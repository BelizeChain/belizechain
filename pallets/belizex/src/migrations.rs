//! Storage migrations for `pallet-belize-belizex`.
//!
//! ## v0 → v1: order escrow (P1-08)
//!
//! `OrderEntry` gained an `escrow` field. Orders written before v1 hold no
//! escrowed funds, so they cannot be settled: matching one would pay out assets
//! the pallet never received. Rather than leave a book of unfundable orders that
//! the matching engine would have to defend against forever, this migration
//! purges the order book and its index.
//!
//! Measured on the live chain before writing this: `OrderBook`, `PairOrders` and
//! `AccountOrderCount` were **all empty** and `NextOrderId` was unset, so this
//! destroyed nothing. The purge is kept anyway because a chain that did have
//! orders (a dev node, a future testnet reset from an older spec) would
//! otherwise carry unbacked orders into a matching engine.
//!
//! No refund is attempted — there is nothing to refund, by construction of the
//! old code, which never transferred funds at placement.

use crate::{AccountOrderCount, Config, OrderBook, PairOrders};
use frame_support::{
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use {
    codec::{Decode, Encode},
    sp_runtime::TryRuntimeError,
    sp_std::vec::Vec,
};

/// Target storage version after this migration.
pub const TARGET_VERSION: u16 = 1;

/// Purge pre-escrow limit orders.
pub struct MigrateV0ToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        if on_chain >= StorageVersion::new(TARGET_VERSION) {
            log::info!(
                "belizex: already at storage v{:?}, skipping escrow migration",
                on_chain
            );
            return T::DbWeight::get().reads(1);
        }

        let (orders, pairs, counts) = (
            OrderBook::<T>::clear(u32::MAX, None),
            PairOrders::<T>::clear(u32::MAX, None),
            AccountOrderCount::<T>::clear(u32::MAX, None),
        );

        StorageVersion::new(TARGET_VERSION).put::<crate::Pallet<T>>();

        log::info!(
            "belizex: purged {} unbacked orders, {} pair indexes, {} counters for escrow migration",
            orders.backend,
            pairs.backend,
            counts.backend
        );

        // One read for the version, then a read+write per removed entry.
        let removed = u64::from(orders.backend)
            .saturating_add(u64::from(pairs.backend))
            .saturating_add(u64::from(counts.backend));
        T::DbWeight::get().reads_writes(1, removed.saturating_add(1))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        let orders = OrderBook::<T>::iter().count() as u32;
        log::info!(
            "belizex pre_upgrade: storage v{:?}, {orders} orders to purge",
            on_chain
        );
        Ok((on_chain, orders).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let (_pre_version, pre_orders): (StorageVersion, u32) =
            Decode::decode(&mut &state[..]).map_err(|_| "belizex: undecodable state")?;

        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        if on_chain != StorageVersion::new(TARGET_VERSION) {
            return Err("belizex: storage version not advanced to v1".into());
        }

        let remaining = OrderBook::<T>::iter().count() as u32;
        if remaining != 0 {
            return Err("belizex: order book not empty after migration".into());
        }

        log::info!("belizex post_upgrade: purged {pre_orders} orders, book is empty");
        Ok(())
    }
}
