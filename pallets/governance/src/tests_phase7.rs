// Phase 7: Advanced Features - Comprehensive Test Suite
// Tests for vote delegation, proposal amendments, DALLA rewards, queue management, and performance metrics

use crate::{
    mock::*, Error, ProposalPriority,
    CouncilMember, BoardRole, CouncilMembers, Proposals, NextProposalId,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::traits::BadOrigin;

// Test account constants
const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

// ===== VOTE DELEGATION TESTS =====

#[test]
fn delegate_vote_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Alice delegates to Bob
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            Some(1000) // Expires at block 1000
        ));

        // Verify delegation created
        let delegation = BelizeGovernance::get_delegation_info(&ALICE);
        assert!(delegation.is_some());
        let delegation = delegation.unwrap();
        assert_eq!(delegation.delegator, ALICE);
        assert_eq!(delegation.delegate, BOB);
        assert_eq!(delegation.expires_at, 1000);
        assert!(delegation.is_active);

        // Verify Bob has 1 delegator
        assert_eq!(BelizeGovernance::count_delegated_votes(&BOB), 1);
    });
}

#[test]
fn delegation_default_expiry_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(100);

        // Alice delegates without specifying expiry
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            None // Default: 1 year from now
        ));

        let delegation = BelizeGovernance::get_delegation_info(&ALICE).unwrap();
        // Should be 100 + 5,256,000 = 5,256,100
        assert_eq!(delegation.expires_at, 5_256_100);
    });
}

#[test]
fn cannot_delegate_to_self() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_noop!(
            BelizeGovernance::delegate_vote(
                RuntimeOrigin::signed(ALICE),
                ALICE,
                Some(1000)
            ),
            Error::<Test>::CannotDelegateToSelf
        );
    });
}

#[test]
fn cannot_double_delegate() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // First delegation succeeds
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            Some(1000)
        ));

        // Second delegation fails
        assert_noop!(
            BelizeGovernance::delegate_vote(
                RuntimeOrigin::signed(ALICE),
                CHARLIE,
                Some(1000)
            ),
            Error::<Test>::DelegationAlreadyExists
        );
    });
}

#[test]
fn too_many_delegators_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create 100 delegators (max limit)
        for i in 0..100 {
            let delegator: u64 = 1000 + i;
            assert_ok!(BelizeGovernance::delegate_vote(
                RuntimeOrigin::signed(delegator),
                BOB,
                Some(1000)
            ));
        }

        // 101st delegation should fail
        assert_noop!(
            BelizeGovernance::delegate_vote(
                RuntimeOrigin::signed(9999),
                BOB,
                Some(1000)
            ),
            Error::<Test>::TooManyDelegators
        );
    });
}

#[test]
fn revoke_delegation_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Alice delegates to Bob
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            Some(1000)
        ));

        assert_eq!(BelizeGovernance::count_delegated_votes(&BOB), 1);

        // Alice revokes delegation
        assert_ok!(BelizeGovernance::revoke_delegation(
            RuntimeOrigin::signed(ALICE)
        ));

        // Verify delegation removed
        assert!(BelizeGovernance::get_delegation_info(&ALICE).is_none());
        assert_eq!(BelizeGovernance::count_delegated_votes(&BOB), 0);
    });
}

#[test]
fn revoke_non_existent_delegation_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_noop!(
            BelizeGovernance::revoke_delegation(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::NoDelegationFound
        );
    });
}

#[test]
fn voting_power_calculation_includes_delegations() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Bob's base voting power
        assert_eq!(BelizeGovernance::calculate_voting_power(&BOB), 1);

        // Alice delegates to Bob
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            Some(1000)
        ));

        // Bob's voting power increases
        assert_eq!(BelizeGovernance::calculate_voting_power(&BOB), 2);

        // Charlie also delegates to Bob
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(CHARLIE),
            BOB,
            Some(1000)
        ));

        // Bob's voting power increases again
        assert_eq!(BelizeGovernance::calculate_voting_power(&BOB), 3);
    });
}

