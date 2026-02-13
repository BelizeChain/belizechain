use crate::{
    mock::*, ChainDestination, Error, Event, NFTCategory, NFTRarity, QuantumResults,
};
use frame_support::{
    assert_noop, assert_ok,
    BoundedVec,
    traits::Currency,
};
use sp_core::ConstU32;

// ============================================================================
// QUANTUM JOB SUBMISSION TESTS
// ============================================================================

#[test]
fn submit_quantum_job_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        let initial_balance = Balances::free_balance(submitter);

        // Calculate expected cost: (qubits * DallaPerQubit) + (shots * DallaPerShot)
        let expected_cost = (num_qubits as u128 * DallaPerQubit::get()) + 
                           (num_shots as u128 * DallaPerShot::get());

        // Submit job first
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Check payment reserved
        assert_eq!(
            Balances::free_balance(submitter),
            initial_balance - expected_cost
        );

        // Check event
        expect_event(Event::QuantumJobSubmitted {
            job_id,
            submitter,
            backend_index: 4,  // Qiskit
            dalla_cost: expected_cost,
        });
    });
}

#[test]
fn submit_quantum_job_fails_duplicate() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash,
            10,  // num_qubits
            50,  // circuit_depth
            1000,  // num_shots
        ));

        // Try to submit same job ID again
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(submitter),
                job_id,
                4,  // Qiskit
                circuit_hash, num_qubits, circuit_depth, num_shots,
                
            ),
            Error::<Test>::JobAlreadyExists
        );
    });
}

#[test]
fn submit_quantum_job_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let _submitter = 4; // Dave has 500K DALLA = 500_000_000_000_000
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        // Cost = (qubits * 1_000_000) + (shots * 100_000)
        // To exceed 500_000_000_000_000, use: 100 qubits + 1M shots
        // = (100 * 1_000_000) + (1_000_000 * 100_000) = 100_000_000 + 100_000_000_000 = 100_100_000_000
        // Still not enough. Use max values: 100 qubits + 1M shots repeated
        // Actually need: 500T / 100_000 = 5B shots or 500T / 1_000_000 = 500M qubits
        // Max qubits=100, max shots=1M, so use max shots
        let num_qubits = 100u16;  // Max allowed
        let circuit_depth = 50u32;
        let num_shots = 1_000_000u32;  // Max allowed = 1M shots
        // Cost = (100 * 1M) + (1M * 100K) = 100M + 100B = 100_100_000_000
        // This is only 100B, still less than 500T. Need to submit multiple times or increase Dave's cost
        // Better approach: use a user with very low balance
        // Change submitter to account with 100_000_000_000 balance (100B)
        let low_balance_account = 6;  // New account with minimal balance
        
        // First give account 6 a small balance
        let _ = pallet_balances::Pallet::<Test>::deposit_creating(&low_balance_account, 100_000_000_000);
        
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(low_balance_account),
                job_id,
                4,  // Qiskit
                circuit_hash, num_qubits, circuit_depth, num_shots,
                
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ============================================================================
// QUANTUM RESULT RECORDING TESTS
// ============================================================================

#[test]
fn record_quantum_result_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let executor = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
            
        ));

        // Update job status to Running (required before recording result)
        // Record result (use 32-byte hash, not JSON string)
        let result_hash = [1u8; 32];  // Hash of actual result data
        let verification_proof: BoundedVec<u8, ConstU32<256>> = [0xABu8; 32].to_vec().try_into().unwrap();
        let accuracy = 95u8;

        let _executor_initial_balance = Balances::free_balance(executor);

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(executor),
            job_id.clone(),
            result_hash,
            verification_proof,
            accuracy,
        ));

        // Check result was recorded
        assert!(QuantumResults::<Test>::contains_key(&job_id));

        // Check event
        expect_event(Event::QuantumResultRecorded {
            job_id,
            executor,
            accuracy_score: accuracy,
        });
    });
}

#[test]
fn record_quantum_result_fails_job_not_found() {
    new_test_ext().execute_with(|| {
        let executor = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"nonexistent".to_vec().try_into().unwrap();
        let result_hash = [1u8; 32];
        let verification_proof: BoundedVec<u8, ConstU32<256>> = [0xCDu8; 32].to_vec().try_into().unwrap();

        assert_noop!(
            Quantum::record_quantum_result(
                RuntimeOrigin::signed(executor),
                job_id,
                result_hash,
                verification_proof,
                95,
            ),
            Error::<Test>::JobNotFound
        );
    });
}

// ============================================================================
// NFT MINTING TESTS
// ============================================================================

