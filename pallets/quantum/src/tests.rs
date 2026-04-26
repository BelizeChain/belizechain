use crate::{
    mock::*, ChainDestination, Error, Event, JobStatus, NFTCategory, NFTRarity, QuantumResults,
};
use frame_support::{assert_noop, assert_ok, traits::Currency, BoundedVec};
use sp_core::ConstU32;

// ============================================================================
// DEBUG: Trace balance flow in buy_nft_works
// ============================================================================

#[test]
fn debug_buy_nft_balance_trace() {
    new_test_ext().execute_with(|| {
        let seller = 1u64;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_nft".to_vec().try_into().unwrap();

        println!(
            "A) free_balance(seller) = {}",
            Balances::free_balance(seller)
        );

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(seller),
            job_id.clone(),
            4,
            [1u8; 32],
            10u16,
            50u32,
            1000u32,
        ));
        println!(
            "B) after submit_quantum_job: free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );

        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(seller),
            job_id.clone(),
            1
        ));
        println!(
            "C) after update_job_status: free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));
        println!(
            "D) after record_quantum_result: free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(seller),
            job_id,
            0,
            true,
            10,
            95
        ));
        println!(
            "E) after mint_achievement_nft: free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );

        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(seller),
            0,
            1_000_000_000_000u128,
            100
        ));
        println!(
            "F) after list_nft (seller_initial): free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );

        assert_ok!(Quantum::buy_nft(RuntimeOrigin::signed(2), 0));
        println!(
            "G) after buy_nft: free={} reserved={}",
            Balances::free_balance(seller),
            Balances::reserved_balance(seller)
        );
    });
}

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
        let expected_cost =
            (num_qubits as u128 * DallaPerQubit::get()) + (num_shots as u128 * DallaPerShot::get());

        // Submit job first
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
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
            backend_index: 4, // Qiskit
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
            4, // Qiskit
            circuit_hash,
            10,   // num_qubits
            50,   // circuit_depth
            1000, // num_shots
        ));

        // Try to submit same job ID again
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(submitter),
                job_id,
                4, // Qiskit
                circuit_hash,
                num_qubits,
                circuit_depth,
                num_shots,
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
        let num_qubits = 100u16; // Max allowed
        let circuit_depth = 50u32;
        let num_shots = 1_000_000u32; // Max allowed = 1M shots
                                      // Cost = (100 * 1M) + (1M * 100K) = 100M + 100B = 100_100_000_000
                                      // This is only 100B, still less than 500T. Need to submit multiple times or increase Dave's cost
                                      // Better approach: use a user with very low balance
                                      // Change submitter to account with 100_000_000_000 balance (100B)
        let low_balance_account = 6; // New account with minimal balance

        // First give account 6 a small balance
        let _ = pallet_balances::Pallet::<Test>::deposit_creating(
            &low_balance_account,
            100_000_000_000,
        );

        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(low_balance_account),
                job_id,
                4, // Qiskit
                circuit_hash,
                num_qubits,
                circuit_depth,
                num_shots,
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running (required before recording result)
        // Record result (use 32-byte hash, not JSON string)
        let result_hash = [1u8; 32]; // Hash of actual result data
        let verification_proof: BoundedVec<u8, ConstU32<256>> =
            [0xABu8; 32].to_vec().try_into().unwrap();
        let accuracy = 95u8;

        let _executor_initial_balance = Balances::free_balance(executor);

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
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
        let verification_proof: BoundedVec<u8, ConstU32<256>> =
            [0xCDu8; 32].to_vec().try_into().unwrap();

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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
            // removed extra payment argument
        ));

        // Update job status to Running
        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        // Mint NFT
        let initial_balance = Balances::free_balance(submitter);

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            0,    // FirstQuantumJob
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
        let num_qubits = 100u16; // 100+ qubits → qubit_bonus = 300
        let circuit_depth = 50u32;
        let num_shots = 1000u32;

        // Submit and complete job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [2u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            100,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        // Mint high-quality NFT (should be Legendary)
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            2, // ShorAlgorithm // 500 base points
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
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
            0, // Approve
            90,
            result_hash,
        ));

        // Validator 2: Approve
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(4),
            job_id.clone(),
            0, // Approve
            95,
            result_hash,
        ));

        // Validator 3: Approve (triggers consensus)
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(5),
            job_id.clone(),
            0, // Approve
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running (use submitter, not validator)
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
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
            0, // Approve
            90,
            result_hash,
        ));

        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(4),
            job_id.clone(),
            0, // Approve
            95,
            result_hash,
        ));

        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(5),
            job_id.clone(),
            0, // Approve
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
            // removed extra payment argument
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0, // FirstQuantumJob
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
        assert!(events
            .iter()
            .any(|e| matches!(e.event, RuntimeEvent::Quantum(Event::NFTListed { .. }))));
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(seller),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(seller),
            job_id,
            0, // FirstQuantumJob
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

        // Check payment splits
        // royalty(5%) + marketplace_fee(2%) + seller_amount(93%) = 100%
        // original_minter == seller → royalty transferred back to seller
        // marketplace_fee goes to treasury
        // Buyer pays full price: seller_amount(93%) + marketplace_fee(2%) + royalty(5%) = 100%
        // Seller receives: seller_amount(93%) + royalty(5%) = 98%
        let royalty = price * 5 / 100;
        let marketplace_fee = price * 2 / 100;
        let seller_net = price - royalty - marketplace_fee; // 93% of price
        assert_eq!(
            Balances::free_balance(seller),
            seller_initial + seller_net + royalty
        );
        assert_eq!(
            Balances::free_balance(buyer),
            buyer_initial - seller_net - marketplace_fee - royalty
        );
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0, // FirstQuantumJob
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0, // FirstQuantumJob
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
                destination_index: 0, // Ethereum
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0, // FirstQuantumJob
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
            4, // Qiskit
            circuit_hash,
            num_qubits,
            circuit_depth,
            num_shots,
        ));

        // Update job status to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1, // Running
        ));

        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            [0xABu8; 32].to_vec().try_into().unwrap(),
            95,
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(owner),
            job_id,
            0, // FirstQuantumJob
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
        assert_eq!(
            bridge_request.destination,
            ChainDestination::Parachain(parachain_id)
        );
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
            [1u8; 32],
            4,
            10,
            100,
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
            [1u8; 32],
            4,
            10,
            100,
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
            [1u8; 32],
            4,
            10,
            100,
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

