use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// AI Model Registration Tests
// ============================================================================

#[test]
fn register_ai_model_works() {
    new_test_ext().execute_with(|| {
        // Register Economic model
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0, // Economic
            test_parameters_hash(1),
            1000, // Training data size
            test_pq_signature(4627),
        ));

        // Verify model was created
        let model = Consensus::ai_models(0).unwrap();
        assert_eq!(model.model_id, 0);
        assert_eq!(model.trainer, ALICE);
        assert_eq!(model.model_type, ModelType::Economic);
        assert_eq!(model.training_data_size, 1000);
        assert!(!model.active); // Requires validation
        assert_eq!(model.accuracy_score, 0);

        // Verify NextModelId incremented
        assert_eq!(Consensus::next_model_id(), 1);

        // Verify model added to trainer's list
        let models = Consensus::model_by_account(ALICE);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], 0);

        // Verify global metrics updated
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.total_models, 1);
    });
}

#[test]
fn register_ai_model_all_types_works() {
    new_test_ext().execute_with(|| {
        // Test all 8 model types
        let types = [
            (0, ModelType::Economic),
            (1, ModelType::Tourism),
            (2, ModelType::Agriculture),
            (3, ModelType::Climate),
            (4, ModelType::Health),
            (5, ModelType::Education),
            (6, ModelType::Transportation),
            (7, ModelType::General),
        ];

        for (i, (type_index, expected_type)) in types.iter().enumerate() {
            assert_ok!(Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                *type_index,
                test_parameters_hash(i as u8),
                1000,
                test_pq_signature(4627),
            ));

            let model = Consensus::ai_models(i as u32).unwrap();
            assert_eq!(model.model_type, *expected_type);
        }

        assert_eq!(Consensus::next_model_id(), 8);
    });
}

#[test]
fn register_ai_model_rejects_invalid_type() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                99, // Invalid type
                test_parameters_hash(1),
                1000,
                test_pq_signature(4627),
            ),
            Error::<Test>::InvalidModelType
        );
    });
}

#[test]
fn register_ai_model_rejects_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Signature wrong length (ML-DSA-87 requires exactly 4627 bytes)
        assert_noop!(
            Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                0,
                test_parameters_hash(1),
                1000,
                test_pq_signature(6000),
            ),
            Error::<Test>::PqVerificationFailed
        );
    });
}

// ============================================================================
// Validator Registration Tests
// ============================================================================

#[test]
fn join_consensus_validator_works() {
    new_test_ext().execute_with(|| {
        let stake_amount = 5_000_000;

        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            stake_amount,
            test_pq_signature(128),
        ));

        // Verify validator was created
        let validator_id = Consensus::validator_by_account(ALICE).unwrap();
        let validator = Consensus::consensus_validators(validator_id).unwrap();
        assert_eq!(validator.validator, ALICE);
        assert_eq!(validator.stake, stake_amount);
        assert!(validator.active);
        assert_eq!(validator.participation_score, 0);
        assert_eq!(validator.quality_score, 85); // From MockStakingProvider
        assert_eq!(validator.reputation, 90); // From MockStakingProvider;

        // Verify NextValidatorId incremented
        assert_eq!(Consensus::next_validator_id(), 1);

        // Verify global metrics updated
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.active_validators, 1);

        // Verify stake is locked
        assert_eq!(Balances::free_balance(ALICE), 100_000_000); // Unchanged
                                                                // Note: Locks don't reduce free balance, only usable balance
    });
}

#[test]
fn join_consensus_validator_requires_minimum_stake() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(ALICE),
                500_000, // Less than MinConsensusStake (1_000_000)
                test_pq_signature(128),
            ),
            Error::<Test>::InsufficientStake
        );
    });
}

#[test]
fn join_consensus_validator_prevents_duplicate() {
    new_test_ext().execute_with(|| {
        // First registration succeeds
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));

        // Second registration fails
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(ALICE),
                5_000_000,
                test_pq_signature(128),
            ),
            Error::<Test>::ValidatorAlreadyActive
        );
    });
}

