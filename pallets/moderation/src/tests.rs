//! Comprehensive tests for the Moderation pallet
#![cfg(test)]

use crate::{mock::*, pallet::*, *};
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// FLAG CONTENT TESTS
// ============================================================================

#[test]
fn flag_content_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(ALICE),
            content_a(),
            0, // HateSpeech
        ));

        assert_eq!(Moderation::flag_count(content_a()), 1);
        assert!(Moderation::content_flag(content_a(), ALICE).is_some());
    });
}

#[test]
fn flag_content_all_reasons_work() {
    new_test_ext().execute_with(|| {
        // HateSpeech (0)
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(ALICE), content_a(), 0,
        ));
        assert_eq!(
            Moderation::content_flag(content_a(), ALICE),
            Some(FlagReason::HateSpeech)
        );

        // Misinformation (1) — different user, same content
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(BOB), content_a(), 1,
        ));
        assert_eq!(
            Moderation::content_flag(content_a(), BOB),
            Some(FlagReason::Misinformation)
        );

        // Spam (2)
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(CHARLIE), content_b(), 2,
        ));
        assert_eq!(
            Moderation::content_flag(content_b(), CHARLIE),
            Some(FlagReason::Spam)
        );

        // IllegalContent (3)
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(DAVE), content_b(), 3,
        ));
        assert_eq!(
            Moderation::content_flag(content_b(), DAVE),
            Some(FlagReason::IllegalContent)
        );

        // AddictivePattern (4)
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(ALICE), content_b(), 4,
        ));
        assert_eq!(
            Moderation::content_flag(content_b(), ALICE),
            Some(FlagReason::AddictivePattern)
        );
    });
}

#[test]
fn flag_content_invalid_reason_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(ALICE), content_a(), 5),
            Error::<Test>::InvalidFlagReason
        );

        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(ALICE), content_a(), 255),
            Error::<Test>::InvalidFlagReason
        );
    });
}

#[test]
fn flag_content_double_flag_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(ALICE), content_a(), 0,
        ));

        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(ALICE), content_a(), 1),
            Error::<Test>::AlreadyFlagged
        );
    });
}

#[test]
fn flag_content_auto_queues_at_threshold() {
    new_test_ext().execute_with(|| {
        // FlagThreshold = 3
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), content_a(), 0));
        assert!(!Moderation::is_queued(content_a()));

        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), content_a(), 1));
        assert!(!Moderation::is_queued(content_a()));

        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), content_a(), 2));
        assert!(Moderation::is_queued(content_a()));

        // Verify auto-queue event was emitted
        System::assert_has_event(
            Event::<Test>::ContentAutoQueued {
                content_hash: content_a(),
                trigger: AutoQueueTrigger::FlagThreshold,
            }.into()
        );
    });
}

#[test]
fn flag_content_already_ruled_fails() {
    new_test_ext().execute_with(|| {
        // Setup: add moderator, flag to threshold, rule
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));

        for (i, user) in [ALICE, BOB, CHARLIE].iter().enumerate() {
            assert_ok!(Moderation::flag_content(
                RuntimeOrigin::signed(*user), content_a(), i as u8,
            ));
        }

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 0,
        ));

        // Now try to flag same content again
        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(DAVE), content_a(), 0),
            Error::<Test>::AlreadyRuled
        );
    });
}

#[test]
fn flag_content_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::flag_content(
            RuntimeOrigin::signed(ALICE), content_a(), 2,
        ));

        System::assert_last_event(
            Event::<Test>::ContentFlagged {
                content_hash: content_a(),
                flagger: ALICE,
                reason: 2,
                total_flags: 1,
            }.into()
        );
    });
}

// ============================================================================
// MODERATOR MANAGEMENT TESTS
// ============================================================================

#[test]
fn add_moderator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));
        assert!(Moderation::moderator_set().contains(&MODERATOR_1));
    });
}

#[test]
fn add_moderator_non_admin_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Moderation::add_moderator(RuntimeOrigin::signed(ALICE), MODERATOR_1),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn add_moderator_already_exists_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));

        assert_noop!(
            Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1),
            Error::<Test>::AlreadyModerator
        );
    });
}

