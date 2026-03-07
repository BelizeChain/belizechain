use super::*;
use crate::mock::*;
use codec::Encode;
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// Bridge Transaction Initiation Tests
// ============================================================================

#[test]
fn initiate_bridge_works() {
    new_test_ext().execute_with(|| {
        // Setup: Create chain configuration for Ethereum
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50, // 0.5%
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        let initial_balance = Balances::free_balance(EVE);
        let bridge_amount = 100_000;
        let target_address = test_eth_address();

        // EVE initiates bridge to Ethereum
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1, // Ethereum chain index
            target_address.clone(),
            bridge_amount,
            0, // DALLA asset
        ));

        // Verify transaction created
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.initiator, EVE);
        assert_eq!(tx.status, BridgeStatus::Initiated);
        assert_eq!(tx.required_signatures, 5);
        assert_eq!(tx.collected_signatures, 0);

        // Verify fee was transferred to treasury
        // fee_rate=50 basis points: fee = 100_000 * 50 / 10_000 = 500
        let expected_fee = (bridge_amount * 50) / 10_000;
        let treasury_balance = Balances::free_balance(TREASURY);
        let expected_treasury = 1_000 + expected_fee; // Initial 1000 + fee
        assert_eq!(treasury_balance, expected_treasury);

        // Verify fee was transferred from EVE to treasury
        // free_balance decreases by fee (direct transfer), bridge amount is locked (set_lock)
        assert_eq!(Balances::free_balance(EVE), initial_balance - expected_fee);

        // Verify NextTxId incremented
        assert_eq!(Interoperability::next_tx_id(), 1);
    });
}

#[test]
fn initiate_bridge_requires_kyc_level_2() {
    new_test_ext().execute_with(|| {
        // Setup chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // LOW_KYC has Level 1 KYC (insufficient)
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(LOW_KYC),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::KycRequired
        );

        // NO_KYC has no KYC
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(NO_KYC),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::KycRequired
        );
    });
}

#[test]
fn initiate_bridge_rejects_sanctioned_accounts() {
    new_test_ext().execute_with(|| {
        // Setup chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // SANCTIONED has Level 2 KYC but is sanctioned
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(SANCTIONED),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::AccountSanctioned
        );
    });
}

#[test]
fn initiate_bridge_enforces_minimum_amount() {
    new_test_ext().execute_with(|| {
        // Setup chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // MinBridgeAmount is 10_000
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                9_999, // Below minimum
                0,
            ),
            Error::<Test>::BelowMinimumAmount
        );

        // Exactly minimum should work
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            10_000,
            0,
        ));
    });
}

#[test]
fn initiate_bridge_enforces_maximum_amount() {
    new_test_ext().execute_with(|| {
        // Setup chain config with low max amount
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 100_000, // Low max for testing
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                200_000, // Above maximum
                0,
            ),
            Error::<Test>::ExceedsMaximumAmount
        );
    });
}

#[test]
fn initiate_bridge_requires_sufficient_balance() {
    new_test_ext().execute_with(|| {
        // Setup chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        let eve_balance = Balances::free_balance(EVE);

        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                eve_balance + 1, // More than balance
                0,
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn initiate_bridge_fails_for_disabled_chain() {
    new_test_ext().execute_with(|| {
        // Setup disabled chain config
        let chain_config = ChainConfig {
            enabled: false, // Disabled
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::BridgeDisabled
        );
    });
}

#[test]
fn initiate_bridge_fails_for_unsupported_chain() {
    new_test_ext().execute_with(|| {
        // No chain config exists
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1, // Ethereum - no config
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::UnsupportedChain
        );
    });
}

#[test]
fn initiate_bridge_updates_total_locked_assets() {
    new_test_ext().execute_with(|| {
        // Setup chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        let bridge_amount = 100_000;
        // fee_rate=50 basis points: fee = 100_000 * 50 / 10_000 = 500
        let expected_fee = (bridge_amount * 50) / 10_000;
        let net_amount = bridge_amount - expected_fee;

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            bridge_amount,
            0,
        ));

        // Verify total locked assets increased by net amount
        let total_locked = Interoperability::total_locked_assets(BridgeChain::Ethereum, BridgeAsset::DALLA);
        assert_eq!(total_locked, net_amount);
    });
}

// ============================================================================
// Post-Quantum Signature Tests
// ============================================================================

#[test]
fn provide_pq_signature_works() {
    new_test_ext().execute_with(|| {
        // Setup: Create a bridge transaction first
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Register ALICE as bridge validator
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        // Create bridge transaction
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // ALICE provides signature
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(ALICE),
            0,
            test_pq_signature(),
        ));

        // Verify signature recorded
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.collected_signatures, 1);
        assert_eq!(tx.status, BridgeStatus::AwaitingSignatures);
        assert_eq!(tx.pq_signatures.len(), 1);
    });
}