// ============================================================================
// UPDATE JOB STATUS TESTS
// ============================================================================

#[test]
fn update_job_status_running_to_completed_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_complete".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,
            [1u8; 32],
            4,
            10,
            100,
        ));
        // Pending → Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1
        ));
        // Running → Completed
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            2
        ));

        let job = Quantum::quantum_jobs(&job_id).expect("job must exist");
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completion_time.is_some());
    });
}

#[test]
fn update_job_status_running_to_failed_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_fail".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,
            [1u8; 32],
            4,
            10,
            100,
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            1
        ));
        // Running → Failed (refunds payment)
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            3
        ));

        let job = Quantum::quantum_jobs(&job_id).expect("job must exist");
        assert_eq!(job.status, JobStatus::Failed);
    });
}

#[test]
fn update_job_status_cancelled_works() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_cancel".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,
            [1u8; 32],
            4,
            10,
            100,
        ));
        // Pending → Cancelled (index 4 or any out-of-range maps to Cancelled)
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4
        ));

        let job = Quantum::quantum_jobs(&job_id).expect("job must exist");
        assert_eq!(job.status, JobStatus::Cancelled);
    });
}

#[test]
fn update_job_status_invalid_transition_fails() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_bad_trans".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(owner),
            job_id.clone(),
            4,
            [1u8; 32],
            4,
            10,
            100,
        ));
        // Pending → Completed is an invalid transition
        assert_noop!(
            Quantum::update_job_status(RuntimeOrigin::signed(owner), job_id.clone(), 2),
            Error::<Test>::InvalidStatusTransition
        );
    });
}

// ============================================================================
// TRANSFER NFT TESTS
// ============================================================================

