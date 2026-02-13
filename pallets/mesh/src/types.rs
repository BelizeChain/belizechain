//! Type definitions for the Meshtastic mesh network pallet
//!
//! Meshtastic uses LoRa (Long Range) radio for mesh networking:
//! - Frequency: 915 MHz (Americas) / 868 MHz (EU) / 433 MHz (Asia)
//! - Range: 1-15+ km depending on terrain and antenna
//! - Bandwidth: ~2.4 kbps effective (237 bytes max per packet)
//! - Phone-to-radio: Bluetooth Low Energy (BLE)
//! - Radio-to-radio: LoRa mesh with store-and-forward

use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::BoundedVec;
use frame_support::pallet_prelude::ConstU32;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;

// ==================================
// Meshtastic Node Types
// ==================================

/// Meshtastic hardware node registered on-chain
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MeshNode<AccountId, BlockNumber> {
    /// On-chain account that owns this node
    pub owner: AccountId,
    /// Unique Meshtastic node ID (4-byte hardware ID)
    pub node_id: MeshtasticNodeId,
    /// Node role in the mesh network
    pub role: MeshNodeRole,
    /// Hardware type
    pub hardware: MeshHardware,
    /// LoRa region configuration
    pub region: LoRaRegion,
    /// GPS coordinates (latitude * 1e7, longitude * 1e7) for coverage mapping
    pub latitude: i32,
    pub longitude: i32,
    /// Altitude in meters
    pub altitude: i16,
    /// Whether this node has internet gateway capability
    pub is_gateway: bool,
    /// Total messages relayed (for relay mining rewards)
    pub messages_relayed: u64,
    /// Total transactions relayed to blockchain
    pub transactions_relayed: u32,
    /// Total emergency alerts broadcast
    pub emergency_alerts_sent: u32,
    /// Block when node was registered
    pub registered_at: BlockNumber,
    /// Block when node last reported (heartbeat)
    pub last_seen: BlockNumber,
    /// Node reputation score (0-10000)
    pub reputation: u32,
    /// Whether node is currently active
    pub active: bool,
}

/// 4-byte Meshtastic node identifier (matches Meshtastic protocol)
pub type MeshtasticNodeId = [u8; 4];

/// Role a mesh node plays in the network
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum MeshNodeRole {
    /// Client node: phone + Meshtastic radio for end-user transactions
    #[default]
    Client,
    /// Router node: dedicated relay, always-on, solar-powered
    Router,
    /// Router-Client hybrid: relays and also used for transactions
    RouterClient,
    /// Gateway node: bridges mesh ↔ internet (submits txns to blockchain)
    Gateway,
    /// Validator relay: validator node with Meshtastic for block header relay
    ValidatorRelay,
    /// Emergency beacon: dedicated disaster/emergency broadcast node
    EmergencyBeacon,
}

/// Supported Meshtastic hardware
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum MeshHardware {
    /// Heltec LoRa 32 V3 (ESP32-S3 + SX1262)
    HeltecV3,
    /// LILYGO T-Beam (ESP32 + SX1276 + GPS + 18650 battery)
    TBeam,
    /// LILYGO T-Beam Supreme (ESP32-S3 + SX1262 + GPS)
    TBeamSupreme,
    /// RAK WisBlock (nRF52840 + SX1262) - low power
    RAKWisBlock,
    /// Heltec Wireless Tracker (ESP32-S3 + SX1262 + GPS + display)
    HeltecTracker,
    /// Station G2 (high-power gateway, 1W output)
    StationG2,
    /// DIY/Custom hardware
    Custom,
    /// Unknown/unspecified
    #[default]
    Unknown,
}

/// LoRa frequency region (ISM band regulations)
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum LoRaRegion {
    /// 915 MHz - United States, Belize, Central America, South America
    #[default]
    US915,
    /// 868 MHz - Europe
    EU868,
    /// 433 MHz - Asia
    CN433,
    /// 923 MHz - Australia, New Zealand
    AU915,
    /// 865 MHz - India
    IN865,
    /// 920 MHz - Japan
    JP920,
    /// 920 MHz - Korea
    KR920,
    /// 923 MHz - Taiwan, Thailand
    AS923,
}

// ==================================
// Mesh Transaction Types
// ==================================

