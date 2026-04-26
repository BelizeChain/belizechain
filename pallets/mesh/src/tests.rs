//! Tests for the Meshtastic mesh network pallet

use crate::{mock::*, pallet::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

// ========================
// Node Registration Tests
// ========================

#[test]
fn register_client_node_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,  // ~17.45° N (Belize City)
            -882_000_000, // ~-88.20° W
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        // Verify node is stored
        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.owner, 1);
        assert_eq!(node.role, MeshNodeRole::Client);
        assert_eq!(node.hardware, MeshHardware::TBeam);
        assert!(!node.is_gateway);
        assert!(node.active);
        assert_eq!(node.reputation, 5000);

        // Verify network stats updated
        let stats = Mesh::network_stats();
        assert_eq!(stats.total_nodes, 1);

        // Verify owner mapping
        let owned = Mesh::nodes_by_owner(1);
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0], node_id);

        // Verify event
        System::assert_has_event(RuntimeEvent::Mesh(Event::NodeRegistered {
            owner: 1,
            node_id,
            role: MeshNodeRole::Client,
            hardware: MeshHardware::TBeam,
            district: BelizeDistrict::Belize,
        }));
    });
}

#[test]
fn register_gateway_node_requires_verified_kyc() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x10, 0x20, 0x30, 0x40];

        // Account 3 has basic KYC (level 1) - should fail for gateway
        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(3),
                node_id,
                MeshNodeRole::Gateway,
                MeshHardware::StationG2,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Urban,
            ),
            Error::<Test>::InsufficientKycForGateway
        );

        // Account 2 has verified KYC (level 2) - should succeed
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            node_id,
            MeshNodeRole::Gateway,
            MeshHardware::StationG2,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Urban,
        ));

        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert!(node.is_gateway);

        let stats = Mesh::network_stats();
        assert_eq!(stats.gateway_count, 1);
    });
}

#[test]
fn register_validator_relay_requires_validator() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xAA, 0xBB, 0xCC, 0xDD];

        // Account 1 is not a validator - should fail
        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(1),
                node_id,
                MeshNodeRole::ValidatorRelay,
                MeshHardware::TBeamSupreme,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                50,
                BelizeDistrict::Cayo,
                TerrainType::Jungle,
            ),
            Error::<Test>::ValidatorNotFound
        );

        // Account 10 is a validator - should succeed
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(10),
            node_id,
            MeshNodeRole::ValidatorRelay,
            MeshHardware::TBeamSupreme,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            50,
            BelizeDistrict::Cayo,
            TerrainType::Jungle,
        ));

        let stats = Mesh::network_stats();
        assert_eq!(stats.validator_relay_count, 1);
    });
}

#[test]
fn no_kyc_cannot_register_node() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xFF, 0xFF, 0xFF, 0xFF];

        // Account 4 has no KYC (level 0)
        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(4),
                node_id,
                MeshNodeRole::Client,
                MeshHardware::HeltecV3,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Urban,
            ),
            Error::<Test>::InsufficientKycLevel
        );
    });
}

#[test]
fn duplicate_node_registration_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(2),
                node_id,
                MeshNodeRole::Router,
                MeshHardware::RAKWisBlock,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Coastal,
            ),
            Error::<Test>::NodeAlreadyRegistered
        );
    });
}

// ========================
// Node Deregistration Tests
// ========================

#[test]
fn deregister_node_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        assert_ok!(Mesh::deregister_node(RuntimeOrigin::signed(1), node_id));

        // Node should be removed
        assert!(Mesh::mesh_nodes(node_id).is_none());

        // Owner list should be empty
        let owned = Mesh::nodes_by_owner(1);
        assert_eq!(owned.len(), 0);

        // Stats should decrement
        let stats = Mesh::network_stats();
        assert_eq!(stats.total_nodes, 0);
    });
}

#[test]
fn cannot_deregister_others_node() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        assert_noop!(
            Mesh::deregister_node(RuntimeOrigin::signed(2), node_id),
            Error::<Test>::NotNodeOwner
        );
    });
}

// ========================
// Heartbeat Tests
// ========================

#[test]
fn heartbeat_updates_last_seen() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Router,
            MeshHardware::RAKWisBlock,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        run_to_block(50);

        assert_ok!(Mesh::node_heartbeat(RuntimeOrigin::signed(1), node_id));

        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.last_seen, 50);
    });
}

// ========================
// Mesh Transaction Tests
// ========================

fn setup_gateway() -> MeshtasticNodeId {
    let gateway_id: MeshtasticNodeId = [0x6A, 0x7E, 0x8A, 0x99];

    // Account 2 has verified KYC (level 2)
    assert_ok!(Mesh::register_node(
        RuntimeOrigin::signed(2),
        gateway_id,
        MeshNodeRole::Gateway,
        MeshHardware::StationG2,
        LoRaRegion::US915,
        174_500_000,
        -882_000_000,
        5,
        BelizeDistrict::Belize,
        TerrainType::Urban,
    ));

    gateway_id
}

#[test]
fn submit_mesh_transaction_works() {
    new_test_ext().execute_with(|| {
        let gateway_id = setup_gateway();
        let tx_hash = H256::from([0x42; 32]);

        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            tx_hash,
            MeshTxType::TransferBbzd,
            [0x01, 0x02, 0x03, 0x04], // sender compact ID
            [0x05, 0x06, 0x07, 0x08], // recipient compact ID
            45_000_000_000,           // 45 bBZD
            1,                        // nonce
            H256::from([0xAA; 32]),   // signature hash
            gateway_id,
            vec![[0x11, 0x22, 0x33, 0x44], [0x55, 0x66, 0x77, 0x88]], // relay path
            3,                                                        // 3 hops
            -80,                                                      // RSSI
            100,                                                      // SNR * 10
        ));

        // Transaction should be pending
        assert!(Mesh::pending_mesh_transactions(tx_hash).is_some());

        // Gateway stats should update
        let gateway = Mesh::mesh_nodes(gateway_id).unwrap();
        assert_eq!(gateway.transactions_relayed, 1);

        // Network stats should update
        let stats = Mesh::network_stats();
        assert_eq!(stats.total_mesh_transactions, 1);
    });
}

