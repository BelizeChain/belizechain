//! Benchmarking for the governance pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::vec::Vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_proposal() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::MinimumDeposit::get() * 10u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        let title: Vec<u8> = b"Benchmark Proposal Title".to_vec();
        let description: Vec<u8> =
            b"Benchmark proposal description for testing weight calculation".to_vec();
        let proposal_type_index: u8 = 0; // Constitutional
        let threshold_index: u8 = 0; // SimpleMajority
        let is_emergency: bool = false;
        let district_index: Option<u8> = None;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            title,
            description,
            proposal_type_index,
            threshold_index,
            is_emergency,
            district_index,
        );
    }

    #[benchmark]
    fn cast_vote() {
        let proposer: T::AccountId = whitelisted_caller();
        let balance = T::MinimumDeposit::get() * 10u32.into();
        let _ = T::Currency::make_free_balance_be(&proposer, balance);

        // Submit a proposal first
        let _ = Pallet::<T>::submit_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            b"Vote Test Proposal".to_vec(),
            b"Testing vote casting".to_vec(),
            0u8,
            0u8,
            false,
            None,
        );

        // Advance block to voting_start so VotingPeriodNotStarted check passes
        let proposal = Proposals::<T>::get(0u32).expect("proposal exists");
        frame_system::Pallet::<T>::set_block_number(proposal.voting_start);

        let voter: T::AccountId = account("voter", 0, 0);
        let voter_balance = T::MinimumDeposit::get() * 10u32.into();
        let _ = T::Currency::make_free_balance_be(&voter, voter_balance);

        let proposal_id: u32 = 0;
        let vote_choice_index: u8 = 0; // Approve
        let conviction: u8 = 1;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(voter),
            proposal_id,
            vote_choice_index,
            conviction,
        );
    }

    #[benchmark]
    fn finalize_proposal() {
        let proposer: T::AccountId = whitelisted_caller();
        let balance = T::MinimumDeposit::get() * 10u32.into();
        let _ = T::Currency::make_free_balance_be(&proposer, balance);

        // Submit a proposal
        let _ = Pallet::<T>::submit_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            b"Finalize Test Proposal".to_vec(),
            b"Testing proposal finalization".to_vec(),
            0u8,
            0u8,
            false,
            None,
        );

        // Advance block past voting_end so finalization is allowed
        let proposal = Proposals::<T>::get(0u32).expect("proposal exists");
        frame_system::Pallet::<T>::set_block_number(proposal.voting_end + 1u32.into());

        let proposal_id: u32 = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(proposer), proposal_id);
    }

    #[benchmark]
    fn update_community_rank() {
        let target: T::AccountId = account("target", 0, 0);
        let new_rank: u32 = 5;

        #[extrinsic_call]
        _(RawOrigin::Root, target, new_rank);
    }

    #[benchmark]
    fn update_pouw_contribution() {
        let target: T::AccountId = account("target", 0, 0);
        let new_contribution: u32 = 100;

        #[extrinsic_call]
        _(RawOrigin::Root, target, new_contribution);
    }

    #[benchmark]
    fn council_override() {
        let proposer: T::AccountId = whitelisted_caller();
        let balance = T::MinimumDeposit::get() * 10u32.into();
        let _ = T::Currency::make_free_balance_be(&proposer, balance);

        // Submit a proposal
        let _ = Pallet::<T>::submit_proposal(
            RawOrigin::Signed(proposer.clone()).into(),
            b"Override Test Proposal".to_vec(),
            b"Testing council override".to_vec(),
            0u8,
            0u8,
            true, // is_emergency must be true for council_override
            None,
        );

        // P0-18: council_override requires JaguarMode to be active
        JaguarMode::<T>::put(EmergencyStatus {
            active: true,
            emergency_type: EmergencyType::Other,
            description: BoundedVec::truncate_from(b"Benchmark emergency".to_vec()),
            declared_at: frame_system::Pallet::<T>::block_number(),
            expires_at: None,
            declared_by: None,
            is_pending: false,
            veto_window_ends_at: None,
        });

        let proposal_id: u32 = 0;
        let override_reason: Vec<u8> = b"Emergency override required".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, proposal_id, override_reason);
    }

    #[benchmark]
    fn update_chain_parameter() {
        let key: Vec<u8> = b"VotingPeriod".to_vec();
        let value: u64 = 28800;

        #[extrinsic_call]
        _(RawOrigin::Root, key, value);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
