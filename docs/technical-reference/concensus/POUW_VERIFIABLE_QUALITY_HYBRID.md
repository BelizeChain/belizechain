# PoUW Verifiable Quality — Hybrid Design

**Status:** Proposal. Nothing here is implemented. The live chain pays
timeliness (20%), adjudicated honesty (20%) and a stake bonus (5%) and nothing
else.
**Raised:** 2026-10-23
**Supersedes:** the removed quantum and model-quality reward terms
**Related:** `POUW_REWARD_VERIFICATION_GAP.md` (items A–D), `pallets/staking`

---

## Why the terms were removed

Both removed terms scored data the caller supplied in the same call:

- **quality** scored size and Shannon entropy of `encrypted_delta`. A validator
  writes its own payload, so it chose its own score. Worse, the entropy penalty
  fires above 7.5 bits/byte — which is exactly what encrypted data looks like —
  so a real encrypted delta scored ~55 while the table assumed ~80.
- **quantum** took `accuracy_score` as a call argument and derived job count and
  `num_qubits × depth` from the same payload. A validator asserted its own
  reward outright.

The fix was to stop paying for them, not to delete the capability. Quality
survives as `MIN_QUALITY_TO_ACCEPT = 40` in `submit_model_delta`: a payload that
fails it is rejected, not paid less.

## What the chain can actually observe

`submit_model_delta(task_id, encrypted_delta, computation_commitment, computation_log)`

| Field | Bound? |
|---|---|
| `encrypted_delta` (≤1024 B) | `computation_commitment = H(delta ‖ who ‖ block)` is enforced, so the payload is bound to submitter and block |
| `computation_commitment` | verified |
| `computation_log: [u8;32]` | only checked non-zero — **nothing verifies it** |

That unused `computation_log` is the natural anchor for a verifiable signal.

## The hybrid

No single mechanism is both privacy-preserving and cheap, so pay the quality
term only when at least one verifiable signal exists, and fall back to the gate
otherwise.

### Signal 1 — Cross-validator agreement (default, no privacy break)

When ≥3 validators submit for the same `task_id`, score each delta by its
agreement with the round consensus: correlation against the median delta, or
agreement of the reported metric. Pay the quality term only for deltas within a
tolerance band, and slash or exclude outliers.

- **Cost:** none on-chain beyond a median computation, and it never sees
  plaintext.
- **Weakness:** honest-but-wrong majorities win, and it is **vacuous today** —
  the chain has one authority. It only becomes meaningful when the validator
  set grows.
- **Privacy:** preserved.

### Signal 2 — Held-out evaluation with commit–reveal (strong, public models only)

Publish a held-out evaluation set as an on-chain hash when the task is created.
Validators commit `H(delta)` during the round, then reveal. A scorer evaluates
each revealed delta against the public set and writes a signed result.

- **Cost:** a reveal transaction and an evaluation run.
- **Weakness:** revealing the delta breaks delta privacy, so this applies only
  to tasks explicitly marked public. It also needs an evaluation harness the
  runtime trusts.
- **Privacy:** not preserved by construction; opt-in per task.

### Signal 3 — Replayable computation via `computation_log` (probabilistic)

Define `computation_log` as a hash of the training transcript (seeds, batch
order, optimizer state) plus the resulting metric. An auditor replays a random
subset and checks the hash.

- **Cost:** deterministic training, and replay is expensive.
- **Weakness:** any nondeterminism (GPU kernels, dataloader order) makes replay
  fail on honest work. Needs a CPU-deterministic reference path.
- **Privacy:** the transcript hash leaks nothing by itself.

### Signal 4 — Consumer attestation (cheap, trust-anchored)

The party that consumes the model signs "delta D moved metric M to value V".
Governance seats the attester as an issuer, exactly as the Identity pallet seats
document issuers.

- **Cost:** one signature.
- **Weakness:** it trusts the consumer. It is an oracle, not a proof.
- **Privacy:** preserved.

## Recommended shape

1. **Keep the gate as the floor.** `MIN_QUALITY_TO_ACCEPT` stays; it is the only
   check that costs nothing and needs no trust.
2. **Enable Signal 1 as soon as the set reaches 3 validators.** It is the only
   privacy-preserving reward signal that needs no new trust anchor.
3. **Add Signal 2 per-task, opt-in**, for public models where the reveal is
   acceptable. This is where a meaningful quality *ranking* comes from.
4. **Treat Signal 4 as the bridge** until 2 and 3 exist: a seated attester is
   better than a self-asserted score, and it is honest about being an oracle.
5. **Do not restore a quality term that reads its own input.** That is the exact
   defect the 2026-10-12 pass removed.

## Reward weight shape

Restoring the full 85% is not required. A defensible split once Signal 1 or 2 is
live:

| Term | Weight | Source |
|---|---|---|
| timeliness | 20% | submission block vs deadline |
| honesty | 20% | adjudicated offences (`SlashingSpans`) |
| stake bonus | 5% | staked capital vs minimum |
| verified quality | 20–30% | Signal 1, 2 or 4 |
| remainder | — | retained or routed to treasury until a signal exists |

The remainder must stay unassigned rather than paid on a proxy. Paying an
unverified proxy is how the original 40% was lost.

## What this needs before it can ship

- A runtime API or storage layout for per-task scoring results.
- A decision on the median/correlation computation and its weight cost.
- For Signal 2: a public evaluation harness, and a task flag that marks a task
  revealable.
- A runtime upgrade, so it is a versioned change with a rollback blob.