#[test]
fn delegation_activity_check_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Alice delegates with short expiry
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(ALICE),
            BOB,
            Some(100)
        ));

        // Active before expiry
        assert!(BelizeGovernance::is_delegation_active(&ALICE));

        // Move past expiry
        System::set_block_number(101);

        // No longer active after expiry
        assert!(!BelizeGovernance::is_delegation_active(&ALICE));
    });
}

// ===== PROPOSAL AMENDMENT TESTS =====

#[test]
fn amend_proposal_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create proposal (voting starts at block 200)
        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Amend before voting starts
        let new_title = b"Updated Title".to_vec().try_into().unwrap();
        let new_description = b"Updated Description".to_vec().try_into().unwrap();

        assert_ok!(BelizeGovernance::amend_proposal(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            Some(new_title),
            Some(new_description)
        ));

        // Verify amendment recorded
        let amendment = BelizeGovernance::get_proposal_amendment(proposal_id);
        assert!(amendment.is_some());
        assert_eq!(amendment.unwrap().proposal_id, proposal_id);

        // Verify proposal updated
        let proposal = Proposals::<Test>::get(proposal_id).unwrap();
        assert_eq!(proposal.title.to_vec(), b"Updated Title".to_vec());
        assert_eq!(proposal.description.to_vec(), b"Updated Description".to_vec());
    });
}

#[test]
fn amend_only_title_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);
        let new_title = b"Just Title Update".to_vec().try_into().unwrap();

        assert_ok!(BelizeGovernance::amend_proposal(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            Some(new_title),
            None // Don't update description
        ));

        let proposal = Proposals::<Test>::get(proposal_id).unwrap();
        assert_eq!(proposal.title.to_vec(), b"Just Title Update".to_vec());
    });
}

#[test]
fn cannot_amend_after_voting_starts() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 10, 50);

        // Move to voting period (LaunchPeriod is 28,800 blocks)
        System::set_block_number(28_801);

        let new_title = b"Too Late".to_vec().try_into().unwrap();

        assert_noop!(
            BelizeGovernance::amend_proposal(
                RuntimeOrigin::signed(ALICE),
                proposal_id,
                Some(new_title),
                None
            ),
            Error::<Test>::CannotAmendAfterVoting
        );
    });
}

#[test]
fn only_proposer_can_amend() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);
        let new_title = b"Unauthorized".to_vec().try_into().unwrap();

        // Bob tries to amend Alice's proposal
        assert_noop!(
            BelizeGovernance::amend_proposal(
                RuntimeOrigin::signed(BOB),
                proposal_id,
                Some(new_title),
                None
            ),
            Error::<Test>::NotProposalAuthor
        );
    });
}

#[test]
fn cannot_amend_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);
        let new_title = b"First Amendment".to_vec().try_into().unwrap();

        // First amendment succeeds
        let new_title_copy = b"First Amendment".to_vec().try_into().unwrap();
        assert_ok!(BelizeGovernance::amend_proposal(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            Some(new_title),
            None
        ));

        // Second amendment fails
        assert_noop!(
            BelizeGovernance::amend_proposal(
                RuntimeOrigin::signed(ALICE),
                proposal_id,
                Some(new_title_copy),
                None
            ),
            Error::<Test>::AmendmentAlreadyExists
        );
    });
}

#[test]
fn amendment_requires_at_least_one_field() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Try to amend with neither field provided
        assert_noop!(
            BelizeGovernance::amend_proposal(
                RuntimeOrigin::signed(ALICE),
                proposal_id,
                None,
                None
            ),
            Error::<Test>::AmendmentNotFound
        );
    });
}

#[test]
fn is_amendment_allowed_check_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Allowed before voting (voting starts at block 28,801)
        assert!(BelizeGovernance::is_amendment_allowed(proposal_id));

        // Not allowed after voting starts
        System::set_block_number(28_801);
        assert!(!BelizeGovernance::is_amendment_allowed(proposal_id));
    });
}

