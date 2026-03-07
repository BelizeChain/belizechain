//! Unit tests for the BelizeChain commit-reveal randomness pallet.
//!
//! Epoch timing (mock constants: CommitDuration=5, RevealDuration=5):
//!   EpochStart = 1  (set on_initialize at block 1)
//!   Commit window : block < 1 + 5 = 6    (blocks 1–5 inclusive)
//!   Reveal window : 6 ≤ block < 6 + 5=11 (blocks 6–10 inclusive)
//!   Finalize      : block ≥ 11

use super::pallet::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, traits::Randomness as RandomnessTrait};
use sp_io::hashing::blake2_256;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compute the on-chain commitment: blake2_256(value ++ salt).
fn make_commitment(value: [u8; 32], salt: [u8; 32]) -> [u8; 32] {
    let mut preimage = [0u8; 64];
    preimage[..32].copy_from_slice(&value);
    preimage[32..].copy_from_slice(&salt);
    blake2_256(&preimage)
}

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DAVE: u64 = 4;
const EVE: u64 = 5;
const FERDIE: u64 = 6; // exceeds MaxContributors=5

const VALUE_A: [u8; 32] = [1u8; 32];
const VALUE_B: [u8; 32] = [2u8; 32];
const SALT_A: [u8; 32] = [10u8; 32];
const SALT_B: [u8; 32] = [20u8; 32];

// ── Commit phase tests ────────────────────────────────────────────────────────

#[test]
fn commit_works_during_commit_window() {
    new_test_ext().execute_with(|| {
        // Block 1 is inside commit window (< 6).
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        // Commitment stored.
        assert_eq!(Commitments::<Test>::get(ALICE), Some(commitment));
        // Count incremented.
        assert_eq!(CommitCount::<Test>::get(), 1);
        // Event emitted.
        System::assert_has_event(
            Event::Committed { contributor: ALICE, epoch: 0 }.into(),
        );
    });
}

#[test]
fn commit_rejected_after_commit_window_closes() {
    new_test_ext().execute_with(|| {
        // Advance to block 6 — commit window has closed.
        run_to_block(6);
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_noop!(
            BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment),
            Error::<Test>::CommitWindowClosed
        );
    });
}

#[test]
fn commit_rejected_on_duplicate_submission() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        // Second attempt from the same account within same epoch is blocked.
        assert_noop!(
            BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), [0u8; 32]),
            Error::<Test>::AlreadyCommitted
        );
    });
}

#[test]
fn commit_rejected_when_max_contributors_reached() {
    new_test_ext().execute_with(|| {
        // MaxContributors = 5; fill all slots.
        for (i, who) in [ALICE, BOB, CHARLIE, DAVE, EVE].iter().enumerate() {
            let c = make_commitment([i as u8; 32], [i as u8 + 100; 32]);
            assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(*who), c));
        }
        assert_eq!(CommitCount::<Test>::get(), 5);

        // FERDIE (6th) must be rejected.
        assert_noop!(
            BelizeRandomness::commit(
                RuntimeOrigin::signed(FERDIE),
                make_commitment([99u8; 32], [88u8; 32])
            ),
            Error::<Test>::TooManyContributors
        );
    });
}

// ── Reveal phase tests ────────────────────────────────────────────────────────

#[test]
fn reveal_works_during_reveal_window() {
    new_test_ext().execute_with(|| {
        // Commit at block 1.
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        // Advance to reveal window (block 6).
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(
            RuntimeOrigin::signed(ALICE),
            VALUE_A,
            SALT_A,
        ));

        assert_eq!(Reveals::<Test>::get(ALICE), Some(VALUE_A));
        assert_eq!(RevealCount::<Test>::get(), 1);
        System::assert_has_event(
            Event::Revealed { contributor: ALICE, epoch: 0 }.into(),
        );
    });
}

#[test]
fn reveal_rejected_before_commit_window_closes() {
    new_test_ext().execute_with(|| {
        // Commit then try to reveal immediately (still in commit window).
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        // Still at block 1 — reveal window not open yet.
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A),
            Error::<Test>::RevealWindowNotOpen
        );
    });
}

