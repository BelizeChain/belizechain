//! Hand-estimated weights for pallet-belize-governance
//!
//! THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.
//! For production, run `frame-benchmarking` to generate accurate weights.

use frame_support::weights::Weight;
use sp_runtime::traits::Get;

use super::WeightInfo;

/// Weights for pallet_belize_governance using estimated DB operations.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: Proposals (r:0 w:1), NextProposalId (r:1 w:1),
    ///          CommunityRank (r:1 w:0), Currency::reserve (r:1 w:1)
    fn submit_proposal() -> Weight {
        Weight::from_parts(40_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: Proposals (r:1 w:1), Votes (r:1 w:1),
    ///          CommunityRank (r:1 w:0), PoUWContribution (r:1 w:0)
    fn cast_vote() -> Weight {
        Weight::from_parts(30_000_000, 2048)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Proposals (r:1 w:1), Votes iteration (r:N w:0),
    ///          Currency::unreserve (r:1 w:1)
    /// O(N) — iterates voters
    fn finalize_proposal() -> Weight {
        Weight::from_parts(60_000_000, 2560)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: CommunityRank (r:0 w:1)
    fn update_community_rank() -> Weight {
        Weight::from_parts(10_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: PoUWContribution (r:0 w:1)
    fn update_pouw_contribution() -> Weight {
        Weight::from_parts(10_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: Proposals (r:1 w:1), CouncilMembers (r:1 w:0)
    fn council_override() -> Weight {
        Weight::from_parts(25_000_000, 1536)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ChainParameters (r:0 w:1)
    fn update_chain_parameter() -> Weight {
        Weight::from_parts(15_000_000, 512)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: TermExpiryQueue (r:1 w:1), CouncilMembers (r:0 w:1)
    /// Called from on_initialize — per-member cost capped at 10 per block.
    fn expire_council_member() -> Weight {
        Weight::from_parts(12_000_000, 1024)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