// ===== PARTICIPATION REWARDS TESTS =====

/// Helper function to fund the governance treasury for reward tests
fn fund_governance_treasury() {
    let treasury_account = BelizeGovernance::account_id();
    // Fund with 100 million DALLA (enough for all reward tests)
    let funding_amount = 100_000_000_000_000_000u128;
    let _ = Balances::make_free_balance_be(&treasury_account, funding_amount);
}

#[test]
fn claim_vote_reward_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Fund governance treasury for rewards
        fund_governance_treasury();

        // Create proposal and vote (voting starts at block 28,801)
        let proposal_id = create_test_proposal(ALICE, 10, 50);
        System::set_block_number(28_801);
        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            1, // vote yes
            0  // no conviction
        ));

        // Claim vote reward
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(BOB),
            0 // Vote reward type
        ));

        // Verify reward tracked (10 DALLA = 10_000_000_000_000)
        let claimed = BelizeGovernance::get_rewards_claimed(&BOB);
        assert_eq!(claimed, 10_000_000_000_000u128);
    });
}

#[test]
fn claim_proposal_reward_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Fund governance treasury for rewards
        fund_governance_treasury();

        // Alice creates proposal
        let _proposal_id = create_test_proposal(ALICE, 10, 50);

        // Claim proposal reward
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(ALICE),
            1 // Proposal reward type
        ));

        // Verify reward (100 DALLA = 100_000_000_000_000)
        let claimed = BelizeGovernance::get_rewards_claimed(&ALICE);
        assert_eq!(claimed, 100_000_000_000_000u128);
    });
}

#[test]
fn claim_council_reward_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Fund governance treasury for rewards
        fund_governance_treasury();

        // Add Bob as council member
        let member = CouncilMember {
            account: BOB,
            role: BoardRole::CitizenDelegate,
            term_end: 1000,
            is_rotating: true,
            community_rank: 100,
            pouw_contribution: 0,
            voting_weight: 100,
            term_start: 1,
            votes_received: 50,
            proposals_authored: 0,
            participation_rate: 0,
        };
        CouncilMembers::<Test>::insert(BOB, member);

        // Claim council reward
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(BOB),
            2 // Council reward type
        ));

        // Verify reward (500 DALLA = 500_000_000_000_000)
        let claimed = BelizeGovernance::get_rewards_claimed(&BOB);
        assert_eq!(claimed, 500_000_000_000_000u128);
    });
}

#[test]
fn cannot_claim_reward_without_participation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try to claim vote reward without voting
        assert_noop!(
            BelizeGovernance::claim_participation_reward(
                RuntimeOrigin::signed(ALICE),
                0
            ),
            Error::<Test>::NoRewardAvailable
        );
    });
}

#[test]
fn cannot_claim_reward_twice() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Fund governance treasury for rewards
        fund_governance_treasury();

        // Create proposal and vote (voting starts at block 28,801)
        let proposal_id = create_test_proposal(ALICE, 10, 50);
        System::set_block_number(28_801);
        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            1, // vote yes
            0  // no conviction
        ));

        // First claim succeeds
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(BOB),
            0
        ));

        // Second claim fails
        assert_noop!(
            BelizeGovernance::claim_participation_reward(
                RuntimeOrigin::signed(BOB),
                0
            ),
            Error::<Test>::RewardAlreadyClaimed
        );
    });
}

#[test]
fn total_rewards_distributed_updates() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Fund governance treasury for rewards
        fund_governance_treasury();

        // Initial total should be zero
        assert_eq!(BelizeGovernance::get_total_rewards_distributed(), 0);

        // Create and vote on proposal (voting starts at block 28,801)
        let proposal_id = create_test_proposal(ALICE, 10, 50);
        System::set_block_number(28_801);
        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            1, // vote yes
            0  // no conviction
        ));

        // Bob claims vote reward (10 DALLA)
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(BOB),
            0
        ));

        assert_eq!(BelizeGovernance::get_total_rewards_distributed(), 10_000_000_000_000u128);

        // Alice claims proposal reward (100 DALLA)
        assert_ok!(BelizeGovernance::claim_participation_reward(
            RuntimeOrigin::signed(ALICE),
            1
        ));

        // Total should be 110 DALLA
        assert_eq!(BelizeGovernance::get_total_rewards_distributed(), 110_000_000_000_000u128);
    });
}

