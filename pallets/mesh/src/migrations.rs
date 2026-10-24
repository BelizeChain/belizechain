//! Storage migrations for `pallet-belize-mesh`.
//!
//! ## v0 → v1: signature type change (P1-07)
//!
//! `MeshTransaction.signature` changed from `H256` (32 bytes) to `[u8; 64]`
//! (64 bytes) when mesh signature verification replaced "accept any non-zero
//! hash". Entries written under the old encoding cannot be decoded as the new
//! type, and — more importantly — the old code verified nothing, so every
//! pre-v1 transaction was accepted without a real signature check.
//!
//! Both facts point the same way: the queue is unreadable and untrustworthy, so
//! this migration purges it and resets the counter that bounds it.
//!
//! `ProcessedMeshTransactions` is deliberately **kept**: it stores only a
//! `BlockNumber` per hash, so its encoding did not change, and keeping it
//! preserves replay protection for transactions that were already bridged.
//!
//! ⚠️ This migration was added on 2026-10-23 because the pallet's
//! `STORAGE_VERSION` had been left at 1 since `7c3a678` with no migration
//! registered. The runtime invariant test
//! (`every_belize_pallet_declares_matching_storage_version`) had been failing
//! since that commit.

use crate::{Config, PendingMeshTransactions, PendingMeshTxCount};
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

/// Purge pending mesh transactions written under the pre-verification encoding.
pub struct MigrateV0ToV1<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        if on_chain >= StorageVersion::new(TARGET_VERSION) {
            log::info!(
                "mesh: already at storage v{:?}, skipping signature migration",
                on_chain
            );
            return T::DbWeight::get().reads(1);
        }

        let removed = PendingMeshTransactions::<T>::clear(u32::MAX, None);
        PendingMeshTxCount::<T>::put(0u32);

        StorageVersion::new(TARGET_VERSION).put::<crate::Pallet<T>>();

        log::info!(
            "mesh: purged {} unverifiable pending transactions for signature migration",
            removed.backend
        );

        T::DbWeight::get().reads_writes(1, u64::from(removed.backend).saturating_add(2))
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        let pending = PendingMeshTransactions::<T>::iter().count() as u32;
        log::info!(
            "mesh pre_upgrade: storage v{:?}, {pending} pending transactions to purge",
            on_chain
        );
        Ok((on_chain, pending).encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
        let (_pre_version, pre_pending): (StorageVersion, u32) =
            Decode::decode(&mut &state[..]).map_err(|_| "mesh: undecodable state")?;

        let on_chain = StorageVersion::get::<crate::Pallet<T>>();
        if on_chain != StorageVersion::new(TARGET_VERSION) {
            return Err("mesh: storage version not advanced to v1".into());
        }
        if PendingMeshTransactions::<T>::iter().next().is_some() {
            return Err("mesh: pending queue not empty after migration".into());
        }
        if PendingMeshTxCount::<T>::get() != 0 {
            return Err("mesh: pending counter not reset".into());
        }

        log::info!("mesh post_upgrade: purged {pre_pending} pending transactions");
        Ok(())
    }
}