#[test]
fn duplicate_mesh_transaction_rejected() {
    new_test_ext().execute_with(|| {
        let gateway_id = setup_gateway();
        let tx_hash = H256::from([0x42; 32]);

        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            tx_hash,
            MeshTxType::TransferDalla,
            [0x01, 0x02, 0x03, 0x04],
            [0x05, 0x06, 0x07, 0x08],
            100_000_000_000,
            1,
            H256::from([0xAA; 32]),
            gateway_id,
            vec![],
            1,
            -70,
            120,
        ));

        // Submit same tx_hash again should fail
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(2),
                tx_hash,
                MeshTxType::TransferDalla,
                [0x01, 0x02, 0x03, 0x04],
                [0x05, 0x06, 0x07, 0x08],
                100_000_000_000,
                1,
                H256::from([0xAA; 32]),
                gateway_id,
                vec![],
                1,
                -70,
                120,
            ),
            Error::<Test>::DuplicateMeshTransaction
        );
    });
}

#[test]
fn excessive_hops_rejected() {
    new_test_ext().execute_with(|| {
        let gateway_id = setup_gateway();
        let tx_hash = H256::from([0x99; 32]);

        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(2),
                tx_hash,
                MeshTxType::TransferDalla,
                [0x01, 0x02, 0x03, 0x04],
                [0x05, 0x06, 0x07, 0x08],
                100_000_000_000,
                1,
                H256::from([0xAA; 32]),
                gateway_id,
                vec![],
                15, // Exceeds default max_hops of 7
                -70,
                120,
            ),
            Error::<Test>::ExcessiveHopCount
        );
    });
}

// ========================
// Emergency Alert Tests
// ========================

#[test]
fn emergency_alert_issuance_works() {
    new_test_ext().execute_with(|| {
        // Account 5 is emergency authority (NEMO)
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::signed(5),
            AlertSeverity::Warning,
            EmergencyType::Hurricane,
            174_500_000,
            -882_000_000,
            50_000, // 50km radius
            b"Hurricane approaching Belize coast. Evacuate coastal areas.".to_vec(),
            1000, // ~1000 blocks duration
            BelizeDistrict::Belize,
        ));

        let alert = Mesh::emergency_alerts(0).unwrap();
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.alert_type, EmergencyType::Hurricane);
        assert!(!alert.resolved);

        let stats = Mesh::network_stats();
        assert_eq!(stats.total_emergency_alerts, 1);

        assert_eq!(Mesh::next_alert_id(), 1);
    });
}

#[test]
fn unauthorized_cannot_issue_emergency_alert() {
    new_test_ext().execute_with(|| {
        // Account 1 is NOT emergency authority
        assert_noop!(
            Mesh::issue_emergency_alert(
                RuntimeOrigin::signed(1),
                AlertSeverity::Emergency,
                EmergencyType::Flooding,
                174_500_000,
                -882_000_000,
                20_000,
                b"Flash flooding in Belize City".to_vec(),
                500,
                BelizeDistrict::Belize,
            ),
            Error::<Test>::NotEmergencyAuthority
        );
    });
}

#[test]
fn root_can_issue_emergency_alert() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Catastrophic,
            EmergencyType::Hurricane,
            174_500_000,
            -882_000_000,
            100_000,
            b"Category 5 hurricane. Take shelter immediately.".to_vec(),
            5000,
            BelizeDistrict::Belize,
        ));

        let alert = Mesh::emergency_alerts(0).unwrap();
        assert_eq!(alert.severity, AlertSeverity::Catastrophic);
    });
}

#[test]
fn resolve_emergency_alert_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::signed(5),
            AlertSeverity::Warning,
            EmergencyType::Flooding,
            174_500_000,
            -882_000_000,
            10_000,
            b"Flooding subsiding".to_vec(),
            500,
            BelizeDistrict::Belize,
        ));

        assert_ok!(Mesh::resolve_emergency_alert(RuntimeOrigin::signed(5), 0));

        let alert = Mesh::emergency_alerts(0).unwrap();
        assert!(alert.resolved);
    });
}

#[test]
fn confirm_emergency_alert_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        // Register a node
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        // Issue an alert
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::signed(5),
            AlertSeverity::Warning,
            EmergencyType::TropicalStorm,
            174_500_000,
            -882_000_000,
            30_000,
            b"Tropical storm warning".to_vec(),
            1000,
            BelizeDistrict::Belize,
        ));

        // Confirm from mesh node
        assert_ok!(Mesh::confirm_emergency_alert(
            RuntimeOrigin::signed(1),
            0,
            node_id,
        ));

        let alert = Mesh::emergency_alerts(0).unwrap();
        assert_eq!(alert.confirmations, 1);
    });
}

// ========================
// Relay Mining Tests
// ========================

#[test]
fn submit_relay_proof_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Router,
            MeshHardware::RAKWisBlock,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            100,
            BelizeDistrict::Cayo,
            TerrainType::Jungle,
        ));

        let content_hash = H256::from([0x55; 32]);
        let source = [0xAA, 0xBB, 0xCC, 0xDD];

        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            content_hash,
            source,
            RelayDestination::NearestGateway,
            -75,
            110,
        ));

        // Rewards are deferred until a second node confirms — register a confirmer node
        let confirmer_node_id: MeshtasticNodeId = [0x02, 0x03, 0x04, 0x05];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2), // different owner
            confirmer_node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Cayo,
            TerrainType::Jungle,
        ));

        // Confirm the relay proof to trigger reward accrual
        assert_ok!(Mesh::confirm_relay_proof(
            RuntimeOrigin::signed(2),
            node_id,
            0, // proof_index
        ));

        // Check relay rewards accumulated after confirmation
        let reward = Mesh::relay_rewards(1);
        assert_eq!(reward, 1_000_000_000); // 1 DALLA for transaction relay

        // Check node stats
        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.messages_relayed, 1);
        assert_eq!(node.reputation, 5001); // Slight boost

        // Check network stats
        let stats = Mesh::network_stats();
        assert_eq!(stats.total_relay_proofs, 1);
    });
}