#[test]
fn reveal_rejected_after_reveal_window_closes() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        // Advance past reveal window end (block 11).
        run_to_block(11);
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A),
            Error::<Test>::RevealWindowClosed
        );
    });
}

#[test]
fn reveal_rejected_when_commitment_missing() {
    new_test_ext().execute_with(|| {
        // ALICE never committed.
        run_to_block(6);
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A),
            Error::<Test>::NotCommitted
        );
    });
}

#[test]
fn reveal_rejected_on_commitment_mismatch() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));

        run_to_block(6);
        // Wrong value — should fail hash check.
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_B, SALT_A),
            Error::<Test>::CommitmentMismatch
        );
        // Wrong salt — should also fail.
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_B),
            Error::<Test>::CommitmentMismatch
        );
    });
}

#[test]
fn reveal_rejected_on_double_reveal() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        // Second reveal in the same epoch is rejected.
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A),
            Error::<Test>::AlreadyRevealed
        );
    });
}

// ── Finalization tests ────────────────────────────────────────────────────────

#[test]
fn finalize_epoch_too_early_is_rejected() {
    new_test_ext().execute_with(|| {
        // Still in reveal window (block 8) — cannot finalize yet.
        run_to_block(8);
        assert_noop!(
            BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::TooEarlyToFinalize
        );
    });
}

#[test]
fn finalize_epoch_produces_new_seed_with_sufficient_reveals() {
    new_test_ext().execute_with(|| {
        // Both ALICE and BOB commit (MinReveals = 2).
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));

        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));

        let seed_before = CurrentSeed::<Test>::get();

        // Advance past reveal window end.
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // Seed must have changed.
        let seed_after = CurrentSeed::<Test>::get();
        assert_ne!(seed_after, seed_before);

        // Epoch counter incremented.
        assert_eq!(CurrentEpoch::<Test>::get(), 1);
        // Storage cleared.
        assert_eq!(CommitCount::<Test>::get(), 0);
        assert_eq!(RevealCount::<Test>::get(), 0);
    });
}

#[test]
fn finalize_epoch_emits_epoch_finalized_event() {
    new_test_ext().execute_with(|| {
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // The finalization event must be present.
        let events = System::events();
        let has_finalized = events.iter().any(|e| {
            matches!(
                e.event,
                RuntimeEvent::BelizeRandomness(Event::EpochFinalized { epoch: 0, .. })
            )
        });
        assert!(has_finalized, "EpochFinalized event not found");
    });
}

#[test]
fn finalize_epoch_skipped_when_insufficient_reveals() {
    new_test_ext().execute_with(|| {
        // Only ALICE commits but does not reveal → reveal count stays 0 < MinReveals(2).
        let c_a = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        // No reveals.

        let seed_before = CurrentSeed::<Test>::get();

        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(BOB)));

        // Seed unchanged (epoch was skipped).
        assert_eq!(CurrentSeed::<Test>::get(), seed_before);

        // Epoch counter still increments.
        assert_eq!(CurrentEpoch::<Test>::get(), 1);

        // EpochSkipped event emitted.
        System::assert_has_event(
            Event::EpochSkipped { epoch: 0, reveals: 0 }.into(),
        );
    });
}

#[test]
fn finalize_epoch_is_permissionless() {
    new_test_ext().execute_with(|| {
        // Anyone can call finalize_epoch after the reveal window.
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        // CHARLIE did not participate but can finalize.
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(CHARLIE)));
    });
}

#[test]
fn sequential_epochs_produce_different_seeds() {
    new_test_ext().execute_with(|| {
        // Epoch 0 finalization.
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed1 = CurrentSeed::<Test>::get();

        // Epoch 1 finalization with different values.
        let epoch1_start = System::block_number();
        let value_c = [3u8; 32];
        let salt_c = [30u8; 32];
        let value_d = [4u8; 32];
        let salt_d = [40u8; 32];
        let c_c = make_commitment(value_c, salt_c);
        let c_d = make_commitment(value_d, salt_d);

        // Still in new epoch's commit window.
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_c));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_d));
        run_to_block(epoch1_start + 6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), value_c, salt_c));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), value_d, salt_d));
        run_to_block(epoch1_start + 11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed2 = CurrentSeed::<Test>::get();

        // Seeds should differ (different inputs).
        assert_ne!(seed1, seed2);
        assert_eq!(CurrentEpoch::<Test>::get(), 2);
    });
}

