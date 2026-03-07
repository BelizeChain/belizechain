//! Weight functions for the community pallet
//! 
//! These are estimated weights based on complexity analysis.
//! For production, run benchmarks to generate accurate weights:
//! 
//! ```bash
//! cargo build --release --features runtime-benchmarks
//! ./target/release/belizechain-node benchmark pallet \
//!     --chain=dev \
//!     --pallet=pallet_belize_community \
//!     --extrinsic='*' \
//!     --steps=50 \
//!     --repeat=20 \
//!     --output=./pallets/community/src/weights.rs
//! ```

use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions needed for pallet_belize_community.
pub trait WeightInfo {
    fn record_participation() -> Weight;
    fn update_srs() -> Weight;
    fn endorse_peer() -> Weight;
    fn set_srs_privacy() -> Weight;
    fn submit_community_proposal() -> Weight;
    fn vote_community_proposal() -> Weight;
    fn finalize_community_proposal() -> Weight;
    fn sanction_account() -> Weight;
    fn lift_sanction() -> Weight;
    fn ethics_council_vote() -> Weight;
    fn complete_education_module() -> Weight;
    fn contribute_to_green_project() -> Weight;
    fn claim_referral_reward() -> Weight;
    fn attest_participation() -> Weight;
}

/// Weights for pallet_belize_community using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: SRSScores (r:1 w:1)
    /// Storage: ParticipationHistory (r:1 w:1)
    /// Storage: Identity (r:1 w:0)
    fn record_participation() -> Weight {
        Weight::from_parts(45_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: SRSScores (r:1 w:1)
    /// Storage: ParticipationHistory (r:1 w:0)
    /// Storage: Endorsements (r:1 w:0)
    /// Storage: EducationRecords (r:1 w:0)
    /// Storage: GreenProjects (r:1 w:0)
    fn update_srs() -> Weight {
        Weight::from_parts(85_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    /// Storage: SRSScores (r:2 w:1)
    /// Storage: Endorsements (r:1 w:1)
    /// Storage: Identity (r:1 w:0)
    fn endorse_peer() -> Weight {
        Weight::from_parts(55_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: SRSScores (r:1 w:1)
    fn set_srs_privacy() -> Weight {
        Weight::from_parts(30_000_000, 512)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    /// Storage: SRSScores (r:1 w:0)
    /// Storage: Proposals (r:1 w:1)
    /// Storage: ProposalCount (r:1 w:1)
    /// Storage: Balances (r:1 w:1)
    /// Storage: Identity (r:1 w:0)
    fn submit_community_proposal() -> Weight {
        Weight::from_parts(75_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: Votes (r:1 w:1)
    /// Storage: SRSScores (r:1 w:0)
    fn vote_community_proposal() -> Weight {
        Weight::from_parts(50_000_000, 1536)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: Balances (r:2 w:2)
    /// Storage: Treasury (r:1 w:1)
    fn finalize_community_proposal() -> Weight {
        Weight::from_parts(90_000_000, 3072)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    /// Storage: Sanctions (r:1 w:1)
    /// Storage: SRSScores (r:1 w:1)
    fn sanction_account() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: Sanctions (r:1 w:1)
    /// Storage: SRSScores (r:1 w:1)
    fn lift_sanction() -> Weight {
        Weight::from_parts(40_000_000, 2048)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: EthicsVotes (r:1 w:1)
    /// Storage: Proposals (r:1 w:1)
    fn ethics_council_vote() -> Weight {
        Weight::from_parts(45_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: EducationModules (r:1 w:1)
    /// Storage: SRSScores (r:1 w:1)
    /// Storage: Balances (r:1 w:1)
    fn complete_education_module() -> Weight {
        Weight::from_parts(65_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    /// Storage: GreenProjects (r:1 w:1)
    /// Storage: SRSScores (r:1 w:1)
    fn contribute_to_green_project() -> Weight {
        Weight::from_parts(55_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: Referrals (r:1 w:1)
    /// Storage: Balances (r:2 w:2)
    /// Storage: SRSScores (r:2 w:2)
    fn claim_referral_reward() -> Weight {
        Weight::from_parts(70_000_000, 2560)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    /// Storage: PendingAttestations (r:1 w:1)  Storage: AttestedActivities (r:1 w:1)
    fn attest_participation() -> Weight {
        Weight::from_parts(25_000_000, 1024)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}

/// Simplified weight implementation for testing and light clients.
impl WeightInfo for () {
    fn record_participation() -> Weight {
        Weight::from_parts(45_000_000, 512)
    }
    fn update_srs() -> Weight {
        Weight::from_parts(85_000_000, 512)
    }
    fn endorse_peer() -> Weight {
        Weight::from_parts(55_000_000, 512)
    }
    fn set_srs_privacy() -> Weight {
        Weight::from_parts(30_000_000, 512)
    }
    fn submit_community_proposal() -> Weight {
        Weight::from_parts(75_000_000, 512)
    }
    fn vote_community_proposal() -> Weight {
        Weight::from_parts(50_000_000, 512)
    }
    fn finalize_community_proposal() -> Weight {
        Weight::from_parts(90_000_000, 512)
    }
    fn sanction_account() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
    fn lift_sanction() -> Weight {
        Weight::from_parts(40_000_000, 512)
    }
    fn ethics_council_vote() -> Weight {
        Weight::from_parts(45_000_000, 512)
    }
    fn complete_education_module() -> Weight {
        Weight::from_parts(65_000_000, 512)
    }
    fn contribute_to_green_project() -> Weight {
        Weight::from_parts(55_000_000, 512)
    }
    fn claim_referral_reward() -> Weight {
        Weight::from_parts(70_000_000, 512)
    }
    fn attest_participation() -> Weight {
        Weight::from_parts(25_000_000, 512)
    }
}