// ========================
// Block Header Relay Tests
// ========================

#[test]
fn relay_block_header_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x7A, 0x11, 0xDA, 0x78];

        // Register validator relay node (account 10 is a validator)
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(10),
            node_id,
            MeshNodeRole::ValidatorRelay,
            MeshHardware::TBeamSupreme,
            LoRaRegion::US915,
            171_000_000,
            -889_000_000,
            200,
            BelizeDistrict::Cayo,
            TerrainType::Mountain,
        ));

        let block_hash = H256::from([0xBB; 32]);
        let parent_hash = H256::from([0xAA; 32]);
        let state_root = H256::from([0xCC; 32]);
        let extrinsics_root = H256::from([0xDD; 32]);

        assert_ok!(Mesh::relay_block_header(
            RuntimeOrigin::signed(10),
            node_id,
            42, // block number
            block_hash,
            parent_hash,
            state_root,
            extrinsics_root,
            [0x01, 0x02, 0x03, 0x04], // author compact
            15,                       // 15 extrinsics
            1707843600,               // timestamp
        ));

        let header = Mesh::mesh_block_headers(42).unwrap();
        assert_eq!(header.block_number, 42);
        assert_eq!(header.block_hash, block_hash);
        assert_eq!(header.extrinsic_count, 15);

        let stats = Mesh::network_stats();
        assert_eq!(stats.total_block_headers_relayed, 1);
    });
}

#[test]
fn non_validator_relay_cannot_relay_block_headers() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        // Register a client node (not validator relay)
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        assert_noop!(
            Mesh::relay_block_header(
                RuntimeOrigin::signed(1),
                node_id,
                42,
                H256::from([0xBB; 32]),
                H256::from([0xAA; 32]),
                H256::from([0xCC; 32]),
                H256::from([0xDD; 32]),
                [0x01, 0x02, 0x03, 0x04],
                15,
                1707843600,
            ),
            Error::<Test>::ValidatorNotFound
        );
    });
}

// ========================
// Governance Config Tests
// ========================

#[test]
fn update_mesh_config_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mesh::update_mesh_config(
            RuntimeOrigin::root(),
            5, // reduce max hops
            ChannelPreset::LongSlow,
            true,
            true,
            true,
            1,
            2,
        ));

        let config = Mesh::mesh_config();
        assert_eq!(config.max_hops, 5);
        assert_eq!(config.channel_preset, ChannelPreset::LongSlow);
    });
}

#[test]
fn non_governance_cannot_update_config() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::update_mesh_config(
                RuntimeOrigin::signed(1),
                5,
                ChannelPreset::LongSlow,
                true,
                true,
                true,
                1,
                2,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// ========================
// Location Update Tests
// ========================

#[test]
fn update_node_location_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::HeltecTracker,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        assert_ok!(Mesh::update_node_location(
            RuntimeOrigin::signed(1),
            node_id,
            180_000_000,  // Moved to ~18.0° N
            -880_000_000, // ~-88.0° W
            15,
        ));

        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.latitude, 180_000_000);
        assert_eq!(node.longitude, -880_000_000);
        assert_eq!(node.altitude, 15);
    });
}

// ========================
// Relay Reward Fund & Claim Tests
// ========================

#[test]
fn fund_relay_rewards_works() {
    new_test_ext().execute_with(|| {
        let pallet_account = Mesh::pallet_account_id();
        let before = Balances::free_balance(pallet_account);

        assert_ok!(Mesh::fund_relay_rewards(
            RuntimeOrigin::signed(1),
            1_000_000_000u128,
        ));

        assert_eq!(
            Balances::free_balance(pallet_account),
            before + 1_000_000_000
        );
    });
}

#[test]
fn fund_relay_rewards_zero_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::fund_relay_rewards(RuntimeOrigin::signed(1), 0u128),
            Error::<Test>::NoRelayRewards
        );
    });
}

#[test]
fn claim_relay_rewards_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xC1, 0xA1, 0x01, 0x01];

        // Register relay node owned by account 1
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        // Submit relay proof
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x77; 32]),
            [0xAA, 0xBB, 0xCC, 0xDD],
            RelayDestination::NearestGateway,
            -100,
            100,
        ));

        // Register a second node (different owner) to confirm
        let confirmer_node_id: MeshtasticNodeId = [0xC2, 0xA2, 0x02, 0x02];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            confirmer_node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        // Confirm the proof to accrue reward to account 1
        assert_ok!(Mesh::confirm_relay_proof(
            RuntimeOrigin::signed(2),
            node_id,
            0,
        ));

        let reward = Mesh::relay_rewards(1u64);
        assert!(reward > 0, "reward must have accrued after confirmation");

        // Fund the pallet so it can pay out (extra 1 satisfies KeepAlive ED)
        assert_ok!(Mesh::fund_relay_rewards(
            RuntimeOrigin::signed(2),
            reward + 1,
        ));

        let balance_before = Balances::free_balance(1u64);

        // Claim rewards
        assert_ok!(Mesh::claim_relay_rewards(RuntimeOrigin::signed(1)));

        assert_eq!(Balances::free_balance(1u64), balance_before + reward);
        assert_eq!(Mesh::relay_rewards(1u64), 0);
    });
}

