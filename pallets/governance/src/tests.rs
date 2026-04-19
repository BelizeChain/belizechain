//! Unit tests for BelizeChain Governance pallet

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Shorthand: submit a standard Economic proposal from account `who`.
/// Returns the proposal_id (always 0 for the first proposal).
fn submit_standard_proposal(who: u64) -> u32 {
    assert_ok!(BelizeGovernance::submit_proposal(
        RuntimeOrigin::signed(who),
        b"Test Proposal".to_vec(),
        b"A description of the test proposal".to_vec(),
        1, // Economic
        0, // SimpleMajority
        false,
        None,
    ));
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// submit_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn submit_proposal_works() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        assert!(BelizeGovernance::proposals(proposal_id).is_some());
    });
}

#[test]
fn submit_proposal_blocked_for_restricted_account() {
    new_test_ext().execute_with(|| {
        // Account 999 is blocked by MockCompliance
        assert_noop!(
            BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(999),
                b"title".to_vec(),
                b"desc".to_vec(),
                1, 0, false, None,
            ),
            Error::<Test>::InsufficientCompliance
        );
    });
}

#[test]
fn submit_proposal_fails_with_invalid_proposal_type() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(1),
                b"title".to_vec(),
                b"desc".to_vec(),
                99, // invalid type index
                0,
                false,
                None,
            ),
            Error::<Test>::InvalidProposalType
        );
    });
}

#[test]
fn submit_proposal_fails_with_invalid_threshold() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(1),
                b"title".to_vec(),
                b"desc".to_vec(),
                0, // Constitutional
                99, // invalid threshold index
                false,
                None,
            ),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn submit_district_local_proposal_requires_district() {
    new_test_ext().execute_with(|| {
        // DistrictLocal (index 7) without a district index should fail
        assert_noop!(
            BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(1),
                b"title".to_vec(),
                b"desc".to_vec(),
                7,    // DistrictLocal
                0,    // SimpleMajority
                false,
                None, // no district
            ),
            Error::<Test>::DistrictRequired
        );
    });
}

#[test]
fn submit_district_local_proposal_with_valid_district() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(1),
            b"Local Budget Request".to_vec(),
            b"Allocate funds to Cayo district".to_vec(),
            7,       // DistrictLocal
            0,       // SimpleMajority
            false,
            Some(1), // Cayo district
        ));
    });
}

#[test]
fn submit_proposal_fails_with_invalid_district_index() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(1),
                b"title".to_vec(),
                b"desc".to_vec(),
                7,    // DistrictLocal
                0,
                false,
                Some(99), // invalid district index
            ),
            Error::<Test>::InvalidDistrict
        );
    });
}

#[test]
fn submit_proposal_increments_counter() {
    new_test_ext().execute_with(|| {
        submit_standard_proposal(1);
        // Second proposal gets id=1
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(2),
            b"Proposal 2".to_vec(),
            b"Another proposal".to_vec(),
            2, // Council
            0,
            false,
            None,
        ));
        assert!(BelizeGovernance::proposals(0).is_some());
        assert!(BelizeGovernance::proposals(1).is_some());
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// cast_vote
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cast_vote_aye_works() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        // Advance into the voting window
        run_to_block(proposal.voting_start + 1);

        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(2),
            id,
            0, // Aye
            1,
        ));

        let updated = BelizeGovernance::proposals(id).unwrap();
        assert!(updated.vote_tally.ayes > 0);
    });
}

#[test]
fn cast_vote_nay_works() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(2),
            id,
            1, // Nay
            1,
        ));

        let updated = BelizeGovernance::proposals(id).unwrap();
        assert!(updated.vote_tally.nays > 0);
    });
}

#[test]
fn cast_vote_abstain_works() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_ok!(BelizeGovernance::cast_vote(
            RuntimeOrigin::signed(2),
            id,
            2, // Abstain
            1,
        ));

        let updated = BelizeGovernance::proposals(id).unwrap();
        assert!(updated.vote_tally.abstentions > 0);
    });
}

#[test]
fn cast_vote_fails_before_voting_start() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        // Block 1 is before voting_start; don't advance.
        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1),
            Error::<Test>::VotingPeriodNotStarted
        );
    });
}

#[test]
fn cast_vote_fails_after_voting_end() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        // Advance past the voting end
        run_to_block(proposal.voting_end + 1);

        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn cast_vote_fails_when_already_voted() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1));
        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1),
            Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn cast_vote_fails_for_nonexistent_proposal() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(1), 9999, 0, 1),
            Error::<Test>::ProposalNotFound
        );
    });
}

#[test]
fn cast_vote_blocked_for_restricted_account() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(999), id, 0, 1),
            Error::<Test>::InsufficientCompliance
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// finalize_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn finalize_proposal_approves_when_ayes_win() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        // Accounts 1,2,3 all have community_rank set via genesis; vote Aye
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(1), id, 0, 1));
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1));
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(3), id, 1, 1)); // Nay

        run_to_block(proposal.voting_end + 1);

        assert_ok!(BelizeGovernance::finalize_proposal(
            RuntimeOrigin::signed(1),
            id
        ));

        let finalized = BelizeGovernance::proposals(id).unwrap();
        assert_eq!(finalized.status, crate::ProposalStatus::Approved);
    });
}

#[test]
fn finalize_proposal_rejects_when_nays_win() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        // Majority nay
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(1), id, 1, 1)); // Nay
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 1, 1)); // Nay
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(3), id, 0, 1)); // Aye

        run_to_block(proposal.voting_end + 1);

        assert_ok!(BelizeGovernance::finalize_proposal(
            RuntimeOrigin::signed(1),
            id
        ));

        let finalized = BelizeGovernance::proposals(id).unwrap();
        assert_eq!(finalized.status, crate::ProposalStatus::Rejected);
    });
}

#[test]
fn finalize_proposal_fails_before_voting_end() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_noop!(
            BelizeGovernance::finalize_proposal(RuntimeOrigin::signed(1), id),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn finalize_proposal_fails_for_nonexistent_proposal() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::finalize_proposal(RuntimeOrigin::signed(1), 9999),
            Error::<Test>::ProposalNotFound
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// update_community_rank
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn update_community_rank_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::update_community_rank(
            RuntimeOrigin::root(),
            4,
            500,
        ));
        assert_eq!(BelizeGovernance::community_ranks(4), 500);
    });
}

#[test]
fn update_community_rank_also_updates_council_member() {
    new_test_ext().execute_with(|| {
        // Account 1 is a council member from genesis
        assert_ok!(BelizeGovernance::update_community_rank(
            RuntimeOrigin::root(),
            1,
            999,
        ));
        let member = BelizeGovernance::council_members(1).expect("council member exists");
        assert_eq!(member.community_rank, 999);
    });
}

#[test]
fn update_community_rank_requires_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::update_community_rank(RuntimeOrigin::signed(1), 2, 100),
            sp_runtime::DispatchError::BadOrigin,
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// update_pouw_contribution
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn update_pouw_contribution_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::update_pouw_contribution(
            RuntimeOrigin::root(),
            4,
            250,
        ));
        assert_eq!(BelizeGovernance::pouw_contributions(4), 250);
    });
}

#[test]
fn update_pouw_contribution_also_updates_council_member() {
    new_test_ext().execute_with(|| {
        // Account 2 is a council member from genesis
        assert_ok!(BelizeGovernance::update_pouw_contribution(
            RuntimeOrigin::root(),
            2,
            888,
        ));
        let member = BelizeGovernance::council_members(2).expect("council member exists");
        assert_eq!(member.pouw_contribution, 888);
    });
}

#[test]
fn update_pouw_contribution_requires_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::update_pouw_contribution(RuntimeOrigin::signed(1), 2, 100),
            sp_runtime::DispatchError::BadOrigin,
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// council_override
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn council_override_works_for_emergency_proposal() {
    new_test_ext().execute_with(|| {
        // Submit an emergency proposal
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(1),
            b"Emergency Action".to_vec(),
            b"Critical system upgrade".to_vec(),
            4, // Emergency type
            0,
            true, // is_emergency = true
            None,
        ));

        // P0-18: JaguarMode must be active for council_override
        declare_and_activate_emergency(0u8, b"Critical threat", 24u32);

        assert_ok!(BelizeGovernance::council_override(
            RuntimeOrigin::root(),
            0,
            b"Urgent security patch".to_vec(),
        ));

        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.status, crate::ProposalStatus::Approved);
    });
}

#[test]
fn council_override_fails_for_non_emergency_proposal_outside_jaguar_mode() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        // Standard proposal, no emergency mode active — P0-18 requires JaguarMode
        assert_noop!(
            BelizeGovernance::council_override(
                RuntimeOrigin::root(),
                id,
                b"reason".to_vec(),
            ),
            Error::<Test>::NotInEmergencyMode
        );
    });
}

