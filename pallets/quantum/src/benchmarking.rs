//! Benchmarks for pallet-belize-quantum (v2 API)
//!
//! Covers 14 WeightInfo functions:
//!   submit_quantum_job, update_job_status, record_quantum_result,
//!   verify_quantum_result (Root), mint_achievement_nft,
//!   transfer_nft, list_nft, buy_nft, delist_nft,
//!   bridge_to_ethereum, bridge_to_parachain, cancel_bridge,
//!   request_verification, submit_verification

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::traits::Currency;
use sp_runtime::BoundedVec;
use sp_std::vec;

const MAX_JOB_ID_LEN: u32 = 64;

/// Create a funded account for benchmarks
fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, 0);
    let amount = T::Currency::minimum_balance().saturating_mul(1_000_000u32.into());
    T::Currency::make_free_balance_be(&caller, amount);
    caller
}

/// Create a valid job_id BoundedVec
fn make_job_id<T: Config>(seed: u8) -> BoundedVec<u8, ConstU32<MAX_JOB_ID_LEN>> {
    let mut id = vec![b'j', b'o', b'b', b'-'];
    id.push(b'0' + (seed % 10));
    BoundedVec::try_from(id).expect("job id within bounds")
}

/// Insert a quantum job directly into storage, bypassing extrinsic validation.
fn insert_quantum_job<T: Config>(
    submitter: &T::AccountId,
    job_id: &BoundedVec<u8, ConstU32<MAX_JOB_ID_LEN>>,
) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let cost = T::Currency::minimum_balance().saturating_mul(100u32.into());

    let job = QuantumJob {
        job_id: job_id.clone(),
        submitter: submitter.clone(),
        backend: QuantumBackend::Qiskit,
        circuit_hash: [1u8; 32],
        num_qubits: 5,
        circuit_depth: 10,
        num_shots: 100,
        status: JobStatus::Pending,
        submission_time: current_block,
        completion_time: None,
        result_hash: None,
        verification_status: VerificationStatus::Unverified,
        dalla_cost: cost,
        executor: None,
    };

    QuantumJobs::<T>::insert(job_id, job);
    let mut jobs = JobsByAccount::<T>::get(submitter);
    let _ = jobs.try_push(job_id.clone());
    JobsByAccount::<T>::insert(submitter, jobs);

    // Reserve funds to match extrinsic behaviour
    let _ = T::Currency::reserve(submitter, cost);
}

/// Insert a quantum result for a job
fn insert_quantum_result<T: Config>(
    executor: &T::AccountId,
    job_id: &BoundedVec<u8, ConstU32<MAX_JOB_ID_LEN>>,
) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let proof: BoundedVec<u8, ConstU32<256>> =
        BoundedVec::try_from(vec![0u8; 32]).expect("proof within bounds");

    let result = QuantumResult {
        job_id: job_id.clone(),
        result_data_hash: [2u8; 32],
        verification_proof: proof,
        accuracy_score: 95,
        validator: BoundedVec::try_from(vec![1u8; 32]).unwrap_or_default(),
        recorded_at: current_block.saturated_into::<u32>(),
    };

    QuantumResults::<T>::insert(job_id, result);

    // Mark job as completed with executor
    QuantumJobs::<T>::mutate(job_id, |maybe_job| {
        if let Some(job) = maybe_job {
            job.status = JobStatus::Completed;
            job.executor = Some(executor.clone());
            job.completion_time = Some(current_block);
        }
    });
}

/// Insert a QuantumAchievement (NFT) directly into storage
fn insert_nft<T: Config>(owner: &T::AccountId, nft_id: u64, transferable: bool) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let metadata: BoundedVec<u8, ConstU32<256>> =
        BoundedVec::try_from(b"ipfs://benchmark".to_vec()).expect("metadata within bounds");

    let nft = QuantumAchievement {
        nft_id,
        job_id: BoundedVec::try_from(b"bench-nft".to_vec()).unwrap_or_default(),
        achievement_type: AchievementType::FirstQuantumJob,
        owner: owner.clone(),
        original_minter: owner.clone(), // Q-2 FIX
        metadata_uri: metadata,
        minted_at: current_block,
        transferable,
        rarity: NFTRarity::Common,
        category: NFTCategory::Special,
        rarity_score: 100,
        circuit_qubits: 5,
        accuracy: 90,
    };

    QuantumAchievements::<T>::insert(nft_id, nft);
    NFTCounter::<T>::put(nft_id + 1);
}