#[test]
fn claim_relay_rewards_no_rewards_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::claim_relay_rewards(RuntimeOrigin::signed(1)),
            Error::<Test>::NoRelayRewards
        );
    });
}

// ============================================
// Extended Coverage Tests
// ============================================

// --- Registration error paths ---

#[test]
fn register_node_insufficient_deposit_fails() {
    new_test_ext().execute_with(|| {
        // Account 6 has only basic KYC (default=1) and no balance
        // Give it just under needed deposit
        let _ = Balances::force_set_balance(RuntimeOrigin::root(), 6, 5_000_000_000); // 5 DALLA < 10 DALLA deposit
        let node_id: MeshtasticNodeId = [0xA1, 0xA2, 0xA3, 0xA4];
        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(6),
                node_id,
                MeshNodeRole::Client,
                MeshHardware::TBeam,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Coastal,
            ),
            Error::<Test>::InsufficientDeposit
        );
    });
}

#[test]
fn register_too_many_nodes_fails() {
    new_test_ext().execute_with(|| {
        // Account 1 has 1000 DALLA, each node costs 10 DALLA deposit
        // BoundedVec limit is 10 nodes per owner
        for i in 0u8..10 {
            let node_id: MeshtasticNodeId = [0xB0 + i, 0x01, 0x01, 0x01];
            assert_ok!(Mesh::register_node(
                RuntimeOrigin::signed(1),
                node_id,
                MeshNodeRole::Client,
                MeshHardware::TBeam,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Coastal,
            ));
        }
        // 11th should fail
        let node_id: MeshtasticNodeId = [0xBB, 0x01, 0x01, 0x01];
        assert_noop!(
            Mesh::register_node(
                RuntimeOrigin::signed(1),
                node_id,
                MeshNodeRole::Client,
                MeshHardware::TBeam,
                LoRaRegion::US915,
                174_500_000,
                -882_000_000,
                5,
                BelizeDistrict::Belize,
                TerrainType::Coastal,
            ),
            Error::<Test>::TooManyNodes
        );
    });
}

#[test]
fn register_router_node_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xC1, 0xC2, 0xC3, 0xC4];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Router,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            50,
            BelizeDistrict::OrangeWalk,
            TerrainType::Jungle,
        ));
        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.role, MeshNodeRole::Router);
        assert!(!node.is_gateway);
    });
}

#[test]
fn register_emergency_beacon_node_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xE1, 0xE2, 0xE3, 0xE4];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::EmergencyBeacon,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            10,
            BelizeDistrict::Toledo,
            TerrainType::Mountain,
        ));
        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert_eq!(node.role, MeshNodeRole::EmergencyBeacon);
    });
}

// --- Deregistration edge cases ---

#[test]
fn deregister_gateway_updates_stats() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();
        let stats_before = Mesh::network_stats();
        assert_ok!(Mesh::deregister_node(RuntimeOrigin::signed(2), gw_id));
        let stats_after = Mesh::network_stats();
        assert_eq!(stats_after.total_nodes, stats_before.total_nodes - 1);
        assert!(Mesh::mesh_nodes(gw_id).is_none());
    });
}

#[test]
fn deregister_refunds_deposit() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xD1, 0xD2, 0xD3, 0xD4];
        let bal_before = Balances::free_balance(1u64);
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        // Balance reduced by deposit (10 DALLA reserved)
        assert!(Balances::free_balance(1u64) < bal_before);

        assert_ok!(Mesh::deregister_node(RuntimeOrigin::signed(1), node_id));
        // Balance fully restored
        assert_eq!(Balances::free_balance(1u64), bal_before);
    });
}

#[test]
fn deregister_nonexistent_node_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0xFF, 0xFF, 0xFF, 0xFF];
        assert_noop!(
            Mesh::deregister_node(RuntimeOrigin::signed(1), node_id),
            Error::<Test>::NodeNotFound
        );
    });
}

#[test]
fn deregister_multi_node_owner_removes_correct_node() {
    new_test_ext().execute_with(|| {
        let node_a: MeshtasticNodeId = [0xA1, 0xA1, 0xA1, 0xA1];
        let node_b: MeshtasticNodeId = [0xB1, 0xB1, 0xB1, 0xB1];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_a,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_b,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_eq!(Mesh::nodes_by_owner(1u64).len(), 2);

        assert_ok!(Mesh::deregister_node(RuntimeOrigin::signed(1), node_a));
        let owned = Mesh::nodes_by_owner(1u64);
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0], node_b);
        assert!(Mesh::mesh_nodes(node_a).is_none());
        assert!(Mesh::mesh_nodes(node_b).is_some());
    });
}

// --- Update Location ---

#[test]
fn update_location_invalid_coordinates_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::update_node_location(RuntimeOrigin::signed(1), node_id, 999_999_999, 0, 0),
            Error::<Test>::InvalidCoordinates
        );
    });
}

#[test]
fn update_location_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::update_node_location(
                RuntimeOrigin::signed(2),
                node_id,
                175_000_000,
                -883_000_000,
                10
            ),
            Error::<Test>::NotNodeOwner
        );
    });
}

#[test]
fn update_location_nonexistent_node_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::update_node_location(
                RuntimeOrigin::signed(1),
                [0xFF; 4],
                174_500_000,
                -882_000_000,
                5
            ),
            Error::<Test>::NodeNotFound
        );
    });
}

// --- Heartbeat edge cases ---

#[test]
fn heartbeat_reactivates_inactive_node() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        // Advance past heartbeat timeout (100 blocks)
        run_to_block(200);
        // Heartbeat should reactivate
        assert_ok!(Mesh::node_heartbeat(RuntimeOrigin::signed(1), node_id));
        let node = Mesh::mesh_nodes(node_id).unwrap();
        assert!(node.active);
        assert_eq!(node.last_seen, 200);
    });
}

