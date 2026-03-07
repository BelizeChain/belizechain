//! Comprehensive tests for the Whistleblower pallet
#![cfg(test)]

use crate::{mock::*, pallet::*};
use frame_support::{assert_noop, assert_ok};
use sp_io::hashing::blake2_256;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Compute the alias hash for a given account and nonce.
fn compute_alias(account: u64, nonce: [u8; 32]) -> [u8; 32] {
    let mut preimage = codec::Encode::encode(&account);
    preimage.extend_from_slice(&nonce);
    blake2_256(&preimage)
}

fn sample_evidence_hash() -> [u8; 32] {
    [0xAB; 32]
}

fn sample_reasoning_hash() -> [u8; 32] {
    [0xCD; 32]
}

// ============================================================================
// REPORT SUBMISSION TESTS
// ============================================================================

#[test]
fn submit_report_works() {
    new_test_ext().execute_with(|| {
        let nonce = [1u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias,
            TARGET,
            sample_evidence_hash(),
            0, // Fraud
        ));

        // Verify report counter incremented
        assert_eq!(Whistleblower::report_counter(), 1);

        // Verify report stored correctly
        let report = Whistleblower::reports(1).expect("report must exist");
        assert_eq!(report.alias_hash, alias);
        assert_eq!(report.target, TARGET);
        assert_eq!(report.evidence_hash, sample_evidence_hash());
        assert_eq!(report.category, ReportCategory::Fraud);
        assert_eq!(report.status, ReportStatus::Pending);
        assert_eq!(report.bond, 1_000); // ReportBond
        assert!(report.reasoning_hash.is_none());

        // Verify bond was reserved
        assert_eq!(Balances::reserved_balance(ALICE), 1_000);
    });
}

#[test]
fn submit_report_all_categories_work() {
    new_test_ext().execute_with(|| {
        let nonce = [1u8; 32];

        // Fraud (0)
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce),
            TARGET,
            sample_evidence_hash(),
            0,
        ));
        assert_eq!(Whistleblower::reports(1).unwrap().category, ReportCategory::Fraud);

        // SystematicAbuse (1)
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(BOB),
            compute_alias(BOB, nonce),
            TARGET,
            sample_evidence_hash(),
            1,
        ));
        assert_eq!(Whistleblower::reports(2).unwrap().category, ReportCategory::SystematicAbuse);

        // ChainExploit (2)
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(CHARLIE),
            compute_alias(CHARLIE, nonce),
            TARGET,
            sample_evidence_hash(),
            2,
        ));
        assert_eq!(Whistleblower::reports(3).unwrap().category, ReportCategory::ChainExploit);
    });
}

#[test]
fn submit_report_invalid_category_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                [0u8; 32],
                TARGET,
                sample_evidence_hash(),
                3, // invalid
            ),
            Error::<Test>::InvalidCategory
        );

        assert_noop!(
            Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                [0u8; 32],
                TARGET,
                sample_evidence_hash(),
                255, // invalid
            ),
            Error::<Test>::InvalidCategory
        );
    });
}

#[test]
fn submit_report_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        let poor_account = 999u64;
        // poor_account has no balance
        assert_noop!(
            Whistleblower::submit_report(
                RuntimeOrigin::signed(poor_account),
                [0u8; 32],
                TARGET,
                sample_evidence_hash(),
                0,
            ),
            Error::<Test>::InsufficientBondBalance
        );
    });
}

#[test]
fn submit_multiple_reports_increments_counter() {
    new_test_ext().execute_with(|| {
        for i in 0..3 {
            let nonce = [i as u8; 32];
            assert_ok!(Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                compute_alias(ALICE, nonce),
                TARGET,
                sample_evidence_hash(),
                0,
            ));
        }
        assert_eq!(Whistleblower::report_counter(), 3);
    });
}

// ============================================================================
// REVIEW REPORT TESTS
// ============================================================================