#[test]
fn council_override_fails_for_nonexistent_proposal() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::council_override(
                RuntimeOrigin::root(),
                9999,
                b"reason".to_vec(),
            ),
            Error::<Test>::ProposalNotFound
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// set_department_manager
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn set_department_manager_works() {
    new_test_ext().execute_with(|| {
        // Finance = index 0
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0,
            4,
        ));
        // Verify by checking that account 4 can now submit a department proposal
        // (DepartmentManagers storage should have account 4 for Finance)
        // We test the side-effect via submit_department_proposal
    });
}

#[test]
fn set_department_manager_requires_root() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::set_department_manager(RuntimeOrigin::signed(1), 0, 2),
            sp_runtime::DispatchError::BadOrigin,
        );
    });
}

#[test]
fn set_department_manager_fails_invalid_department() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 99, 1),
            Error::<Test>::InvalidDepartment
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Emergency / JaguarMode
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn declare_emergency_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::declare_emergency(
            RuntimeOrigin::root(),
            3,                              // SecurityThreat
            b"Critical network threat".to_vec(),
            24,                             // 24 hours
        ));
        // CONS-029: emergency starts pending during veto window
        let mode = BelizeGovernance::emergency_status();
        assert!(mode.is_some());
        assert!(!mode.as_ref().unwrap().active);
        assert!(mode.unwrap().is_pending);
        // Advance past veto window to activate
        run_to_block(System::block_number() + 101);
        let mode = BelizeGovernance::emergency_status();
        assert!(mode.is_some());
        assert!(mode.unwrap().active);
    });
}

#[test]
fn declare_emergency_twice_fails() {
    new_test_ext().execute_with(|| {
        // Declare and activate emergency
        declare_and_activate_emergency(3, b"First emergency", 24);
        assert_noop!(
            BelizeGovernance::declare_emergency(
                RuntimeOrigin::root(),
                3,
                b"Second emergency".to_vec(),
                24,
            ),
            Error::<Test>::EmergencyAlreadyActive
        );
    });
}

#[test]
fn end_emergency_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::declare_emergency(
            RuntimeOrigin::root(),
            3,
            b"Emergency".to_vec(),
            24,
        ));
        assert_ok!(BelizeGovernance::end_emergency(RuntimeOrigin::root()));
        let mode = BelizeGovernance::emergency_status();
        // After ending, mode is cleared or marked inactive
        assert!(mode.is_none_or(|m| !m.active));
    });
}

#[test]
fn end_emergency_without_active_emergency_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::end_emergency(RuntimeOrigin::root()),
            Error::<Test>::NoActiveEmergency
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Delegation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn delegate_vote_works() {
    new_test_ext().execute_with(|| {
        // Account 4 delegates to account 1 with no expiry
        assert_ok!(BelizeGovernance::delegate_vote(
            RuntimeOrigin::signed(4),
            1,
            None,
        ));
    });
}

#[test]
fn delegate_to_self_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::delegate_vote(RuntimeOrigin::signed(1), 1, None),
            Error::<Test>::CannotDelegateToSelf
        );
    });
}

#[test]
fn delegate_twice_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::delegate_vote(RuntimeOrigin::signed(4), 1, None));
        assert_noop!(
            BelizeGovernance::delegate_vote(RuntimeOrigin::signed(4), 2, None),
            Error::<Test>::DelegationAlreadyExists
        );
    });
}

#[test]
fn revoke_delegation_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::delegate_vote(RuntimeOrigin::signed(4), 1, None));
        assert_ok!(BelizeGovernance::revoke_delegation(RuntimeOrigin::signed(4)));
    });
}

#[test]
fn revoke_delegation_without_active_delegation_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::revoke_delegation(RuntimeOrigin::signed(4)),
            Error::<Test>::NoDelegationFound
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn submit_proposal_emits_event() {
    new_test_ext().execute_with(|| {
        submit_standard_proposal(1);
        let events = System::events();
        assert!(
            events.iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::BelizeGovernance(Event::ProposalSubmitted { proposal_id: 0, .. })
            )),
            "ProposalSubmitted event expected"
        );
    });
}

#[test]
fn cast_vote_emits_event() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let proposal = BelizeGovernance::proposals(id).unwrap();
        run_to_block(proposal.voting_start + 1);

        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 0, 1));
        let events = System::events();
        assert!(
            events.iter().any(|r| matches!(
                &r.event,
                RuntimeEvent::BelizeGovernance(Event::VoteCast { proposal_id: 0, voter: 2, .. })
            )),
            "VoteCast event expected"
        );
    });
}

#[test]
fn update_community_rank_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::update_community_rank(RuntimeOrigin::root(), 5, 77));
        assert_eq!(last_event(), RuntimeEvent::BelizeGovernance(Event::CommunityRankUpdated {
            account: 5,
            old_rank: 0,
            new_rank: 77,
        }));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// add_board_member / remove_board_member
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_board_member_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10u64,
            1, // TechnicalSteward
            1, // 1-year term
        ));
        assert!(BelizeGovernance::council_members(10u64).is_some());
        let events = System::events();
        assert!(events.iter().any(|r| matches!(
            &r.event,
            RuntimeEvent::BelizeGovernance(Event::BoardMemberAdded { member: 10, role_index: 1, .. })
        )));
    });
}

#[test]
fn add_board_member_invalid_term_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::add_board_member(RuntimeOrigin::root(), 10u64, 1, 3),
            Error::<Test>::InvalidTermDuration
        );
    });
}

#[test]
fn add_board_member_invalid_role_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::add_board_member(RuntimeOrigin::root(), 10u64, 7, 1),
            Error::<Test>::InvalidBoardRole
        );
    });
}

#[test]
fn remove_board_member_works() {
    new_test_ext().execute_with(|| {
        // Add first so we can remove
        assert_ok!(BelizeGovernance::add_board_member(RuntimeOrigin::root(), 10u64, 1, 1));
        assert!(BelizeGovernance::council_members(10u64).is_some());
        assert_ok!(BelizeGovernance::remove_board_member(
            RuntimeOrigin::root(),
            10u64,
            b"Term completed".to_vec(),
        ));
        assert!(BelizeGovernance::council_members(10u64).is_none());
    });
}

#[test]
fn remove_board_member_nonexistent_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::remove_board_member(RuntimeOrigin::root(), 99u64, b"".to_vec()),
            Error::<Test>::MemberNotFound
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// propose_treasury_spend / approve_treasury_spend / execute_treasury_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn propose_treasury_spend_works() {
    new_test_ext().execute_with(|| {
        // amount < 10K DALLA (10_000_000_000_000) → threshold = 1
        let amount: u128 = 1_000_000_000_000u128;
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1),
            5u64,
            amount,
            b"Test treasury proposal".to_vec(),
            None,
        ));
        assert!(crate::TreasurySpendProposals::<Test>::get(0).is_some());
    });
}

#[test]
fn approve_treasury_spend_works() {
    new_test_ext().execute_with(|| {
        let amount: u128 = 1_000_000_000_000u128;
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1), 5u64, amount, b"Test".to_vec(), None,
        ));
        // Council member 2 approves
        assert_ok!(BelizeGovernance::approve_treasury_spend(RuntimeOrigin::signed(2), 0));
        let proposal = crate::TreasurySpendProposals::<Test>::get(0).unwrap();
        assert_eq!(proposal.approvals.len(), 1);
    });
}

#[test]
fn approve_treasury_spend_non_council_fails() {
    new_test_ext().execute_with(|| {
        let amount: u128 = 1_000_000_000_000u128;
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1), 5u64, amount, b"Test".to_vec(), None,
        ));
        // Account 4 is NOT a council member
        assert_noop!(
            BelizeGovernance::approve_treasury_spend(RuntimeOrigin::signed(4), 0),
            Error::<Test>::NotCouncilMember
        );
    });
}

#[test]
fn execute_treasury_proposal_works() {
    new_test_ext().execute_with(|| {
        let amount: u128 = 1_000_000_000_000u128; // 1K DALLA, threshold = 1
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1), 5u64, amount, b"Test".to_vec(), None,
        ));
        // Council member 1 approves — threshold = 1 so now ready
        assert_ok!(BelizeGovernance::approve_treasury_spend(RuntimeOrigin::signed(1), 0));

        // Fund national treasury reserve and pallet account
        crate::NationalTreasuryReserve::<Test>::put(amount + 1);
        let treasury_account = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), treasury_account, amount + 1_000_000));

        let recipient_before = Balances::free_balance(5u64);
        assert_ok!(BelizeGovernance::execute_treasury_proposal(RuntimeOrigin::signed(2), 0));
        assert!(Balances::free_balance(5u64) > recipient_before);
        let proposal = crate::TreasurySpendProposals::<Test>::get(0).unwrap();
        assert!(proposal.executed);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// create_referendum / vote_on_referendum / finalize_referendum
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn create_referendum_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Should we build a new park?".to_vec(),
            b"Proposal to build a community park in Belmopan".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30,  // 30% quorum
            100, // 100 blocks duration
            None,
        ));
        assert!(crate::Referendums::<Test>::get(0).is_some());
    });
}