/// Insert an NFT listing directly into storage
fn insert_listing<T: Config>(
    nft_id: u64,
    seller: &T::AccountId,
    price: <T::Currency as Currency<T::AccountId>>::Balance,
) {
    let current_block = frame_system::Pallet::<T>::block_number();
    // SAFETY(saturated_into): benchmark helper; expiry = current + 1000 blocks.
    let expiry_u64: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0) + 1000u64;
    let expiry: BlockNumberFor<T> = expiry_u64.saturated_into();

    let listing = NFTListing {
        nft_id,
        original_minter: seller.clone(),
        seller: seller.clone(),
        price,
        expiry,
        listed_at: current_block,
    };

    NFTListings::<T>::insert(nft_id, listing);
    ListingCounter::<T>::mutate(|c| *c = c.saturating_add(1));
}

/// Insert a bridge request directly into storage
fn insert_bridge<T: Config>(nft_id: u64, owner: &T::AccountId) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let recipient: BoundedVec<u8, ConstU32<64>> =
        BoundedVec::try_from(vec![0xABu8; 20]).expect("recipient within bounds");

    let request = BridgeRequest {
        nft_id,
        owner: owner.clone(),
        destination: ChainDestination::Ethereum,
        recipient,
        requested_at: current_block,
        claimed: false,
        claim_tx_hash: None,
    };

    BridgeRequests::<T>::insert(nft_id, request);
    BridgeCounter::<T>::mutate(|c| *c = c.saturating_add(1));

    // Lock NFT
    QuantumAchievements::<T>::mutate(nft_id, |maybe_nft| {
        if let Some(nft) = maybe_nft {
            nft.transferable = false;
        }
    });
}

