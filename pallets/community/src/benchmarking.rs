//! Benchmarking for the community pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn record_participation() {
        let caller: T::AccountId = whitelisted_caller();
        let activity_code: u8 = 0; // VotedOnProposal

        #[extrinsic_call]
        record_participation(RawOrigin::Root, caller.clone(), activity_code);

        // Verify participation was recorded
        assert!(!ParticipationHistory::<T>::get(&caller).is_empty());
    }

    #[benchmark]
    fn update_srs() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 0, 0);
        
        // Ensure target has some participation history for the update
        let activity = ParticipationRecord {
            activity_type: ActivityType::VoteCast,
            block_number: frame_system::Pallet::<T>::block_number(),
            value: 100u32,
        };
        ParticipationHistory::<T>::insert(&target, BoundedVec::try_from(vec![activity]).unwrap());

        #[extrinsic_call]
        update_srs(RawOrigin::Signed(caller.clone()), target.clone());

        // Verify SRS was updated
        assert!(SocialResponsibilityScores::<T>::get(&target).is_some());
    }

    #[benchmark]
    fn endorse_peer() {
        let endorser: T::AccountId = whitelisted_caller();
        let endorsee: T::AccountId = account("endorsee", 0, 0);
        let skill_code: u8 = 0; // Leadership

        #[extrinsic_call]
        endorse_peer(RawOrigin::Signed(endorser.clone()), endorsee.clone(), skill_code);

        // Verify endorsement count increased
        assert!(PeerEndorsements::<T>::get(&endorsee) > 0);
    }

    #[benchmark]
    fn set_srs_privacy() {
        let caller: T::AccountId = whitelisted_caller();
        // First ensure the caller has an SRS record
        let initial_score = SRSData {
            score: 1000u32,
            governance_score: 500u32,
            education_score: 200u32,
            sustainability_score: 200u32,
            participation_score: 100u32,
            peer_endorsements: 0u32,
            honesty_rating: 0u32,
            last_updated: frame_system::Pallet::<T>::block_number(),
            tier: SRSTier::Bronze,
            total_contributions: 0u32,
            public_display: false,
            anonymous_hash: None,
        };
        SocialResponsibilityScores::<T>::insert(&caller, initial_score);
        
        let is_public: bool = true;

        #[extrinsic_call]
        set_srs_privacy(RawOrigin::Signed(caller.clone()), is_public);

        // Verify privacy setting was updated
        assert_eq!(SocialResponsibilityScores::<T>::get(&caller).unwrap().public_display, is_public);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
