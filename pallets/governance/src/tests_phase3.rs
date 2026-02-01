// ============================================================================
// tests_phase3.rs - Comprehensive test suite for Phase 3: Departmental Governance
// Test Suite 10: Departmental Governance Tests (16 tests)

use crate::{
    mock::{new_test_ext, BelizeGovernance, RuntimeOrigin, RuntimeEvent, System, Test, last_event},
    Event, Department, CrossDepartmentApprovals, Error,
};
use frame_support::{assert_ok, assert_noop};
// ============================================================================

#[test]
fn set_department_manager_works() {
    new_test_ext().execute_with(|| {
        // Root sets Finance department manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0, // Finance
            10, // Account 10 as manager
        ));

        // Verify manager was set
        assert_eq!(BelizeGovernance::department_managers(Department::Finance), Some(10));

        // Check event
        assert_eq!(
            last_event(),
            RuntimeEvent::BelizeGovernance(Event::DepartmentManagerSet {
                department_index: 0,
                manager: 10,
            })
        );
    });
}

#[test]
fn set_department_manager_requires_root() {
    new_test_ext().execute_with(|| {
        // Non-root account cannot set department manager
        assert_noop!(
            BelizeGovernance::set_department_manager(
                RuntimeOrigin::signed(1),
                0, // Finance
                10,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_department_manager_invalid_department_fails() {
    new_test_ext().execute_with(|| {
        // Invalid department index (only 0-7 valid)
        assert_noop!(
            BelizeGovernance::set_department_manager(
                RuntimeOrigin::root(),
                99, // Invalid
                10,
            ),
            Error::<Test>::InvalidDepartment
        );
    });
}

#[test]
fn submit_department_proposal_works() {
    new_test_ext().execute_with(|| {
        // Set Alice as Education department manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            1, // Education
            1, // Alice
        ));

        // Alice submits education proposal
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1, // Education
            b"Student Stipend Program".to_vec(),
            b"Monthly 50 DALLA stipends for verified students".to_vec(),
            1, // Economic
            0, // Simple majority
            false, // No cross-department approval needed
        ));

        // Verify proposal was created
        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.proposer, 1);
        assert_eq!(proposal.department, Some(Department::Education));
        assert!(!proposal.requires_cross_approval);

        // Verify department proposal count incremented
        assert_eq!(BelizeGovernance::department_proposal_count(Department::Education), 1);

        // Check event
        assert_eq!(
            last_event(),
            RuntimeEvent::BelizeGovernance(Event::DepartmentProposalSubmitted {
                proposal_id: 0,
                department_index: 1,
                proposer: 1,
            })
        );
    });
}

#[test]
fn submit_department_proposal_requires_manager() {
    new_test_ext().execute_with(|| {
        // Set Alice as Finance manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0, // Finance
            1, // Alice
        ));

        // Bob tries to submit Finance proposal (he's not the manager)
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(2),
                0, // Finance
                b"Budget Proposal".to_vec(),
                b"Test".to_vec(),
                1,
                0,
                false,
            ),
            Error::<Test>::NotDepartmentManager
        );
    });
}

#[test]
fn submit_department_proposal_uninitialized_department_fails() {
    new_test_ext().execute_with(|| {
        // Try to submit proposal for department without manager
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(1),
                2, // Health (no manager set)
                b"Health Proposal".to_vec(),
                b"Test".to_vec(),
                1,
                0,
                false,
            ),
            Error::<Test>::DepartmentNotFound
        );
    });
}

#[test]
fn submit_department_proposal_with_cross_approval() {
    new_test_ext().execute_with(|| {
        // Set Alice as Education manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            1, // Education
            1, // Alice
        ));

        // Alice submits proposal requiring Finance approval
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1, // Education
            b"Large Equipment Purchase".to_vec(),
            b"Requires Finance approval for 100k DALLA".to_vec(),
            1, // Economic
            1, // Supermajority
            true, // Requires cross-department approval
        ));

        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert!(proposal.requires_cross_approval);
        assert_eq!(proposal.cross_approved_by.len(), 0); // No approvals yet
    });
}

#[test]
fn approve_cross_department_works() {
    new_test_ext().execute_with(|| {
        // Set Alice as Education manager, Bob as Finance manager
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            1, // Education
            1, // Alice
        ));
        assert_ok!(BelizeGovernance::set_department_manager(
            RuntimeOrigin::root(),
            0, // Finance
            2, // Bob
        ));

        // Alice submits Education proposal requiring Finance approval
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1, // Education
            b"Cross-dept Proposal".to_vec(),
            b"Needs Finance approval".to_vec(),
            1,
            0,
            true, // Requires cross-approval
        ));

        // Bob (Finance manager) approves
        assert_ok!(BelizeGovernance::approve_cross_department(
            RuntimeOrigin::signed(2),
            0, // Proposal ID
            0, // Finance department
        ));

        // Verify approval recorded
        assert!(CrossDepartmentApprovals::<Test>::get(0, Department::Finance));

        // Verify proposal updated
        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.cross_approved_by.len(), 1);
        assert_eq!(proposal.cross_approved_by[0], Department::Finance);

        // Check event
        let all_events = System::events();
        let gov_events: Vec<_> = all_events
            .iter()
            .filter_map(|r| {
                if let RuntimeEvent::BelizeGovernance(e) = &r.event {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect();
        
        assert!(matches!(
            gov_events.last(),
            Some(Event::CrossDepartmentApproved { proposal_id: 0, department_index: 0, approver: 2 })
        ));
    });
}

