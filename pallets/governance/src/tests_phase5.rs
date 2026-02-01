// Phase 5: Execution Layer Tests
// Tests for proposal execution, treasury spending, runtime upgrades, parameter changes

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, assert_noop};

// Test accounts (u64)
const ALICE: u64 = 1;
const BOB: u64 = 2;
#[allow(dead_code)]
const CHARLIE: u64 = 3;

// ===== EXECUTION LAYER BASICS =====

#[test]
fn execute_proposal_works() {
    new_test_ext().execute_with(|| {
        // Create a proposal
        let title = b"Test Execution".to_vec();
        let description = b"Testing proposal execution".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0, // Constitutional
            0, // SimpleMajority
            false,
        None,
        ));

        // Manually set proposal as approved
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::ParameterChange {
            parameter: GovernanceParameter::VotingPeriod,
            new_value: 100000u32,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify execution
        let proposal = Proposals::<Test>::get(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        assert!(proposal.executed_at.is_some());
        assert!(ExecutedProposals::<Test>::contains_key(0));
    });
}

#[test]
fn execute_proposal_not_approved_fails() {
    new_test_ext().execute_with(|| {
        // Create a proposal
        let title = b"Test".to_vec();
        let description = b"Test".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Try to execute without approval
        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::ProposalNotApproved
        );
    });
}

#[test]
fn execute_proposal_already_executed_fails() {
    new_test_ext().execute_with(|| {
        // Create and approve proposal
        let title = b"Test".to_vec();
        let description = b"Test".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Manually approve and add action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::ParameterChange {
            parameter: GovernanceParameter::VotingPeriod,
            new_value: 100000u32,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute once
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Try to execute again
        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::AlreadyExecuted
        );
    });
}

#[test]
fn execute_proposal_no_action_fails() {
    new_test_ext().execute_with(|| {
        // Create and approve proposal without action
        let title = b"Test".to_vec();
        let description = b"Test".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Manually approve but don't add action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        Proposals::<Test>::insert(0, proposal);

        // Try to execute
        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::NoActionDefined
        );
    });
}

// ===== TREASURY SPENDING =====

#[test]
fn treasury_spend_execution_works() {
    new_test_ext().execute_with(|| {
        // Fund the treasury account
        let treasury_account = BelizeGovernance::account_id();
        let _ = Balances::deposit_creating(&treasury_account, 100_000);

        // Create and approve treasury spend proposal
        let title = b"Treasury Spend".to_vec();
        let description = b"Test spending".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal as approved with treasury spend action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::TreasurySpend {
            recipient: BOB,
            amount: 10_000,
        });
        Proposals::<Test>::insert(0, proposal);

        let bob_balance_before = Balances::free_balance(BOB);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify BOB received funds
        let bob_balance_after = Balances::free_balance(BOB);
        assert_eq!(bob_balance_after, bob_balance_before + 10_000);
    });
}

#[test]
fn treasury_spend_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        // Don't fund treasury
        
        // Create and approve treasury spend proposal
        let title = b"Treasury Spend".to_vec();
        let description = b"Test".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with treasury spend exceeding balance
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::TreasurySpend {
            recipient: BOB,
            amount: 1_000_000,
        });
        Proposals::<Test>::insert(0, proposal);

        // Try to execute
        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::InsufficientTreasuryBalance
        );
    });
}

// ===== RUNTIME UPGRADES =====

#[test]
fn runtime_upgrade_execution_works() {
    new_test_ext().execute_with(|| {
        // Create runtime upgrade proposal
        let title = b"Runtime Upgrade".to_vec();
        let description = b"Upgrade to v2".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with runtime upgrade action
        let code_hash_vec: Vec<u8> = (0..32).collect();
        let bounded_hash: BoundedVec<u8, ConstU32<32>> = code_hash_vec.try_into().unwrap();
        
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::RuntimeUpgrade {
            code_hash: bounded_hash,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify pending runtime upgrade was stored
        assert!(PendingRuntimeUpgrade::<Test>::exists());
    });
}

#[test]
fn runtime_upgrade_invalid_hash_size_fails() {
    new_test_ext().execute_with(|| {
        // Create runtime upgrade proposal
        let title = b"Runtime Upgrade".to_vec();
        let description = b"Invalid upgrade".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Try to set proposal with invalid hash size (only 16 bytes)
        let code_hash_vec: Vec<u8> = (0..16).collect();
        let bounded_hash: BoundedVec<u8, ConstU32<32>> = code_hash_vec.try_into().unwrap();
        
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::RuntimeUpgrade {
            code_hash: bounded_hash,
        });
        Proposals::<Test>::insert(0, proposal);

        // Try to execute - should fail
        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::RuntimeCodeTooLarge
        );
    });
}