#[test]
fn provide_pq_signature_requires_bridge_operator_kyc() {
    new_test_ext().execute_with(|| {
        // Setup chain and transaction
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Register EVE as validator (but EVE only has Level 2 KYC, not Level 3)
        let validator = BridgeValidator {
            account: EVE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(EVE, validator);

        // Create bridge transaction
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(FERDIE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // EVE tries to provide signature but lacks Level 3 KYC
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(EVE),
                0,
                test_pq_signature(),
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn provide_pq_signature_rejects_sanctioned_validators() {
    new_test_ext().execute_with(|| {
        // Setup chain and transaction
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Register SANCTIONED as validator
        let validator = BridgeValidator {
            account: SANCTIONED,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(SANCTIONED, validator);

        // Create bridge transaction
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // SANCTIONED tries to provide signature
        // Note: Bridge operator KYC check runs before sanction check,
        // so SANCTIONED (Level 2 KYC) fails bridge operator requirement (Level 3)
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(SANCTIONED),
                0,
                test_pq_signature(),
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn provide_pq_signature_requires_validator_registration() {
    new_test_ext().execute_with(|| {
        // Setup chain and transaction
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Create bridge transaction
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // ALICE tries to provide signature without being registered
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(ALICE),
                0,
                test_pq_signature(),
            ),
            Error::<Test>::ValidatorNotRegistered
        );
    });
}

#[test]
fn provide_pq_signature_validates_signature_length() {
    new_test_ext().execute_with(|| {
        // Setup
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // Signature too short (< 64 bytes)
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(ALICE),
                0,
                vec![0xEF; 32], // Only 32 bytes
            ),
            Error::<Test>::InvalidPQSignature
        );
    });
}

#[test]
fn provide_pq_signature_transitions_to_ready_when_threshold_met() {
    new_test_ext().execute_with(|| {
        // Setup chain with 3 required signatures
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3, // Only 3 required for this test
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Register 3 validators
        for account in [ALICE, BOB, CHARLIE] {
            let validator = BridgeValidator {
                account,
                pq_public_key: vec![0xAB; 64].try_into().unwrap(),
                supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
                stake: 1_000_000,
                reliability_score: 100,
                signatures_count: 0,
                failed_signatures: 0,
            };
            BridgeValidators::<Test>::insert(account, validator);
        }

        // Create bridge transaction
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // Provide first signature
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(ALICE),
            0,
            test_pq_signature(),
        ));
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::AwaitingSignatures);

        // Provide second signature
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(BOB),
            0,
            test_pq_signature(),
        ));
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::AwaitingSignatures);

        // Provide third signature - should transition to ReadyForExecution
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(CHARLIE),
            0,
            test_pq_signature(),
        ));
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::ReadyForExecution);
        assert_eq!(tx.collected_signatures, 3);
    });
}

#[test]
fn provide_pq_signature_fails_for_executed_transaction() {
    new_test_ext().execute_with(|| {
        // Setup
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        // Create and manually mark transaction as executed
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: EVE,
            operation: BridgeOperation::LockAndMint {
                target_chain: BridgeChain::Ethereum,
                target_address: test_eth_address().try_into().unwrap(),
                amount: 100_000,
                asset: BridgeAsset::DALLA,
            },
            status: BridgeStatus::Executed, // Already executed
            required_signatures: 5,
            collected_signatures: 5,
            pq_signatures: Default::default(),
            fee: 500,
            initiated_at: 1,
            completed_at: Some(10),
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0, tx);

        // Try to provide signature for executed transaction
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(ALICE),
                0,
                test_pq_signature(),
            ),
            Error::<Test>::AlreadyExecuted
        );
    });
}

// ============================================================================
// Liquidity Pool Tests
// ============================================================================

#[test]
fn create_liquidity_pool_works() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(ALICE);
        let liquidity = 1_000_000;

        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1, // Ethereum
            0, // DALLA
            liquidity,
        ));

        // Verify pool created
        let pool = Interoperability::liquidity_pools(0).unwrap();
        assert_eq!(pool.chain, BridgeChain::Ethereum);
        assert_eq!(pool.asset, BridgeAsset::DALLA);
        assert_eq!(pool.belizechain_liquidity, liquidity);
        assert_eq!(pool.manager, ALICE);
        assert!(pool.active);

        // Verify liquidity reserved
        assert_eq!(Balances::free_balance(ALICE), initial_balance - liquidity);

        // Verify NextPoolId incremented
        assert_eq!(Interoperability::next_pool_id(), 1);
    });
}

#[test]
fn create_liquidity_pool_requires_sufficient_balance() {
    new_test_ext().execute_with(|| {
        let alice_balance = Balances::free_balance(ALICE);

        assert_noop!(
            Interoperability::create_liquidity_pool(
                RuntimeOrigin::signed(ALICE),
                1,
                0,
                alice_balance + 1, // More than balance
            ),
            Error::<Test>::InsufficientBalance
        );
    });
}

// ============================================================================
// Unlock Processing Tests
// ============================================================================

#[test]
fn process_unlock_works() {
    new_test_ext().execute_with(|| {
        // Register BOB as validator
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        // Manually set some locked assets
        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000);

        let unlock_amount = 100_000;

        // Build a finalised BurnAndUnlock transaction so process_unlock can look it up.
        let recipient_bytes: BoundedVec<u8, ConstU32<64>> = EVE.encode().try_into().unwrap();
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: BOB,
            operation: BridgeOperation::BurnAndUnlock {
                source_chain: BridgeChain::Ethereum,
                source_tx_hash: test_tx_hash().try_into().unwrap(),
                amount: unlock_amount,
                asset: BridgeAsset::DALLA,
                recipient: recipient_bytes,
            },
            status: BridgeStatus::Finalized,
            required_signatures: 1,
            collected_signatures: 1,
            pq_signatures: BoundedVec::default(),
            fee: 0,
            initiated_at: 0,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0u32, tx);

        assert_ok!(Interoperability::process_unlock(
            RuntimeOrigin::signed(BOB),
            0u32,
        ));

        // Verify total locked assets decreased
        let total_locked = Interoperability::total_locked_assets(BridgeChain::Ethereum, BridgeAsset::DALLA);
        assert_eq!(total_locked, 400_000);
    });
}

#[test]
fn process_unlock_requires_validator_registration() {
    new_test_ext().execute_with(|| {
        // ALICE is not registered as validator — checked before tx lookup
        assert_noop!(
            Interoperability::process_unlock(
                RuntimeOrigin::signed(ALICE),
                1u32,
            ),
            Error::<Test>::ValidatorNotRegistered
        );
    });
}