/// Insert a verification request for a job
fn insert_verification_request<T: Config>(
    job_id: &BoundedVec<u8, ConstU32<MAX_JOB_ID_LEN>>,
) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let created: u32 = current_block.saturated_into::<u32>();

    let request = VerificationRequest {
        job_id: job_id.clone(),
        required_verifications: 3,
        verifications: BoundedVec::default(),
        approvals: 0,
        rejections: 0,
        consensus_reached: false,
        consensus_result: None,
        created_at: created,
        deadline: created + 1000,
    };

    VerificationRequests::<T>::insert(job_id, request);
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_quantum_job() {
        let caller = funded_account::<T>("submitter", 0);
        let job_id = make_job_id::<T>(1);

        #[extrinsic_call]
        submit_quantum_job(
            RawOrigin::Signed(caller),
            job_id,
            0u8,         // backend_index: Simulator
            [1u8; 32],   // circuit_hash
            5u16,        // num_qubits
            10u32,       // circuit_depth
            100u32,      // num_shots
        );
    }

    #[benchmark]
    fn update_job_status() {
        let caller = funded_account::<T>("submitter", 0);
        let job_id = make_job_id::<T>(2);
        insert_quantum_job::<T>(&caller, &job_id);

        #[extrinsic_call]
        update_job_status(
            RawOrigin::Signed(caller),
            job_id,
            1u8,  // status_index: Running
        );
    }

    #[benchmark]
    fn record_quantum_result() {
        let submitter = funded_account::<T>("submitter", 0);
        let executor = funded_account::<T>("executor", 1);
        let job_id = make_job_id::<T>(3);
        insert_quantum_job::<T>(&submitter, &job_id);

        // Set executor reputation to required minimum (100)
        ValidatorReputation::<T>::insert(&executor, 100u32);

        // Set job status to Running with executor
        QuantumJobs::<T>::mutate(&job_id, |maybe_job| {
            if let Some(job) = maybe_job {
                job.status = JobStatus::Running;
                job.executor = Some(executor.clone());
            }
        });

        let proof: BoundedVec<u8, ConstU32<256>> =
            BoundedVec::try_from(vec![0u8; 32]).expect("proof within bounds");

        #[extrinsic_call]
        record_quantum_result(
            RawOrigin::Signed(executor),
            job_id,
            [2u8; 32],  // result_data_hash
            proof,       // verification_proof
            95u8,        // accuracy_score
        );
    }

    // Root-only extrinsic
    #[benchmark]
    fn verify_quantum_result() {
        let submitter = funded_account::<T>("submitter", 0);
        let executor = funded_account::<T>("executor", 1);
        let job_id = make_job_id::<T>(4);
        insert_quantum_job::<T>(&submitter, &job_id);
        insert_quantum_result::<T>(&executor, &job_id);

        #[extrinsic_call]
        verify_quantum_result(
            RawOrigin::Root,
            job_id,
            true,  // verification_passed
        );
    }

    #[benchmark]
    fn mint_achievement_nft() {
        let caller = funded_account::<T>("minter", 0);
        let executor = funded_account::<T>("executor", 1);
        let job_id = make_job_id::<T>(5);
        insert_quantum_job::<T>(&caller, &job_id);
        insert_quantum_result::<T>(&executor, &job_id);

        // Mark job as verified (required for minting)
        QuantumJobs::<T>::mutate(&job_id, |maybe_job| {
            if let Some(job) = maybe_job {
                job.verification_status = VerificationStatus::Verified;
            }
        });

        // Ensure NFTCounter starts at 0
        NFTCounter::<T>::put(0u64);

        #[extrinsic_call]
        mint_achievement_nft(
            RawOrigin::Signed(caller),
            job_id,
            0u8,    // achievement_type_index: FirstQuantumJob
            true,   // transferable
            5u16,   // circuit_qubits
            90u8,   // accuracy
        );
    }

    #[benchmark]
    fn transfer_nft() {
        let owner = funded_account::<T>("owner", 0);
        let recipient: T::AccountId = account("recipient", 1, 0);
        let nft_id = 100u64;
        insert_nft::<T>(&owner, nft_id, true);

        #[extrinsic_call]
        transfer_nft(RawOrigin::Signed(owner), nft_id, recipient);
    }

    #[benchmark]
    fn list_nft() {
        let seller = funded_account::<T>("seller", 0);
        let nft_id = 200u64;
        insert_nft::<T>(&seller, nft_id, true);

        let price = T::Currency::minimum_balance().saturating_mul(50u32.into());
        let duration: BlockNumberFor<T> = 1000u32.into();

        #[extrinsic_call]
        list_nft(RawOrigin::Signed(seller), nft_id, price, duration);
    }

    #[benchmark]
    fn buy_nft() {
        let seller = funded_account::<T>("seller", 0);
        let buyer = funded_account::<T>("buyer", 1);
        let nft_id = 300u64;
        insert_nft::<T>(&seller, nft_id, true);

        let price = T::Currency::minimum_balance().saturating_mul(50u32.into());
        insert_listing::<T>(nft_id, &seller, price);

        #[extrinsic_call]
        buy_nft(RawOrigin::Signed(buyer), nft_id);
    }

    #[benchmark]
    fn delist_nft() {
        let seller = funded_account::<T>("seller", 0);
        let nft_id = 400u64;
        insert_nft::<T>(&seller, nft_id, true);

        let price = T::Currency::minimum_balance().saturating_mul(50u32.into());
        insert_listing::<T>(nft_id, &seller, price);

        #[extrinsic_call]
        delist_nft(RawOrigin::Signed(seller), nft_id);
    }

    #[benchmark]
    fn bridge_to_ethereum() {
        let owner = funded_account::<T>("owner", 0);
        let nft_id = 500u64;
        insert_nft::<T>(&owner, nft_id, true);

        // 20-byte Ethereum address
        let recipient: BoundedVec<u8, ConstU32<64>> =
            BoundedVec::try_from(vec![0xABu8; 20]).expect("recipient within bounds");

        #[extrinsic_call]
        bridge_to_ethereum(RawOrigin::Signed(owner), nft_id, recipient);
    }

    #[benchmark]
    fn bridge_to_parachain() {
        let owner = funded_account::<T>("owner", 0);
        let nft_id = 600u64;
        insert_nft::<T>(&owner, nft_id, true);

        let recipient: BoundedVec<u8, ConstU32<64>> =
            BoundedVec::try_from(vec![0xCDu8; 32]).expect("recipient within bounds");

        #[extrinsic_call]
        bridge_to_parachain(RawOrigin::Signed(owner), nft_id, 2000u32, recipient);
    }

    #[benchmark]
    fn cancel_bridge() {
        let owner = funded_account::<T>("owner", 0);
        let nft_id = 700u64;
        insert_nft::<T>(&owner, nft_id, true);
        insert_bridge::<T>(nft_id, &owner);

        #[extrinsic_call]
        cancel_bridge(RawOrigin::Signed(owner), nft_id);
    }

    #[benchmark]
    fn request_verification() {
        let submitter = funded_account::<T>("submitter", 0);
        let executor = funded_account::<T>("executor", 1);
        let job_id = make_job_id::<T>(6);
        insert_quantum_job::<T>(&submitter, &job_id);
        insert_quantum_result::<T>(&executor, &job_id);

        #[extrinsic_call]
        request_verification(
            RawOrigin::Signed(submitter),
            job_id,
            3u8,  // required_verifications
        );
    }

    #[benchmark]
    fn submit_verification() {
        let submitter = funded_account::<T>("submitter", 0);
        let executor = funded_account::<T>("executor", 1);
        let validator = funded_account::<T>("validator", 2);
        let job_id = make_job_id::<T>(7);
        insert_quantum_job::<T>(&submitter, &job_id);
        insert_quantum_result::<T>(&executor, &job_id);
        insert_verification_request::<T>(&job_id);

        // Set validator reputation to required minimum (50)
        ValidatorReputation::<T>::insert(&validator, 100u32);

        #[extrinsic_call]
        submit_verification(
            RawOrigin::Signed(validator),
            job_id,
            0u8,        // vote_index: Approve
            90u8,       // confidence
            [3u8; 32],  // result_hash
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
