// tests_phase4.rs - Comprehensive test suite for Phase 4: Foundation Board System
// Test Suite 11: Board Member Management and Delegate Elections (16 tests)

use crate::{
    mock::{new_test_ext, BelizeGovernance, RuntimeOrigin, RuntimeEvent, System, Test, last_event},
    Event, BoardRole, TermExpiryQueue, DelegateNominees, DelegateVoters, CurrentElection, Error,
};
use frame_support::{assert_ok, assert_noop};

// ===== Test Suite 11: Board Member Management (12 tests) =====

#[test]
fn add_board_member_works() {
    new_test_ext().execute_with(|| {
        // Root adds a founder board member with 2-year term
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,  // Account ID
            0,   // Founder role index
            2,   // 2-year term
        ));

        // Verify member added to council
        assert!(BelizeGovernance::council_members(10).is_some());
        let member = BelizeGovernance::council_members(10).unwrap();
        assert_eq!(member.role, BoardRole::Founder);
        assert!(member.term_end > 0u64);
        assert!(!member.is_rotating);

        // Verify board composition updated
        assert_eq!(BelizeGovernance::board_composition(BoardRole::Founder), 1);

        // Verify event
        assert_eq!(
            last_event(),
            RuntimeEvent::BelizeGovernance(Event::BoardMemberAdded {
                member: 10,
                role_index: 0,
                term_end: member.term_end,
            })
        );
    });
}

#[test]
fn add_board_member_requires_council_origin() {
    new_test_ext().execute_with(|| {
        // Non-root account cannot add board members
        assert_noop!(
            BelizeGovernance::add_board_member(
                RuntimeOrigin::signed(5),
                10,
                0,  // Founder
                2,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_board_member_role_limit_enforced() {
    new_test_ext().execute_with(|| {
        // Add first founder (max is 1)
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,
            0,  // Founder
            2,
        ));

        // Try to add second founder (should fail)
        assert_noop!(
            BelizeGovernance::add_board_member(
                RuntimeOrigin::root(),
                11,
                0,  // Founder
                2,
            ),
            Error::<Test>::BoardRoleLimitExceeded
        );
    });
}

#[test]
fn add_board_member_invalid_role_fails() {
    new_test_ext().execute_with(|| {
        // Invalid role index
        assert_noop!(
            BelizeGovernance::add_board_member(
                RuntimeOrigin::root(),
                10,
                99,  // Invalid role
                2,
            ),
            Error::<Test>::InvalidBoardRole
        );
    });
}

#[test]
fn add_board_member_invalid_term_duration_fails() {
    new_test_ext().execute_with(|| {
        // Term duration too short (0 years)
        assert_noop!(
            BelizeGovernance::add_board_member(
                RuntimeOrigin::root(),
                10,
                0,  // Founder
                0,  // Invalid: 0 years
            ),
            Error::<Test>::InvalidTermDuration
        );

        // Term duration too long (3 years)
        assert_noop!(
            BelizeGovernance::add_board_member(
                RuntimeOrigin::root(),
                10,
                0,  // Founder
                3,  // Invalid: >2 years
            ),
            Error::<Test>::InvalidTermDuration
        );
    });
}

#[test]
fn add_board_member_citizen_delegate_is_rotating() {
    new_test_ext().execute_with(|| {
        // Add citizen delegate (rotating position)
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,
            4,  // CitizenDelegate role index
            1,  // 1-year term
        ));

        let member = BelizeGovernance::council_members(10).unwrap();
        assert_eq!(member.role, BoardRole::CitizenDelegate);
        assert!(member.is_rotating);  // Citizen delegates rotate
    });
}

#[test]
fn add_multiple_board_roles_works() {
    new_test_ext().execute_with(|| {
        // Add founder
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,
            0,  // Founder
            2,
        ));

        // Add technical steward (max is 2)
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            11,
            1,  // TechnicalSteward
            2,
        ));

        // Add another technical steward
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            12,
            1,  // TechnicalSteward
            2,
        ));

        // Verify board composition
        assert_eq!(BelizeGovernance::board_composition(BoardRole::Founder), 1);
        assert_eq!(BelizeGovernance::board_composition(BoardRole::TechnicalSteward), 2);
    });
}

#[test]
fn remove_board_member_works() {
    new_test_ext().execute_with(|| {
        // Add a board member first
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,
            0,  // Founder
            2,
        ));

        // Remove the board member
        assert_ok!(BelizeGovernance::remove_board_member(
            RuntimeOrigin::root(),
            10,
            b"Voluntary resignation".to_vec(),
        ));

        // Verify member removed
        assert!(BelizeGovernance::council_members(10).is_none());

        // Verify board composition updated
        assert_eq!(BelizeGovernance::board_composition(BoardRole::Founder), 0);

        // Verify event
        let all_events = System::events();
        let removal_event = all_events.into_iter()
            .filter_map(|r| {
                if let RuntimeEvent::BelizeGovernance(e) = &r.event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .find(|e| matches!(e, Event::BoardMemberRemoved { .. }));

        assert!(removal_event.is_some());
    });
}