#[test]
fn vote_on_referendum_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Park?".to_vec(), b"Build a park".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30, 100, None,
        ));
        // Account 1 has community_rank=100, pouw=200 → weight=300
        assert_ok!(BelizeGovernance::vote_on_referendum(RuntimeOrigin::signed(1), 0, 0));
        let referendum = crate::Referendums::<Test>::get(0).unwrap();
        assert_eq!(referendum.vote_counts[0], 300); // weight=300 for option 0
        assert_eq!(referendum.total_votes, 300);
    });
}

#[test]
fn finalize_referendum_passes_with_quorum() {
    new_test_ext().execute_with(|| {
        // Account 1: weight = 100+200 = 300 out of 1000 eligible = 30% participation
        // Set quorum= 30 so quorum is met
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Park?".to_vec(), b"Build a park".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30, // 30% quorum
            10, // 10-block voting window
            None,
        ));
        assert_ok!(BelizeGovernance::vote_on_referendum(RuntimeOrigin::signed(1), 0, 0));
        // Advance past voting_end (current_block > voting_start + 10 = 11)
        run_to_block(12);
        assert_ok!(BelizeGovernance::finalize_referendum(RuntimeOrigin::signed(2), 0));
        let referendum = crate::Referendums::<Test>::get(0).unwrap();
        assert_eq!(referendum.status, crate::ReferendumStatus::Passed);
        assert_eq!(referendum.winning_option, Some(0));
    });
}

#[test]
fn finalize_referendum_fails_without_quorum() {
    new_test_ext().execute_with(|| {
        // Account 4 has community_rank=0, pouw=0 → weight=1
        // 1/1000 = 0% participation < quorum=50 → fails
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Park?".to_vec(), b"Build a park".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            50, // 50% quorum — won't be met
            10, None,
        ));
        assert_ok!(BelizeGovernance::vote_on_referendum(RuntimeOrigin::signed(4), 0, 0));
        run_to_block(12);
        assert_ok!(BelizeGovernance::finalize_referendum(RuntimeOrigin::signed(2), 0));
        let referendum = crate::Referendums::<Test>::get(0).unwrap();
        assert_eq!(referendum.status, crate::ReferendumStatus::Failed);
    });
}

#[test]
fn finalize_referendum_fails_before_voting_end() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Park?".to_vec(), b"Build a park".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30, 100, None,
        ));
        // Voting ends at block 101; we're at block 1
        assert_noop!(
            BelizeGovernance::finalize_referendum(RuntimeOrigin::signed(2), 0),
            Error::<Test>::VotingPeriodNotEnded
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// update_chain_parameter
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn update_chain_parameter_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::update_chain_parameter(
            RuntimeOrigin::root(),
            b"max_validators".to_vec(),
            100u64,
        ));
        let key: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<32>> =
            b"max_validators".to_vec().try_into().unwrap();
        assert_eq!(crate::ChainParameters::<Test>::get(&key), Some(100u64));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// set_proposal_priority
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn set_proposal_priority_works() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        assert_ok!(BelizeGovernance::set_proposal_priority(
            RuntimeOrigin::root(),
            id,
            3, // High
        ));
        let events = System::events();
        assert!(events.iter().any(|r| matches!(
            &r.event,
            RuntimeEvent::BelizeGovernance(Event::ProposalQueuedWithPriority { proposal_id: 0, priority_index: 3 })
        )));
    });
}

#[test]
fn set_proposal_priority_invalid_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        assert_noop!(
            BelizeGovernance::set_proposal_priority(RuntimeOrigin::root(), id, 5),
            Error::<Test>::InvalidPriority
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// amend_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn amend_proposal_works() {
    new_test_ext().execute_with(|| {
        // Account 1 submits a proposal (voting hasn't started yet at block 1)
        let id = submit_standard_proposal(1);

        let new_title: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<256>> =
            b"Updated Title".to_vec().try_into().unwrap();

        assert_ok!(BelizeGovernance::amend_proposal(
            RuntimeOrigin::signed(1),
            id,
            Some(new_title.clone()),
            None,
        ));

        let proposal = BelizeGovernance::proposals(id).unwrap();
        assert_eq!(proposal.title, new_title);
    });
}

#[test]
fn amend_proposal_wrong_author_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        let new_title: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<256>> =
            b"Hijacked Title".to_vec().try_into().unwrap();
        // Account 2 is NOT the proposer
        assert_noop!(
            BelizeGovernance::amend_proposal(RuntimeOrigin::signed(2), id, Some(new_title), None),
            Error::<Test>::NotProposalAuthor
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// claim_participation_reward
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn claim_participation_reward_council_works() {
    new_test_ext().execute_with(|| {
        // Account 1 is a genesis council member (reward_type=2 = 500 DALLA)
        let reward_amount: u128 = 500_000_000_000_000u128;

        // Fund the governance treasury (pallet account) so the transfer succeeds
        let treasury_account = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(
            RuntimeOrigin::root(),
            treasury_account,
            reward_amount + 1_000_000,
        ));

        let before = Balances::free_balance(1u64);
        assert_ok!(BelizeGovernance::claim_participation_reward(RuntimeOrigin::signed(1), 2));
        assert_eq!(Balances::free_balance(1u64), before + reward_amount);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// submit_department_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn submit_department_proposal_works() {
    new_test_ext().execute_with(|| {
        // Set account 4 as Finance (dept=0) manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0u8, // Finance
            4u64,
        ));
        // Account 4 has 50B balance — enough for MinimumDeposit (1B)
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(4),
            0u8,
            b"Finance Reform".to_vec(),
            b"Description of the finance reform".to_vec(),
            1u8, // Economic
            0u8, // SimpleMajority
            false,
        ));
        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.department, Some(crate::Department::Finance));
        assert_eq!(proposal.proposer, 4u64);
    });
}

#[test]
fn submit_department_proposal_not_manager_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0u8,
            4u64,
        ));
        // Account 5 is NOT the Finance manager
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(5),
                0u8,
                b"title".to_vec(),
                b"desc".to_vec(),
                1u8,
                0u8,
                false,
            ),
            Error::<Test>::NotDepartmentManager
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// approve_cross_department
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn approve_cross_department_works() {
    new_test_ext().execute_with(|| {
        // Finance (0) manager = 4, Education (1) manager = 5
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0u8, 4u64));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1u8, 5u64));
        // Account 4 submits a proposal requiring cross-department approval
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(4),
            0u8,
            b"Cross-Dept Reform".to_vec(),
            b"Requires education department approval too".to_vec(),
            1u8,
            0u8,
            true, // requires_cross_approval
        ));
        // Account 5 (Education manager) approves the proposal
        assert_ok!(BelizeGovernance::approve_cross_department(
            RuntimeOrigin::signed(5),
            0u32,  // proposal_id
            1u8,   // Education = dept index 1
        ));
        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.cross_approved_by.len(), 1);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// nominate_for_delegate
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nominate_for_delegate_works() {
    new_test_ext().execute_with(|| {
        // Activate a delegate election round
        crate::CurrentElection::<Test>::put(1u64);
        // Account 1 nominates account 4
        assert_ok!(BelizeGovernance::nominate_for_delegate(
            RuntimeOrigin::signed(1),
            4u64,
        ));
        assert!(crate::DelegateNominees::<Test>::contains_key(4u64));
    });
}

