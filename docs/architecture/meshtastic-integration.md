# Meshtastic Integration Architecture

**BelizeChain Off-Grid Communication via Meshtastic LoRa Mesh Networking**

**Date**: February 2026  
**Status**: Implementation  
**Pallet**: `pallet-belize-mesh`

---

## Executive Summary

BelizeChain integrates Meshtastic open-source LoRa mesh networking to solve Belize's critical connectivity gaps. This replaces any previous "Bluetooth LE only" approach with a proper **Meshtastic protocol stack** that uses:

- **Bluetooth Low Energy (BLE)**: Phone ↔ Meshtastic radio communication (short range, ~10m)
- **LoRa 915 MHz**: Radio ↔ Radio mesh communication (long range, 1-25+ km per hop)
- **Internet**: Gateway ↔ Blockchain bridging (when available)

This architecture enables off-grid payments, validator consensus, and emergency broadcasts across all of Belize, including remote cayes, Maya Mountains, and dense jungle regions.

---

## Problem Statement

### Belize's Connectivity Challenges

1. **Remote Cayes**: Ambergris Caye, Caye Caulker, Turneffe Atoll, and 200+ cayes have limited or no cell service
2. **Dense Jungle**: Toledo and Cayo districts have thick jungle canopy that blocks traditional signals  
3. **Hurricane Season**: June-November regularly knocks out internet, cell towers, and power
4. **Rural Communities**: Only 36% of Belizean households have internet access (ITU 2024)
5. **Economic Barriers**: Mobile data costs are high relative to average income (~BZ$14/day)

### Why "Bluetooth LE Only" Was Insufficient

A Bluetooth LE-only approach has critical limitations:
- **Range**: BLE reaches only ~10-30 meters (line of sight)
- **No mesh routing**: BLE doesn't naturally form multi-hop mesh networks
- **Requires proximity**: Both parties must be physically close
- **No infrastructure relay**: Can't relay through intermediate nodes to reach gateways
- **No emergency broadcast**: Can't push alerts to wide areas

### Why Meshtastic

Meshtastic solves all of these by adding LoRa radio mesh networking:
- **Range**: 1-25+ km per hop (915 MHz LoRa)
- **Mesh routing**: Automatic store-and-forward through multiple hops (up to 7)
- **Off-grid**: Operates without any internet, cell, or power infrastructure
- **Solar-powered**: Router nodes run on small solar panels indefinitely
- **Open source**: MIT-licensed, no vendor lock-in
- **Affordable hardware**: T-Beam ~$30, Heltec V3 ~$20

---

## System Architecture

### Three-Layer Communication Stack

```
┌─────────────────────────────────────────────────────────────────────┐
│                         LAYER 3: BLOCKCHAIN                         │
│                    BelizeChain Substrate Runtime                     │
│           pallet-belize-mesh (Node Registry, Relay Mining,          │
│              Emergency Alerts, Block Header Relay)                  │
│                    ws://node.belizechain.org:9944                   │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ RPC/WebSocket (Internet)
                                 │
┌────────────────────────────────┴────────────────────────────────────┐
│                       LAYER 2: LoRa MESH                            │
│                   Meshtastic Protocol (915 MHz)                     │
│                                                                     │
│  ┌──────────┐    LoRa    ┌──────────┐    LoRa    ┌──────────┐     │
│  │  Client  │◄──────────►│  Router  │◄──────────►│ Gateway  │─────│
│  │  Node    │            │  (Solar) │            │(Internet)│     │
│  └────┬─────┘            └──────────┘            └──────────┘     │
│       │                       ▲                       ▲            │
│       │ LoRa                  │ LoRa                  │ LoRa      │
│       ▼                       ▼                       ▼            │
│  ┌──────────┐            ┌──────────┐            ┌──────────┐     │
│  │ Validator│            │ Emergency│            │  Client  │     │
│  │  Relay   │            │  Beacon  │            │  Node    │     │
│  └──────────┘            └──────────┘            └────┬─────┘     │
│                                                       │            │
└───────────────────────────────────────────────────────┼────────────┘
                                                        │ BLE
┌───────────────────────────────────────────────────────┴────────────┐
│                        LAYER 1: BLE (LOCAL)                        │
│              Bluetooth Low Energy (Phone ↔ Radio)                  │
│                                                                     │
│  ┌─────────────────────┐         ┌─────────────────────┐          │
│  │     Maya Wallet     │   BLE   │   Meshtastic Radio  │          │
│  │    (iOS/Android)    │◄───────►│    (T-Beam/Heltec)  │          │
│  │                     │  ~10m   │                     │          │
│  │  • Sign transactions│         │  • LoRa TX/RX       │          │
│  │  • View balance     │         │  • GPS positioning  │          │
│  │  • Send/receive     │         │  • Mesh routing     │          │
│  │  • Emergency alerts │         │  • Store & forward  │          │
│  └─────────────────────┘         └─────────────────────┘          │
└───────────────────────────────────────────────────────────────────┘
```

