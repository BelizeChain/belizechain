//! Benchmarking for the consensus pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_ai_model() {
        let caller: T::AccountId = whitelisted_caller();
        let model_type_index: u8 = 0; // NLP
        let parameters_hash: [u8; 32] = [1u8; 32];
        let training_data_size: u32 = 1000;
        let pq_signature: Vec<u8> = vec![0u8; 4627];

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            model_type_index,
            parameters_hash,
            training_data_size,
            pq_signature,
        );
    }

    #[benchmark]
    fn join_validator() {
        let caller: T::AccountId = whitelisted_caller();
        let stake = T::MinConsensusStake::get();
        let _ = T::Currency::make_free_balance_be(&caller, stake * 10u32.into());
        let pq_public_key: Vec<u8> = vec![0u8; 2592];

        #[extrinsic_call]
        join_consensus_validator(RawOrigin::Signed(caller), stake, pq_public_key);
    }

    #[benchmark]
    fn validate_model() {
        let caller: T::AccountId = whitelisted_caller();

        // Register an AI model first
        let _ = Pallet::<T>::register_ai_model(
            RawOrigin::Signed(caller.clone()).into(),
            0u8,
            [1u8; 32],
            1000u32,
            vec![0u8; 4627],
        );

        let model_id: u32 = 0;
        let accuracy_score: u32 = 95;

        #[extrinsic_call]
        validate_ai_model(RawOrigin::Root, model_id, accuracy_score);
    }

    #[benchmark]
    fn start_consensus_round() {
        let duration: BlockNumberFor<T> = 100u32.into();

        #[extrinsic_call]
        _(RawOrigin::Root, duration);
    }

    #[benchmark]
    fn submit_ai_work() {
        let caller: T::AccountId = whitelisted_caller();
        let stake = T::MinConsensusStake::get();
        let _ = T::Currency::make_free_balance_be(&caller, stake * 10u32.into());

        let validator_id: u32 = 0;
        let model_id: u32 = 0;
        let round_id: u32 = 0;

        // ── 1. Insert validator directly into storage ──
        let validator = ConsensusValidator {
            validator: caller.clone(),
            stake,
            models: BoundedVec::new(),
            participation_score: 0,
            quality_score: 80, // well above MinModelQualityScore (60)
            total_rewards: Zero::zero(),
            active: true,
            pq_public_key: vec![0u8; 2592].try_into().expect("pk fits"),
            reputation: 50,
            sustainability_score: 50,
            eligible_rounds: 0,
            uptime_rounds: 0,
        };
        ConsensusValidators::<T>::insert(validator_id, validator);
        ValidatorByAccount::<T>::insert(&caller, validator_id);
        NextValidatorId::<T>::put(1u32);

        // ── 2. Insert active AI model owned by caller ──
        let model = AIModel {
            model_id,
            trainer: caller.clone(),
            model_type: ModelType::General,
            parameters_hash: [1u8; 32],
            accuracy_score: 95,
            training_data_size: 1000,
            validated_at: Some(0u64),
            consensus_rounds: 0,
            useful_work_score: 0,
            active: true,
            pq_signature: vec![0u8; 4627].try_into().expect("sig fits"),
        };
        AIModels::<T>::insert(model_id, model);
        NextModelId::<T>::put(1u32);

        // ── 3. Insert consensus round that includes our validator ──
        let round = ConsensusRound {
            round_id,
            start_block: frame_system::Pallet::<T>::block_number(),
            duration: 100u32.into(),
            validators: vec![(validator_id, 80u32)].try_into().expect("fits"),
            ai_work_submissions: BoundedVec::new(),
            status: RoundStatus::InProgress,
            completed_at: None,
            total_useful_work: 0,
        };
        ConsensusRounds::<T>::insert(round_id, round);
        CurrentConsensusRound::<T>::put(round_id);
        NextRoundId::<T>::put(1u32);

        let work_type_index: u8 = 0; // Training
        let result_hash: [u8; 32] = [2u8; 32];
        let computation_time: u32 = 60;
        let pq_signature: Vec<u8> = vec![0u8; 4627];

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            model_id,
            work_type_index,
            result_hash,
            computation_time,
            pq_signature,
        );
    }

    #[benchmark]
    fn finalize_consensus_round() {
        // Start a round first
        let _ = Pallet::<T>::start_consensus_round(RawOrigin::Root.into(), 1u32.into());

        #[extrinsic_call]
        _(RawOrigin::Root);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
