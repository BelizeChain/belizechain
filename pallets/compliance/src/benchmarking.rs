//! Benchmarking for the compliance pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn verify_account() {
        let target: T::AccountId = account("target", 0, 0);
        let verification_level: u8 = 3; // Full KYC
        let risk_level: u8 = 1; // Low risk

        #[extrinsic_call]
        _(RawOrigin::Root, target, verification_level, risk_level);
    }

    #[benchmark]
    fn update_risk_level() {
        let target: T::AccountId = account("target", 0, 0);

        // First verify the account
        let _ = Pallet::<T>::verify_account(RawOrigin::Root.into(), target.clone(), 3u8, 1u8);

        let new_risk_level: u8 = 2;

        #[extrinsic_call]
        _(RawOrigin::Root, target, new_risk_level);
    }

    #[benchmark]
    fn whitelist_account() {
        let target: T::AccountId = account("target", 0, 0);

        // First verify the account
        let _ = Pallet::<T>::verify_account(RawOrigin::Root.into(), target.clone(), 3u8, 1u8);

        #[extrinsic_call]
        _(RawOrigin::Root, target);
    }

    #[benchmark]
    fn restrict_account() {
        let target: T::AccountId = account("target", 0, 0);

        // First verify the account
        let _ = Pallet::<T>::verify_account(RawOrigin::Root.into(), target.clone(), 3u8, 1u8);

        let reason: Vec<u8> = b"Suspicious activity detected".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, target, reason);
    }

    #[benchmark]
    fn flag_suspicious_activity() {
        let target: T::AccountId = account("target", 0, 0);

        // First verify the account
        let _ = Pallet::<T>::verify_account(RawOrigin::Root.into(), target.clone(), 3u8, 1u8);

        let activity_type: u8 = 1;
        let description: Vec<u8> = b"Unusual transaction patterns".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, target, activity_type, description);
    }

    #[benchmark]
    fn add_sanctions_entry() {
        let entity_hash: [u8; 32] = [1u8; 32];
        let list_source: Vec<u8> = b"International sanctions list".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, entity_hash, list_source);
    }

    #[benchmark]
    fn remove_sanctions_entry() {
        let entity_hash: [u8; 32] = [1u8; 32];

        // Add entry first
        let _ = Pallet::<T>::add_sanctions_entry(
            RawOrigin::Root.into(),
            entity_hash,
            b"International sanctions list".to_vec(),
        );

        #[extrinsic_call]
        _(RawOrigin::Root, entity_hash);
    }

    #[benchmark]
    fn check_compliance() {
        // Placeholder — check_compliance is an internal function, not an extrinsic
        #[block]
        {}
    }

    #[benchmark]
    fn update_verification_level() {
        let target: T::AccountId = account("target", 0, 0);

        // First verify the account
        let _ = Pallet::<T>::verify_account(RawOrigin::Root.into(), target.clone(), 1u8, 1u8);

        #[extrinsic_call]
        sync_verification_from_identity(RawOrigin::Signed(target));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