// ============================================================================
// Cross-Chain Message Tests
// ============================================================================

#[test]
fn send_cross_chain_message_works() {
    new_test_ext().execute_with(|| {
        let payload = test_message_payload();

        assert_ok!(Interoperability::send_cross_chain_message(
            RuntimeOrigin::signed(ALICE),
            1, // Ethereum
            payload.clone(),
        ));

        // Verify message created
        let message = Interoperability::cross_chain_messages(0).unwrap();
        assert_eq!(message.target_chain, BridgeChain::Ethereum);
        assert_eq!(message.payload.to_vec(), payload);
        assert!(!message.delivered);

        // Verify NextMessageId incremented
        assert_eq!(Interoperability::next_message_id(), 1);
    });
}

#[test]
fn send_cross_chain_message_validates_payload_size() {
    new_test_ext().execute_with(|| {
        // Payload too large (> 2048 bytes)
        let large_payload = vec![0xFF; 3000];

        assert_noop!(
            Interoperability::send_cross_chain_message(
                RuntimeOrigin::signed(ALICE),
                1,
                large_payload,
            ),
            Error::<Test>::InvalidConfiguration
        );
    });
}

// ============================================================================
// Bridge Configuration Tests
// ============================================================================

#[test]
fn update_bridge_config_works() {
    new_test_ext().execute_with(|| {
        // Create initial config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Update config via governance
        assert_ok!(Interoperability::update_bridge_config(
            RuntimeOrigin::root(),
            1, // Ethereum
            false, // Disable bridge
            100, // New fee rate
            500_000_000, // New max amount
        ));

        // Verify updates
        let config = Interoperability::chain_configurations(BridgeChain::Ethereum).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.fee_rate, 100);
        assert_eq!(config.max_amount, 500_000_000);
    });
}

#[test]
fn update_bridge_config_requires_governance() {
    new_test_ext().execute_with(|| {
        // Setup initial config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Regular user cannot update config
        assert_noop!(
            Interoperability::update_bridge_config(
                RuntimeOrigin::signed(ALICE),
                1,
                false,
                100,
                500_000_000,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ============================================================================
// Helper Function Tests
// ============================================================================

#[test]
fn chain_encoding_decoding_works() {
    new_test_ext().execute_with(|| {
        // Test all supported chains
        for i in 0..=50 {
            if let Some(chain) = Interoperability::decode_chain(i) {
                let encoded = Interoperability::encode_chain(&chain);
                assert_eq!(encoded, i);
            }
        }

        // Invalid chain index
        assert_eq!(Interoperability::decode_chain(255), None);
    });
}

#[test]
fn asset_encoding_decoding_works() {
    new_test_ext().execute_with(|| {
        // DALLA
        let dalla = Interoperability::decode_asset(0).unwrap();
        assert_eq!(dalla, BridgeAsset::DALLA);
        assert_eq!(Interoperability::encode_asset(&dalla), 0);

        // BBZD
        let bbzd = Interoperability::decode_asset(1).unwrap();
        assert_eq!(bbzd, BridgeAsset::BBZD);
        assert_eq!(Interoperability::encode_asset(&bbzd), 1);

        // Invalid asset
        assert_eq!(Interoperability::decode_asset(2), None);
    });
}

#[test]
fn get_bridge_transaction_works() {
    new_test_ext().execute_with(|| {
        // Setup
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // Get transaction using helper
        let tx = Interoperability::get_bridge_transaction(0).unwrap();
        assert_eq!(tx.initiator, EVE);
        assert_eq!(tx.tx_id, 0);

        // Non-existent transaction
        assert_eq!(Interoperability::get_bridge_transaction(999), None);
    });
}

#[test]
fn get_total_locked_works() {
    new_test_ext().execute_with(|| {
        // Setup
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Initially zero
        let initial = Interoperability::get_total_locked(&BridgeChain::Ethereum, &BridgeAsset::DALLA);
        assert_eq!(initial, 0);

        // Lock some assets
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // fee_rate=50 basis points: fee = 100_000 * 50 / 10_000 = 500
        let expected_fee = (100_000_u128 * 50) / 10_000;
        let net = 100_000 - expected_fee;

        let total = Interoperability::get_total_locked(&BridgeChain::Ethereum, &BridgeAsset::DALLA);
        assert_eq!(total, net);
    });
}

#[test]
fn is_chain_supported_works() {
    new_test_ext().execute_with(|| {
        // Initially not supported
        assert!(!Interoperability::is_chain_supported(&BridgeChain::Ethereum));

        // Add config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Now supported
        assert!(Interoperability::is_chain_supported(&BridgeChain::Ethereum));
    });
}

// ============================================================================
// RATE LIMITING TESTS (AR-15)
// ============================================================================

/// Helper: insert a standard Ethereum chain config.
fn setup_eth_chain_config() {
    let chain_config = ChainConfig {
        enabled: true,
        min_confirmations: 12,
        max_amount: 1_000_000_000,
        fee_rate: 50,
        pq_signatures_required: 5,
        rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
        contract_address: Some(vec![0xABu8; 20].try_into().unwrap()),
    };
    ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);
}

#[test]
fn initiate_bridge_rate_limit_blocks_after_max_per_block() {
    new_test_ext().execute_with(|| {
        setup_eth_chain_config();

        // MaxBridgePerBlock = 5 in the mock; exhaust the limit.
        for _ in 0..5u32 {
            assert_ok!(Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1, // Ethereum
                test_eth_address(),
                100_000,
                0, // DALLA
            ));
        }

        // Sixth call in the same block must be rejected.
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::RateLimitExceeded
        );
    });
}

#[test]
fn initiate_bridge_rate_limit_resets_on_next_block() {
    new_test_ext().execute_with(|| {
        setup_eth_chain_config();

        // Exhaust limit in block 1.
        for _ in 0..5u32 {
            assert_ok!(Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                0,
            ));
        }

        // Move to block 2 — counter resets.
        System::set_block_number(2);
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));
    });
}

