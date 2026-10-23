# Bridges & Interoperability

**Read this before any other file in `docs/bridges/`.** Every other document in this
directory describes a design target, not deployed behaviour. This page is the authority on
what exists.

**Verified 2026-10-23 against `Cargo.toml`, `runtime/src/lib.rs` and
`pallets/interoperability/src/lib.rs`.**

---

## There is no XCM

BelizeChain does not speak XCM and cannot, without becoming a different kind of chain.

XCM is a message format between consensus systems that **share a security root** — parachains
in the same relay ecosystem. It is not a bridge protocol. It does nothing for Ethereum,
Solana, or Bitcoin, and it is not how you move native DOT into a sovereign L1.

| Question | Answer |
|---|---|
| Is `pallet_xcm` in the runtime? | **No.** Zero commits in git history. |
| Is `XcmConfig` defined? | **No.** |
| Is there a cumulus collator? | **No.** `cumulus` appears in no `Cargo.toml`. |
| Is there a parachain slot? | **No.** |
| Are the `staging-xcm*` crates used? | **No.** Removed from `Cargo.toml` on 2026-10-23. |

The `staging-xcm` crate still appears in `Cargo.lock`, because `pallet-contracts` and
`cumulus-primitives-core` depend on it transitively. That is an SDK internal, not a BelizeChain
feature, and it is not reachable from any runtime call.

### Why not build it

XCM is not a feature you add to a chain; it is a relationship the chain enters.

```
cumulus collator binary      ← different binary than belizechain-node
+ relay-chain slot            ← auction or parathread
+ relay chain as security root
+ XcmConfig / Barrier / AssetTransactor / IsReserve
+ pallet-xcm + pallet-assets at new runtime indices
```

A parachain inherits relay-chain governance and upgrade cadence. That directly contradicts the
sovereignty thesis this chain is built on. Treat it as an architecture decision, not a backlog
item.

---

## What does exist: `pallet-belize-interoperability`

Runtime index **27**. 1,971 lines. Benchmarked weights — `register_bridge_validator` and
`submit_incoming_unlock` have real measured weights, not placeholders borrowed from
`initiate_bridge`.

| Surface | Value |
|---|---|
| `BridgeChain` | 51 variants, indices 0–50 (L1s, L2s, and a generic `XCM` aggregation marker) |
| `BridgeAsset` | **DALLA and BBZD only** — a deliberate sovereignty constraint |
| Signature scheme | ML-DSA-87 (post-quantum), via `fips204` |
| KYC gate | Level 2+ for messaging, **Level 3** for bridge operators |
| Sanctions | Screened on every messaging and registration path |
| Escrow | Liquidity pools per `(chain, asset)` with `create_liquidity_pool` / `withdraw_liquidity` |
| Disputes | `dispute_bridge_transaction` with validator adjudication |

Extrinsics: `initiate_bridge`, `provide_pq_signature`, `create_liquidity_pool`, `process_unlock`,
`submit_incoming_unlock`, `send_cross_chain_message`, `update_bridge_config`,
`dispute_bridge_transaction`, `register_bridge_validator`, `remove_bridge_validator`,
`withdraw_liquidity`, `confirm_burn_proof`.

---

## Two gaps, both operational

Neither is a code defect. Both are things nobody has staffed yet.

### 1. No relayer

Nothing outside `pallets/interoperability/src/tests.rs` ever calls `process_unlock` or
`submit_incoming_unlock`. No service in `infra/docker-compose.ceiba.yml` drives the bridge, and
no relayer code exists in any repo in the organisation.

The state machine is complete. It is unstaffed. Inbound value cannot arrive until something
watches a foreign chain and submits unlocks.

### 2. Messages are recorded, not delivered

`send_cross_chain_message` writes `CrossChainMessage { delivered: false, .. }` — and **nothing in
the pallet ever sets it to `true`**. There is no deliver or acknowledge extrinsic. The storage map
is an append-only log with a boolean that is permanently false.

Calling it a transport today would be false. It is a signed, KYC-gated, fee-charged record of
intent.

---

## Status summary

| Capability | State |
|---|---|
| XCM to Polkadot / Kusama / Moonbeam | **Not built.** Requires a parachain. |
| Bridge state machine (lock/mint, burn/unlock) | **Built**, runtime index 27 |
| Post-quantum bridge signatures | **Built** (ML-DSA-87) |
| Liquidity pools, escrow, disputes | **Built** |
| Outbound message recording | **Built** |
| Outbound message *delivery* | **Not built** — no ack path |
| Inbound unlock submission | **Built but unstaffed** — no relayer |
| Any bridge worker in `infra/` | **Does not exist** |

---

## Where the old claims live

The following files still describe XCM as shipped. They are design documents, and should be read
with this page in hand:

- `docs/bridges/RELAYER_NETWORK.md` — banner added 2026-10-23
- `docs/bridges/ASSET_MANAGEMENT.md` — wDOT / wAUSD / wKSM table
- `docs/bridges/SECURITY_MONITORING.md` — `bridge.polkadot.xcm` metric
- `docs/operations/bridges-dex-landledger.md` — `api.tx.xcmPallet.limitedReserveTransferAssets`
- `docs/developer-guides/migration-guides.md` — `impl pallet_xcm::Config` block
- `docs/security/{AUDIT_FRAMEWORK,PENETRATION_TESTING_PROGRAM,INCIDENT_RESPONSE_PLAN}.md`
- `docs/performance/PERFORMANCE_SCALABILITY_FRAMEWORK.md` — "✅ COMPLETE, 3,350 lines"

---

## Next step, if you want working bridges

The missing piece is a relayer service in `infra/`, plus one runtime extrinsic to acknowledge
message delivery. Neither requires XCM, and neither requires a parachain.

Full breakdown in **[RELAYER_SCOPE.md](RELAYER_SCOPE.md)** — including the four prerequisites
that must be answered before any relayer code is worth writing. The short version: the cheapest
real win is the acknowledge extrinsic, and the expensive blocker is that there is no deployed
bridge contract on any foreign chain yet.
