//! Benchmarking for the governance pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::SaturatedConversion;
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

    // ══════════════════════════════════════════════════════════════════════════
    // Shared setup helpers
    // ══════════════════════════════════════════════════════════════════════════

    /// A funded account that passes the governance compliance gate.
    ///
    /// The runtime's compliance provider short-circuits to "eligible" under
    /// `runtime-benchmarks`, so nothing has to be seeded here — and the real
    /// eligibility check stays on the measured path.
    fn eligible_caller<T: Config>(name: &'static str, seed: u32) -> T::AccountId {
        let who: T::AccountId = account(name, seed, 0);
        let _ = T::Currency::make_free_balance_be(&who, T::MinimumDeposit::get() * 100u32.into());
        who
    }

    /// Writes a proposal straight into storage and returns its id.
    ///
    /// Seeding storage rather than calling `submit_proposal` keeps setup cheap
    /// and side-effect free (no deposit, no proposal cooldown, and no drift in
    /// `NextProposalId` beyond the id we hand out).
    fn seed_proposal<T: Config>(
        proposer: &T::AccountId,
        proposal_type: ProposalType,
        status: ProposalStatus,
        voting_start: BlockNumberFor<T>,
        voting_end: BlockNumberFor<T>,
    ) -> u32 {
        let id = NextProposalId::<T>::get();
        NextProposalId::<T>::put(id.saturating_add(1));
        Proposals::<T>::insert(
            id,
            Proposal::<T::AccountId, BlockNumberFor<T>, BalanceOf<T>> {
                id,
                proposer: proposer.clone(),
                title: BoundedVec::truncate_from(b"Benchmark proposal".to_vec()),
                description: BoundedVec::truncate_from(b"Benchmark proposal body".to_vec()),
                proposal_type,
                threshold: VotingThreshold::SimpleMajority,
                deposit: T::MinimumDeposit::get(),
                voting_start,
                voting_end,
                vote_tally: VoteTally {
                    ayes: 0,
                    nays: 0,
                    abstentions: 0,
                    total_weight: 0,
                    participation: 0,
                },
                status,
                is_emergency: false,
                department: None,
                district: None,
                requires_cross_approval: false,
                cross_approved_by: BoundedVec::default(),
                action: None,
                executed_at: None,
            },
        );
        id
    }

    /// Seats `account` on the council, which `execute_proposal` and
    /// `veto_emergency` require.
    fn seat_council_member<T: Config>(account: &T::AccountId) {
        let now = frame_system::Pallet::<T>::block_number();
        CouncilMembers::<T>::insert(
            account,
            CouncilMember::<T::AccountId, BlockNumberFor<T>> {
                account: account.clone(),
                role: BoardRole::Founder,
                term_end: now.saturating_add(100u32.into()),
                is_rotating: false,
                community_rank: 0,
                pouw_contribution: 0,
                voting_weight: 1,
                term_start: now,
                votes_received: 0,
                proposals_authored: 0,
                participation_rate: 100,
                consecutive_terms: 0,
            },
        );
    }

    /// Puts the chain into an active emergency, matching what `declare_emergency`
    /// followed by the veto window would produce.
    fn activate_emergency<T: Config>() {
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
    }

    /// Starts a district election and returns the record the pallet stored.
    ///
    /// The id is the block number the election started in, so it is read back
    /// from storage rather than assumed.
    fn start_election<T: Config>(district_index: u8) -> Election<BlockNumberFor<T>> {
        Pallet::<T>::start_district_election(
            RawOrigin::Root.into(),
            district_index,
            1u32,
            100u32,
            100u32,
        )
        .expect("start_district_election should succeed");
        DistrictElections::<T>::iter()
            .next()
            .map(|(_, election)| election)
            .expect("the election is recorded")
    }

    /// Registers `count` distinct candidates in a district's open election.
    fn register_candidates<T: Config>(district_index: u8, count: u32) {
        for index in 0..count {
            let candidate = eligible_caller::<T>("candidate", index);
            Pallet::<T>::register_candidate(
                RawOrigin::Signed(candidate).into(),
                district_index,
                b"Benchmark election platform".to_vec(),
            )
            .expect("register_candidate should succeed");
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Internal costs charged from `on_initialize`
    // ══════════════════════════════════════════════════════════════════════════

    /// Per-seat cost of expiring one council term.
    ///
    /// `on_initialize` adds this once per expired seat (up to 10 per block), so
    /// the measured unit is a single seat rather than the whole hook.
    #[benchmark]
    fn expire_council_member() {
        let account: T::AccountId = account("expiring", 0, 0);
        seat_council_member::<T>(&account);

        #[block]
        {
            Pallet::<T>::expire_seat(&account);
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Departmental governance
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn set_department_manager() {
        let manager: T::AccountId = account("manager", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, 2u8, manager);
    }

    #[benchmark]
    fn submit_department_proposal() {
        let manager = eligible_caller::<T>("manager", 0);
        Pallet::<T>::set_department_manager(RawOrigin::Root.into(), 2u8, manager.clone())
            .expect("set_department_manager should succeed");

        let title: Vec<u8> = b"Department budget request".to_vec();
        let description: Vec<u8> = b"Benchmark department proposal".to_vec();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(manager),
            2u8,
            title,
            description,
            0u8,
            0u8,
            true,
        );
    }

    #[benchmark]
    fn approve_cross_department() {
        let proposer = eligible_caller::<T>("manager", 0);
        let approver = eligible_caller::<T>("manager", 1);
        Pallet::<T>::set_department_manager(RawOrigin::Root.into(), 2u8, proposer.clone())
            .expect("set_department_manager should succeed");
        Pallet::<T>::set_department_manager(RawOrigin::Root.into(), 3u8, approver.clone())
            .expect("set_department_manager should succeed");
        Pallet::<T>::submit_department_proposal(
            RawOrigin::Signed(proposer).into(),
            2u8,
            b"Cross-department proposal".to_vec(),
            b"Requires a second department's approval".to_vec(),
            0u8,
            0u8,
            true,
        )
        .expect("submit_department_proposal should succeed");

        let proposal_id: u32 = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(approver), proposal_id, 3u8);
    }

    #[benchmark]
    fn add_board_member() {
        let member: T::AccountId = account("board-member", 0, 0);
        let origin = T::CouncilOrigin::try_successful_origin()
            .expect("CouncilOrigin exposes a successful origin for benchmarks");

        #[extrinsic_call]
        _(origin, member, 0u8, 1u8);
    }

    #[benchmark]
    fn remove_board_member() {
        let member: T::AccountId = account("board-member", 0, 0);
        let origin = T::CouncilOrigin::try_successful_origin()
            .expect("CouncilOrigin exposes a successful origin for benchmarks");
        Pallet::<T>::add_board_member(origin.clone(), member.clone(), 0u8, 1u8)
            .expect("add_board_member should succeed");

        let reason: Vec<u8> = b"Term ended".to_vec();

        #[extrinsic_call]
        _(origin, member, reason);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Voting, delegation, proposals
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn nominate_for_delegate() {
        let nominator = eligible_caller::<T>("nominator", 0);
        let nominee = eligible_caller::<T>("nominee", 0);
        let election_id = frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
        CurrentElection::<T>::put(election_id);

        #[extrinsic_call]
        _(RawOrigin::Signed(nominator), nominee);
    }

    #[benchmark]
    fn vote_for_delegate() {
        let voter = eligible_caller::<T>("voter", 0);
        let nominee = eligible_caller::<T>("nominee", 0);
        let election_id = frame_system::Pallet::<T>::block_number().saturating_add(1u32.into());
        CurrentElection::<T>::put(election_id);
        DelegateNominees::<T>::insert(&nominee, 0u32);

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), nominee);
    }

    #[benchmark]
    fn execute_proposal() {
        let caller = eligible_caller::<T>("council", 0);
        seat_council_member::<T>(&caller);
        let start = frame_system::Pallet::<T>::block_number();
        let end = start.saturating_add(100u32.into());
        let proposal_id = seed_proposal::<T>(
            &caller,
            ProposalType::Economic,
            ProposalStatus::Approved,
            start,
            end,
        );
        // Worst case: `execute_action` runs, and the treasury actually pays out.
        Proposals::<T>::mutate(proposal_id, |maybe| {
            if let Some(proposal) = maybe {
                proposal.action = Some(ProposalAction::TreasurySpend {
                    recipient: caller.clone(),
                    amount: T::MinimumDeposit::get(),
                });
            }
        });
        let _ = T::Currency::make_free_balance_be(
            &Pallet::<T>::account_id(),
            T::MinimumDeposit::get() * 1_000u32.into(),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), proposal_id);
    }

    #[benchmark]
    fn amend_proposal() {
        let proposer = eligible_caller::<T>("proposer", 0);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &proposer,
            ProposalType::Economic,
            ProposalStatus::Pending,
            now.saturating_add(10u32.into()),
            now.saturating_add(1_000u32.into()),
        );

        let new_title = Some(BoundedVec::truncate_from(b"Amended title".to_vec()));
        let new_description = None;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(proposer),
            proposal_id,
            new_title,
            new_description,
        );
    }

    #[benchmark]
    fn claim_participation_reward() {
        let caller = eligible_caller::<T>("voter", 0);
        let start = frame_system::Pallet::<T>::block_number();
        let end = start.saturating_add(1_000u32.into());
        // Worst case for the vote-based reward: the bounded scan looks at the
        // full window (`max_check = min(NextProposalId, 50)`) before it finds the
        // caller's vote, so the vote sits on the last proposal in that window.
        let mut last_proposal = 0u32;
        for _ in 0..50u32 {
            last_proposal = seed_proposal::<T>(
                &caller,
                ProposalType::Economic,
                ProposalStatus::Voting,
                start,
                end,
            );
        }
        Votes::<T>::insert(
            last_proposal,
            &caller,
            Vote {
                vote: VoteChoice::Aye,
                weight: 1,
                conviction: 0,
                timestamp: 0,
            },
        );
        // Payouts are fixed DALLA amounts (10 DALLA for the vote reward), so the
        // pallet account needs far more than a single `MinimumDeposit`.
        let _ = T::Currency::make_free_balance_be(
            &Pallet::<T>::account_id(),
            T::MinimumDeposit::get() * 1_000_000u32.into(),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u8);
    }

    #[benchmark]
    fn set_proposal_priority() {
        let proposer: T::AccountId = account("proposer", 0, 0);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &proposer,
            ProposalType::Economic,
            ProposalStatus::Pending,
            now,
            now.saturating_add(1_000u32.into()),
        );

        #[extrinsic_call]
        _(RawOrigin::Root, proposal_id, 3u8);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Emergency mode
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn declare_emergency() {
        let origin = T::CouncilOrigin::try_successful_origin()
            .expect("CouncilOrigin exposes a successful origin for benchmarks");
        let description: Vec<u8> = b"Benchmark hurricane emergency".to_vec();

        #[extrinsic_call]
        _(origin, 5u8, description, 48u32);
    }

    #[benchmark]
    fn end_emergency() {
        let origin = T::CouncilOrigin::try_successful_origin()
            .expect("CouncilOrigin exposes a successful origin for benchmarks");
        Pallet::<T>::declare_emergency(origin.clone(), 5u8, b"Benchmark emergency".to_vec(), 48u32)
            .expect("declare_emergency should succeed");

        #[extrinsic_call]
        _(origin);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // District elections
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn start_district_election() {
        #[extrinsic_call]
        _(RawOrigin::Root, 0u8, 1u32, 100u32, 100u32);
    }

    #[benchmark]
    fn register_candidate() {
        start_election::<T>(0u8);
        let candidate = eligible_caller::<T>("candidate", 0);
        let platform: Vec<u8> = b"Benchmark election platform".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Signed(candidate), 0u8, platform);
    }

    #[benchmark]
    fn vote_in_district_election() {
        let election = start_election::<T>(0u8);
        let candidate = eligible_caller::<T>("candidate", 0);
        Pallet::<T>::register_candidate(
            RawOrigin::Signed(candidate.clone()).into(),
            0u8,
            b"Benchmark election platform".to_vec(),
        )
        .expect("register_candidate should succeed");
        // Registration closes and voting opens at `registration_end`.
        frame_system::Pallet::<T>::set_block_number(election.registration_end);
        let voter = eligible_caller::<T>("voter", 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), 0u8, candidate);
    }

    #[benchmark]
    fn finalize_district_election() {
        let election = start_election::<T>(0u8);
        // Worst case: the call iterates and sorts up to `MaxCandidatesPerElection`
        // (200 on the runtime) candidates before seating the top `seats`.
        register_candidates::<T>(0u8, T::MaxCandidatesPerElection::get());
        frame_system::Pallet::<T>::set_block_number(
            election.voting_end.saturating_add(1u32.into()),
        );

        #[extrinsic_call]
        _(RawOrigin::Root, 0u8);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Vote delegation
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn delegate_vote() {
        let delegator = eligible_caller::<T>("delegator", 0);
        let delegate = eligible_caller::<T>("delegate", 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(delegator), delegate, None);
    }

    #[benchmark]
    fn revoke_delegation() {
        let delegator = eligible_caller::<T>("delegator", 0);
        let delegate = eligible_caller::<T>("delegate", 0);
        Pallet::<T>::delegate_vote(RawOrigin::Signed(delegator.clone()).into(), delegate, None)
            .expect("delegate_vote should succeed");

        #[extrinsic_call]
        _(RawOrigin::Signed(delegator));
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Referendums
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn create_referendum() {
        let creator = eligible_caller::<T>("creator", 0);
        let title: Vec<u8> = b"Benchmark referendum".to_vec();
        let description: Vec<u8> = b"Benchmark referendum description".to_vec();
        // Worst case: the maximum of 10 options is written back in `vote_counts`.
        let options: Vec<Vec<u8>> = (0u8..10u8)
            .map(|index| sp_std::vec![b'o', b'p', b't', index])
            .collect();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(creator),
            title,
            description,
            options,
            10u8,
            100u32,
            None,
        );
    }

    #[benchmark]
    fn vote_on_referendum() {
        let creator = eligible_caller::<T>("creator", 0);
        let voter = eligible_caller::<T>("voter", 0);
        Pallet::<T>::create_referendum(
            RawOrigin::Signed(creator).into(),
            b"Benchmark referendum".to_vec(),
            b"Benchmark referendum description".to_vec(),
            sp_std::vec![b"yes".to_vec(), b"no".to_vec()],
            10u8,
            100u32,
            None,
        )
        .expect("create_referendum should succeed");

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), 0u32, 0u8);
    }

    #[benchmark]
    fn finalize_referendum() {
        let creator = eligible_caller::<T>("creator", 0);
        let voter = eligible_caller::<T>("voter", 0);
        Pallet::<T>::create_referendum(
            RawOrigin::Signed(creator).into(),
            b"Benchmark referendum".to_vec(),
            b"Benchmark referendum description".to_vec(),
            sp_std::vec![b"yes".to_vec(), b"no".to_vec()],
            10u8,
            100u32,
            None,
        )
        .expect("create_referendum should succeed");
        Pallet::<T>::vote_on_referendum(RawOrigin::Signed(voter.clone()).into(), 0u32, 0u8)
            .expect("vote_on_referendum should succeed");
        let referendum = Referendums::<T>::get(0u32).expect("referendum exists");
        frame_system::Pallet::<T>::set_block_number(
            referendum.voting_end.saturating_add(1u32.into()),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), 0u32);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // District budgets and treasury spend
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn allocate_district_budget() {
        let amount: BalanceOf<T> = T::MinimumDeposit::get() * 100u32.into();

        #[extrinsic_call]
        _(RawOrigin::Root, 0u8, amount, 1_000u32);
    }

    #[benchmark]
    fn propose_treasury_spend() {
        let proposer = eligible_caller::<T>("proposer", 0);
        let recipient: T::AccountId = account("recipient", 0, 0);
        let amount: BalanceOf<T> = T::MinimumDeposit::get();
        let description: Vec<u8> = b"Benchmark treasury spend".to_vec();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(proposer),
            recipient,
            amount,
            description,
            None,
        );
    }

    /// Upper bound (exclusive) of the single-approval treasury band.
    ///
    /// `propose_treasury_spend` picks a threshold of 1 below this amount, 3 below
    /// `100_000_000_000_000`, and 4 above it. Benchmark amounts must stay in the
    /// band whose threshold the setup actually satisfies — and must not be derived
    /// from `MinimumDeposit`, which is a *runtime* parameter (10 DALLA at 12
    /// decimals) that lands in the 3-approval band.
    const SINGLE_APPROVAL_TREASURY_LIMIT: u128 = 10_000_000_000_000;

    /// A treasury amount that needs exactly one council approval.
    fn single_approval_amount<T: Config>() -> BalanceOf<T> {
        (SINGLE_APPROVAL_TREASURY_LIMIT / 10).saturated_into()
    }

    #[benchmark]
    fn approve_treasury_spend() {
        let proposer = eligible_caller::<T>("proposer", 0);
        let approver = eligible_caller::<T>("approver", 0);
        seat_council_member::<T>(&approver);
        let recipient: T::AccountId = account("recipient", 0, 0);
        Pallet::<T>::propose_treasury_spend(
            RawOrigin::Signed(proposer).into(),
            recipient,
            single_approval_amount::<T>(),
            b"Benchmark treasury spend".to_vec(),
            None,
        )
        .expect("propose_treasury_spend should succeed");

        #[extrinsic_call]
        _(RawOrigin::Signed(approver), 0u32);
    }

    #[benchmark]
    fn execute_treasury_proposal() {
        let proposer = eligible_caller::<T>("proposer", 0);
        let approver = eligible_caller::<T>("approver", 0);
        seat_council_member::<T>(&approver);
        let recipient: T::AccountId = account("recipient", 0, 0);
        let amount = single_approval_amount::<T>();
        Pallet::<T>::propose_treasury_spend(
            RawOrigin::Signed(proposer.clone()).into(),
            recipient,
            amount,
            b"Benchmark treasury spend".to_vec(),
            None,
        )
        .expect("propose_treasury_spend should succeed");
        // The proposal's threshold is 1 at this amount, so one approval is enough.
        Pallet::<T>::approve_treasury_spend(RawOrigin::Signed(approver).into(), 0u32)
            .expect("approve_treasury_spend should succeed");
        // The national reserve must cover the spend exactly (worst case for the
        // `InsufficientNationalTreasury` check), but the actual transfer is
        // `KeepAlive`, so the pallet account has to hold more than the spend or it
        // would be reaped.
        NationalTreasuryReserve::<T>::put(amount);
        let _ =
            T::Currency::make_free_balance_be(&Pallet::<T>::account_id(), amount * 10u32.into());

        #[extrinsic_call]
        _(RawOrigin::Signed(proposer), 0u32);
    }

    #[benchmark]
    fn transfer_district_budget() {
        let amount: BalanceOf<T> = T::MinimumDeposit::get() * 100u32.into();
        Pallet::<T>::allocate_district_budget(RawOrigin::Root.into(), 0u8, amount, 1_000u32)
            .expect("allocate_district_budget should succeed");
        let reason: Vec<u8> = b"Reallocation to a district in need".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, 0u8, 1u8, amount / 2u32.into(), reason);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Emergency execution
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn execute_emergency_proposal() {
        let caller = eligible_caller::<T>("council", 0);
        activate_emergency::<T>();
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &caller,
            ProposalType::Emergency,
            ProposalStatus::Voting,
            now,
            now.saturating_add(1_000u32.into()),
        );
        // 67% ayes clears the supermajority bar; the action is executed too, so
        // this benchmark covers the expensive half of the call.
        Proposals::<T>::mutate(proposal_id, |maybe| {
            if let Some(proposal) = maybe {
                proposal.is_emergency = true;
                proposal.vote_tally.ayes = 67;
                proposal.vote_tally.nays = 33;
                proposal.vote_tally.total_weight = 100;
                proposal.action = Some(ProposalAction::TreasurySpend {
                    recipient: caller.clone(),
                    amount: T::MinimumDeposit::get(),
                });
            }
        });
        let _ = T::Currency::make_free_balance_be(
            &Pallet::<T>::account_id(),
            T::MinimumDeposit::get() * 1_000u32.into(),
        );

        #[extrinsic_call]
        _(RawOrigin::Root, proposal_id);
    }

    #[benchmark]
    fn fast_track_referendum() {
        let creator = eligible_caller::<T>("creator", 0);
        Pallet::<T>::create_referendum(
            RawOrigin::Signed(creator).into(),
            b"Benchmark referendum".to_vec(),
            b"Benchmark referendum description".to_vec(),
            sp_std::vec![b"yes".to_vec(), b"no".to_vec()],
            10u8,
            100u32,
            None,
        )
        .expect("create_referendum should succeed");
        activate_emergency::<T>();

        #[extrinsic_call]
        _(RawOrigin::Root, 0u32);
    }

    #[benchmark]
    fn emergency_override_proposal() {
        let proposer = eligible_caller::<T>("proposer", 0);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &proposer,
            ProposalType::Emergency,
            ProposalStatus::Voting,
            now,
            now.saturating_add(1_000u32.into()),
        );
        activate_emergency::<T>();
        let justification: Vec<u8> = b"Immediate action required".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, proposal_id, true, justification);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Commit-reveal voting
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn commit_vote() {
        let voter = eligible_caller::<T>("voter", 0);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &voter,
            ProposalType::Economic,
            ProposalStatus::Voting,
            now,
            now.saturating_add(1_000u32.into()),
        );
        // `blake2_256(vote_choice_byte || salt)` — 0 = Aye.
        let mut payload = sp_std::vec![0u8];
        payload.extend_from_slice(&[7u8; 32]);
        let commitment = blake2_256(&payload);

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), proposal_id, commitment);
    }

    #[benchmark]
    fn reveal_vote() {
        let voter = eligible_caller::<T>("voter", 0);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &voter,
            ProposalType::Economic,
            ProposalStatus::Voting,
            now,
            now.saturating_add(1_000u32.into()),
        );
        let salt = [7u8; 32];
        let mut payload = sp_std::vec![0u8];
        payload.extend_from_slice(&salt);
        VoteCommitments::<T>::insert(proposal_id, &voter, blake2_256(&payload));

        #[extrinsic_call]
        _(RawOrigin::Signed(voter), proposal_id, 0u8, salt, 0u8);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Runtime upgrades, exit proofs, house ratification
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn apply_pending_runtime_upgrade() {
        // The weight attribute adds `SystemWeightInfo::set_code()`, whose cost the
        // SDK exposes as a fixed constant because `set_code` cannot be benchmarked
        // (it cannot extract a runtime version from a synthetic blob). This
        // benchmark therefore measures the extrinsic's own logic only.
        let code: Vec<u8> = sp_std::vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        PendingRuntimeUpgrade::<T>::put(blake2_256(&code));

        #[extrinsic_call]
        _(RawOrigin::Root, code);
    }

    #[benchmark]
    fn generate_exit_proof() {
        let caller = eligible_caller::<T>("exiting", 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    #[benchmark]
    fn ratify_runtime_upgrade() {
        let ratifier = eligible_caller::<T>("ratifier", 0);
        // Seat the caller so the real house-membership lookup passes.
        T::DualHouseProvider::make_house_member(&ratifier);
        let code_hash = blake2_256(b"benchmark runtime upgrade");
        PendingRuntimeUpgrade::<T>::put(code_hash);

        #[extrinsic_call]
        _(RawOrigin::Signed(ratifier), code_hash);
    }

    #[benchmark]
    fn ratify_constitutional_proposal() {
        let ratifier = eligible_caller::<T>("ratifier", 0);
        // Seat the caller in both houses: needs the real `pallet_collective`
        // membership lookup to pass, and `(true, true)` is the heaviest branch.
        T::DualHouseProvider::make_house_member(&ratifier);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal_id = seed_proposal::<T>(
            &ratifier,
            ProposalType::Constitutional,
            ProposalStatus::Approved,
            now,
            now.saturating_add(1_000u32.into()),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(ratifier), proposal_id);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Chain parameter locks
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn lock_chain_parameter() {
        let key: Vec<u8> = b"VotingPeriod".to_vec();
        ChainParameters::<T>::insert(BoundedVec::truncate_from(key.clone()), 28800u64);

        #[extrinsic_call]
        _(RawOrigin::Root, key);
    }

    #[benchmark]
    fn unlock_chain_parameter() {
        let key: Vec<u8> = b"VotingPeriod".to_vec();
        ChainParameters::<T>::insert(BoundedVec::truncate_from(key.clone()), 28800u64);
        Pallet::<T>::lock_chain_parameter(RawOrigin::Root.into(), key.clone())
            .expect("lock_chain_parameter should succeed");

        #[extrinsic_call]
        _(RawOrigin::Root, key);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Emergency veto — worst case
    // ══════════════════════════════════════════════════════════════════════════

    #[benchmark]
    fn veto_emergency() {
        let caller = eligible_caller::<T>("vetoer", 0);
        JaguarMode::<T>::put(EmergencyStatus {
            active: false,
            emergency_type: EmergencyType::Other,
            description: BoundedVec::truncate_from(b"Benchmark emergency".to_vec()),
            declared_at: frame_system::Pallet::<T>::block_number(),
            expires_at: None,
            declared_by: None,
            is_pending: true,
            veto_window_ends_at: None,
        });
        // This call walks up to `MaxCandidatesPerElection` council members to
        // derive the veto threshold, then clears the veto map once the threshold
        // is crossed. Seat a full council and pre-cast just enough vetoes that
        // this call takes the clearing branch.
        let council_count = T::MaxCandidatesPerElection::get();
        for index in 0..council_count {
            let member: T::AccountId = account("council-member", index, 0);
            seat_council_member::<T>(&member);
        }
        seat_council_member::<T>(&caller);
        for index in 0..(council_count / 3) {
            let early_veto: T::AccountId = account("early-veto", index, 0);
            let veto_block: BlockNumberFor<T> = 0u32.into();
            EmergencyVetoes::<T>::insert(&early_veto, veto_block);
        }

        #[extrinsic_call]
        _(RawOrigin::Signed(caller));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
