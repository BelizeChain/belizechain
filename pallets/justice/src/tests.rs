//! Comprehensive tests for the Justice pallet

use crate::{mock::*, pallet::*};
use frame_support::{assert_noop, assert_ok};

fn sample_evidence() -> [u8; 32] {
    [0xEE; 32]
}

// ============================================================================
// OPEN DISPUTE TESTS
// ============================================================================

#[test]
fn open_dispute_works() {
    new_test_ext().execute_with(|| {
        let reserved_before = Balances::reserved_balance(ALICE);

        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE),
            BOB,
            sample_evidence(),
            0, // Minor
        ));

        assert_eq!(Justice::dispute_counter(), 1);

        let dispute = Justice::disputes(1).expect("dispute must exist");
        assert_eq!(dispute.disputant, ALICE);
        assert_eq!(dispute.target, BOB);
        assert_eq!(dispute.evidence_hash, sample_evidence());
        assert_eq!(dispute.severity, DisputeSeverity::Minor);
        assert_eq!(dispute.status, DisputeStatus::Pending);
        assert!(dispute.resolution.is_none());
        assert_eq!(dispute.bond, 500); // OpenDisputeBond
        assert!(dispute.appeal_evidence.is_none());

        // Bond reserved
        assert_eq!(Balances::reserved_balance(ALICE), reserved_before + 500);

        // Target now in cooling-off
        assert_eq!(Justice::rehab_status(BOB), RehabStatus::InCoolingOff);
        assert!(Justice::cooling_off_end(BOB).is_some());
    });
}

#[test]
fn open_dispute_all_severities() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 0,
        ));
        assert_eq!(Justice::disputes(1).unwrap().severity, DisputeSeverity::Minor);

        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 1,
        ));
        assert_eq!(Justice::disputes(2).unwrap().severity, DisputeSeverity::Moderate);

        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 2,
        ));
        assert_eq!(Justice::disputes(3).unwrap().severity, DisputeSeverity::Severe);
    });
}

#[test]
fn open_dispute_invalid_severity_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Justice::open_dispute(
                RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 3,
            ),
            Error::<Test>::InvalidSeverity
        );

        assert_noop!(
            Justice::open_dispute(
                RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 255,
            ),
            Error::<Test>::InvalidSeverity
        );
    });
}

#[test]
fn open_dispute_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let poor = 999u64;
        assert_noop!(
            Justice::open_dispute(
                RuntimeOrigin::signed(poor), BOB, sample_evidence(), 0,
            ),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn open_dispute_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 0,
        ));

        System::assert_last_event(
            Event::<Test>::DisputeOpened {
                dispute_id: 1,
                disputant: ALICE,
                target: BOB,
            }.into()
        );
    });
}

// ============================================================================
// MEDIATOR MANAGEMENT TESTS
// ============================================================================

#[test]
fn add_mediator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), MEDIATOR));

        let list = Justice::mediator_list();
        assert!(list.contains(&MEDIATOR));
    });
}

#[test]
fn add_mediator_non_governance_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Justice::add_mediator(RuntimeOrigin::signed(ALICE), MEDIATOR),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_mediator_list_full_fails() {
    new_test_ext().execute_with(|| {
        // MaxMediators = 10
        for i in 0..10 {
            assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), 100 + i));
        }

        assert_noop!(
            Justice::add_mediator(RuntimeOrigin::root(), 200),
            Error::<Test>::MediatorListFull
        );
    });
}

#[test]
fn remove_mediator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), MEDIATOR));
        assert!(Justice::mediator_list().contains(&MEDIATOR));

        assert_ok!(Justice::remove_mediator(RuntimeOrigin::root(), MEDIATOR));
        assert!(!Justice::mediator_list().contains(&MEDIATOR));
    });
}

#[test]
fn remove_mediator_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Justice::remove_mediator(RuntimeOrigin::root(), 999),
            Error::<Test>::MediatorNotFound
        );
    });
}

#[test]
fn add_remove_mediator_emits_events() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), MEDIATOR));
        System::assert_last_event(
            Event::<Test>::MediatorAdded { mediator: MEDIATOR }.into()
        );

        assert_ok!(Justice::remove_mediator(RuntimeOrigin::root(), MEDIATOR));
        System::assert_last_event(
            Event::<Test>::MediatorRemoved { mediator: MEDIATOR }.into()
        );
    });
}

// ============================================================================
// MEDIATOR RULING TESTS
// ============================================================================

fn setup_dispute_with_mediator() -> u32 {
    assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), MEDIATOR));

    assert_ok!(Justice::open_dispute(
        RuntimeOrigin::signed(ALICE),
        BOB,
        sample_evidence(),
        2, // Severe
    ));

    Justice::dispute_counter()
}

#[test]
fn mediator_ruling_dismissed_works() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR),
            id,
            0, // Dismissed
            0,
        ));

        let dispute = Justice::disputes(id).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Ruled);
        assert_eq!(dispute.resolution, Some(DisputeResolution::Dismissed));

        // Disputant bond refunded on Dismissed
        assert_eq!(Balances::reserved_balance(ALICE), 0);
    });
}

