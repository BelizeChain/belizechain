/// Runtime upgrade utilities for forkless blockchain upgrades.
/// Enables seamless pallet updates without hard forks.
///
/// ## Adding a new migration
///
/// 1. Bump `CURRENT_RUNTIME_VERSION` by one.
/// 2. Add a new migration struct (e.g. `MigrateV2ToV3<T>`) implementing
///    `OnRuntimeUpgrade`.
/// 3. Register it inside `CoordinatedUpgrade::on_runtime_upgrade()` behind
///    a version guard: `if on_chain < N { ... }`.
/// 4. Implement `pre_upgrade` / `post_upgrade` for `try-runtime` validation.
use codec::{Decode, Encode};
use frame_support::{
    traits::OnRuntimeUpgrade,
    weights::Weight,
};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use sp_std::prelude::*;

/// Current runtime migration version.
/// Bump this whenever a new migration is added.
const CURRENT_RUNTIME_VERSION: u32 = 1;

/// Migration tracker for coordinated multi-pallet upgrades
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct MigrationStatus {
    pub completed_migrations: u32,
    pub total_migrations: u32,
    pub current_step: MigrationStep,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum MigrationStep {
    NotStarted,
    InProgress(u32), // Step number
    Completed,
    Failed,
}

// MG-1 FIX: Use hashed storage key instead of raw unhashed bytes
// The key is the twox_128 hash of the prefix, giving a proper namespaced key.
const MIGRATION_VERSION_KEY: &[u8] = b"belizechain::migration_version";

/// Read the on-chain migration version.
/// MG-1 FIX: Uses twox_128 hashed key for proper storage namespacing.
fn on_chain_version() -> u32 {
    let key = sp_core::hashing::twox_128(MIGRATION_VERSION_KEY);
    frame_support::storage::unhashed::get::<u32>(&key)
        .unwrap_or(0)
}

/// Write the on-chain migration version.
fn set_on_chain_version(v: u32) {
    let key = sp_core::hashing::twox_128(MIGRATION_VERSION_KEY);
    frame_support::storage::unhashed::put::<u32>(&key, &v);
}

/// Template for pallet storage migrations
/// 
/// Usage:
/// ```rust,ignore
/// pub struct MigrateV1ToV2<T>(PhantomData<T>);
/// 
/// impl<T: Config> OnRuntimeUpgrade for MigrateV1ToV2<T> {
///     fn on_runtime_upgrade() -> Weight {
///         let current = StorageVersion::get::<Pallet<T>>();
///         if current == 1 {
///             // Perform migration logic
///             StorageVersion::new(2).put::<Pallet<T>>();
///             T::DbWeight::get().reads_writes(n, n)
///         } else {
///             Weight::zero()
///         }
///     }
/// }
/// ```
pub struct StorageMigration<T> {
    _phantom: PhantomData<T>,
}

impl<T> StorageMigration<T> {
    /// Check if migration is needed
    pub fn needs_migration(current_version: u16, target_version: u16) -> bool {
        current_version < target_version
    }

    /// Validate migration completed successfully
    #[cfg(feature = "try-runtime")]
    pub fn post_upgrade_check(_expected_version: u16) -> Result<(), &'static str> {
        // Implement version validation
        Ok(())
    }
}

/// Multi-step runtime upgrade coordinator.
///
/// Ensures all pallets upgrade atomically. Each migration is gated behind
/// a version check so it only runs once, even if the on-runtime-upgrade
/// hook fires multiple times (e.g. during `try-runtime`).
pub struct CoordinatedUpgrade<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config> OnRuntimeUpgrade for CoordinatedUpgrade<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain = on_chain_version();
        let mut total_weight = Weight::zero();

        if on_chain >= CURRENT_RUNTIME_VERSION {
            log::info!(
                "✅ Runtime already at migration version {on_chain}, nothing to do."
            );
            return total_weight;
        }

        log::info!(
            "🔄 Starting coordinated runtime upgrade: v{on_chain} → v{CURRENT_RUNTIME_VERSION}"
        );

        // ── V0 → V1: Initial migration (no-op, establishes version tracking) ──
        if on_chain < 1 {
            log::info!("  ↳ Applying migration V0 → V1 (version tracking bootstrap)");
            // Future migrations go here. Example:
            // total_weight = total_weight.saturating_add(
            //     MigrateEconomyV1ToV2::<T>::on_runtime_upgrade()
            // );
            total_weight = total_weight.saturating_add(
                Weight::from_parts(1_000_000, 0) // bookkeeping weight
            );
            // MG-2 FIX: Advance version per-step for rollback safety
            set_on_chain_version(1);
        }

        // ── Add new version gates above this line ──
        // if on_chain < 2 {
        //     log::info!("  ↳ Applying migration V1 → V2 ...");
        //     total_weight = total_weight.saturating_add(MigrateXxx::<T>::on_runtime_upgrade());
        //     set_on_chain_version(2); // MG-2: advance after each step
        // }

        // MG-2 FIX: Version is now advanced per-step above. Only log completion.
        log::info!(
            "✅ Runtime upgrade completed: now at migration version {}",
            on_chain_version()
        );
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        let v = on_chain_version();
        log::info!("Pre-upgrade check: on-chain version = {v}");
        Ok(v.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let pre_version = u32::decode(&mut &state[..])
            .map_err(|_| "Failed to decode pre-upgrade version")?;
        let post_version = on_chain_version();
        log::info!(
            "Post-upgrade check: {pre_version} → {post_version} (target: {CURRENT_RUNTIME_VERSION})"
        );
        assert_eq!(
            post_version, CURRENT_RUNTIME_VERSION,
            "On-chain version mismatch after upgrade"
        );
        Ok(())
    }
}

/// Helper for testing migrations in try-runtime
#[cfg(feature = "try-runtime")]
pub mod test_utils {
    use super::*;

    /// Simulate runtime upgrade without applying changes
    pub fn dry_run_upgrade<T: OnRuntimeUpgrade>() -> Result<Weight, sp_runtime::TryRuntimeError> {
        let pre_state = T::pre_upgrade()?;
        let weight = T::on_runtime_upgrade();
        T::post_upgrade(pre_state)?;
        Ok(weight)
    }
}

/// Example: Economy pallet migration from V1 to V2
/// (Add account type field to existing accounts)
#[cfg(feature = "example-migration")]
pub mod example {
    use super::*;
    
    pub struct MigrateEconomyV1ToV2<T>(PhantomData<T>);
    
    impl<T: frame_system::Config> OnRuntimeUpgrade for MigrateEconomyV1ToV2<T> {
        fn on_runtime_upgrade() -> Weight {
            log::info!("Migrating Economy pallet from V1 to V2...");
            
            // Migration logic here
            // Example: Iterate through all accounts and add new field
            
            Weight::from_parts(10_000_000, 0)
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
            // Validate current state before migration
            Ok(Vec::new())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            // Validate state after migration
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status() {
        let status = MigrationStatus {
            completed_migrations: 2,
            total_migrations: 5,
            current_step: MigrationStep::InProgress(3),
        };
        assert_eq!(status.completed_migrations, 2);
    }

    #[test]
    fn test_needs_migration() {
        assert!(StorageMigration::<()>::needs_migration(1, 2));
        assert!(!StorageMigration::<()>::needs_migration(2, 2));
        assert!(!StorageMigration::<()>::needs_migration(3, 2));
    }
}