#[test]
fn join_consensus_validator_rejects_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Public key too long (> 3000 bytes, ML-DSA-87 BoundedVec limit)
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(ALICE),
                5_000_000,
                test_pq_signature(4000), // Exceeds 3000-byte limit
            ),
            Error::<Test>::InvalidPQSignature
        );
    });
}

// ============================================================================
// AI Model Validation Tests
// ============================================================================

#[test]
fn validate_ai_model_works() {
    new_test_ext().execute_with(|| {
        // Register model first
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));

        // Validate model (AI Authority only)
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,    // model_id
            8500, // 85% accuracy
        ));

        // Verify model is now active and validated
        let model = Consensus::ai_models(0).unwrap();
        assert!(model.active);
        assert_eq!(model.accuracy_score, 8500);
        assert!(model.validated_at.is_some());

        // Verify global metrics
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.total_models, 1);
        assert_eq!(metrics.rounds_completed, 0);
    });
}

#[test]
fn validate_ai_model_requires_authority() {
    new_test_ext().execute_with(|| {
        // Register model
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));

        // Regular user cannot validate
        assert_noop!(
            Consensus::validate_ai_model(RuntimeOrigin::signed(ALICE), 0, 8500,),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn validate_ai_model_fails_for_nonexistent() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Consensus::validate_ai_model(
                RuntimeOrigin::root(),
                999, // Doesn't exist
                8500,
            ),
            Error::<Test>::ModelNotFound
        );
    });
}

// ============================================================================
// Consensus Round Tests
// ============================================================================

#[test]
fn start_consensus_round_works() {
    new_test_ext().execute_with(|| {
        // Register 3 validators
        for account in [ALICE, BOB, CHARLIE].iter() {
            assert_ok!(Consensus::join_consensus_validator(
                RuntimeOrigin::signed(*account),
                5_000_000,
                test_pq_signature(128),
            ));
        }

        // Start round (AI Authority only)
        assert_ok!(Consensus::start_consensus_round(
            RuntimeOrigin::root(),
            10, // Duration: 10 blocks
        ));

        // Verify round was created
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        assert_eq!(round.round_id, 0);
        assert_eq!(round.start_block, 1);
        assert_eq!(round.duration, 10);
        assert_eq!(round.status, RoundStatus::InProgress);
        assert_eq!(round.validators.len(), 3);

        // Verify NextRoundId incremented
        assert_eq!(Consensus::next_round_id(), 1);

        // Verify global metrics
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.rounds_completed, 0);
        assert_eq!(metrics.active_validators, 3);
    });
}

#[test]
fn start_consensus_round_requires_authority() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Consensus::start_consensus_round(RuntimeOrigin::signed(ALICE), 10,),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// AI Work Submission Tests
// ============================================================================

#[test]
fn submit_ai_work_works() {
    new_test_ext().execute_with(|| {
        // Setup: Register validator
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));

        // Register and validate model
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500,));

        // Start consensus round
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10,));

        // Submit AI work
        let work_hash = [1u8; 32];
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0, // model_id
            0, // work_type: ModelTraining
            work_hash,
            500, // computation_time (milliseconds)
            test_pq_signature(128),
        ));

        // Verify submission was recorded in round
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        assert_eq!(round.ai_work_submissions.len(), 1);
        assert_eq!(round.ai_work_submissions[0].model_id, 0);

        // Verify model's consensus rounds incremented
        let model = Consensus::ai_models(0).unwrap();
        assert_eq!(model.consensus_rounds, 1);
    });
}

#[test]
fn submit_ai_work_requires_active_round() {
    new_test_ext().execute_with(|| {
        // Setup validator and model (no round started)
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500,));

        // Try to submit without active round
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [1u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::RoundNotInProgress
        );
    });
}

#[test]
fn submit_ai_work_requires_active_model() {
    new_test_ext().execute_with(|| {
        // Setup validator and round
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10,));

        // Register model but DON'T validate it
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));

        // Try to submit with inactive model
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [1u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::QualityBelowThreshold // Code returns this error for inactive models
        );
    });
}

