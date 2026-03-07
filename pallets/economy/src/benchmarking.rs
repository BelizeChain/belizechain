//! Benchmarking for the economy pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::traits::Currency;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn issue_bbzd() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        // Authorize minter
        AuthorizedMinters::<T>::insert(&caller, true);

        // Seed reserves so mint doesn't fail with InsufficientReserves
        CentralBankReserves::<T>::put(1_000_000u128);

        let recipient: T::AccountId = account("recipient", 0, 0);
        let amount: u128 = 1000;
        let deposit_reference: BoundedVec<u8, ConstU32<64>> =
            b"BENCH-DEP-001"[..].to_vec().try_into().expect("fits");

        #[extrinsic_call]
        mint_bbzd(RawOrigin::Signed(caller), recipient, amount, deposit_reference);
    }

    #[benchmark]
    fn redeem_bbzd() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        // Pre-populate some bBZD issuance AND give caller bBZD balance
        TotalBbzdSupply::<T>::put(1_000_000u128);
        BBZDBalances::<T>::insert(&caller, 1_000_000u128);

        let amount: u128 = 100;
        let bank_account: BoundedVec<u8, ConstU32<64>> =
            b"BZ-BANK-001"[..].to_vec().try_into().expect("fits");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount, bank_account);
    }

    #[benchmark]
    fn process_redemption() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        // Authorize minter
        AuthorizedMinters::<T>::insert(&caller, true);

        // Create a pending redemption
        let redeemer: T::AccountId = account("redeemer", 0, 0);
        let _ = T::Currency::make_free_balance_be(&redeemer, balance);
        TotalBbzdSupply::<T>::put(1_000_000u128);
        BBZDBalances::<T>::insert(&redeemer, 1_000_000u128);

        let _ = Pallet::<T>::redeem_bbzd(
            RawOrigin::Signed(redeemer).into(),
            100u128,
            b"BZ-BANK-PROC"[..].to_vec().try_into().expect("fits"),
        );

        let redemption_id: u64 = 0;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), redemption_id);
    }

    #[benchmark]
    fn set_minter_authorization() {
        let target: T::AccountId = account("minter", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, target, true);
    }

    #[benchmark]
    fn update_reserves() {
        let new_reserves: u128 = 10_000_000;

        #[extrinsic_call]
        _(RawOrigin::Root, new_reserves);
    }

    #[benchmark]
    fn pay_tourism_incentive() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        // Fund treasury so incentive payment succeeds
        let treasury = T::Treasury::get();
        let _ = T::Currency::make_free_balance_be(&treasury, balance);

        let vendor: T::AccountId = account("vendor", 0, 0);
        let amount: BalanceOf<T> = T::Currency::minimum_balance() * 100u32.into();
        let category_id: u8 = 0; // Accommodation

        #[extrinsic_call]
        process_tourism_payment(RawOrigin::Signed(caller), vendor, amount, category_id);
    }

    #[benchmark]
    fn burn_dalla() {
        let caller: T::AccountId = whitelisted_caller();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&caller, balance);

        let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), amount);
    }

    #[benchmark]
    fn governance_burn() {
        let treasury = T::Treasury::get();
        let balance = T::Currency::minimum_balance() * 1_000_000u32.into();
        let _ = T::Currency::make_free_balance_be(&treasury, balance);

        let amount: BalanceOf<T> = T::Currency::minimum_balance() * 10u32.into();

        #[extrinsic_call]
        _(RawOrigin::Root, amount);
    }

    // Orphaned WeightInfo functions — no corresponding extrinsics exist
    #[benchmark]
    fn send_remittance() {
        #[block]
        {}
    }

    #[benchmark]
    fn update_inflation() {
        #[block]
        {}
    }

    #[benchmark]
    fn update_peg_rate() {
        #[block]
        {}
    }

    #[benchmark]
    fn emergency_shutdown() {
        #[block]
        {}
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