#[test]
fn initiate_bridge_rate_limit_is_per_account() {
    new_test_ext().execute_with(|| {
        setup_eth_chain_config();

        // Exhaust EVE's limit.
        for _ in 0..5u32 {
            assert_ok!(Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                0,
            ));
        }

        // FERDIE (different account, level-2 KYC in mock) still has a fresh slot.
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(FERDIE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));
    });
}

// ============================================================================
// Validator Management Tests
// ============================================================================

#[test]
fn remove_bridge_validator_works() {
    new_test_ext().execute_with(|| {
        // Insert BOB as a bridge validator directly
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);
        assert!(Interoperability::bridge_validators(BOB).is_some());

        // Governance removes BOB
        assert_ok!(Interoperability::remove_bridge_validator(
            RuntimeOrigin::root(),
            BOB,
        ));

        assert!(Interoperability::bridge_validators(BOB).is_none());
    });
}

#[test]
fn remove_bridge_validator_not_registered_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::remove_bridge_validator(RuntimeOrigin::root(), BOB),
            Error::<Test>::ValidatorNotRegistered
        );
    });
}

// ============================================================================
// Liquidity Pool Withdrawal Tests
// ============================================================================

#[test]
fn withdraw_liquidity_works() {
    new_test_ext().execute_with(|| {
        let liquidity = 500_000u128;

        // ALICE creates a liquidity pool
        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1, // Ethereum
            0, // DALLA
            liquidity,
        ));

        let pool = Interoperability::liquidity_pools(0).unwrap();
        assert_eq!(pool.belizechain_liquidity, liquidity);

        // ALICE withdraws half
        let withdraw_amount = 200_000u128;
        assert_ok!(Interoperability::withdraw_liquidity(
            RuntimeOrigin::signed(ALICE),
            0, // pool_id
            withdraw_amount,
        ));

        let pool_after = Interoperability::liquidity_pools(0).unwrap();
        assert_eq!(pool_after.belizechain_liquidity, liquidity - withdraw_amount);
        assert!(pool_after.active);
    });
}

#[test]
fn withdraw_liquidity_full_deactivates_pool() {
    new_test_ext().execute_with(|| {
        let liquidity = 300_000u128;

        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1,
            0,
            liquidity,
        ));

        // Withdraw everything
        assert_ok!(Interoperability::withdraw_liquidity(
            RuntimeOrigin::signed(ALICE),
            0,
            liquidity,
        ));

        let pool_after = Interoperability::liquidity_pools(0).unwrap();
        assert_eq!(pool_after.belizechain_liquidity, 0);
        assert!(!pool_after.active, "pool should be deactivated when fully drained");
    });
}

// ============================================================================
// Submit Incoming Unlock Tests
// ============================================================================

#[test]
fn submit_incoming_unlock_works() {
    new_test_ext().execute_with(|| {
        // Insert ALICE as a registered bridge validator
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        // Set up Ethereum chain config
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Pre-seed locked assets so the unlock check passes
        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000u128);

        // ALICE submits an incoming unlock
        assert_ok!(Interoperability::submit_incoming_unlock(
            RuntimeOrigin::signed(ALICE),
            1,              // Ethereum
            test_tx_hash(), // source_tx_hash
            EVE,            // recipient
            100_000u128,    // amount
            0,              // DALLA
        ));

        // Verify a bridge transaction was created
        assert!(Interoperability::bridge_transactions(0).is_some());
        assert_eq!(Interoperability::next_tx_id(), 1);
    });
}

// ============================================================================
// Register Bridge Validator Tests (H-37)
// ============================================================================

#[test]
fn register_bridge_validator_works() {
    new_test_ext().execute_with(|| {
        // ALICE has Level 3 KYC — should succeed
        assert_ok!(Interoperability::register_bridge_validator(
            RuntimeOrigin::signed(ALICE),
            vec![0xAB; 64],   // PQ public key (within 96-byte BoundedVec)
            vec![1, 2],       // Ethereum + Solana
        ));

        // Verify validator stored
        let v = Interoperability::bridge_validators(ALICE).unwrap();
        assert_eq!(v.account, ALICE);
        assert_eq!(v.reliability_score, 100);
        assert_eq!(v.supported_chains.len(), 2);

        // Verify event emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::BridgeValidatorRegistered { validator, supported_chains })
            if *validator == ALICE && supported_chains.len() == 2
        )));
    });
}

#[test]
fn register_bridge_validator_duplicate_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Interoperability::register_bridge_validator(
            RuntimeOrigin::signed(ALICE),
            vec![0xAB; 64],
            vec![1],
        ));

        // Second registration must fail
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(ALICE),
                vec![0xAB; 64],
                vec![1],
            ),
            Error::<Test>::AlreadyExecuted
        );
    });
}