#[test]
fn submit_ai_work_requires_validator() {
    new_test_ext().execute_with(|| {
        // Setup model and round (but NO validator registration)
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500,));

        // Register a different validator to start round
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10,));

        // ALICE tries to submit without being a validator
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [1u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::ValidatorNotFound
        );
    });
}

#[test]
fn submit_ai_work_rejects_invalid_work_type() {
    new_test_ext().execute_with(|| {
        // Full setup
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500,));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10,));

        // Invalid work type index
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                99, // Invalid work type
                [1u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::InvalidWorkType
        );
    });
}

// ============================================================================
// Helper Function Tests
// ============================================================================

#[test]
fn account_id_generation_works() {
    new_test_ext().execute_with(|| {
        let account_id = Consensus::account_id();
        // Should be deterministic based on PalletId
        assert!(account_id != 0);
    });
}

#[test]
fn sync_validator_reputation_works() {
    new_test_ext().execute_with(|| {
        // Register validator
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));

        // Sync reputation (updates Staking provider, not local storage)
        let result = Consensus::sync_validator_reputation(&ALICE, 85);
        assert!(result);

        // Validator's reputation field stays as initialized (90 from MockStakingProvider)
        // The sync_validator_reputation only updates the external Staking pallet
        let validator_id = Consensus::validator_by_account(ALICE).unwrap();
        let validator = Consensus::consensus_validators(validator_id).unwrap();
        assert_eq!(validator.reputation, 90); // Still the initial value from provider
    });
}

#[test]
fn get_validator_contribution_score_works() {
    new_test_ext().execute_with(|| {
        // Register validator
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));

        // Calculate contribution score from MockStakingProvider:
        // ALICE: quality=85, reputation=90, stake=10_000_000
        // blended_quality = (reputation*60 + quality*40) / 100
        //                 = (90*60 + 85*40) / 100 = (5400+3400)/100 = 88
        // sustainability_score starts at 50, eligible_rounds=0 → uptime fallback=50
        // score = (blended_quality*50 + stake*20 + sustainability*20 + uptime*10) / 100
        //       = (88*50 + 10*20 + 50*20 + 50*10) / 100
        //       = (4400 + 200 + 1000 + 500) / 100 = 61
        let score = Consensus::get_validator_contribution_score(&ALICE);
        assert_eq!(score, 61);

        // After registering model, scores stay same (calculated from Staking provider)
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));

        // Score stays same (calculated from Staking provider, not Consensus pallet state)
        let score2 = Consensus::get_validator_contribution_score(&ALICE);
        assert_eq!(score2, 61);

        // Verify model was registered and tracked in ModelsByAccount
        let models = Consensus::model_by_account(ALICE);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], 0);
    });
}

// ============================================================================
// RATE LIMITING TESTS (AR-15)
// ============================================================================

/// Full setup helper: register + validate a model, then start a consensus round.
fn setup_consensus_for_rate_limit() {
    assert_ok!(Consensus::join_consensus_validator(
        RuntimeOrigin::signed(ALICE),
        5_000_000,
        test_pq_signature(128),
    ));
    assert_ok!(Consensus::register_ai_model(
        RuntimeOrigin::signed(ALICE),
        0,
        test_parameters_hash(1),
        1000,
        test_pq_signature(4627),
    ));
    assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));
    assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));
}

#[test]
fn submit_ai_work_rate_limit_blocks_after_max_per_block() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();

        // MaxSubmitPerBlock = 5 in the mock; exhaust the limit.
        for i in 0..5u8 {
            assert_ok!(Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0, // ModelTraining
                [i; 32],
                500,
                test_pq_signature(128),
            ));
        }

        // Sixth submission in the same block must be rejected.
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [99u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::RateLimitExceeded
        );
    });
}

#[test]
fn submit_ai_work_rate_limit_resets_on_next_block() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();

        // Exhaust the limit in block 1.
        for i in 0..5u8 {
            assert_ok!(Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [i; 32],
                500,
                test_pq_signature(128),
            ));
        }

        // Move to block 2 — counter must reset.
        System::set_block_number(2);

        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [55u8; 32],
            500,
            test_pq_signature(128),
        ));
    });
}

// ============================================================================
// Consensus Round Finalization Tests
// ============================================================================

