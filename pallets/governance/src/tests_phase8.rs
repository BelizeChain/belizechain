// Governance Enhancements - Test Suite
// Tests for referendum system, treasury management, and emergency governance

use crate::{
    mock::*, Error, BelizeDistrict, ReferendumStatus,
    Referendums, NextReferendumId, ReferendumVotes,
    TreasurySpendProposals, NextTreasuryProposalId, DistrictBudgets,
    JaguarMode, EmergencyStatus, EmergencyType,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

// Test account constants
const ALICE: u64 = 1;
const BOB: u64 = 2;

// ===== REFERENDUM SYSTEM TESTS =====

#[test]
fn create_referendum_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let title = b"Should we build a new hospital?".to_vec();
        let description = b"Proposal to construct a 200-bed hospital in Belmopan".to_vec();
        let options = vec![
            b"Yes".to_vec(),
            b"No".to_vec(),
            b"Abstain".to_vec(),
        ];

        // Create referendum
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(ALICE),
            title,
            description,
            options,
            30, // 30% quorum
            50_400, // Duration in blocks (7 days)
            None, // National referendum
        ));

        // Verify referendum created
        let referendum_id = NextReferendumId::<Test>::get() - 1;
        let referendum = Referendums::<Test>::get(referendum_id).unwrap();
        
        assert_eq!(referendum.id, referendum_id);
        assert_eq!(referendum.quorum_percentage, 30);
        assert_eq!(referendum.district, None);
        assert_eq!(referendum.status, ReferendumStatus::Active);
    });
}

#[test]
fn create_district_referendum_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create district-specific referendum
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(ALICE),
            b"Cayo District Road Repairs".to_vec(),
            b"Should we prioritize road repairs in Cayo?".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            25, // 25% quorum
            50_400,
            Some(1), // District index 1 (Cayo)
        ));

        // Verify district referendum created
        let referendum_id = NextReferendumId::<Test>::get() - 1;
        let referendum = Referendums::<Test>::get(referendum_id).unwrap();
        
        assert_eq!(referendum.district, Some(BelizeDistrict::Cayo));
        assert_eq!(referendum.quorum_percentage, 25);
    });
}

#[test]
fn vote_on_referendum_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create referendum
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(ALICE),
            b"Test Referendum".to_vec(),
            b"Test Description".to_vec(),
            vec![b"Option A".to_vec(), b"Option B".to_vec()],
            30,
            50_400,
            None,
        ));

        let referendum_id = NextReferendumId::<Test>::get() - 1;

        // Bob casts vote for option 0
        assert_ok!(BelizeGovernance::vote_on_referendum(
            RuntimeOrigin::signed(BOB),
            referendum_id,
            0, // Vote for option 0
        ));

        // Verify voter marked
        assert!(ReferendumVotes::<Test>::contains_key(referendum_id, BOB));
    });
}

#[test]
fn vote_on_referendum_prevents_double_voting() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Create referendum
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(ALICE),
            b"Test".to_vec(),
            b"Test".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30,
            50_400,
            None,
        ));

        let referendum_id = NextReferendumId::<Test>::get() - 1;

        // Bob votes
        assert_ok!(BelizeGovernance::vote_on_referendum(
            RuntimeOrigin::signed(BOB),
            referendum_id,
            0,
        ));

        // Bob tries to vote again - should fail
        assert_noop!(
            BelizeGovernance::vote_on_referendum(
                RuntimeOrigin::signed(BOB),
                referendum_id,
                1,
            ),
            Error::<Test>::AlreadyVotedOnReferendum
        );
    });
}

// ===== TREASURY MANAGEMENT TESTS =====

#[test]
fn allocate_district_budget_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let district_index = 1; // Cayo
        let amount = 1_000_000;
        let fiscal_year_blocks = 5_256_000; // 1 year

        // Allocate budget as Root
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            district_index,
            amount,
            fiscal_year_blocks,
        ));

        // Verify budget allocated
        let budget = DistrictBudgets::<Test>::get(BelizeDistrict::Cayo).unwrap();
        assert_eq!(budget.district, BelizeDistrict::Cayo);
        assert_eq!(budget.allocated, amount);
        assert_eq!(budget.spent, 0);
        assert!(budget.is_active);
    });
}

#[test]
fn allocate_district_budget_requires_root() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Alice tries to allocate budget - should fail
        assert_noop!(
            BelizeGovernance::allocate_district_budget(
                RuntimeOrigin::signed(ALICE),
                1,
                1_000_000,
                5_256_000,
            ),
            BadOrigin
        );
    });
}

