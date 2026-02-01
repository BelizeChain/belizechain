//! Benchmarking for oracle pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn add_operator() {
        let operator: T::AccountId = account("operator", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, operator.clone());

        assert!(OracleOperators::<T>::get(&operator));
    }

    #[benchmark]
    fn remove_operator() {
        let operator: T::AccountId = account("operator", 0, 0);
        OracleOperators::<T>::insert(&operator, true);

        #[extrinsic_call]
        _(RawOrigin::Root, operator.clone());

        assert!(!OracleOperators::<T>::get(&operator));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