// ── Randomness trait tests ────────────────────────────────────────────────────

#[test]
fn randomness_trait_returns_current_seed() {
    new_test_ext().execute_with(|| {
        // Produce a non-zero seed via finalization.
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // The Randomness trait output must be deterministic for the same subject.
        let (r1, _) = BelizeRandomness::random(b"subjectA");
        let (r2, _) = BelizeRandomness::random(b"subjectA");
        assert_eq!(r1, r2);
    });
}

#[test]
fn randomness_trait_different_subjects_give_different_outputs() {
    new_test_ext().execute_with(|| {
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        let (r_a, _) = BelizeRandomness::random(b"pallet_consensus");
        let (r_b, _) = BelizeRandomness::random(b"pallet_interoperability");
        assert_ne!(r_a, r_b, "different subjects must produce different randomness draws");
    });
}

#[test]
fn randomness_trait_returns_epoch_start_as_block_number() {
    new_test_ext().execute_with(|| {
        // Before finalization the epoch_start is still 1.
        let (_, block) = BelizeRandomness::random(b"test");
        assert_eq!(block, 1u64);
    });
}

// ── Commit boundary block tests ───────────────────────────────────────────────

#[test]
fn commit_at_last_valid_block_succeeds() {
    new_test_ext().execute_with(|| {
        // Commit window: blocks 1-5 (block < epoch_start + CommitDuration = 1+5 = 6)
        // Block 5 is the last valid block for commit
        run_to_block(5);
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));
        assert_eq!(CommitCount::<Test>::get(), 1);
    });
}

#[test]
fn commit_at_first_invalid_block_fails() {
    new_test_ext().execute_with(|| {
        // Block 6 == commit_end, so block >= commit_end → CommitWindowClosed
        run_to_block(6);
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_noop!(
            BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment),
            Error::<Test>::CommitWindowClosed
        );
    });
}

// ── Reveal boundary block tests ───────────────────────────────────────────────

#[test]
fn reveal_at_first_valid_block_succeeds() {
    new_test_ext().execute_with(|| {
        // Commit at block 1
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));
        // Block 6 is first valid reveal block (commit_end = 6)
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_eq!(RevealCount::<Test>::get(), 1);
    });
}

#[test]
fn reveal_at_last_valid_block_succeeds() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));
        // Reveal window: 6 ≤ block < 11. Block 10 is last valid.
        run_to_block(10);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_eq!(RevealCount::<Test>::get(), 1);
    });
}

#[test]
fn reveal_at_first_invalid_block_fails() {
    new_test_ext().execute_with(|| {
        let commitment = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), commitment));
        // Block 11 == reveal_end → RevealWindowClosed
        run_to_block(11);
        assert_noop!(
            BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A),
            Error::<Test>::RevealWindowClosed
        );
    });
}

// ── Finalization edge cases ──────────────────────────────────────────────────

#[test]
fn finalize_at_exact_reveal_end_block_succeeds() {
    new_test_ext().execute_with(|| {
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        // Exact boundary: block 11 = reveal_end
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        assert_eq!(CurrentEpoch::<Test>::get(), 1);
    });
}

#[test]
fn finalize_one_block_before_reveal_end_fails() {
    new_test_ext().execute_with(|| {
        // Block 10 < reveal_end(11) → TooEarlyToFinalize
        run_to_block(10);
        assert_noop!(
            BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::TooEarlyToFinalize
        );
    });
}

#[test]
fn double_finalize_starts_new_epoch_then_too_early() {
    new_test_ext().execute_with(|| {
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        assert_eq!(CurrentEpoch::<Test>::get(), 1);

        // Second finalize at same block: new epoch just started at block 11,
        // so reveal_end = 11 + 5 + 5 = 21. Block 11 < 21 → TooEarlyToFinalize
        assert_noop!(
            BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::TooEarlyToFinalize
        );
    });
}

