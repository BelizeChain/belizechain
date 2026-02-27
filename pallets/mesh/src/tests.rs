//! Tests for the Meshtastic mesh network pallet

use crate::{mock::*, types::*, Error, Event, pallet::*};
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
            45_000_000_000,            // 45 bBZD
            1,                          // nonce
            H256::from([0xAA; 32]),    // signature hash
            gateway_id,
            vec![[0x11, 0x22, 0x33, 0x44], [0x55, 0x66, 0x77, 0x88]], // relay path
            3,  // 3 hops
            -80, // RSSI
            100, // SNR * 10
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

        // Check relay rewards accumulated
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
            42,          // block number
            block_hash,
            parent_hash,
            state_root,
            extrinsics_root,
            [0x01, 0x02, 0x03, 0x04], // author compact
            15,           // 15 extrinsics
            1707843600,   // timestamp
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
            5,  // reduce max hops
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
