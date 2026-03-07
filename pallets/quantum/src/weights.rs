//! Hand-estimated weights for pallet-belize-quantum
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_quantum using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: QuantumJobs (r:1 w:1), NextJobId (r:1 w:1),
    ///          Currency::reserve (r:1 w:1), JobsByCreator (r:0 w:1),
    ///          ActiveJobCount (r:1 w:1)
    fn submit_quantum_job() -> Weight {
        Weight::from_parts(45_000_000, 3072)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    /// Storage: QuantumJobs (r:1 w:1), ActiveJobCount (r:1 w:1)
    fn update_job_status() -> Weight {
        Weight::from_parts(20_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: QuantumJobs (r:1 w:0), QuantumResults (r:1 w:1),
    ///          ResultsByJob (r:0 w:1)
    fn record_quantum_result() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: QuantumResults (r:1 w:1)
    fn verify_quantum_result() -> Weight {
        Weight::from_parts(20_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: NFTs (r:1 w:1), NextNftId (r:1 w:1),
    ///          NFTsByOwner (r:0 w:1)
    fn mint_achievement_nft() -> Weight {
        Weight::from_parts(30_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: NFTs (r:1 w:1), NFTsByOwner (r:0 w:2)
    fn transfer_nft() -> Weight {
        Weight::from_parts(20_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: NFTs (r:1 w:0), Listings (r:1 w:1),
    ///          ActiveAuctions (r:1 w:0), ListingCounter (r:1 w:1)
    fn list_nft() -> Weight {
        Weight::from_parts(25_000_000, 4096)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Listings (r:1 w:1), NFTs (r:1 w:1),
    ///          Currency::transfer (r:2 w:2)
    fn buy_nft() -> Weight {
        Weight::from_parts(45_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: Listings (r:1 w:1)
    fn delist_nft() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: NFTs (r:1 w:1), Listings (r:1 w:0),
    ///          ActiveAuctions (r:1 w:0), BridgeRequests (r:1 w:1),
    ///          NextBridgeRequestId (r:1 w:1)
    fn bridge_to_ethereum() -> Weight {
        Weight::from_parts(40_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: NFTs (r:1 w:1), Listings (r:1 w:0),
    ///          ActiveAuctions (r:1 w:0), BridgeRequests (r:1 w:1),
    ///          NextBridgeRequestId (r:1 w:1)
    fn bridge_to_parachain() -> Weight {
        Weight::from_parts(45_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: QuantumJobs (r:1 w:0), QuantumResults (r:1 w:0),
    ///          VerificationRequests (r:0 w:1)
    fn request_verification() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: VerificationRequests (r:1 w:1), QuantumJobs (r:1 w:1),
    ///          QuantumResults (r:1 w:0), Reputation (r:1 w:1)
    /// O(N) — may iterate verification submissions
    fn submit_verification() -> Weight {
        Weight::from_parts(50_000_000, 3072)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: BridgeRequests (r:1 w:1), NFTs (r:1 w:1)
    fn cancel_bridge() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