// ===== PARAMETER CHANGES =====

#[test]
fn parameter_change_execution_works() {
    new_test_ext().execute_with(|| {
        // Create parameter change proposal
        let title = b"Change Voting Period".to_vec();
        let description = b"Increase voting time".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with parameter change action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::ParameterChange {
            parameter: GovernanceParameter::VotingPeriod,
            new_value: 200000u32,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify parameter was changed
        let new_value = BelizeGovernance::get_parameter(GovernanceParameter::VotingPeriod);
        assert_eq!(new_value, 200000u32);
    });
}

#[test]
fn parameter_change_all_parameters_work() {
    new_test_ext().execute_with(|| {
        // Test all governance parameters can be changed
        let parameters = [(GovernanceParameter::VotingPeriod, 100000u32),
            (GovernanceParameter::LaunchPeriod, 50000u32),
            (GovernanceParameter::MinimumDeposit, 5000u32),
            (GovernanceParameter::SupermajorityThreshold, 75u32),
            (GovernanceParameter::CouncilSize, 20u32),
            (GovernanceParameter::EmergencyTimeout, 100000u32)];

        for (idx, (parameter, new_value)) in parameters.iter().enumerate() {
            // Create proposal
            let title = format!("Parameter Change {}", idx).into_bytes();
            let description = b"Test".to_vec();
            
            assert_ok!(BelizeGovernance::submit_proposal(
                RuntimeOrigin::signed(ALICE),
                title,
                description,
                0,
                0,
                false,
            None,
            ));

            // Set and execute
            let mut proposal = Proposals::<Test>::get(idx as u32).unwrap();
            proposal.status = ProposalStatus::Approved;
            proposal.action = Some(ProposalAction::ParameterChange {
                parameter: *parameter,
                new_value: *new_value,
            });
            Proposals::<Test>::insert(idx as u32, proposal);

            assert_ok!(BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                idx as u32,
            ));

            // Verify change
            assert_eq!(BelizeGovernance::get_parameter(*parameter), *new_value);
        }
    });
}

#[test]
fn parameter_change_invalid_value_fails() {
    new_test_ext().execute_with(|| {
        // Test invalid values are rejected
        
        // VotingPeriod too low (< 3600)
        let title = b"Invalid Period".to_vec();
        let description = b"Test".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::ParameterChange {
            parameter: GovernanceParameter::VotingPeriod,
            new_value: 1000u32, // Too low
        });
        Proposals::<Test>::insert(0, proposal);

        assert_noop!(
            BelizeGovernance::execute_proposal(
                RuntimeOrigin::signed(ALICE),
                0,
            ),
            Error::<Test>::InvalidParameterValue
        );
    });
}

// ===== DEPARTMENT ACTIONS =====

#[test]
fn department_action_execution_works() {
    new_test_ext().execute_with(|| {
        // Create department action proposal
        let title = b"Department Action".to_vec();
        let description = b"Execute department task".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with department action
        let call_data: Vec<u8> = vec![1, 2, 3, 4];
        let bounded_data: BoundedVec<u8, ConstU32<1024>> = call_data.try_into().unwrap();
        
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::DepartmentAction {
            department: Department::Finance,
            call_data: bounded_data,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify execution completed
        let proposal = Proposals::<Test>::get(0).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
    });
}

// ===== EMERGENCY ACTIONS =====

#[test]
fn emergency_action_activate_works() {
    new_test_ext().execute_with(|| {
        // Create emergency action proposal
        let title = b"Activate Emergency".to_vec();
        let description = b"Emergency mode".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with emergency action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::EmergencyAction {
            action_type: EmergencyActionType::ActivateEmergency,
        });
        Proposals::<Test>::insert(0, proposal);

        // Verify emergency mode is off
        // assert!(!EmergencyType::<Test>::get());

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify emergency mode is on
        // assert!(EmergencyType::<Test>::get());
    });
}

