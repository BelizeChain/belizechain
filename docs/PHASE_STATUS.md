# BelizeChain Phase Status

> **Last Updated**: 2026-10-25
>
> **This file is a structural summary, not live state.** Volatile values —
> `spec_version`, head height, validator count, endpoints, deployed contract
> addresses — are deliberately **not** restated here, because a second copy
> drifts. Read [`ECOSYSTEM_STATE.md`](ECOSYSTEM_STATE.md) §2 for the current
> values and the date each was verified.
>
> **SDK**: Polkadot SDK `stable2609` (rev `7a8daa68e18718c18823c1109513fdc1c8aa71fe`)

---

## Custom Pallet Roster (20 pallets, runtime indexes 20–39)

All 20 custom pallets are compiled into the runtime. The source of truth is
`runtime/src/lib.rs` `construct_runtime!` plus the `Cargo.toml` workspace
members — not this table.

The "Phase" column is development history. "Verified" there means the pallet was
exercised on a testnet at that phase; it is **not** a statement about current
live behaviour. The "Live Testnet Extrinsics" section at the end of this file is
a dated record from 2026-09-03.

| Index | Pallet | Category | Status / Phase |
|---|---|---|---|
| 20 | Economy | Economy | Phase 1 (Verified) |
| 21 | Identity | Identity | Phase 1 (Verified) |
| 22 | Governance | Governance | Phase 1 (Verified) |
| 23 | Compliance | Identity | Phase 1 (Verified) |
| 24 | Staking | Economy | Phase 1 (Verified) |
| 25 | Oracle | Infrastructure | Phase 1 (Verified) |
| 26 | Payroll | Economy | Phase 2 (Verified) |
| 27 | Interoperability | Infrastructure | Phase 2 (Verified) |
| 28 | BelizeX (DEX) | Economy | Phase 2 (Verified) |
| 29 | LandLedger | Governance | Phase 2 (Verified) |
| 30 | Consensus (PoUW) | Infrastructure | Phase 2 (Verified) |
| 31 | Quantum | Infrastructure | Phase 2 (Verified) |
| 32 | Community | Governance | Phase 3 (Verified) |
| 33 | BNS | Infrastructure | Phase 3 (Verified) |
| 34 | Mesh | Infrastructure | Phase 3 (Verified) |
| 35 | Justice | Governance | **Phase 4A (Verified on live testnet)** |
| 36 | Whistleblower | Governance | **Phase 4B (Verified on live testnet)** |
| 37 | Moderation | Governance | **Phase 5C (Verified on live testnet)** |
| 38 | StorageProof | Infrastructure | In the runtime; present in live chain metadata |
| 39 | ZkAttestation | Identity | In the source at spec 122; verify-only until a trusted setup exists |

`pallet_contracts` lives at index 13 and is a Substrate-standard pallet; it is
**not** counted in the 20 custom pallets.

---

## Pre-Mainnet Removal Checklist

> **No mainnet is planned.** These items stay for as long as the network is a
> testnet — removing them is a mainnet-launch task, not general cleanup. Do not
> "fix" them opportunistically; doing so would remove the bootstrap path the
> testnet still depends on.

The following are scaffolded for testnet bootstrapping and **must be removed
before a mainnet launch**:

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

Ports and routes below mirror `infra/docker-compose.ceiba.yml` and
`infra/nginx/nginx.conf`; if they disagree, the compose/nginx files win. Live health
belongs in `ECOSYSTEM_STATE.md` §2, not here.

| Service | Container Port | Public Route | Status |
|---|---|---|---|
| `ceiba-node` (Substrate) | 9944 (internal) | `wss://testnet.belizechain.org/ws`, `/rpc` | Live |
| Nawal AI | 8080 | `/api/nawal/` | Live; FL workload + Prometheus pending |
| Kinich Quantum | 8888 | `/api/kinich/` | Live; activation workflows pending |
| Pakit Storage | 8001 | `/api/pakit/` | Live; DAG activation workflows pending |
| Pakit P2P | 8081 (internal) | — | Live |
| UI (Maya Wallet + Blue Hole Portal) | 3000 | `/` | Page wiring tracked in `ui/UI_WIRING_STATUS_CURRENT.md` — do not restate a count here |
| IPFS Gateway | container only | `/ipfs/` (read-only) | Live, legacy pinned content |

**Note**: Pakit and Nawal local development ports differ from the Ceiba
container ports (Pakit local 8080 / Ceiba 8001; Nawal local 8000 / Ceiba 8080).

---

## GEM Contract Deployment

Contract addresses are **not** listed here. The addresses this section used to
carry were wiped by the 2026-09-29 re-genesis and must not be reused or quoted.

Current addresses live in the gem repo's deployment manifest
(`gem/deployment-*.json`) and in the UI's `CEIBA_GEM_DEPLOYMENT` config. If those
two disagree, the manifest is the record of what was deployed — verify on-chain
before quoting any address.

Contract inventory and per-contract line counts are in the gem repo's README;
this file does not restate them.

---

## Live Testnet Extrinsics Verification (2026-09-03)

The ethical safeguard and governance pallets were exercised end-to-end against the live 2-validator Ceiba testnet (`100.81.45.25` — the host **retired 2026-10-07**; this paragraph is a dated record, not current state. See `docs/ECOSYSTEM_STATE.md` for the live chain) using `@polkadot/api` (`scripts/live-pallet-integration-test.js`). All submitted extrinsics were signed, included in canonical blocks, and verified in on-chain storage:

1. **`pallet-belize-moderation` (Index 37, Phase 5C)**:
   - Call: `belizeModeration.flagContent(contentHash, 2)` (Reason: Spam).
   - In block: `#8668` (`0xf5ad18a74ab405de044fce846cdb27d648b226fecf4ea773bc3665438083da3a`).
   - Verified on-chain: `contentFlags` storage entry set to `Spam`, `flagCounts` incremented to `1`.

2. **`pallet-belize-justice` (Index 35, Phase 4A)**:
   - Call: `belizeJustice.openDispute(target, evidenceHash, 1)` (Severity: Moderate).
   - In block: `#8669` (`0xfad71a385124ce3985554db66ec647b71bb9aadf2f98344fe02985cfb624b3ea`).
   - Verified on-chain: Dispute `#2` created with status `Pending`, 100 DALLA bond reserved from disputant, target placed in `RehabStatus::InCoolingOff` until block `#1304669`.

3. **`pallet-belize-whistleblower` (Index 36, Phase 4B)**:
   - Call: `belizeWhistleblower.submitReport(commitment, target, evidenceHash, 1)` (Category: Fraud).
   - In block: `#8670` (`0xd361f3f25ffee63845c378a417aa1a9da97ab194c2a9f6205652b8cac043c546`).
   - Verified on-chain: Whistleblower report `#2` committed with status `Pending`, 10 DALLA bond escrowed, Blake2-256 domain commitment verified.
