//! Benchmarking for the community pallet

use super::*;
use crate::types::ProjectType;
use frame_benchmarking::v2::*;
use frame_support::traits::{ConstU32, Currency, Get, ReservableCurrency};
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn record_participation() {
        let caller: T::AccountId = whitelisted_caller();
        let activity_code: u8 = 0; // VotedOnProposal
        grant_kyc::<T>(&caller);

        #[extrinsic_call]
        record_participation(RawOrigin::Root, caller.clone(), activity_code);

        // Verify participation was recorded
        assert!(!ParticipationHistory::<T>::get(&caller).is_empty());
    }

    #[benchmark]
    fn update_srs() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 0, 0);
        grant_kyc::<T>(&target);

        // Advance block number past SrsUpdateCooldown so the extrinsic doesn't reject
        let cooldown = T::SrsUpdateCooldown::get();
        frame_system::Pallet::<T>::set_block_number(cooldown);

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

    /// A well-formed SRS record: Silver tier, so tier-gated calls accept it.
    fn sample_srs<BlockNumber>(block_number: BlockNumber) -> SRSData<BlockNumber> {
        SRSData {
            score: 5_000u32,
            governance_score: 2_000u32,
            education_score: 1_000u32,
            sustainability_score: 1_000u32,
            participation_score: 1_000u32,
            peer_endorsements: 0u32,
            honesty_rating: 0u32,
            last_updated: block_number,
            tier: SRSTier::Silver,
            total_contributions: 0u32,
            public_display: false,
            anonymous_hash: None,
        }
    }

    #[benchmark]
    fn endorse_peer() {
        let endorser: T::AccountId = whitelisted_caller();
        let endorsee: T::AccountId = account("endorsee", 0, 0);
        let skill_code: u8 = 0; // Leadership
        grant_kyc::<T>(&endorser);
        grant_kyc::<T>(&endorsee);

        // Endorser must have SRS record with tier >= Silver
        SocialResponsibilityScores::<T>::insert(
            &endorser,
            sample_srs(frame_system::Pallet::<T>::block_number()),
        );

        #[extrinsic_call]
        endorse_peer(
            RawOrigin::Signed(endorser.clone()),
            endorsee.clone(),
            skill_code,
        );

        // Verify endorsement count increased
        assert!(PeerEndorsements::<T>::get(&endorsee) > 0);
    }

    #[benchmark]
    fn set_srs_privacy() {
        let caller: T::AccountId = whitelisted_caller();
        grant_kyc::<T>(&caller);
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
        assert_eq!(
            SocialResponsibilityScores::<T>::get(&caller)
                .unwrap()
                .public_display,
            is_public
        );
    }

    // ==================== shared setup helpers ====================

    /// Free balance handed to benchmark accounts: enough for deposits, treasury
    /// transfers and existence requirements, small enough to leave the reward
    /// mints comfortably clear of `MaxSupply`.
    /// Balance handed to a benchmark account: large enough to cover every
    /// deposit and transfer these benchmarks perform.
    const FUNDED_BALANCE: u128 = 1_000_000_000_000_000;

    /// Proposal value used by the proposal benchmarks. Must be at least the
    /// runtime's `ExistentialDeposit` (0.001 DALLA), otherwise the treasury
    /// payout at finalization destroys the beneficiary's account and the
    /// transfer fails.
    const PROPOSAL_AMOUNT: u128 = 1_000_000_000_000;

    fn fund<T: Config>(account: &T::AccountId) {
        let balance: BalanceOf<T> = FUNDED_BALANCE.saturated_into();
        T::Currency::make_free_balance_be(account, balance);
    }

    /// Benchmark accounts are hash-derived and so never appear in the mock's KYC
    /// allowlist. Opt them in so KYC-gated calls are reachable; the runtime
    /// build bypasses KYC entirely in benchmark mode, so this is a no-op there.
    #[cfg(not(test))]
    fn grant_kyc<T: Config>(_account: &T::AccountId) {}

    #[cfg(test)]
    fn grant_kyc<T: Config>(account: &T::AccountId) {
        crate::mock::grant_benchmark_kyc(account);
    }

    /// A `BoundedVec` filled to its configured maximum — worst-case encoding.
    fn max_bytes<L: Get<u32>>() -> BoundedVec<u8, L> {
        BoundedVec::try_from(vec![b'x'; L::get() as usize]).unwrap_or_default()
    }

    /// Worst-case participation history. `calculate_governance_score` and
    /// `calculate_participation_score` both walk the whole history, so an SRS
    /// recalculation is only fully measured against a full one.
    fn full_history<T: Config>(account: &T::AccountId) {
        let record = ParticipationRecord {
            activity_type: ActivityType::VoteCast,
            block_number: frame_system::Pallet::<T>::block_number(),
            value: 100u32,
        };
        let history =
            BoundedVec::try_from(vec![record; T::MaxParticipationHistory::get() as usize])
                .unwrap_or_default();
        ParticipationHistory::<T>::insert(account, history);
    }

    /// Seed a proposal with its deposit genuinely reserved, so finalisation
    /// measures the matching `unreserve` / `slash_reserved`.
    fn seed_proposal<T: Config>(
        proposal_id: u32,
        proposer: &T::AccountId,
        beneficiary: &T::AccountId,
        amount: BalanceOf<T>,
        status: ProposalStatus,
        voting_deadline: BlockNumberFor<T>,
    ) {
        // Mirror the pallet's own deposit rule: `ProposalDepositPercentage` is
        // expressed in basis points and applied as `amount * pct / 10_000`.
        let deposit = amount
            .saturated_into::<u128>()
            .saturating_mul(T::ProposalDepositPercentage::get() as u128)
            .saturating_div(10_000u128)
            .saturated_into::<BalanceOf<T>>();
        fund::<T>(proposer);
        T::Currency::reserve(proposer, deposit).expect("benchmark account is funded");
        ProposalCount::<T>::put(proposal_id.saturating_add(1));
        CommunityProposals::<T>::insert(
            proposal_id,
            CommunityProposal {
                proposer: proposer.clone(),
                proposal_type: CommunityProposalType::LocalProject,
                beneficiary: beneficiary.clone(),
                amount,
                title: max_bytes::<ConstU32<128>>(),
                description: max_bytes::<ConstU32<1024>>(),
                deposit,
                status,
                submission_block: frame_system::Pallet::<T>::block_number(),
                voting_deadline,
                votes_for: 0u32,
                votes_against: 0u32,
                total_votes: 0u32,
            },
        );
    }

    // ==================== proposals, votes, ethics ====================

    #[benchmark]
    fn submit_community_proposal() {
        let caller: T::AccountId = whitelisted_caller();
        let beneficiary: T::AccountId = account("beneficiary", 0, 0);
        fund::<T>(&caller);
        grant_kyc::<T>(&caller);
        grant_kyc::<T>(&beneficiary);

        // Worst case for `check_ethics_filter`: the proposer already has an SRS
        // record, so the honesty lookup runs too, and the threshold is
        // unreachable, so the proposal is flagged and the extra event fires.
        SocialResponsibilityScores::<T>::insert(
            &caller,
            sample_srs(frame_system::Pallet::<T>::block_number()),
        );
        EthicsFilterConfig::<T>::put(EthicsConfig {
            min_honesty_rating: u32::MAX,
            council_review_threshold: 0u64,
            min_council_votes: 1u32,
            council_members: BoundedVec::default(),
        });

        #[extrinsic_call]
        submit_community_proposal(
            RawOrigin::Signed(caller.clone()),
            0u8,
            beneficiary,
            PROPOSAL_AMOUNT.saturated_into(),
            max_bytes::<ConstU32<128>>(),
            max_bytes::<ConstU32<1024>>(),
        );

        assert_eq!(
            CommunityProposals::<T>::get(0u32)
                .expect("proposal stored")
                .status,
            ProposalStatus::EthicsReview
        );
    }

    #[benchmark]
    fn vote_community_proposal() {
        let voter: T::AccountId = whitelisted_caller();
        let proposer: T::AccountId = account("proposer", 0, 0);
        let beneficiary: T::AccountId = account("beneficiary", 0, 0);
        grant_kyc::<T>(&voter);

        seed_proposal::<T>(
            0u32,
            &proposer,
            &beneficiary,
            PROPOSAL_AMOUNT.saturated_into(),
            ProposalStatus::Active,
            frame_system::Pallet::<T>::block_number() + 1_000u32.into(),
        );
        // An SRS record makes the vote carry a weight, which is the branch that
        // reads `SocialResponsibilityScores`.
        SocialResponsibilityScores::<T>::insert(
            &voter,
            sample_srs(frame_system::Pallet::<T>::block_number()),
        );

        #[extrinsic_call]
        vote_community_proposal(RawOrigin::Signed(voter.clone()), 0u32, true);

        assert_eq!(
            CommunityProposals::<T>::get(0u32)
                .expect("proposal stored")
                .votes_for,
            5_000u32
        );
    }

    #[benchmark]
    fn withdraw_vote() {
        let voter: T::AccountId = whitelisted_caller();
        let proposer: T::AccountId = account("proposer", 0, 0);
        let beneficiary: T::AccountId = account("beneficiary", 0, 0);

        seed_proposal::<T>(
            0u32,
            &proposer,
            &beneficiary,
            PROPOSAL_AMOUNT.saturated_into(),
            ProposalStatus::Active,
            frame_system::Pallet::<T>::block_number() + 1_000u32.into(),
        );
        ProposalVotes::<T>::insert(
            0u32,
            &voter,
            Vote {
                approve: true,
                weight: 42u32,
            },
        );

        #[extrinsic_call]
        withdraw_vote(RawOrigin::Signed(voter.clone()), 0u32);

        assert!(!ProposalVotes::<T>::contains_key(0u32, &voter));
    }

    #[benchmark]
    fn finalize_community_proposal() {
        let proposer: T::AccountId = account("proposer", 0, 0);
        let beneficiary: T::AccountId = account("beneficiary", 0, 0);
        let voter: T::AccountId = whitelisted_caller();

        // Voting has closed with a full quorum and a majority in favour, so the
        // approval path runs: the treasury transfer succeeds, the deposit is
        // returned and both confirmation events fire.
        seed_proposal::<T>(
            0u32,
            &proposer,
            &beneficiary,
            PROPOSAL_AMOUNT.saturated_into(),
            ProposalStatus::Active,
            frame_system::Pallet::<T>::block_number(),
        );
        fund::<T>(&T::CommunityTreasuryAccount::get());
        CommunityProposals::<T>::mutate(0u32, |proposal| {
            let proposal = proposal.as_mut().expect("proposal just seeded");
            proposal.votes_for = 1_000u32;
            proposal.total_votes = T::MinProposalVoters::get();
        });
        frame_system::Pallet::<T>::set_block_number(
            frame_system::Pallet::<T>::block_number() + 1u32.into(),
        );

        #[extrinsic_call]
        finalize_community_proposal(RawOrigin::Signed(voter), 0u32);

        assert_eq!(
            CommunityProposals::<T>::get(0u32)
                .expect("proposal stored")
                .status,
            ProposalStatus::Approved
        );
    }

    #[benchmark]
    fn ethics_council_vote() {
        let voter: T::AccountId = whitelisted_caller();
        let proposer: T::AccountId = account("proposer", 0, 0);
        let beneficiary: T::AccountId = account("beneficiary", 0, 0);

        // A full council makes `try_finalize_ethics_review` walk every seat, and
        // a one-vote threshold means this vote also applies the decision.
        let mut members = vec![voter.clone()];
        for i in 0..9u32 {
            members.push(account("council", i, 0));
        }
        EthicsFilterConfig::<T>::put(EthicsConfig {
            min_honesty_rating: 5_000u32,
            council_review_threshold: 50_000u64,
            min_council_votes: 1u32,
            council_members: BoundedVec::try_from(members).unwrap_or_default(),
        });

        seed_proposal::<T>(
            0u32,
            &proposer,
            &beneficiary,
            PROPOSAL_AMOUNT.saturated_into(),
            ProposalStatus::EthicsReview,
            frame_system::Pallet::<T>::block_number(),
        );

        // A lone rejection is enough here, which takes the branch that also
        // returns the deposit.
        #[extrinsic_call]
        ethics_council_vote(RawOrigin::Signed(voter.clone()), 0u32, false);

        assert_eq!(
            CommunityProposals::<T>::get(0u32)
                .expect("proposal stored")
                .status,
            ProposalStatus::Rejected
        );
    }

    // ==================== sanctions ====================

    #[benchmark]
    fn sanction_account() {
        let account: T::AccountId = account("sanctioned", 0, 0);

        #[extrinsic_call]
        sanction_account(
            RawOrigin::Root,
            account.clone(),
            max_bytes::<ConstU32<128>>(),
        );

        assert!(
            SanctionedAccounts::<T>::get(&account)
                .expect("sanction stored")
                .active
        );
    }

    #[benchmark]
    fn lift_sanction() {
        let account: T::AccountId = account("sanctioned", 0, 0);
        SanctionedAccounts::<T>::insert(
            &account,
            SanctionStatus {
                active: true,
                reason: max_bytes::<ConstU32<128>>(),
                sanctioned_at: 0u32,
            },
        );

        #[extrinsic_call]
        lift_sanction(RawOrigin::Root, account.clone());

        assert!(!SanctionedAccounts::<T>::contains_key(&account));
    }

    // ==================== education, green projects, referrals ====================

    #[benchmark]
    fn complete_education_module() {
        let caller: T::AccountId = whitelisted_caller();
        let module_id: u32 = 0u32;

        // `update_srs_after_education` recalculates every score, and the two
        // history-walking helpers are the expensive part of that.
        full_history::<T>(&caller);
        EducationModules::<T>::insert(
            module_id,
            EducationModule {
                id: module_id,
                title: max_bytes::<ConstU32<128>>(),
                description: max_bytes::<ConstU32<256>>(),
                reward_amount: T::EducationRewardAmount::get().saturated_into(),
                total_completions: 0u32,
                max_completions: Some(u32::MAX),
                active: true,
            },
        );

        #[extrinsic_call]
        complete_education_module(
            RawOrigin::Signed(caller.clone()),
            module_id,
            max_bytes::<ConstU32<256>>(),
        );

        assert!(CompletedEducation::<T>::contains_key(&caller, module_id));
    }

    #[benchmark]
    fn contribute_to_green_project() {
        let caller: T::AccountId = whitelisted_caller();
        let project_id: u32 = 0u32;
        let amount: u64 = 1u64;

        fund::<T>(&caller);
        // The contribution is transferred to the community treasury, and 1 unit
        // is below `ExistentialDeposit`, so the destination account must already
        // exist and be funded or the transfer reverts.
        fund::<T>(&T::CommunityTreasuryAccount::get());
        full_history::<T>(&caller);
        // One unit short of a milestone, and with no prior contribution, so the
        // contributor counter, the aggregate stats and the milestone event all
        // take their heaviest path.
        GreenProjects::<T>::insert(
            project_id,
            GreenProject {
                id: project_id,
                project_type: ProjectType::Reforestation,
                title: max_bytes::<ConstU32<128>>(),
                amount_contributed: 99_999u64,
                total_contributors: 0u32,
                active: true,
            },
        );

        #[extrinsic_call]
        contribute_to_green_project(RawOrigin::Signed(caller.clone()), project_id, amount);

        let project = GreenProjects::<T>::get(project_id).expect("project stored");
        assert_eq!(project.amount_contributed, 100_000u64);
        assert_eq!(project.total_contributors, 1u32);
    }

    #[benchmark]
    fn claim_referral_reward() {
        let referrer: T::AccountId = whitelisted_caller();
        let referee: T::AccountId = account("referee", 0, 0);
        grant_kyc::<T>(&referrer);
        grant_kyc::<T>(&referee);

        // The referee has to look like a real, already-onboarded user.
        SocialResponsibilityScores::<T>::insert(
            &referee,
            sample_srs(frame_system::Pallet::<T>::block_number()),
        );

        #[extrinsic_call]
        claim_referral_reward(RawOrigin::Signed(referrer.clone()), referee.clone());

        assert!(RefereeHasReferrer::<T>::get(&referee));
    }

    // ==================== oracle attestations ====================

    #[benchmark]
    fn attest_participation() {
        let caller: T::AccountId = whitelisted_caller();
        let subject: T::AccountId = account("subject", 0, 0);

        // The runtime wires `OracleAttestationOrigin` to a collective-membership
        // check *and* the call requires a signed origin, so unlike the admin-only
        // extrinsics this one cannot be benchmarked with `RawOrigin::Root`.
        T::make_council_member(&caller);

        // One vote short of quorum, so the call takes the heavier finalisation
        // path: record the activity, clear the pending attestations and reset
        // the counter.
        let key = (subject.clone(), 2u8);
        AttestationCount::<T>::insert(&key, T::MinAttestationsRequired::get().saturating_sub(1));

        #[extrinsic_call]
        attest_participation(RawOrigin::Signed(caller.clone()), subject.clone(), 2u8);

        assert!(AttestedActivities::<T>::get(&key));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