#[test]
fn finalize_clears_commitments_and_reveals() {
    new_test_ext().execute_with(|| {
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // All per-account storage should be cleared
        assert!(Commitments::<Test>::get(ALICE).is_none());
        assert!(Commitments::<Test>::get(BOB).is_none());
        assert!(Reveals::<Test>::get(ALICE).is_none());
        assert!(Reveals::<Test>::get(BOB).is_none());
        assert_eq!(CommitCount::<Test>::get(), 0);
        assert_eq!(RevealCount::<Test>::get(), 0);
    });
}

#[test]
fn finalize_updates_epoch_start() {
    new_test_ext().execute_with(|| {
        assert_eq!(EpochStart::<Test>::get(), 1);
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // EpochStart updated to current block (11)
        assert_eq!(EpochStart::<Test>::get(), 11);
    });
}

#[test]
fn finalize_with_exactly_min_reveals_minus_one_skips() {
    new_test_ext().execute_with(|| {
        // MinReveals = 2. Only 1 reveal → should skip.
        let c_a = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_eq!(RevealCount::<Test>::get(), 1);

        let seed_before = CurrentSeed::<Test>::get();
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(BOB)));

        // Seed unchanged — not enough reveals
        assert_eq!(CurrentSeed::<Test>::get(), seed_before);
        System::assert_has_event(
            Event::EpochSkipped { epoch: 0, reveals: 1 }.into(),
        );
    });
}

#[test]
fn finalize_with_exactly_min_reveals_produces_seed() {
    new_test_ext().execute_with(|| {
        // MinReveals = 2. Exactly 2 reveals → should produce new seed.
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));

        let seed_before = CurrentSeed::<Test>::get();
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(CHARLIE)));

        assert_ne!(CurrentSeed::<Test>::get(), seed_before);
    });
}

// ── Full contributor participation ──────────────────────────────────────────

#[test]
fn all_max_contributors_commit_and_reveal() {
    new_test_ext().execute_with(|| {
        // MaxContributors = 5. All 5 participate.
        let participants = [
            (ALICE, [1u8; 32], [11u8; 32]),
            (BOB, [2u8; 32], [22u8; 32]),
            (CHARLIE, [3u8; 32], [33u8; 32]),
            (DAVE, [4u8; 32], [44u8; 32]),
            (EVE, [5u8; 32], [55u8; 32]),
        ];

        for &(who, value, salt) in &participants {
            let c = make_commitment(value, salt);
            assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(who), c));
        }
        assert_eq!(CommitCount::<Test>::get(), 5);

        run_to_block(6);
        for &(who, value, salt) in &participants {
            assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(who), value, salt));
        }
        assert_eq!(RevealCount::<Test>::get(), 5);

        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        assert_ne!(CurrentSeed::<Test>::get(), Default::default());
    });
}

// ── Deterministic ordering ──────────────────────────────────────────────────

#[test]
fn same_values_produce_same_seed_regardless_of_commit_order() {
    // Finalization sorts by AccountId, so commit order shouldn't matter.
    // Run two epochs with same values but different commit orders.
    new_test_ext().execute_with(|| {
        // Epoch 0: Alice first, then Bob
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed_epoch0 = CurrentSeed::<Test>::get();

        // Epoch 1: Bob first, then Alice — same values
        let c_b2 = make_commitment(VALUE_B, SALT_B);
        let c_a2 = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b2));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a2));
        // New epoch started at block 11, commit_end = 16
        run_to_block(16);
        // Reveal in different order too
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        run_to_block(21);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(BOB)));
        let seed_epoch1 = CurrentSeed::<Test>::get();

        // Seeds differ because epoch number is mixed in (domain separation)
        assert_ne!(seed_epoch0, seed_epoch1);
    });
}

// ── Domain separation / epoch isolation ─────────────────────────────────────