fn setup_funded_report(category: u8) -> u32 {
    let nonce = [1u8; 32];
    let alias = compute_alias(ALICE, nonce);

    // Fund pool first
    assert_ok!(Whistleblower::fund_whistleblower_pool(
        RuntimeOrigin::root(),
        100_000,
    ));

    assert_ok!(Whistleblower::submit_report(
        RuntimeOrigin::signed(ALICE),
        alias,
        TARGET,
        sample_evidence_hash(),
        category,
    ));

    Whistleblower::report_counter()
}

#[test]
fn review_report_verified_works() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0); // Fraud

        let pool_before = Whistleblower::whistleblower_pool();

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), // ReviewerOrigin
            report_id,
            0, // Verified
            sample_reasoning_hash(),
        ));

        let report = Whistleblower::reports(report_id).unwrap();
        assert_eq!(report.status, ReportStatus::Verified);
        assert_eq!(report.reasoning_hash, Some(sample_reasoning_hash()));

        // Verify reward escrowed from pool
        let escrow = Whistleblower::escrowed_reward(report_id).unwrap();
        assert_eq!(escrow, 10_000); // FraudReward
        assert_eq!(Whistleblower::whistleblower_pool(), pool_before - 10_000);
    });
}

#[test]
fn review_report_dismissed_works() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(),
            report_id,
            1, // Dismissed
            sample_reasoning_hash(),
        ));

        let report = Whistleblower::reports(report_id).unwrap();
        assert_eq!(report.status, ReportStatus::Dismissed);

        // No reward escrowed
        assert!(Whistleblower::escrowed_reward(report_id).is_none());
    });
}

#[test]
fn review_report_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(),
                999, // non-existent
                0,
                sample_reasoning_hash(),
            ),
            Error::<Test>::ReportNotFound
        );
    });
}

#[test]
fn review_report_already_verified_fails() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        // First review: verified
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(),
            report_id,
            0,
            sample_reasoning_hash(),
        ));

        // Second review must fail
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(),
                report_id,
                0,
                sample_reasoning_hash(),
            ),
            Error::<Test>::InvalidReportStatus
        );
    });
}

#[test]
fn review_report_invalid_verdict_fails() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(),
                report_id,
                5, // invalid verdict
                sample_reasoning_hash(),
            ),
            Error::<Test>::InvalidReportStatus
        );
    });
}

#[test]
fn review_report_insufficient_pool_fails() {
    new_test_ext().execute_with(|| {
        // Submit report WITHOUT funding pool
        let nonce = [1u8; 32];
        let alias = compute_alias(ALICE, nonce);
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias,
            TARGET,
            sample_evidence_hash(),
            0,
        ));
        let report_id = Whistleblower::report_counter();

        // Pool has 0 balance, trying to verify should fail
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(),
                report_id,
                0, // Verified
                sample_reasoning_hash(),
            ),
            Error::<Test>::InsufficientPool
        );
    });
}

#[test]
fn review_report_verified_each_category_reward() {
    new_test_ext().execute_with(|| {
        // Fund pool generously
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(),
            1_000_000,
        ));

        // Fraud report
        let nonce = [1u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce),
            TARGET,
            sample_evidence_hash(),
            0, // Fraud
        ));
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 1, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::escrowed_reward(1).unwrap(), 10_000); // FraudReward

        // SystematicAbuse report
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(BOB),
            compute_alias(BOB, nonce),
            TARGET,
            sample_evidence_hash(),
            1, // Abuse
        ));
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 2, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::escrowed_reward(2).unwrap(), 5_000); // AbuseReward

        // ChainExploit report
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(CHARLIE),
            compute_alias(CHARLIE, nonce),
            TARGET,
            sample_evidence_hash(),
            2, // Exploit
        ));
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 3, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::escrowed_reward(3).unwrap(), 50_000); // ExploitReward
    });
}

// ============================================================================
// CLAIM REWARD TESTS
// ============================================================================