/// Helper: mint NFT for account `owner` and return nft_id 0.
fn setup_nft(owner: u64) {
    let job_id: BoundedVec<u8, ConstU32<64>> = b"job_for_nft".to_vec().try_into().unwrap();
    assert_ok!(Quantum::submit_quantum_job(
        RuntimeOrigin::signed(owner),
        job_id.clone(),
        4,
        [1u8; 32],
        4,
        10,
        100,
    ));
    assert_ok!(Quantum::update_job_status(
        RuntimeOrigin::signed(owner),
        job_id.clone(),
        1
    ));
    assert_ok!(Quantum::record_quantum_result(
        RuntimeOrigin::signed(2),
        job_id.clone(),
        [1u8; 32],
        [0xABu8; 32].to_vec().try_into().unwrap(),
        95,
    ));
    assert_ok!(Quantum::verify_quantum_result(
        RuntimeOrigin::root(),
        job_id.clone(),
        true
    ));
    assert_ok!(Quantum::mint_achievement_nft(
        RuntimeOrigin::signed(owner),
        job_id,
        0, // FirstQuantumJob
        true,
        10,
        95,
    ));
}

#[test]
fn transfer_nft_works() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        let recipient = 3u64;
        setup_nft(owner);

        let nft_id = 0u64;
        assert_eq!(Quantum::quantum_achievements(nft_id).unwrap().owner, owner);

        assert_ok!(Quantum::transfer_nft(
            RuntimeOrigin::signed(owner),
            nft_id,
            recipient
        ));

        // Ownership must have changed.
        assert_eq!(
            Quantum::quantum_achievements(nft_id).unwrap().owner,
            recipient
        );
    });
}

#[test]
fn transfer_nft_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        setup_nft(owner);
        let nft_id = 0u64;

        // Account 3 does not own this NFT.
        assert_noop!(
            Quantum::transfer_nft(RuntimeOrigin::signed(3), nft_id, 4),
            Error::<Test>::NotAuthorized
        );
    });
}

// ============================================================================
// DELIST NFT TESTS
// ============================================================================

#[test]
fn delist_nft_works() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        setup_nft(owner);
        let nft_id = 0u64;

        // List the NFT.
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(owner),
            nft_id,
            1_000_000_000_000u128,
            100,
        ));
        assert!(Quantum::nft_listings(nft_id).is_some());

        // Delist it.
        assert_ok!(Quantum::delist_nft(RuntimeOrigin::signed(owner), nft_id));
        assert!(Quantum::nft_listings(nft_id).is_none());
    });
}

#[test]
fn delist_nft_not_listed_fails() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        setup_nft(owner);
        let nft_id = 0u64;

        // NFT is not listed — delisting must fail.
        assert_noop!(
            Quantum::delist_nft(RuntimeOrigin::signed(owner), nft_id),
            Error::<Test>::ListingNotFound
        );
    });
}

// ============================================================================
// CANCEL BRIDGE TESTS
// ============================================================================

#[test]
fn cancel_bridge_works() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        setup_nft(owner);
        let nft_id = 0u64;

        let eth_address: BoundedVec<u8, ConstU32<64>> =
            b"0x1234567890123456789012345678901234567890"
                .to_vec()
                .try_into()
                .unwrap();

        // Initiate bridge — NFT becomes locked.
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(owner),
            nft_id,
            eth_address
        ));
        assert!(Quantum::bridge_requests(nft_id).is_some());

        // Cancel bridge — NFT must be unlocked and request removed.
        assert_ok!(Quantum::cancel_bridge(RuntimeOrigin::signed(owner), nft_id));
        assert!(Quantum::bridge_requests(nft_id).is_none());
        // NFT should be transferable again.
        assert!(Quantum::quantum_achievements(nft_id).unwrap().transferable);
    });
}

#[test]
fn cancel_bridge_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let owner = 1u64;
        setup_nft(owner);
        let nft_id = 0u64;

        let eth_address: BoundedVec<u8, ConstU32<64>> =
            b"0x1234567890123456789012345678901234567890"
                .to_vec()
                .try_into()
                .unwrap();

        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(owner),
            nft_id,
            eth_address
        ));

        // Account 3 did not initiate the bridge.
        assert_noop!(
            Quantum::cancel_bridge(RuntimeOrigin::signed(3), nft_id),
            Error::<Test>::NotAuthorized
        );
    });
}

