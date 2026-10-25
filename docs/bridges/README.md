# Bridges & Interoperability

**Read this before any other file in `docs/bridges/`.** Every other document in this
directory describes a design target, not deployed behaviour. This page is the authority on
what exists.

**Verified 2026-10-23 against `Cargo.toml`, `runtime/src/lib.rs` and
`pallets/interoperability/src/lib.rs`.** Re-verified 2026-10-25: the relayer and the
acknowledge path are described below as they now are — the earlier revision of this page said
neither existed, which was true when it was written and stopped being true two days later.

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
`submit_incoming_unlock`, `send_cross_chain_message`, `confirm_external_delivery` (index 12),
`set_evm_address` (index 13), `update_bridge_config`, `dispute_bridge_transaction`,
`register_bridge_validator`, `remove_bridge_validator`, `withdraw_liquidity`, `confirm_burn_proof`.

---

## Two gaps, both operational

Neither is a code defect. Both are things nobody has staffed yet.

### 1. The relayer exists but is not deployed against any foreign chain

A relayer service lives in **`infra/relayer/`** (commit `8dd98aa`, service `relayer` in
`docker-compose.ceiba.yml`). It polls `Interoperability.bridgeTransactions`, persists cursor
state in Postgres, exposes Prometheus metrics, and in `confirm` mode submits
`confirm_external_delivery`.

Two adapters ship:

| Surface | `null` (default) | `evm` |
|---|---|---|
| `RELAYER_ADAPTER` | default; the safe choice with no counterparty | opt-in |
| `supports()` | `false` for every chain | only the `EVM_CHAIN_INDICES` it is given |
| `hasObserved()` | `false`, always | `true` only on a verified `MessageDelivered` log |
| Deployed? | n/a | **No** — needs a receiver contract on a foreign chain |

The adapter contract is deliberately one method — *can you prove this payload was observed on
the target chain?* — and `null` answers no. That is the correct default when no foreign chain
holds the payload.

The `evm` adapter verifies a receiver contract's
`MessageDelivered(uint256 indexed messageId, bytes payload)` logs, requiring all four of: the
configured receiver address, the message id in the topic, a payload whose blake2b-256 matches the
on-chain `messageHash`, and `EVM_CONFIRMATIONS` of depth with the block re-read to confirm it is
still canonical. Delivery is created by `submitDelivery()`, which builds the call, prices it,
signs an EIP-1559 transaction locally and broadcasts it.

`infra/relayer/contracts/BelizeBridgeReceiver.sol` is the counterparty. It requires
**M-of-N threshold signatures** over a digest binding chain id, contract address, message id and
payload hash — so one compromised key cannot inject a message, signatures cannot be replayed onto
a fork or another message, and duplicate or non-canonical (high-s) signatures are rejected. It
compiles clean with solc 0.8.38 (`npm run compile:contracts`).

Its trust model must be stated plainly: there is no light client, because `pallet_beefy` and
`pallet_beefy_mmr` are absent from `construct_runtime!` and BEEFY is what makes Substrate finality
verifiable on an EVM chain. So delivery is **attested by `threshold` validators** — stronger than
a single deliverer, still weaker than ML-DSA-87 verification, and **not trustless**.

Because there is no light client, the bridge has **two validator registries that cannot see each
other**: `BridgeValidators` on this chain (keyed by account, holding ML-DSA-87 public keys) and
`isValidator` in the receiver contract (keyed by address, holding `secp256k1` keys). Neither key
derives from the other. `set_evm_address` (call index 13, map `EvmAddressOf`) records the
declaration that links them, and `infra/relayer/scripts/check-validator-sets.mjs` compares the two
sets and exits non-zero on drift. **That makes drift visible, not impossible** — a contract
validator with no chain-side account is an address that can help meet a quorum nobody here answers
to, and only the reconciliation check will say so.

Inbound is still unstaffed and separately so: nothing outside
`pallets/interoperability/src/tests.rs` calls `process_unlock` or `submit_incoming_unlock`, and
the relayer has no inbound path at all. Inbound value cannot arrive until something watches a
foreign chain and submits unlocks. Note also that the relayer only *observes* foreign delivery;
it does not send the foreign-chain transaction, so whoever calls `deliverMessage` is still an
open operational decision.

### 2. Messages are deliverable, but nothing attests delivery in practice

`send_cross_chain_message` queues a `BridgeTransaction` with `MessagePassing`; `initiate_bridge`
queues one with `LockAndMint`. Both reach `ReadyForExecution` only after the configured number of
ML-DSA-87 validator signatures, and `confirm_external_delivery` (call index 12) is the only writer
of `BridgeTransaction.external_confirmation` — the field that ties our record to something that
happened on the far side. It was declared but never written before, so `Finalized` meant nothing
beyond our own bookkeeping. Only a registered bridge validator holding KYC Level 3 that is not
sanctioned may record it.

The missing piece is therefore **the observation and the foreign call**, not the accounting. Both
are now code-complete but not deployed: nothing has called `deliverMessage` or `deliverValue` on
any chain, so no log exists for the adapter to find, so on a live chain `external_confirmation`
stays empty. Calling it a transport today would still be false: what exists is a signed,
KYC-gated, fee-charged record of intent, a gated way to close it, and an adapter that can verify
closure once a counterparty is deployed.

---

## Status summary

| Capability | State |
|---|---|
| XCM to Polkadot / Kusama / Moonbeam | **Not built.** Requires a parachain. |
| Bridge state machine (lock/mint, burn/unlock) | **Built**, runtime index 27 |
| Post-quantum bridge signatures | **Built** (ML-DSA-87) |
| Liquidity pools, escrow, disputes | **Built** |
| Outbound message recording | **Built** |
| Outbound delivery ack (call index 12) | **Built**, gated to KYC-3 bridge validators |
| Foreign-chain observation | **Built, undeployed** — `evm` adapter verifies receiver logs; `null` is the default |
| Foreign-chain submission | **Built, undeployed** — EIP-1559 signer + `deliverMessage` encoder |
| Inbound unlock submission | **Built but unstaffed** — no inbound relayer path |
| Bridge worker in `infra/` | **Exists** — `relayer`, observe-only by default, `null` or `evm` adapter |
| Deployed counterparty on any foreign chain | **Does not exist** — M-of-N receiver compiles, never deployed |
| Trustless (light-client) verification | **Blocked** — needs `pallet-beefy` + `pallet-beefy-mmr` in the runtime |

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

The ack extrinsic this page used to ask for now exists, as does an M-of-N receiver, an adapter that
verifies its logs, and a submitter that creates delivery. Option 1 was chosen: an EVM chain, a
deployed receiver contract, and `eth_getLogs` observation. What remains is operational, in order:

1. Choose the validator set and threshold, and deploy `BelizeBridgeReceiver` to a testnet.
   Compiling is verified; deploying creates a resource on a foreign network and needs a decision.
2. Record the address in `EVM_RECEIVER_ADDRESS` and the set in the relayer's config. Nothing
   reconciles the contract's validator registry with `register_bridge_validator` yet — that gap is
   a real one, not a note.
3. Register the observing account via `register_bridge_validator` (KYC Level 3) and provision
   `RELAYER_SEED`.
4. Only then flip `RELAYER_MODE` to `acknowledge`.

For actual trustlessness the blocker is BEEFY. `pallet_beefy` + `pallet_beefy_mmr` provide the
commitment an EVM contract can verify; without them no light client can be written, and every
bridge design is a federation with a different name. That is a runtime change plus a Solidity
verifier, not a patch.

Full breakdown in **[RELAYER_SCOPE.md](RELAYER_SCOPE.md)**.