fn setup_verified_report() -> (u32, [u8; 32]) {
    let nonce = [42u8; 32];
    let alias = compute_alias(ALICE, nonce);

    assert_ok!(Whistleblower::fund_whistleblower_pool(
        RuntimeOrigin::root(),
        100_000,
    ));

    assert_ok!(Whistleblower::submit_report(
        RuntimeOrigin::signed(ALICE),
        alias,
        TARGET,
        sample_evidence_hash(),
        0, // Fraud
    ));

    let report_id = Whistleblower::report_counter();

    assert_ok!(Whistleblower::review_report(
        RuntimeOrigin::root(),
        report_id,
        0, // Verified
        sample_reasoning_hash(),
    ));

    (report_id, nonce)
}

#[test]
fn claim_reward_works() {
    new_test_ext().execute_with(|| {
        let (report_id, nonce) = setup_verified_report();
        let balance_before = Balances::free_balance(ALICE);

        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(ALICE),
            report_id,
            nonce,
        ));

        // Reward paid out
        let balance_after = Balances::free_balance(ALICE);
        assert_eq!(balance_after, balance_before + 10_000); // FraudReward

        // Escrow cleared
        assert!(Whistleblower::escrowed_reward(report_id).is_none());
    });
}

#[test]
fn claim_reward_wrong_nonce_fails() {
    new_test_ext().execute_with(|| {
        let (report_id, _nonce) = setup_verified_report();

        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(ALICE),
                report_id,
                [99u8; 32], // wrong nonce
            ),
            Error::<Test>::AliasHashMismatch
        );
    });
}

#[test]
fn claim_reward_wrong_account_fails() {
    new_test_ext().execute_with(|| {
        let (report_id, nonce) = setup_verified_report();

        // BOB tries to claim with ALICE's nonce — hash won't match
        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(BOB),
                report_id,
                nonce,
            ),
            Error::<Test>::AliasHashMismatch
        );
    });
}

#[test]
fn claim_reward_not_verified_fails() {
    new_test_ext().execute_with(|| {
        // Submit but don't verify
        let nonce = [1u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias,
            TARGET,
            sample_evidence_hash(),
            0,
        ));

        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(ALICE),
                1,
                nonce,
            ),
            Error::<Test>::InvalidReportStatus
        );
    });
}

#[test]
fn claim_reward_report_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(ALICE),
                999,
                [0u8; 32],
            ),
            Error::<Test>::ReportNotFound
        );
    });
}

#[test]
fn claim_reward_double_claim_fails() {
    new_test_ext().execute_with(|| {
        let (report_id, nonce) = setup_verified_report();

        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(ALICE),
            report_id,
            nonce,
        ));

        // Second claim — escrow already taken
        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(ALICE),
                report_id,
                nonce,
            ),
            Error::<Test>::NoEscrowedReward
        );
    });
}

// ============================================================================
// FUND POOL TESTS
// ============================================================================

#[test]
fn fund_whistleblower_pool_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Whistleblower::whistleblower_pool(), 0);

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(),
            50_000,
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 50_000);

        // Fund again — cumulative
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(),
            25_000,
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 75_000);
    });
}

// ============================================================================
// REPORT CATEGORY CONVERSION TESTS
// ============================================================================

#[test]
fn report_category_from_u8_covers_all() {
    assert_eq!(ReportCategory::from_u8(0), Some(ReportCategory::Fraud));
    assert_eq!(ReportCategory::from_u8(1), Some(ReportCategory::SystematicAbuse));
    assert_eq!(ReportCategory::from_u8(2), Some(ReportCategory::ChainExploit));
    assert_eq!(ReportCategory::from_u8(3), None);
    assert_eq!(ReportCategory::from_u8(255), None);
}

// ============================================================================
// EVENT EMISSION TESTS
// ============================================================================