#[test]
fn nominate_for_delegate_no_election_fails() {
    new_test_ext().execute_with(|| {
        // No CurrentElection set → should fail
        assert_noop!(
            BelizeGovernance::nominate_for_delegate(RuntimeOrigin::signed(1), 4u64),
            Error::<Test>::NoActiveElection
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// vote_for_delegate
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vote_for_delegate_works() {
    new_test_ext().execute_with(|| {
        crate::CurrentElection::<Test>::put(1u64);
        // Nominate account 4
        assert_ok!(BelizeGovernance::nominate_for_delegate(
            RuntimeOrigin::signed(1),
            4u64,
        ));
        // Account 2 votes for account 4
        assert_ok!(BelizeGovernance::vote_for_delegate(
            RuntimeOrigin::signed(2),
            4u64,
        ));
        assert_eq!(crate::DelegateNominees::<Test>::get(4u64), 1u32);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_proposal_works() {
    new_test_ext().execute_with(|| {
        use crate::{
            GovernanceParameter, Proposal, ProposalAction, ProposalStatus, ProposalType,
            VoteTally, VotingThreshold,
        };
        // Inject an Approved proposal with a ParameterChange action directly into storage
        let proposal = Proposal {
            id: 0u32,
            proposer: 1u64,
            title: b"Param Change".to_vec().try_into().unwrap(),
            description: b"Update voting period param".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128,
            voting_start: 1u64,
            voting_end: 1_000_000u64,
            vote_tally: VoteTally {
                ayes: 100,
                nays: 0,
                abstentions: 0,
                total_weight: 100,
                participation: 100,
            },
            status: ProposalStatus::Approved,
            is_emergency: false,
            department: None,
            district: None,
            requires_cross_approval: false,
            cross_approved_by: Default::default(),
            action: Some(ProposalAction::ParameterChange {
                parameter: GovernanceParameter::VotingPeriod,
                new_value: 50_400u32,
            }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        // Account 1 is a genesis council member — can execute
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        let updated = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(updated.status, ProposalStatus::Executed);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// start_district_election
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn start_district_election_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(),
            0u8,   // Belize district
            2u32,  // 2 seats
            100u32,
            200u32,
        ));
        assert!(crate::DistrictElections::<Test>::contains_key(crate::BelizeDistrict::Belize));
    });
}

#[test]
fn start_district_election_duplicate_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(), 0, 2, 100, 200,
        ));
        assert_noop!(
            BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200),
            Error::<Test>::ElectionAlreadyActive
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// register_candidate
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn register_candidate_works() {
    new_test_ext().execute_with(|| {
        // Election starts at block 1; registration_end = 1 + 100 = 101
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(), 0, 2, 100, 200,
        ));
        // Block 1 is within registration period (< 101)
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(1),
            0u8,
            b"Roads and schools for all".to_vec(),
        ));
        // election.id == 1 (block number when started)
        assert!(crate::ElectionCandidates::<Test>::contains_key(1u64, 1u64));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// vote_in_district_election
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vote_in_district_election_works() {
    new_test_ext().execute_with(|| {
        // Start election: reg_end=101, voting_start=101, voting_end=301
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(), 0, 2, 100, 200,
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(1),
            0u8,
            b"platform".to_vec(),
        ));
        // Advance past registration_end into voting phase
        System::set_block_number(101);
        // Account 2 votes for account 1
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(2),
            0u8,
            1u64,
        ));
        let election = crate::DistrictElections::<Test>::get(crate::BelizeDistrict::Belize).unwrap();
        assert_eq!(election.total_votes, 1);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// finalize_district_election
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn finalize_district_election_works() {
    new_test_ext().execute_with(|| {
        // Start election: reg_end=101, voting_start=101, voting_end=301
        assert_ok!(BelizeGovernance::start_district_election(
            RuntimeOrigin::root(), 0, 2, 100, 200,
        ));
        assert_ok!(BelizeGovernance::register_candidate(
            RuntimeOrigin::signed(1),
            0u8,
            b"platform".to_vec(),
        ));
        // Vote during voting phase
        System::set_block_number(101);
        assert_ok!(BelizeGovernance::vote_in_district_election(
            RuntimeOrigin::signed(2),
            0u8,
            1u64,
        ));
        // Advance past voting_end
        System::set_block_number(302);
        assert_ok!(BelizeGovernance::finalize_district_election(RuntimeOrigin::root(), 0u8));
        // Election status should be Finalized
        let election = crate::DistrictElections::<Test>::get(crate::BelizeDistrict::Belize).unwrap();
        assert_eq!(election.status, crate::ElectionStatus::Finalized);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// allocate_district_budget
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn allocate_district_budget_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            0u8,             // Belize district
            1_000_000u128,
            1_000u32,        // 1000 blocks fiscal year
        ));
        let budget = crate::DistrictBudgets::<Test>::get(crate::BelizeDistrict::Belize).unwrap();
        assert_eq!(budget.allocated, 1_000_000u128);
        assert!(budget.is_active);
    });
}

#[test]
fn allocate_district_budget_duplicate_active_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(), 0u8, 1_000_000u128, 100u32,
        ));
        assert_noop!(
            BelizeGovernance::allocate_district_budget(
                RuntimeOrigin::root(), 0u8, 500_000u128, 100u32,
            ),
            Error::<Test>::DistrictBudgetAlreadyExists
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// transfer_district_budget
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn transfer_district_budget_works() {
    new_test_ext().execute_with(|| {
        // Allocate budget for Belize (0)
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(), 0u8, 1_000_000u128, 1_000u32,
        ));
        // Transfer 300_000 from Belize (0) to Cayo (1)
        assert_ok!(BelizeGovernance::transfer_district_budget(
            RuntimeOrigin::root(),
            0u8,
            1u8,
            300_000u128,
            b"Emergency reallocation to Cayo".to_vec(),
        ));
        let from_b = crate::DistrictBudgets::<Test>::get(crate::BelizeDistrict::Belize).unwrap();
        let to_b   = crate::DistrictBudgets::<Test>::get(crate::BelizeDistrict::Cayo).unwrap();
        assert_eq!(from_b.allocated, 700_000u128);
        assert_eq!(to_b.allocated, 300_000u128);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_emergency_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_emergency_proposal_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        // Activate JaguarMode (CouncilOrigin = EnsureRoot in mock)
        declare_and_activate_emergency(0u8, b"Test hurricane", 24u32);
        // Inject emergency proposal with ayes=70, nays=30 → 70% approval (>= 66%)
        let proposal = Proposal {
            id: 5u32,
            proposer: 1u64,
            title: b"Emergency Measure".to_vec().try_into().unwrap(),
            description: b"desc".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Emergency,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128,
            voting_start: 1u64,
            voting_end: 1_000_000u64,
            vote_tally: VoteTally {
                ayes: 70,
                nays: 30,
                abstentions: 0,
                total_weight: 100,
                participation: 100,
            },
            status: ProposalStatus::Voting,
            is_emergency: true,
            department: None,
            district: None,
            requires_cross_approval: false,
            cross_approved_by: Default::default(),
            action: None,
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(5u32, proposal);
        assert_ok!(BelizeGovernance::execute_emergency_proposal(RuntimeOrigin::root(), 5u32));
        let updated = BelizeGovernance::proposals(5).unwrap();
        assert_eq!(updated.status, ProposalStatus::Executed);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// fast_track_referendum
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fast_track_referendum_works() {
    new_test_ext().execute_with(|| {
        // Activate JaguarMode
        declare_and_activate_emergency(0u8, b"Emergency", 24u32);
        // Create referendum (no deposit required, just compliance)
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(1),
            b"Referendum Title".to_vec(),
            b"Should we do X?".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            50u8,
            10_000u32,
            None,
        ));
        let before = crate::Referendums::<Test>::get(0u32).unwrap().voting_end;
        // Fast-track: reduces voting_end to current_block + 10_800
        assert_ok!(BelizeGovernance::fast_track_referendum(RuntimeOrigin::root(), 0u32));
        let after = crate::Referendums::<Test>::get(0u32).unwrap().voting_end;
        // New deadline must be earlier than original (10_800 < 10_000 + launch_period)
        let current = System::block_number();
        assert_eq!(after, current + 10_800);
        let _ = before; // suppress unused warning
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// emergency_override_proposal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn emergency_override_proposal_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        // Activate JaguarMode
        declare_and_activate_emergency(0u8, b"Emergency", 24u32);
        // Inject a pending proposal to override
        let proposal = Proposal {
            id: 7u32,
            proposer: 1u64,
            title: b"Override candidate".to_vec().try_into().unwrap(),
            description: b"desc".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128,
            voting_start: 1u64,
            voting_end: 1_000u64,
            vote_tally: VoteTally {
                ayes: 0, nays: 0, abstentions: 0, total_weight: 0, participation: 0,
            },
            status: ProposalStatus::Pending,
            is_emergency: false,
            department: None,
            district: None,
            requires_cross_approval: false,
            cross_approved_by: Default::default(),
            action: None,
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(7u32, proposal);
        // Root force-executes the proposal
        assert_ok!(BelizeGovernance::emergency_override_proposal(
            RuntimeOrigin::root(),
            7u32,
            true, // execute
            b"Force execution during hurricane".to_vec(),
        ));
        let updated = BelizeGovernance::proposals(7).unwrap();
        assert_eq!(updated.status, ProposalStatus::Executed);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// commit_vote
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn commit_vote_works() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        // Advance to voting_start: block 1 + LaunchPeriod (28_800) = 28_801
        System::set_block_number(28_801);
        // Commitment = blake2_256([vote_choice_byte=0(Aye)] ++ [salt])
        let salt = [1u8; 32];
        let mut preimage = [0u8; 33];
        preimage[0] = 0u8; // Aye
        preimage[1..].copy_from_slice(&salt);
        let commitment = sp_io::hashing::blake2_256(&preimage);
        assert_ok!(BelizeGovernance::commit_vote(
            RuntimeOrigin::signed(1),
            proposal_id,
            commitment,
        ));
        assert!(crate::VoteCommitments::<Test>::contains_key(proposal_id, 1u64));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// reveal_vote
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn reveal_vote_works() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        System::set_block_number(28_801);
        let salt = [2u8; 32];
        let mut preimage = [0u8; 33];
        preimage[0] = 0u8; // Aye
        preimage[1..].copy_from_slice(&salt);
        let commitment = sp_io::hashing::blake2_256(&preimage);
        // Commit first
        assert_ok!(BelizeGovernance::commit_vote(
            RuntimeOrigin::signed(1),
            proposal_id,
            commitment,
        ));
        // Now reveal
        assert_ok!(BelizeGovernance::reveal_vote(
            RuntimeOrigin::signed(1),
            proposal_id,
            0u8,  // Aye
            salt,
            1u8,  // conviction = 1
        ));
        // Vote recorded; commitment removed
        assert!(crate::Votes::<Test>::contains_key(proposal_id, 1u64));
        assert!(!crate::VoteCommitments::<Test>::contains_key(proposal_id, 1u64));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// submit_department_proposal — additional error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn submit_department_proposal_invalid_dept_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(4),
                8u8, // invalid index
                b"title".to_vec(), b"desc".to_vec(), 1, 0, false,
            ),
            Error::<Test>::InvalidDepartment
        );
    });
}

