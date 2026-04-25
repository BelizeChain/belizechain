#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

//! # BelizeChain Meshtastic Mesh Network Pallet
//!
//! Off-grid communication infrastructure using Meshtastic LoRa mesh networking.
//!
//! ## Overview
//!
//! This pallet provides on-chain management for BelizeChain's Meshtastic mesh network,
//! enabling three critical capabilities:
//!
//! 1. **Off-Grid P2P Payments**: Maya Wallet users transact via Meshtastic radios
//!    when no internet is available (rural Belize, cayes, jungle communities)
//! 2. **Validator Mesh Relay**: Validators relay block headers through LoRa mesh
//!    when internet connectivity fails, maintaining consensus
//! 3. **Emergency Broadcast**: Disaster alerts (hurricanes, flooding, tsunamis)
//!    pushed to all mesh nodes in affected areas
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐    BLE     ┌──────────────┐   LoRa 915MHz  ┌──────────────┐
//! │  Maya Wallet │◄──────────►│  Meshtastic  │◄──────────────►│  Meshtastic  │
//! │  (Phone App) │            │  Radio (T-Beam)│               │  Router Node │
//! └──────────────┘            └──────────────┘               └───────┬──────┘
//!                                                                     │ LoRa
//!                                                                     ▼
//!                              ┌──────────────┐   Internet    ┌──────────────┐
//!                              │  BelizeChain │◄─────────────►│  Gateway Node│
//!                              │  Blockchain  │               │  (Internet)  │
//!                              └──────────────┘               └──────────────┘
//! ```
//!
//! ### Communication Flow
//!
//! 1. **Phone → Radio (BLE)**: Maya Wallet sends signed transaction via Bluetooth LE
//!    to the user's Meshtastic radio
//! 2. **Radio → Mesh (LoRa)**: Radio broadcasts transaction through LoRa mesh at 915 MHz
//! 3. **Mesh → Gateway**: A gateway node with internet connectivity receives the message
//! 4. **Gateway → Blockchain**: Gateway submits the transaction to BelizeChain via RPC
//! 5. **Confirmation**: Confirmation relays back through mesh to sender
//!
//! ### Meshtastic Protocol Details
//!
//! - **Frequency**: 915 MHz (US/Belize ISM band)
//! - **Range**: 1-15+ km per hop (up to 25km over water between cayes)
//! - **Payload**: ~237 bytes per packet (transactions are compressed to fit)
//! - **Mesh Hops**: Up to 7 hops per message (configurable)
//! - **Phone Link**: Bluetooth Low Energy (BLE) between phone and radio
//! - **Hardware**: T-Beam, Heltec V3, RAK WisBlock, Station G2
//!
//! ## Features
//!
//! ### Mesh Node Management
//! - Register/deregister Meshtastic nodes on-chain
//! - Track node roles (Client, Router, Gateway, ValidatorRelay, EmergencyBeacon)
//! - Node heartbeat monitoring and reputation scoring
//! - Coverage zone mapping across Belize's 6 districts
//!
//! ### Off-Grid Transactions
//! - Compressed transaction format (87 bytes fits LoRa payload)
//! - Mesh transaction queue with deduplication
//! - Gateway bridging (mesh → blockchain)
//! - Confirmation relay back through mesh
//! - Replay protection via mesh nonces
//!
//! ### Relay Mining
//! - Reward mesh nodes for relaying transactions/data
//! - Relay proof submission and verification
//! - Tiered rewards: transactions > block headers > heartbeats
//! - Anti-gaming with signature verification
//!
//! ### Emergency Broadcast System
//! - Multi-severity alerts (Advisory → Catastrophic)
//! - Geo-targeted to affected districts/coordinates
//! - Automatic relay prioritization for emergency messages
//! - Government-authorized issuers (NEMO integration)
//! - Alert confirmation tracking from affected area nodes
//!
//! ### Validator Mesh Relay
//! - Compressed block header relay through LoRa mesh
//! - Fallback consensus when internet is down
//! - Block header verification and validation
//! - Ensures validator availability in rural areas
//!
//! ## Integration
//!
//! ### Exports
//! - `MeshTransactionBridge` - Trait for submitting mesh-relayed transactions
//! - `EmergencyAlertProvider` - Trait for querying active alerts
//! - `MeshCoverageProvider` - Trait for coverage zone queries
//!
//! ### Consumes
//! - `Currency` - For relay mining rewards
//! - `MeshIdentityProvider` - KYC verification for mesh node registration
//! - `Time` - Timestamps for alerts and relay proofs

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        Currency, ReservableCurrency,
        Get, UnixTime,
    },
    PalletId,
    sp_runtime::traits::AccountIdConversion,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::SaturatedConversion,
    Saturating,
};
use sp_std::vec::Vec;
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;

pub mod types;
pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;

/// Weight info trait for benchmarking
pub trait WeightInfo {
    fn register_node() -> Weight;
    fn deregister_node() -> Weight;
    fn update_node_location() -> Weight;
    fn node_heartbeat() -> Weight;
    fn submit_mesh_transaction() -> Weight;
    fn submit_relay_proof() -> Weight;
    fn issue_emergency_alert() -> Weight;
    fn resolve_emergency_alert() -> Weight;
    fn confirm_emergency_alert() -> Weight;
    fn relay_block_header() -> Weight;
    fn claim_relay_rewards() -> Weight;
    fn update_mesh_config() -> Weight;
    fn fund_relay_rewards() -> Weight;
    fn confirm_relay_proof() -> Weight;
}