// ============================================================================
// Root Verification Override Tests
// ============================================================================

#[test]
fn verify_quantum_result_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1u64;
        let executor = 2u64;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_verify_root".to_vec().try_into().unwrap();

        // Submit job
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4, // Qiskit
            [1u8; 32],
            10u16,
            50u32,
            1000u32,
        ));

        // Advance to Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        // Record result from executor
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xABu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(executor),
            job_id.clone(),
            [1u8; 32],
            proof,
            95u8,
        ));

        // Root verifies the result as passed
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true,
        ));

        let job = Quantum::quantum_jobs(&job_id).unwrap();
        assert_eq!(job.verification_status.to_index(), 2); // Verified
    });
}

#[test]
fn verify_quantum_result_failed_works() {
    new_test_ext().execute_with(|| {
        let submitter = 1u64;
        let executor = 2u64;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"job_verify_fail".to_vec().try_into().unwrap();

        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            4,
            [2u8; 32],
            8u16,
            30u32,
            500u32,
        ));

        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1, // Running
        ));

        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xCDu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(executor),
            job_id.clone(),
            [2u8; 32],
            proof,
            40u8,
        ));

        // Root marks verification as failed
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            false,
        ));

        let job = Quantum::quantum_jobs(&job_id).unwrap();
        assert_eq!(job.verification_status.to_index(), 3); // Failed
    });
}

#[test]
fn verify_quantum_result_no_result_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"no_result_job".to_vec().try_into().unwrap();

        // No result recorded — should fail with ResultNotFound
        assert_noop!(
            Quantum::verify_quantum_result(RuntimeOrigin::root(), job_id, true),
            Error::<Test>::ResultNotFound
        );
    });
}

// ============================================================================
// Extended Error Path & Edge Case Tests
// ============================================================================

#[test]
fn submit_job_zero_qubits_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"zero_qubits".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                0,
                [1u8; 32],
                0u16,
                10,
                100
            ),
            Error::<Test>::InvalidCircuitParameters
        );
    });
}

#[test]
fn submit_job_too_many_qubits_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"many_qubits".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                0,
                [1u8; 32],
                101u16,
                10,
                100
            ),
            Error::<Test>::InvalidCircuitParameters
        );
    });
}

#[test]
fn submit_job_zero_shots_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"zero_shots".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                0,
                [1u8; 32],
                10u16,
                10,
                0u32
            ),
            Error::<Test>::InvalidCircuitParameters
        );
    });
}

#[test]
fn submit_job_too_many_shots_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"max_shots".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                0,
                [1u8; 32],
                10u16,
                10,
                1_000_001u32
            ),
            Error::<Test>::InvalidCircuitParameters
        );
    });
}

#[test]
fn submit_job_boundary_100_qubits_works() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"boundary_q".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            [1u8; 32],
            100u16,
            10,
            100
        ));
    });
}

#[test]
fn submit_job_boundary_1m_shots_works() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"boundary_s".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            [1u8; 32],
            10u16,
            10,
            1_000_000u32
        ));
    });
}

#[test]
fn submit_job_updates_account_stats() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"stats_job".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id,
            4,
            [1u8; 32],
            10u16,
            50,
            1000
        ));

        let stats = Quantum::account_stats(1);
        assert_eq!(stats.total_jobs, 1);
        assert_eq!(stats.total_qubits, 10);
        assert_eq!(stats.total_shots, 1000);
        assert!(stats.total_spent > 0);
    });
}

#[test]
fn submit_job_increments_total_jobs() {
    new_test_ext().execute_with(|| {
        assert_eq!(Quantum::total_quantum_jobs(), 0);
        let job_id1: BoundedVec<u8, ConstU32<64>> = b"j1".to_vec().try_into().unwrap();
        let job_id2: BoundedVec<u8, ConstU32<64>> = b"j2".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id1,
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(2),
            job_id2,
            0,
            [2u8; 32],
            5,
            10,
            100
        ));
        assert_eq!(Quantum::total_quantum_jobs(), 2);
    });
}

