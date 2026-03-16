//! Benchmarking for the justice pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::{Currency, Get};
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn open_dispute() {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = account("target", 0, 0);
        let bond = T::OpenDisputeBond::get();
        let _ = T::Currency::make_free_balance_be(&caller, bond * 10u32.into());
        let evidence_hash: [u8; 32] = [1u8; 32];
        let severity: u8 = 0; // Minor

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), target, evidence_hash, severity);
    }

    #[benchmark]
    fn mediator_ruling() {
        // Setup: open a dispute first
        let disputant: T::AccountId = account("disputant", 0, 0);
        let target: T::AccountId = account("target", 0, 0);
        let bond = T::OpenDisputeBond::get();
        let _ = T::Currency::make_free_balance_be(&disputant, bond * 10u32.into());

        let _ = Pallet::<T>::open_dispute(
            RawOrigin::Signed(disputant).into(),
            target.clone(),
            [1u8; 32],
            0u8,
        );

        // Add mediator to the list
        let mediator: T::AccountId = whitelisted_caller();
        MediatorList::<T>::try_mutate(|list| {
            list.try_push(mediator.clone())
        }).expect("mediator list not full");

        #[extrinsic_call]
        _(RawOrigin::Signed(mediator), 1u32, 0u8, 0u32);
    }

    #[benchmark]
    fn appeal_ruling() {
        // Setup: open dispute and issue ruling
        let disputant: T::AccountId = account("disputant", 0, 0);
        let target: T::AccountId = whitelisted_caller();
        let bond = T::OpenDisputeBond::get();
        let _ = T::Currency::make_free_balance_be(&disputant, bond * 10u32.into());

        let _ = Pallet::<T>::open_dispute(
            RawOrigin::Signed(disputant).into(),
            target.clone(),
            [1u8; 32],
            0u8,
        );

        let mediator: T::AccountId = account("mediator", 0, 0);
        MediatorList::<T>::try_mutate(|list| {
            list.try_push(mediator.clone())
        }).expect("mediator list not full");

        let _ = Pallet::<T>::mediator_ruling(
            RawOrigin::Signed(mediator).into(),
            1u32,
            1u8, // Upheld
            0u32,
        );

        let counter_evidence: [u8; 32] = [2u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(target), 1u32, counter_evidence);
    }

    #[benchmark]
    fn complete_rehabilitation() {
        let target: T::AccountId = account("target", 0, 0);
        RehabilitationStatus::<T>::insert(&target, RehabStatus::InRehabilitation);
        // Set cooling-off to block 0 so it has elapsed
        CoolingOffEnd::<T>::insert(&target, BlockNumberFor::<T>::default());

        #[extrinsic_call]
        _(RawOrigin::Root, target);
    }

    #[benchmark]
    fn add_mediator() {
        let mediator: T::AccountId = account("mediator", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, mediator);
    }

    #[benchmark]
    fn remove_mediator() {
        let mediator: T::AccountId = account("mediator", 0, 0);
        MediatorList::<T>::try_mutate(|list| {
            list.try_push(mediator.clone())
        }).expect("mediator list not full");

        #[extrinsic_call]
        _(RawOrigin::Root, mediator);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
