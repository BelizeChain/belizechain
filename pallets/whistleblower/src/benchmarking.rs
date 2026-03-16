//! Benchmarking for the whistleblower pallet

use super::*;
use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_report() {
        let caller: T::AccountId = whitelisted_caller();
        let bond = T::ReportBond::get();
        let _ = T::Currency::make_free_balance_be(&caller, bond * 10u32.into());
        let alias_hash: [u8; 32] = [1u8; 32];
        let target: T::AccountId = account("target", 0, 0);
        let evidence_hash: [u8; 32] = [2u8; 32];
        let category: u8 = 0; // Fraud

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), alias_hash, target, evidence_hash, category);
    }

    #[benchmark]
    fn review_report() {
        // Setup: submit a report first
        let reporter: T::AccountId = account("reporter", 0, 0);
        let bond = T::ReportBond::get();
        let _ = T::Currency::make_free_balance_be(&reporter, bond * 10u32.into());

        let _ = Pallet::<T>::submit_report(
            RawOrigin::Signed(reporter).into(),
            [1u8; 32],
            account("target", 0, 0),
            [2u8; 32],
            0u8,
        );

        // Fund the pool so Verified verdict succeeds
        let reward = T::FraudReward::get();
        WhistleblowerPool::<T>::put(reward);

        let reasoning_hash: [u8; 32] = [3u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Root, 1u32, 0u8, reasoning_hash);
    }

    #[benchmark]
    fn claim_reward() {
        // Setup: submit and verify a report
        let reporter: T::AccountId = whitelisted_caller();
        let bond = T::ReportBond::get();
        let _ = T::Currency::make_free_balance_be(&reporter, bond * 10u32.into());

        // Compute alias_hash = blake2_256(reporter ++ nonce)
        let nonce: [u8; 32] = [42u8; 32];
        let mut preimage = reporter.encode();
        preimage.extend_from_slice(&nonce);
        let alias_hash = sp_io::hashing::blake2_256(&preimage);

        let _ = Pallet::<T>::submit_report(
            RawOrigin::Signed(reporter.clone()).into(),
            alias_hash,
            account("target", 0, 0),
            [2u8; 32],
            0u8,
        );

        let reward = T::FraudReward::get();
        WhistleblowerPool::<T>::put(reward);

        let _ = Pallet::<T>::review_report(
            RawOrigin::Root.into(),
            1u32,
            0u8, // Verified
            [3u8; 32],
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(reporter), 1u32, nonce);
    }

    #[benchmark]
    fn fund_whistleblower_pool() {
        let amount: BalanceOf<T> = 1_000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Root, amount);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
