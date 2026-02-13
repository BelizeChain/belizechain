# Meshtastic Mesh Network Pallet

**BelizeChain's off-grid communication infrastructure using Meshtastic LoRa mesh networking.**

## Overview

The Mesh pallet (`pallet-belize-mesh`) brings Meshtastic LoRa mesh networking to BelizeChain, enabling three critical capabilities for Belize's unique geography:

1. **Off-Grid P2P Payments** — Maya Wallet users transact via Meshtastic radios when no internet is available (rural communities, cayes, jungle areas)
2. **Validator Mesh Relay** — Validators relay block headers through LoRa mesh when internet connectivity fails, maintaining consensus
3. **Emergency Broadcast System** — Disaster alerts (hurricanes, flooding, tsunamis) pushed to all mesh nodes in affected areas via NEMO

## Why Meshtastic?

Belize's geography presents unique connectivity challenges:

| Challenge | Meshtastic Solution |
|-----------|-------------------|
| Remote cayes with no cell towers | LoRa reaches 15-25 km over water |
| Dense jungle canopy blocks signals | Mesh routing through multiple hops |
| Hurricane season knocks out internet | Mesh operates independently of infrastructure |
| Rural communities lack broadband | Off-grid P2P payments via LoRa |
| Emergency alerts need to reach everyone | Broadcast to all mesh nodes simultaneously |

## Architecture

```
┌──────────────┐    BLE     ┌──────────────┐   LoRa 915MHz   ┌──────────────┐
│  Maya Wallet │◄──────────►│  Meshtastic  │◄───────────────►│  Meshtastic  │
│  (Phone App) │            │  Radio       │                  │  Router Node │
│              │            │  (T-Beam)    │                  │  (Solar)     │
└──────────────┘            └──────────────┘                  └───────┬──────┘
                                                                      │ LoRa
                                                                      ▼
                             ┌──────────────┐    Internet     ┌──────────────┐
                             │  BelizeChain │◄───────────────►│  Gateway Node│
                             │  Blockchain  │                 │  (Station G2)│
                             │  (Substrate) │                 │  + Internet  │
                             └──────────────┘                 └──────────────┘
```

### Communication Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Phone ↔ Radio | Bluetooth Low Energy (BLE) | Maya Wallet talks to Meshtastic radio |
| Radio ↔ Radio | LoRa 915 MHz (ISM band) | Mesh networking between radios |
| Radio ↔ Blockchain | Internet (via Gateway) | Gateway bridges mesh to chain |
| On-Chain | Substrate Pallet | Node registry, relay mining, alerts |

### Transaction Flow

```
1. User signs transaction offline in Maya Wallet
2. Transaction compressed to ~87 bytes (fits LoRa 237-byte payload)
3. Sent via BLE to user's Meshtastic radio (T-Beam, Heltec V3, etc.)
4. Radio broadcasts via LoRa mesh at 915 MHz
5. Message hops through mesh (up to 7 hops, store-and-forward)
6. Gateway node receives message and has internet connectivity
7. Gateway submits transaction to BelizeChain via RPC
8. Blockchain processes transaction
9. Confirmation relays back through mesh to sender's radio
10. Radio sends confirmation via BLE to Maya Wallet
```

## Supported Hardware

| Hardware | Chipset | Features | Best For |
|----------|---------|----------|----------|
| **LILYGO T-Beam** | ESP32 + SX1276 + GPS | GPS, 18650 battery, good range | General purpose |
| **LILYGO T-Beam Supreme** | ESP32-S3 + SX1262 + GPS | Better radio, USB-C | Premium client |
| **Heltec LoRa 32 V3** | ESP32-S3 + SX1262 | OLED display, compact | Client nodes |
| **Heltec Wireless Tracker** | ESP32-S3 + SX1262 + GPS | Display + GPS + compact | Mobile tracking |
| **RAK WisBlock** | nRF52840 + SX1262 | Ultra low-power, modular | Router nodes (solar) |
| **Station G2** | High-power gateway | 1W output, ethernet | Gateway nodes |