#[test]
fn submit_job_reserves_correct_amount() {
    new_test_ext().execute_with(|| {
        let initial = Balances::free_balance(1);
        let job_id: BoundedVec<u8, ConstU32<64>> = b"reserve_job".to_vec().try_into().unwrap();
        // cost = 10 * 1_000_000 + 1000 * 100_000 = 10_000_000 + 100_000_000 = 110_000_000
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            [1u8; 32],
            10u16,
            50,
            1000
        ));
        assert_eq!(Balances::reserved_balance(1), 110_000_000);
        assert_eq!(Balances::free_balance(1), initial - 110_000_000);
    });
}

#[test]
fn update_job_status_not_authorized_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"auth_test".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        // Account 3 is neither submitter nor executor
        assert_noop!(
            Quantum::update_job_status(RuntimeOrigin::signed(3), job_id, 1),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn update_job_failed_refunds_cost() {
    new_test_ext().execute_with(|| {
        let initial = Balances::free_balance(1);
        let job_id: BoundedVec<u8, ConstU32<64>> = b"refund_job".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            10,
            50,
            1000
        ));
        let reserved = Balances::reserved_balance(1);
        assert!(reserved > 0);

        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        )); // Running
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id,
            3
        )); // Failed

        // Funds should be refunded
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::free_balance(1), initial);
    });
}

#[test]
fn update_job_cancelled_from_pending_refunds() {
    new_test_ext().execute_with(|| {
        let initial = Balances::free_balance(1);
        let job_id: BoundedVec<u8, ConstU32<64>> = b"cancel_job".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id,
            4
        )); // Cancelled
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::free_balance(1), initial);
    });
}

#[test]
fn update_completed_job_cannot_be_cancelled() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"done_cancel".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            2
        ));
        // Audit fix: completed jobs cannot be cancelled
        assert_noop!(
            Quantum::update_job_status(RuntimeOrigin::signed(1), job_id, 4),
            Error::<Test>::InvalidStatusTransition
        );
    });
}

#[test]
fn record_result_already_recorded_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"dup_result".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xABu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof.clone(),
            90
        ));
        assert_noop!(
            Quantum::record_quantum_result(RuntimeOrigin::signed(2), job_id, [1u8; 32], proof, 90),
            Error::<Test>::ResultAlreadyRecorded
        );
    });
}

#[test]
fn mint_nft_charges_fee() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"fee_nft".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        let before = Balances::free_balance(1);
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            true,
            5,
            95
        ));
        let after = Balances::free_balance(1);
        // NFTMintingFee = 500_000_000_000
        assert_eq!(before - after, 500_000_000_000);
    });
}

#[test]
fn mint_nft_not_authorized_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"auth_nft".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));

        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));

        // Account 3 is neither submitter nor executor
        assert_noop!(
            Quantum::mint_achievement_nft(RuntimeOrigin::signed(3), job_id, 0, true, 5, 95),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn transfer_nnt_non_transferable_fails() {
    new_test_ext().execute_with(|| {
        // Create non-transferable NFT
        let job_id: BoundedVec<u8, ConstU32<64>> = b"notr_nft".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            false,
            5,
            95 // transferable = false
        ));

        assert_noop!(
            Quantum::transfer_nft(RuntimeOrigin::signed(1), 0, 2),
            Error::<Test>::NFTNotTransferable
        );
    });
}

#[test]
fn transfer_nft_locked_in_bridge_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        let eth_addr: BoundedVec<u8, ConstU32<64>> = b"0x1234567890123456789012345678901234567890"
            .to_vec()
            .try_into()
            .unwrap();
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(1),
            0,
            eth_addr
        ));

        assert_noop!(
            Quantum::transfer_nft(RuntimeOrigin::signed(1), 0, 2),
            Error::<Test>::NFTLockedInBridge
        );
    });
}