#[test]
fn submit_department_proposal_no_manager_set_fails() {
    new_test_ext().execute_with(|| {
        // No manager set for Finance (index=0)
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(4),
                0u8, b"title".to_vec(), b"desc".to_vec(), 1, 0, false,
            ),
            Error::<Test>::DepartmentNotFound
        );
    });
}

#[test]
fn submit_department_proposal_invalid_proposal_type_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 4));
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(4),
                0u8, b"title".to_vec(), b"desc".to_vec(), 99u8, 0, false,
            ),
            Error::<Test>::InvalidProposalType
        );
    });
}

#[test]
fn submit_department_proposal_invalid_threshold_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 4));
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(4),
                0u8, b"title".to_vec(), b"desc".to_vec(), 1u8, 99u8, false,
            ),
            Error::<Test>::InvalidThreshold
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// approve_cross_department — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn approve_cross_department_dept_not_found_fails() {
    new_test_ext().execute_with(|| {
        // No manager set for Education (1)
        assert_noop!(
            BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(5), 0u32, 1u8),
            Error::<Test>::DepartmentNotFound
        );
    });
}

#[test]
fn approve_cross_department_proposal_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 5));
        assert_noop!(
            BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(5), 999u32, 1u8),
            Error::<Test>::ProposalNotFound
        );
    });
}

#[test]
fn approve_cross_department_no_cross_approval_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 4));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 5));
        // Submit WITHOUT requires_cross_approval
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(4), 0, b"t".to_vec(), b"d".to_vec(), 1, 0, false,
        ));
        assert_noop!(
            BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(5), 0u32, 1u8),
            Error::<Test>::CrossApprovalNotRequired
        );
    });
}

#[test]
fn approve_cross_department_already_approved_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 4));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 5));
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(4), 0, b"t".to_vec(), b"d".to_vec(), 1, 0, true,
        ));
        assert_ok!(BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(5), 0, 1));
        assert_noop!(
            BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(5), 0u32, 1u8),
            Error::<Test>::DepartmentAlreadyApproved
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_proposal — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_proposal_not_council_member_fails() {
    new_test_ext().execute_with(|| {
        // Account 4 is NOT a council member
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(4), 0u32),
            Error::<Test>::NotCouncilMember
        );
    });
}

#[test]
fn execute_proposal_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 999u32),
            Error::<Test>::ProposalNotFound
        );
    });
}

#[test]
fn execute_proposal_not_approved_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        let proposal = Proposal {
            id: 0, proposer: 1, status: ProposalStatus::Pending,
            title: b"t".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 0, nays: 0, abstentions: 0, total_weight: 0, participation: 0 },
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: None, executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::ProposalNotApproved
        );
    });
}

#[test]
fn execute_proposal_no_action_defined_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        let proposal = Proposal {
            id: 0, proposer: 1, status: ProposalStatus::Approved,
            title: b"t".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: None, // no action
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::NoActionDefined
        );
    });
}

#[test]
fn execute_proposal_already_executed_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        let proposal = Proposal {
            id: 0, proposer: 1, status: ProposalStatus::Executed,
            title: b"t".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: None,
            executed_at: Some(1u64), // already executed
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::AlreadyExecuted
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// start_district_election — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn start_district_election_invalid_district_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::start_district_election(RuntimeOrigin::root(), 6u8, 2, 100, 200),
            Error::<Test>::InvalidDistrict
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// register_candidate — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn register_candidate_past_registration_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200));
        // Advance past registration_end (1 + 100 = 101)
        System::set_block_number(105);
        assert_noop!(
            BelizeGovernance::register_candidate(RuntimeOrigin::signed(1), 0u8, b"platform".to_vec()),
            Error::<Test>::NotInRegistrationPhase
        );
    });
}

#[test]
fn register_candidate_duplicate_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200));
        assert_ok!(BelizeGovernance::register_candidate(RuntimeOrigin::signed(1), 0u8, b"platform".to_vec()));
        assert_noop!(
            BelizeGovernance::register_candidate(RuntimeOrigin::signed(1), 0u8, b"platform2".to_vec()),
            Error::<Test>::CandidateAlreadyRegistered
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// vote_in_district_election — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vote_in_district_election_registration_phase_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200));
        // Block 1 < registration_end (101) → still in Registration phase
        assert_noop!(
            BelizeGovernance::vote_in_district_election(RuntimeOrigin::signed(2), 0u8, 1u64),
            Error::<Test>::NotInVotingPhase
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// finalize_district_election — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn finalize_district_election_too_early_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200));
        // voting_end = 1 + 100 + 200 = 301; still at block 1
        assert_noop!(
            BelizeGovernance::finalize_district_election(RuntimeOrigin::root(), 0u8),
            Error::<Test>::VotingPeriodNotEnded
        );
    });
}

#[test]
fn finalize_district_election_no_candidates_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0, 2, 100, 200));
        System::set_block_number(302); // past voting_end
        assert_noop!(
            BelizeGovernance::finalize_district_election(RuntimeOrigin::root(), 0u8),
            Error::<Test>::InsufficientCandidates
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// commit_vote — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn commit_vote_before_voting_start_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        // Block 1 < voting_start (28_801) → VotingPeriodNotStarted
        let commitment = [0u8; 32];
        assert_noop!(
            BelizeGovernance::commit_vote(RuntimeOrigin::signed(1), proposal_id, commitment),
            Error::<Test>::VotingPeriodNotStarted
        );
    });
}

#[test]
fn commit_vote_already_committed_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        System::set_block_number(28_801);
        let commitment = [1u8; 32];
        assert_ok!(BelizeGovernance::commit_vote(RuntimeOrigin::signed(1), proposal_id, commitment));
        assert_noop!(
            BelizeGovernance::commit_vote(RuntimeOrigin::signed(1), proposal_id, commitment),
            Error::<Test>::VoteAlreadyCommitted
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// reveal_vote — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn reveal_vote_no_commitment_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        System::set_block_number(28_801);
        // No commit made → NoCommitmentFound
        assert_noop!(
            BelizeGovernance::reveal_vote(
                RuntimeOrigin::signed(1), proposal_id, 0u8, [0u8; 32], 1u8,
            ),
            Error::<Test>::NoCommitmentFound
        );
    });
}

#[test]
fn reveal_vote_wrong_hash_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        System::set_block_number(28_801);
        let correct_salt = [3u8; 32];
        // Commit with correct_salt
        let mut preimage = [0u8; 33];
        preimage[0] = 0u8;
        preimage[1..].copy_from_slice(&correct_salt);
        let commitment = sp_io::hashing::blake2_256(&preimage);
        assert_ok!(BelizeGovernance::commit_vote(RuntimeOrigin::signed(1), proposal_id, commitment));
        // Reveal with WRONG salt
        assert_noop!(
            BelizeGovernance::reveal_vote(
                RuntimeOrigin::signed(1), proposal_id, 0u8, [9u8; 32], 1u8,
            ),
            Error::<Test>::CommitmentHashMismatch
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// apply_pending_runtime_upgrade — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn apply_pending_runtime_upgrade_no_pending_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::apply_pending_runtime_upgrade(
                RuntimeOrigin::root(), b"some_code".to_vec(),
            ),
            Error::<Test>::RuntimeUpgradeNotPending
        );
    });
}