#[test]
fn emergency_action_deactivate_works() {
    new_test_ext().execute_with(|| {
        // Note: Emergency API was updated - see tests_phase8.rs for current tests
        
        // Create deactivate proposal
        let title = b"Deactivate Emergency".to_vec();
        let description = b"End emergency".to_vec();
        
        assert_ok!(BelizeGovernance::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            0,
            0,
            false,
        None,
        ));

        // Set proposal with deactivate action
        let mut proposal = Proposals::<Test>::get(0).unwrap();
        proposal.status = ProposalStatus::Approved;
        proposal.action = Some(ProposalAction::EmergencyAction {
            action_type: EmergencyActionType::DeactivateEmergency,
        });
        Proposals::<Test>::insert(0, proposal);

        // Execute proposal
        assert_ok!(BelizeGovernance::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            0,
        ));

        // Verify emergency mode is off
        // assert!(!EmergencyType::<Test>::get());
    });
}

// ===== DEPARTMENT TREASURY =====

#[test]
fn department_treasury_allocation_works() {
    new_test_ext().execute_with(|| {
        // Allocate funds to department
        assert_ok!(BelizeGovernance::allocate_to_department(
            Department::Finance,
            50_000,
            0,
        ));

        // Verify allocation
        let balance = BelizeGovernance::department_treasury_balance(Department::Finance);
        assert_eq!(balance, 50_000);
    });
}

#[test]
fn multiple_department_allocations_accumulate() {
    new_test_ext().execute_with(|| {
        // Multiple allocations to same department
        assert_ok!(BelizeGovernance::allocate_to_department(
            Department::Education,
            10_000,
            0,
        ));
        assert_ok!(BelizeGovernance::allocate_to_department(
            Department::Education,
            20_000,
            1,
        ));
        assert_ok!(BelizeGovernance::allocate_to_department(
            Department::Education,
            15_000,
            2,
        ));

        // Verify total
        let balance = BelizeGovernance::department_treasury_balance(Department::Education);
        assert_eq!(balance, 45_000);
    });
}

// ===== HELPER METHOD TESTS =====

#[test]
fn get_action_type_index_works() {
    new_test_ext().execute_with(|| {
        let action1 = ProposalAction::TreasurySpend::<u64, u128> {
            recipient: ALICE,
            amount: 1000,
        };
        assert_eq!(BelizeGovernance::get_action_type_index(&action1), 0);

        let action2 = ProposalAction::RuntimeUpgrade {
            code_hash: BoundedVec::default(),
        };
        assert_eq!(BelizeGovernance::get_action_type_index(&action2), 1);

        let action3 = ProposalAction::ParameterChange {
            parameter: GovernanceParameter::VotingPeriod,
            new_value: 100,
        };
        assert_eq!(BelizeGovernance::get_action_type_index(&action3), 2);
    });
}

#[test]
fn governance_parameter_indices_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::VotingPeriod), 0);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::LaunchPeriod), 1);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::MinimumDeposit), 2);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::SupermajorityThreshold), 3);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::CouncilSize), 4);
        assert_eq!(BelizeGovernance::governance_parameter_index(GovernanceParameter::EmergencyTimeout), 5);
    });
}

#[test]
fn department_indices_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::department_index(Department::Finance), 0);
        assert_eq!(BelizeGovernance::department_index(Department::Education), 1);
        assert_eq!(BelizeGovernance::department_index(Department::Health), 2);
        assert_eq!(BelizeGovernance::department_index(Department::Works), 3);
        assert_eq!(BelizeGovernance::department_index(Department::Justice), 4);
        assert_eq!(BelizeGovernance::department_index(Department::Tourism), 5);
        assert_eq!(BelizeGovernance::department_index(Department::Agriculture), 6);
        assert_eq!(BelizeGovernance::department_index(Department::Defense), 7);
    });
}

#[test]
fn emergency_action_type_indices_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::ActivateEmergency), 0);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::DeactivateEmergency), 1);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::FreezeAccount), 2);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::UnfreezeAccount), 3);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::HaltGovernance), 4);
        assert_eq!(BelizeGovernance::emergency_action_type_index(EmergencyActionType::ResumeGovernance), 5);
    });
}

// ===== GENESIS INITIALIZATION =====

#[test]
fn genesis_initializes_parameters() {
    new_test_ext().execute_with(|| {
        // Verify all governance parameters are initialized
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::VotingPeriod), VOTING_PERIOD);
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::LaunchPeriod), 28800u32);
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::MinimumDeposit), 1000u32);
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::SupermajorityThreshold), 66u32);
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::CouncilSize), 15u32);
        assert_eq!(BelizeGovernance::get_parameter(GovernanceParameter::EmergencyTimeout), 86400u32);
    });
}