#[test]
fn submit_report_emits_event() {
    new_test_ext().execute_with(|| {
        let nonce = [1u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias,
            TARGET,
            sample_evidence_hash(),
            0,
        ));

        System::assert_last_event(
            Event::<Test>::ReportSubmitted {
                report_id: 1,
                target: TARGET,
                category: 0,
            }
            .into(),
        );
    });
}

#[test]
fn review_report_emits_event() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(),
            report_id,
            0,
            sample_reasoning_hash(),
        ));

        System::assert_has_event(
            Event::<Test>::ReportReviewed {
                report_id,
                verdict: 0,
            }
            .into(),
        );
    });
}

#[test]
fn claim_reward_emits_event() {
    new_test_ext().execute_with(|| {
        let (report_id, nonce) = setup_verified_report();

        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(ALICE),
            report_id,
            nonce,
        ));

        System::assert_has_event(
            Event::<Test>::RewardClaimed {
                report_id,
                claimant: ALICE,
                amount: 10_000,
            }
            .into(),
        );
    });
}

#[test]
fn fund_pool_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(),
            50_000,
        ));

        System::assert_last_event(
            Event::<Test>::PoolFunded {
                amount: 50_000,
                new_total: 50_000,
            }
            .into(),
        );
    });
}

// ============================================================================
// EXPANDED COVERAGE TESTS
// ============================================================================

// ── Origin guard tests ────────────────────────────────────────────────────

#[test]
fn review_report_fails_with_signed_origin() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        // Non-root signed origin should fail
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::signed(BOB),
                report_id,
                0,
                sample_reasoning_hash(),
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn fund_pool_signed_origin_increments_without_transfer() {
    // BUG/VULN: fund_whistleblower_pool accepts signed origins and increments
    // the pool counter WITHOUT actually reserving any balance. This means any
    // user can inflate the pool counter, leading to phantom rewards.
    // See lib.rs fund_whistleblower_pool: GovernanceOrigin check + ensure_signed
    // are combined with and_then — both root and signed fall into the else branch.
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(ALICE);

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::signed(ALICE),
            50_000,
        ));

        // Pool counter incremented but no balance was actually transferred
        assert_eq!(Whistleblower::whistleblower_pool(), 50_000);
        assert_eq!(Balances::free_balance(ALICE), balance_before); // unchanged!
        assert_eq!(Balances::reserved_balance(ALICE), 0); // nothing reserved
    });
}

// ── Review on already-dismissed report ────────────────────────────────────

#[test]
fn review_report_already_dismissed_fails() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        // Dismiss first
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(),
            report_id,
            1, // Dismissed
            sample_reasoning_hash(),
        ));

        // Try to review again — Dismissed is not Pending|UnderReview
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(),
                report_id,
                0,
                sample_reasoning_hash(),
            ),
            Error::<Test>::InvalidReportStatus
        );
    });
}

// ── Claim on dismissed report ─────────────────────────────────────────────

#[test]
fn claim_reward_on_dismissed_report_fails() {
    new_test_ext().execute_with(|| {
        let nonce = [42u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 100_000,
        ));

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias,
            TARGET,
            sample_evidence_hash(),
            0,
        ));
        let report_id = Whistleblower::report_counter();

        // Dismiss
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 1, sample_reasoning_hash(),
        ));

        // Attempt claim — Dismissed status
        assert_noop!(
            Whistleblower::claim_reward(
                RuntimeOrigin::signed(ALICE),
                report_id,
                nonce,
            ),
            Error::<Test>::InvalidReportStatus
        );
    });
}

// ── Claim rewards for all categories (Abuse and Exploit) ──────────────────

#[test]
fn claim_reward_systematic_abuse_correct_amount() {
    new_test_ext().execute_with(|| {
        let nonce = [10u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 100_000,
        ));

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias, TARGET, sample_evidence_hash(), 1, // SystematicAbuse
        ));
        let report_id = Whistleblower::report_counter();

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 0, sample_reasoning_hash(),
        ));

        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(ALICE), report_id, nonce,
        ));

        assert_eq!(Balances::free_balance(ALICE), balance_before + 5_000); // AbuseReward
    });
}