#[test]
fn apply_pending_runtime_upgrade_hash_mismatch_fails() {
    new_test_ext().execute_with(|| {
        // Store a fake approved hash
        crate::PendingRuntimeUpgrade::<Test>::put([1u8; 32]);
        assert_noop!(
            BelizeGovernance::apply_pending_runtime_upgrade(
                RuntimeOrigin::root(), b"wrong_code".to_vec(),
            ),
            Error::<Test>::RuntimeCodeHashMismatch
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// allocate_district_budget — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn allocate_district_budget_zero_fiscal_year_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::allocate_district_budget(
                RuntimeOrigin::root(), 0u8, 1_000_000u128, 0u32,
            ),
            Error::<Test>::InvalidFiscalYearPeriod
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// transfer_district_budget — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn transfer_district_budget_no_source_fails() {
    new_test_ext().execute_with(|| {
        // Belize (0) has no budget → DistrictBudgetNotFound
        assert_noop!(
            BelizeGovernance::transfer_district_budget(
                RuntimeOrigin::root(), 0u8, 1u8, 100u128, b"realloc".to_vec(),
            ),
            Error::<Test>::DistrictBudgetNotFound
        );
    });
}

#[test]
fn transfer_district_budget_insufficient_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::allocate_district_budget(RuntimeOrigin::root(), 0, 100u128, 1000));
        // Try to transfer 200 from Belize (0) which only has 100 allocated
        assert_noop!(
            BelizeGovernance::transfer_district_budget(
                RuntimeOrigin::root(), 0u8, 1u8, 200u128, b"too much".to_vec(),
            ),
            Error::<Test>::TransferExceedsSourceBudget
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// claim_participation_reward — vote and proposal type
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn claim_participation_reward_vote_type_works() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_standard_proposal(1);
        // Advance to voting_start and cast a vote from account 4
        System::set_block_number(28_801);
        assert_ok!(BelizeGovernance::cast_vote(RuntimeOrigin::signed(4), proposal_id, 0u8, 1u8));
        let reward: u128 = 10_000_000_000_000;
        let treasury = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), treasury, reward + 1_000));
        let before = Balances::free_balance(4u64);
        assert_ok!(BelizeGovernance::claim_participation_reward(RuntimeOrigin::signed(4), 0u8));
        assert_eq!(Balances::free_balance(4u64), before + reward);
    });
}

#[test]
fn claim_participation_reward_proposal_type_works() {
    new_test_ext().execute_with(|| {
        submit_standard_proposal(4); // account 4 submits a proposal
        let reward: u128 = 100_000_000_000_000;
        let treasury = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), treasury, reward + 1_000));
        let before = Balances::free_balance(4u64);
        assert_ok!(BelizeGovernance::claim_participation_reward(RuntimeOrigin::signed(4), 1u8));
        assert_eq!(Balances::free_balance(4u64), before + reward);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// nominate_for_delegate — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nominate_for_delegate_non_compliant_nominee_fails() {
    new_test_ext().execute_with(|| {
        crate::CurrentElection::<Test>::put(1u64);
        // nominee 999 is blocked by MockCompliance
        assert_noop!(
            BelizeGovernance::nominate_for_delegate(RuntimeOrigin::signed(1), 999u64),
            Error::<Test>::InsufficientCompliance
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// vote_for_delegate — error paths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vote_for_delegate_nominee_not_found_fails() {
    new_test_ext().execute_with(|| {
        crate::CurrentElection::<Test>::put(1u64);
        // Account 4 was never nominated → NomineeNotFound
        assert_noop!(
            BelizeGovernance::vote_for_delegate(RuntimeOrigin::signed(2), 4u64),
            Error::<Test>::NomineeNotFound
        );
    });
}

#[test]
fn vote_for_delegate_already_voted_fails() {
    new_test_ext().execute_with(|| {
        crate::CurrentElection::<Test>::put(1u64);
        assert_ok!(BelizeGovernance::nominate_for_delegate(RuntimeOrigin::signed(1), 4u64));
        assert_ok!(BelizeGovernance::vote_for_delegate(RuntimeOrigin::signed(2), 4u64));
        assert_noop!(
            BelizeGovernance::vote_for_delegate(RuntimeOrigin::signed(2), 4u64),
            Error::<Test>::AlreadyVotedInElection
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// execute_proposal – execute_action branch coverage
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn execute_proposal_treasury_spend_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        let treasury_acct = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), treasury_acct, 10_000_000_000u128));
        let before = Balances::free_balance(5u64);
        let spend_amount: u128 = 1_000u128;
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"Treasury".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::TreasurySpend { recipient: 5u64, amount: spend_amount }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        assert!(Balances::free_balance(5u64) > before);
        let updated = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(updated.status, crate::ProposalStatus::Executed);
    });
}

#[test]
fn execute_proposal_treasury_spend_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        // Don't fund the treasury — balance is 0
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"T".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::TreasurySpend { recipient: 5u64, amount: 999_999_999_999u128 }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::InsufficientTreasuryBalance
        );
    });
}

#[test]
fn execute_proposal_runtime_upgrade_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        let code_hash: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<32>> =
            [2u8; 32].to_vec().try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"Upgrade".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Technical,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::RuntimeUpgrade { code_hash }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        let pending = crate::PendingRuntimeUpgrade::<Test>::get();
        assert!(pending.is_some());
        assert_eq!(pending.unwrap(), [2u8; 32]);
    });
}

#[test]
fn execute_proposal_runtime_upgrade_bad_hash_size_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        // 16 bytes — not 32; execute_runtime_upgrade checks len == 32
        let code_hash: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<32>> =
            [9u8; 16].to_vec().try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"Upgrade".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Technical,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::RuntimeUpgrade { code_hash }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::RuntimeCodeTooLarge
        );
    });
}

#[test]
fn execute_proposal_department_action_noop_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, Department};
        use codec::Encode as _;
        let dept_call = crate::GovDepartmentCall::<u64, u128>::NoOp;
        let call_data: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<1024>> =
            dept_call.encode().try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"DeptNoOp".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: Some(Department::Finance), district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::DepartmentAction { department: Department::Finance, call_data }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
    });
}

#[test]
fn execute_proposal_department_action_update_policy_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, Department};
        use codec::Encode as _;
        let policy_key: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<64>> =
            b"max_budget".to_vec().try_into().unwrap();
        let policy_val: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<256>> =
            b"1000000".to_vec().try_into().unwrap();
        let dept_call = crate::GovDepartmentCall::<u64, u128>::UpdatePolicy {
            policy_key, policy_value: policy_val,
        };
        let call_data: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<1024>> =
            dept_call.encode().try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"Policy".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: Some(Department::Education), district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::DepartmentAction { department: Department::Education, call_data }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
    });
}

#[test]
fn execute_proposal_department_action_spend_budget_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, Department};
        use codec::Encode as _;
        let treasury_acct = BelizeGovernance::account_id();
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), treasury_acct, 10_000_000_000u128));
        let dept_call = crate::GovDepartmentCall::<u64, u128>::SpendBudget { recipient: 5u64, amount: 500u128 };
        let call_data: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<1024>> =
            dept_call.encode().try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"SpendBudget".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: Some(Department::Finance), district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::DepartmentAction { department: Department::Finance, call_data }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
    });
}

#[test]
fn execute_proposal_department_action_invalid_call_data_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, Department};
        // 4 bytes of 0xFF cannot be SCALE-decoded as a valid DepartmentCall
        let call_data: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<1024>> =
            vec![0xFF, 0xFF, 0xFF, 0xFF].try_into().unwrap();
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"T".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: Some(Department::Finance), district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::DepartmentAction { department: Department::Finance, call_data }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::InvalidCallData
        );
    });
}

#[test]
fn execute_proposal_emergency_activate_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, EmergencyActionType};
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"E".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Emergency,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: true, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::EmergencyAction { action_type: EmergencyActionType::ActivateEmergency }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        assert!(crate::JaguarMode::<Test>::get().is_some());
    });
}

#[test]
fn execute_proposal_emergency_deactivate_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, EmergencyActionType};
        // Pre-set JaguarMode
        assert_ok!(BelizeGovernance::declare_emergency(RuntimeOrigin::root(), 0u8, b"test".to_vec(), 24u32));
        assert!(crate::JaguarMode::<Test>::get().is_some());
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"E".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Emergency,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: true, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::EmergencyAction { action_type: EmergencyActionType::DeactivateEmergency }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        assert!(crate::JaguarMode::<Test>::get().is_none());
    });
}

#[test]
fn execute_proposal_emergency_freeze_account_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, EmergencyActionType};
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"F".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Emergency,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: true, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::EmergencyAction { action_type: EmergencyActionType::FreezeAccount }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
    });
}

#[test]
fn execute_proposal_emergency_halt_and_resume_works() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, EmergencyActionType};
        let make_proposal = |id: u32, action_type: EmergencyActionType| Proposal {
            id, proposer: 1u64,
            title: b"H".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Emergency,
            threshold: VotingThreshold::Supermajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: true, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::EmergencyAction { action_type }),
            executed_at: None,
        };
        // HaltGovernance
        crate::Proposals::<Test>::insert(0u32, make_proposal(0, EmergencyActionType::HaltGovernance));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        // ResumeGovernance
        crate::Proposals::<Test>::insert(1u32, make_proposal(1, EmergencyActionType::ResumeGovernance));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 1u32));
        // UnfreezeAccount
        crate::Proposals::<Test>::insert(2u32, make_proposal(2, EmergencyActionType::UnfreezeAccount));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 2u32));
    });
}

