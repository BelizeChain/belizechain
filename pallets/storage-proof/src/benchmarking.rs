//! Benchmarking for pallet-storage-proof.

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, EnsureOrigin, Get};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

/// Builds a valid `levels`-deep Merkle proof.
///
/// Generic over `T` so it can use the pallet's own root derivation rather than a
/// copy of the fold, which could silently drift from what is verified.
fn build_valid_proof<T: Config>(levels: u32) -> (ContentId, MerkleProof) {
    let leaf = sp_io::hashing::blake2_256(b"belizechain-bench-leaf");
    let siblings: Vec<MerkleStep> = (0..levels)
        .map(|i| MerkleStep {
            node: sp_io::hashing::blake2_256(&i.to_le_bytes()),
            node_is_left: false,
        })
        .collect();

    let root = <Pallet<T>>::derive_merkle_root(&leaf, &siblings);
    (
        root,
        MerkleProof {
            leaf,
            siblings: BoundedVec::truncate_from(siblings),
        },
    )
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// Interval, term and escrow used to seed benchmark commitments.
    fn benchmark_terms<T: Config>() -> (BlockNumberFor<T>, BlockNumberFor<T>, BalanceOf<T>) {
        let interval = T::MinProofInterval::get();
        let duration = interval.saturating_mul(10u32.into());
        let escrow = T::MinEscrow::get().saturating_mul(10u32.into());
        (interval, duration, escrow)
    }

    /// Opens a commitment for `root`, funded by `client`, and returns the interval.
    ///
    /// A benchmark cannot fabricate an anchor — the commitment *is* the anchor —
    /// so this calls the same `commit_storage` path a real client takes. There is
    /// no benchmark-only backdoor into the registry.
    fn open_commitment<T: Config>(client: &T::AccountId, root: ContentId) -> BlockNumberFor<T> {
        let (interval, duration, escrow) = benchmark_terms::<T>();
        let _ = T::Currency::make_free_balance_be(client, escrow.saturating_mul(100u32.into()));

        Pallet::<T>::commit_storage(
            RawOrigin::Signed(client.clone()).into(),
            root,
            escrow,
            interval,
            duration,
        )
        .expect("benchmark commitment setup must succeed");

        interval
    }

    /// Moves the chain forward by `blocks`.
    fn advance<T: frame_system::Config>(blocks: BlockNumberFor<T>) {
        let now = frame_system::Pallet::<T>::block_number();
        frame_system::Pallet::<T>::set_block_number(now.saturating_add(blocks));
    }

    /// `s` is the number of Merkle siblings; the trait exposes it as a weight
    /// component because verification cost is linear in the proof depth.
    #[benchmark]
    fn submit_storage_proof(s: Linear<1, MAX_PROOF_STEPS>) {
        let caller: T::AccountId = whitelisted_caller();
        let (root, proof) = build_valid_proof::<T>(s);
        let interval = open_commitment::<T>(&caller, root);
        // The first paid proof is due one interval after the commitment.
        advance::<T>(interval);

        #[extrinsic_call]
        submit_storage_proof(RawOrigin::Signed(caller), root, proof, ProofType::Merkle);
    }

    #[benchmark]
    fn revoke_proof() {
        let caller: T::AccountId = whitelisted_caller();
        let (root, proof) = build_valid_proof::<T>(MAX_PROOF_STEPS);
        let interval = open_commitment::<T>(&caller, root);
        advance::<T>(interval);

        let _ = Pallet::<T>::submit_storage_proof(
            RawOrigin::Signed(caller).into(),
            root,
            proof,
            ProofType::Merkle,
        );

        // Resolve the privileged origin the same way both the mock and the real
        // runtime do, so this benchmark works with either origin configuration.
        let origin =
            T::RevocationOrigin::try_successful_origin().expect("revocation origin is configured");

        #[extrinsic_call]
        revoke_proof(origin, root);
    }

    #[benchmark]
    fn commit_storage() {
        let client: T::AccountId = whitelisted_caller();
        let (root, _) = build_valid_proof::<T>(1);
        let (interval, duration, escrow) = benchmark_terms::<T>();
        let _ = T::Currency::make_free_balance_be(&client, escrow.saturating_mul(10u32.into()));

        #[extrinsic_call]
        commit_storage(RawOrigin::Signed(client), root, escrow, interval, duration);
    }

    #[benchmark]
    fn refund_expired() {
        let client: T::AccountId = whitelisted_caller();
        let (root, _) = build_valid_proof::<T>(1);
        let (_, duration, _) = benchmark_terms::<T>();
        let _ = open_commitment::<T>(&client, root);
        // Past the term, so the refund is permitted.
        advance::<T>(duration.saturating_add(1u32.into()));

        #[extrinsic_call]
        refund_expired(RawOrigin::Signed(client), root);
    }

    #[benchmark]
    fn challenge_provider() {
        let caller: T::AccountId = whitelisted_caller();
        let (root, proof) = build_valid_proof::<T>(1);
        let interval = open_commitment::<T>(&caller, root);
        advance::<T>(interval);

        // A challenge only makes sense once a provider has claimed the deal.
        let _ = Pallet::<T>::submit_storage_proof(
            RawOrigin::Signed(caller.clone()).into(),
            root,
            proof,
            ProofType::Merkle,
        );

        #[extrinsic_call]
        challenge_provider(RawOrigin::Signed(caller), root);
    }

    #[benchmark]
    fn claim_challenge_failed() {
        let caller: T::AccountId = whitelisted_caller();
        let (root, proof) = build_valid_proof::<T>(1);
        let interval = open_commitment::<T>(&caller, root);
        advance::<T>(interval);

        let _ = Pallet::<T>::submit_storage_proof(
            RawOrigin::Signed(caller.clone()).into(),
            root,
            proof,
            ProofType::Merkle,
        );
        let _ = Pallet::<T>::challenge_provider(RawOrigin::Signed(caller.clone()).into(), root);

        // Window elapsed with no answering proof.
        advance::<T>(T::ChallengeWindow::get());

        #[extrinsic_call]
        claim_challenge_failed(RawOrigin::Signed(caller), root);
    }

    #[benchmark]
    fn release_provider() {
        let client: T::AccountId = whitelisted_caller();
        let (root, _) = build_valid_proof::<T>(1);
        let _ = open_commitment::<T>(&client, root);

        #[extrinsic_call]
        release_provider(RawOrigin::Signed(client), root);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