#[test]
fn finalize_consensus_round_works() {
    new_test_ext().execute_with(|| {
        // Setup: Register validator, model, validate it
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0, // Economic model type
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,
            8500, // quality = 85%
        ));

        // Start round with duration = 5 blocks (from block 1)
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        let round_id = Consensus::current_consensus_round().unwrap();

        // Submit AI work
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [42u8; 32],
            500,
            test_pq_signature(128),
        ));

        // Attempting to finalize before round end should fail
        assert_noop!(
            Consensus::finalize_consensus_round(RuntimeOrigin::root()),
            Error::<Test>::RoundNotInProgress
        );

        // Advance past round end: start=1, duration=5 → end=6; go to block 7
        System::set_block_number(7);

        let balance_before = Balances::free_balance(ALICE);

        // Finalize round
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        // Current round cleared
        assert!(Consensus::current_consensus_round().is_none());

        // Round marked as completed
        let round = Consensus::consensus_rounds(round_id).unwrap();
        assert_eq!(round.status, RoundStatus::Completed);
        assert!(round.completed_at.is_some());

        // Validator should have received reward
        let balance_after = Balances::free_balance(ALICE);
        assert!(
            balance_after > balance_before,
            "Validator should receive consensus reward"
        );

        // Global metrics updated
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.rounds_completed, 1);

        // Events emitted
        System::assert_has_event(
            Event::ConsensusRoundCompleted {
                round_id,
                total_useful_work: round.total_useful_work,
                participating_validators: 1,
            }
            .into(),
        );
    });
}

#[test]
fn finalize_consensus_round_fails_without_active_round() {
    new_test_ext().execute_with(|| {
        // No round started — should fail
        assert_noop!(
            Consensus::finalize_consensus_round(RuntimeOrigin::root()),
            Error::<Test>::RoundNotInProgress
        );
    });
}

#[test]
fn finalize_consensus_round_requires_authority() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        System::set_block_number(7);

        // Non-root cannot finalize
        assert_noop!(
            Consensus::finalize_consensus_round(RuntimeOrigin::signed(ALICE)),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// Expanded Coverage — Error Paths
// ============================================================================

#[test]
fn register_model_id_overflow_fails() {
    new_test_ext().execute_with(|| {
        NextModelId::<Test>::put(u32::MAX);
        assert_noop!(
            Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                0,
                test_parameters_hash(1),
                1000,
                test_pq_signature(4627),
            ),
            Error::<Test>::IdOverflow
        );
    });
}

#[test]
fn join_validator_id_overflow_fails() {
    new_test_ext().execute_with(|| {
        NextValidatorId::<Test>::put(u32::MAX);
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(ALICE),
                5_000_000,
                test_pq_signature(128),
            ),
            Error::<Test>::IdOverflow
        );
    });
}

#[test]
fn start_round_id_overflow_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        NextRoundId::<Test>::put(u32::MAX);
        assert_noop!(
            Consensus::start_consensus_round(RuntimeOrigin::root(), 10),
            Error::<Test>::IdOverflow
        );
    });
}

#[test]
fn start_round_while_active_round_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));
        // Try starting another round while one is in progress
        assert_noop!(
            Consensus::start_consensus_round(RuntimeOrigin::root(), 5),
            Error::<Test>::RoundAlreadyInProgress
        );
    });
}

#[test]
fn submit_work_invalid_pq_signature_fails() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // Signature too long (> 5000 bytes, ML-DSA-87 BoundedVec limit)
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [1u8; 32],
                500,
                test_pq_signature(6000), // Exceeds 5000-byte limit
            ),
            Error::<Test>::InvalidPQSignature
        );
    });
}

#[test]
fn submit_work_nonexistent_model_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                999,
                0,
                [1u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::ModelNotFound
        );
    });
}

// ============================================================================
// Expanded Coverage — Validate AI Model Edge Cases
// ============================================================================

#[test]
fn validate_model_below_threshold_stays_inactive() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        // MinModelQualityScore = 50 in mock — 49 is below threshold
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 49));
        let model = Consensus::ai_models(0).unwrap();
        assert!(!model.active);
        assert_eq!(model.accuracy_score, 49);
    });
}

