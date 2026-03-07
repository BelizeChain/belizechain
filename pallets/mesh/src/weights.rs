//! Hand-estimated weights for pallet-belize-mesh
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_mesh using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: MeshNodes (r:1 w:1), NodeCount (r:1 w:1),
    ///          Currency::reserve (r:1 w:1)
    fn register_node() -> Weight {
        Weight::from_parts(50_000_000, 3072)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: MeshNodes (r:1 w:1), NodeCount (r:1 w:1),
    ///          Currency::unreserve (r:1 w:1)
    fn deregister_node() -> Weight {
        Weight::from_parts(35_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: MeshNodes (r:1 w:1)
    fn update_node_location() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: MeshNodes (r:1 w:1)
    fn node_heartbeat() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: MeshNodes (r:1 w:0), MeshTransactions (r:1 w:1),
    ///          TxCounter (r:1 w:1), Currency::transfer (r:2 w:2)
    fn submit_mesh_transaction() -> Weight {
        Weight::from_parts(80_000_000, 3584)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: MeshNodes (r:1 w:0), RelayProofs (r:1 w:1)
    fn submit_relay_proof() -> Weight {
        Weight::from_parts(40_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: MeshNodes (r:1 w:0), EmergencyAlerts (r:1 w:1),
    ///          AlertCounter (r:1 w:1)
    fn issue_emergency_alert() -> Weight {
        Weight::from_parts(60_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: EmergencyAlerts (r:1 w:1)
    fn resolve_emergency_alert() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: EmergencyAlerts (r:1 w:1), MeshNodes (r:1 w:0)
    fn confirm_emergency_alert() -> Weight {
        Weight::from_parts(30_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: MeshNodes (r:1 w:0), RelayHeaders (r:1 w:1),
    ///          HeaderCounter (r:1 w:1)
    fn relay_block_header() -> Weight {
        Weight::from_parts(70_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: MeshNodes (r:1 w:0), RelayRewards (r:1 w:1),
    ///          Currency::transfer (r:2 w:2)
    fn claim_relay_rewards() -> Weight {
        Weight::from_parts(50_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: MeshConfig (r:0 w:1)
    fn update_mesh_config() -> Weight {
        Weight::from_parts(15_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: RewardPool (r:1 w:1), Currency::transfer (r:2 w:2)
    fn fund_relay_rewards() -> Weight {
        Weight::from_parts(35_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: RelayProofs (r:1 w:1), MeshNodes (r:1 w:0)
    fn confirm_relay_proof() -> Weight {
        Weight::from_parts(40_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