#[test]
fn remove_board_member_not_found_fails() {
    new_test_ext().execute_with(|| {
        // Try to remove non-existent member
        assert_noop!(
            BelizeGovernance::remove_board_member(
                RuntimeOrigin::root(),
                99,  // Doesn't exist
                b"test".to_vec(),
            ),
            Error::<Test>::MemberNotFound
        );
    });
}

#[test]
fn nominate_for_delegate_works() {
    new_test_ext().execute_with(|| {
        // Start an election
        CurrentElection::<Test>::put(100u64);

        // Nominate a citizen for delegate
        assert_ok!(BelizeGovernance::nominate_for_delegate(
            RuntimeOrigin::signed(1),
            20,  // Nominee account
        ));

        // Verify nominee added
        assert_eq!(BelizeGovernance::delegate_nominees(20), 0);  // 0 votes initially

        // Verify event
        assert_eq!(
            last_event(),
            RuntimeEvent::BelizeGovernance(Event::DelegateNominated {
                nominee: 20,
                nominator: 1,
            })
        );
    });
}

#[test]
fn nominate_for_delegate_no_election_fails() {
    new_test_ext().execute_with(|| {
        // Try to nominate without active election
        assert_noop!(
            BelizeGovernance::nominate_for_delegate(
                RuntimeOrigin::signed(1),
                20,
            ),
            Error::<Test>::NoActiveElection
        );
    });
}

#[test]
fn vote_for_delegate_works() {
    new_test_ext().execute_with(|| {
        // Start an election
        let election_id = 100u64;
        CurrentElection::<Test>::put(election_id);

        // Add a nominee
        DelegateNominees::<Test>::insert(20, 0);

        // Vote for the nominee
        assert_ok!(BelizeGovernance::vote_for_delegate(
            RuntimeOrigin::signed(1),
            20,  // Nominee
        ));

        // Verify vote recorded
        assert_eq!(DelegateVoters::<Test>::get(election_id, 1), Some(20));

        // Verify nominee vote count increased
        assert_eq!(BelizeGovernance::delegate_nominees(20), 1);

        // Verify event
        assert_eq!(
            last_event(),
            RuntimeEvent::BelizeGovernance(Event::DelegateVoteCast {
                voter: 1,
                nominee: 20,
                weight: 1,
            })
        );
    });
}

#[test]
fn vote_for_delegate_twice_fails() {
    new_test_ext().execute_with(|| {
        // Start an election
        let election_id = 100u64;
        CurrentElection::<Test>::put(election_id);

        // Add nominees
        DelegateNominees::<Test>::insert(20, 0);
        DelegateNominees::<Test>::insert(21, 0);

        // First vote succeeds
        assert_ok!(BelizeGovernance::vote_for_delegate(
            RuntimeOrigin::signed(1),
            20,
        ));

        // Second vote by same account fails
        assert_noop!(
            BelizeGovernance::vote_for_delegate(
                RuntimeOrigin::signed(1),
                21,  // Different nominee
            ),
            Error::<Test>::AlreadyVotedInElection
        );
    });
}

#[test]
fn vote_for_delegate_nominee_not_found_fails() {
    new_test_ext().execute_with(|| {
        // Start an election
        CurrentElection::<Test>::put(100u64);

        // Try to vote for non-existent nominee
        assert_noop!(
            BelizeGovernance::vote_for_delegate(
                RuntimeOrigin::signed(1),
                99,  // Not nominated
            ),
            Error::<Test>::NomineeNotFound
        );
    });
}

#[test]
fn board_role_helper_methods_work() {
    new_test_ext().execute_with(|| {
        // Test role names
        assert_eq!(BoardRole::Founder.name(), "Founder");
        assert_eq!(BoardRole::TechnicalSteward.name(), "Technical Steward");
        assert_eq!(BoardRole::CitizenDelegate.name(), "Citizen Delegate");

        // Test rotation check
        assert!(!BoardRole::Founder.is_rotating());
        assert!(!BoardRole::TechnicalSteward.is_rotating());
        assert!(BoardRole::CitizenDelegate.is_rotating());

        // Test max counts
        assert_eq!(BoardRole::Founder.max_count(), 1);
        assert_eq!(BoardRole::TechnicalSteward.max_count(), 2);
        assert_eq!(BoardRole::CitizenDelegate.max_count(), 3);
        assert_eq!(BoardRole::FSCRepresentative.max_count(), 1);
    });
}

#[test]
fn term_expiry_queue_works() {
    new_test_ext().execute_with(|| {
        // Add a board member with 1-year term
        assert_ok!(BelizeGovernance::add_board_member(
            RuntimeOrigin::root(),
            10,
            0,  // Founder
            1,  // 1 year
        ));

        let member = BelizeGovernance::council_members(10).unwrap();
        let term_end = member.term_end;

        // Verify member added to expiry queue
        let queue = TermExpiryQueue::<Test>::get(term_end);
        assert!(queue.contains(&10));
    });
}
