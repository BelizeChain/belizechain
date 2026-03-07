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
        let pq_signature: Vec<u8> = vec![0u8; 64];

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
        let pq_public_key: Vec<u8> = vec![0u8; 64];

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
            vec![0u8; 64],
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

        // Register as consensus validator
        let _ = Pallet::<T>::join_consensus_validator(
            RawOrigin::Signed(caller.clone()).into(),
            stake,
            vec![0u8; 64],
        );

        // Register an AI model
        let _ = Pallet::<T>::register_ai_model(
            RawOrigin::Signed(caller.clone()).into(),
            0u8,
            [1u8; 32],
            1000u32,
            vec![0u8; 64],
        );

        // Activate the model (register_ai_model sets active=false by default)
        let _ = Pallet::<T>::validate_ai_model(
            RawOrigin::Root.into(),
            0u32,
            95u32, // well above MinModelQualityScore (60)
        );

        // Start a consensus round
        let _ = Pallet::<T>::start_consensus_round(
            RawOrigin::Root.into(),
            100u32.into(),
        );

        let model_id: u32 = 0;
        let work_type_index: u8 = 0; // Training
        let result_hash: [u8; 32] = [2u8; 32];
        let computation_time: u32 = 60;
        let pq_signature: Vec<u8> = vec![0u8; 64];

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
        let _ = Pallet::<T>::start_consensus_round(
            RawOrigin::Root.into(),
            1u32.into(),
        );

        #[extrinsic_call]
        _(RawOrigin::Root);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