### Off-Grid Payment Flow

```
Step 1: SIGN (Maya Wallet)
┌─────────────────────────────────────────────┐
│  Maya Wallet                                 │
│  ┌─────────────────────────────────────┐    │
│  │ Send 45 bBZD to Maria              │    │
│  │                                     │    │
│  │ [Sign with biometric] ✅            │    │
│  │                                     │    │
│  │ Transaction signed offline          │    │
│  │ Compressed to 87 bytes              │    │
│  │ Ready for mesh transmission         │    │
│  └─────────────────────────────────────┘    │
└──────────────────────┬──────────────────────┘
                       │ BLE (Bluetooth Low Energy)
                       ▼
Step 2: TRANSMIT (Meshtastic Radio)
┌─────────────────────────────────────────────┐
│  T-Beam Radio (ESP32 + SX1276 + GPS)        │
│                                              │
│  Received via BLE: 87-byte compressed tx     │
│  Broadcasting on LoRa 915 MHz...             │
│  Channel: LongFast preset                    │
│  Power: 20 dBm (100 mW)                     │
└──────────────────────┬──────────────────────┘
                       │ LoRa 915 MHz (5 km)
                       ▼
Step 3: RELAY (Router Nodes)
┌─────────────────────────────────────────────┐
│  Solar Router Node (RAK WisBlock)            │
│                                              │
│  Received mesh message (hop 1)               │
│  RSSI: -85 dBm, SNR: 10 dB                  │
│  Store-and-forward: retransmitting...        │
│  Recording relay proof for mining            │
└──────────────────────┬──────────────────────┘
                       │ LoRa 915 MHz (8 km)
                       ▼
Step 4: GATEWAY (Bridge to Internet)
┌─────────────────────────────────────────────┐
│  Gateway Node (Station G2 + Internet)        │
│                                              │
│  Received mesh message (hop 2)               │
│  Decompressing transaction...                │
│  Submitting to BelizeChain via RPC           │
│  → ws://node.belizechain.org:9944            │
└──────────────────────┬──────────────────────┘
                       │ Internet (RPC)
                       ▼
Step 5: ON-CHAIN (BelizeChain)
┌─────────────────────────────────────────────┐
│  BelizeChain Runtime                         │
│                                              │
│  Mesh::submit_mesh_transaction()             │
│  ✅ Signature verified                       │
│  ✅ Nonce valid (no replay)                  │
│  ✅ Sufficient balance                       │
│  ✅ 45 bBZD transferred to Maria             │
│                                              │
│  Relay mining rewards accrued:               │
│  • Router node: +1 DALLA                     │
│  • Gateway node: +1 DALLA                    │
└──────────────────────┬──────────────────────┘
                       │ Confirmation
                       ▼ (reverse path through mesh)
Step 6: CONFIRMATION (Back to Maya Wallet)
┌─────────────────────────────────────────────┐
│  Maya Wallet                                 │
│  ┌─────────────────────────────────────┐    │
│  │ ✅ Transaction Confirmed!           │    │
│  │                                     │    │
│  │ Sent: 45 bBZD to Maria             │    │
│  │ Block: #1,234,567                   │    │
│  │ Hops: 2 (radio → router → gateway) │    │
│  │ Time: ~45 seconds                   │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

### Compressed Transaction Format (87 bytes)

```
┌───────────────────────────────────────────────────────┐
│ Byte  │ Field              │ Size  │ Description      │
├───────┼────────────────────┼───────┼──────────────────┤
│ 0     │ Version            │ 1B    │ Protocol version │
│ 1     │ Transaction Type   │ 1B    │ DALLA/bBZD/Vote  │
│ 2-5   │ Sender Compact ID  │ 4B    │ First 4B of hash │
│ 6-9   │ Recipient Compact  │ 4B    │ First 4B of hash │
│ 10-17 │ Amount             │ 8B    │ picoDALLA/picoBZD│
│ 18-21 │ Nonce              │ 4B    │ Replay protection│
│ 22-53 │ Signature Hash     │ 32B   │ Ed25519/PQ sig   │
│ 54    │ Flags              │ 1B    │ Urgent/Confirm   │
│ 55-86 │ Full Signature     │ 32B   │ Remaining sig    │
├───────┼────────────────────┼───────┼──────────────────┤
│ Total │                    │ 87B   │ Fits LoRa 237B   │
└───────────────────────────────────────────────────────┘
```

---

## Validator Mesh Relay

When internet connectivity fails (hurricane, infrastructure damage), validators maintain consensus awareness through LoRa mesh relay of compressed block headers.

```
Validator A                    Validator B                    Validator C
(San Ignacio)                  (Belmopan)                    (Belize City)
┌──────────────┐              ┌──────────────┐              ┌──────────────┐
│ Produced      │   LoRa      │ Received      │   LoRa      │ Received      │
│ Block #42     │────────────►│ Block #42     │────────────►│ Block #42     │
│               │  12 km      │ Header via    │   15 km     │ Header via    │
│ Internet: ❌  │              │ mesh          │              │ mesh          │
│ Mesh: ✅      │              │ Internet: ❌  │              │ Internet: ❌  │
└──────────────┘              └──────────────┘              └──────────────┘
```

Block headers are compressed to fit LoRa payload:

| Field | Size | Total |
|-------|------|-------|
| Block number | 4B | 4B |
| Block hash | 32B | 36B |
| Parent hash | 32B | 68B |
| State root | 32B | 100B |
| Extrinsics root | 32B | 132B |
| Author compact | 4B | 136B |
| Extrinsic count | 2B | 138B |
| Timestamp | 4B | 142B |
| **Total** | | **142B** (fits 237B limit) |

---

## Emergency Broadcast System

### Integration with NEMO

The National Emergency Management Organization (NEMO) of Belize is the primary authority for emergency alerts. The mesh pallet integrates with NEMO through:

1. **Authorized accounts**: NEMO officials have `emergency_authority` flag in Identity pallet
2. **Severity levels**: Advisory → Watch → Warning → Emergency → Catastrophic
3. **Geo-targeting**: Alerts target specific GPS coordinates + radius
4. **District targeting**: Alerts specify which Belize district(s) are affected
5. **Automatic relay**: All mesh nodes prioritize emergency messages over normal traffic

### Hurricane Alert Example

```
╔══════════════════════════════════════════════════════╗
║  🔴 EMERGENCY ALERT                                  ║
║                                                       ║
║  Type: HURRICANE                                      ║
║  Severity: CATASTROPHIC                               ║
║  Issued by: NEMO Belize                               ║
║                                                       ║
║  Category 4 Hurricane approaching Belize coast.       ║
║  Expected landfall: Belize City area                  ║
║  Evacuate all coastal areas immediately.              ║
║                                                       ║
║  Affected District: Belize, Stann Creek               ║
║  Radius: 100 km                                       ║
║                                                       ║
║  Relayed by: 47 mesh nodes                            ║
║  Confirmed by: 23 nodes in affected area              ║
╚══════════════════════════════════════════════════════╝
```

---

## Relay Mining Economics

Mesh node operators earn DALLA rewards for providing network coverage and relaying data. This incentivizes building out mesh infrastructure across Belize.

### Reward Structure

| Action | Reward | Rationale |
|--------|--------|-----------|
| Relay transaction | 1.0 DALLA | Core value: enabling off-grid payments |
| Relay emergency alert | 2.0 DALLA | Critical service: saving lives |
| Relay block header | 0.5 DALLA | Consensus support |
| Relay heartbeat | 0.1 DALLA | Network health monitoring |
| Relay confirmation | 0.1 DALLA | User experience |

### ROI Estimate for Node Operators

| Setup | Cost | Monthly Reward Est. | Payback |
|-------|------|-------------------|---------|
| T-Beam client | ~$30 | 10-30 DALLA | 1-3 months |
| RAK router (solar) | ~$80 | 50-200 DALLA | 1-2 months |
| Station G2 gateway | ~$150 | 200-500 DALLA | 1 month |

### Anti-Gaming Protections

1. **Relay proofs require valid node registration** (KYC-backed)
2. **Signature verification** on relay proofs
3. **Reputation scoring** (0-10,000) — low-rep nodes get reduced rewards
4. **Heartbeat timeout** — inactive nodes can't claim rewards
5. **Governance-adjustable parameters** — rates can be tuned

---

## Deployment Plan

### Phase 1: Core Infrastructure (Q1 2026)
- Deploy gateway nodes in 6 district capitals
- Register 25+ router nodes along major highways
- Maya Wallet Meshtastic BLE integration
- On-chain pallet deployment

### Phase 2: Coverage Expansion (Q2 2026)
- Deploy router nodes on cayes (San Pedro, Caye Caulker, Turneffe)
- Maya Mountains coverage (Cayo district)
- Toledo district deep-south coverage
- Relay mining incentive program launch

### Phase 3: Emergency System (Q3 2026)
- NEMO integration for authorized alert issuance
- Emergency beacon deployment at NEMO offices
- Hurricane season preparedness deployment
- Alert confirmation tracking

### Phase 4: Full Coverage (Q4 2026)
- 200+ mesh nodes nationwide
- All 6 districts with gateway coverage
- Validator mesh relay for consensus resilience
- Cross-border mesh (Guatemala, Mexico border areas)

---

## Hardware Deployment Recommendations

### Gateway Nodes (6 minimum)

| Location | Hardware | Power | Internet |
|----------|----------|-------|----------|
| Belize City | Station G2 | Grid + UPS | Fiber |
| Belmopan | Station G2 | Grid + UPS | Fiber |
| San Pedro | Station G2 | Solar + Battery | Starlink |
| San Ignacio | Station G2 | Grid + UPS | DSL |
| Dangriga | Station G2 | Grid + Solar | LTE |
| Punta Gorda | Station G2 | Solar + Battery | LTE/Starlink |

### Router Nodes (50+ target)

| Area | Hardware | Power | Coverage |
|------|----------|-------|----------|
| Highway corridors | RAK WisBlock | Solar + 18650 | 8-15 km |
| Village centers | Heltec V3 | Solar | 3-8 km |
| Mountain peaks | T-Beam Supreme | Solar + 18650 | 10-20+ km |
| Caye hilltops | RAK WisBlock | Solar | 15-25 km (over water) |

---

## Security Considerations

1. **All transactions are signed offline** — private keys never leave Maya Wallet
2. **Replay protection** via mesh nonces (separate from blockchain nonces)
3. **Gateway verification** — only KYC-verified (Level 2+) accounts can run gateways
4. **Emergency alerts require authority** — only NEMO-authorized accounts
5. **Rate limiting** — maximum relay proofs per claim period
6. **Node registration deposit** — 10 DALLA anti-spam deposit (refundable)
7. **Reputation system** — low-reputation nodes get reduced relay rewards

---

## Related Documentation

- [Pallet README](../../pallets/mesh/README.md)
- [Multi-Repository Overview](./multi-repo-overview.md)
- [Infrastructure Guide](./INFRASTRUCTURE_GUIDE.md)
- [Security Overview](../security/SECURITY_OVERVIEW.md)