#[test]
fn add_moderator_set_full_fails() {
    new_test_ext().execute_with(|| {
        // MaxModerators = 10
        for i in 0..10 {
            assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), 100 + i));
        }

        assert_noop!(
            Moderation::add_moderator(RuntimeOrigin::root(), 200),
            Error::<Test>::ModeratorSetFull
        );
    });
}

#[test]
fn remove_moderator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));
        assert!(Moderation::moderator_set().contains(&MODERATOR_1));

        assert_ok!(Moderation::remove_moderator(RuntimeOrigin::root(), MODERATOR_1));
        assert!(!Moderation::moderator_set().contains(&MODERATOR_1));
    });
}

#[test]
fn remove_moderator_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Moderation::remove_moderator(RuntimeOrigin::root(), 999),
            Error::<Test>::ModeratorNotFound
        );
    });
}

#[test]
fn moderator_events() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));
        System::assert_last_event(
            Event::<Test>::ModeratorAdded { account: MODERATOR_1 }.into()
        );

        assert_ok!(Moderation::remove_moderator(RuntimeOrigin::root(), MODERATOR_1));
        System::assert_last_event(
            Event::<Test>::ModeratorRemoved { account: MODERATOR_1 }.into()
        );
    });
}

// ============================================================================
// REVIEW CONTENT TESTS
// ============================================================================

fn setup_queued_content(hash: [u8; 32]) {
    // Add moderator
    assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));

    // Flag to threshold (3)
    assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash, 0));
    assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash, 1));
    assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash, 2));
}

#[test]
fn review_content_cleared_works() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 0, // Cleared
        ));

        // Dequeued
        assert!(!Moderation::is_queued(content_a()));
        // Ruling stored
        assert_eq!(Moderation::ruling(content_a()), Some(ModerationRuling::Cleared));
    });
}

#[test]
fn review_content_removed_works() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 1, // Removed
        ));

        assert!(!Moderation::is_queued(content_a()));
        assert_eq!(Moderation::ruling(content_a()), Some(ModerationRuling::Removed));
        assert!(Moderation::is_removed(&content_a()));
    });
}

#[test]
fn review_content_escalated_works() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 2, // Escalated
        ));

        assert!(!Moderation::is_queued(content_a()));
        assert_eq!(Moderation::ruling(content_a()), Some(ModerationRuling::Escalated));
    });
}

#[test]
fn review_content_not_moderator_fails() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_noop!(
            Moderation::review_content(
                RuntimeOrigin::signed(DAVE), content_a(), 0,
            ),
            Error::<Test>::NotModerator
        );
    });
}

#[test]
fn review_content_not_in_queue_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));

        // content_a() was never flagged/queued
        assert_noop!(
            Moderation::review_content(
                RuntimeOrigin::signed(MODERATOR_1), content_a(), 0,
            ),
            Error::<Test>::NotInQueue
        );
    });
}

#[test]
fn review_content_invalid_ruling_fails() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_noop!(
            Moderation::review_content(
                RuntimeOrigin::signed(MODERATOR_1), content_a(), 3,
            ),
            Error::<Test>::InvalidRuling
        );
    });
}

#[test]
fn review_content_emits_event() {
    new_test_ext().execute_with(|| {
        setup_queued_content(content_a());

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 1,
        ));

        System::assert_last_event(
            Event::<Test>::ContentRuled {
                content_hash: content_a(),
                moderator: MODERATOR_1,
                ruling: 1,
            }.into()
        );
    });
}

// ============================================================================
// NAWAL ASSESSMENT TESTS
// ============================================================================

#[test]
fn submit_nawal_assessment_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 30,
        ));

        assert_eq!(Moderation::nawal_score(content_a()), Some(30));
        // Score 30 <= 50 threshold, should NOT auto-queue
        assert!(!Moderation::is_queued(content_a()));
    });
}

#[test]
fn submit_nawal_assessment_high_score_auto_queues() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 75, // > 50
        ));

        assert!(Moderation::is_queued(content_a()));
        assert_eq!(Moderation::nawal_score(content_a()), Some(75));

        System::assert_has_event(
            Event::<Test>::ContentAutoQueued {
                content_hash: content_a(),
                trigger: AutoQueueTrigger::NawalScore,
            }.into()
        );
    });
}