/// Compressed transaction for LoRa transmission
/// Must fit within Meshtastic's ~237 byte payload limit
///
/// Format: [version(1)] [type(1)] [sender(4)] [recipient(4)] [amount(8)]
///         [nonce(4)] [signature(64)] [flags(1)] = 87 bytes base
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MeshTransaction<BlockNumber> {
    /// Protocol version (for forward compatibility)
    pub version: u8,
    /// Transaction type
    pub tx_type: MeshTxType,
    /// Compact sender identifier (derived from AccountId)
    pub sender_compact: [u8; 4],
    /// Compact recipient identifier (derived from AccountId)
    pub recipient_compact: [u8; 4],
    /// Amount in smallest unit (picoDALLA or picoBZD)
    pub amount: u64,
    /// Sender's mesh transaction nonce (prevents replay)
    pub nonce: u32,
    /// Ed25519 or post-quantum compressed signature
    pub signature_hash: H256,
    /// Transaction flags
    pub flags: MeshTxFlags,
    /// Block when this was received by a gateway
    pub received_at: Option<BlockNumber>,
    /// Gateway node that bridged this to the blockchain
    pub gateway_node: Option<MeshtasticNodeId>,
    /// Relay path: sequence of node IDs that relayed this message
    pub relay_path: BoundedVec<MeshtasticNodeId, ConstU32<8>>,
    /// Number of hops through the mesh
    pub hop_count: u8,
    /// RSSI (signal strength) at final gateway in dBm
    pub rssi: i16,
    /// SNR (signal-to-noise ratio) at final gateway in dB * 10
    pub snr: i16,
}

/// Mesh transaction type
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum MeshTxType {
    /// Transfer DALLA tokens
    #[default]
    TransferDalla,
    /// Transfer bBZD stablecoin
    TransferBbzd,
    /// Governance vote (compact: proposal_id + vote)
    GovernanceVote,
    /// Identity verification ping (heartbeat / proof-of-presence)
    IdentityPing,
    /// Mesh node status update
    NodeStatus,
}

/// Transaction flags (bitfield)
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MeshTxFlags {
    /// Bit 0: urgent (higher relay priority)
    pub urgent: bool,
    /// Bit 1: requires confirmation relay back through mesh
    pub confirm: bool,
    /// Bit 2: encrypted payload (E2E between sender/recipient)
    pub encrypted: bool,
    /// Bit 3: is a retry/rebroadcast
    pub retry: bool,
}

impl Default for MeshTxFlags {
    fn default() -> Self {
        MeshTxFlags {
            urgent: false,
            confirm: true,
            encrypted: false,
            retry: false,
        }
    }
}

// ==================================
// Emergency Alert Types
// ==================================

/// Emergency alert broadcast through the mesh network
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EmergencyAlert<AccountId, BlockNumber> {
    /// Unique alert ID
    pub alert_id: u32,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Type of emergency
    pub alert_type: EmergencyType,
    /// Who issued the alert
    pub issuer: AccountId,
    /// GPS center point (lat * 1e7, lon * 1e7)
    pub latitude: i32,
    pub longitude: i32,
    /// Affected radius in meters
    pub radius_meters: u32,
    /// Short message (fits in LoRa payload)
    pub message: BoundedVec<u8, ConstU32<128>>,
    /// Block when alert was created
    pub created_at: BlockNumber,
    /// Block when alert expires
    pub expires_at: BlockNumber,
    /// Whether alert has been resolved
    pub resolved: bool,
    /// Number of mesh nodes that relayed this alert
    pub relay_count: u32,
    /// Number of confirmations from mesh nodes in affected area
    pub confirmations: u32,
}

/// Emergency severity levels
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialOrd, Ord, Default)]
pub enum AlertSeverity {
    /// Advisory: informational, no immediate danger
    #[default]
    Advisory,
    /// Watch: conditions possible, be prepared
    Watch,
    /// Warning: event expected, take action
    Warning,
    /// Emergency: immediate threat to life/property
    Emergency,
    /// Catastrophic: widespread destruction, all-hands response
    Catastrophic,
}

/// Types of emergencies
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum EmergencyType {
    /// Hurricane (Belize is in hurricane belt)
    Hurricane,
    /// Tropical storm
    TropicalStorm,
    /// Flooding (coastal and river flooding)
    Flooding,
    /// Earthquake
    Earthquake,
    /// Tsunami warning
    Tsunami,
    /// Wildfire
    Wildfire,
    /// Severe weather (lightning, hail, tornado)
    SevereWeather,
    /// Public safety (active threat, chemical spill, etc.)
    PublicSafety,
    /// Infrastructure failure (bridge collapse, dam breach, power grid)
    InfrastructureFailure,
    /// Medical emergency (disease outbreak, hospital overflow)
    MedicalEmergency,
    /// Search and rescue
    SearchAndRescue,
    /// General alert
    #[default]
    General,
}

// ==================================
// Validator Mesh Relay Types
// ==================================