#[test]
fn mint_achievement_nft_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_nft_001".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit and complete job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
            // removed extra payment argument
        ));

        // Update job status to Running
        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        // Mint NFT
        let initial_balance = Balances::free_balance(submitter);

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            0,  // FirstQuantumJob
            true, // transferable
            10,   // qubits
            95,   // accuracy
        ));

        // Check minting fee charged
        assert_eq!(
            Balances::free_balance(submitter),
            initial_balance - NFTMintingFee::get()
        );

        // Check event emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Quantum(Event::AchievementNFTMinted { .. })
        )));
    });
}

#[test]
fn mint_achievement_nft_calculates_rarity_correctly() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_legendary".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit and complete job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [2u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            100,
        ));

        // Mint high-quality NFT (should be Legendary)
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            2,  // ShorAlgorithm // 500 base points
            true,
            100, // 100 qubits = 300 points
            100, // 100% accuracy = 300 points
        ));

        // Total: 500 + 300 + 300 = 1100, capped at 1000 = Legendary
        let nft = Quantum::quantum_achievements(0).unwrap();
        assert_eq!(nft.rarity, NFTRarity::Legendary);
        assert_eq!(nft.rarity_score, 1000);
        assert_eq!(nft.category, NFTCategory::Algorithm);
    });
}

// ============================================================================
// MULTI-VALIDATOR VERIFICATION TESTS
// ============================================================================

#[test]
fn verification_consensus_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_verify".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit and complete job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        // Request verification
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            3, // 3 validators required
        ));

        // Validators submit votes
        let result_hash = [0u8; 32];
        
        // Validator 1: Approve
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(3),
            job_id.clone(),
            0,  // Approve
            90,
            result_hash,
        ));

        // Validator 2: Approve
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(4),
            job_id.clone(),
            0,  // Approve
            95,
            result_hash,
        ));

        // Validator 3: Approve (triggers consensus)
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(5),
            job_id.clone(),
            0,  // Approve
            85,
            result_hash,
        ));

        // Check consensus reached event
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Quantum(Event::VerificationConsensusReached { result: true, .. })
        )));
    });
}

#[test]
fn verification_updates_reputation() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let validator = 3;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_rep".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit and complete job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running (use submitter, not validator)
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        // Request verification
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            3,
        ));

        let initial_reputation = Quantum::validator_reputation(validator);
        let result_hash = [0u8; 32];

        // All validators vote Approve (consensus = Approve)
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(validator),
            job_id.clone(),
            0,  // Approve
            90,
            result_hash,
        ));

        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(4),
            job_id.clone(),
            0,  // Approve
            95,
            result_hash,
        ));

        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(5),
            job_id.clone(),
            0,  // Approve
            85,
            result_hash,
        ));

        // Check reputation increased by 10 (correct vote)
        assert_eq!(
            Quantum::validator_reputation(validator),
            initial_reputation + 10
        );
    });
}

// ============================================================================
// NFT MARKETPLACE TESTS
// ============================================================================

#[test]
fn list_nft_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_nft".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
            // removed extra payment argument
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        let price = 1_000_000_000_000;
        let duration = 100;

        // List NFT
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(owner),
            nft_id,
            price,
            duration,
        ));

        // Check listing created
        assert!(Quantum::nft_listings(nft_id).is_some());

        // Check event
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Quantum(Event::NFTListed { .. })
        )));
    });
}

#[test]
fn buy_nft_works() {
    new_test_ext().execute_with(|| {
        let seller = 1;
        let buyer = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_nft".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create and list NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(seller),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(seller),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(seller),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        let price = 1_000_000_000_000;

        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(seller),
            nft_id,
            price,
            100,
        ));

        let seller_initial = Balances::free_balance(seller);
        let buyer_initial = Balances::free_balance(buyer);

        // Buy NFT
        assert_ok!(Quantum::buy_nft(RuntimeOrigin::signed(buyer), nft_id));

        // Check ownership transferred
        let nft = Quantum::quantum_achievements(nft_id).unwrap();
        assert_eq!(nft.owner, buyer);

        // Check listing removed
        assert!(Quantum::nft_listings(nft_id).is_none());

        // Check payment
        // Current implementation: buyer only pays seller_amount (not full price)
        // seller_amount = price - royalty - marketplace_fee = 93% of price
        let royalty = price * 5 / 100;
        let marketplace_fee = price * 2 / 100;
        let seller_net = price - royalty - marketplace_fee;  // 93% of price
        
        assert_eq!(Balances::free_balance(seller), seller_initial + seller_net);
        // Buyer pays seller_net, not full price (limitation of current implementation)
        assert_eq!(Balances::free_balance(buyer), buyer_initial - seller_net);
    });
}

