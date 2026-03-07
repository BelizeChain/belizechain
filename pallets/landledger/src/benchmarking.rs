//! Benchmarking for the landledger pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_property() {
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::RegistrationDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());

        let title_number = b"BZ-PROP-BENCH-001".to_vec();
        let description = b"Benchmark test property".to_vec();
        let coordinates: (i64, i64) = (17_000_000, -88_000_000); // Valid Belize coords
        let area_sqm: u32 = 1000;
        let property_type_index: u8 = 0; // Residential
        let assessed_value: u128 = 100_000;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            title_number,
            description,
            coordinates,
            area_sqm,
            property_type_index,
            assessed_value,
        );
    }

    #[benchmark]
    fn transfer_property() {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);
        let deposit = T::RegistrationDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());

        // Pre-fund pallet account so it can receive transfer tax (must exist above ED)
        let pallet_account = Pallet::<T>::account_id();
        let min_balance = T::Currency::minimum_balance();
        let _ = T::Currency::make_free_balance_be(&pallet_account, min_balance * 10u32.into());

        // Register a property first
        let _ = Pallet::<T>::register_property(
            RawOrigin::Signed(caller.clone()).into(),
            b"BZ-PROP-XFER-001".to_vec(),
            b"Transfer test property".to_vec(),
            (17_000_000, -88_000_000),
            1000,
            0,
            100_000,
        );

        let property_id: u32 = 0;

        // Verify property so transfer is allowed (requires government_verified = true)
        let _ = Pallet::<T>::verify_property(RawOrigin::Root.into(), property_id);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            property_id,
            new_owner,
            150_000u128,
            0u8, // Sale
        );
    }

    #[benchmark]
    fn verify_property() {
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::RegistrationDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());

        // Register a property first
        let _ = Pallet::<T>::register_property(
            RawOrigin::Signed(caller.clone()).into(),
            b"BZ-PROP-VERI-001".to_vec(),
            b"Verify test property".to_vec(),
            (17_000_000, -88_000_000),
            1000,
            0,
            100_000,
        );

        let property_id: u32 = 0;

        #[extrinsic_call]
        _(RawOrigin::Root, property_id);
    }

    #[benchmark]
    fn survey_property() {
        let surveyor: T::AccountId = account("surveyor", 0, 0);
        let owner: T::AccountId = whitelisted_caller();
        let deposit = T::RegistrationDeposit::get();
        let _ = T::Currency::make_free_balance_be(&owner, deposit * 10u32.into());

        // Register surveyor
        let _ = Pallet::<T>::register_surveyor(
            RawOrigin::Root.into(),
            surveyor.clone(),
        );

        // Register a property
        let _ = Pallet::<T>::register_property(
            RawOrigin::Signed(owner.clone()).into(),
            b"BZ-PROP-SURV-001".to_vec(),
            b"Survey test property".to_vec(),
            (17_000_000, -88_000_000),
            1000,
            0,
            100_000,
        );

        let property_id: u32 = 0;
        let verified_area: u32 = 1050;
        let updated_coords: Option<(i64, i64)> = Some((17_100_000, -88_100_000));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(surveyor),
            property_id,
            verified_area,
            updated_coords,
        );
    }

    #[benchmark]
    fn register_surveyor() {
        let surveyor: T::AccountId = account("surveyor", 1, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, surveyor);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