#[test]
fn mediator_ruling_upheld_works() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR),
            id,
            1, // Upheld
            0,
        ));

        let dispute = Justice::disputes(id).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Ruled);
        assert_eq!(dispute.resolution, Some(DisputeResolution::Upheld));
    });
}

#[test]
fn mediator_ruling_mediated_works() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR),
            id,
            2,    // Mediated
            5000, // 50% slash
        ));

        let dispute = Justice::disputes(id).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Ruled);
        assert_eq!(
            dispute.resolution,
            Some(DisputeResolution::Mediated { slash_bps: 5000 })
        );
    });
}

#[test]
fn mediator_ruling_not_approved_mediator_fails() {
    new_test_ext().execute_with(|| {
        // Open dispute without adding DAVE as mediator
        assert_ok!(Justice::open_dispute(
            RuntimeOrigin::signed(ALICE), BOB, sample_evidence(), 0,
        ));

        assert_noop!(
            Justice::mediator_ruling(
                RuntimeOrigin::signed(DAVE), 1, 0, 0,
            ),
            Error::<Test>::NotApprovedMediator
        );
    });
}

#[test]
fn mediator_ruling_dispute_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::add_mediator(RuntimeOrigin::root(), MEDIATOR));

        assert_noop!(
            Justice::mediator_ruling(
                RuntimeOrigin::signed(MEDIATOR), 999, 0, 0,
            ),
            Error::<Test>::DisputeNotFound
        );
    });
}

#[test]
fn mediator_ruling_invalid_status_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        // First ruling
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 0, 0,
        ));

        // Second ruling — already Ruled
        assert_noop!(
            Justice::mediator_ruling(
                RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
            ),
            Error::<Test>::InvalidDisputeStatus
        );
    });
}

#[test]
fn mediator_ruling_invalid_code_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_noop!(
            Justice::mediator_ruling(
                RuntimeOrigin::signed(MEDIATOR), id, 5, 0,
            ),
            Error::<Test>::InvalidDisputeStatus
        );
    });
}

#[test]
fn mediator_ruling_emits_event() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        System::assert_has_event(
            Event::<Test>::MediatorRulingIssued {
                dispute_id: id,
                resolution: DisputeResolution::Upheld,
            }.into()
        );
    });
}

// ============================================================================
// MEDIATOR RULING WITH ESCROWED SLASH
// ============================================================================

#[test]
fn mediator_ruling_dismissed_with_escrow_refunds() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        // Escrow a slash on BOB
        assert_ok!(Justice::escrow_slash(&BOB, 10_000));
        assert_eq!(Balances::reserved_balance(BOB), 10_000);

        // Dismissed ruling → slash refunded
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 0, 0,
        ));

        assert_eq!(Balances::reserved_balance(BOB), 0);
        assert!(Justice::slash_pending(BOB).is_none());

        System::assert_has_event(
            Event::<Test>::SlashRefunded { account: BOB, amount: 10_000 }.into()
        );
    });
}

#[test]
fn mediator_ruling_upheld_with_escrow_slashes() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        let balance_before = Balances::free_balance(BOB);
        assert_ok!(Justice::escrow_slash(&BOB, 10_000));

        // Upheld → slash executed
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        // BOB lost the slashed amount
        let balance_after = Balances::free_balance(BOB);
        assert!(balance_after < balance_before);

        System::assert_has_event(
            Event::<Test>::SlashExecuted { account: BOB, amount: 10_000 }.into()
        );
    });
}

#[test]
fn mediator_ruling_mediated_with_escrow_partial_slash() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::escrow_slash(&BOB, 10_000));

        // Mediated at 50% (5000 bps)
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 2, 5000,
        ));

        // Slash and refund events should both be emitted
        System::assert_has_event(
            Event::<Test>::SlashExecuted { account: BOB, amount: 5_000 }.into()
        );
        System::assert_has_event(
            Event::<Test>::SlashRefunded { account: BOB, amount: 5_000 }.into()
        );
    });
}

// ============================================================================
// APPEAL RULING TESTS
// ============================================================================

#[test]
fn appeal_ruling_works() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        // Issue ruling
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        // Target (BOB) appeals
        let counter_evidence = [0xCC; 32];
        assert_ok!(Justice::appeal_ruling(
            RuntimeOrigin::signed(BOB),
            id,
            counter_evidence,
        ));

        let dispute = Justice::disputes(id).unwrap();
        assert_eq!(dispute.status, DisputeStatus::Appealed);
        assert_eq!(dispute.appeal_evidence, Some(counter_evidence));
    });
}

#[test]
fn appeal_ruling_not_target_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        // ALICE (disputant) tries to appeal — not the target
        assert_noop!(
            Justice::appeal_ruling(
                RuntimeOrigin::signed(ALICE), id, [0u8; 32],
            ),
            Error::<Test>::NotDisputeTarget
        );
    });
}

