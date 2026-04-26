//! Benchmarking for pallet-belize-interoperability
//!
//! Uses `frame_benchmarking::v2` API.
//! GovernanceOrigin = EnsureRoot in runtime → `RawOrigin::Root`.
//! KYC checks bypass: initiate_bridge requires KYC level 2+, which
//! cannot be easily set in benchmarks. We pre-populate ChainConfigurations
//! and use process_unlock (which goes via BridgeValidators check) to test
//! the bridge path. For initiate_bridge we test the signed path and rely on
//! the KYC provider being permissive in benchmark mode.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_std::vec;

const SEED: u32 = 0;

/// Pre-populate a ChainConfig for BridgeChain index 1 (Ethereum).
fn setup_chain_config<T: Config>() {
    let config = ChainConfig {
        enabled: true,
        min_confirmations: 12,
        max_amount: 1_000_000_000_000_000_000u128, // 1M DALLA
        fee_rate: 100,                             // 1%
        pq_signatures_required: 1,
        rpc_endpoint: vec![b'h', b't', b't', b'p'].try_into().expect("rpc fits"),
        contract_address: None,
    };
    ChainConfigurations::<T>::insert(BridgeChain::Ethereum, config);
}

/// Pre-populate a BridgeValidator.
fn setup_bridge_validator<T: Config>(idx: u32) -> T::AccountId {
    let who: T::AccountId = account("bridge_val", idx, SEED);
    let validator = BridgeValidator {
        account: who.clone(),
        pq_public_key: vec![1u8; 96].try_into().unwrap_or_default(),
        supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap_or_default(),
        stake: 1_000_000_000_000u128,
        reliability_score: 100u8,
        signatures_count: 0u32,
        failed_signatures: 0u32,
    };
    BridgeValidators::<T>::insert(&who, validator);
    who
}