#[test]
fn transfer_nft_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Quantum::transfer_nft(RuntimeOrigin::signed(1), 999, 2),
            Error::<Test>::NFTNotFound
        );
    });
}

#[test]
fn list_nft_not_transferable_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"list_notr".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id.clone(),
            true
        ));
        assert_ok!(Quantum::mint_achievement_nft(
            RuntimeOrigin::signed(1),
            job_id,
            0,
            false,
            5,
            95
        ));
        assert_noop!(
            Quantum::list_nft(RuntimeOrigin::signed(1), 0, 1_000_000, 100),
            Error::<Test>::NFTNotTransferable
        );
    });
}

#[test]
fn list_nft_already_listed_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        assert_ok!(Quantum::list_nft(RuntimeOrigin::signed(1), 0, 1_000, 100));
        assert_noop!(
            Quantum::list_nft(RuntimeOrigin::signed(1), 0, 2_000, 100),
            Error::<Test>::NFTAlreadyListed
        );
    });
}

#[test]
fn list_nft_not_owner_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        assert_noop!(
            Quantum::list_nft(RuntimeOrigin::signed(2), 0, 1_000, 100),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn buy_nft_listing_expired_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(1),
            0,
            1_000_000_000_000,
            10
        ));
        // Advance past the expiry
        run_to_block(15);
        assert_noop!(
            Quantum::buy_nft(RuntimeOrigin::signed(2), 0),
            Error::<Test>::ListingExpired
        );
    });
}

#[test]
fn buy_nft_listing_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Quantum::buy_nft(RuntimeOrigin::signed(2), 999),
            Error::<Test>::ListingNotFound
        );
    });
}

#[test]
fn buy_nft_transfers_ownership() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        assert_ok!(Quantum::list_nft(
            RuntimeOrigin::signed(1),
            0,
            1_000_000_000_000,
            100
        ));
        assert_ok!(Quantum::buy_nft(RuntimeOrigin::signed(2), 0));
        assert_eq!(Quantum::quantum_achievements(0).unwrap().owner, 2);
        // Listing should be removed
        assert!(Quantum::nft_listings(0).is_none());
    });
}

#[test]
fn delist_nft_not_seller_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        assert_ok!(Quantum::list_nft(RuntimeOrigin::signed(1), 0, 1_000, 100));
        assert_noop!(
            Quantum::delist_nft(RuntimeOrigin::signed(2), 0),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn bridge_to_ethereum_invalid_address_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        // address not 20 or 42 bytes
        let bad_addr: BoundedVec<u8, ConstU32<64>> = b"0x1234".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::bridge_to_ethereum(RuntimeOrigin::signed(1), 0, bad_addr),
            Error::<Test>::InvalidRecipientAddress
        );
    });
}

#[test]
fn bridge_already_initiated_blocked_by_non_transferable() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        let eth_addr: BoundedVec<u8, ConstU32<64>> = b"0x1234567890123456789012345678901234567890"
            .to_vec()
            .try_into()
            .unwrap();
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(1),
            0,
            eth_addr.clone()
        ));
        // First bridge marks NFT non-transferable, so second attempt fails with that error
        assert_noop!(
            Quantum::bridge_to_ethereum(RuntimeOrigin::signed(1), 0, eth_addr),
            Error::<Test>::NFTNotTransferable
        );
    });
}

#[test]
fn bridge_locks_nft_not_transferable() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        let eth_addr: BoundedVec<u8, ConstU32<64>> = b"0x1234567890123456789012345678901234567890"
            .to_vec()
            .try_into()
            .unwrap();
        assert!(Quantum::quantum_achievements(0).unwrap().transferable);
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(1),
            0,
            eth_addr
        ));
        assert!(!Quantum::quantum_achievements(0).unwrap().transferable);
    });
}

#[test]
fn cancel_bridge_request_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Quantum::cancel_bridge(RuntimeOrigin::signed(1), 999),
            Error::<Test>::BridgeRequestNotFound
        );
    });
}