#[test]
fn validate_model_at_threshold_activates() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        // Exactly at MinModelQualityScore boundary
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 50));
        let model = Consensus::ai_models(0).unwrap();
        assert!(model.active);
    });
}

// ============================================================================
// Expanded Coverage — Work Types & Scoring
// ============================================================================

#[test]
fn submit_all_work_types_succeed() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // 6 work types: 0-5
        for wt in 0..5u8 {
            assert_ok!(Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                wt,
                [wt; 32],
                500,
                test_pq_signature(128),
            ));
        }
        // Move to next block for rate limit then submit last one
        System::set_block_number(2);
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            5,
            [5u8; 32],
            500,
            test_pq_signature(128),
        ));
    });
}

#[test]
fn fast_computation_bonus_applied() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // computation_time < 1000 → bonus of 100 added to quality
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            999, // just under threshold
            test_pq_signature(128),
        ));
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        let sub = &round.ai_work_submissions[0];
        // Model accuracy = 8500, bonus = 100 → quality_score = 8600
        assert_eq!(sub.quality_score, 8600);
    });
}

#[test]
fn no_bonus_above_computation_threshold() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // computation_time >= 1000 → no bonus
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            1000, // at threshold => no bonus
            test_pq_signature(128),
        ));
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        let sub = &round.ai_work_submissions[0];
        assert_eq!(sub.quality_score, 8500); // no bonus
    });
}

#[test]
fn sustainability_contribution_zero_computation_time() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // CONS-015: computation_time == 0 is now rejected
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [1u8; 32],
                0,
                test_pq_signature(128),
            ),
            Error::<Test>::InvalidComputationTime
        );
    });
}

#[test]
fn sustainability_contribution_high_computation_time() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // computation_time = 5000 → sustainability = min(100, 500*8600/5000)
        // = min(100, 4300000/5000) = min(100, 860) = 100
        // Actually 8600 because compute < 1000 bonus: quality = 8500+100 = 8600
        // Nah wait, 5000 >= 1000 → no bonus, quality = 8500
        // sustainability = min(100, 500*8500/5000) = min(100, 850) = 100
        // Still 100! Need very high time: 500*8500/X < 100 → X > 42500
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            50000,
            test_pq_signature(128),
        ));
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        // 500 * 8500 / 50000 = 85
        assert_eq!(round.ai_work_submissions[0].sustainability_contribution, 85);
    });
}

// ============================================================================
// Expanded Coverage — Multi-Validator Rounds
// ============================================================================

#[test]
fn multi_validator_round_proportional_rewards() {
    new_test_ext().execute_with(|| {
        // Register 3 validators
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(CHARLIE),
            5_000_000,
            test_pq_signature(128),
        ));

        // Each validator registers + validates their own model (ModelNotOwned check)
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(BOB),
            1,
            test_parameters_hash(2),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 1, 8500));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(CHARLIE),
            2,
            test_parameters_hash(3),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 2, 8500));

        // Start round
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));

        // All 3 submit work on their own models with different computation times
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(BOB),
            1,
            1,
            [2u8; 32],
            1500,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(CHARLIE),
            2,
            2,
            [3u8; 32],
            2000,
            test_pq_signature(128),
        ));

        let bal_a = Balances::free_balance(ALICE);
        let bal_b = Balances::free_balance(BOB);
        let bal_c = Balances::free_balance(CHARLIE);

        // Finalize
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        // All should have received some reward
        assert!(Balances::free_balance(ALICE) > bal_a);
        assert!(Balances::free_balance(BOB) > bal_b);
        assert!(Balances::free_balance(CHARLIE) > bal_c);

        // ALICE (fast computation, bonus=100) should have higher quality => bigger reward
        let reward_a = Balances::free_balance(ALICE) - bal_a;
        let reward_b = Balances::free_balance(BOB) - bal_b;
        assert!(
            reward_a > reward_b,
            "Faster computation should earn more: {} vs {}",
            reward_a,
            reward_b
        );
    });
}