## Mesh Node Roles

| Role | Description | KYC Required | Rewards |
|------|-------------|-------------|---------|
| **Client** | End-user with phone + radio | Basic (Level 1) | None |
| **Router** | Dedicated relay node (solar-powered, always-on) | Basic (Level 1) | Relay mining |
| **RouterClient** | Hybrid: relays + personal use | Basic (Level 1) | Relay mining |
| **Gateway** | Bridges mesh ↔ internet | Verified (Level 2) | Relay mining (2x) |
| **ValidatorRelay** | Validator with mesh for consensus fallback | Full (Level 3) + Validator | Block relay mining |
| **EmergencyBeacon** | Dedicated emergency broadcast | Basic (Level 1) | Emergency relay mining |

## Relay Mining

Mesh node operators earn DALLA rewards for relaying data through the network:

| Relay Type | Reward | Description |
|-----------|--------|-------------|
| Transaction | 1 DALLA | Relayed a user's P2P payment |
| Block Header | 0.5 DALLA | Relayed validator consensus data |
| Emergency Alert | 2 DALLA | Relayed a disaster alert |
| Heartbeat | 0.1 DALLA | Relayed node status |
| Confirmation | 0.1 DALLA | Relayed transaction confirmation |

Rewards accumulate on-chain and are claimable via `claim_relay_rewards()`.

## Emergency Broadcast System

### Severity Levels

| Level | Description | Priority | Color |
|-------|-------------|----------|-------|
| Advisory | Informational, no immediate danger | Normal | Blue |
| Watch | Conditions possible, be prepared | Elevated | Yellow |
| Warning | Event expected, take action | High | Orange |
| Emergency | Immediate threat to life/property | Critical | Red |
| Catastrophic | Widespread destruction, all-hands | Maximum | Purple |

### Emergency Types

- Hurricane (Belize is in the hurricane belt)
- Tropical Storm
- Flooding (coastal + river)
- Earthquake / Tsunami
- Wildfire
- Severe Weather
- Public Safety
- Infrastructure Failure
- Medical Emergency
- Search and Rescue

### Authorized Issuers

Only accounts with emergency authority can issue alerts:
- NEMO (National Emergency Management Organization)
- Government officials (via governance origin)
- Root/sudo (for testing)

## LoRa Configuration

### Belize Region

| Parameter | Value |
|-----------|-------|
| Frequency | 915 MHz (US/Americas ISM band) |
| Bandwidth | 125-500 kHz |
| Spreading Factor | SF7-SF12 |
| Max Payload | 237 bytes |
| Effective Data Rate | ~2.4 kbps |
| Default Preset | LongFast |
| Max Hops | 7 (configurable via governance) |

### Range Estimates by Terrain

| Terrain | Estimated Range | Coverage Area |
|---------|----------------|---------------|
| Coastal (cayes, beaches) | 15-25 km | 707-1,963 km² |
| Rural flat (farmland) | 8-15 km | 201-707 km² |
| Urban (Belize City) | 1-3 km | 3-28 km² |
| Suburban | 3-8 km | 28-201 km² |
| Jungle | 2-5 km | 13-79 km² |
| Mountain (Maya Mountains) | 5-20+ km | 79-1,257 km² |
| Over water (island-to-island) | 15-25+ km | Excellent |

## Extrinsics