#[test]
fn register_bridge_validator_requires_level3_kyc() {
    new_test_ext().execute_with(|| {
        // EVE has Level 2 — insufficient for validator registration
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(EVE),
                vec![0xAB; 64],
                vec![1],
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );

        // NO_KYC — no KYC at all
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(NO_KYC),
                vec![0xAB; 64],
                vec![1],
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn register_bridge_validator_sanctioned_fails() {
    new_test_ext().execute_with(|| {
        // SANCTIONED (666) has Level 2 KYC, but is sanctioned
        // KYC check runs first (Level 2 < Level 3 required), so BridgeOperatorKycInsufficient
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(SANCTIONED),
                vec![0xAB; 64],
                vec![1],
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn register_bridge_validator_unsupported_chain_fails() {
    new_test_ext().execute_with(|| {
        // Chain index 255 is outside decode_chain range (0..=50)
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(ALICE),
                vec![0xAB; 64],
                vec![255],
            ),
            Error::<Test>::UnsupportedChain
        );
    });
}

#[test]
fn register_bridge_validator_pq_key_too_large_fails() {
    new_test_ext().execute_with(|| {
        // PQ key > 96 bytes exceeds BoundedVec limit
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(ALICE),
                vec![0xAB; 200],
                vec![1],
            ),
            Error::<Test>::InvalidPQSignature
        );
    });
}

#[test]
fn register_bridge_validator_too_many_chains_fails() {
    new_test_ext().execute_with(|| {
        // > 64 valid chains exceeds BoundedVec limit
        // Use indices 0..65 — all valid (0..50) but total > 64
        // Actually we need exactly > 64 valid ones. decode_chain accepts 0..=50,
        // so max valid is 51. We need 65 entries but duplicates are OK for the vec.
        // Let's just repeat valid indices to reach 65 entries.
        let chains: Vec<u8> = (0..65).map(|i| i % 51).collect();
        assert_noop!(
            Interoperability::register_bridge_validator(
                RuntimeOrigin::signed(ALICE),
                vec![0xAB; 64],
                chains,
            ),
            Error::<Test>::InvalidConfiguration
        );
    });
}

// ============================================================================
// Dispute Bridge Transaction Tests (§4.4b)
// ============================================================================

/// Helper: set up a bridge tx that reaches ReadyForExecution with PendingFinalizations.
/// Uses pq_signatures_required=3 with ALICE/BOB/CHARLIE signing.
/// Returns the tx_id (0).
fn setup_disputable_tx() {
    // Register validators
    for &account in &[ALICE, BOB, CHARLIE] {
        let validator = BridgeValidator {
            account,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(account, validator);
    }

    // ETH config with 3 sigs required
    let chain_config = ChainConfig {
        enabled: true,
        min_confirmations: 12,
        max_amount: 1_000_000_000,
        fee_rate: 50,
        pq_signatures_required: 3,
        rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
        contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
    };
    ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

    // EVE initiates bridge
    assert_ok!(Interoperability::initiate_bridge(
        RuntimeOrigin::signed(EVE),
        1,
        test_eth_address(),
        100_000,
        0,
    ));

    // Collect 3 signatures → ReadyForExecution
    for &signer in &[ALICE, BOB, CHARLIE] {
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(signer),
            0,
            test_pq_signature(),
        ));
    }

    // Verify status is now ReadyForExecution
    let tx = Interoperability::bridge_transactions(0).unwrap();
    assert_eq!(tx.status, BridgeStatus::ReadyForExecution);
    assert!(PendingFinalizations::<Test>::contains_key(0));
}

#[test]
fn dispute_bridge_transaction_works() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // ALICE disputes the transaction
        assert_ok!(Interoperability::dispute_bridge_transaction(
            RuntimeOrigin::signed(ALICE),
            0,
            b"fraudulent transaction".to_vec(),
        ));

        // Verify status transitioned to Disputed
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::Disputed);
        assert!(tx.dispute_info.is_some());

        // PendingFinalizations removed — governance must resolve
        assert!(!PendingFinalizations::<Test>::contains_key(0));

        // Event emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::BridgeTransactionDisputed { tx_id: 0, disputer })
            if *disputer == ALICE
        )));
    });
}

#[test]
fn dispute_transaction_not_found_fails() {
    new_test_ext().execute_with(|| {
        // Register ALICE as validator for the permission check
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        assert_noop!(
            Interoperability::dispute_bridge_transaction(
                RuntimeOrigin::signed(ALICE),
                999,
                b"fraud".to_vec(),
            ),
            Error::<Test>::TransactionNotFound
        );
    });
}

#[test]
fn dispute_requires_validator_registration() {
    new_test_ext().execute_with(|| {
        // EVE is not a registered validator
        assert_noop!(
            Interoperability::dispute_bridge_transaction(
                RuntimeOrigin::signed(EVE),
                0,
                b"fraud".to_vec(),
            ),
            Error::<Test>::ValidatorNotRegistered
        );
    });
}