#[test]
fn appeal_ruling_wrong_status_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();
        // Status is still Pending, not Ruled
        assert_noop!(
            Justice::appeal_ruling(
                RuntimeOrigin::signed(BOB), id, [0u8; 32],
            ),
            Error::<Test>::InvalidDisputeStatus
        );
    });
}

#[test]
fn appeal_ruling_double_appeal_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        assert_ok!(Justice::appeal_ruling(
            RuntimeOrigin::signed(BOB), id, [0xAA; 32],
        ));

        // Second appeal
        assert_noop!(
            Justice::appeal_ruling(
                RuntimeOrigin::signed(BOB), id, [0xBB; 32],
            ),
            Error::<Test>::InvalidDisputeStatus
        );
    });
}

#[test]
fn appeal_ruling_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Justice::appeal_ruling(
                RuntimeOrigin::signed(BOB), 999, [0u8; 32],
            ),
            Error::<Test>::DisputeNotFound
        );
    });
}

#[test]
fn appeal_ruling_emits_event() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 1, 0,
        ));

        assert_ok!(Justice::appeal_ruling(
            RuntimeOrigin::signed(BOB), id, [0xCC; 32],
        ));

        System::assert_last_event(
            Event::<Test>::RulingAppealed { dispute_id: id, by: BOB }.into()
        );
    });
}

// ============================================================================
// REHABILITATION TESTS
// ============================================================================

#[test]
fn complete_rehabilitation_works() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        // Escrow some amount for the target — this is required so that
        // the Dismissed ruling transitions the target to InRehabilitation.
        assert_ok!(Justice::escrow_slash(&BOB, 5_000));

        // Dismissed ruling → target enters InRehabilitation (via escrow path)
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 0, 0,
        ));
        assert_eq!(Justice::rehab_status(BOB), RehabStatus::InRehabilitation);

        // Advance past cooling-off period (block 1 + 100 = 101)
        run_to_block(102);

        // Governance reinstates
        assert_ok!(Justice::complete_rehabilitation(
            RuntimeOrigin::root(),
            BOB,
        ));

        assert_eq!(Justice::rehab_status(BOB), RehabStatus::Reinstated);
        assert!(Justice::cooling_off_end(BOB).is_none());
    });
}

#[test]
fn complete_rehabilitation_cooling_off_active_fails() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();

        // Escrow so the Dismissed ruling transitions to InRehabilitation
        assert_ok!(Justice::escrow_slash(&BOB, 5_000));

        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 0, 0,
        ));

        // Don't advance blocks — cooling-off still active
        assert_noop!(
            Justice::complete_rehabilitation(RuntimeOrigin::root(), BOB),
            Error::<Test>::CoolingOffActive
        );
    });
}

#[test]
fn complete_rehabilitation_invalid_state_fails() {
    new_test_ext().execute_with(|| {
        // BOB starts as Clean
        assert_noop!(
            Justice::complete_rehabilitation(RuntimeOrigin::root(), BOB),
            Error::<Test>::InvalidRehabState
        );
    });
}

#[test]
fn complete_rehabilitation_emits_event() {
    new_test_ext().execute_with(|| {
        let id = setup_dispute_with_mediator();
        assert_ok!(Justice::escrow_slash(&BOB, 5_000));
        assert_ok!(Justice::mediator_ruling(
            RuntimeOrigin::signed(MEDIATOR), id, 0, 0,
        ));
        run_to_block(102);

        assert_ok!(Justice::complete_rehabilitation(RuntimeOrigin::root(), BOB));

        System::assert_last_event(
            Event::<Test>::AccountReinstated { account: BOB }.into()
        );
    });
}

// ============================================================================
// ESCROW SLASH (PUBLIC API) TESTS
// ============================================================================

#[test]
fn escrow_slash_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Justice::escrow_slash(&BOB, 5_000));
        assert_eq!(Justice::slash_pending(BOB), Some(5_000));
        assert_eq!(Balances::reserved_balance(BOB), 5_000);
    });
}

#[test]
fn has_pending_review_works() {
    new_test_ext().execute_with(|| {
        assert!(!Justice::has_pending_review(&BOB));
        assert_ok!(Justice::escrow_slash(&BOB, 1_000));
        assert!(Justice::has_pending_review(&BOB));
    });
}

#[test]
fn rehabilitation_status_query_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Justice::rehabilitation_status(&BOB), RehabStatus::Clean);
    });
}

// ============================================================================
// SEVERITY CONVERSION TESTS
// ============================================================================

#[test]
fn severity_from_u8_covers_all() {
    assert_eq!(DisputeSeverity::from_u8(0), Some(DisputeSeverity::Minor));
    assert_eq!(DisputeSeverity::from_u8(1), Some(DisputeSeverity::Moderate));
    assert_eq!(DisputeSeverity::from_u8(2), Some(DisputeSeverity::Severe));
    assert_eq!(DisputeSeverity::from_u8(3), None);
    assert_eq!(DisputeSeverity::from_u8(255), None);
}

#[test]
fn rehab_status_default_is_clean() {
    assert_eq!(RehabStatus::default(), RehabStatus::Clean);
}