#[test]
fn execute_proposal_parameter_change_invalid_value_fails() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, GovernanceParameter};
        // VotingPeriod valid range: 3600..=604800 — use 100 (out of range)
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"P".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::ParameterChange { parameter: GovernanceParameter::VotingPeriod, new_value: 100u32 }),
            executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        assert_noop!(
            BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32),
            Error::<Test>::InvalidParameterValue
        );
    });
}

#[test]
fn execute_proposal_all_parameter_types_work() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalAction, ProposalStatus, ProposalType, VoteTally, VotingThreshold, GovernanceParameter};
        let make_param_proposal = |id: u32, parameter: GovernanceParameter, new_value: u32| Proposal {
            id, proposer: 1u64,
            title: b"P".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 1u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 100, nays: 0, abstentions: 0, total_weight: 100, participation: 100 },
            status: ProposalStatus::Approved,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: Some(ProposalAction::ParameterChange { parameter, new_value }),
            executed_at: None,
        };
        // LaunchPeriod: valid range 1800..=86400
        crate::Proposals::<Test>::insert(0u32, make_param_proposal(0, GovernanceParameter::LaunchPeriod, 3600u32));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 0u32));
        // MinimumDeposit: valid range 100..=1000000
        crate::Proposals::<Test>::insert(1u32, make_param_proposal(1, GovernanceParameter::MinimumDeposit, 500u32));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 1u32));
        // SupermajorityThreshold: valid range 51..=100
        crate::Proposals::<Test>::insert(2u32, make_param_proposal(2, GovernanceParameter::SupermajorityThreshold, 66u32));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 2u32));
        // CouncilSize: valid range 5..=50
        crate::Proposals::<Test>::insert(3u32, make_param_proposal(3, GovernanceParameter::CouncilSize, 10u32));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 3u32));
        // EmergencyTimeout: valid range 3600..=604800
        crate::Proposals::<Test>::insert(4u32, make_param_proposal(4, GovernanceParameter::EmergencyTimeout, 7200u32));
        assert_ok!(BelizeGovernance::execute_proposal(RuntimeOrigin::signed(1), 4u32));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// cast_vote – invalid choice index
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cast_vote_invalid_choice_index_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_standard_proposal(1);
        run_to_block(28_801); // past LaunchPeriod, voting active
        // Index 3 is invalid (valid: 0=Aye, 1=Nay, 2=Abstain)
        assert_noop!(
            BelizeGovernance::cast_vote(RuntimeOrigin::signed(2), id, 3u8, 0u8),
            Error::<Test>::InvalidThreshold
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// District elections – additional districts (covers seat_allocation arms)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn start_district_election_all_districts_work() {
    new_test_ext().execute_with(|| {
        // District 1 = Cayo (2 seats)
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 1u8, 2u32, 50u32, 100u32));
        // District 2 = Corozal (1 seat)
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 2u8, 1u32, 50u32, 100u32));
        // District 3 = OrangeWalk (2 seats)
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 3u8, 2u32, 50u32, 100u32));
        // District 4 = StannCreek (2 seats)
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 4u8, 2u32, 50u32, 100u32));
        // District 5 = Toledo (2 seats)
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 5u8, 2u32, 50u32, 100u32));
    });
}

#[test]
fn start_district_election_too_many_seats_fails() {
    new_test_ext().execute_with(|| {
        // Corozal only has 1 seat; request 5 → InvalidDistrict
        assert_noop!(
            BelizeGovernance::start_district_election(RuntimeOrigin::root(), 2u8, 5u32, 50u32, 100u32),
            Error::<Test>::InvalidDistrict
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Enum method coverage: Department, BoardRole, BelizeDistrict
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn department_enum_methods_work() {
    new_test_ext().execute_with(|| {
        use crate::Department;
        assert_eq!(Department::Finance.name(), "Finance");
        assert_eq!(Department::Education.name(), "Education");
        assert_eq!(Department::Health.name(), "Health");
        assert_eq!(Department::Works.name(), "Public Works");
        assert_eq!(Department::Justice.name(), "Justice");
        assert_eq!(Department::Tourism.name(), "Tourism");
        assert_eq!(Department::Agriculture.name(), "Agriculture");
        assert_eq!(Department::Defense.name(), "Defense");
        assert_eq!(Department::Finance.prefix(), "gov.finance");
        assert_eq!(Department::Education.prefix(), "gov.education");
        assert_eq!(Department::Health.prefix(), "gov.health");
        assert_eq!(Department::Works.prefix(), "gov.works");
        assert_eq!(Department::Justice.prefix(), "gov.justice");
        assert_eq!(Department::Tourism.prefix(), "gov.tourism");
        assert_eq!(Department::Agriculture.prefix(), "gov.agriculture");
        assert_eq!(Department::Defense.prefix(), "gov.defense");
    });
}

#[test]
fn board_role_enum_methods_work() {
    new_test_ext().execute_with(|| {
        use crate::BoardRole;
        assert_eq!(BoardRole::Founder.name(), "Founder");
        assert_eq!(BoardRole::TechnicalSteward.name(), "Technical Steward");
        assert_eq!(BoardRole::FSCRepresentative.name(), "FSC Representative");
        assert_eq!(BoardRole::BTBDelegate.name(), "BTB Delegate");
        assert_eq!(BoardRole::CitizenDelegate.name(), "Citizen Delegate");
        assert_eq!(BoardRole::SecurityAuditor.name(), "Security Auditor");
        assert_eq!(BoardRole::CultureEthicsAdvisor.name(), "Culture & Ethics Advisor");
        // All roles now rotate (term limits apply to all)
        assert!(BoardRole::Founder.is_rotating());
        assert!(BoardRole::TechnicalSteward.is_rotating());
        assert!(BoardRole::FSCRepresentative.is_rotating());
        assert!(BoardRole::BTBDelegate.is_rotating());
        assert!(BoardRole::CitizenDelegate.is_rotating());
        assert!(BoardRole::SecurityAuditor.is_rotating());
        assert!(BoardRole::CultureEthicsAdvisor.is_rotating());
        assert_eq!(BoardRole::Founder.max_count(), 1);
        assert_eq!(BoardRole::TechnicalSteward.max_count(), 2);
        assert_eq!(BoardRole::FSCRepresentative.max_count(), 1);
        assert_eq!(BoardRole::BTBDelegate.max_count(), 1);
        assert_eq!(BoardRole::CitizenDelegate.max_count(), 3);
        assert_eq!(BoardRole::SecurityAuditor.max_count(), 2);
        assert_eq!(BoardRole::CultureEthicsAdvisor.max_count(), 1);
    });
}

#[test]
fn belize_district_enum_methods_work() {
    new_test_ext().execute_with(|| {
        use crate::BelizeDistrict;
        assert_eq!(BelizeDistrict::Belize.seat_allocation(), 3);
        assert_eq!(BelizeDistrict::Cayo.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::Corozal.seat_allocation(), 1);
        assert_eq!(BelizeDistrict::OrangeWalk.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::StannCreek.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::Toledo.seat_allocation(), 2);
        assert_eq!(BelizeDistrict::Belize.to_index(), 0);
        assert_eq!(BelizeDistrict::Cayo.to_index(), 1);
        assert_eq!(BelizeDistrict::Corozal.to_index(), 2);
        assert_eq!(BelizeDistrict::OrangeWalk.to_index(), 3);
        assert_eq!(BelizeDistrict::StannCreek.to_index(), 4);
        assert_eq!(BelizeDistrict::Toledo.to_index(), 5);
        assert_eq!(BelizeDistrict::from_index(0), Some(BelizeDistrict::Belize));
        assert_eq!(BelizeDistrict::from_index(1), Some(BelizeDistrict::Cayo));
        assert_eq!(BelizeDistrict::from_index(2), Some(BelizeDistrict::Corozal));
        assert_eq!(BelizeDistrict::from_index(3), Some(BelizeDistrict::OrangeWalk));
        assert_eq!(BelizeDistrict::from_index(4), Some(BelizeDistrict::StannCreek));
        assert_eq!(BelizeDistrict::from_index(5), Some(BelizeDistrict::Toledo));
        assert_eq!(BelizeDistrict::from_index(6), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Pallet helper function coverage (pub fn on impl Pallet<T>)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn helper_index_functions_work() {
    new_test_ext().execute_with(|| {
        use crate::{Department, BelizeDistrict, GovernanceParameter, EmergencyActionType, ElectionStatus, ProposalAction};
        // department_index all variants
        assert_eq!(BelizeGovernance::department_index(Department::Finance), 0);
        assert_eq!(BelizeGovernance::department_index(Department::Education), 1);
        assert_eq!(BelizeGovernance::department_index(Department::Health), 2);
        assert_eq!(BelizeGovernance::department_index(Department::Works), 3);
        assert_eq!(BelizeGovernance::department_index(Department::Justice), 4);
        assert_eq!(BelizeGovernance::department_index(Department::Tourism), 5);
        assert_eq!(BelizeGovernance::department_index(Department::Agriculture), 6);
        assert_eq!(BelizeGovernance::department_index(Department::Defense), 7);
        // district_index all variants
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Belize), 0);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Cayo), 1);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Corozal), 2);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::OrangeWalk), 3);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::StannCreek), 4);
        assert_eq!(BelizeGovernance::district_index(BelizeDistrict::Toledo), 5);
        // governance_parameter_index all variants
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::VotingPeriod), 0);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::LaunchPeriod), 1);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::MinimumDeposit), 2);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::SupermajorityThreshold), 3);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::CouncilSize), 4);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::EmergencyTimeout), 5);
        // emergency_action_type_index all variants
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::ActivateEmergency), 0);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::DeactivateEmergency), 1);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::FreezeAccount), 2);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::UnfreezeAccount), 3);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::HaltGovernance), 4);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::ResumeGovernance), 5);
        // election_status_index all variants
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Registration), 0);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Voting), 1);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Finalized), 2);
        assert_eq!(BelizeGovernance::election_status_index(ElectionStatus::Cancelled), 3);
        // get_action_type_index all variants
        assert_eq!(BelizeGovernance::get_action_type_index(&ProposalAction::TreasurySpend { recipient: 1u64, amount: 0u128 }), 0);
        let ch: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<32>> = [0u8; 32].to_vec().try_into().unwrap();
        assert_eq!(BelizeGovernance::get_action_type_index(&ProposalAction::RuntimeUpgrade { code_hash: ch }), 1);
        assert_eq!(BelizeGovernance::get_action_type_index(&ProposalAction::ParameterChange { parameter: GovernanceParameter::VotingPeriod, new_value: 0 }), 2);
        let cd: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<1024>> = vec![].try_into().unwrap();
        assert_eq!(BelizeGovernance::get_action_type_index(&ProposalAction::DepartmentAction { department: Department::Finance, call_data: cd }), 3);
        assert_eq!(BelizeGovernance::get_action_type_index(&ProposalAction::EmergencyAction { action_type: EmergencyActionType::ActivateEmergency }), 4);
    });
}