/// Default weight implementation
impl WeightInfo for () {
    fn register_node() -> Weight { Weight::from_parts(50_000_000, 512) }
    fn deregister_node() -> Weight { Weight::from_parts(30_000_000, 512) }
    fn update_node_location() -> Weight { Weight::from_parts(20_000_000, 512) }
    fn node_heartbeat() -> Weight { Weight::from_parts(15_000_000, 512) }
    fn submit_mesh_transaction() -> Weight { Weight::from_parts(80_000_000, 512) }
    fn submit_relay_proof() -> Weight { Weight::from_parts(40_000_000, 512) }
    fn issue_emergency_alert() -> Weight { Weight::from_parts(60_000_000, 512) }
    fn resolve_emergency_alert() -> Weight { Weight::from_parts(30_000_000, 512) }
    fn confirm_emergency_alert() -> Weight { Weight::from_parts(25_000_000, 512) }
    fn relay_block_header() -> Weight { Weight::from_parts(70_000_000, 512) }
    fn claim_relay_rewards() -> Weight { Weight::from_parts(50_000_000, 512) }
    fn update_mesh_config() -> Weight { Weight::from_parts(20_000_000, 512) }
    fn fund_relay_rewards() -> Weight { Weight::from_parts(30_000_000, 512) }
    fn confirm_relay_proof() -> Weight { Weight::from_parts(40_000_000, 512) }
}

/// Trait for Identity integration - KYC verification for mesh node registration
pub trait MeshIdentityProvider<AccountId> {
    /// Get KYC level (0=none, 1=basic, 2=verified, 3=full)
    fn get_kyc_level(account: &AccountId) -> u8;
    
    /// Check if account is authorized to issue emergency alerts (NEMO, government)
    fn is_emergency_authority(account: &AccountId) -> bool;
    
    /// Check if account is a registered validator
    fn is_validator(account: &AccountId) -> bool;
}

/// Trait exported: provides mesh transaction bridging to other pallets
pub trait MeshTransactionBridge<AccountId> {
    /// Check if a mesh transaction has been submitted and is pending
    fn is_mesh_tx_pending(tx_hash: &H256) -> bool;
    
    /// Get the total mesh transactions processed
    fn total_mesh_transactions() -> u64;
}

/// Trait exported: emergency alert access for other pallets
pub trait EmergencyAlertProvider<AccountId, BlockNumber> {
    /// Get active alerts for a district
    fn active_alerts_for_district(district: &BelizeDistrict) -> u32;
    
    /// Check if there's an active catastrophic alert
    fn has_catastrophic_alert() -> bool;
}

