# Bridge Relayer — Scope

**Status: NOT BUILT.** No relayer exists in any repository. This document scopes what would
have to be built, and what has to change on chain first.

**Verified 2026-10-23** against `pallets/interoperability/src/lib.rs`, `infra/docker-compose.ceiba.yml`,
and a search of every repo in the organisation.

---

## What is missing

Two independent gaps. They are not the same problem and do not ship together.

### Gap 1 — messages can be sent but never delivered

`send_cross_chain_message` (call index 4) writes:

```rust
let message = CrossChainMessage {
    message_id,
    source_chain: BridgeChain::Polkadot,
    target_chain: target_chain.clone(),
    payload: payload_bounded,
    message_hash,
    delivered: false,
    timestamp,
};
CrossChainMessages::<T>::insert(message_id, message);
```

`delivered` is written `false` at line 1406 and **no code path in the pallet ever writes `true`.**
There is no deliver, acknowledge, or fail extrinsic. `CrossChainMessages` is an append-only log
with a boolean that is permanently false, and `NextMessageId` increments forever.

This is a **runtime** gap. No relayer can fix it, because there is nothing to call.

### Gap 2 — inbound unlocks have an extrinsic but no operator

`process_unlock` and `submit_incoming_unlock` both exist, both are benchmarked with real measured
weights, and both are exercised by `pallets/interoperability/src/tests.rs`. Nothing else calls
them. There is no service in `infra/docker-compose.ceiba.yml` and no relayer code in any repo.

This is an **infrastructure** gap. The chain side is complete.

---

## Prerequisites before any relayer work

| # | Item | Where | Why it blocks |
|---|---|---|---|
| 1 | Acknowledge/fail extrinsic writing `delivered = true` | `pallets/interoperability` | Outbound messages are undeliverable by construction |
| 2 | `register_bridge_validator` must be reachable | runtime | Requires **KYC L3** via `Identity::verify_bridge_operator` — who holds that on Ceiba today? |
| 3 | A second chain to relay to | external | Nothing to bridge to yet. Ethereum has no deployed contract; `gem/` has no bridge contract |
| 4 | Decide the trust model | design | `BridgeValidators` is a permissioned set with ML-DSA-87 keys. It is not a light client. State that plainly or build one. |

Item 3 is the real blocker. **A relayer with no counterparty chain is a service that polls and
does nothing.** Items 1 and 2 are cheap; item 3 is a whole other workstream.

---

## What the service would look like, if items 1–4 are settled

```
infra/relayer/                  ← new service, new image
  main.py                       ← poll loop
  chains/<chain>.py             ← one adapter per foreign chain
  submitter.py                  ← signs + submits to BelizeChain
  state.py                      ← last-processed height, idempotency keys
```

| Concern | Shape |
|---|---|
| Runtime | Python, matching `pakit`/`nawal`/`kinich` conventions |
| Compose | New service in `docker-compose.ceiba.yml`, internal only, no published port |
| Chain access | `ws://ceiba-node:9944` (in-network) |
| Signing | ML-DSA-87 key per relayer, from `/opt/belizechain/.env`, never baked into an image |
| Idempotency | Persist last-processed height in postgres — the existing `indexer` DB is the natural home |
| Failure mode | Must **stop**, not retry blindly. A relayer that double-submits an unlock drains escrow |
| Monitoring | Prometheus metrics + an Alertmanager rule for "height gap > N blocks" |

Non-negotiables, given the pallet's design:

- The relayer is a **permissioned bridge validator**, not an anonymous participant. It must pass
  KYC L3 and be registered before it can submit anything.
- Every inbound submission carries an ML-DSA-87 signature (`provide_pq_signature`). Key handling
  is the whole security model — compromise equals escrow drain.
- `dispute_bridge_transaction` exists, so a wrong submission is contestable, not final. That is a
  safety net, not a licence to be sloppy.

---

## Effort, honestly

| Piece | Size | Risk |
|---|---|---|
| `acknowledge_message` extrinsic + benchmark + weights + tests | Small — 1 extrinsic, ~1 day with bench | Low |
| Relayer service skeleton + one chain adapter | Medium — needs the foreign chain's finality rules | Medium |
| Ethereum adapter (contract, events, reorg handling) | **Large** — there is no deployed bridge contract in `gem/` | High |
| Key management for ML-DSA-87 signers | Medium | **High** — this is the money |
| Monitoring, alerting, runbook | Small, but mandatory before it holds value | Low |

**Do not build the relayer before item 3 is answered.** Everything above the Ethereum adapter is
a few days of work. The adapter plus a deployed, audited bridge contract is the actual project.

---

## Recommendation

1. **Now, cheap:** add the acknowledge extrinsic. It makes outbound messages a real transport
   instead of a log, and it is testable without any counterparty chain.
2. **Now, cheap:** answer who on Ceiba holds KYC L3 and would operate a bridge validator.
3. **Later, expensive:** pick the first foreign chain. Ethereum needs a contract written, audited,
   and deployed before a relayer has anything to do.

Until 1–3 are done, `docs/bridges/README.md` remains the accurate statement of capability.