#[test]
fn approve_cross_department_requires_manager() {
    new_test_ext().execute_with(|| {
        // Set up departments and proposal
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 1));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 2));
        
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1,
            b"Test".to_vec(),
            b"Test".to_vec(),
            1,
            0,
            true,
        ));

        // Charlie (not Finance manager) tries to approve
        assert_noop!(
            BelizeGovernance::approve_cross_department(
                RuntimeOrigin::signed(3),
                0,
                0, // Finance
            ),
            Error::<Test>::NotDepartmentManager
        );
    });
}

#[test]
fn approve_cross_department_no_approval_required_fails() {
    new_test_ext().execute_with(|| {
        // Set up departments
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 1));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 2));
        
        // Submit proposal WITHOUT cross-approval requirement
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1,
            b"Test".to_vec(),
            b"Test".to_vec(),
            1,
            0,
            false, // No cross-approval needed
        ));

        // Finance tries to approve anyway
        assert_noop!(
            BelizeGovernance::approve_cross_department(
                RuntimeOrigin::signed(2),
                0,
                0,
            ),
            Error::<Test>::CrossApprovalNotRequired
        );
    });
}

#[test]
fn approve_cross_department_twice_fails() {
    new_test_ext().execute_with(|| {
        // Set up departments
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 1));
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 2));
        
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1,
            b"Test".to_vec(),
            b"Test".to_vec(),
            1,
            0,
            true,
        ));

        // First approval succeeds
        assert_ok!(BelizeGovernance::approve_cross_department(
            RuntimeOrigin::signed(2),
            0,
            0,
        ));

        // Second approval from same department fails
        assert_noop!(
            BelizeGovernance::approve_cross_department(
                RuntimeOrigin::signed(2),
                0,
                0,
            ),
            Error::<Test>::DepartmentAlreadyApproved
        );
    });
}

#[test]
fn multiple_departments_can_approve() {
    new_test_ext().execute_with(|| {
        // Set up three departments
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 1, 1)); // Education - Alice
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 2)); // Finance - Bob
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 2, 3)); // Health - Charlie

        // Education submits large cross-department proposal
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            1,
            b"Major Initiative".to_vec(),
            b"Requires multiple department approvals".to_vec(),
            1,
            1, // Supermajority
            true,
        ));

        // Finance approves
        assert_ok!(BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(2), 0, 0));
        
        // Health approves
        assert_ok!(BelizeGovernance::approve_cross_department(RuntimeOrigin::signed(3), 0, 2));

        // Verify both approvals recorded
        let proposal = BelizeGovernance::proposals(0).unwrap();
        assert_eq!(proposal.cross_approved_by.len(), 2);
        assert!(proposal.cross_approved_by.contains(&Department::Finance));
        assert!(proposal.cross_approved_by.contains(&Department::Health));
    });
}

#[test]
fn department_proposal_count_increments() {
    new_test_ext().execute_with(|| {
        // Set Alice as Finance manager
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 1));

        // Submit multiple Finance proposals
        for i in 0..3 {
            assert_ok!(BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(1),
                0,
                format!("Finance Proposal {}", i).as_bytes().to_vec(),
                b"Test".to_vec(),
                1,
                0,
                false,
            ));
        }

        // Verify count
        assert_eq!(BelizeGovernance::department_proposal_count(Department::Finance), 3);
        
        // Other departments still at 0
        assert_eq!(BelizeGovernance::department_proposal_count(Department::Education), 0);
    });
}

#[test]
fn department_enum_methods_work() {
    new_test_ext().execute_with(|| {
        // Test department name
        assert_eq!(Department::Finance.name(), "Finance");
        assert_eq!(Department::Education.name(), "Education");
        assert_eq!(Department::Works.name(), "Public Works");
        
        // Test department prefix
        assert_eq!(Department::Finance.prefix(), "gov.finance");
        assert_eq!(Department::Education.prefix(), "gov.education");
        assert_eq!(Department::Tourism.prefix(), "gov.tourism");
    });
}

#[test]
fn general_and_department_proposals_coexist() {
    new_test_ext().execute_with(|| {
        // Set Alice as Finance manager
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 1));

        // Submit general proposal (Bob)
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(2),
            b"General Proposal".to_vec(),
            b"Not department-specific".to_vec(),
            1,
            0,
            false,
        None,
        ));

        // Submit department proposal (Alice)
        assert_ok!(BelizeGovernance::submit_department_proposal(
            RuntimeOrigin::signed(1),
            0,
            b"Finance Proposal".to_vec(),
            b"Department-specific".to_vec(),
            1,
            0,
            false,
        ));

        // Both exist
        let general = BelizeGovernance::proposals(0).unwrap();
        let departmental = BelizeGovernance::proposals(1).unwrap();

        assert_eq!(general.department, None);
        assert_eq!(departmental.department, Some(Department::Finance));
        
        // Both have unique IDs
        assert_eq!(general.id, 0);
        assert_eq!(departmental.id, 1);
    });
}

#[test]
fn restricted_account_cannot_submit_department_proposal() {
    new_test_ext().execute_with(|| {
        // Set account 999 (restricted) as manager
        assert_ok!(BelizeGovernance::set_department_manager(RuntimeOrigin::root(), 0, 999));

        // Account 999 tries to submit proposal
        assert_noop!(
            BelizeGovernance::submit_department_proposal(
                RuntimeOrigin::signed(999),
                0,
                b"Test".to_vec(),
                b"Test".to_vec(),
                1,
                0,
                false,
            ),
            Error::<Test>::InsufficientCompliance
        );
    });
}
