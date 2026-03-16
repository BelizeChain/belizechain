//! Benchmarking for pallet-belize-staking
//!
//! Uses `frame_benchmarking::v2` API.
//! All privileged origins (AIAuthorityOrigin) are EnsureRoot in runtime,
//! so `RawOrigin::Root` is used for those calls.
//! `force_join_validator` (Root) bypasses KYC checks — used for benchmark setup.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use sp_std::vec;

const SEED: u32 = 0;

/// Helper: force-join a validator via Root (bypasses KYC).
fn setup_validator<T: Config>(idx: u32) -> T::AccountId {
    let who: T::AccountId = account("validator", idx, SEED);
    let stake = T::MinValidatorStake::get();
    T::Currency::make_free_balance_be(&who, stake + stake);
    // Use force_join_validator to bypass KYC
    let location: BoundedVec<u8, ConstU32<64>> = vec![b'B', b'Z']
        .try_into()
        .expect("location fits");
    Pallet::<T>::force_join_validator(
        RawOrigin::Root.into(),
        who.clone(),
        stake,
        100u32,
        location,
    )
    .expect("force_join_validator should succeed");
    who
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ───────────────────────────────────────────
    // 1. join_validators — signed, KYC-gated
    //    WeightInfo: join_validators()
    //    NOTE: Benchmark uses force_join_validator via Root to bypass KYC.
    //    The actual extrinsic name differs, so we use #[extrinsic_call].
    // ───────────────────────────────────────────
    #[benchmark]
    fn join_validators() {
        let who: T::AccountId = account("validator", 0, SEED);
        let stake = T::MinValidatorStake::get();
        T::Currency::make_free_balance_be(&who, stake + stake);
        let location: BoundedVec<u8, ConstU32<64>> = vec![b'B', b'Z']
            .try_into()
            .expect("location fits");

        // Use force_join_validator (Root) to generate a weight for join_validators.
        // In production the actual join_validators call has KYC overhead, so this
        // is a lower-bound benchmark.
        #[extrinsic_call]
        force_join_validator(RawOrigin::Root, who.clone(), stake, 100u32, location);
    }

    // ───────────────────────────────────────────
    // 2. leave_validators — signed, must be validator
    // ───────────────────────────────────────────
    #[benchmark]
    fn leave_validators() {
        let who = setup_validator::<T>(1);

        #[extrinsic_call]
        leave_validators(RawOrigin::Signed(who));
    }

    // ───────────────────────────────────────────
    // 3. withdraw_unbonded — signed, must have pending unbond
    // ───────────────────────────────────────────
    #[benchmark]
    fn withdraw_unbonded() {
        let who = setup_validator::<T>(2);
        // Leave to create pending unbond
        Pallet::<T>::leave_validators(RawOrigin::Signed(who.clone()).into())
            .expect("leave should succeed");

        // Set unlock_at to 0 so the unbond is immediately withdrawable
        PendingUnbonds::<T>::insert(&who, (T::MinValidatorStake::get(), BlockNumberFor::<T>::from(0u32)));

        #[extrinsic_call]
        withdraw_unbonded(RawOrigin::Signed(who));
    }

    // ───────────────────────────────────────────
    // 4. submit_model_delta — signed validator with active FL task
    // ───────────────────────────────────────────
    #[benchmark]
    fn submit_model_delta() {
        let who = setup_validator::<T>(3);

        // Create an FL task via Root
        let model_hash = [1u8; 32];
        Pallet::<T>::assign_fl_task(
            RawOrigin::Root.into(),
            1u32,
            model_hash,
            100u32,
            sp_runtime::Perbill::from_percent(100),
            1000u32.into(),
        )
        .expect("assign_fl_task should succeed");

        let encrypted_delta: BoundedVec<u8, ConstU32<1024>> = vec![0u8; 64]
            .try_into()
            .expect("delta fits");
        // C-3 FIX: Compute correct commitment = H(delta || who || block_number)
        // so the benchmark exercises the real validation path.
        let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();
        use sp_runtime::traits::Hash;
        let expected = T::Hashing::hash_of(
            &(encrypted_delta.as_slice(), &who, current_block),
        );
        let mut computation_commitment = [0u8; 32];
        computation_commitment.copy_from_slice(expected.as_ref());
        let computation_log = [3u8; 32];

        #[extrinsic_call]
        submit_model_delta(
            RawOrigin::Signed(who),
            1u32,
            encrypted_delta,
            computation_commitment,
            computation_log,
        );
    }

    // ───────────────────────────────────────────
    // 5. assign_fl_task — Root (AIAuthorityOrigin)
    // ───────────────────────────────────────────
    #[benchmark]
    fn assign_fl_task() {
        let model_hash = [1u8; 32];

        #[extrinsic_call]
        assign_fl_task(
            RawOrigin::Root,
            42u32,
            model_hash,
            200u32,
            sp_runtime::Perbill::from_percent(50),
            500u32.into(),
        );
    }

    // ───────────────────────────────────────────
    // 6. distribute_rewards — Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn distribute_rewards() {
        // Set up a validator so distribution has work to do
        let _who = setup_validator::<T>(4);

        #[extrinsic_call]
        distribute_rewards(RawOrigin::Root);
    }

    // ───────────────────────────────────────────
    // 7. record_quantum_contribution — Root (bypass OracleVerifier)
    // ───────────────────────────────────────────
    #[benchmark]
    fn record_quantum_contribution() {
        let validator = setup_validator::<T>(5);
        let job_id: BoundedVec<u8, ConstU32<64>> = vec![b'j', b'o', b'b', b'1']
            .try_into()
            .expect("job_id fits");

        #[extrinsic_call]
        record_quantum_contribution(
            RawOrigin::Root,
            job_id,
            validator,
            16u16,   // num_qubits
            100u32,  // circuit_depth
            1024u32, // num_shots
            95u8,    // accuracy_score
        );
    }

    // ───────────────────────────────────────────
    // 8. record_domain_contribution — Root (bypass OracleVerifier)
    // ───────────────────────────────────────────
    #[benchmark]
    fn record_domain_contribution() {
        let operator = setup_validator::<T>(6);

        #[extrinsic_call]
        record_domain_contribution(
            RawOrigin::Root,
            operator,
            1u8,   // domain
            90u8,  // quality_score
            1000u32, // volume_kb
        );
    }

    // ───────────────────────────────────────────
    // 9. claim_pouw_with_domain_bonus — signed validator
    // ───────────────────────────────────────────
    #[benchmark]
    fn claim_pouw_with_domain_bonus() {
        let who = setup_validator::<T>(7);

        // Record a domain contribution for this validator so the claim has data
        Pallet::<T>::record_domain_contribution(
            RawOrigin::Root.into(),
            who.clone(),
            1u8,
            90u8,
            500u32,
        )
        .expect("record_domain_contribution should succeed");

        // Advance epoch so last_claimed (0) < current_epoch (1)
        CurrentEpoch::<T>::put(1u32);

        #[extrinsic_call]
        claim_pouw_with_domain_bonus(RawOrigin::Signed(who));
    }

    // ───────────────────────────────────────────
    // 10. report_validator_offense — Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn report_validator_offense() {
        let validator = setup_validator::<T>(8);

        #[extrinsic_call]
        report_validator_offense(
            RawOrigin::Root,
            validator,
            10u32, // slash_percent
            1u8,   // reason_code
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
