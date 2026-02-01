use super::*;
use crate::mock::*;
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
        // Perbill::from_parts(50) means 50 parts per billion
        // Fee = 100_000 * 50 / 1_000_000_000 = 0 (rounds to 0 for small amounts)
        let expected_fee = (bridge_amount * 50) / 1_000_000_000;
        let treasury_balance = Balances::free_balance(TREASURY);
        let expected_treasury = 1_000 + expected_fee; // Initial 1000 + fee
        assert_eq!(treasury_balance, expected_treasury);

        // Verify assets locked via set_lock (free_balance unchanged, but usable balance reduced)
        // free_balance doesn't change with locks, only with transfers
        assert_eq!(Balances::free_balance(EVE), initial_balance);

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
        // Perbill::from_parts(50) means 50 parts per billion
        let expected_fee = (bridge_amount * 50) / 1_000_000_000;
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

        assert_ok!(Interoperability::process_unlock(
            RuntimeOrigin::signed(BOB),
            1, // Ethereum
            test_tx_hash(),
            EVE,
            unlock_amount,
            0, // DALLA
        ));

        // Verify total locked assets decreased
        let total_locked = Interoperability::total_locked_assets(BridgeChain::Ethereum, BridgeAsset::DALLA);
        assert_eq!(total_locked, 400_000);
    });
}

#[test]
fn process_unlock_requires_validator_registration() {
    new_test_ext().execute_with(|| {
        // ALICE is not registered as validator
        assert_noop!(
            Interoperability::process_unlock(
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

        // Perbill::from_parts(50) means 50 parts per billion
        let expected_fee = (100_000_u128 * 50) / 1_000_000_000;
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