#[test]
fn verify_result_non_root_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"noroot".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::verify_quantum_result(RuntimeOrigin::signed(1), job_id, true),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn request_verification_works() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"req_verify".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            3
        ));
        assert!(Quantum::verification_requests(&job_id).is_some());
    });
}

#[test]
fn request_verification_not_submitter_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"nosubmit".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_noop!(
            Quantum::request_verification(RuntimeOrigin::signed(3), job_id, 3),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn request_verification_no_result_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"no_res_v".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_noop!(
            Quantum::request_verification(RuntimeOrigin::signed(1), job_id, 3),
            Error::<Test>::ResultNotFound
        );
    });
}

#[test]
fn request_verification_duplicate_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"dup_vreq".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            3
        ));
        assert_noop!(
            Quantum::request_verification(RuntimeOrigin::signed(1), job_id, 3),
            Error::<Test>::VerificationRequestAlreadyExists
        );
    });
}

#[test]
fn submit_verification_invalid_confidence_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"bad_conf".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            3
        ));
        assert_noop!(
            Quantum::submit_verification(RuntimeOrigin::signed(3), job_id, 0, 101, [1u8; 32]),
            Error::<Test>::InvalidVerificationVote
        );
    });
}

#[test]
fn submit_verification_no_request_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"noreq_v".to_vec().try_into().unwrap();
        // Create job and record result but skip request_verification
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));
        // No request_verification call — should fail
        assert_noop!(
            Quantum::submit_verification(RuntimeOrigin::signed(3), job_id, 0, 80, [1u8; 32]),
            Error::<Test>::VerificationRequestNotFound
        );
    });
}

#[test]
fn submit_verification_deadline_passed_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"deadline_v".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            3
        ));
        // Deadline is 100 blocks from creation
        run_to_block(105);
        assert_noop!(
            Quantum::submit_verification(RuntimeOrigin::signed(3), job_id, 0, 80, [1u8; 32]),
            Error::<Test>::VerificationDeadlinePassed
        );
    });
}

#[test]
fn submit_verification_duplicate_validator_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"dup_val".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            90
        ));
        assert_ok!(Quantum::request_verification(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            3
        ));
        assert_ok!(Quantum::submit_verification(
            RuntimeOrigin::signed(3),
            job_id.clone(),
            0,
            80,
            [1u8; 32]
        ));
        assert_noop!(
            Quantum::submit_verification(RuntimeOrigin::signed(3), job_id, 0, 80, [1u8; 32]),
            Error::<Test>::ValidatorAlreadyVerified
        );
    });
}

#[test]
fn nft_counter_increments() {
    new_test_ext().execute_with(|| {
        assert_eq!(Quantum::nft_counter(), 0);
        setup_nft(1);
        assert_eq!(Quantum::nft_counter(), 1);
    });
}

#[test]
fn bridge_to_parachain_not_owner_fails() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        let recipient: BoundedVec<u8, ConstU32<64>> = b"5FHneW46".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::bridge_to_parachain(RuntimeOrigin::signed(3), 0, 1000, recipient),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn multiple_jobs_by_same_account_tracked() {
    new_test_ext().execute_with(|| {
        for i in 0..5u8 {
            let job_id: BoundedVec<u8, ConstU32<64>> =
                format!("multi_job_{i}").into_bytes().try_into().unwrap();
            assert_ok!(Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                0,
                [i; 32],
                5,
                10,
                100
            ));
        }
        let jobs = Quantum::jobs_by_account(1);
        assert_eq!(jobs.len(), 5);
        let stats = Quantum::account_stats(1);
        assert_eq!(stats.total_jobs, 5);
    });
}

#[test]
fn total_dalla_spent_accumulates() {
    new_test_ext().execute_with(|| {
        assert_eq!(Quantum::total_dalla_spent(), 0);
        let j1: BoundedVec<u8, ConstU32<64>> = b"spend1".to_vec().try_into().unwrap();
        let j2: BoundedVec<u8, ConstU32<64>> = b"spend2".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            j1,
            0,
            [1u8; 32],
            10,
            10,
            1000
        ));
        let spent1 = Quantum::total_dalla_spent();
        assert!(spent1 > 0);
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(2),
            j2,
            0,
            [2u8; 32],
            10,
            10,
            1000
        ));
        assert_eq!(Quantum::total_dalla_spent(), spent1 * 2);
    });
}