/// Compressed block header for mesh relay between validators
/// When internet connectivity fails, validators relay critical
/// consensus data through the LoRa mesh network
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MeshBlockHeader {
    /// Block number
    pub block_number: u32,
    /// Block hash (Blake2b-256)
    pub block_hash: H256,
    /// Parent hash
    pub parent_hash: H256,
    /// State root
    pub state_root: H256,
    /// Extrinsics root
    pub extrinsics_root: H256,
    /// Block author (compact validator ID)
    pub author_compact: [u8; 4],
    /// Number of extrinsics in block
    pub extrinsic_count: u16,
    /// Timestamp (Unix seconds, compact)
    pub timestamp: u32,
}

/// Proof that a mesh node relayed data (for relay mining rewards)
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RelayProof<BlockNumber> {
    /// Node that performed the relay
    pub relayer: MeshtasticNodeId,
    /// What was relayed
    pub relay_type: RelayType,
    /// Hash of relayed content
    pub content_hash: H256,
    /// Source node
    pub source_node: MeshtasticNodeId,
    /// Destination node (or broadcast)
    pub destination: RelayDestination,
    /// RSSI at relay point
    pub rssi: i16,
    /// SNR at relay point
    pub snr: i16,
    /// Timestamp
    pub relayed_at: BlockNumber,
    /// Whether delivery was confirmed
    pub confirmed: bool,
}

/// Type of content relayed
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RelayType {
    /// Relayed a user transaction
    Transaction,
    /// Relayed a block header (validator consensus)
    BlockHeader,
    /// Relayed an emergency alert
    EmergencyAlert,
    /// Relayed a node heartbeat/status
    Heartbeat,
    /// Relayed a transaction confirmation back to sender
    Confirmation,
}

/// Relay destination
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RelayDestination {
    /// Specific node
    Node(MeshtasticNodeId),
    /// Broadcast to all nodes in range
    Broadcast,
    /// Nearest gateway (for transaction submission)
    NearestGateway,
}

// ==================================
// Mesh Network Statistics
// ==================================

/// Aggregate mesh network statistics (stored on-chain)
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct MeshNetworkStats {
    /// Total registered mesh nodes
    pub total_nodes: u32,
    /// Active nodes (heartbeat within last 100 blocks)
    pub active_nodes: u32,
    /// Total gateway nodes
    pub gateway_count: u32,
    /// Total router nodes  
    pub router_count: u32,
    /// Total validator relay nodes
    pub validator_relay_count: u32,
    /// Total emergency beacon nodes
    pub emergency_beacon_count: u32,
    /// Total transactions relayed through mesh
    pub total_mesh_transactions: u64,
    /// Total emergency alerts broadcast
    pub total_emergency_alerts: u32,
    /// Total block headers relayed (validator mesh)
    pub total_block_headers_relayed: u64,
    /// Total relay proofs submitted
    pub total_relay_proofs: u64,
    /// Total relay mining rewards distributed (in picoDALLA)
    pub total_relay_rewards: u128,
    /// Estimated coverage area in square kilometers
    pub estimated_coverage_km2: u32,
}

// ==================================
// Coverage & Topology Types
// ==================================

/// Coverage zone registered by a mesh node
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CoverageZone {
    /// Center latitude (* 1e7)
    pub latitude: i32,
    /// Center longitude (* 1e7)
    pub longitude: i32,
    /// Estimated coverage radius in meters
    pub radius_meters: u32,
    /// Belize district this node covers
    pub district: BelizeDistrict,
    /// Terrain type (affects signal propagation)
    pub terrain: TerrainType,
}

/// Belize administrative districts
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum BelizeDistrict {
    /// Belize District (Belize City)
    #[default]
    Belize,
    /// Cayo District (San Ignacio, western)
    Cayo,
    /// Orange Walk District (northern)
    OrangeWalk,
    /// Corozal District (northernmost, border with Mexico)
    Corozal,
    /// Stann Creek District (coastal, Dangriga)
    StannCreek,
    /// Toledo District (southernmost, Punta Gorda)
    Toledo,
}

/// Terrain classification (affects LoRa range estimates)
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub enum TerrainType {
    /// Flat coastal (best LoRa range, 10-15+ km)
    #[default]
    Coastal,
    /// Urban (reduced range, 1-3 km due to buildings)
    Urban,
    /// Suburban (moderate range, 3-8 km)
    Suburban,
    /// Rural flat (excellent range, 8-15+ km)
    RuralFlat,
    /// Jungle/forest (reduced range, 2-5 km, canopy absorption)
    Jungle,
    /// Mountainous (variable, 5-20+ km with elevation advantage)
    Mountain,
    /// Island/caye (excellent over water, 15-25+ km)
    Island,
    /// River valley (good range along valleys, 5-12 km)
    RiverValley,
}
