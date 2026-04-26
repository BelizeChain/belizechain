//! Benchmarking for pallet-belize-mesh
//!
//! Provides machine-profiled weight functions for all 14 extrinsics
//! exposed via the `WeightInfo` trait.
//!
//! NOTE: `register_node` depends on cross-pallet KYC via `T::Identity::get_kyc_level`.
//! For benchmarks, writing the node directly to storage bypasses KYC for derived
//! benchmarks. The `register_node` benchmark itself will use the real extrinsic
//! (KYC must return >= 1 for the caller, or a runtime-benchmarks bypass is needed).

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::pallet::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::traits::Bounded;
use sp_std::vec;

/// Helper: insert a mesh node directly into storage (bypasses KYC check).
fn insert_node<T: Config>(
    owner: &T::AccountId,
    node_id: MeshtasticNodeId,
    role: MeshNodeRole,
    is_gateway: bool,
) {
    let current_block = <frame_system::Pallet<T>>::block_number();
    let node = MeshNode {
        owner: owner.clone(),
        node_id,
        role: role.clone(),
        hardware: MeshHardware::Unknown,
        region: LoRaRegion::US915,
        latitude: 174_000_000,   // ~17.4°N (Belize)
        longitude: -883_000_000, // ~-88.3°W
        altitude: 10,
        is_gateway,
        messages_relayed: 0,
        transactions_relayed: 0,
        emergency_alerts_sent: 0,
        registered_at: current_block,
        last_seen: current_block,
        reputation: 5000,
        active: true,
    };
    MeshNodes::<T>::insert(node_id, node);

    // Update NodesByOwner
    NodesByOwner::<T>::mutate(owner, |owned| {
        let _ = owned.try_push(node_id);
    });

    // Update NetworkStats
    NetworkStats::<T>::mutate(|stats| {
        stats.total_nodes += 1;
        if is_gateway {
            stats.gateway_count += 1;
        }
        match &role {
            MeshNodeRole::Router | MeshNodeRole::RouterClient => stats.router_count += 1,
            MeshNodeRole::ValidatorRelay => stats.validator_relay_count += 1,
            MeshNodeRole::EmergencyBeacon => stats.emergency_beacon_count += 1,
            _ => {}
        }
    });
}

/// Helper: create a funded account with a registered Client node.
fn setup_node_owner<T: Config>(seed: u32) -> (T::AccountId, MeshtasticNodeId) {
    let owner: T::AccountId = account("mesh_owner", seed, 0);
    T::Currency::make_free_balance_be(
        &owner,
        <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 4u32.into(),
    );
    let node_id: MeshtasticNodeId = [0, 0, 0, seed as u8];
    insert_node::<T>(&owner, node_id, MeshNodeRole::Client, false);
    (owner, node_id)
}

/// Helper: create a gateway node owner.
fn setup_gateway_owner<T: Config>(seed: u32) -> (T::AccountId, MeshtasticNodeId) {
    let owner: T::AccountId = account("gateway", seed, 0);
    T::Currency::make_free_balance_be(
        &owner,
        <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 4u32.into(),
    );
    let node_id: MeshtasticNodeId = [1, 0, 0, seed as u8];
    insert_node::<T>(&owner, node_id, MeshNodeRole::Gateway, true);
    (owner, node_id)
}

/// Helper: create a ValidatorRelay node owner.
fn setup_validator_relay<T: Config>(seed: u32) -> (T::AccountId, MeshtasticNodeId) {
    let owner: T::AccountId = account("validator_relay", seed, 0);
    T::Currency::make_free_balance_be(
        &owner,
        <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 4u32.into(),
    );
    let node_id: MeshtasticNodeId = [2, 0, 0, seed as u8];
    insert_node::<T>(&owner, node_id, MeshNodeRole::ValidatorRelay, false);
    (owner, node_id)
}

#[benchmarks]
mod benchmarks {
    use super::*;

    // ── 1. register_node ─────────────────────────────────────────────
    // NOTE: Requires T::Identity::get_kyc_level to return >= 1 for caller.
    // Also requires NodeRegistrationDeposit available (funded via make_free_balance_be).
    #[benchmark]
    fn register_node() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(
            &caller,
            <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 2u32.into(),
        );
        // Set permissive mesh config: min_kyc_for_registration = 0
        let config = MeshNetworkConfig {
            min_kyc_for_registration: 0,
            min_kyc_for_gateway: 0,
            ..Default::default()
        };
        MeshConfig::<T>::put(config);

        let node_id: MeshtasticNodeId = [9, 9, 9, 1];