#[test]
fn finalize_zero_submissions_round() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        let round_id = Consensus::current_consensus_round().unwrap();

        let bal = Balances::free_balance(ALICE);
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        // Round completed but no rewards distributed
        assert_eq!(Balances::free_balance(ALICE), bal);
        let round = Consensus::consensus_rounds(round_id).unwrap();
        assert_eq!(round.status, RoundStatus::Completed);
        assert!(Consensus::current_consensus_round().is_none());
    });
}

#[test]
fn double_finalize_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));
        // Second finalize — no active round
        assert_noop!(
            Consensus::finalize_consensus_round(RuntimeOrigin::root()),
            Error::<Test>::RoundNotInProgress
        );
    });
}

// ============================================================================
// Expanded Coverage — Rate Limit Isolation
// ============================================================================

#[test]
fn rate_limit_per_account_isolation() {
    new_test_ext().execute_with(|| {
        // Setup with two validators
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));
        // BOB needs his own model (ModelNotOwned check)
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(BOB),
            1,
            test_parameters_hash(2),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 1, 8500));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));

        // ALICE exhausts her rate limit (5 calls)
        for i in 0..5u8 {
            assert_ok!(Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [i; 32],
                500,
                test_pq_signature(128),
            ));
        }
        // ALICE is rate-limited
        assert_noop!(
            Consensus::submit_ai_work(
                RuntimeOrigin::signed(ALICE),
                0,
                0,
                [99u8; 32],
                500,
                test_pq_signature(128),
            ),
            Error::<Test>::RateLimitExceeded
        );
        // BOB can still submit on his own model (separate counter)
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(BOB),
            1,
            0,
            [10u8; 32],
            500,
            test_pq_signature(128),
        ));
    });
}

// ============================================================================
// Expanded Coverage — Model Registration Edge Cases
// ============================================================================

#[test]
fn multiple_models_per_account_tracked() {
    new_test_ext().execute_with(|| {
        for i in 0..5u8 {
            assert_ok!(Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                i % 8,
                test_parameters_hash(i),
                1000,
                test_pq_signature(4627),
            ));
        }
        let models = Consensus::model_by_account(ALICE);
        assert_eq!(models.len(), 5);
        for i in 0..5u32 {
            assert_eq!(models[i as usize], i);
        }
        assert_eq!(Consensus::next_model_id(), 5);
        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.total_models, 5);
    });
}

#[test]
fn register_model_different_accounts() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(BOB),
            1,
            test_parameters_hash(2),
            2000,
            test_pq_signature(4627),
        ));
        assert_eq!(Consensus::model_by_account(ALICE).len(), 1);
        assert_eq!(Consensus::model_by_account(BOB).len(), 1);
        assert_eq!(Consensus::next_model_id(), 2);
    });
}

// ============================================================================
// Expanded Coverage — Validator Selection / Multiple Validators
// ============================================================================

#[test]
fn validator_selection_includes_all_active() {
    new_test_ext().execute_with(|| {
        // Register 3 validators
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(CHARLIE),
            5_000_000,
            test_pq_signature(128),
        ));

        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));
        let round_id = Consensus::current_consensus_round().unwrap();
        let round = Consensus::consensus_rounds(round_id).unwrap();
        // All 3 should be selected
        assert_eq!(round.validators.len(), 3);
    });
}

#[test]
fn low_quality_validator_can_join_and_participate() {
    new_test_ext().execute_with(|| {
        // LOW_QUALITY account has quality=35, reputation=40
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(LOW_QUALITY),
            1_000_000,
            test_pq_signature(128),
        ));
        let vid = Consensus::validator_by_account(LOW_QUALITY).unwrap();
        let val = Consensus::consensus_validators(vid).unwrap();
        assert_eq!(val.quality_score, 35);
        assert_eq!(val.reputation, 40);
        assert!(val.active);
    });
}

#[test]
fn start_round_requires_authority() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_noop!(
            Consensus::start_consensus_round(RuntimeOrigin::signed(ALICE), 5),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// Expanded Coverage — Event Verification
// ============================================================================

#[test]
fn register_model_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        System::assert_has_event(
            Event::AIModelRegistered {
                model_id: 0,
                trainer: ALICE,
                model_type_index: 0,
            }
            .into(),
        );
    });
}