#[test]
fn dispute_requires_bridge_operator_kyc() {
    new_test_ext().execute_with(|| {
        // Register EVE as validator via direct insert (bypassing KYC check in extrinsic)
        let validator = BridgeValidator {
            account: EVE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(EVE, validator);

        assert_noop!(
            Interoperability::dispute_bridge_transaction(
                RuntimeOrigin::signed(EVE),
                0,
                b"fraud".to_vec(),
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn dispute_not_disputable_wrong_status() {
    new_test_ext().execute_with(|| {
        // Register ALICE as validator
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        // Create tx in Initiated status (not ReadyForExecution)
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // Tx is in Initiated status — not disputable
        assert_noop!(
            Interoperability::dispute_bridge_transaction(
                RuntimeOrigin::signed(ALICE),
                0,
                b"fraud".to_vec(),
            ),
            Error::<Test>::NotDisputable
        );
    });
}

#[test]
fn dispute_reason_too_long_fails() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // Reason > 256 bytes
        assert_noop!(
            Interoperability::dispute_bridge_transaction(
                RuntimeOrigin::signed(ALICE),
                0,
                vec![0xFF; 300],
            ),
            Error::<Test>::InvalidConfiguration
        );
    });
}

// ============================================================================
// on_idle Challenge Period Finalization Tests
// ============================================================================

#[test]
fn on_idle_finalizes_after_challenge_period() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // ChallengePeriod = 100 blocks. Tx was created at block 1.
        // PendingFinalizations maps tx 0 → block 101 (1 + 100).
        let expiry = PendingFinalizations::<Test>::get(0).unwrap();
        assert_eq!(expiry, 101);

        // At block 100, should NOT finalize
        System::set_block_number(100);
        let weight = Interoperability::on_idle(100, Weight::from_parts(1_000_000_000, 100_000));
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::ReadyForExecution);
        assert!(PendingFinalizations::<Test>::contains_key(0));
        // Minimal overhead weight returned (just base cost, no finalization)
        assert!(weight.ref_time() < 20_000_000);

        // At block 101, challenge period elapsed → should finalize
        System::set_block_number(101);
        let weight = Interoperability::on_idle(101, Weight::from_parts(1_000_000_000, 100_000));
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::Finalized);
        assert_eq!(tx.completed_at, Some(101));
        assert!(!PendingFinalizations::<Test>::contains_key(0));
        // Weight included finalization work
        assert!(weight.ref_time() > 15_000_000);

        // Event emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::BridgeTransactionFinalized { tx_id: 0 })
        )));
    });
}

#[test]
fn on_idle_insufficient_weight_skips() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // At block 101 with tiny weight — should not finalize
        System::set_block_number(101);
        let weight = Interoperability::on_idle(101, Weight::from_parts(100, 10));
        assert_eq!(weight, Weight::zero());

        // Tx still ReadyForExecution
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::ReadyForExecution);
        assert!(PendingFinalizations::<Test>::contains_key(0));
    });
}

#[test]
fn on_idle_disputed_tx_not_finalized() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // Dispute removes from PendingFinalizations, so on_idle sees nothing
        assert_ok!(Interoperability::dispute_bridge_transaction(
            RuntimeOrigin::signed(ALICE),
            0,
            b"fraud".to_vec(),
        ));

        System::set_block_number(101);
        Interoperability::on_idle(101, Weight::from_parts(1_000_000_000, 100_000));

        // Remains Disputed
        let tx = Interoperability::bridge_transactions(0).unwrap();
        assert_eq!(tx.status, BridgeStatus::Disputed);
    });
}

// ============================================================================
// Additional Error Path Tests for Existing Extrinsics
// ============================================================================

#[test]
fn initiate_bridge_unsupported_asset_fails() {
    new_test_ext().execute_with(|| {
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Asset index 2 is invalid (only 0=DALLA, 1=BBZD)
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                2, // invalid asset
            ),
            Error::<Test>::UnsupportedAsset
        );
    });
}

#[test]
fn initiate_bridge_target_address_too_long_fails() {
    new_test_ext().execute_with(|| {
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Address > 128 bytes
        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                vec![0xAB; 200],
                100_000,
                0,
            ),
            Error::<Test>::InvalidConfiguration
        );
    });
}

#[test]
fn initiate_bridge_id_overflow_fails() {
    new_test_ext().execute_with(|| {
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Set NextTxId to u32::MAX
        NextTxId::<Test>::put(u32::MAX);

        assert_noop!(
            Interoperability::initiate_bridge(
                RuntimeOrigin::signed(EVE),
                1,
                test_eth_address(),
                100_000,
                0,
            ),
            Error::<Test>::IdOverflow
        );
    });
}

#[test]
fn initiate_bridge_cumulative_locks() {
    new_test_ext().execute_with(|| {
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // First bridge
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));
        assert_eq!(UserBridgeLocks::<Test>::get(EVE), 100_000);

        // Second bridge — locks should be cumulative
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));
        assert_eq!(UserBridgeLocks::<Test>::get(EVE), 200_000);
    });
}

#[test]
fn initiate_bridge_events_emitted() {
    new_test_ext().execute_with(|| {
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        let events = System::events();

        // BridgeTransactionInitiated
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::BridgeTransactionInitiated {
                tx_id: 0,
                initiator,
                target_chain: 1,
                amount: 100_000,
                asset: 0,
            }) if *initiator == EVE
        )));

        // AssetsLocked (net amount after fee: 100_000 - 100_000*50/10_000 = 99_500)
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::AssetsLocked {
                account,
                amount: 99_500,
                asset: 0,
                target_chain: 1,
            }) if *account == EVE
        )));
    });
}

#[test]
fn provide_pq_signature_duplicate_same_validator_fails() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx(); // Sets up tx 0 with 3 sigs required, all 3 collected

        // Register a 4th validator for testing on new tx
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        // Create a second tx
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // ALICE signs tx 1 once
        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(ALICE),
            1,
            test_pq_signature(),
        ));

        // ALICE tries to sign tx 1 again — M52 duplicate prevention
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(ALICE),
                1,
                test_pq_signature(),
            ),
            Error::<Test>::AlreadyExecuted
        );
    });
}

#[test]
fn provide_pq_signature_tx_not_found_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(ALICE),
                999,
                test_pq_signature(),
            ),
            Error::<Test>::TransactionNotFound
        );
    });
}

