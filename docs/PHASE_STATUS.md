# BelizeChain Phase Status

> **Last Updated**: 2026-02-12
> **Runtime**: `spec_version = 105`, `impl_version = 1`, `tx_version = 1`
> **SDK**: Polkadot SDK `stable2603` (rev `2e4dd0bc22366a5af820492528869a493b5a5208`)
> **Network**: Ceiba testnet (`chain_id = belizechain_testnet`, SS58 prefix `1981`)

---

## Custom Pallet Roster (18 pallets, runtime indexes 20–37)

| Index | Pallet | Category | Phase |
|---|---|---|---|
| 20 | Economy | Economy | Phase 1 |
| 21 | Identity | Identity | Phase 1 |
| 22 | Governance | Governance | Phase 1 |
| 23 | Compliance | Identity | Phase 1 |
| 24 | Staking | Economy | Phase 1 |
| 25 | Oracle | Infrastructure | Phase 1 |
| 26 | Payroll | Economy | Phase 2 |
| 27 | Interoperability | Infrastructure | Phase 2 |
| 28 | BelizeX (DEX) | Economy | Phase 2 |
| 29 | LandLedger | Governance | Phase 2 |
| 30 | Consensus (PoUW) | Infrastructure | Phase 2 |
| 31 | Quantum | Infrastructure | Phase 2 |
| 32 | Community | Governance | Phase 3 |
| 33 | BNS | Infrastructure | Phase 3 |
| 34 | Mesh | Infrastructure | Phase 3 |
| 35 | Justice | Governance | **Phase 4A** |
| 36 | Whistleblower | Governance | **Phase 4B** |
| 37 | Moderation | Governance | **Phase 5C** |

`pallet_contracts` lives at index 13 and is a Substrate-standard pallet; it is
**not** counted in the 18 custom pallets.

---

## Pre-Mainnet Removal Checklist

The following are scaffolded for testnet bootstrapping and **must be removed
before mainnet launch**:

- `pallet_sudo` at runtime index 6 (see `runtime/src/lib.rs:600` and
  AR-3 note at line 221).
- `EnsureRoot` fallbacks in all governance origin types
  (`runtime/src/lib.rs:877–920`). Tracked by the `TODO: remove EnsureRoot
  fallback before mainnet launch` marker at line 880.
- `EnsureRoot` on collective `SetMembersOrigin` /
  `DisapproveOrigin` / `KillOrigin` (`runtime/src/lib.rs:844–870`) — replaced
  by on-chain governance post-launch.

---

## Service Activation Status (Ceiba testnet)

| Service | Container Port | Public Route | Status |
|---|---|---|---|
| `ceiba-node` (Substrate) | 9944 (internal) | `wss://testnet.belizechain.org/ws`, `/rpc` | Live |
| Nawal AI | 8080 | `/api/nawal/` | Live; FL workload + Prometheus pending |
| Kinich Quantum | 8888 | `/api/kinich/` | Live; activation workflows pending |
| Pakit Storage | 8001 | `/api/pakit/` | Live; DAG activation workflows pending |
| Pakit P2P | 8081 (internal) | — | Live |
| UI (Maya Wallet + Blue Hole Portal) | 3000 | `/` | 9/14 wallet pages live |
| IPFS Gateway | container only | `/ipfs/` (read-only) | Live, legacy pinned content |

**Note**: Pakit and Nawal local development ports differ from the Ceiba
container ports (Pakit local 8080 / Ceiba 8001; Nawal local 8000 / Ceiba 8080).

---

## GEM Contract Deployment (May 11–12 2026)

Live on `belizechain_testnet` (full addresses in
`gem/deployment-*.json` artifacts in the gem repo):

- DALLA Token (PSP22) — `r1Vkg9k4vy7YcgSES8HMPrbtAsr6QWZcCixDQ19saBYfQv9me`
- BeliNFT (PSP34) — `r1V67KtGB3i2mL4421WSd5WVfWPRuwwUVu6VMwDYfzobjVKHE`
- Simple DAO — `r1WcVzmmX6bXvX4wpoR1W8ffA3Chr8UU7PtxXE8JQG4RZkzSK`
- Faucet (1000 DALLA / 100 blocks) — `r1WXkqkVdPb4ap9SYPi7zdKa2PXjYUGzve5HsPn6FaezKRJvX`
- PSP37 Multi-Token — `r1U6k8Unb1gbnhQ8KHmqxKQvmWEQ5nxqeSHPL7VBkiUGKRYwM`
- BelizeX DEX — Factory `r1UZZBGTX6cRLSYSvL2i6cCGXvRF9JY9TtM4rmaXtgAr73DGQ`,
  Router `r1XKmdL9wopmVJedPepW76oPYTV1tFdTaPsHT73CqfZruPww1`,
  Pair code hash `0x96de81afced1e99600f0f54e513fcdb95d167c01a8a7c644c9f849f43f8a5c69`

**Pending deployment**: `access-control` (library, optional standalone deploy),
`hello-belizechain` (tutorial-only).