#[test]
fn join_validator_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        System::assert_has_event(
            Event::ValidatorJoined {
                validator_id: 0,
                validator: ALICE,
                stake: 5_000_000,
            }
            .into(),
        );
    });
}

#[test]
fn start_round_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 10));
        System::assert_has_event(
            Event::ConsensusRoundStarted {
                round_id: 0,
                validators_count: 1,
            }
            .into(),
        );
    });
}

#[test]
fn submit_work_emits_event() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        System::set_block_number(1);
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        let round_id = Consensus::current_consensus_round().unwrap();
        let vid = Consensus::validator_by_account(ALICE).unwrap();
        System::assert_has_event(
            Event::AIWorkSubmitted {
                round_id,
                validator_id: vid,
                model_id: 0,
                work_type_index: 0,
                quality_score: 8600,              // 8500 + 100 bonus (fast)
                sustainability_contribution: 100, // time=500 < 1000, sust capped at 100
            }
            .into(),
        );
    });
}

#[test]
fn validate_model_emits_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));
        System::assert_has_event(
            Event::ModelQualityUpdated {
                model_id: 0,
                new_quality_score: 8500,
            }
            .into(),
        );
    });
}

// ============================================================================
// Expanded Coverage — Sustainability EMA
// ============================================================================

#[test]
fn sustainability_ema_multiple_submissions() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        // Submit with high sustainability (fast computation, time=1 → sust=100)
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            1,
            test_pq_signature(128),
        ));
        let vid = Consensus::validator_by_account(ALICE).unwrap();
        let val = Consensus::consensus_validators(vid).unwrap();
        // Initial sustainability starts at 50 by default
        // sust = min(100, 500 * quality / 1). quality = 8500 + 100 (fast bonus) = 8600
        // = min(100, 4300000) = 100
        // EMA: (50*70 + 100*30) / 100 = (3500 + 3000) / 100 = 65
        assert_eq!(val.sustainability_score, 65);

        // Submit again with low sustainability (time=50000 → sust = 500*8600/50000 = 86)
        // Actually time=0 → quality=8500+100=8600. Wait, time=50000 >= 1000 → no bonus, quality=8500
        // sust = 500*8500/50000 = 85
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            1,
            [2u8; 32],
            50000,
            test_pq_signature(128),
        ));
        let val2 = Consensus::consensus_validators(vid).unwrap();
        // EMA: (65*70 + 85*30) / 100 = (4550 + 2550) / 100 = 71
        assert_eq!(val2.sustainability_score, 71);
    });
}

// ============================================================================
// Expanded Coverage — Uptime and Eligible Rounds
// ============================================================================

#[test]
fn eligible_rounds_and_uptime_tracked() {
    new_test_ext().execute_with(|| {
        // Register 2 validators
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));

        // Round 1
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        // Only ALICE submits
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        let vid_a = Consensus::validator_by_account(ALICE).unwrap();
        let val_a = Consensus::consensus_validators(vid_a).unwrap();
        // ALICE: eligible_rounds=1 (was in round), uptime_rounds=1 (submitted)
        assert_eq!(val_a.eligible_rounds, 1);
        assert_eq!(val_a.uptime_rounds, 1);

        let vid_b = Consensus::validator_by_account(BOB).unwrap();
        let val_b = Consensus::consensus_validators(vid_b).unwrap();
        // BOB: eligible_rounds=1 (was in round), uptime_rounds=0 (didn't submit)
        assert_eq!(val_b.eligible_rounds, 1);
        assert_eq!(val_b.uptime_rounds, 0);
    });
}

// ============================================================================
// Expanded Coverage — Contribution Score with Different Accounts
// ============================================================================