#[test]
fn heartbeat_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::node_heartbeat(RuntimeOrigin::signed(2), node_id),
            Error::<Test>::NotNodeOwner
        );
    });
}

#[test]
fn heartbeat_nonexistent_node_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::node_heartbeat(RuntimeOrigin::signed(1), [0xFF; 4]),
            Error::<Test>::NodeNotFound
        );
    });
}

// --- Mesh Transaction error paths ---

#[test]
fn mesh_tx_invalid_signature_fails() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(2),
                H256::from([0x42; 32]),
                MeshTxType::TransferBbzd,
                [0x01; 4],
                [0x02; 4],
                1_000_000,
                1,
                H256::zero(), // zero signature
                gw_id,
                vec![],
                1,
                -80,
                100,
            ),
            Error::<Test>::InvalidMeshSignature
        );
    });
}

#[test]
fn mesh_tx_inactive_gateway_fails() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();
        // Manually set gateway as inactive
        MeshNodes::<Test>::mutate(gw_id, |maybe_node| {
            if let Some(node) = maybe_node {
                node.active = false;
            }
        });
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(2),
                H256::from([0x42; 32]),
                MeshTxType::TransferBbzd,
                [0x01; 4],
                [0x02; 4],
                1_000_000,
                1,
                H256::from([0xAA; 32]),
                gw_id,
                vec![],
                1,
                -80,
                100,
            ),
            Error::<Test>::NodeInactive
        );
    });
}

#[test]
fn mesh_tx_not_gateway_owner_fails() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway(); // owned by account 2
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(1), // not the gateway owner
                H256::from([0x42; 32]),
                MeshTxType::TransferBbzd,
                [0x01; 4],
                [0x02; 4],
                1_000_000,
                1,
                H256::from([0xAA; 32]),
                gw_id,
                vec![],
                1,
                -80,
                100,
            ),
            Error::<Test>::NotNodeOwner
        );
    });
}

#[test]
fn mesh_tx_nonexistent_gateway_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(1),
                H256::from([0x42; 32]),
                MeshTxType::TransferBbzd,
                [0x01; 4],
                [0x02; 4],
                1_000_000,
                1,
                H256::from([0xAA; 32]),
                [0xFF; 4],
                vec![],
                1,
                -80,
                100,
            ),
            Error::<Test>::NodeNotFound
        );
    });
}

#[test]
fn mesh_tx_updates_gateway_transactions_relayed() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();
        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            H256::from([0x42; 32]),
            MeshTxType::TransferBbzd,
            [0x01; 4],
            [0x02; 4],
            1_000_000,
            1,
            H256::from([0xAA; 32]),
            gw_id,
            vec![],
            1,
            -80,
            100,
        ));
        let node = Mesh::mesh_nodes(gw_id).unwrap();
        assert_eq!(node.transactions_relayed, 1);

        // Submit another
        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            H256::from([0x43; 32]),
            MeshTxType::TransferBbzd,
            [0x01; 4],
            [0x02; 4],
            2_000_000,
            2,
            H256::from([0xBB; 32]),
            gw_id,
            vec![],
            1,
            -80,
            100,
        ));
        assert_eq!(Mesh::mesh_nodes(gw_id).unwrap().transactions_relayed, 2);
    });
}

// --- Relay Proof error paths ---

#[test]
fn relay_proof_inactive_node_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        // Manually set node as inactive
        MeshNodes::<Test>::mutate(node_id, |maybe_node| {
            if let Some(node) = maybe_node {
                node.active = false;
            }
        });
        assert_noop!(
            Mesh::submit_relay_proof(
                RuntimeOrigin::signed(1),
                node_id,
                RelayType::Transaction,
                H256::from([0x11; 32]),
                [0xA1; 4],
                RelayDestination::Node([0xB1; 4]),
                -80,
                100,
            ),
            Error::<Test>::NodeInactive
        );
    });
}

#[test]
fn relay_proof_not_owner_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::submit_relay_proof(
                RuntimeOrigin::signed(2),
                node_id,
                RelayType::Transaction,
                H256::from([0x11; 32]),
                [0xA1; 4],
                RelayDestination::Node([0xB1; 4]),
                -80,
                100,
            ),
            Error::<Test>::NotNodeOwner
        );
    });
}

#[test]
fn relay_proof_nonexistent_node_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::submit_relay_proof(
                RuntimeOrigin::signed(1),
                [0xFF; 4],
                RelayType::Transaction,
                H256::from([0x11; 32]),
                [0xA1; 4],
                RelayDestination::Node([0xB1; 4]),
                -80,
                100,
            ),
            Error::<Test>::NodeNotFound
        );
    });
}

#[test]
fn relay_proof_disabled_fails() {
    new_test_ext().execute_with(|| {
        // Disable relay mining via config
        assert_ok!(Mesh::update_mesh_config(
            RuntimeOrigin::root(),
            8,
            ChannelPreset::LongFast,
            false, // relay_mining_active = false
            true,
            true,
            1,
            2,
        ));
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::submit_relay_proof(
                RuntimeOrigin::signed(1),
                node_id,
                RelayType::Transaction,
                H256::from([0x11; 32]),
                [0xA1; 4],
                RelayDestination::Node([0xB1; 4]),
                -80,
                100,
            ),
            Error::<Test>::RelayMiningDisabled
        );
    });
}

#[test]
fn relay_proof_updates_messages_relayed_and_reputation() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        let node_before = Mesh::mesh_nodes(node_id).unwrap();
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));
        let node_after = Mesh::mesh_nodes(node_id).unwrap();
        assert!(node_after.messages_relayed > node_before.messages_relayed);
        assert!(node_after.reputation >= node_before.reputation);
    });
}

// --- Confirm Relay Proof error paths ---

#[test]
fn confirm_relay_proof_own_proof_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));
        // Owner trying to confirm own proof
        assert_noop!(
            Mesh::confirm_relay_proof(RuntimeOrigin::signed(1), node_id, 0),
            Error::<Test>::CannotConfirmOwnProof
        );
    });
}