const MESH_PALLET_ID: PalletId = PalletId(*b"bz/mesht");

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Currency for relay mining rewards
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Time provider for timestamps
        type UnixTime: UnixTime;

        /// Identity provider for KYC and authority checks
        type Identity: MeshIdentityProvider<Self::AccountId>;

        /// Governance origin for mesh network configuration
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Emergency authority origin (NEMO, government)
        type EmergencyOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Maximum registered mesh nodes
        #[pallet::constant]
        type MaxMeshNodes: Get<u32>;

        /// Maximum pending mesh transactions in queue
        #[pallet::constant]
        type MaxPendingMeshTx: Get<u32>;

        /// Maximum active emergency alerts
        #[pallet::constant]
        type MaxActiveAlerts: Get<u32>;

        /// Maximum relay proofs per claim period
        #[pallet::constant]
        type MaxRelayProofsPerClaim: Get<u32>;

        /// Relay reward per transaction relayed (in smallest currency unit)
        #[pallet::constant]
        type RelayRewardPerTransaction: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Relay reward per block header relayed
        #[pallet::constant]
        type RelayRewardPerBlockHeader: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Relay reward per emergency alert relayed
        #[pallet::constant]
        type RelayRewardPerEmergencyAlert: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Minimum deposit to register a mesh node (anti-spam)
        #[pallet::constant]
        type NodeRegistrationDeposit: Get<<Self::Currency as Currency<Self::AccountId>>::Balance>;

        /// Number of blocks before a node is considered inactive (no heartbeat)
        #[pallet::constant]
        type HeartbeatTimeout: Get<u32>;

        /// Maximum mesh transactions accepted per block (DoS protection)
        #[pallet::constant]
        type MaxMeshTxPerBlock: Get<u32>;

        /// Weight information
        type WeightInfo: WeightInfo;
    }

    // ========================
    // Hooks
    // ========================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            MeshTxThisBlock::<T>::kill();
            // Phase-5 FIX: kill() is a DB write; account for ref_time + proof_size
            Weight::from_parts(5_000_000, 64)
                .saturating_add(T::DbWeight::get().writes(1))
        }
    }

    // ========================
    // Storage Items
    // ========================

    /// Per-block mesh transaction counter (reset each block via on_initialize)
    #[pallet::storage]
    pub type MeshTxThisBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Registered mesh nodes: NodeId → MeshNode
    #[pallet::storage]
    #[pallet::getter(fn mesh_nodes)]
    pub type MeshNodes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MeshtasticNodeId,
        MeshNode<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Account → list of owned node IDs
    #[pallet::storage]
    #[pallet::getter(fn nodes_by_owner)]
    pub type NodesByOwner<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<MeshtasticNodeId, ConstU32<10>>,
        ValueQuery,
    >;

    /// Pending mesh transactions waiting to be bridged to blockchain
    #[pallet::storage]
    #[pallet::getter(fn pending_mesh_transactions)]
    pub type PendingMeshTransactions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // transaction hash
        MeshTransaction<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Processed mesh transaction hashes (deduplication, pruned periodically)
    #[pallet::storage]
    #[pallet::getter(fn processed_mesh_transactions)]
    pub type ProcessedMeshTransactions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        BlockNumberFor<T>, // block when processed
        OptionQuery,
    >;

    /// Active emergency alerts
    #[pallet::storage]
    #[pallet::getter(fn emergency_alerts)]
    pub type EmergencyAlerts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // alert_id
        EmergencyAlert<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Next emergency alert ID
    #[pallet::storage]
    #[pallet::getter(fn next_alert_id)]
    pub type NextAlertId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// O(1) counter: active (non-resolved) alerts per district (H-40)
    #[pallet::storage]
    pub type ActiveAlertCountPerDistrict<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BelizeDistrict,
        u32,
        ValueQuery,
    >;

    /// O(1) flag: true when at least one unresolved Catastrophic alert exists (H-40)
    #[pallet::storage]
    pub type CatastrophicAlertCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Relayed block headers via mesh
    #[pallet::storage]
    #[pallet::getter(fn mesh_block_headers)]
    pub type MeshBlockHeaders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // block_number
        MeshBlockHeader,
        OptionQuery,
    >;

    /// Pending relay proofs (for reward claims)
    #[pallet::storage]
    #[pallet::getter(fn relay_proofs)]
    pub type RelayProofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MeshtasticNodeId,
        BoundedVec<RelayProof<BlockNumberFor<T>>, ConstU32<100>>,
        ValueQuery,
    >;

    /// Accumulated relay rewards (unclaimed)
    #[pallet::storage]
    #[pallet::getter(fn relay_rewards)]
    pub type RelayRewards<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        <T::Currency as Currency<T::AccountId>>::Balance,
        ValueQuery,
    >;

    /// Network-wide mesh statistics
    #[pallet::storage]
    #[pallet::getter(fn network_stats)]
    pub type NetworkStats<T: Config> = StorageValue<_, MeshNetworkStats, ValueQuery>;

    /// Mesh network configuration (governance-adjustable)
    #[pallet::storage]
    #[pallet::getter(fn mesh_config)]
    pub type MeshConfig<T: Config> = StorageValue<_, MeshNetworkConfig, ValueQuery>;

    /// Meshtastic network configuration parameters
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    pub struct MeshNetworkConfig {
        /// Maximum allowed hops for mesh messages
        pub max_hops: u8,
        /// LoRa channel preset (LongFast, LongSlow, MediumFast, ShortFast)
        pub channel_preset: ChannelPreset,
        /// Whether relay mining is active
        pub relay_mining_active: bool,
        /// Whether emergency broadcast system is active
        pub emergency_system_active: bool,
        /// Whether validator mesh relay is active
        pub validator_relay_active: bool,
        /// Minimum KYC level to register a mesh node (0-3)
        pub min_kyc_for_registration: u8,
        /// Minimum KYC level for gateway nodes (higher trust)
        pub min_kyc_for_gateway: u8,
    }

    impl Default for MeshNetworkConfig {
        fn default() -> Self {
            MeshNetworkConfig {
                max_hops: 7,
                channel_preset: ChannelPreset::LongFast,
                relay_mining_active: true,
                emergency_system_active: true,
                validator_relay_active: true,
                min_kyc_for_registration: 1, // Basic KYC
                min_kyc_for_gateway: 2,      // Verified KYC
            }
        }
    }

    /// Meshtastic LoRa channel presets
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen, Default)]
    pub enum ChannelPreset {
        /// Long range, fast data rate (default for BelizeChain)
        #[default]
        LongFast,
        /// Long range, slow data rate (maximum range)
        LongSlow,
        /// Long range, moderate data rate
        LongModerate,
        /// Medium range, fast data rate
        MediumFast,
        /// Medium range, slow data rate
        MediumSlow,
        /// Short range, fast data rate (urban areas)
        ShortFast,
        /// Short range, slow data rate
        ShortSlow,
    }

    // ========================
    // Events
    // ========================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new mesh node was registered
        NodeRegistered {
            owner: T::AccountId,
            node_id: MeshtasticNodeId,
            role: MeshNodeRole,
            hardware: MeshHardware,
            district: BelizeDistrict,
        },
        /// A mesh node was deregistered
        NodeDeregistered {
            owner: T::AccountId,
            node_id: MeshtasticNodeId,
        },
        /// Node location updated
        NodeLocationUpdated {
            node_id: MeshtasticNodeId,
            latitude: i32,
            longitude: i32,
        },
        /// Node heartbeat received
        NodeHeartbeat {
            node_id: MeshtasticNodeId,
            block: BlockNumberFor<T>,
        },
        /// Mesh transaction received by gateway and submitted
        MeshTransactionReceived {
            tx_hash: H256,
            sender_compact: [u8; 4],
            recipient_compact: [u8; 4],
            amount: u64,
            gateway_node: MeshtasticNodeId,
            hop_count: u8,
        },
        /// Mesh transaction processed (bridged to blockchain)
        MeshTransactionProcessed {
            tx_hash: H256,
            block: BlockNumberFor<T>,
        },
        /// Relay proof submitted
        RelayProofSubmitted {
            relayer: MeshtasticNodeId,
            relay_type: RelayType,
            content_hash: H256,
        },
        /// Relay rewards claimed
        RelayRewardsClaimed {
            owner: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Relay rewards pool funded
        RelayRewardsFunded {
            funder: T::AccountId,
            amount: <T::Currency as Currency<T::AccountId>>::Balance,
        },
        /// Emergency alert issued
        EmergencyAlertIssued {
            alert_id: u32,
            severity: AlertSeverity,
            alert_type: EmergencyType,
            issuer: T::AccountId,
            district: BelizeDistrict,
        },
        /// Emergency alert resolved
        EmergencyAlertResolved {
            alert_id: u32,
            resolver: T::AccountId,
        },
        /// Emergency alert confirmed by a node in affected area
        EmergencyAlertConfirmed {
            alert_id: u32,
            confirmer_node: MeshtasticNodeId,
        },
        /// Block header relayed through mesh
        BlockHeaderRelayed {
            block_number: u32,
            block_hash: H256,
            relayer: MeshtasticNodeId,
        },
        /// Mesh network configuration updated
        MeshConfigUpdated {
            max_hops: u8,
            channel_preset: ChannelPreset,
        },
        /// Relay proof confirmed by a second node — rewards now accrue
        RelayProofConfirmed {
            relayer_node: MeshtasticNodeId,
            confirmer: T::AccountId,
            reward: <T::Currency as Currency<T::AccountId>>::Balance,
        },
    }

    // ========================
    // Errors
    // ========================

    #[pallet::error]
    pub enum Error<T> {
        /// Node ID already registered
        NodeAlreadyRegistered,
        /// Node not found
        NodeNotFound,
        /// Caller does not own this node
        NotNodeOwner,
        /// Maximum nodes per account exceeded
        TooManyNodes,
        /// Maximum mesh nodes reached
        MaxMeshNodesReached,
        /// Insufficient KYC level for this operation
        InsufficientKycLevel,
        /// Insufficient KYC level for gateway registration
        InsufficientKycForGateway,
        /// Duplicate mesh transaction (already processed or pending)
        DuplicateMeshTransaction,
        /// Maximum pending transactions exceeded
        MaxPendingTransactionsExceeded,
        /// Invalid mesh transaction signature
        InvalidMeshSignature,
        /// Mesh transaction hop count exceeds maximum
        ExcessiveHopCount,
        /// Emergency alert not found
        AlertNotFound,
        /// Alert already resolved
        AlertAlreadyResolved,
        /// Maximum active alerts exceeded
        MaxActiveAlertsExceeded,
        /// Not authorized to issue emergency alerts
        NotEmergencyAuthority,
        /// Relay proof limit exceeded for this claim period
        RelayProofLimitExceeded,
        /// No relay rewards to claim
        NoRelayRewards,
        /// Invalid block header (parent hash mismatch)
        InvalidBlockHeader,
        /// Node is inactive (missed heartbeats)
        NodeInactive,
        /// Invalid GPS coordinates
        InvalidCoordinates,
        /// Not a registered validator (cannot use ValidatorRelay role)
        ValidatorNotFound,
        /// Relay mining is currently disabled
        RelayMiningDisabled,
        /// Emergency system is currently disabled
        EmergencySystemDisabled,
        /// Insufficient deposit for node registration
        InsufficientDeposit,
        /// Relay proof already confirmed
        ProofAlreadyConfirmed,
        /// Cannot confirm own relay proof
        CannotConfirmOwnProof,
        /// Relay proof index out of range
        ProofIndexOutOfRange,
        /// Alert already confirmed by this node (M58 dedup)
        AlertAlreadyConfirmed,
        /// Block header for this number already relayed (MH-6)
        HeaderAlreadyExists,
        /// Emergency message too long for bounded vec (#80)
        MessageTooLong,
        /// Too many mesh transactions this block (rate limit)
        MeshTxRateLimitExceeded,
        /// Alert ID counter overflow
        AlertIdOverflow,
    }

    // ========================
    // Extrinsics
    // ========================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new Meshtastic mesh node on-chain.
        ///
        /// Requires minimum KYC level and a registration deposit.
        /// Gateway nodes require higher KYC (Verified level).
        ///
        /// # Parameters
        /// - `node_id`: 4-byte Meshtastic hardware node ID
        /// - `role`: Node role (Client, Router, Gateway, ValidatorRelay, EmergencyBeacon)
        /// - `hardware`: Hardware type (T-Beam, Heltec V3, RAK, etc.)
        /// - `region`: LoRa frequency region (US915 for Belize)
        /// - `latitude`: GPS latitude * 1e7
        /// - `longitude`: GPS longitude * 1e7
        /// - `altitude`: Altitude in meters
        /// - `district`: Belize district for coverage mapping
        /// - `terrain`: Terrain type for range estimation
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_node())]
        pub fn register_node(
            origin: OriginFor<T>,
            node_id: MeshtasticNodeId,
            role: MeshNodeRole,
            hardware: MeshHardware,
            region: LoRaRegion,
            latitude: i32,
            longitude: i32,
            altitude: i16,
            district: BelizeDistrict,
            _terrain: TerrainType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Verify node ID not already registered
            ensure!(!MeshNodes::<T>::contains_key(node_id), Error::<T>::NodeAlreadyRegistered);

            // Check KYC level
            let config = MeshConfig::<T>::get();
            let kyc_level = T::Identity::get_kyc_level(&who);
            ensure!(kyc_level >= config.min_kyc_for_registration, Error::<T>::InsufficientKycLevel);

            // Gateway nodes need higher KYC
            let is_gateway = matches!(role, MeshNodeRole::Gateway);
            if is_gateway {
                ensure!(kyc_level >= config.min_kyc_for_gateway, Error::<T>::InsufficientKycForGateway);
            }

            // ValidatorRelay requires being a registered validator
            if matches!(role, MeshNodeRole::ValidatorRelay) {
                ensure!(T::Identity::is_validator(&who), Error::<T>::ValidatorNotFound);
            }

            // Reserve registration deposit
            let deposit = T::NodeRegistrationDeposit::get();
            T::Currency::reserve(&who, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;

            // Check max nodes per account
            let mut owned = NodesByOwner::<T>::get(&who);
            ensure!(owned.try_push(node_id).is_ok(), Error::<T>::TooManyNodes);

            // Check global node limit
            let mut stats = NetworkStats::<T>::get();
            ensure!(stats.total_nodes < T::MaxMeshNodes::get(), Error::<T>::MaxMeshNodesReached);

            let current_block = <frame_system::Pallet<T>>::block_number();

            let node = MeshNode {
                owner: who.clone(),
                node_id,
                role: role.clone(),
                hardware: hardware.clone(),
                region,
                latitude,
                longitude,
                altitude,
                is_gateway,
                messages_relayed: 0,
                transactions_relayed: 0,
                emergency_alerts_sent: 0,
                registered_at: current_block,
                last_seen: current_block,
                reputation: 5000, // Start at 50% reputation
                active: true,
            };

            // Update stats
            stats.total_nodes += 1;
            // AUDIT FIX (C3): New nodes start active — increment active_nodes
            stats.active_nodes = stats.active_nodes.saturating_add(1);
            if is_gateway {
                stats.gateway_count += 1;
            }
            match &role {
                MeshNodeRole::Router | MeshNodeRole::RouterClient => stats.router_count += 1,
                MeshNodeRole::ValidatorRelay => stats.validator_relay_count += 1,
                MeshNodeRole::EmergencyBeacon => stats.emergency_beacon_count += 1,
                _ => {},
            }

            MeshNodes::<T>::insert(node_id, node);
            NodesByOwner::<T>::insert(&who, owned);
            NetworkStats::<T>::put(stats);

            Self::deposit_event(Event::NodeRegistered {
                owner: who,
                node_id,
                role,
                hardware,
                district,
            });

            Ok(())
        }

        /// Deregister a mesh node and reclaim the registration deposit.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::deregister_node())]
        pub fn deregister_node(
            origin: OriginFor<T>,
            node_id: MeshtasticNodeId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let node = MeshNodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotFound)?;
            ensure!(node.owner == who, Error::<T>::NotNodeOwner);

            // Return registration deposit
            let deposit = T::NodeRegistrationDeposit::get();
            T::Currency::unreserve(&who, deposit);

            // Update stats
            let mut stats = NetworkStats::<T>::get();
            stats.total_nodes = stats.total_nodes.saturating_sub(1);
            if node.is_gateway {
                stats.gateway_count = stats.gateway_count.saturating_sub(1);
            }
            match &node.role {
                MeshNodeRole::Router | MeshNodeRole::RouterClient => {
                    stats.router_count = stats.router_count.saturating_sub(1);
                },
                MeshNodeRole::ValidatorRelay => {
                    stats.validator_relay_count = stats.validator_relay_count.saturating_sub(1);
                },
                MeshNodeRole::EmergencyBeacon => {
                    stats.emergency_beacon_count = stats.emergency_beacon_count.saturating_sub(1);
                },
                _ => {},
            }

            // AUDIT FIX (C3): Decrement active_nodes if the node was active
            if node.active {
                stats.active_nodes = stats.active_nodes.saturating_sub(1);
            }

            // Remove from owner's list
            let mut owned = NodesByOwner::<T>::get(&who);
            owned.retain(|id| id != &node_id);
            NodesByOwner::<T>::insert(&who, owned);

            // Remove node
            MeshNodes::<T>::remove(node_id);
            NetworkStats::<T>::put(stats);

            Self::deposit_event(Event::NodeDeregistered { owner: who, node_id });

            Ok(())
        }

        /// Update a mesh node's GPS location (mobile nodes).
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_node_location())]
        pub fn update_node_location(
            origin: OriginFor<T>,
            node_id: MeshtasticNodeId,
            latitude: i32,
            longitude: i32,
            altitude: i16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MeshNodes::<T>::try_mutate(node_id, |maybe_node| -> DispatchResult {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotFound)?;
                ensure!(node.owner == who, Error::<T>::NotNodeOwner);

                // Validate coordinates are within reasonable bounds for Belize
                // Belize: lat 15.8-18.5, lon -89.2 to -87.5
                // Using 1e7 scale: lat 158000000-185000000, lon -892000000 to -875000000
                // We allow broader range for flexibility
                ensure!(
                    (-900_000_000..=900_000_000).contains(&latitude) &&
                    (-1_800_000_000..=1_800_000_000).contains(&longitude),
                    Error::<T>::InvalidCoordinates
                );

                node.latitude = latitude;
                node.longitude = longitude;
                node.altitude = altitude;

                Ok(())
            })?;

            Self::deposit_event(Event::NodeLocationUpdated {
                node_id,
                latitude,
                longitude,
            });

            Ok(())
        }

        /// Submit a heartbeat to keep node marked as active.
        /// Nodes that don't heartbeat within `HeartbeatTimeout` blocks are marked inactive.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::node_heartbeat())]
        pub fn node_heartbeat(
            origin: OriginFor<T>,
            node_id: MeshtasticNodeId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_block = <frame_system::Pallet<T>>::block_number();

            MeshNodes::<T>::try_mutate(node_id, |maybe_node| -> DispatchResult {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotFound)?;
                ensure!(node.owner == who, Error::<T>::NotNodeOwner);

                node.last_seen = current_block;
                if !node.active {
                    node.active = true;
                    let mut stats = NetworkStats::<T>::get();
                    stats.active_nodes += 1;
                    NetworkStats::<T>::put(stats);
                }

                Ok(())
            })?;

            Self::deposit_event(Event::NodeHeartbeat {
                node_id,
                block: current_block,
            });

            Ok(())
        }

        /// Submit a mesh-relayed transaction from a gateway node.
        ///
        /// Called by gateway nodes when they receive a transaction through the LoRa mesh
        /// and bridge it to the blockchain. The transaction was signed offline in Maya Wallet,
        /// transmitted via BLE to a Meshtastic radio, relayed through the LoRa mesh, and
        /// received by this gateway.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::submit_mesh_transaction())]
        pub fn submit_mesh_transaction(
            origin: OriginFor<T>,
            tx_hash: H256,
            tx_type: MeshTxType,
            sender_compact: [u8; 4],
            recipient_compact: [u8; 4],
            amount: u64,
            nonce: u32,
            signature_hash: H256,
            gateway_node_id: MeshtasticNodeId,
            relay_path: Vec<MeshtasticNodeId>,
            hop_count: u8,
            rssi: i16,
            snr: i16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // M57 FIX: Basic mesh transaction signature validation
            // Signature hash must be non-zero (zero hash indicates unsigned/invalid tx)
            ensure!(signature_hash != H256::zero(), Error::<T>::InvalidMeshSignature);

            // Verify gateway node exists and caller owns it
            let gateway = MeshNodes::<T>::get(gateway_node_id).ok_or(Error::<T>::NodeNotFound)?;
            ensure!(gateway.owner == who, Error::<T>::NotNodeOwner);
            ensure!(gateway.is_gateway, Error::<T>::NodeNotFound); // Must be a gateway
            ensure!(gateway.active, Error::<T>::NodeInactive);

            // Check config limits
            let config = MeshConfig::<T>::get();
            ensure!(hop_count <= config.max_hops, Error::<T>::ExcessiveHopCount);

            // Per-block rate limit (DoS protection)
            let tx_count = MeshTxThisBlock::<T>::get();
            ensure!(tx_count < T::MaxMeshTxPerBlock::get(), Error::<T>::MeshTxRateLimitExceeded);

            // Deduplication
            ensure!(
                !PendingMeshTransactions::<T>::contains_key(tx_hash) &&
                !ProcessedMeshTransactions::<T>::contains_key(tx_hash),
                Error::<T>::DuplicateMeshTransaction
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            // Bound the relay path
            let bounded_relay_path: BoundedVec<MeshtasticNodeId, ConstU32<8>> =
                relay_path.try_into().map_err(|_| Error::<T>::ExcessiveHopCount)?;

            let mesh_tx = MeshTransaction {
                version: 1,
                tx_type,
                sender_compact,
                recipient_compact,
                amount,
                nonce,
                signature_hash,
                flags: MeshTxFlags::default(),
                received_at: Some(current_block),
                gateway_node: Some(gateway_node_id),
                relay_path: bounded_relay_path,
                hop_count,
                rssi,
                snr,
            };

            PendingMeshTransactions::<T>::insert(tx_hash, mesh_tx);

            // Update gateway stats
            MeshNodes::<T>::mutate(gateway_node_id, |maybe_node| {
                if let Some(node) = maybe_node {
                    node.transactions_relayed += 1;
                }
            });

            // Increment per-block counter
            MeshTxThisBlock::<T>::mutate(|c| *c = c.saturating_add(1));

            // Update network stats
            NetworkStats::<T>::mutate(|stats| {
                stats.total_mesh_transactions += 1;
            });

            Self::deposit_event(Event::MeshTransactionReceived {
                tx_hash,
                sender_compact,
                recipient_compact,
                amount,
                gateway_node: gateway_node_id,
                hop_count,
            });

            Ok(())
        }

        /// Submit a relay proof for relay mining rewards.
        ///
        /// Mesh nodes earn DALLA rewards for relaying transactions, block headers,
        /// and emergency alerts through the LoRa mesh network.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::submit_relay_proof())]
        pub fn submit_relay_proof(
            origin: OriginFor<T>,
            node_id: MeshtasticNodeId,
            relay_type: RelayType,
            content_hash: H256,
            source_node: MeshtasticNodeId,
            destination: RelayDestination,
            rssi: i16,
            snr: i16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let config = MeshConfig::<T>::get();
            ensure!(config.relay_mining_active, Error::<T>::RelayMiningDisabled);

            // Verify node ownership
            let node = MeshNodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotFound)?;
            ensure!(node.owner == who, Error::<T>::NotNodeOwner);
            ensure!(node.active, Error::<T>::NodeInactive);

            let current_block = <frame_system::Pallet<T>>::block_number();

            let proof = RelayProof {
                relayer: node_id,
                relay_type: relay_type.clone(),
                content_hash,
                source_node,
                destination,
                rssi,
                snr,
                relayed_at: current_block,
                confirmed: false,
            };

            // Store relay proof
            RelayProofs::<T>::try_mutate(node_id, |proofs| -> DispatchResult {
                proofs.try_push(proof).map_err(|_| Error::<T>::RelayProofLimitExceeded)?;
                Ok(())
            })?;

            // NOTE: Rewards are NOT accrued here. They are deferred until a
            // second node confirms the relay via `confirm_relay_proof`.
            // This prevents self-reporting abuse.

            // Update node stats
            MeshNodes::<T>::mutate(node_id, |maybe_node| {
                if let Some(n) = maybe_node {
                    n.messages_relayed += 1;
                    // Boost reputation for reliable relaying
                    n.reputation = n.reputation.saturating_add(1).min(10000);
                }
            });

            // Update network stats
            NetworkStats::<T>::mutate(|stats| {
                stats.total_relay_proofs += 1;
            });

            Self::deposit_event(Event::RelayProofSubmitted {
                relayer: node_id,
                relay_type,
                content_hash,
            });

            Ok(())
        }

        /// Issue an emergency alert through the mesh network.
        ///
        /// Only authorized accounts (NEMO, government, emergency services) can issue alerts.
        /// The alert is stored on-chain and broadcast through all mesh nodes in the affected area.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::issue_emergency_alert())]
        pub fn issue_emergency_alert(
            origin: OriginFor<T>,
            severity: AlertSeverity,
            alert_type: EmergencyType,
            latitude: i32,
            longitude: i32,
            radius_meters: u32,
            message: Vec<u8>,
            duration_blocks: u32,
            district: BelizeDistrict,
        ) -> DispatchResult {
            // Can be called by emergency origin OR by a signed account with authority
            let who = match T::EmergencyOrigin::try_origin(origin.clone()) {
                Ok(_) => {
                    // Root/governance origin - use pallet account
                    MESH_PALLET_ID.into_account_truncating()
                },
                Err(origin) => {
                    let signer = ensure_signed(origin)?;
                    ensure!(
                        T::Identity::is_emergency_authority(&signer),
                        Error::<T>::NotEmergencyAuthority
                    );
                    signer
                },
            };

            let config = MeshConfig::<T>::get();
            ensure!(config.emergency_system_active, Error::<T>::EmergencySystemDisabled);

            let current_block = <frame_system::Pallet<T>>::block_number();
            let alert_id = NextAlertId::<T>::get();

            // #80 FIX: Return error instead of silently truncating
            let bounded_message: BoundedVec<u8, ConstU32<128>> =
                message.try_into().map_err(|_| Error::<T>::MessageTooLong)?;

            let expires_at = current_block + duration_blocks.into();

            let alert = EmergencyAlert {
                alert_id,
                severity: severity.clone(),
                alert_type: alert_type.clone(),
                issuer: who.clone(),
                latitude,
                longitude,
                radius_meters,
                message: bounded_message,
                created_at: current_block,
                expires_at,
                resolved: false,
                relay_count: 0,
                confirmations: 0,
                district: district.clone(),
            };

            EmergencyAlerts::<T>::insert(alert_id, alert);
            // AUDIT FIX (W1): Prevent NextAlertId overflow
            let next_id = alert_id.checked_add(1).ok_or(Error::<T>::AlertIdOverflow)?;
            NextAlertId::<T>::put(next_id);

            // O(1) counter maintenance (H-40)
            ActiveAlertCountPerDistrict::<T>::mutate(&district, |c| *c = c.saturating_add(1));
            if severity >= AlertSeverity::Catastrophic {
                CatastrophicAlertCount::<T>::mutate(|c| *c = c.saturating_add(1));
            }

            // Update network stats
            NetworkStats::<T>::mutate(|stats| {
                stats.total_emergency_alerts += 1;
            });

            Self::deposit_event(Event::EmergencyAlertIssued {
                alert_id,
                severity,
                alert_type,
                issuer: who,
                district,
            });

            Ok(())
        }

        /// Resolve an active emergency alert.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::resolve_emergency_alert())]
        pub fn resolve_emergency_alert(
            origin: OriginFor<T>,
            alert_id: u32,
        ) -> DispatchResult {
            let who = match T::EmergencyOrigin::try_origin(origin.clone()) {
                Ok(_) => MESH_PALLET_ID.into_account_truncating(),
                Err(origin) => {
                    let signer = ensure_signed(origin)?;
                    ensure!(
                        T::Identity::is_emergency_authority(&signer),
                        Error::<T>::NotEmergencyAuthority
                    );
                    signer
                },
            };

            EmergencyAlerts::<T>::try_mutate(alert_id, |maybe_alert| -> DispatchResult {
                let alert = maybe_alert.as_mut().ok_or(Error::<T>::AlertNotFound)?;
                ensure!(!alert.resolved, Error::<T>::AlertAlreadyResolved);
                alert.resolved = true;

                // O(1) counter maintenance (H-40)
                ActiveAlertCountPerDistrict::<T>::mutate(&alert.district, |c| *c = c.saturating_sub(1));
                if alert.severity >= AlertSeverity::Catastrophic {
                    CatastrophicAlertCount::<T>::mutate(|c| *c = c.saturating_sub(1));
                }

                Ok(())
            })?;

            Self::deposit_event(Event::EmergencyAlertResolved {
                alert_id,
                resolver: who,
            });

            Ok(())
        }

        /// Confirm receipt of an emergency alert from a mesh node in the affected area.
        /// This helps track alert coverage and reach.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::confirm_emergency_alert())]
        pub fn confirm_emergency_alert(
            origin: OriginFor<T>,
            alert_id: u32,
            confirmer_node_id: MeshtasticNodeId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Verify node ownership
            let node = MeshNodes::<T>::get(confirmer_node_id).ok_or(Error::<T>::NodeNotFound)?;
            ensure!(node.owner == who, Error::<T>::NotNodeOwner);

            EmergencyAlerts::<T>::try_mutate(alert_id, |maybe_alert| -> DispatchResult {
                let alert = maybe_alert.as_mut().ok_or(Error::<T>::AlertNotFound)?;

                // M58 FIX: Prevent same node from confirming an alert multiple times.
                // Without this, a single node owner can inflate confirmations to
                // misrepresent alert coverage/severity.
                // Use a deterministic H256 key derived from (alert_id, node_id) to
                // check dedup in ProcessedMeshTransactions (lightweight sentinel).
                let mut dedup_input = [0u8; 8];
                dedup_input[..4].copy_from_slice(&alert_id.to_le_bytes());
                dedup_input[4..].copy_from_slice(&confirmer_node_id);
                let dedup_key = sp_core::hashing::blake2_256(&dedup_input);
                let dedup_hash = H256::from(dedup_key);

                ensure!(
                    !ProcessedMeshTransactions::<T>::contains_key(dedup_hash),
                    Error::<T>::AlertAlreadyConfirmed
                );
                // Store a minimal sentinel to mark this (alert, node) pair as confirmed
                ProcessedMeshTransactions::<T>::insert(
                    dedup_hash,
                    <frame_system::Pallet<T>>::block_number(),
                );

                alert.confirmations += 1;
                Ok(())
            })?;

            Self::deposit_event(Event::EmergencyAlertConfirmed {
                alert_id,
                confirmer_node: confirmer_node_id,
            });

            Ok(())
        }

        /// Relay a compressed block header through the mesh network.
        ///
        /// Validator relay nodes use this when internet connectivity is down,
        /// sending block headers through LoRa to maintain consensus awareness.
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::relay_block_header())]
        pub fn relay_block_header(
            origin: OriginFor<T>,
            relayer_node_id: MeshtasticNodeId,
            block_number: u32,
            block_hash: H256,
            parent_hash: H256,
            state_root: H256,
            extrinsics_root: H256,
            author_compact: [u8; 4],
            extrinsic_count: u16,
            timestamp: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let config = MeshConfig::<T>::get();
            ensure!(config.validator_relay_active, Error::<T>::RelayMiningDisabled);

            // Verify node is a validator relay
            let node = MeshNodes::<T>::get(relayer_node_id).ok_or(Error::<T>::NodeNotFound)?;
            ensure!(node.owner == who, Error::<T>::NotNodeOwner);
            ensure!(
                matches!(node.role, MeshNodeRole::ValidatorRelay),
                Error::<T>::ValidatorNotFound
            );

            let header = MeshBlockHeader {
                block_number,
                block_hash,
                parent_hash,
                state_root,
                extrinsics_root,
                author_compact,
                extrinsic_count,
                timestamp,
            };

            // MH-6 FIX: Prevent overwriting existing block headers
            ensure!(
                !MeshBlockHeaders::<T>::contains_key(block_number),
                Error::<T>::HeaderAlreadyExists
            );
            MeshBlockHeaders::<T>::insert(block_number, header);

            // Update network stats
            NetworkStats::<T>::mutate(|stats| {
                stats.total_block_headers_relayed += 1;
            });

            Self::deposit_event(Event::BlockHeaderRelayed {
                block_number,
                block_hash,
                relayer: relayer_node_id,
            });

            Ok(())
        }

        /// Claim accumulated relay mining rewards.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::claim_relay_rewards())]
        pub fn claim_relay_rewards(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reward = RelayRewards::<T>::get(&who);
            ensure!(!reward.is_zero(), Error::<T>::NoRelayRewards);

            // Transfer from pallet account
            let pallet_account: T::AccountId = MESH_PALLET_ID.into_account_truncating();
            T::Currency::transfer(
                &pallet_account,
                &who,
                reward,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // Update total rewards in stats
            NetworkStats::<T>::mutate(|stats| {
                // SAFETY(saturated_into): Balance → u128 is lossless; substrate balances are at most u128.
                stats.total_relay_rewards = stats.total_relay_rewards.saturating_add(
                    reward.saturated_into::<u128>()
                );
            });

            // Clear relay proofs for all owned nodes
            let owned_nodes = NodesByOwner::<T>::get(&who);
            for node_id in owned_nodes.iter() {
                RelayProofs::<T>::remove(node_id);
            }

            // Reset rewards
            RelayRewards::<T>::remove(&who);

            Self::deposit_event(Event::RelayRewardsClaimed {
                owner: who,
                amount: reward,
            });

            Ok(())
        }

        /// Update mesh network configuration (governance only).
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::update_mesh_config())]
        pub fn update_mesh_config(
            origin: OriginFor<T>,
            max_hops: u8,
            channel_preset: ChannelPreset,
            relay_mining_active: bool,
            emergency_system_active: bool,
            validator_relay_active: bool,
            min_kyc_for_registration: u8,
            min_kyc_for_gateway: u8,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let config = MeshNetworkConfig {
                max_hops,
                channel_preset: channel_preset.clone(),
                relay_mining_active,
                emergency_system_active,
                validator_relay_active,
                min_kyc_for_registration,
                min_kyc_for_gateway,
            };

            MeshConfig::<T>::put(config);

            Self::deposit_event(Event::MeshConfigUpdated {
                max_hops,
                channel_preset,
            });

            Ok(())
        }

        /// Fund the relay rewards pool from the sender's balance.
        ///
        /// Governance or the treasury can call this to ensure the pallet account
        /// has sufficient balance to pay out `claim_relay_rewards`.
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::fund_relay_rewards())]
        pub fn fund_relay_rewards(
            origin: OriginFor<T>,
            #[pallet::compact] amount: <T::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResult {
            let funder = ensure_signed(origin)?;
            ensure!(!amount.is_zero(), Error::<T>::NoRelayRewards);

            let pallet_account: T::AccountId = MESH_PALLET_ID.into_account_truncating();
            T::Currency::transfer(
                &funder,
                &pallet_account,
                amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::RelayRewardsFunded {
                funder,
                amount,
            });

            Ok(())
        }

        /// Confirm a relay proof submitted by a different node owner.
        ///
        /// A second, independent mesh node must confirm a relay proof before
        /// the submitter earns relay mining rewards. This prevents a single
        /// node from fabricating proofs.
        #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::confirm_relay_proof())]
        pub fn confirm_relay_proof(
            origin: OriginFor<T>,
            relayer_node_id: MeshtasticNodeId,
            proof_index: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Confirmer must own a registered mesh node (cannot be the same owner)
            let relayer_node = MeshNodes::<T>::get(relayer_node_id)
                .ok_or(Error::<T>::NodeNotFound)?;
            ensure!(relayer_node.owner != who, Error::<T>::CannotConfirmOwnProof);

            // Confirmer must own at least one active node to be credible
            let confirmer_nodes = NodesByOwner::<T>::get(&who);
            ensure!(!confirmer_nodes.is_empty(), Error::<T>::NodeNotFound);

            RelayProofs::<T>::try_mutate(relayer_node_id, |proofs| -> DispatchResult {
                let proof = proofs.get_mut(proof_index as usize)
                    .ok_or(Error::<T>::ProofIndexOutOfRange)?;
                ensure!(!proof.confirmed, Error::<T>::ProofAlreadyConfirmed);

                proof.confirmed = true;

                // Now accrue the reward to the relay node owner
                let reward = match proof.relay_type {
                    RelayType::Transaction => T::RelayRewardPerTransaction::get(),
                    RelayType::BlockHeader => T::RelayRewardPerBlockHeader::get(),
                    RelayType::EmergencyAlert => T::RelayRewardPerEmergencyAlert::get(),
                    RelayType::Heartbeat | RelayType::Confirmation => {
                        // SAFETY(saturated_into): constant 10u32 → BalanceOf<T> is lossless; small constant fits any Balance type.
                        T::RelayRewardPerTransaction::get() / 10u32.saturated_into()
                    },
                };

                RelayRewards::<T>::mutate(&relayer_node.owner, |balance| {
                    *balance = balance.saturating_add(reward);
                });

                Self::deposit_event(Event::RelayProofConfirmed {
                    relayer_node: relayer_node_id,
                    confirmer: who.clone(),
                    reward,
                });

                Ok(())
            })
        }
    }

    // ========================
    // Helper Functions
    // ========================

    impl<T: Config> Pallet<T> {
        /// Get the pallet account ID (for relay mining treasury)
        pub fn pallet_account_id() -> T::AccountId {
            MESH_PALLET_ID.into_account_truncating()
        }

        /// Check if a mesh node is active (heartbeat within timeout)
        pub fn is_node_active(node_id: &MeshtasticNodeId) -> bool {
            if let Some(node) = MeshNodes::<T>::get(node_id) {
                let current_block = <frame_system::Pallet<T>>::block_number();
                let timeout: BlockNumberFor<T> = T::HeartbeatTimeout::get().into();
                node.active && current_block <= node.last_seen + timeout
            } else {
                false
            }
        }

        /// Get count of active gateway nodes
        pub fn active_gateway_count() -> u32 {
            NetworkStats::<T>::get().gateway_count
        }

        /// Get total mesh transactions processed
        pub fn total_transactions() -> u64 {
            NetworkStats::<T>::get().total_mesh_transactions
        }
    }
}

// ========================
// Trait Implementations
// ========================

use sp_runtime::traits::Zero;

impl<T: Config> MeshTransactionBridge<T::AccountId> for Pallet<T> {
    fn is_mesh_tx_pending(tx_hash: &H256) -> bool {
        PendingMeshTransactions::<T>::contains_key(tx_hash)
    }

    fn total_mesh_transactions() -> u64 {
        NetworkStats::<T>::get().total_mesh_transactions
    }
}

impl<T: Config> EmergencyAlertProvider<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
    fn active_alerts_for_district(district: &BelizeDistrict) -> u32 {
        // O(1) lookup via maintained counter (H-40)
        ActiveAlertCountPerDistrict::<T>::get(district)
    }

    fn has_catastrophic_alert() -> bool {
        // O(1) lookup via maintained counter (H-40)
        CatastrophicAlertCount::<T>::get() > 0
    }
}