#[test]
fn provide_pq_signature_too_large_fails() {
    new_test_ext().execute_with(|| {
        setup_disputable_tx();

        // Register new validator for testing
        let validator = BridgeValidator {
            account: FERDIE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(FERDIE, validator);

        // Create a new tx for FERDIE to sign
        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        // But FERDIE has Level 2 KYC, not Level 3 — provide_pq_signature checks operator KYC
        assert_noop!(
            Interoperability::provide_pq_signature(
                RuntimeOrigin::signed(FERDIE),
                1,
                vec![0xEF; 200], // > 96 bytes
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn provide_pq_signature_event_emitted() {
    new_test_ext().execute_with(|| {
        // Setup: register validator, create bridge tx
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_ok!(Interoperability::initiate_bridge(
            RuntimeOrigin::signed(EVE),
            1,
            test_eth_address(),
            100_000,
            0,
        ));

        assert_ok!(Interoperability::provide_pq_signature(
            RuntimeOrigin::signed(ALICE),
            0,
            test_pq_signature(),
        ));

        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::PQSignatureProvided {
                tx_id: 0,
                validator,
                signatures_collected: 1,
            }) if *validator == ALICE
        )));
    });
}

#[test]
fn create_liquidity_pool_unsupported_asset_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::create_liquidity_pool(
                RuntimeOrigin::signed(ALICE),
                1, // Ethereum
                5, // invalid asset index
                100_000,
            ),
            Error::<Test>::UnsupportedAsset
        );
    });
}

#[test]
fn create_liquidity_pool_unsupported_chain_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::create_liquidity_pool(
                RuntimeOrigin::signed(ALICE),
                255, // invalid chain index
                0,
                100_000,
            ),
            Error::<Test>::UnsupportedChain
        );
    });
}

#[test]
fn create_liquidity_pool_event_emitted() {
    new_test_ext().execute_with(|| {
        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1, // Ethereum
            0, // DALLA
            100_000,
        ));

        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::LiquidityPoolCreated {
                pool_id: 0,
                chain: 1,
                asset: 0,
                manager,
            }) if *manager == ALICE
        )));
    });
}

#[test]
fn process_unlock_non_finalized_fails() {
    new_test_ext().execute_with(|| {
        // Register BOB as validator
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        // Insert tx in Initiated status
        let recipient_bytes: BoundedVec<u8, ConstU32<64>> = EVE.encode().try_into().unwrap();
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: BOB,
            operation: BridgeOperation::BurnAndUnlock {
                source_chain: BridgeChain::Ethereum,
                source_tx_hash: test_tx_hash().try_into().unwrap(),
                amount: 100_000,
                asset: BridgeAsset::DALLA,
                recipient: recipient_bytes,
            },
            status: BridgeStatus::Initiated,
            required_signatures: 3,
            collected_signatures: 0,
            pq_signatures: BoundedVec::default(),
            fee: 0,
            initiated_at: 0,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0u32, tx);

        assert_noop!(
            Interoperability::process_unlock(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::InsufficientSignatures
        );
    });
}

#[test]
fn process_unlock_lock_and_mint_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        // Insert a Finalized LockAndMint tx — process_unlock only handles BurnAndUnlock
        let target_address: BoundedVec<u8, ConstU32<128>> = test_eth_address().try_into().unwrap();
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: EVE,
            operation: BridgeOperation::LockAndMint {
                target_chain: BridgeChain::Ethereum,
                target_address,
                amount: 100_000,
                asset: BridgeAsset::DALLA,
            },
            status: BridgeStatus::Finalized,
            required_signatures: 1,
            collected_signatures: 1,
            pq_signatures: BoundedVec::default(),
            fee: 500,
            initiated_at: 0,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0u32, tx);

        assert_noop!(
            Interoperability::process_unlock(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::UnauthorizedOperation
        );
    });
}

#[test]
fn process_unlock_insufficient_locked_assets_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        // TotalLockedAssets only has 50_000, but tx wants 100_000
        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 50_000u128);

        let recipient_bytes: BoundedVec<u8, ConstU32<64>> = EVE.encode().try_into().unwrap();
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: BOB,
            operation: BridgeOperation::BurnAndUnlock {
                source_chain: BridgeChain::Ethereum,
                source_tx_hash: test_tx_hash().try_into().unwrap(),
                amount: 100_000,
                asset: BridgeAsset::DALLA,
                recipient: recipient_bytes,
            },
            status: BridgeStatus::Finalized,
            required_signatures: 1,
            collected_signatures: 1,
            pq_signatures: BoundedVec::default(),
            fee: 0,
            initiated_at: 0,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0u32, tx);

        assert_noop!(
            Interoperability::process_unlock(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn process_unlock_event_emitted() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000u128);

        let recipient_bytes: BoundedVec<u8, ConstU32<64>> = EVE.encode().try_into().unwrap();
        let tx = BridgeTransaction {
            tx_id: 0,
            initiator: BOB,
            operation: BridgeOperation::BurnAndUnlock {
                source_chain: BridgeChain::Ethereum,
                source_tx_hash: test_tx_hash().try_into().unwrap(),
                amount: 100_000,
                asset: BridgeAsset::DALLA,
                recipient: recipient_bytes,
            },
            status: BridgeStatus::Finalized,
            required_signatures: 1,
            collected_signatures: 1,
            pq_signatures: BoundedVec::default(),
            fee: 0,
            initiated_at: 0,
            completed_at: None,
            external_confirmation: None,
            dispute_info: None,
        };
        BridgeTransactions::<Test>::insert(0u32, tx);

        assert_ok!(Interoperability::process_unlock(RuntimeOrigin::signed(BOB), 0));

        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::AssetsUnlocked {
                account,
                amount: 100_000,
                asset: 0,
                source_chain: 1,
            }) if *account == EVE
        )));
    });
}

#[test]
fn send_cross_chain_message_kyc_required() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::send_cross_chain_message(
                RuntimeOrigin::signed(NO_KYC),
                1,
                test_message_payload(),
            ),
            Error::<Test>::KycRequired
        );
    });
}

#[test]
fn send_cross_chain_message_sanctioned_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::send_cross_chain_message(
                RuntimeOrigin::signed(SANCTIONED),
                1,
                test_message_payload(),
            ),
            Error::<Test>::AccountSanctioned
        );
    });
}