#[test]
fn confirm_relay_proof_index_out_of_range_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));
        // Register confirmer's own node so they pass the NodeNotFound check
        let confirmer_node: MeshtasticNodeId = [0xC2, 0xC2, 0xC2, 0xC2];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            confirmer_node,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        // Bad index
        assert_noop!(
            Mesh::confirm_relay_proof(RuntimeOrigin::signed(2), node_id, 99),
            Error::<Test>::ProofIndexOutOfRange
        );
    });
}

#[test]
fn confirm_relay_proof_already_confirmed_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));
        // Register confirmer node
        let confirmer_node: MeshtasticNodeId = [0xC1, 0xC1, 0xC1, 0xC1];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            confirmer_node,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::confirm_relay_proof(
            RuntimeOrigin::signed(2),
            node_id,
            0
        ));
        // Second confirm fails
        assert_noop!(
            Mesh::confirm_relay_proof(RuntimeOrigin::signed(2), node_id, 0),
            Error::<Test>::ProofAlreadyConfirmed
        );
    });
}

// --- Emergency Alert error paths ---

#[test]
fn emergency_alert_system_disabled_fails() {
    new_test_ext().execute_with(|| {
        // Disable emergency system
        assert_ok!(Mesh::update_mesh_config(
            RuntimeOrigin::root(),
            8,
            ChannelPreset::LongFast,
            true,
            false, // emergency_system_active = false
            true,
            1,
            2,
        ));
        assert_noop!(
            Mesh::issue_emergency_alert(
                RuntimeOrigin::signed(5), // emergency authority
                AlertSeverity::Emergency,
                EmergencyType::Hurricane,
                174_500_000,
                -882_000_000,
                10_000,
                b"Hurricane incoming".to_vec(),
                100,
                BelizeDistrict::Belize,
            ),
            Error::<Test>::EmergencySystemDisabled
        );
    });
}

#[test]
fn emergency_alert_message_too_long_fails() {
    new_test_ext().execute_with(|| {
        let long_msg = vec![0x41u8; 200]; // > 128 bytes
        assert_noop!(
            Mesh::issue_emergency_alert(
                RuntimeOrigin::signed(5),
                AlertSeverity::Catastrophic,
                EmergencyType::Hurricane,
                174_500_000,
                -882_000_000,
                10_000,
                long_msg,
                100,
                BelizeDistrict::Belize,
            ),
            Error::<Test>::MessageTooLong
        );
    });
}

#[test]
fn resolve_alert_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Mesh::resolve_emergency_alert(RuntimeOrigin::root(), 999),
            Error::<Test>::AlertNotFound
        );
    });
}

#[test]
fn resolve_alert_already_resolved_fails() {
    new_test_ext().execute_with(|| {
        // Issue alert
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Emergency,
            EmergencyType::Hurricane,
            174_500_000,
            -882_000_000,
            10_000,
            b"Storm".to_vec(),
            100,
            BelizeDistrict::Belize,
        ));
        let alert_id = Mesh::next_alert_id() - 1;
        // Resolve once
        assert_ok!(Mesh::resolve_emergency_alert(
            RuntimeOrigin::root(),
            alert_id
        ));
        // Resolve again
        assert_noop!(
            Mesh::resolve_emergency_alert(RuntimeOrigin::root(), alert_id),
            Error::<Test>::AlertAlreadyResolved
        );
    });
}

#[test]
fn confirm_alert_not_found_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_noop!(
            Mesh::confirm_emergency_alert(RuntimeOrigin::signed(1), 999, node_id),
            Error::<Test>::AlertNotFound
        );
    });
}

#[test]
fn confirm_alert_duplicate_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Emergency,
            EmergencyType::Flooding,
            174_500_000,
            -882_000_000,
            5_000,
            b"Flood".to_vec(),
            50,
            BelizeDistrict::Belize,
        ));
        let alert_id = Mesh::next_alert_id() - 1;
        assert_ok!(Mesh::confirm_emergency_alert(
            RuntimeOrigin::signed(1),
            alert_id,
            node_id
        ));
        assert_noop!(
            Mesh::confirm_emergency_alert(RuntimeOrigin::signed(1), alert_id, node_id),
            Error::<Test>::AlertAlreadyConfirmed
        );
    });
}

#[test]
fn confirm_alert_not_node_owner_fails() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Advisory,
            EmergencyType::Wildfire,
            174_500_000,
            -882_000_000,
            2_000,
            b"Fire".to_vec(),
            50,
            BelizeDistrict::Cayo,
        ));
        let alert_id = Mesh::next_alert_id() - 1;
        assert_noop!(
            Mesh::confirm_emergency_alert(RuntimeOrigin::signed(2), alert_id, node_id),
            Error::<Test>::NotNodeOwner
        );
    });
}

#[test]
fn multiple_alerts_same_district_track_count() {
    new_test_ext().execute_with(|| {
        for i in 0u8..3 {
            assert_ok!(Mesh::issue_emergency_alert(
                RuntimeOrigin::root(),
                AlertSeverity::Emergency,
                EmergencyType::Hurricane,
                174_500_000,
                -882_000_000,
                10_000,
                vec![0x41 + i; 10],
                100,
                BelizeDistrict::Belize,
            ));
        }
        assert_eq!(
            ActiveAlertCountPerDistrict::<Test>::get(BelizeDistrict::Belize),
            3
        );
        // Resolve one
        assert_ok!(Mesh::resolve_emergency_alert(RuntimeOrigin::root(), 0));
        assert_eq!(
            ActiveAlertCountPerDistrict::<Test>::get(BelizeDistrict::Belize),
            2
        );
    });
}

// --- Block Header Relay error paths ---