#[test]
fn submit_nawal_assessment_boundary_score_does_not_auto_queue() {
    new_test_ext().execute_with(|| {
        // Exactly at threshold — NOT greater than
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 50,
        ));

        assert!(!Moderation::is_queued(content_a()));
    });
}

#[test]
fn submit_nawal_assessment_overwrites_previous() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 20,
        ));
        assert_eq!(Moderation::nawal_score(content_a()), Some(20));

        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 80,
        ));
        assert_eq!(Moderation::nawal_score(content_a()), Some(80));
    });
}

#[test]
fn submit_nawal_assessment_already_queued_does_not_requeue() {
    new_test_ext().execute_with(|| {
        // First assessment queues it
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 75,
        ));
        assert!(Moderation::is_queued(content_a()));

        // Second assessment with even higher score doesn't duplicate queue event
        // (the content is already queued, so no new ContentAutoQueued event)
        let events_before = System::events().len();
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 90,
        ));
        // Only NawalAssessmentSubmitted event, no second ContentAutoQueued
        let new_events: Vec<_> = System::events()[events_before..].to_vec();
        assert_eq!(new_events.len(), 1); // Only NawalAssessmentSubmitted
    });
}

#[test]
fn submit_nawal_assessment_already_ruled_does_not_queue() {
    new_test_ext().execute_with(|| {
        // Setup and rule content
        setup_queued_content(content_a());
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 0,
        ));
        assert!(!Moderation::is_queued(content_a()));

        // Submit high nawal score — should NOT re-queue since already ruled
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 95,
        ));
        assert!(!Moderation::is_queued(content_a()));
    });
}

#[test]
fn submit_nawal_assessment_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 42,
        ));

        System::assert_last_event(
            Event::<Test>::NawalAssessmentSubmitted {
                content_hash: content_a(),
                score: 42,
            }.into()
        );
    });
}

#[test]
fn submit_nawal_assessment_non_oracle_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Moderation::submit_nawal_assessment(
                RuntimeOrigin::signed(ALICE), content_a(), 90,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// PUBLIC API TESTS
// ============================================================================

#[test]
fn is_removed_works() {
    new_test_ext().execute_with(|| {
        assert!(!Moderation::is_removed(&content_a()));

        setup_queued_content(content_a());
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), content_a(), 1, // Removed
        ));
        assert!(Moderation::is_removed(&content_a()));
    });
}

#[test]
fn is_queued_for_review_works() {
    new_test_ext().execute_with(|| {
        assert!(!Moderation::is_queued_for_review(&content_a()));

        // Flag to threshold
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), content_a(), 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), content_a(), 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), content_a(), 2));

        assert!(Moderation::is_queued_for_review(&content_a()));
    });
}

#[test]
fn nawal_risk_score_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Moderation::nawal_risk_score(&content_a()), None);

        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 55,
        ));
        assert_eq!(Moderation::nawal_risk_score(&content_a()), Some(55));
    });
}

// ============================================================================
// FLAG REASON / RULING CONVERSION TESTS
// ============================================================================

#[test]
fn flag_reason_from_index_covers_all() {
    assert_eq!(FlagReason::from_index(0), Some(FlagReason::HateSpeech));
    assert_eq!(FlagReason::from_index(1), Some(FlagReason::Misinformation));
    assert_eq!(FlagReason::from_index(2), Some(FlagReason::Spam));
    assert_eq!(FlagReason::from_index(3), Some(FlagReason::IllegalContent));
    assert_eq!(FlagReason::from_index(4), Some(FlagReason::AddictivePattern));
    assert_eq!(FlagReason::from_index(5), None);
    assert_eq!(FlagReason::from_index(255), None);
}

#[test]
fn moderation_ruling_from_index_covers_all() {
    assert_eq!(ModerationRuling::from_index(0), Some(ModerationRuling::Cleared));
    assert_eq!(ModerationRuling::from_index(1), Some(ModerationRuling::Removed));
    assert_eq!(ModerationRuling::from_index(2), Some(ModerationRuling::Escalated));
    assert_eq!(ModerationRuling::from_index(3), None);
    assert_eq!(ModerationRuling::from_index(255), None);
}