| # | Extrinsic | Description | Origin |
|---|-----------|-------------|--------|
| 0 | `register_node` | Register Meshtastic node on-chain | Signed (KYC required) |
| 1 | `deregister_node` | Remove node, reclaim deposit | Signed (owner) |
| 2 | `update_node_location` | Update GPS coordinates | Signed (owner) |
| 3 | `node_heartbeat` | Keep node marked as active | Signed (owner) |
| 4 | `submit_mesh_transaction` | Gateway submits mesh-relayed tx | Signed (gateway owner) |
| 5 | `submit_relay_proof` | Submit proof for relay mining | Signed (node owner) |
| 6 | `issue_emergency_alert` | Broadcast emergency alert | Emergency authority |
| 7 | `resolve_emergency_alert` | Mark alert as resolved | Emergency authority |
| 8 | `confirm_emergency_alert` | Confirm alert receipt | Signed (node owner) |
| 9 | `relay_block_header` | Relay compressed block header | Signed (validator relay) |
| 10 | `claim_relay_rewards` | Claim accumulated rewards | Signed |
| 11 | `update_mesh_config` | Update network config | Governance |

## Storage Items

| Storage | Type | Description |
|---------|------|-------------|
| `MeshNodes` | Map(NodeId → MeshNode) | Registered Meshtastic nodes |
| `NodesByOwner` | Map(AccountId → Vec<NodeId>) | Nodes owned per account |
| `PendingMeshTransactions` | Map(Hash → MeshTx) | Transactions received from mesh |
| `ProcessedMeshTransactions` | Map(Hash → BlockNumber) | Processed tx deduplication |
| `EmergencyAlerts` | Map(AlertId → Alert) | Active/resolved alerts |
| `MeshBlockHeaders` | Map(BlockNum → Header) | Block headers relayed via mesh |
| `RelayProofs` | Map(NodeId → Vec<Proof>) | Relay proofs for mining |
| `RelayRewards` | Map(AccountId → Balance) | Unclaimed relay rewards |
| `NetworkStats` | Value(MeshNetworkStats) | Aggregate network statistics |
| `MeshConfig` | Value(MeshNetworkConfig) | Governance-adjustable config |

## Coverage Target: Belize

```
        ┌─────────────────────────────────────┐
        │           COROZAL                    │
        │    📡 Corozal Town                   │
        │                                      │
        ├───────────────┬─────────────────────┤
        │  ORANGE WALK  │                      │
        │  📡 OW Town   │        BELIZE        │
        │               │  📡 Belize City      │
        ├───────────────┤  📡 San Pedro (caye) │
        │     CAYO      │  📡 Caye Caulker     │
        │ 📡 San Ignacio│                      │
        │ 📡 Belmopan   ├─────────────────────┤
        │               │    STANN CREEK       │
        │               │  📡 Dangriga         │
        │               │  📡 Placencia        │
        ├───────────────┤                      │
        │    TOLEDO      │                      │
        │ 📡 Punta Gorda│                      │
        └───────────────┴──────────────────────┘

Target: 50+ mesh nodes covering all 6 districts
Gateway nodes in: Belize City, San Pedro, Belmopan, Dangriga, PG, San Ignacio
```

## Integration with Other Pallets

| Pallet | Integration |
|--------|-------------|
| **Identity** | KYC verification for node registration |
| **Economy** | DALLA/bBZD transfer via mesh transactions |
| **Governance** | Governance votes via compact mesh messages |
| **Staking** | Validator verification for ValidatorRelay nodes |
| **Community** | SRS boost for mesh operators (community service) |
| **Oracle** | Price feed relay through mesh (future) |

## Testing

```bash
# Run mesh pallet tests
cargo test -p pallet-belize-mesh

# Run with output
cargo test -p pallet-belize-mesh -- --nocapture
```

## Future Enhancements

- **Mesh-native payments**: Direct DALLA/bBZD settlement within mesh without gateway
- **Store-and-forward queue**: Batch transactions when gateway is unavailable
- **Position reporting**: Automatic location sharing for search & rescue
- **Mesh coverage map**: On-chain visualization of network coverage
- **Multi-region support**: Extend mesh to Guatemala/Mexico border areas
- **Satellite gateway**: Starlink/Iridium gateway for ultra-remote areas