#[test]
fn propose_treasury_spend_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let recipient = BOB;
        let amount = 50_000_000_000_000; // Between 10-100 trillion (requires 3/7 signatures)
        let description = b"Hospital equipment purchase".to_vec();

        // Alice proposes treasury spend
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(ALICE),
            recipient,
            amount,
            description,
            Some(1), // Cayo district
        ));

        // Verify proposal created
        let proposal_id = NextTreasuryProposalId::<Test>::get() - 1;
        let proposal = TreasurySpendProposals::<Test>::get(proposal_id).unwrap();
        
        assert_eq!(proposal.id, proposal_id);
        assert_eq!(proposal.proposer, ALICE);
        assert_eq!(proposal.recipient, recipient);
        assert_eq!(proposal.amount, amount);
        assert_eq!(proposal.district, Some(BelizeDistrict::Cayo));
        assert_eq!(proposal.threshold, 3); // 50K requires 3/7 signatures
        assert_eq!(proposal.approvals.len(), 0);
        assert!(!proposal.executed);
    });
}

#[test]
fn propose_treasury_spend_tiered_thresholds() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Test small amount (< 10 trillion) - requires 1 signature
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(ALICE),
            BOB,
            5_000_000_000_000, // Below 10 trillion threshold
            b"Small purchase".to_vec(),
            None,
        ));
        let proposal_id_1 = NextTreasuryProposalId::<Test>::get() - 1;
        let proposal_1 = TreasurySpendProposals::<Test>::get(proposal_id_1).unwrap();
        assert_eq!(proposal_1.threshold, 1);

        // Test medium amount (10-100 trillion) - requires 3 signatures
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(ALICE),
            BOB,
            50_000_000_000_000, // Between 10-100 trillion
            b"Medium purchase".to_vec(),
            None,
        ));
        let proposal_id_2 = NextTreasuryProposalId::<Test>::get() - 1;
        let proposal_2 = TreasurySpendProposals::<Test>::get(proposal_id_2).unwrap();
        assert_eq!(proposal_2.threshold, 3);

        // Test large amount (> 100 trillion) - requires 4 signatures
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(ALICE),
            BOB,
            200_000_000_000_000, // Above 100 trillion
            b"Large purchase".to_vec(),
            None,
        ));
        let proposal_id_3 = NextTreasuryProposalId::<Test>::get() - 1;
        let proposal_3 = TreasurySpendProposals::<Test>::get(proposal_id_3).unwrap();
        assert_eq!(proposal_3.threshold, 4);
    });
}

#[test]
fn transfer_district_budget_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Allocate budget to Cayo
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            1, // Cayo
            1_000_000,
            5_256_000,
        ));

        // Transfer 200K from Cayo to Toledo
        assert_ok!(BelizeGovernance::transfer_district_budget(
            RuntimeOrigin::root(),
            1, // From Cayo
            5, // To Toledo
            200_000,
            b"Emergency relief funding".to_vec(),
        ));

        // Verify balances updated
        let cayo_budget = DistrictBudgets::<Test>::get(BelizeDistrict::Cayo).unwrap();
        assert_eq!(cayo_budget.allocated, 800_000);

        let toledo_budget = DistrictBudgets::<Test>::get(BelizeDistrict::Toledo).unwrap();
        assert_eq!(toledo_budget.allocated, 200_000);
    });
}

#[test]
fn transfer_district_budget_requires_root() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Allocate budget first
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            1,
            1_000_000,
            5_256_000,
        ));

        // Alice tries to transfer - should fail
        assert_noop!(
            BelizeGovernance::transfer_district_budget(
                RuntimeOrigin::signed(ALICE),
                1,
                5,
                200_000,
                b"Unauthorized transfer".to_vec(),
            ),
            BadOrigin
        );
    });
}

// ===== EMERGENCY GOVERNANCE TESTS =====

#[test]
fn jaguar_mode_activation_test() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Simulate JaguarMode activation
        let emergency_status = EmergencyStatus {
            active: true,
            emergency_type: EmergencyType::Hurricane,
            description: b"Category 5 hurricane approaching".to_vec().try_into().unwrap(),
            declared_at: 1,
            expires_at: Some(10_000),
            declared_by: Some([0u8; 32]),
        };
        JaguarMode::<Test>::put(emergency_status);

        // Verify JaguarMode is active
        let mode = JaguarMode::<Test>::get().unwrap();
        assert!(mode.active);
        assert_eq!(mode.emergency_type, EmergencyType::Hurricane);
    });
}