// ============================================================================
// EXPANDED COVERAGE TESTS
// ============================================================================

#[test]
fn remove_moderator_requires_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_1));
        assert_noop!(
            Moderation::remove_moderator(RuntimeOrigin::signed(ALICE), MODERATOR_1),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn removed_moderator_cannot_review() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        setup_queued_content(hash); // adds MODERATOR_1

        // Remove the moderator
        assert_ok!(Moderation::remove_moderator(RuntimeOrigin::root(), MODERATOR_1));

        // Should fail with NotModerator
        assert_noop!(
            Moderation::review_content(RuntimeOrigin::signed(MODERATOR_1), hash, 1),
            Error::<Test>::NotModerator
        );
    });
}

#[test]
fn already_ruled_content_rejected_for_all_rulings() {
    new_test_ext().execute_with(|| {
        // Test with Removed ruling (1)
        let hash_removed = [0x11; 32];
        setup_queued_content(hash_removed); // adds MODERATOR_1
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_removed, 1,
        ));
        // Subsequent flag on ruled content should fail
        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(DAVE), hash_removed, 0),
            Error::<Test>::AlreadyRuled
        );

        // Test with Escalated ruling (2) — moderator already added by first setup
        let hash_escalated = [0x22; 32];
        // Flag manually (moderator already exists)
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash_escalated, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash_escalated, 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash_escalated, 2));
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_escalated, 2,
        ));
        assert_noop!(
            Moderation::flag_content(RuntimeOrigin::signed(DAVE), hash_escalated, 0),
            Error::<Test>::AlreadyRuled
        );
    });
}

#[test]
fn review_already_ruled_content_fails() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        setup_queued_content(hash);
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash, 0, // Cleared
        ));
        // Content is dequeued after ruling → NotInQueue
        assert_noop!(
            Moderation::review_content(RuntimeOrigin::signed(MODERATOR_1), hash, 1),
            Error::<Test>::NotInQueue
        );
    });
}

#[test]
fn nawal_score_255_accepted() {
    new_test_ext().execute_with(|| {
        // Score 255 > threshold 50, should auto-queue
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 255,
        ));
        assert_eq!(Moderation::nawal_risk_score(&content_a()), Some(255));
        assert!(Moderation::is_queued_for_review(&content_a()));
    });
}

#[test]
fn nawal_score_zero_does_not_autoqueue() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 0,
        ));
        assert_eq!(Moderation::nawal_risk_score(&content_a()), Some(0));
        assert!(!Moderation::is_queued_for_review(&content_a()));
    });
}

#[test]
fn nawal_score_exactly_100_auto_queues() {
    new_test_ext().execute_with(|| {
        assert_ok!(Moderation::submit_nawal_assessment(
            RuntimeOrigin::root(), content_a(), 100,
        ));
        assert!(Moderation::is_queued_for_review(&content_a()));
    });
}

#[test]
fn combined_flags_below_and_nawal_above_queues() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        // 2 flags (below threshold of 3) → not queued
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash, 1));
        assert!(!Moderation::is_queued_for_review(&hash));

        // Nawal score above threshold → queued
        assert_ok!(Moderation::submit_nawal_assessment(RuntimeOrigin::root(), hash, 60));
        assert!(Moderation::is_queued_for_review(&hash));
    });
}

#[test]
fn flags_accepted_after_auto_queue() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        setup_queued_content(hash); // 3 flags, auto-queued
        assert!(Moderation::is_queued_for_review(&hash));

        // DAVE can still flag (4th flag)
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(DAVE), hash, 2));
        assert_eq!(FlagCounts::<Test>::get(hash), 4);
    });
}

#[test]
fn multiple_content_items_queued_independently() {
    new_test_ext().execute_with(|| {
        let hash_a = content_a();
        let hash_b = content_b();
        setup_queued_content(hash_a); // adds MODERATOR_1
        // Flag hash_b manually (moderator already added)
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash_b, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash_b, 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash_b, 2));

        assert!(Moderation::is_queued_for_review(&hash_a));
        assert!(Moderation::is_queued_for_review(&hash_b));

        // Rule content_a as Removed, content_b stays queued
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_a, 1,
        ));
        assert!(!Moderation::is_queued_for_review(&hash_a));
        assert!(Moderation::is_queued_for_review(&hash_b));
        assert!(Moderation::is_removed(&hash_a));
    });
}

