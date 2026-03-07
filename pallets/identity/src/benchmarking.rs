//! Benchmarking for the identity pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

/// Fund an account with enough balance for any identity operation.
fn fund_account<T: Config>(who: &T::AccountId) {
    // 100_000 × min_balance covers operation fees plus issuer bond in all chain specs.
    let balance = T::Currency::minimum_balance() * 100_000u32.into();
    let _ = T::Currency::make_free_balance_be(who, balance);
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_identity() {
        let caller: T::AccountId = whitelisted_caller();
        fund_account::<T>(&caller);

        // Ensure operation fee is zero so the benchmark measures the extrinsic, not a transfer
        OperationFee::<T>::kill();

        let name: BoundedVec<u8, T::MaxNameLen> =
            b"Alice Johnson"[..].to_vec().try_into().expect("name fits");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), name);
    }

    #[benchmark]
    fn link_account() {
        let caller: T::AccountId = whitelisted_caller();
        fund_account::<T>(&caller);
        OperationFee::<T>::kill();

        let name: BoundedVec<u8, T::MaxNameLen> =
            b"Bob Smith"[..].to_vec().try_into().expect("name fits");

        // Register identity first
        let _ = Pallet::<T>::register_identity(
            RawOrigin::Signed(caller.clone()).into(),
            name,
        );

        let new_account: T::AccountId = account("linked", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), new_account);
    }

    #[benchmark]
    fn update_did() {
        let caller: T::AccountId = whitelisted_caller();
        fund_account::<T>(&caller);
        OperationFee::<T>::kill();

        let name: BoundedVec<u8, T::MaxNameLen> =
            b"Carol White"[..].to_vec().try_into().expect("name fits");

        // Register identity first
        let _ = Pallet::<T>::register_identity(
            RawOrigin::Signed(caller.clone()).into(),
            name,
        );

        let cid: BoundedVec<u8, T::MaxAnchorLen> =
            b"QmBenchmarkDIDDoc123"[..].to_vec().try_into().expect("cid fits");

        #[extrinsic_call]
        update_did_doc(RawOrigin::Signed(caller), cid);
    }

    #[benchmark]
    fn admin_simple() {
        // Uses add_issuer as representative admin operation
        let issuer: T::AccountId = account("issuer", 0, 0);
        fund_account::<T>(&issuer);
        let attr: u8 = 0; // SSN

        // Clear issuer bond requirement so add_issuer succeeds without a pre-deposit
        IssuerBondAmount::<T>::kill();

        #[extrinsic_call]
        add_issuer(RawOrigin::Root, attr, issuer);
    }

    #[benchmark]
    fn issue_attestation() {
        let issuer: T::AccountId = whitelisted_caller();
        fund_account::<T>(&issuer);
        OperationFee::<T>::kill();
        IssuerBondAmount::<T>::kill();

        let target: T::AccountId = account("target", 0, 0);
        fund_account::<T>(&target);

        // Register target identity
        let name: BoundedVec<u8, T::MaxNameLen> =
            b"Target Person"[..].to_vec().try_into().expect("name fits");
        let _ = Pallet::<T>::register_identity(
            RawOrigin::Signed(target.clone()).into(),
            name,
        );

        // Authorize issuer
        let _ = Pallet::<T>::add_issuer(
            RawOrigin::Root.into(),
            0u8, // SSN
            issuer.clone(),
        );

        let hash = sp_core::H256::from([1u8; 32]);
        let anchor: BoundedVec<u8, T::MaxAnchorLen> =
            b"ipfs://QmBenchAttest"[..].to_vec().try_into().expect("anchor fits");

        #[extrinsic_call]
        issue_ssn(RawOrigin::Signed(issuer), target, hash, anchor, true);
    }

    #[benchmark]
    fn revoke() {
        let target: T::AccountId = account("target", 0, 0);
        fund_account::<T>(&target);
        OperationFee::<T>::kill();
        IssuerBondAmount::<T>::kill();

        let issuer: T::AccountId = whitelisted_caller();
        fund_account::<T>(&issuer);

        // Register target identity
        let name: BoundedVec<u8, T::MaxNameLen> =
            b"Target Revoke"[..].to_vec().try_into().expect("name fits");
        let _ = Pallet::<T>::register_identity(
            RawOrigin::Signed(target.clone()).into(),
            name,
        );

        // Authorize and issue attestation
        let _ = Pallet::<T>::add_issuer(
            RawOrigin::Root.into(),
            0u8,
            issuer.clone(),
        );
        let hash = sp_core::H256::from([2u8; 32]);
        let anchor: BoundedVec<u8, T::MaxAnchorLen> =
            b"ipfs://QmRevoke"[..].to_vec().try_into().expect("anchor fits");
        let _ = Pallet::<T>::issue_ssn(
            RawOrigin::Signed(issuer).into(),
            target.clone(),
            hash,
            anchor,
            true,
        );

        let attr: u8 = 0; // SSN

        #[extrinsic_call]
        _(RawOrigin::Root, target, attr);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