#[test]
fn relay_header_already_exists_fails() {
    new_test_ext().execute_with(|| {
        // Register validator relay node
        let vr_id: MeshtasticNodeId = [0xA0, 0xA1, 0xA2, 0xA3];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(10),
            vr_id,
            MeshNodeRole::ValidatorRelay,
            MeshHardware::StationG2,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            50,
            BelizeDistrict::Belize,
            TerrainType::Urban,
        ));
        // First relay
        assert_ok!(Mesh::relay_block_header(
            RuntimeOrigin::signed(10),
            vr_id,
            42,
            H256::from([0x01; 32]),
            H256::from([0x02; 32]),
            H256::from([0x03; 32]),
            H256::from([0x04; 32]),
            [0x11; 4],
            5,
            1000,
        ));
        // Duplicate
        assert_noop!(
            Mesh::relay_block_header(
                RuntimeOrigin::signed(10),
                vr_id,
                42,
                H256::from([0x01; 32]),
                H256::from([0x02; 32]),
                H256::from([0x03; 32]),
                H256::from([0x04; 32]),
                [0x11; 4],
                5,
                1000,
            ),
            Error::<Test>::HeaderAlreadyExists
        );
    });
}

#[test]
fn relay_header_disabled_fails() {
    new_test_ext().execute_with(|| {
        // Disable validator relay
        assert_ok!(Mesh::update_mesh_config(
            RuntimeOrigin::root(),
            8,
            ChannelPreset::LongFast,
            true,
            true,
            false, // validator_relay_active = false
            1,
            2,
        ));
        let vr_id: MeshtasticNodeId = [0xA0, 0xA1, 0xA2, 0xA3];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(10),
            vr_id,
            MeshNodeRole::ValidatorRelay,
            MeshHardware::StationG2,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            50,
            BelizeDistrict::Belize,
            TerrainType::Urban,
        ));
        assert_noop!(
            Mesh::relay_block_header(
                RuntimeOrigin::signed(10),
                vr_id,
                42,
                H256::from([0x01; 32]),
                H256::from([0x02; 32]),
                H256::from([0x03; 32]),
                H256::from([0x04; 32]),
                [0x11; 4],
                5,
                1000,
            ),
            Error::<Test>::RelayMiningDisabled
        );
    });
}

// --- Fund Relay Rewards edge cases ---

#[test]
fn fund_relay_rewards_insufficient_balance_fails() {
    new_test_ext().execute_with(|| {
        // Account 4 has 50 DALLA, try to fund more
        assert!(Mesh::fund_relay_rewards(
            RuntimeOrigin::signed(4),
            100_000_000_000_000, // way more than balance
        )
        .is_err());
    });
}

// --- Config update ---

#[test]
fn config_change_affects_max_hops() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();

        // Set max_hops to 2
        assert_ok!(Mesh::update_mesh_config(
            RuntimeOrigin::root(),
            2,
            ChannelPreset::LongFast,
            true,
            true,
            true,
            1,
            2,
        ));

        // Submit with 3 hops should now fail (max is 2)
        assert_noop!(
            Mesh::submit_mesh_transaction(
                RuntimeOrigin::signed(2),
                H256::from([0x42; 32]),
                MeshTxType::TransferBbzd,
                [0x01; 4],
                [0x02; 4],
                1_000_000,
                1,
                H256::from([0xAA; 32]),
                gw_id,
                vec![[0x11; 4], [0x22; 4], [0x33; 4]],
                3,
                -80,
                100,
            ),
            Error::<Test>::ExcessiveHopCount
        );

        // Submit with 2 hops should work
        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            H256::from([0x42; 32]),
            MeshTxType::TransferBbzd,
            [0x01; 4],
            [0x02; 4],
            1_000_000,
            1,
            H256::from([0xAA; 32]),
            gw_id,
            vec![[0x11; 4], [0x22; 4]],
            2,
            -80,
            100,
        ));
    });
}

// --- Network Stats verification ---

#[test]
fn network_stats_track_registrations_accurately() {
    new_test_ext().execute_with(|| {
        assert_eq!(Mesh::network_stats().total_nodes, 0);
        let node_a: MeshtasticNodeId = [0xA1; 4];
        let node_b: MeshtasticNodeId = [0xB1; 4];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_a,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_eq!(Mesh::network_stats().total_nodes, 1);

        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            node_b,
            MeshNodeRole::Gateway,
            MeshHardware::StationG2,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Urban,
        ));
        assert_eq!(Mesh::network_stats().total_nodes, 2);

        assert_ok!(Mesh::deregister_node(RuntimeOrigin::signed(1), node_a));
        assert_eq!(Mesh::network_stats().total_nodes, 1);
    });
}

#[test]
fn network_stats_track_mesh_transactions() {
    new_test_ext().execute_with(|| {
        let gw_id = setup_gateway();
        assert_eq!(Mesh::network_stats().total_mesh_transactions, 0);
        assert_ok!(Mesh::submit_mesh_transaction(
            RuntimeOrigin::signed(2),
            H256::from([0x42; 32]),
            MeshTxType::TransferBbzd,
            [0x01; 4],
            [0x02; 4],
            1_000_000,
            1,
            H256::from([0xAA; 32]),
            gw_id,
            vec![],
            1,
            -80,
            100,
        ));
        assert_eq!(Mesh::network_stats().total_mesh_transactions, 1);
    });
}

#[test]
fn network_stats_track_relay_proofs() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_eq!(Mesh::network_stats().total_relay_proofs, 0);
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));
        assert_eq!(Mesh::network_stats().total_relay_proofs, 1);
    });
}

#[test]
fn network_stats_track_emergency_alerts() {
    new_test_ext().execute_with(|| {
        assert_eq!(Mesh::network_stats().total_emergency_alerts, 0);
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Catastrophic,
            EmergencyType::Hurricane,
            174_500_000,
            -882_000_000,
            50_000,
            b"Cat5".to_vec(),
            500,
            BelizeDistrict::Belize,
        ));
        assert_eq!(Mesh::network_stats().total_emergency_alerts, 1);
    });
}