        #[extrinsic_call]
        register_node(
            RawOrigin::Signed(caller),
            node_id,
            MeshNodeRole::Client,
            MeshHardware::Unknown,
            LoRaRegion::US915,
            174_000_000i32,  // ~17.4°N
            -883_000_000i32, // ~-88.3°W
            10i16,
            BelizeDistrict::Belize,
            TerrainType::Coastal,
        );
    }

    // ── 2. deregister_node ───────────────────────────────────────────
    #[benchmark]
    fn deregister_node() {
        let (owner, node_id) = setup_node_owner::<T>(0);
        // Reserve deposit (register_node normally does this)
        let deposit = T::NodeRegistrationDeposit::get();
        T::Currency::reserve(&owner, deposit).expect("should reserve");

        #[extrinsic_call]
        deregister_node(RawOrigin::Signed(owner), node_id);
    }

    // ── 3. update_node_location ──────────────────────────────────────
    #[benchmark]
    fn update_node_location() {
        let (owner, node_id) = setup_node_owner::<T>(0);

        #[extrinsic_call]
        update_node_location(
            RawOrigin::Signed(owner),
            node_id,
            175_000_000i32,  // updated lat
            -884_000_000i32, // updated lon
            25i16,
        );
    }

    // ── 4. node_heartbeat ────────────────────────────────────────────
    #[benchmark]
    fn node_heartbeat() {
        let (owner, node_id) = setup_node_owner::<T>(0);

        #[extrinsic_call]
        node_heartbeat(RawOrigin::Signed(owner), node_id);
    }

    // ── 5. submit_mesh_transaction ───────────────────────────────────
    #[benchmark]
    fn submit_mesh_transaction() {
        let (owner, gw_node_id) = setup_gateway_owner::<T>(0);
        let tx_hash = H256::repeat_byte(0xAA);

        #[extrinsic_call]
        submit_mesh_transaction(
            RawOrigin::Signed(owner),
            tx_hash,
            MeshTxType::TransferDalla,
            [1, 2, 3, 4],            // sender_compact
            [5, 6, 7, 8],            // recipient_compact
            1_000_000u64,            // amount
            1u32,                    // nonce
            H256::repeat_byte(0xBB), // signature_hash
            gw_node_id,
            vec![], // relay_path (direct to gateway)
            0u8,    // hop_count
            -70i16, // rssi
            10i16,  // snr
        );
    }

    // ── 6. submit_relay_proof ────────────────────────────────────────
    #[benchmark]
    fn submit_relay_proof() {
        let (owner, node_id) = setup_node_owner::<T>(0);
        let content_hash = H256::repeat_byte(0xCC);
        let source_node: MeshtasticNodeId = [3, 3, 3, 3];

        #[extrinsic_call]
        submit_relay_proof(
            RawOrigin::Signed(owner),
            node_id,
            RelayType::Transaction,
            content_hash,
            source_node,
            RelayDestination::NearestGateway,
            -80i16,
            8i16,
        );
    }

    // ── 7. issue_emergency_alert ─────────────────────────────────────
    // EmergencyOrigin is Root in runtime.
    #[benchmark]
    fn issue_emergency_alert() {
        let message = b"Benchmark alert".to_vec();

        #[extrinsic_call]
        issue_emergency_alert(
            RawOrigin::Root,
            AlertSeverity::Advisory,
            EmergencyType::General,
            174_000_000i32,
            -883_000_000i32,
            5000u32, // radius_meters
            message,
            1000u32, // duration_blocks
            BelizeDistrict::Belize,
        );
    }

    // ── 8. resolve_emergency_alert ───────────────────────────────────
    #[benchmark]
    fn resolve_emergency_alert() {
        // Pre-create an alert
        let bounded_msg: BoundedVec<u8, ConstU32<128>> =
            b"Benchmark alert".to_vec().try_into().unwrap_or_default();
        let current_block = <frame_system::Pallet<T>>::block_number();
        let pallet_acct: T::AccountId = MESH_PALLET_ID.into_account_truncating();
        let alert = EmergencyAlert {
            alert_id: 0,
            severity: AlertSeverity::Advisory,
            alert_type: EmergencyType::General,
            issuer: pallet_acct,
            latitude: 174_000_000,
            longitude: -883_000_000,
            radius_meters: 5000,
            message: bounded_msg,
            created_at: current_block,
            expires_at: current_block + 1000u32.into(),
            resolved: false,
            relay_count: 0,
            confirmations: 0,
            district: BelizeDistrict::Belize,
        };
        EmergencyAlerts::<T>::insert(0u32, alert);
        NextAlertId::<T>::put(1u32);

        #[extrinsic_call]
        resolve_emergency_alert(RawOrigin::Root, 0u32);
    }

    // ── 9. confirm_emergency_alert ───────────────────────────────────
    #[benchmark]
    fn confirm_emergency_alert() {
        let (owner, node_id) = setup_node_owner::<T>(0);

        // Pre-create an alert
        let bounded_msg: BoundedVec<u8, ConstU32<128>> =
            b"Active alert".to_vec().try_into().unwrap_or_default();
        let current_block = <frame_system::Pallet<T>>::block_number();
        let pallet_acct: T::AccountId = MESH_PALLET_ID.into_account_truncating();
        let alert = EmergencyAlert {
            alert_id: 0,
            severity: AlertSeverity::Watch,
            alert_type: EmergencyType::General,
            issuer: pallet_acct,
            latitude: 174_000_000,
            longitude: -883_000_000,
            radius_meters: 5000,
            message: bounded_msg,
            created_at: current_block,
            expires_at: current_block + 1000u32.into(),
            resolved: false,
            relay_count: 0,
            confirmations: 0,
            district: BelizeDistrict::Belize,
        };
        EmergencyAlerts::<T>::insert(0u32, alert);
        NextAlertId::<T>::put(1u32);

        #[extrinsic_call]
        confirm_emergency_alert(RawOrigin::Signed(owner), 0u32, node_id);
    }

    // ── 10. relay_block_header ───────────────────────────────────────
    #[benchmark]
    fn relay_block_header() {
        let (owner, node_id) = setup_validator_relay::<T>(0);

        #[extrinsic_call]
        relay_block_header(
            RawOrigin::Signed(owner),
            node_id,
            42u32,                   // block_number
            H256::repeat_byte(0x11), // block_hash
            H256::repeat_byte(0x22), // parent_hash
            H256::repeat_byte(0x33), // state_root
            H256::repeat_byte(0x44), // extrinsics_root
            [1, 2, 3, 4],            // author_compact
            5u16,                    // extrinsic_count
            1_700_000_000u32,        // timestamp (Unix epoch seconds)
        );
    }

    // ── 11. claim_relay_rewards ──────────────────────────────────────
    #[benchmark]
    fn claim_relay_rewards() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(
            &caller,
            <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 4u32.into(),
        );

        // Give caller a node so the proof cleanup has work to do
        let node_id: MeshtasticNodeId = [4, 4, 4, 4];
        insert_node::<T>(&caller, node_id, MeshNodeRole::Client, false);

        // Pre-populate reward balance
        let reward = T::RelayRewardPerTransaction::get().saturating_mul(10u32.into());
        RelayRewards::<T>::insert(&caller, reward);

        // Fund pallet account so transfer succeeds
        let pallet_acct: T::AccountId = MESH_PALLET_ID.into_account_truncating();
        T::Currency::make_free_balance_be(
            &pallet_acct,
            <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 4u32.into(),
        );

        #[extrinsic_call]
        claim_relay_rewards(RawOrigin::Signed(caller));
    }

    // ── 12. update_mesh_config ───────────────────────────────────────
    // GovernanceOrigin is Root in runtime.
    #[benchmark]
    fn update_mesh_config() {
        #[extrinsic_call]
        update_mesh_config(
            RawOrigin::Root,
            10u8, // max_hops
            ChannelPreset::LongFast,
            true, // relay_mining_active
            true, // emergency_system_active
            true, // validator_relay_active
            1u8,  // min_kyc_for_registration
            2u8,  // min_kyc_for_gateway
        );
    }

    // ── 13. fund_relay_rewards ───────────────────────────────────────
    #[benchmark]
    fn fund_relay_rewards() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(
            &caller,
            <T::Currency as Currency<T::AccountId>>::Balance::max_value() / 2u32.into(),
        );
        let amount = T::RelayRewardPerTransaction::get().saturating_mul(100u32.into());

        #[extrinsic_call]
        fund_relay_rewards(RawOrigin::Signed(caller), amount);
    }

    // ── 14. confirm_relay_proof ──────────────────────────────────────
    #[benchmark]
    fn confirm_relay_proof() {
        // Set up the relayer (different account from confirmer)
        let (relayer_owner, relayer_node_id) = setup_node_owner::<T>(0);
        let _ = relayer_owner; // ownership verified via storage

        // Submit a relay proof for the relayer's node
        let current_block = <frame_system::Pallet<T>>::block_number();
        let proof = RelayProof {
            relayer: relayer_node_id,
            relay_type: RelayType::Transaction,
            content_hash: H256::repeat_byte(0xDD),
            source_node: [5, 5, 5, 5],
            destination: RelayDestination::NearestGateway,
            rssi: -75,
            snr: 9,
            relayed_at: current_block,
            confirmed: false,
        };
        RelayProofs::<T>::mutate(relayer_node_id, |proofs| {
            let _ = proofs.try_push(proof);
        });

        // Set up the confirmer (different owner, owns a node)
        let (confirmer, _confirmer_node) = setup_node_owner::<T>(1);

        #[extrinsic_call]
        confirm_relay_proof(
            RawOrigin::Signed(confirmer),
            relayer_node_id,
            0u32, // proof_index
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
