# BEEFY Re-Genesis — Pinned

**Status: DEFERRED by owner decision (2026-10-25).** Not abandoned, not blocked on
investigation — the analysis is complete and the plan is settled. It is waiting because it is a
large build and the time is better spent elsewhere first.

**Do not attempt this as a hot runtime upgrade.** The reason is below and it is a
block-production hazard, not a preference.

---

## Why it cannot be a hot upgrade

`pallet_beefy` has **no extrinsic that sets the authority set.** Verified in
`polkadot-sdk` rev `7a8daa68e…`, `substrate/frame/beefy/src/lib.rs`:

| Call index | Extrinsic |
|---|---|
| 0 | `report_double_voting` |
| 1 | `report_double_voting_unsigned` |
| 2 | `set_new_genesis` (root; sets a block number, **not** authorities) |
| 3 | `report_fork_voting` |
| 4 | `report_fork_voting_unsigned` |

The only writers of `Authorities` are:

- `fn initialize(&Vec<T::BeefyId>)` — **private**, and returns `Err(())` when `Authorities` is
  already non-empty, so it cannot run on a live chain.
- `SessionManager::on_new_session` — i.e. authorities come **only** from session keys.

So enabling BEEFY requires `SessionKeys { babe, grandpa }` → `{ babe, grandpa, beefy }`.

`SessionKeys` is a fixed SCALE tuple persisted by `pallet_session` in `NextKeys`, `QueuedKeys`
and `KeyOwner`. Appending a field makes every stored value undecodable, and **no migration can
repair it** — the new field is an off-chain ECDSA key that simply is not on chain.

**Consequence:** shipping it as `system.set_code` breaks session-key decoding at the next epoch
boundary, which stops block import permanently. That is exactly how Chain C died on 2026-10-04.

From genesis the keys exist and nothing needs migrating, which is why re-genesis is the correct
route rather than a workaround.

---

## What it unblocks

The bridge is currently **permissioned**: delivery is attested by `threshold` validators
(`BridgeValidators` holds ML-DSA-87 keys; `BelizeBridgeReceiver.sol` holds `secp256k1`
addresses). Neither chain can read the other's set, so they can drift — `set_evm_address` plus
`infra/relayer/scripts/check-validator-sets.mjs` make drift *visible*, not impossible.

BEEFY is the mechanism that makes Substrate finality verifiable on a foreign chain. With
`pallet_beefy` + `pallet_beefy_mmr`, the EVM receiver could verify an MMR proof of our
finalized header instead of trusting a validator list — that is the difference between
"attested" and "trustless".

---

## What a re-genesis destroys

Everything on Chain D. Confirmed by query on 2026-10-25:

| Item | State |
|---|---|
| DALLA / bBZD balances | Live, non-zero |
| BNS `.bz` names | Live |
| LandLedger records | Live |
| Identity / KYC records | Live |
| GEM contracts | Were deployed 2026-10-08; **would need redeployment** |
| `ChainConfigurations` | 13 chains, seeded by migration 121 |
| `BridgeValidators` / `BridgeTransactions` | 3 / 2 rows |

Capture before executing:

```bash
R=http://100.119.97.38:9944
# runtime blob (rollback)
python3 scripts/upgrade-runtime.py --help   # see contract note in ECOSYSTEM_STATE §2.1
# storage dump per pallet, then re-seed via genesis patch or extrinsics
```

---

## The settled plan (execute in this order)

1. **Add `beefy` to `SessionKeys`** in `runtime/src/lib.rs`, plus `pallet_beefy` and
   `pallet_beefy_mmr` to `construct_runtime!`. `BeefyId` must be ECDSA (`sp_consensus_beefy`)
   so `BelizeBridgeReceiver.sol` can recover the signer.
2. **Wire `sc-consensus-beefy` into the node** service (`node/src/service.rs`, 428 lines) —
   the node currently starts BABE + GRANDPA only.
3. **`pallet_beefy_mmr` leaf conversion**: `BeefyAuthorityToMerkleLeaf` must emit **uncompressed**
   65-byte keys so the EVM side derives addresses the same way the contract does.
4. **Regenerate the chain spec** and re-genesis. Genesis runtime is currently **107**, so a new
   spec boots at 107 and upgrades forward to 121+.
5. **Re-seed**: chains config, validators, balances, BNS, LandLedger, and redeploy GEM.
6. **Then** replace the attested trust model with MMR verification in the receiver contract.

Step 6 is the only step that changes the bridge's security claim. Steps 1–5 are infrastructure.

---

## Why not XCM instead

XCM was removed deliberately (see `ECOSYSTEM_STATE.md` §4) but is **not** permanently ruled out —
the workspace dependencies can be restored if a genuine need appears. It is not the right tool
here: XCM requires a cumulus collator and a parachain slot, which contradicts the single-node
sovereign model, and it does not solve EVM finality verification. BEEFY does.

---

## Cost estimate

| Step | Effort | Risk |
|---|---|---|
| Runtime: session keys + 2 pallets | Medium | Low |
| Node: BEEFY consensus service | Medium | Medium — consensus wiring |
| Chain spec + re-genesis + re-seed | Medium | **High** — irreversible |
| Contract: MMR proof verifier | Large | Medium |
| Re-verify everything end to end | Medium | Low |
