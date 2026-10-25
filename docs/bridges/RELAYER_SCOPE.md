# Bridge Relayer — Scope

**Status: BUILT, RUNNING IN `observe` MODE.** The relayer lives in `infra/relayer/` and is
defined (commented out) in `infra/docker-compose.ceiba.yml`. It polls
`Interoperability.bridgeTransactions`, asks a foreign-chain adapter whether each deliverable
transaction has landed, and — in `confirm` mode — submits
`interoperability.confirmExternalDelivery`.

**Verified 2026-10-25** against `pallets/interoperability/src/lib.rs`,
`infra/relayer/`, `infra/relayer/contracts/`, and `infra/docker-compose.ceiba.yml`.

> This file previously said "NOT BUILT. No relayer exists in any repository" and described
> `CrossChainMessage` / `CrossChainMessages` / `acknowledge_message`. All three were removed
> in runtime spec 121 when the pallet's duplicate message path was collapsed into the
> `BridgeTransaction` path it should have used from the start. A scope document that
> describes a service as absent while it runs in the same repository is worse than no
> document, so the obsolete text is gone rather than annotated.

---

## The two gaps, and where they stand

### Gap 1 — outbound delivery: closed on chain, open off chain

`send_cross_chain_message` (call index 4) queues a `BridgeTransaction` with
`BridgeOperation::MessagePassing`, sharing `NextTxId`, `BridgeStatus` and `BridgeValidators`
with value transfers. `confirm_external_delivery` (call index 12) is the **only** writer of
`BridgeTransaction.external_confirmation`, and it requires the status to already be
`ReadyForExecution` or `Finalized` — that is, the threshold of ML-DSA-87 signatures must have
been met first.

What remains is the counterparty: `BelizeBridgeReceiver.sol` exists and compiles, but is not
deployed on any foreign chain. Until it is, `observe` mode is the only honest configuration
and the pending backlog grows by design.

### Gap 2 — the trust model is a permissioned set, not a light client

`BridgeValidators` is an allowlist of ML-DSA-87 keys. The EVM receiver is an allowlist of
`secp256k1` addresses. **Neither chain can read the other's set**, so the two are independent
registries that can drift. `infra/relayer/scripts/check-validator-sets.mjs` compares them and
exits non-zero on drift; nothing enforces agreement.

Closing this properly needs `pallet_beefy` + `pallet_beefy_mmr` in the runtime, which is how
Polkadot proves finality to a foreign chain. Neither pallet is present in `construct_runtime!`.
That is the hard blocker, and it is a runtime change, not a relayer change.

---

## Prerequisites before the relayer may leave `observe`

| # | Item | Where | State |
|---|---|---|---|
| 1 | `confirmExternalDelivery` on the live chain | runtime upgrade to spec 121 | Built, not deployed |
| 2 | `BelizeBridgeReceiver` deployed on the target chain | external | **Not done** — nothing to bridge to |
| 3 | Validators registered with both a PQ key and an EVM address | `register_bridge_validator` + `set_evm_address` | Mechanisms exist; nobody registered |
| 4 | Validator sets reconciled | `scripts/check-validator-sets.mjs` → `ALIGNED` | Tool built; cannot pass until 2 and 3 |
| 5 | Trust model documented as-is, not as trustless | this file | Done |

Item 2 is the real blocker. **A relayer with no counterparty contract polls and does nothing**,
which is exactly what the current `null` adapter configuration is for.

---

## The service as built

```
infra/relayer/
  src/main.ts                 ← connect, migrate, poll loop, metrics server
  src/relayer.ts              ← one pass: read → filter → observe → confirm
  src/config.ts               ← fail-closed env parsing
  src/state.ts                ← postgres: processed ids, idempotency
  src/adapters/types.ts       ← ChainAdapter interface
  src/adapters/null.ts        ← observes nothing, authorises nothing
  src/adapters/evm.ts         ← eth_getLogs over the receiver's events
  src/submit/                 ← RLP, ABI, EIP-1559 signing, delivery submission
  contracts/                  ← BelizeBridgeReceiver.sol, BelizeWrappedToken.sol
  scripts/compile-contracts.mjs
  scripts/check-validator-sets.mjs
```

| Concern | Shape |
|---|---|
| Runtime | Node 24 + TypeScript (ESM, `--experimental-strip-types`), no build step needed to run |
| Compose | `relayer` service in `docker-compose.ceiba.yml`, internal only, no published port |
| Chain access | `ws://ceiba-node:9944` (in-network) |
| Signing | SS58 seed from `/opt/belizechain/.env`, never baked into an image |
| Foreign signing | `secp256k1` via `@noble/curves`; the signed bytes are re-decoded and the signer recovered in tests |
| Idempotency | `relayer.message_acks` in postgres — a restart does not re-submit |
| Failure mode | Refuses to confirm unless the adapter names the exact foreign transaction hash |
| Monitoring | `belizechain_relayer_{pending_messages,confirmed_total,errors_total,...}` |

Non-negotiables, given the pallet's design:

- The relayer is a **permissioned bridge validator**, not an anonymous participant. It must pass
  KYC L3 and be registered before it can submit anything.
- Every bridge transaction carries ML-DSA-87 signatures (`provide_pq_signature`) before it is
  deliverable. Key handling is the whole security model — compromise equals escrow drain.
- `dispute_bridge_transaction` exists, so a wrong submission is contestable, not final. That is
  a safety net, not a licence to be sloppy.

---

## Effort, honestly

| Piece | Size | Risk | State |
|---|---|---|---|
| `confirm_external_delivery` extrinsic + benchmark + weights + tests | Small | Low | **Done** (spec 121) |
| `set_evm_address` registry + reconciliation script | Small | Low | **Done** (spec 121) |
| Relayer service + EVM adapter + contract | Medium–Large | Medium | **Done, undeployed** |
| Deploying the receiver to a foreign testnet | Medium | Medium | **Not started** |
| Key management for ML-DSA-87 signers | Medium | **High** — this is the money | Not started |
| Light-client verification (BEEFY) | Large — runtime feature, not a service | **High** | **Blocked**: neither pallet is in the runtime |

**Do not enable `confirm` mode before items 1–4 in the table above hold.** Everything except
the foreign deployment is built; the foreign deployment is the actual project.

---

## Recommendation

1. **Now, cheap:** leave the relayer in `observe` and alert on backlog growth. That is a real
   signal about the bridge's readiness, not noise.
2. **Before any foreign deployment:** decide whether this bridge is permissioned-and-documented
   or trustless-and-BEEFY. Both are legitimate; only the first is reachable today.
3. **When a target chain is chosen:** deploy `BelizeBridgeReceiver` + `BelizeWrappedToken`,
   register the validators on both sides, and require a green
   `check-validator-sets.mjs` before flipping `RELAYER_MODE=confirm`.