#[test]
fn claim_reward_chain_exploit_correct_amount() {
    new_test_ext().execute_with(|| {
        let nonce = [20u8; 32];
        let alias = compute_alias(ALICE, nonce);

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 100_000,
        ));

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias, TARGET, sample_evidence_hash(), 2, // ChainExploit
        ));
        let report_id = Whistleblower::report_counter();

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 0, sample_reasoning_hash(),
        ));

        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(ALICE), report_id, nonce,
        ));

        assert_eq!(Balances::free_balance(ALICE), balance_before + 50_000); // ExploitReward
    });
}

// ── Pool balance tracking ─────────────────────────────────────────────────

#[test]
fn pool_depletes_across_multiple_verified_reports() {
    new_test_ext().execute_with(|| {
        // Fund pool with exactly enough for 2 fraud rewards (10K each)
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 20_000,
        ));

        // Report 1 — Fraud (costs 10K from pool)
        let nonce1 = [1u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce1), TARGET, sample_evidence_hash(), 0,
        ));
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 1, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 10_000);

        // Report 2 — Fraud (costs 10K from pool)
        let nonce2 = [2u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(BOB),
            compute_alias(BOB, nonce2), TARGET, sample_evidence_hash(), 0,
        ));
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 2, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 0);

        // Report 3 — Fraud, but pool is empty → InsufficientPool
        let nonce3 = [3u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(CHARLIE),
            compute_alias(CHARLIE, nonce3), TARGET, sample_evidence_hash(), 0,
        ));
        assert_noop!(
            Whistleblower::review_report(
                RuntimeOrigin::root(), 3, 0, sample_reasoning_hash(),
            ),
            Error::<Test>::InsufficientPool
        );
    });
}

#[test]
fn pool_exactly_matches_reward_drains_to_zero() {
    new_test_ext().execute_with(|| {
        // Fund exactly the Fraud reward amount
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 10_000, // = FraudReward
        ));

        let nonce = [1u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce), TARGET, sample_evidence_hash(), 0,
        ));

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), 1, 0, sample_reasoning_hash(),
        ));

        assert_eq!(Whistleblower::whistleblower_pool(), 0);
        assert_eq!(Whistleblower::escrowed_reward(1).unwrap(), 10_000);
    });
}

// ── Bond reservation edge cases ──────────────────────────────────────────

#[test]
fn multiple_reports_reserve_cumulative_bonds() {
    new_test_ext().execute_with(|| {
        // ALICE has 1_000_000. Each report reserves 1_000 bond.
        for i in 0..5u8 {
            let nonce = [i; 32];
            assert_ok!(Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                compute_alias(ALICE, nonce),
                TARGET,
                sample_evidence_hash(),
                0,
            ));
        }
        assert_eq!(Balances::reserved_balance(ALICE), 5_000); // 5 × 1_000
        assert_eq!(Whistleblower::report_counter(), 5);
    });
}

#[test]
fn bond_exhaustion_prevents_further_reports() {
    new_test_ext().execute_with(|| {
        // ALICE balance = 1_000_000, bond = 1_000
        // Can submit 999 reports (need 1 ED remaining), or until balance insufficient.
        // After 999 reports: 999_000 reserved, 1_000 free, ED=1 → can submit one more?
        // Actually: reserve checks if free_balance - ED >= bond amount.
        // At 1000 reports: 1_000_000 reserved → free = 0, but ED=1 means last bond fails.

        // Submit reports until bond cannot be reserved
        let mut count = 0u32;
        loop {
            let nonce = count.to_le_bytes();
            let mut nonce_32 = [0u8; 32];
            nonce_32[..4].copy_from_slice(&nonce);

            let result = Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                compute_alias(ALICE, nonce_32),
                TARGET,
                sample_evidence_hash(),
                0,
            );

            if result.is_err() {
                break;
            }
            count += 1;
            // Safety cap
            if count > 1500 {
                break;
            }
        }

        // ALICE had 1M, bond=1000 → can submit ~999 reports before running out
        assert!(count >= 999);
        assert!(count <= 1000);
    });
}