#[test]
fn emergency_type_indexing_works() {
    new_test_ext().execute_with(|| {
        // Test EmergencyType to index conversion
        assert_eq!(EmergencyType::Hurricane.to_index(), 0);
        assert_eq!(EmergencyType::HealthCrisis.to_index(), 1);
        assert_eq!(EmergencyType::EconomicCrisis.to_index(), 2);
        assert_eq!(EmergencyType::SecurityThreat.to_index(), 3);
        assert_eq!(EmergencyType::InfrastructureFailure.to_index(), 4);
        assert_eq!(EmergencyType::Other.to_index(), 5);
    });
}

// ===== DISTRICT PROPOSAL TESTS =====

#[test]
fn belize_district_indexing_works() {
    new_test_ext().execute_with(|| {
        // Test district to index conversion
        assert_eq!(BelizeDistrict::Belize.to_index(), 0);
        assert_eq!(BelizeDistrict::Cayo.to_index(), 1);
        assert_eq!(BelizeDistrict::Corozal.to_index(), 2);
        assert_eq!(BelizeDistrict::OrangeWalk.to_index(), 3);
        assert_eq!(BelizeDistrict::StannCreek.to_index(), 4);
        assert_eq!(BelizeDistrict::Toledo.to_index(), 5);

        // Test index to district conversion
        assert_eq!(BelizeDistrict::from_index(0), Some(BelizeDistrict::Belize));
        assert_eq!(BelizeDistrict::from_index(1), Some(BelizeDistrict::Cayo));
        assert_eq!(BelizeDistrict::from_index(2), Some(BelizeDistrict::Corozal));
        assert_eq!(BelizeDistrict::from_index(3), Some(BelizeDistrict::OrangeWalk));
        assert_eq!(BelizeDistrict::from_index(4), Some(BelizeDistrict::StannCreek));
        assert_eq!(BelizeDistrict::from_index(5), Some(BelizeDistrict::Toledo));
        assert_eq!(BelizeDistrict::from_index(6), None); // Invalid index
    });
}

#[test]
fn allocate_district_budget_rejects_invalid_index() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Try to allocate budget with invalid district index
        assert_noop!(
            BelizeGovernance::allocate_district_budget(
                RuntimeOrigin::root(),
                10, // Invalid index (> 5)
                1_000_000,
                5_256_000,
            ),
            Error::<Test>::InvalidDistrict
        );
    });
}

// ===== INTEGRATION TESTS =====

#[test]
fn referendum_workflow_integration() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Step 1: Create referendum for budget allocation
        assert_ok!(BelizeGovernance::create_referendum(
            RuntimeOrigin::signed(ALICE),
            b"Allocate 500K to Cayo District".to_vec(),
            b"Should we allocate additional budget to Cayo for infrastructure?".to_vec(),
            vec![b"Yes".to_vec(), b"No".to_vec()],
            30,
            50_400,
            Some(1), // Cayo district
        ));

        let referendum_id = NextReferendumId::<Test>::get() - 1;

        // Step 2: Cast votes
        assert_ok!(BelizeGovernance::vote_on_referendum(
            RuntimeOrigin::signed(ALICE),
            referendum_id,
            0, // Vote Yes
        ));

        assert_ok!(BelizeGovernance::vote_on_referendum(
            RuntimeOrigin::signed(BOB),
            referendum_id,
            0, // Vote Yes
        ));

        // Step 3: Based on referendum result, allocate budget
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            1, // Cayo
            500_000,
            5_256_000,
        ));

        // Verify budget allocated
        let budget = DistrictBudgets::<Test>::get(BelizeDistrict::Cayo).unwrap();
        assert_eq!(budget.allocated, 500_000);
    });
}

#[test]
fn treasury_proposal_workflow() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Setup: Allocate district budget first
        assert_ok!(BelizeGovernance::allocate_district_budget(
            RuntimeOrigin::root(),
            1, // Cayo
            1_000_000,
            5_256_000,
        ));

        // Step 1: Propose treasury spend (requires 3/7 signatures)
        let amount = 50_000_000_000_000; // Between 10-100 trillion (requires 3/7)
        assert_ok!(BelizeGovernance::propose_treasury_spend(
            RuntimeOrigin::signed(ALICE),
            BOB,
            amount,
            b"Medical equipment".to_vec(),
            Some(1), // Cayo district
        ));

        let proposal_id = NextTreasuryProposalId::<Test>::get() - 1;
        let proposal = TreasurySpendProposals::<Test>::get(proposal_id).unwrap();
        
        // Verify proposal created with correct threshold
        assert_eq!(proposal.threshold, 3);
        assert_eq!(proposal.amount, amount);
        assert_eq!(proposal.district, Some(BelizeDistrict::Cayo));
    });
}