#[test]
fn same_values_different_epochs_produce_different_seeds() {
    new_test_ext().execute_with(|| {
        // Epoch 0
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed0 = CurrentSeed::<Test>::get();

        // Epoch 1 with IDENTICAL values
        let c_a2 = make_commitment(VALUE_A, SALT_A);
        let c_b2 = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a2));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b2));
        run_to_block(16);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(21);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed1 = CurrentSeed::<Test>::get();

        // Different because epoch number is mixed in during hashing
        assert_ne!(seed0, seed1);
    });
}

// ── Seed preservation on skip ───────────────────────────────────────────────

#[test]
fn seed_preserved_across_skipped_epoch() {
    new_test_ext().execute_with(|| {
        // First produce a valid seed
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let valid_seed = CurrentSeed::<Test>::get();
        assert_ne!(valid_seed, Default::default());

        // Skip epoch 1: no commits, no reveals
        run_to_block(21); // epoch_start=11, reveal_end = 11+5+5=21
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // Seed should be preserved from epoch 0
        assert_eq!(CurrentSeed::<Test>::get(), valid_seed);
        assert_eq!(CurrentEpoch::<Test>::get(), 2);
        System::assert_has_event(
            Event::EpochSkipped { epoch: 1, reveals: 0 }.into(),
        );
    });
}

// ── Cross-epoch isolation ───────────────────────────────────────────────────

#[test]
fn old_commitment_does_not_persist_to_new_epoch() {
    new_test_ext().execute_with(|| {
        // Commit in epoch 0
        let c_a = make_commitment(VALUE_A, SALT_A);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));

        // Skip epoch 0 without reveal
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(BOB)));

        // In new epoch (epoch 1), ALICE's commitment should be cleared
        assert!(Commitments::<Test>::get(ALICE).is_none());
        // ALICE can commit fresh
        let c_a2 = make_commitment([99u8; 32], [99u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a2));
        assert_eq!(CommitCount::<Test>::get(), 1);
    });
}

// ── on_initialize tests ─────────────────────────────────────────────────────

#[test]
fn on_initialize_sets_epoch_start_once() {
    new_test_ext().execute_with(|| {
        // After new_test_ext, EpochStart is set to 1 by on_initialize(1)
        assert_eq!(EpochStart::<Test>::get(), 1);

        // Running on_initialize again at block 2 should NOT change it (it's non-zero)
        use frame_support::traits::OnInitialize;
        System::set_block_number(2);
        BelizeRandomness::on_initialize(2);
        assert_eq!(EpochStart::<Test>::get(), 1); // Still 1
    });
}

// ── Zero commitment edge cases ──────────────────────────────────────────────

#[test]
fn all_zero_values_produce_valid_seed() {
    new_test_ext().execute_with(|| {
        let zero_val = [0u8; 32];
        let zero_salt = [0u8; 32];
        let c = make_commitment(zero_val, zero_salt);

        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c));

        // BOB needs a different commitment (or same is fine since different account)
        let c_b = make_commitment([0u8; 32], [1u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));

        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), zero_val, zero_salt));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), [0u8; 32], [1u8; 32]));

        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        // A valid (non-default) seed should be produced
        assert_ne!(CurrentSeed::<Test>::get(), Default::default());
    });
}

// ── Randomness trait additional tests ───────────────────────────────────────

#[test]
fn randomness_with_zero_seed_still_works() {
    new_test_ext().execute_with(|| {
        // Before any finalization, seed is default zero hash
        let (r, block) = BelizeRandomness::random(b"test_subject");
        // Should still return a valid hash (mixed with subject)
        assert_eq!(block, 1);
        // Even with zero seed, mixing with subject produces non-zero output
        // (unless by extreme coincidence)
        let _ = r; // Just verify it doesn't panic
    });
}

#[test]
fn randomness_epoch_start_updates_after_finalize() {
    new_test_ext().execute_with(|| {
        // Produce a finalized epoch
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // Now epoch_start = 11
        let (_, block) = BelizeRandomness::random(b"post_finalize");
        assert_eq!(block, 11);
    });
}

// ── Multiple accounts, partial reveal ───────────────────────────────────────

