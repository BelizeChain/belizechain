//! Benchmarking setup for pallet-quantum

use super::*;
use crate::Pallet as Quantum;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
    submit_quantum_job {
        let caller: T::AccountId = whitelisted_caller();
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 100u32;
        let num_shots = 1024u32;
        
        // Fund the caller
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        
    }: _(RawOrigin::Signed(caller.clone()), job_id.clone(), 4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots)
    verify {
        assert!(QuantumJobs::<T>::contains_key(&job_id));
    }

    update_job_status {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_002".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        
        // Submit job first
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
    }: _(RawOrigin::Signed(executor.clone()), job_id.clone(), 1)
    verify {
        let job = QuantumJobs::<T>::get(&job_id).unwrap();
        assert_eq!(job.status, 1);
    }

    record_quantum_result {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_003".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"{'counts': {'00': 512, '11': 512}}".to_vec().try_into().unwrap();
        let accuracy = 95u8;
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Submit job
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
    }: _(RawOrigin::Signed(executor), job_id.clone(), result, accuracy)
    verify {
        assert!(QuantumResults::<T>::contains_key(&job_id));
    }

    verify_quantum_result {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let verifier: T::AccountId = account("verifier", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_004".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        let proof: BoundedVec<u8, ConstU32<256>> = 
            b"verification_proof".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Submit and record result
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
    }: _(RawOrigin::Signed(verifier), job_id.clone(), proof)
    verify {
        let job = QuantumJobs::<T>::get(&job_id).unwrap();
        assert_eq!(job.verification_status, VerificationStatus::Verified);
    }

    mint_achievement_nft {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_005".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Submit and record result
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
    }: _(RawOrigin::Signed(caller), job_id, 0,  // FirstQuantumJob
            true, 10u16, 95u8)
    verify {
        assert_eq!(NFTCounter::<T>::get(), 1);
    }

    transfer_nft {
        let caller: T::AccountId = whitelisted_caller();
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_006".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(caller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        
    }: _(RawOrigin::Signed(caller), nft_id, recipient.clone())
    verify {
        let nft = QuantumAchievements::<T>::get(nft_id).unwrap();
        assert_eq!(nft.owner, recipient);
    }

    list_nft {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_007".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(caller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        let price = 1_000_000_000_000u32.into();
        let duration = 100u32.into();
        
    }: _(RawOrigin::Signed(caller), nft_id, price, duration)
    verify {
        assert!(NFTListings::<T>::contains_key(nft_id));
    }

    buy_nft {
        let seller: T::AccountId = account("seller", 0, SEED);
        let buyer: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_008".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&seller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&buyer, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create and list NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(seller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(seller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        Quantum::<T>::list_nft(
            RawOrigin::Signed(seller).into(),
            nft_id,
            1_000_000_000_000u32.into(),
            100u32.into(),
        ).unwrap();
        
    }: _(RawOrigin::Signed(buyer.clone()), nft_id)
    verify {
        let nft = QuantumAchievements::<T>::get(nft_id).unwrap();
        assert_eq!(nft.owner, buyer);
    }

    delist_nft {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_009".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create and list NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(caller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        Quantum::<T>::list_nft(
            RawOrigin::Signed(caller.clone()).into(),
            nft_id,
            1_000_000_000_000u32.into(),
            100u32.into(),
        ).unwrap();
        
    }: _(RawOrigin::Signed(caller), nft_id)
    verify {
        assert!(!NFTListings::<T>::contains_key(nft_id));
    }

    bridge_to_ethereum {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_010".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        let eth_address: BoundedVec<u8, ConstU32<64>> = 
            b"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(caller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        
    }: _(RawOrigin::Signed(caller), nft_id, eth_address)
    verify {
        assert!(BridgeRequests::<T>::contains_key(nft_id));
    }

    bridge_to_parachain {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_011".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        let recipient: BoundedVec<u8, ConstU32<64>> = 
            b"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Create NFT
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::mint_achievement_nft(
            RawOrigin::Signed(caller.clone()).into(),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ).unwrap();
        
        let nft_id = 0;
        let parachain_id = 1000u32;
        
    }: _(RawOrigin::Signed(caller), nft_id, parachain_id, recipient)
    verify {
        assert!(BridgeRequests::<T>::contains_key(nft_id));
    }

    request_verification {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_012".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Submit and record result
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
    }: _(RawOrigin::Signed(caller), job_id.clone(), 3u8)
    verify {
        assert!(VerificationRequests::<T>::contains_key(&job_id));
    }

    submit_verification {
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, SEED);
        let validator: T::AccountId = account("validator", 0, SEED);
        let job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>> = 
            b"benchmark_job_013".to_vec().try_into().unwrap();
        let circuit: BoundedVec<u8, ConstU32<MAX_CIRCUIT_SIZE>> = 
            b"OPENQASM 2.0; qreg q[5];".to_vec().try_into().unwrap();
        let result: BoundedVec<u8, ConstU32<MAX_RESULT_SIZE>> = 
            b"result_data".to_vec().try_into().unwrap();
        
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        T::Currency::make_free_balance_be(&executor, BalanceOf::<T>::max_value());
        
        // Submit, record, and request verification
        Quantum::<T>::submit_quantum_job(
            RawOrigin::Signed(caller.clone()).into(),
            job_id.clone(),
            4,  // Qiskit
            [1u8; 32],  // circuit_hash
            10u16,  // num_qubits
            100u32,  // circuit_depth
            1024u32,  // num_shots
        ).unwrap();
        
        Quantum::<T>::record_quantum_result(
            RawOrigin::Signed(executor).into(),
            job_id.clone(),
            result,
            result,  // verification_proof
            95,
        ).unwrap();
        
        Quantum::<T>::request_verification(
            RawOrigin::Signed(caller).into(),
            job_id.clone(),
            3,
        ).unwrap();
        
        let result_hash = [0u8; 32];
        
    }: _(RawOrigin::Signed(validator), job_id.clone(), 0,  // Approve
            90u8, result_hash)
    verify {
        let request = VerificationRequests::<T>::get(&job_id).unwrap();
        assert_eq!(request.verifications.len(), 1);
    }
}

impl_benchmark_test_suite!(Quantum, crate::mock::new_test_ext(), crate::mock::Test);