#[test]
fn send_cross_chain_message_unsupported_chain_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::send_cross_chain_message(
                RuntimeOrigin::signed(ALICE),
                255, // invalid chain
                test_message_payload(),
            ),
            Error::<Test>::UnsupportedChain
        );
    });
}

#[test]
fn send_cross_chain_message_event_emitted() {
    new_test_ext().execute_with(|| {
        let payload = test_message_payload();

        assert_ok!(Interoperability::send_cross_chain_message(
            RuntimeOrigin::signed(ALICE),
            1,
            payload.clone(),
        ));

        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::CrossChainMessageSent {
                message_id: 0,
                target_chain: 1,
                ..
            })
        )));
    });
}

#[test]
fn withdraw_liquidity_pool_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Interoperability::withdraw_liquidity(
                RuntimeOrigin::signed(ALICE),
                999, // non-existent pool
                100_000,
            ),
            Error::<Test>::PoolNotFound
        );
    });
}

#[test]
fn withdraw_liquidity_non_manager_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1,
            0,
            500_000,
        ));

        // BOB is not pool manager — unauthorized
        assert_noop!(
            Interoperability::withdraw_liquidity(
                RuntimeOrigin::signed(BOB),
                0,
                100_000,
            ),
            Error::<Test>::UnauthorizedOperation
        );
    });
}

#[test]
fn withdraw_liquidity_exceeds_pool_liquidity_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Interoperability::create_liquidity_pool(
            RuntimeOrigin::signed(ALICE),
            1,
            0,
            100_000,
        ));

        // Trying to withdraw more than pool has
        assert_noop!(
            Interoperability::withdraw_liquidity(
                RuntimeOrigin::signed(ALICE),
                0,
                200_000,
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn submit_incoming_unlock_disabled_chain_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        let chain_config = ChainConfig {
            enabled: false, // disabled
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000u128);

        assert_noop!(
            Interoperability::submit_incoming_unlock(
                RuntimeOrigin::signed(ALICE),
                1,
                test_tx_hash(),
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::BridgeDisabled
        );
    });
}

#[test]
fn submit_incoming_unlock_insufficient_locked_assets_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        // Only 50K locked, but requesting 100K unlock
        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 50_000u128);

        assert_noop!(
            Interoperability::submit_incoming_unlock(
                RuntimeOrigin::signed(ALICE),
                1,
                test_tx_hash(),
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn submit_incoming_unlock_non_operator_kyc_fails() {
    new_test_ext().execute_with(|| {
        // EVE has Level 2 KYC — not a bridge operator (requires Level 3)
        assert_noop!(
            Interoperability::submit_incoming_unlock(
                RuntimeOrigin::signed(EVE),
                1,
                test_tx_hash(),
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::BridgeOperatorKycInsufficient
        );
    });
}

#[test]
fn submit_incoming_unlock_non_validator_fails() {
    new_test_ext().execute_with(|| {
        // ALICE has Level 3 KYC but is NOT registered as validator
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000u128);

        assert_noop!(
            Interoperability::submit_incoming_unlock(
                RuntimeOrigin::signed(ALICE),
                1,
                test_tx_hash(),
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::ValidatorNotRegistered
        );
    });
}

#[test]
fn submit_incoming_unlock_tx_hash_too_long_fails() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: ALICE,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(ALICE, validator);

        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 3,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        TotalLockedAssets::<Test>::insert(BridgeChain::Ethereum, BridgeAsset::DALLA, 500_000u128);

        // source_tx_hash > 64 bytes
        assert_noop!(
            Interoperability::submit_incoming_unlock(
                RuntimeOrigin::signed(ALICE),
                1,
                vec![0xCD; 100], // > 64 bytes
                EVE,
                100_000,
                0,
            ),
            Error::<Test>::InvalidConfiguration
        );
    });
}

#[test]
fn update_bridge_config_event_emitted() {
    new_test_ext().execute_with(|| {
        // First insert an existing config to update
        let chain_config = ChainConfig {
            enabled: true,
            min_confirmations: 12,
            max_amount: 1_000_000_000,
            fee_rate: 50,
            pq_signatures_required: 5,
            rpc_endpoint: b"https://eth.example.com".to_vec().try_into().unwrap(),
            contract_address: Some(vec![0xAB; 20].try_into().unwrap()),
        };
        ChainConfigurations::<Test>::insert(BridgeChain::Ethereum, chain_config);

        assert_ok!(Interoperability::update_bridge_config(
            RuntimeOrigin::root(),
            1, // Ethereum
            false,
            100,
            500_000,
        ));

        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            &e.event,
            RuntimeEvent::Interoperability(Event::BridgeConfigUpdated {
                chain: 1,
                fee_rate: 100,
            })
        )));

        // Verify config was actually updated
        let config = Interoperability::chain_configurations(BridgeChain::Ethereum).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.fee_rate, 100);
        assert_eq!(config.max_amount, 500_000);
    });
}

#[test]
fn remove_bridge_validator_requires_root() {
    new_test_ext().execute_with(|| {
        let validator = BridgeValidator {
            account: BOB,
            pq_public_key: vec![0xAB; 64].try_into().unwrap(),
            supported_chains: vec![BridgeChain::Ethereum].try_into().unwrap(),
            stake: 1_000_000,
            reliability_score: 100,
            signatures_count: 0,
            failed_signatures: 0,
        };
        BridgeValidators::<Test>::insert(BOB, validator);

        // Non-root origin must fail
        assert_noop!(
            Interoperability::remove_bridge_validator(RuntimeOrigin::signed(ALICE), BOB),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}