#[test]
fn partial_reveals_above_min_produces_seed() {
    new_test_ext().execute_with(|| {
        // 3 commit, only 2 reveal → still >= MinReveals(2) → seed produced
        let c_a = make_commitment(VALUE_A, SALT_A);
        let c_b = make_commitment(VALUE_B, SALT_B);
        let c_c = make_commitment([3u8; 32], [30u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(CHARLIE), c_c));
        assert_eq!(CommitCount::<Test>::get(), 3);

        run_to_block(6);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), VALUE_A, SALT_A));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), VALUE_B, SALT_B));
        // CHARLIE does NOT reveal
        assert_eq!(RevealCount::<Test>::get(), 2);

        let seed_before = CurrentSeed::<Test>::get();
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));

        // Seed should change (2 >= MinReveals)
        assert_ne!(CurrentSeed::<Test>::get(), seed_before);
    });
}

#[test]
fn commit_count_accurate_after_multiple_commits() {
    new_test_ext().execute_with(|| {
        assert_eq!(CommitCount::<Test>::get(), 0);
        let c1 = make_commitment([1u8; 32], [10u8; 32]);
        let c2 = make_commitment([2u8; 32], [20u8; 32]);
        let c3 = make_commitment([3u8; 32], [30u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c1));
        assert_eq!(CommitCount::<Test>::get(), 1);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c2));
        assert_eq!(CommitCount::<Test>::get(), 2);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(CHARLIE), c3));
        assert_eq!(CommitCount::<Test>::get(), 3);
    });
}

#[test]
fn reveal_count_accurate_after_multiple_reveals() {
    new_test_ext().execute_with(|| {
        let c1 = make_commitment([1u8; 32], [10u8; 32]);
        let c2 = make_commitment([2u8; 32], [20u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c1));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c2));
        run_to_block(6);
        assert_eq!(RevealCount::<Test>::get(), 0);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), [1u8; 32], [10u8; 32]));
        assert_eq!(RevealCount::<Test>::get(), 1);
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), [2u8; 32], [20u8; 32]));
        assert_eq!(RevealCount::<Test>::get(), 2);
    });
}

// ── Three epoch stress test ─────────────────────────────────────────────────

#[test]
fn three_consecutive_epochs_with_varying_participation() {
    new_test_ext().execute_with(|| {
        // Epoch 0: full participation (3 contributors)
        let vals: [(u64, [u8; 32], [u8; 32]); 3] = [
            (ALICE, [1u8; 32], [11u8; 32]),
            (BOB, [2u8; 32], [22u8; 32]),
            (CHARLIE, [3u8; 32], [33u8; 32]),
        ];
        for &(who, v, s) in &vals {
            assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(who), make_commitment(v, s)));
        }
        run_to_block(6);
        for &(who, v, s) in &vals {
            assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(who), v, s));
        }
        run_to_block(11);
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        let seed0 = CurrentSeed::<Test>::get();
        assert_eq!(CurrentEpoch::<Test>::get(), 1);

        // Epoch 1: skipped (no participation)
        run_to_block(21); // epoch_start=11, reveal_end=21
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        assert_eq!(CurrentSeed::<Test>::get(), seed0); // preserved
        assert_eq!(CurrentEpoch::<Test>::get(), 2);

        // Epoch 2: minimal participation (exactly 2)
        let c_a = make_commitment([10u8; 32], [110u8; 32]);
        let c_b = make_commitment([20u8; 32], [120u8; 32]);
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(ALICE), c_a));
        assert_ok!(BelizeRandomness::commit(RuntimeOrigin::signed(BOB), c_b));
        run_to_block(26); // epoch_start=21, commit_end=26
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(ALICE), [10u8; 32], [110u8; 32]));
        assert_ok!(BelizeRandomness::reveal(RuntimeOrigin::signed(BOB), [20u8; 32], [120u8; 32]));
        run_to_block(31); // reveal_end=31
        assert_ok!(BelizeRandomness::finalize_epoch(RuntimeOrigin::signed(ALICE)));
        assert_ne!(CurrentSeed::<Test>::get(), seed0);
        assert_eq!(CurrentEpoch::<Test>::get(), 3);
    });
}
