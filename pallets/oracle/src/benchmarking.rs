//! Benchmarking for oracle pallet

use super::*;
// The pallet module keeps its type imports private, so pull them in here too.
use crate::types::*;
use frame_benchmarking::v2::*;
use frame_support::pallet_prelude::{BoundedVec, ConstU32};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

/// Registers `operator` as an authorized oracle operator.
///
/// Mirrors the state `add_operator` produces, including the O(1) counter that
/// gates `T::MaxOperators`.
fn register_operator<T: Config>(seed: u32) -> T::AccountId {
    let operator: T::AccountId = account("operator", seed, 0);
    OracleOperators::<T>::insert(&operator, true);
    OperatorCount::<T>::mutate(|c| *c = c.saturating_add(1));
    operator
}

/// Inserts an IoT device owned by `owner`, as `register_iot_device` would.
fn insert_device<T: Config>(owner: &T::AccountId, device_id: [u8; 32]) {
    let now = frame_system::Pallet::<T>::block_number();
    IoTDevices::<T>::insert(
        device_id,
        IoTDevice {
            device_id,
            device_type: DeviceType::WeatherStation,
            owner: owner.clone(),
            location: Some((17_000_000, -88_000_000)),
            registered_at: now,
            last_active: now,
            reputation_score: 0,
            data_submissions: 0,
            verified: false,
        },
    );
}

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

    #[benchmark]
    fn submit_price() {
        let operator = register_operator::<T>(0);

        // A feed is only published once `MinConsensusOperators` operators agree,
        // and the runtime requires 3. Two operators report first so the measured
        // call is the one that actually reaches consensus — aggregation plus the
        // feed write — rather than a lone submission.
        for seed in 1..T::MinConsensusOperators::get() {
            let peer = register_operator::<T>(seed);
            let _ = Pallet::<T>::submit_price(RawOrigin::Signed(peer).into(), 0u8, 1u8, 100u128);
        }

        #[extrinsic_call]
        submit_price(RawOrigin::Signed(operator), 0u8, 1u8, 100u128);

        // Consensus has been reached, so a feed must now be published.
        assert!(PriceFeeds::<T>::iter().next().is_some());
    }

    #[benchmark]
    fn verify_merchant() {
        let operator = register_operator::<T>(0);
        let merchant: T::AccountId = account("merchant", 0, 0);

        #[extrinsic_call]
        verify_merchant(
            RawOrigin::Signed(operator),
            merchant,
            0u8, // MerchantCategory::Accommodation
            BoundedVec::<u8, ConstU32<MAX_CERT_LEN>>::truncate_from(b"cert".to_vec()),
            BoundedVec::<u8, ConstU32<MAX_CERT_LEN>>::truncate_from(b"license".to_vec()),
            Some((17_000_000, -88_000_000)),
        );
    }

    #[benchmark]
    fn add_sanction() {
        let account: T::AccountId = account("target", 0, 0);

        #[extrinsic_call]
        add_sanctioned_entity(
            RawOrigin::Root,
            account,
            0u8, // SanctionSource::OFAC
            BoundedVec::<u8, ConstU32<MAX_REASON_LEN>>::truncate_from(b"sanctioned".to_vec()),
            None,
        );
    }

    #[benchmark]
    fn remove_sanction() {
        let account: T::AccountId = account("target", 0, 0);
        let now = frame_system::Pallet::<T>::block_number();
        SanctionedEntities::<T>::insert(
            &account,
            SanctionInfo {
                source: SanctionSource::OFAC,
                added_at: now,
                reason: BoundedVec::truncate_from(b"sanctioned".to_vec()),
                active: true,
                expires_at: None,
            },
        );

        #[extrinsic_call]
        remove_sanction(RawOrigin::Root, account);
    }

    #[benchmark]
    fn verify_identity() {
        let operator = register_operator::<T>(0);
        let subject: T::AccountId = account("subject", 0, 0);

        #[extrinsic_call]
        verify_identity(
            RawOrigin::Signed(operator),
            subject,
            1u8,       // KycLevel::Basic
            [1u8; 32], // non-zero id_hash
            BoundedVec::<u8, ConstU32<64>>::truncate_from(b"oracle-provider".to_vec()),
            true,
            true,
        );
    }

    #[benchmark]
    fn register_land() {
        let operator = register_operator::<T>(0);
        let owner: T::AccountId = account("owner", 0, 0);

        #[extrinsic_call]
        register_land(
            RawOrigin::Signed(operator),
            [3u8; 32],
            owner,
            100_000u128,
            false,
            0u8,
        );
    }

    #[benchmark]
    fn register_iot_device() {
        let owner: T::AccountId = account("owner", 0, 0);

        #[extrinsic_call]
        register_iot_device(
            RawOrigin::Signed(owner),
            [4u8; 32],
            3u8, // DeviceType::WeatherStation
            Some((17_000_000, -88_000_000)),
        );
    }

    #[benchmark]
    fn submit_iot_data() {
        let operator = register_operator::<T>(0);
        insert_device::<T>(&operator, [5u8; 32]);

        #[extrinsic_call]
        submit_iot_data(
            RawOrigin::Signed(operator),
            [5u8; 32],
            0u8,  // DataFeedType::PriceFeed
            None, // domain
            BoundedVec::<u8, ConstU32<MAX_DATA_LEN>>::truncate_from(b"temperature=27.5".to_vec()),
            [6u8; 32],
            Some((17_000_000, -88_000_000)),
            95u8,
            95u8,
            95u8,
            95u8,
            95u8,
        );
    }

    #[benchmark]
    fn verify_iot_device() {
        let operator = register_operator::<T>(0);
        insert_device::<T>(&operator, [7u8; 32]);

        #[extrinsic_call]
        verify_iot_device(RawOrigin::Signed(operator), [7u8; 32]);
    }

    #[benchmark]
    fn claim_oracle_rewards() {
        let operator = register_operator::<T>(0);
        let now = frame_system::Pallet::<T>::block_number();
        OracleOperatorStatsMap::<T>::insert(
            &operator,
            OracleOperatorStats {
                total_submissions: 1_000,
                avg_quality_score: 900,
                agritech_submissions: 0,
                marine_submissions: 0,
                education_submissions: 0,
                tech_submissions: 0,
                general_submissions: 1_000,
                uptime_percentage: 9_000,
                last_active: now,
                total_rewards: 0,
            },
        );
        // The reward is paid from the treasury account (~1M DALLA).
        let treasury = T::TreasuryAccount::get();
        let huge: <T::Currency as Currency<T::AccountId>>::Balance =
            1_000_000_000_000_000_000u128.saturated_into();
        T::Currency::make_free_balance_be(&treasury, huge);

        #[extrinsic_call]
        claim_oracle_rewards(RawOrigin::Signed(operator));
    }

    #[benchmark]
    fn update_exchange_rate() {
        #[extrinsic_call]
        update_exchange_rate(RawOrigin::Root, 0u8, 1u8, 2u128);
    }

    #[benchmark]
    fn resolve_kyc_dispute() {
        let subject: T::AccountId = account("subject", 0, 0);
        OracleDisputeFlag::<T>::insert(&subject, true);

        #[extrinsic_call]
        resolve_kyc_dispute(RawOrigin::Root, subject);
    }

    #[benchmark]
    fn submit_behavior_flag() {
        let operator = register_operator::<T>(0);
        let target: T::AccountId = account("target", 0, 0);

        #[extrinsic_call]
        submit_behavior_flag(RawOrigin::Signed(operator), target, 0u8);
    }

    #[benchmark]
    fn clear_behavior_flag() {
        let target: T::AccountId = account("target", 0, 0);
        BehaviorFlags::<T>::insert(&target, 0u8);

        #[extrinsic_call]
        clear_behavior_flag(RawOrigin::Root, target);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
