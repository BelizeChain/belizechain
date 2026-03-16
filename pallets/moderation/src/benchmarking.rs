//! Benchmarking for the moderation pallet

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn flag_content() {
        let caller: T::AccountId = whitelisted_caller();
        let content_hash: ContentHash = [1u8; 32];
        let reason_index: u8 = 0; // HateSpeech

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), content_hash, reason_index);
    }

    #[benchmark]
    fn review_content() {
        // Setup: add moderator and queue content
        let moderator: T::AccountId = whitelisted_caller();
        ModeratorSet::<T>::try_mutate(|mods| {
            mods.try_push(moderator.clone())
        }).expect("mod set not full");

        let content_hash: ContentHash = [1u8; 32];
        ModerationQueue::<T>::insert(content_hash, true);

        let ruling_index: u8 = 0; // Cleared

        #[extrinsic_call]
        _(RawOrigin::Signed(moderator), content_hash, ruling_index);
    }

    #[benchmark]
    fn add_moderator() {
        let account: T::AccountId = account("moderator", 0, 0);

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    #[benchmark]
    fn remove_moderator() {
        let account: T::AccountId = account("moderator", 0, 0);
        ModeratorSet::<T>::try_mutate(|mods| {
            mods.try_push(account.clone())
        }).expect("mod set not full");

        #[extrinsic_call]
        _(RawOrigin::Root, account);
    }

    #[benchmark]
    fn submit_nawal_assessment() {
        let content_hash: ContentHash = [1u8; 32];
        let score: u8 = 75;

        #[extrinsic_call]
        _(RawOrigin::Root, content_hash, score);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