// ===== PROPOSAL PRIORITY QUEUE TESTS =====

#[test]
fn set_proposal_priority_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Root can set priority
        assert_ok!(BelizeGovernance::set_proposal_priority(
            RuntimeOrigin::root(),
            proposal_id,
            4 // Critical priority
        ));

        // Verify in queue
        let queue = BelizeGovernance::get_priority_queue(ProposalPriority::Critical);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0], proposal_id);
    });
}

#[test]
fn priority_queue_supports_multiple_proposals() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create 3 proposals
        let proposal1 = create_test_proposal(ALICE, 200, 400);
        let proposal2 = create_test_proposal(BOB, 200, 400);
        let proposal3 = create_test_proposal(CHARLIE, 200, 400);

        // All go to high priority
        assert_ok!(BelizeGovernance::set_proposal_priority(
            RuntimeOrigin::root(),
            proposal1,
            3
        ));
        assert_ok!(BelizeGovernance::set_proposal_priority(
            RuntimeOrigin::root(),
            proposal2,
            3
        ));
        assert_ok!(BelizeGovernance::set_proposal_priority(
            RuntimeOrigin::root(),
            proposal3,
            3
        ));

        let queue = BelizeGovernance::get_priority_queue(ProposalPriority::High);
        assert_eq!(queue.len(), 3);
    });
}

#[test]
fn priority_queue_max_capacity_enforced() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create 50 proposals (max queue size)
        for _ in 0..50 {
            let proposal_id = create_test_proposal(ALICE, 200, 400);
            assert_ok!(BelizeGovernance::set_proposal_priority(
                RuntimeOrigin::root(),
                proposal_id,
                2 // Normal priority
            ));
        }

        // 51st proposal should fail
        let proposal_id = create_test_proposal(ALICE, 200, 400);
        assert_noop!(
            BelizeGovernance::set_proposal_priority(
                RuntimeOrigin::root(),
                proposal_id,
                2
            ),
            Error::<Test>::PriorityQueueFull
        );
    });
}

#[test]
fn invalid_priority_rejected() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Priority 0 is invalid
        assert_noop!(
            BelizeGovernance::set_proposal_priority(
                RuntimeOrigin::root(),
                proposal_id,
                0
            ),
            Error::<Test>::InvalidPriority
        );

        // Priority 5 is invalid (max is 4)
        assert_noop!(
            BelizeGovernance::set_proposal_priority(
                RuntimeOrigin::root(),
                proposal_id,
                5
            ),
            Error::<Test>::InvalidPriority
        );
    });
}

#[test]
fn only_root_can_set_priority() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proposal_id = create_test_proposal(ALICE, 200, 400);

        // Non-root cannot set priority
        assert_noop!(
            BelizeGovernance::set_proposal_priority(
                RuntimeOrigin::signed(ALICE),
                proposal_id,
                3
            ),
            BadOrigin
        );
    });
}

#[test]
fn proposal_priority_index_helper_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Low), 1);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Normal), 2);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::High), 3);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Critical), 4);
    });
}

// ===== HELPER FUNCTION =====

fn create_test_proposal(proposer: u64, _voting_start: u64, _voting_end: u64) -> u32 {
    let proposal_id = NextProposalId::<Test>::get();
    let title = b"Test Proposal".to_vec();
    let description = b"Test Description".to_vec();

    assert_ok!(BelizeGovernance::submit_proposal(
        RuntimeOrigin::signed(proposer),
        title,
        description,
        0, // proposal_type_index (Constitutional)
        0, // threshold_index (Simple)
        false, // is_emergency
    None,
    ));

    proposal_id
}