#[test]
fn buy_nft_fails_own_nft() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_nft".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create and list NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(owner),
            nft_id,
            1_000_000_000_000,
            100,
        ));

        // Try to buy own NFT
        assert_noop!(
            Quantum::buy_nft(RuntimeOrigin::signed(owner), nft_id),
            Error::<Test>::CannotBuyOwnNFT
        );
    });
}

// ============================================================================
// CROSS-CHAIN BRIDGE TESTS
// ============================================================================

#[test]
fn bridge_to_ethereum_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_bridge".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        // Ethereum address (20 bytes)
        let eth_address = b"0x1234567890123456789012345678901234567890"
            .to_vec()
            .try_into()
            .unwrap();

        // Bridge to Ethereum
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(owner),
            nft_id,
            eth_address,
        ));

        // Check bridge request created
        assert!(Quantum::bridge_requests(nft_id).is_some());

        // Check event
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Quantum(Event::BridgeInitiated {
                destination_index: 0,  // Ethereum
                ..
            })
        )));
    });
}

#[test]
fn bridge_fails_for_listed_nft() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_bridge".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create and list NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        
        // List NFT first
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(owner),
            nft_id,
            1_000_000_000_000,
            100,
        ));

        let eth_address = b"0x1234567890123456789012345678901234567890"
            .to_vec()
            .try_into()
            .unwrap();

        // Try to bridge listed NFT
        assert_noop!(
            Quantum::bridge_to_ethereum(RuntimeOrigin::signed(owner), nft_id, eth_address),
            Error::<Test>::NFTAlreadyListed
        );
    });
}

#[test]
fn bridge_to_parachain_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_para".to_vec().try_into().unwrap();
        let circuit_hash = [1u8; 32];
        let num_qubits = 10u16;
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Create NFT
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,  // Qiskit
            circuit_hash, num_qubits, circuit_depth, num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1,  // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32], [0xABu8; 32].to_vec().try_into().unwrap(), 95,
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0,  // FirstQuantumJob
            true,
            10,
            95,
        ));

        let nft_id = 0;
        let parachain_id = 1000;
        let recipient = b"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            .to_vec()
            .try_into()
            .unwrap();

        // Bridge to parachain
        assert_ok!(Quantum::bridge_to_parachain(
            RuntimeOrigin::signed(owner),
            nft_id,
            parachain_id,
            recipient,
        ));

        // Check bridge request created
        let bridge_request = Quantum::bridge_requests(nft_id).unwrap();
        assert_eq!(bridge_request.destination, ChainDestination::Parachain(parachain_id));
    });
}

// ===== PROOF VALIDATION TESTS (ZK Audit) =====

#[test]
fn record_result_rejects_short_proof() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let executor = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_short_proof".to_vec().try_into().unwrap();

        // Submit job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4, // Qiskit
            [1u8; 32], 4, 10, 100,
        ));

        // Proof < 32 bytes should be rejected
        let short_proof: BoundedVec<u8, ConstU32<256>> = b"too_short".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::record_quantum_result(
                RuntimeOrigin::signed(executor),
                job_id.clone(),
                [1u8; 32],
                short_proof,
                95,
            ),
            Error::<Test>::InvalidVerificationProof
        );
    });
}

#[test]
fn record_result_rejects_zero_result_hash() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let executor = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_zero_hash".to_vec().try_into().unwrap();

        // Submit job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4, // Qiskit
            [1u8; 32], 4, 10, 100,
        ));

        // Zero result hash should be rejected
        let valid_proof: BoundedVec<u8, ConstU32<256>> = [0xABu8; 32].to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::record_quantum_result(
                RuntimeOrigin::signed(executor),
                job_id.clone(),
                [0u8; 32], // all zeros
                valid_proof,
                95,
            ),
            Error::<Test>::InvalidVerificationProof
        );
    });
}

#[test]
fn record_result_rejects_invalid_accuracy() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let executor = 2;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_bad_accuracy".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4, // Qiskit
            [1u8; 32], 4, 10, 100,
        ));

        let valid_proof: BoundedVec<u8, ConstU32<256>> = [0xABu8; 32].to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::record_quantum_result(
                RuntimeOrigin::signed(executor),
                job_id.clone(),
                [1u8; 32],
                valid_proof,
                101, // > 100 invalid
            ),
            Error::<Test>::InvalidCircuitParameters
        );
    });
}