#[test]
fn two_moderators_rule_different_content() {
    new_test_ext().execute_with(|| {
        let hash_a = content_a();
        let hash_b = content_b();
        setup_queued_content(hash_a); // adds MODERATOR_1
        assert_ok!(Moderation::add_moderator(RuntimeOrigin::root(), MODERATOR_2));
        // Flag hash_b manually (moderator already added)
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash_b, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash_b, 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash_b, 2));

        // MODERATOR_1 rules content_a, MODERATOR_2 rules content_b
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_a, 0, // Cleared
        ));
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_2), hash_b, 1, // Removed
        ));

        assert!(!Moderation::is_removed(&hash_a)); // Cleared
        assert!(Moderation::is_removed(&hash_b));   // Removed
    });
}

#[test]
fn is_removed_false_for_cleared_and_escalated() {
    new_test_ext().execute_with(|| {
        let hash_c = [0x33; 32];
        let hash_e = [0x44; 32];
        setup_queued_content(hash_c); // adds MODERATOR_1
        // Flag hash_e manually
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash_e, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash_e, 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash_e, 2));

        // Clear one
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_c, 0,
        ));
        assert!(!Moderation::is_removed(&hash_c));

        // Escalate the other
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_e, 2,
        ));
        assert!(!Moderation::is_removed(&hash_e));
    });
}

#[test]
fn is_queued_returns_false_after_ruling() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        setup_queued_content(hash);
        assert!(Moderation::is_queued_for_review(&hash));

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash, 0,
        ));
        assert!(!Moderation::is_queued_for_review(&hash));
    });
}

#[test]
fn content_flagged_total_flags_incremental() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash, 0));
        System::assert_has_event(
            Event::ContentFlagged { content_hash: hash, flagger: ALICE, reason: 0, total_flags: 1 }.into()
        );

        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash, 1));
        System::assert_has_event(
            Event::ContentFlagged { content_hash: hash, flagger: BOB, reason: 1, total_flags: 2 }.into()
        );

        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash, 2));
        System::assert_has_event(
            Event::ContentFlagged { content_hash: hash, flagger: CHARLIE, reason: 2, total_flags: 3 }.into()
        );
    });
}

#[test]
fn content_ruled_event_for_cleared_and_escalated() {
    new_test_ext().execute_with(|| {
        let hash_c = [0x55; 32];
        let hash_e = [0x66; 32];
        setup_queued_content(hash_c); // adds MODERATOR_1
        // Flag hash_e manually
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(ALICE), hash_e, 0));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(BOB), hash_e, 1));
        assert_ok!(Moderation::flag_content(RuntimeOrigin::signed(CHARLIE), hash_e, 2));

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_c, 0,
        ));
        System::assert_has_event(
            Event::ContentRuled { content_hash: hash_c, moderator: MODERATOR_1, ruling: 0 }.into()
        );

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash_e, 2,
        ));
        System::assert_has_event(
            Event::ContentRuled { content_hash: hash_e, moderator: MODERATOR_1, ruling: 2 }.into()
        );
    });
}

#[test]
fn flag_counts_persist_after_ruling() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        setup_queued_content(hash); // 3 flags
        assert_eq!(FlagCounts::<Test>::get(hash), 3);

        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash, 0,
        ));
        // Counts persist (not cleaned up by ruling)
        assert_eq!(FlagCounts::<Test>::get(hash), 3);
    });
}

#[test]
fn nawal_assessment_persists_after_ruling() {
    new_test_ext().execute_with(|| {
        let hash = content_a();
        assert_ok!(Moderation::submit_nawal_assessment(RuntimeOrigin::root(), hash, 80));
        setup_queued_content(hash);
        assert_ok!(Moderation::review_content(
            RuntimeOrigin::signed(MODERATOR_1), hash, 1,
        ));
        // Assessment persists
        assert_eq!(Moderation::nawal_risk_score(&hash), Some(80));
    });
}