// ── Report metadata ──────────────────────────────────────────────────────

#[test]
fn report_submitted_at_block_correct() {
    new_test_ext().execute_with(|| {
        System::set_block_number(42);

        let nonce = [1u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce),
            TARGET,
            sample_evidence_hash(),
            0,
        ));

        let report = Whistleblower::reports(1).unwrap();
        assert_eq!(report.submitted_at, 42);
    });
}

#[test]
fn report_stores_correct_evidence_hash() {
    new_test_ext().execute_with(|| {
        let custom_evidence = [0xDE; 32];
        let nonce = [1u8; 32];

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce),
            TARGET,
            custom_evidence,
            1,
        ));

        let report = Whistleblower::reports(1).unwrap();
        assert_eq!(report.evidence_hash, custom_evidence);
        assert_eq!(report.category, ReportCategory::SystematicAbuse);
    });
}

// ── Self-reporting (reporter = target) ───────────────────────────────────

#[test]
fn report_against_self_allowed() {
    // Protocol doesn't prevent self-reports — they'd just be dismissed
    new_test_ext().execute_with(|| {
        let nonce = [1u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce),
            ALICE, // self-report
            sample_evidence_hash(),
            0,
        ));

        let report = Whistleblower::reports(1).unwrap();
        assert_eq!(report.target, ALICE);
    });
}

// ── Multiple reports against same target ─────────────────────────────────

#[test]
fn multiple_reports_same_target_allowed() {
    new_test_ext().execute_with(|| {
        for i in 0..3u8 {
            let nonce = [i; 32];
            assert_ok!(Whistleblower::submit_report(
                RuntimeOrigin::signed(ALICE),
                compute_alias(ALICE, nonce),
                TARGET,
                sample_evidence_hash(),
                i % 3, // rotate categories
            ));
        }
        assert_eq!(Whistleblower::report_counter(), 3);

        // All reports target the same account
        for id in 1..=3u32 {
            assert_eq!(Whistleblower::reports(id).unwrap().target, TARGET);
        }
    });
}

// ── Alias uniqueness ─────────────────────────────────────────────────────

#[test]
fn different_nonces_produce_different_aliases() {
    new_test_ext().execute_with(|| {
        let nonce1 = [1u8; 32];
        let nonce2 = [2u8; 32];
        let alias1 = compute_alias(ALICE, nonce1);
        let alias2 = compute_alias(ALICE, nonce2);

        assert_ne!(alias1, alias2);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias1, TARGET, sample_evidence_hash(), 0,
        ));
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            alias2, TARGET, sample_evidence_hash(), 0,
        ));

        let r1 = Whistleblower::reports(1).unwrap();
        let r2 = Whistleblower::reports(2).unwrap();
        assert_ne!(r1.alias_hash, r2.alias_hash);
    });
}

#[test]
fn same_nonce_different_accounts_produce_different_aliases() {
    let nonce = [1u8; 32];
    let alias_alice = compute_alias(ALICE, nonce);
    let alias_bob = compute_alias(BOB, nonce);
    assert_ne!(alias_alice, alias_bob);
}

// ── Fund pool cumulative events ──────────────────────────────────────────

#[test]
fn fund_pool_cumulative_events_have_correct_totals() {
    new_test_ext().execute_with(|| {
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 30_000,
        ));
        System::assert_has_event(
            Event::<Test>::PoolFunded { amount: 30_000, new_total: 30_000 }.into()
        );

        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 20_000,
        ));
        System::assert_has_event(
            Event::<Test>::PoolFunded { amount: 20_000, new_total: 50_000 }.into()
        );
    });
}