#[test]
fn all_backend_indices_work() {
    new_test_ext().execute_with(|| {
        // All backend indices 0-7 should be valid
        for backend in 0..8u8 {
            let job_id: BoundedVec<u8, ConstU32<64>> = format!("backend_{backend}")
                .into_bytes()
                .try_into()
                .unwrap();
            assert_ok!(Quantum::submit_quantum_job(
                RuntimeOrigin::signed(1),
                job_id,
                backend,
                [backend; 32],
                5,
                10,
                100
            ));
        }
        assert_eq!(Quantum::total_quantum_jobs(), 8);
    });
}

#[test]
fn update_job_status_job_not_found_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"ghost_job".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::update_job_status(RuntimeOrigin::signed(1), job_id, 1),
            Error::<Test>::JobNotFound
        );
    });
}

#[test]
fn record_result_job_not_running_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"not_running".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            5,
            10,
            100
        ));
        // Job is Pending, not Running
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_noop!(
            Quantum::record_quantum_result(RuntimeOrigin::signed(2), job_id, [1u8; 32], proof, 90),
            Error::<Test>::InvalidStatusTransition
        );
    });
}

#[test]
fn mint_nft_job_not_found_fails() {
    new_test_ext().execute_with(|| {
        let job_id: BoundedVec<u8, ConstU32<64>> = b"no_such_job".to_vec().try_into().unwrap();
        assert_noop!(
            Quantum::mint_achievement_nft(RuntimeOrigin::signed(1), job_id, 0, true, 5, 95),
            Error::<Test>::JobNotFound
        );
    });
}

#[test]
fn bridge_to_ethereum_20_byte_address_works() {
    new_test_ext().execute_with(|| {
        setup_nft(1);
        // 20-byte raw address (not hex-encoded)
        let raw_addr: BoundedVec<u8, ConstU32<64>> = vec![0xABu8; 20].try_into().unwrap();
        assert_ok!(Quantum::bridge_to_ethereum(
            RuntimeOrigin::signed(1),
            0,
            raw_addr
        ));
    });
}

#[test]
fn verify_result_pays_executor() {
    new_test_ext().execute_with(|| {
        let executor = 2u64;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"pay_exec".to_vec().try_into().unwrap();
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            0,
            [1u8; 32],
            10,
            50,
            1000
        ));
        let reserved = Balances::reserved_balance(1);
        assert!(reserved > 0);
        let exec_before = Balances::free_balance(executor);

        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(1),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(executor),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));
        // Root verifies — executor gets the reserved funds
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id,
            true
        ));

        assert_eq!(Balances::reserved_balance(1), 0);
        let exec_after = Balances::free_balance(executor);
        assert_eq!(exec_after, exec_before + reserved);
    });
}

#[test]
fn verify_result_failed_refunds_submitter() {
    new_test_ext().execute_with(|| {
        let submitter = 1u64;
        let job_id: BoundedVec<u8, ConstU32<64>> = b"refund_v".to_vec().try_into().unwrap();
        let initial = Balances::free_balance(submitter);
        assert_ok!(Quantum::submit_quantum_job(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            0,
            [1u8; 32],
            10,
            50,
            1000
        ));
        assert_ok!(Quantum::update_job_status(
            RuntimeOrigin::signed(submitter),
            job_id.clone(),
            1
        ));
        let proof: BoundedVec<u8, ConstU32<256>> = vec![0xAAu8; 32].try_into().unwrap();
        assert_ok!(Quantum::record_quantum_result(
            RuntimeOrigin::signed(2),
            job_id.clone(),
            [1u8; 32],
            proof,
            95
        ));
        // Verification fails — submitter gets refund
        assert_ok!(Quantum::verify_quantum_result(
            RuntimeOrigin::root(),
            job_id,
            false
        ));
        assert_eq!(Balances::free_balance(submitter), initial);
        assert_eq!(Balances::reserved_balance(submitter), 0);
    });
}
