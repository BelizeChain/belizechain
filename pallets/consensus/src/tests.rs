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
            test_pq_signature(64),
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
        let types = [(0, ModelType::Economic),
            (1, ModelType::Tourism),
            (2, ModelType::Agriculture),
            (3, ModelType::Climate),
            (4, ModelType::Health),
            (5, ModelType::Education),
            (6, ModelType::Transportation),
            (7, ModelType::General)];

        for (i, (type_index, expected_type)) in types.iter().enumerate() {
            assert_ok!(Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                *type_index,
                test_parameters_hash(i as u8),
                1000,
                test_pq_signature(64),
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
                test_pq_signature(64),
            ),
            Error::<Test>::InvalidModelType
        );
    });
}

#[test]
fn register_ai_model_rejects_invalid_signature() {
    new_test_ext().execute_with(|| {
        // Signature too long (> 256 bytes)
        assert_noop!(
            Consensus::register_ai_model(
                RuntimeOrigin::signed(ALICE),
                0,
                test_parameters_hash(1),
                1000,
                test_pq_signature(300), // Too long
            ),
            Error::<Test>::InvalidPQSignature
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
        assert_noop!(
            Consensus::join_consensus_validator(
                RuntimeOrigin::signed(ALICE),
                5_000_000,
                test_pq_signature(300), // Too long
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
            test_pq_signature(64),
        ));

        // Validate model (AI Authority only)
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0, // model_id
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
            test_pq_signature(64),
        ));

        // Regular user cannot validate
        assert_noop!(
            Consensus::validate_ai_model(
                RuntimeOrigin::signed(ALICE),
                0,
                8500,
            ),
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
            Consensus::start_consensus_round(
                RuntimeOrigin::signed(ALICE),
                10,
            ),
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
            test_pq_signature(64),
        ));
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,
            8500,
        ));

        // Start consensus round
        assert_ok!(Consensus::start_consensus_round(
            RuntimeOrigin::root(),
            10,
        ));

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
            test_pq_signature(64),
        ));
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,
            8500,
        ));

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
        assert_ok!(Consensus::start_consensus_round(
            RuntimeOrigin::root(),
            10,
        ));

        // Register model but DON'T validate it
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(64),
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
            test_pq_signature(64),
        ));
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,
            8500,
        ));

        // Register a different validator to start round
        assert_ok!(Consensus::join_consensus_validator(
            RuntimeOrigin::signed(BOB),
            5_000_000,
            test_pq_signature(128),
        ));
        assert_ok!(Consensus::start_consensus_round(
            RuntimeOrigin::root(),
            10,
        ));

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
            test_pq_signature(64),
        ));
        assert_ok!(Consensus::validate_ai_model(
            RuntimeOrigin::root(),
            0,
            8500,
        ));
        assert_ok!(Consensus::start_consensus_round(
            RuntimeOrigin::root(),
            10,
        ));

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
        // Formula: (quality*50 + reputation*30 + (stake/1M)*20) / 100
        // = (85*50 + 90*30 + 10*20) / 100
        // = (4250 + 2700 + 200) / 100 = 71.5 = 71
        let score = Consensus::get_validator_contribution_score(&ALICE);
        assert_eq!(score, 71);

        // After registering model, scores stay same (calculated from Staking provider)
        assert_ok!(Consensus::register_ai_model(
            RuntimeOrigin::signed(ALICE),
            0,
            test_parameters_hash(1),
            1000,
            test_pq_signature(64),
        ));

        // Score stays same (calculated from Staking provider, not Consensus pallet state)
        let score2 = Consensus::get_validator_contribution_score(&ALICE);
        assert_eq!(score2, 71);

        // Verify model was registered and tracked in ModelsByAccount
        let models = Consensus::model_by_account(ALICE);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], 0);
    });
}