// ── Full lifecycle tests ──────────────────────────────────────────────────

#[test]
fn full_lifecycle_submit_verify_claim() {
    new_test_ext().execute_with(|| {
        // 1. Fund the pool
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 100_000,
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 100_000);

        // 2. Submit report
        let nonce = [77u8; 32];
        let alias = compute_alias(BOB, nonce);
        let balance_before = Balances::free_balance(BOB);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(BOB),
            alias, TARGET, sample_evidence_hash(), 2, // ChainExploit
        ));
        let report_id = Whistleblower::report_counter();
        assert_eq!(Balances::reserved_balance(BOB), 1_000); // bond

        // 3. Verify report
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 0, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), 50_000); // 100K - 50K exploit reward

        // 4. Claim reward
        assert_ok!(Whistleblower::claim_reward(
            RuntimeOrigin::signed(BOB), report_id, nonce,
        ));
        assert_eq!(Balances::free_balance(BOB), balance_before - 1_000 + 50_000);
        // Note: bond remains reserved (not auto-unreserved on claim)
        // The reward (50K) is deposited via deposit_creating

        assert!(Whistleblower::escrowed_reward(report_id).is_none());
    });
}

#[test]
fn full_lifecycle_submit_dismiss() {
    new_test_ext().execute_with(|| {
        assert_ok!(Whistleblower::fund_whistleblower_pool(
            RuntimeOrigin::root(), 100_000,
        ));
        let pool_after_fund = Whistleblower::whistleblower_pool();

        let nonce = [55u8; 32];
        let alias = compute_alias(CHARLIE, nonce);

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(CHARLIE),
            alias, TARGET, sample_evidence_hash(), 0,
        ));
        let report_id = Whistleblower::report_counter();

        // Dismiss — pool should NOT be reduced
        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 1, sample_reasoning_hash(),
        ));
        assert_eq!(Whistleblower::whistleblower_pool(), pool_after_fund);

        // No escrow created
        assert!(Whistleblower::escrowed_reward(report_id).is_none());

        let report = Whistleblower::reports(report_id).unwrap();
        assert_eq!(report.status, ReportStatus::Dismissed);
        assert_eq!(report.reasoning_hash, Some(sample_reasoning_hash()));
    });
}

// ── Review verdict event values ──────────────────────────────────────────

#[test]
fn review_dismissed_emits_verdict_1() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 1, sample_reasoning_hash(),
        ));

        System::assert_has_event(
            Event::<Test>::ReportReviewed { report_id, verdict: 1 }.into()
        );
    });
}

#[test]
fn review_verified_emits_verdict_0() {
    new_test_ext().execute_with(|| {
        let report_id = setup_funded_report(0);

        assert_ok!(Whistleblower::review_report(
            RuntimeOrigin::root(), report_id, 0, sample_reasoning_hash(),
        ));

        System::assert_has_event(
            Event::<Test>::ReportReviewed { report_id, verdict: 0 }.into()
        );
    });
}

// ── Submit report emits correct category values ──────────────────────────

#[test]
fn submit_report_event_category_values() {
    new_test_ext().execute_with(|| {
        let nonce = [1u8; 32];

        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(ALICE),
            compute_alias(ALICE, nonce), TARGET, sample_evidence_hash(), 1,
        ));
        System::assert_has_event(
            Event::<Test>::ReportSubmitted { report_id: 1, target: TARGET, category: 1 }.into()
        );

        let nonce2 = [2u8; 32];
        assert_ok!(Whistleblower::submit_report(
            RuntimeOrigin::signed(BOB),
            compute_alias(BOB, nonce2), TARGET, sample_evidence_hash(), 2,
        ));
        System::assert_has_event(
            Event::<Test>::ReportSubmitted { report_id: 2, target: TARGET, category: 2 }.into()
        );
    });
}
