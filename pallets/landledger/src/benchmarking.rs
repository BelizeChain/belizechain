//! Benchmarking for the landledger pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    /// `PropertyRecord::encumbrances` capacity.
    const MAX_ENCUMBRANCES: u32 = 10;

    /// The mock gates property registration on KYC; benchmark accounts are
    /// hash-derived and so opt in explicitly. The runtime provider is permissive
    /// for benchmarks, so this is a no-op there.
    #[cfg(not(test))]
    fn grant_kyc<T: Config>(_account: &T::AccountId) {}

    #[cfg(test)]
    fn grant_kyc<T: Config>(account: &T::AccountId) {
        crate::mock::grant_benchmark_kyc(account);
    }

    /// Seed a worst-case property: every fixed-size field filled to its capacity,
    /// so any mutation rewrites the largest possible `PropertyRecord`.
    fn seed_property<T: Config>(property_id: u32, encumbrances: u32) {
        let encumbrance = Encumbrance {
            encumbrance_type: EncumbranceType::Mortgage,
            holder: account("holder", 0, 0),
            amount: Some(1_000u128),
            description: BoundedVec::try_from(vec![b'x'; 256]).unwrap_or_default(),
            active: true,
        };
        Properties::<T>::insert(
            property_id,
            PropertyRecord {
                property_id,
                owner: account("owner", 0, 0),
                title_number: BoundedVec::try_from(vec![b'x'; 64]).unwrap_or_default(),
                description: BoundedVec::try_from(vec![b'x'; 256]).unwrap_or_default(),
                coordinates: (17_000_000, -88_000_000),
                area_sqm: 1_000u32,
                property_type: PropertyType::Residential,
                assessed_value: 100_000u128,
                registered_at: 0u64,
                last_transferred: None,
                government_verified: true,
                surveyed: true,
                environmental_clearance: false,
                is_tourism_property: false,
                zoning: ZoningType::UrbanResidential,
                encumbrances: BoundedVec::try_from(vec![encumbrance; encumbrances as usize])
                    .unwrap_or_default(),
            },
        );
    }

    #[benchmark]
    fn register_property() {
        let caller: T::AccountId = whitelisted_caller();
        let deposit = T::RegistrationDeposit::get();
        let _ = T::Currency::make_free_balance_be(&caller, deposit * 10u32.into());
        grant_kyc::<T>(&caller);

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
        // Both parties need KYC: the seller to register, the buyer for the
        // transaction itself.
        grant_kyc::<T>(&caller);
        grant_kyc::<T>(&new_owner);

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

        // Mark property as surveyed (required for transfer)
        Properties::<T>::mutate(property_id, |maybe_prop| {
            if let Some(prop) = maybe_prop {
                prop.surveyed = true;
            }
        });

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
        grant_kyc::<T>(&caller);

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
        grant_kyc::<T>(&owner);

        // Register surveyor
        let _ = Pallet::<T>::register_surveyor(RawOrigin::Root.into(), surveyor.clone());

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

    #[benchmark]
    fn remove_surveyor() {
        let surveyor: T::AccountId = account("surveyor", 1, 0);
        GovernmentSurveyors::<T>::insert(&surveyor, true);

        #[extrinsic_call]
        _(RawOrigin::Root, surveyor.clone());

        assert!(!GovernmentSurveyors::<T>::contains_key(&surveyor));
    }

    #[benchmark]
    fn add_encumbrance() {
        let property_id: u32 = 0;
        // One short of the cap, so the push rewrites the largest possible
        // `PropertyRecord`.
        seed_property::<T>(property_id, MAX_ENCUMBRANCES - 1);

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            property_id,
            EncumbranceType::TaxLien,
            account("holder", 1, 0),
            Some(500u128),
            vec![b'x'; 256],
        );

        assert_eq!(
            Properties::<T>::get(property_id)
                .expect("benchmark seeded a property")
                .encumbrances
                .len(),
            MAX_ENCUMBRANCES as usize
        );
    }

    #[benchmark]
    fn remove_encumbrance() {
        let property_id: u32 = 0;
        // A full encumbrance list makes the read-modify-write below as large as
        // it can get.
        seed_property::<T>(property_id, MAX_ENCUMBRANCES);

        #[extrinsic_call]
        _(RawOrigin::Root, property_id, 0u32);

        assert!(
            !Properties::<T>::get(property_id)
                .expect("benchmark seeded a property")
                .encumbrances[0]
                .active
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