/// Create a bridge transaction in storage for use by provide_pq_signature and dispute.
fn setup_bridge_tx<T: Config>(initiator: &T::AccountId) -> u32 {
    let tx_id = NextTxId::<T>::get();
    let current_block = frame_system::Pallet::<T>::block_number();
    let bridge_tx = BridgeTransaction {
        tx_id,
        initiator: initiator.clone(),
        operation: BridgeOperation::LockAndMint {
            target_chain: BridgeChain::Ethereum,
            target_address: vec![0u8; 20].try_into().unwrap_or_default(),
            amount: 1_000_000_000_000u128,
            asset: BridgeAsset::DALLA,
        },
        status: BridgeStatus::Initiated,
        required_signatures: 1,
        collected_signatures: 0,
        pq_signatures: BoundedVec::default(),
        fee: 10_000_000_000u128,
        initiated_at: current_block,
        completed_at: None,
        external_confirmation: None,
        dispute_info: None,
    };
    BridgeTransactions::<T>::insert(tx_id, bridge_tx);
    NextTxId::<T>::put(tx_id.saturating_add(1));
    tx_id
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ───────────────────────────────────────────
    // 1. initiate_bridge — signed, KYC level 2+
    // ───────────────────────────────────────────
    #[benchmark]
    fn initiate_bridge() {
        setup_chain_config::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let amount = 100_000_000_000_000u128; // 100 DALLA (above MinBridgeAmount of 50 DALLA)
                                              // SAFETY(saturated_into): benchmark seed balance, well within Balance range
        T::Currency::make_free_balance_be(&caller, (amount * 10).saturated_into());

        // Pre-fund treasury so the fee transfer doesn't fail due to ExistentialDeposit
        let treasury = T::Treasury::get();
        T::Currency::make_free_balance_be(&treasury, (amount * 10).saturated_into());

        #[extrinsic_call]
        initiate_bridge(
            RawOrigin::Signed(caller),
            1u8,           // Ethereum
            vec![0u8; 20], // target_address
            amount.saturated_into(),
            0u8, // DALLA
        );
    }

    // ───────────────────────────────────────────
    // 2. provide_signature → provide_pq_signature
    //    WeightInfo fn: provide_signature()
    //    Extrinsic name: provide_pq_signature
    // ───────────────────────────────────────────
    #[benchmark]
    fn provide_signature() {
        setup_chain_config::<T>();
        let validator = setup_bridge_validator::<T>(0);
        let initiator: T::AccountId = account("init", 0, SEED);
        let tx_id = setup_bridge_tx::<T>(&initiator);

        #[extrinsic_call]
        provide_pq_signature(
            RawOrigin::Signed(validator),
            tx_id,
            vec![0u8; 64], // pq_signature
        );
    }

    // ───────────────────────────────────────────
    // 3. create_liquidity_pool — signed
    // ───────────────────────────────────────────
    #[benchmark]
    fn create_liquidity_pool() {
        setup_chain_config::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let initial_liquidity = 10_000_000_000_000u128;
        T::Currency::make_free_balance_be(&caller, (initial_liquidity * 10).saturated_into());

        #[extrinsic_call]
        create_liquidity_pool(
            RawOrigin::Signed(caller),
            1u8, // Ethereum
            0u8, // DALLA
            initial_liquidity.saturated_into(),
        );
    }

    // ───────────────────────────────────────────
    // 4. process_unlock — signed bridge validator
    //    B-1 fix: now takes tx_id referencing a Finalized BurnAndUnlock BridgeTransaction
    // ───────────────────────────────────────────
    #[benchmark]
    fn process_unlock() {
        setup_chain_config::<T>();
        let validator = setup_bridge_validator::<T>(1);
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let amount = 500_000_000_000u128;

        // Pre-populate TotalLockedAssets so the unlock check passes
        TotalLockedAssets::<T>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, amount * 2);

        // Pre-populate UserBridgeLocks for the recipient
        UserBridgeLocks::<T>::insert(&recipient, amount);
        T::Currency::make_free_balance_be(&recipient, (amount * 10).saturated_into());
        T::Currency::set_lock(
            BRIDGE_LOCK_ID,
            &recipient,
            amount.saturated_into(),
            frame_support::traits::WithdrawReasons::all(),
        );

        // Create a BurnAndUnlock BridgeTransaction and set it to Finalized
        let tx_id = NextTxId::<T>::get();
        let current_block = frame_system::Pallet::<T>::block_number();
        let recipient_encoded = recipient.encode();
        let recipient_bounded: BoundedVec<u8, ConstU32<64>> =
            recipient_encoded.try_into().unwrap_or_default();
        let bridge_tx = BridgeTransaction {
            tx_id,
            initiator: validator.clone(),
            operation: BridgeOperation::BurnAndUnlock {
                source_chain: BridgeChain::Ethereum,
                source_tx_hash: vec![0u8; 32].try_into().unwrap_or_default(),
                amount,
                asset: BridgeAsset::DALLA,
                recipient: recipient_bounded,
            },
            status: BridgeStatus::Finalized,
            required_signatures: 1,
            collected_signatures: 1,
            pq_signatures: BoundedVec::default(),
            fee: 0u128,
            initiated_at: current_block,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<T>::insert(tx_id, bridge_tx);
        NextTxId::<T>::put(tx_id.saturating_add(1));

        // Fund the escrow account so the unlock transfer succeeds
        let escrow: T::AccountId = T::PalletId::get().into_account_truncating();
        T::Currency::make_free_balance_be(&escrow, (amount * 10).saturated_into());

        #[extrinsic_call]
        process_unlock(RawOrigin::Signed(validator), tx_id);
    }

    // ───────────────────────────────────────────
    // 5. send_message → send_cross_chain_message
    //    WeightInfo fn: send_message()
    //    Extrinsic name: send_cross_chain_message
    // ───────────────────────────────────────────
    #[benchmark]
    fn send_message() {
        setup_chain_config::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let fee = T::MinBridgeAmount::get();
        T::Currency::make_free_balance_be(&caller, fee * 10u32.into());

        // Pre-fund treasury so it stays above ExistentialDeposit
        let treasury = T::Treasury::get();
        T::Currency::make_free_balance_be(&treasury, fee * 10u32.into());

        #[extrinsic_call]
        send_cross_chain_message(
            RawOrigin::Signed(caller),
            1u8,           // Ethereum
            vec![1u8; 64], // payload
        );
    }

    // ───────────────────────────────────────────
    // 6. update_config → update_bridge_config
    //    WeightInfo fn: update_config()
    //    Extrinsic name: update_bridge_config
    //    Origin: GovernanceOrigin = Root
    // ───────────────────────────────────────────
    #[benchmark]
    fn update_config() {
        setup_chain_config::<T>();

        #[extrinsic_call]
        update_bridge_config(
            RawOrigin::Root,
            1u8,                           // chain: Ethereum
            true,                          // enabled
            200u32,                        // fee_rate
            2_000_000_000_000_000_000u128, // max_amount
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