#[test]
fn helper_getter_functions_work() {
    new_test_ext().execute_with(|| {
        use crate::{GovernanceParameter, BelizeDistrict, Department, ProposalPriority};
        let aid = BelizeGovernance::account_id();
        assert_ne!(aid, 0u64);
        let vp = BelizeGovernance::get_parameter(GovernanceParameter::VotingPeriod);
        assert!(vp > 0);
        assert_eq!(BelizeGovernance::department_treasury_balance(Department::Finance), 0u128);
        assert_eq!(BelizeGovernance::council_size(), 3u32);
        assert_eq!(BelizeGovernance::board_composition(crate::BoardRole::Founder), 0u32);
        assert_eq!(BelizeGovernance::delegate_nominees(42u64), 0u32);
        assert!(BelizeGovernance::current_election().is_none());
        assert!(!BelizeGovernance::has_voted_in_election(1u64, &42u64));
        assert!(BelizeGovernance::get_district_election(BelizeDistrict::Belize).is_none());
        assert!(BelizeGovernance::get_district_representatives(BelizeDistrict::Belize).is_empty());
        assert!(!BelizeGovernance::has_voted_in_district_election(1u64, &42u64));
        assert!(BelizeGovernance::get_priority_queue(ProposalPriority::Low).is_empty());
        assert!(BelizeGovernance::get_priority_queue(ProposalPriority::Normal).is_empty());
        assert!(BelizeGovernance::get_priority_queue(ProposalPriority::High).is_empty());
        assert!(BelizeGovernance::get_priority_queue(ProposalPriority::Critical).is_empty());
        assert_eq!(BelizeGovernance::get_rewards_claimed(&42u64), 0u128);
        assert_eq!(BelizeGovernance::get_total_rewards_distributed(), 0u128);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Low), 1u8);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Normal), 2u8);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::High), 3u8);
        assert_eq!(BelizeGovernance::proposal_priority_index(ProposalPriority::Critical), 4u8);
    });
}

#[test]
fn helper_delegation_functions_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::count_delegated_votes(&1u64), 0u32);
        assert!(!BelizeGovernance::is_delegation_active(&1u64));
        assert!(BelizeGovernance::get_delegation_info(&1u64).is_none());
        // After delegate_vote: account 4 delegates to account 1
        assert_ok!(BelizeGovernance::delegate_vote(RuntimeOrigin::signed(4), 1u64, Some(1_000_000u64)));
        assert_eq!(BelizeGovernance::count_delegated_votes(&1u64), 1u32);
        assert!(BelizeGovernance::is_delegation_active(&4u64));
        assert!(BelizeGovernance::get_delegation_info(&4u64).is_some());
        // calculate_voting_power: account 1 has 1 base + 1 delegated from 4
        assert_eq!(BelizeGovernance::calculate_voting_power(&1u64), 2u32);
    });
}

#[test]
fn helper_is_delegation_active_expired_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::delegate_vote(RuntimeOrigin::signed(4), 1u64, Some(5u64)));
        run_to_block(10); // Past expiry of block 5
        assert!(!BelizeGovernance::is_delegation_active(&4u64));
    });
}

#[test]
fn helper_proposal_amendment_functions_work() {
    new_test_ext().execute_with(|| {
        use crate::{Proposal, ProposalStatus, ProposalType, VoteTally, VotingThreshold};
        assert!(BelizeGovernance::get_proposal_amendment(0u32).is_none());
        assert!(!BelizeGovernance::is_amendment_allowed(99u32));
        let proposal = Proposal {
            id: 0u32, proposer: 1u64,
            title: b"t".to_vec().try_into().unwrap(),
            description: b"d".to_vec().try_into().unwrap(),
            proposal_type: ProposalType::Economic,
            threshold: VotingThreshold::SimpleMajority,
            deposit: 0u128, voting_start: 100u64, voting_end: 1_000_000u64,
            vote_tally: VoteTally { ayes: 0, nays: 0, abstentions: 0, total_weight: 0, participation: 0 },
            status: ProposalStatus::Pending,
            is_emergency: false, department: None, district: None,
            requires_cross_approval: false, cross_approved_by: Default::default(),
            action: None, executed_at: None,
        };
        crate::Proposals::<Test>::insert(0u32, proposal);
        // At block 1 (before voting_start=100), amendment is allowed
        assert!(BelizeGovernance::is_amendment_allowed(0u32));
        // Move past voting_start
        run_to_block(101);
        assert!(!BelizeGovernance::is_amendment_allowed(0u32));
    });
}

#[test]
fn helper_allocate_to_department_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::department_treasury_balance(crate::Department::Education), 0u128);
        assert_ok!(BelizeGovernance::allocate_to_department(crate::Department::Education, 5_000u128, 0u32));
        assert_eq!(BelizeGovernance::department_treasury_balance(crate::Department::Education), 5_000u128);
    });
}

#[test]
fn helper_get_election_candidate_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(BelizeGovernance::start_district_election(RuntimeOrigin::root(), 0u8, 2u32, 100u32, 200u32));
        assert_ok!(BelizeGovernance::register_candidate(RuntimeOrigin::signed(4), 0u8, b"Platform for Belize City".to_vec()));
        // Election ID = block number when election was started (block 1)
        let candidate = BelizeGovernance::get_election_candidate(1u64, 4u64);
        assert!(candidate.is_some());
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// propose_treasury_spend – additional coverage
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn propose_treasury_spend_large_amounts_work() {
    new_test_ext().execute_with(|| {
        // Medium amount (10K–100K DALLA → threshold=3)
        let medium: u128 = 50_000_000_000_000u128;
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1), 5u64, medium, b"medium spend".to_vec(), None,
        ));
        // Large amount (>100K DALLA → threshold=4)
        let large: u128 = 200_000_000_000_000u128;
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(1), 5u64, large, b"large spend".to_vec(), None,
        ));
    });
}

#[test]
fn propose_treasury_spend_invalid_district_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BelizeGovernance::propose_treasury_spend(
                RuntimeOrigin::signed(1), 5u64, 1_000u128, b"d".to_vec(), Some(99u8),
            ),
            Error::<Test>::InvalidDistrict
        );
    });
}

#[test]
fn propose_treasury_spend_description_too_long_fails() {
    new_test_ext().execute_with(|| {
        let long_desc = vec![b'x'; 513]; // > 512 bytes
        assert_noop!(
            BelizeGovernance::propose_treasury_spend(
                RuntimeOrigin::signed(1), 5u64, 1_000u128, long_desc, None,
            ),
            Error::<Test>::TreasuryDescriptionTooLong
        );
    });
}