#[test]
fn contribution_score_low_quality_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(LOW_QUALITY),
            1_000_000,
            test_pq_signature(128),
        ));
        // LOW_QUALITY: quality=35, reputation=40, stake=1_000_000
        // blended_quality = (40*60 + 35*40) / 100 = (2400+1400)/100 = 38
        // sustainability starts at 50, eligible_rounds=0 → uptime = 50
        // stake_normalized = 1_000_000 / 1_000_000 = 1 → capped at 10 max
        // score = (38*50 + 1*20 + 50*20 + 50*10) / 100
        //       = (1900 + 20 + 1000 + 500) / 100 = 34
        let score = Consensus::get_validator_contribution_score(&LOW_QUALITY);
        assert_eq!(score, 34);
    });
}

#[test]
fn contribution_score_eve_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(EVE),
            5_000_000,
            test_pq_signature(128),
        ));
        // EVE: quality=65, reputation=70, stake=5_000_000
        // blended_quality = (70*60 + 65*40) / 100 = (4200+2600)/100 = 68
        // sustainability = 50, uptime = 50
        // stake_normalized = 5_000_000 / 1_000_000 = 5 → capped
        // score = (68*50 + 5*20 + 50*20 + 50*10) / 100
        //       = (3400 + 100 + 1000 + 500) / 100 = 50
        let score = Consensus::get_validator_contribution_score(&EVE);
        assert_eq!(score, 50);
    });
}

// ============================================================================
// Expanded Coverage — Global Metrics Tracking
// ============================================================================

#[test]
fn global_metrics_updated_across_operations() {
    new_test_ext().execute_with(|| {
        // Register 2 models
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(BOB),
            1,
            test_parameters_hash(2),
            2000,
            test_pq_signature(4627),
        ));
        let m1 = Consensus::global_ai_metrics();
        assert_eq!(m1.total_models, 2);
        assert_eq!(m1.active_validators, 0);

        // Register 2 validators
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        let m2 = Consensus::global_ai_metrics();
        assert_eq!(m2.active_validators, 2);

        // Validate one model
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));

        // Start and finalize a round
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        let m3 = Consensus::global_ai_metrics();
        assert_eq!(m3.rounds_completed, 1);
        assert!(m3.total_useful_work > 0);
    });
}

// ============================================================================
// Expanded Coverage — Sequential Rounds
// ============================================================================

#[test]
fn consecutive_rounds_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(ALICE),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(4627),
        ));
        assert_ok!(Consensus::validate_ai_model(RuntimeOrigin::root(), 0, 8500));

        // Round 1
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        System::set_block_number(7);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        // Round 2
        assert_ok!(Consensus::start_consensus_round(RuntimeOrigin::root(), 5));
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            1,
            [2u8; 32],
            800,
            test_pq_signature(128),
        ));
        System::set_block_number(13);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        let metrics = Consensus::global_ai_metrics();
        assert_eq!(metrics.rounds_completed, 2);

        let vid = Consensus::validator_by_account(ALICE).unwrap();
        let val = Consensus::consensus_validators(vid).unwrap();
        assert_eq!(val.eligible_rounds, 2);
        assert_eq!(val.uptime_rounds, 2);
    });
}

// ============================================================================
// Expanded Coverage — Edge Cases
// ============================================================================

#[test]
fn no_funds_account_cannot_stake_below_minimum() {
    new_test_ext().execute_with(|| {
        // NO_FUNDS has 100_000 balance but min stake is 1_000_000
        // Even with enough balance, stake below MinConsensusStake is rejected
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(NO_FUNDS),
                500_000,
                test_pq_signature(128),
            ),
            Error::<Test>::InsufficientStake
        );
    });
}

#[test]
fn finalize_rewards_emits_distribution_event() {
    new_test_ext().execute_with(|| {
        setup_consensus_for_rate_limit();
        assert_ok!(Consensus::submit_ai_work(
            RuntimeOrigin::signed(ALICE),
            0,
            0,
            [1u8; 32],
            500,
            test_pq_signature(128),
        ));
        // Round started at block 1, duration=10 → end=11
        System::set_block_number(12);
        assert_ok!(Consensus::finalize_consensus_round(RuntimeOrigin::root()));

        System::assert_has_event(
            Event::ConsensusRewardsDistributed {
                round_id: 0,
                total_rewards: 100_000, // ConsensusReward
            }
            .into(),
        );
    });
}