#[test]
fn network_stats_track_block_headers_relayed() {
    new_test_ext().execute_with(|| {
        let vr_id: MeshtasticNodeId = [0xA0, 0xA1, 0xA2, 0xA3];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(10),
            vr_id,
            MeshNodeRole::ValidatorRelay,
            MeshHardware::StationG2,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            50,
            BelizeDistrict::Belize,
            TerrainType::Urban,
        ));
        assert_eq!(Mesh::network_stats().total_block_headers_relayed, 0);
        assert_ok!(Mesh::relay_block_header(
            RuntimeOrigin::signed(10),
            vr_id,
            100,
            H256::from([0x01; 32]),
            H256::from([0x02; 32]),
            H256::from([0x03; 32]),
            H256::from([0x04; 32]),
            [0x11; 4],
            10,
            5000,
        ));
        assert_eq!(Mesh::network_stats().total_block_headers_relayed, 1);
    });
}

// --- Catastrophic alert tracking ---

#[test]
fn catastrophic_alert_count_tracked() {
    new_test_ext().execute_with(|| {
        assert_eq!(CatastrophicAlertCount::<Test>::get(), 0);
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Catastrophic,
            EmergencyType::Earthquake,
            174_500_000,
            -882_000_000,
            100_000,
            b"Major quake".to_vec(),
            1000,
            BelizeDistrict::Belize,
        ));
        assert_eq!(CatastrophicAlertCount::<Test>::get(), 1);

        // Resolve it
        let alert_id = Mesh::next_alert_id() - 1;
        assert_ok!(Mesh::resolve_emergency_alert(
            RuntimeOrigin::root(),
            alert_id
        ));
        assert_eq!(CatastrophicAlertCount::<Test>::get(), 0);
    });
}

// --- Relay types beyond Transaction ---

#[test]
fn relay_proof_block_header_type_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::BlockHeader,
            H256::from([0x22; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -90,
            80,
        ));
        let proofs = Mesh::relay_proofs(node_id);
        assert_eq!(proofs.len(), 1);
        assert_eq!(proofs[0].relay_type, RelayType::BlockHeader);
    });
}

#[test]
fn relay_proof_emergency_alert_type_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::EmergencyAlert,
            H256::from([0x33; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -70,
            120,
        ));
        let proofs = Mesh::relay_proofs(node_id);
        assert_eq!(proofs[0].relay_type, RelayType::EmergencyAlert);
    });
}

#[test]
fn relay_proof_heartbeat_type_works() {
    new_test_ext().execute_with(|| {
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Heartbeat,
            H256::from([0x44; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -60,
            150,
        ));
        let proofs = Mesh::relay_proofs(node_id);
        assert_eq!(proofs[0].relay_type, RelayType::Heartbeat);
    });
}

// --- Relay rewards flow ---

#[test]
fn claim_relay_rewards_clears_proofs_and_stats() {
    new_test_ext().execute_with(|| {
        // Register relayer
        let node_id: MeshtasticNodeId = [0x01, 0x02, 0x03, 0x04];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(1),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));

        // Submit proof
        assert_ok!(Mesh::submit_relay_proof(
            RuntimeOrigin::signed(1),
            node_id,
            RelayType::Transaction,
            H256::from([0x11; 32]),
            [0xA1; 4],
            RelayDestination::Node([0xB1; 4]),
            -80,
            100,
        ));

        // Register confirmer and confirm
        let confirmer_node: MeshtasticNodeId = [0xC1; 4];
        assert_ok!(Mesh::register_node(
            RuntimeOrigin::signed(2),
            confirmer_node,
            MeshNodeRole::Client,
            MeshHardware::TBeam,
            LoRaRegion::US915,
            174_500_000,
            -882_000_000,
            5,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        ));
        assert_ok!(Mesh::confirm_relay_proof(
            RuntimeOrigin::signed(2),
            node_id,
            0
        ));

        let reward = Mesh::relay_rewards(1u64);
        assert!(reward > 0);

        // Fund pallet
        assert_ok!(Mesh::fund_relay_rewards(
            RuntimeOrigin::signed(1),
            reward + 1
        ));

        // Claim
        assert_ok!(Mesh::claim_relay_rewards(RuntimeOrigin::signed(1)));
        assert_eq!(Mesh::relay_rewards(1u64), 0);
        // Proofs cleared
        assert_eq!(Mesh::relay_proofs(node_id).len(), 0);
    });
}

// --- Resolve via signed authority ---

#[test]
fn resolve_alert_signed_authority_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::signed(5), // emergency authority
            AlertSeverity::Warning,
            EmergencyType::Flooding,
            174_500_000,
            -882_000_000,
            5_000,
            b"Flood receding".to_vec(),
            50,
            BelizeDistrict::StannCreek,
        ));
        let alert_id = Mesh::next_alert_id() - 1;
        assert_ok!(Mesh::resolve_emergency_alert(
            RuntimeOrigin::signed(5),
            alert_id
        ));
        let alert = Mesh::emergency_alerts(alert_id).unwrap();
        assert!(alert.resolved);
    });
}

#[test]
fn resolve_alert_non_authority_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mesh::issue_emergency_alert(
            RuntimeOrigin::root(),
            AlertSeverity::Emergency,
            EmergencyType::Wildfire,
            174_500_000,
            -882_000_000,
            5_000,
            b"Fire".to_vec(),
            50,
            BelizeDistrict::Cayo,
        ));
        let alert_id = Mesh::next_alert_id() - 1;
        assert_noop!(
            Mesh::resolve_emergency_alert(RuntimeOrigin::signed(1), alert_id),
            Error::<Test>::NotEmergencyAuthority
        );
    });
}
