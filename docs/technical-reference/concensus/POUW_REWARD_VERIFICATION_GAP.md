# PoUW Reward Verification Gap

**Status:** FIXED 2026-10-12 — items **A, B, C and D applied**. No unverified reward
term remains in the PoUW payout path.
**Severity:** High for N-validator operation. Currently latent (1 validator, no adversaries).
**Raised:** 2026-09-30
**Related:** `POUW_CONSENSUS_DESIGN.md` (the design this audits), `pallets/staking`, `pallets/storage-proof`

---

## Fix applied 2026-10-12 (`pallets/staking`)

### A — `honesty_score` is now a function of adjudicated offences  ✅ DONE

It was a constant (95 on join, 50 on force-join, and **no code anywhere ever
updated it**), so 20% of every reward was a flat payment for existing.

It now starts at `INITIAL_HONESTY_SCORE` (100) and loses
`HONESTY_PENALTY_PER_OFFENCE` (20) per proven offence:

- `slash_validator` decays the stored field — the only writer after join.
- `honesty_from_spans` derives it from `SlashingSpans`, which **persists across
  leave/rejoin**. Joining seeds from that same counter, so a validator cannot
  restore their honesty by exiting the set and rejoining.
- Severity is deliberately *not* factored in here: the stake slash already
  carries severity, and honesty answers only "has this account been proven to
  have misbehaved".

Covered by `adjudicated_offence_decays_honesty`, which also asserts the stored
score agrees with the persisted span count.

### B — `quality_score` demoted from a reward term to an anti-spam gate  ✅ DONE

The 25% `quality` term is **removed from `calculate_validator_reward`**. The
heuristic still runs, but now as `MIN_QUALITY_TO_ACCEPT` (40) in
`submit_model_delta`: a payload that fails it is **rejected outright**
(`Error::QualityBelowThreshold`) instead of being paid a reduced amount.

This is the correct shape for this heuristic. It inspects the size and Shannon
entropy of an *encrypted* blob, so it is trustworthy for rejecting empty,
undersized and degenerate payloads, and untrustworthy for ranking real ones.

> **Note found while implementing:** the entropy penalty also fires on
> **>7.5 bits/byte**, which is exactly what *encrypted* data looks like. A real
> encrypted delta therefore scores ~55, not the ~80 the table below assumed for
> "ordinary entropy". The threshold of 40 was chosen against that: it admits
> encrypted deltas (55) while rejecting undersized (30) and degenerate (10)
> payloads. A threshold of 60 would have rejected every genuine submission.

### C — storage proofs are paid from client escrow, not minted  ✅ DONE

This was the one pallet already performing real on-chain verification, and it had
no payout path. It now pays a provider from escrow that a **client** funded.

**Two gaps had to be closed, not one.**

1. **The verification was circular.** `submit_storage_proof(content_id, proof, _)` took
   *both* the root and the Merkle path from the caller and checked only that they agreed
   — proving the caller built a self-consistent tree, nothing more. Any account could pass
   it with arbitrary bytes.
2. **Nothing funded the storage.** With no payer, any reward must be *minted*, and minting
   against a caller-declared root is free money.

**The first fix attempt failed, and the reason matters.** Anchoring the root in LandLedger
cannot work: pakit derives `content_id = derive_merkle_root(BLAKE2(block content), parent
hashes)` and submits it via `storageProof.submit_storage_proof`, while LandLedger commits
`blake2_256(property_id ‖ title_number)`. Those are **disjoint value spaces**, so the anchor
would have rejected *every real pakit proof*. That work was reverted before shipping.

**What shipped instead: a storage commitment registry**, which closes both gaps at once by
removing the mint. A client opens a commitment with their own escrow (`commit_storage`); the
first accepted proof claims the deal's provider slot; each accepted proof releases one tranche
from that escrow. Every balance movement is `reserve` / `repatriate_reserved` / `unreserve`,
so **`total_issuance` is invariant and there is no `deposit_creating` anywhere.**

The consequences are the interesting part:

- **The anchor no longer needs to be unfakeable** — only cheap to check and honest about who
  is paying. Self-dealing moves an actor's own money between its own accounts for a net loss
  of fees, so a self-declared root buys nothing.
- **An incorrectly accepted proof moves the client's own escrow**, never the chain's.
  Verification quality is a client-protection concern, not a consensus-safety one — a much
  more honest requirement than the minted model demanded.
- **pakit needs no change.** `commit_storage` is a *client* action, so the existing
  integration keeps working, and `derive_merkle_root` is byte-for-byte unchanged — the
  parity asserted with `pakit-storage/core/merkle_storage_proofs.py` still holds.
- `ContentAlreadyProven` was **removed**: under a tranche model a duplicate guard would cap
  every deal at a single payment. Spacing (`ProofTooSoon`) bounds payment instead.

**Retention is still not proven, and is not claimed to be.** A Merkle path shows the provider
*could* produce a path at proof time, not that it still holds the bytes. What was added is a
client-initiated challenge (`challenge_provider` → `claim_challenge_failed`) that makes
non-retention detectable and costly for the party whose money is at stake, plus a discretionary
`release_provider` exit. `STORAGE_COMMITMENT_REGISTRY_DESIGN.md` §6 states the limit in full;
§15 records the three defects found during implementation and the invariants that make some
guards unreachable.

> **Superseded content removed.** An earlier revision of this section described a LandLedger
> anchor gate, a minted `ProofReward`, and a `MERKLE_NODE_DOMAIN` separator. All three were
> reverted. The preimage-collision analysis in that revision was *correct* — the untagged fold
> really did collide with LandLedger's 64-byte preimage, confirmed empirically
> (`untagged fold == anchored hash: True`) — but none of it was needed once the mint was
> removed, because there was nothing left to farm.

### D — the `quantum_score` weight is removed rather than faked  ✅ DONE

The 30% `quantum` term was the last unverified weight: `ValidatorQuantumStatsMap`
was read but never reliably populated. It has been **removed from
`calculate_validator_reward`**, taking its reader (`ValidatorQuantumStatsMap`) out
of the reward path with it.

This is the honest option, not the convenient one. There is no way to verify a
quantum-compute claim on-chain today, and inventing a proxy would have recreated
exactly the defect item B removed — a payment for an unverifiable input. The
pallet's own comment records that every surviving term is non-assertable and why,
and explicitly accepts the resulting emission reduction: weights now sum to **45%**
of `MaxValidatorReward`, so total PoUW validator emission is **40% lower** than
before A–D.

Reinstating a quantum term requires a mechanical solution verifier for quantum
jobs. Until then, quantum work is unpaid by this pallet — which is the correct
trade against paying for unverified claims.

### Resulting weights

Staking rewards (`calculate_validator_reward`) — every term now has a real verifier:

| Term | Weight | Verifier |
|---|---|---|
| Timeliness | 20% | objective (submission block vs deadline) |
| Honesty | 20% | **adjudicated offences** (item A) |
| Stake bonus | 5% | objective (capital) |
| ~~Quality~~ | ~~25%~~ | **removed — now a gate** (item B) |
| ~~Quantum~~ | ~~30%~~ | **removed — no verifier exists** (item D) |
| **Total** | **45%** | — |

Separately, `pallet_storage_proof` now pays a provider from **client escrow** rather
than minting, against a mechanically verifiable Merkle path (item C). Because the payer
is a funded commitment rather than the chain, no supply ceiling is involved and
`total_issuance` is unchanged.

**Unverified share of the payout: 0 of 45 points.** Every rewarded point is
backed by a check that cannot be satisfied by declaration alone.

---

## Summary (as originally raised, 2026-09-30)

Every validator reward in `pallet_belize_staking` was paid against an input that either
verified nothing, verified a constant, or verified only the *shape* of an opaque blob.
There was no reward term that depended on a checkable *result*.

Simultaneously, the one pallet that **did** perform real on-chain verification —
`pallet_storage_proof` — had **no payout path at all**.

Both halves are now closed: the unverifiable terms are removed (A, B, D) and the
verifiable one is paid from a funder's escrow instead of a mint (C).

The system therefore pays for claims and does not pay for proofs.

## Evidence

`pallets/staking/src/lib.rs` — `calculate_validator_reward`.

> **This table is the pre-fix state (2026-09-30) and is kept as the record of what
> was wrong.** All four items — A, B, C and D — have since been applied; see
> "Fix applied" above for the current weights (45%, every term verifiable).

| Weight | Input | How it is produced | Verifies |
|---|---|---|---|
| 25% | `quality_score` | `evaluate_model_quality` | **shape of a blob** |
| 20% | `timeliness_score` | `calculate_timeliness_score` | **when**, not what |
| 20% | `honesty_score` | **never computed** | **nothing** |
| 30% | `quantum_score` | counters in `record_quantum_contribution` | **nothing** |
| 5% | stake bonus | stake ratio | **capital**, not work |

**75% of every reward was therefore unverified.**

### `evaluate_model_quality` does not evaluate model quality

It receives the delta **encrypted**, so by construction it cannot judge quality. It
computes only:

- **size** (`:1557-1563`): `0..=31 → 10`, `32..=127 → 30`, `128..=511 → 60`, `_ → 80`
- **Shannon entropy penalty** (`:1603-1614`): `<1.0 bits/byte → −50`, `<3.0 → −30`,
  `>7.5 → −25`, else `0`
- returns `size_score − penalty`, capped at 100

Consequence: **any ≥512-byte payload of ordinary entropy scores 80**, with no ground
truth, no held-out evaluation, and no comparison against a baseline.

### `honesty_score` is a constant

`honesty_score` appears in exactly four places in the pallet: the struct field
(`:200`), initialisation to `95` on join (`:724`), initialisation to `50` on force-join
(`:1264`), and its use in the reward formula (`:1667`). **No extrinsic, hook, or helper
ever updates it.** One fifth of every reward is a flat participation payment.

### Slashing cannot fire for useful-work misconduct

`SlashReason` (`:550`) defines `InvalidProof` and `ModelPoisoning`. Neither can ever be
produced:

- `report_validator_offense` (`:806`) is **`ensure_root`** — governance-only, and
  nothing detects misconduct to prompt it.
- The only automatic path is `BelizeSlashHandler` (`runtime/src/lib.rs:550`), which
  fires on **consensus** offences (equivocation) and always passes
  `SlashReason::ConsensusViolation` (`:585`).

So the enum variants exist; the detection that would use them does not.

### The one correct verifier was unpaid (fixed as item C)

`pallets/storage-proof/src/lib.rs` verifies Merkle proofs fully on-chain —
`derive_merkle_root` against the CID commitment — and hard-errors on the `Groth16` path
when the feature is off, explicitly preserving the invariant *"never accept a proof we
could not verify."*

As originally raised, its **only** currency calls were `reserve`/`unreserve` for a
refundable anti-spam deposit. **There was no transfer, mint, or payout** — while the
verification was also circular, because the root it checked was caller-supplied. Item C
closed both: a client must have funded the content, so the verification is no longer
circular, and each accepted proof pays a tranche from that client's escrow.

## Root cause

Not carelessness — an unimplementable interim.

The design document states the intent plainly
(`docs/technical-reference/concensus/POUW_CONSENSUS_DESIGN.md`):

> Step 3: `generate_zk_proof(local_update, compute_log)`
>
> Computation commitments (blake2_256 hashes) for verification
> **NOTE: True ZK proofs (sp-arkworks) roadmapped for 2028**

and, under Success Metrics:

> **Computation verification**: Commitment validation (structural checks, entropy
> scoring)

So the entropy heuristic was a **documented, deliberate interim** — and the design also
specified `honesty_score: verify_computation_integrity()`, which is exactly the function
that was never written.

**The defect is not the interim. It is that the interim cannot do what the design
assigned to it.** A commitment proves *integrity* (the blob was not altered); it cannot
prove *validity* (the blob is useful). You can commit to garbage. The reward weights were
set as though verification existed, and 75% of the payout inherited a guarantee that was
deferred to 2028.

There is also a privacy constraint that makes this genuinely hard: the delta is
**encrypted** because data must never leave the validator. **The chain cannot verify what
it cannot see.** Any fix must move verification to somewhere that has the plaintext, or
rest on something publicly checkable.

## Framing: the sovereignty stake

The system's legitimacy claim is that contributors work *for the system*, not for any
person or government. That claim holds only while verification is **public and
objective**.

A heuristic score function is not a neutral judge. Whoever maintains the heuristic
decides what counts as useful work — a central authority expressed in Rust rather than in
a boardroom. Unverified proof-of-useful-work is a centralized judgment of value wearing
decentralized clothing.

This is the same reasoning the codebase already applied once: the S6-1 comment in
`runtime/src/lib.rs` records that `quality_score` was **decoupled from BABE weights**
because it is gameable, leaving it "for reward calculation only, where gaming has
economic (not consensus safety) consequences." This finding is the natural continuation of
that decision — the economic surface is now large enough to matter.

## Economic shape

Gaming today is **not expensive** — that is the point. A conforming submission is a
≥512-byte buffer of moderate entropy, which costs essentially nothing to produce. The
honest path costs real compute and pays the same.

**The cheat is cheaper than the honest path, and currently pays identically.** With a
single validator and no competition this is inert. With N validators drawing on a finite
pool, the rational strategy is to fabricate — and the reward formula actively subsidises
it.

## Recommended fix: reward verified results, not effort

**Rule: every rewarded input must have a verifier.** Three acceptable shapes, cheapest
first:

1. **Deterministic re-derivation** — anyone recomputes and compares
2. **Sample / challenge** — random spot-check of a subset
3. **Redundancy** — N parties do the same work and must agree (works anywhere, costs N×)

Applied — see the "Fix applied 2026-10-12" section at the top for what actually landed,
including the two places this recommendation was incomplete.

**A. `honesty_score` (20%) — do first, no new machinery.** ✅ *Applied as recommended.* Either derive it from
*already-adjudicated* faults (slashing events, revoked or failed proofs) starting at 100
and decaying on proven offence, or delete the term and redistribute the weight. Either
way: stop paying a fifth of every reward for existing.

**B. `quality_score` (25%) — reclassify as a gate, not a score.** ✅ *Applied as recommended.* The heuristic is not
worthless: it rejects empty, undersized, and random-garbage payloads. That is a legitimate
*anti-spam filter*, and a filter is cheap and reversible — a false accept is caught later,
a false reject is just resubmitted. Keep it as the gate it effectively is; remove it from
the reward calculation. This eliminates a quarter of unearned payout with no new code.

**C. Storage — wire the payout; the verifier already exists.** ⚠️ *Applied, but this recommendation was incomplete.*
`pallet_storage_proof`
proves the work correctly and is the only component that got the ordering right:
verifier first, trust second. Add a reward for accepted proofs. **This is the template,
and it is the cheapest real win available.**

> **Correction:** "the verifier already exists" was wrong. The Merkle check took both the
> root and the proof from the caller, so it verified self-consistency, not content — and
> paying against it on a *minted* reward would have been free money. The payout needed a
> payer, which is what the storage commitment registry supplies. Ordering *within* the
> pallet was right; the *funding* was what was missing.

**D. `quantum_score` (30%) — verify the solution, not the computation.** ⚠️ *Not applied as recommended — the term was deleted instead.*
For
optimisation-type jobs there is no need to re-run a quantum computer: score the *returned
solution* classically against the objective and constraints. Small circuits are also
classically simulable, so deterministic re-derivation covers a wide class. Sample and
re-execute the remainder.

> **Why deleted rather than verified:** the recommended solution-verifier does not exist
> yet, and `ValidatorQuantumStatsMap` was not reliably populated either — so there was no
> input to check. Building a scoring heuristic now would have reproduced the exact defect
> item B removed. The weight was removed and the 40% emission reduction accepted; a
> quantum term can return once a mechanical verifier for returned solutions exists.

**E. `timeliness` (20%) and stake (5%) — leave alone.** Both are objective and low-risk.
Note that timeliness measures *punctuality*, so it should not carry more weight than that.

**Order: A → B → C → D.**

## Nawal's role: detector, not verifier

A natural proposal is that Nawal — being the AI the system already runs — should judge
submissions and flag bad ones for non-payment. That instinct is **half right, and the
distinction matters.**

The test to apply to any proposed verifier:

> **Can a third party disagree with the verdict and prove it wrong?**

- **Yes** → it is a verifier. Usable to gate payment.
- **No** → it is an oracle. Whoever controls it decides value. Not usable to gate payment.

Nawal as a general judge of "is this work useful" **fails that test**, for five reasons:

1. **It becomes the central authority.** Replacing a gameable formula with a gameable
   model does not decentralize anything — it relocates the authority. This is the exact
   shape the sovereignty mission exists to reject.
2. **It cannot see the work.** The delta is encrypted by design. Giving Nawal plaintext
   breaks `enforce_no_data_transmission()`; giving it keys makes it a breach target and a
   trust root.
3. **It is non-deterministic.** LLM inference is not bit-exact across hardware, drivers,
   and quantisation. A verdict that cannot be reproduced cannot be consensus state — so it
   could only ever be an off-chain signal.
4. **It has an adversarial surface.** A Merkle proof cannot be talked into passing; a model
   can. Making Nawal the gate creates a new and potentially cheaper attack class.
5. **It is a single point of failure.** Nawal down means nobody is paid; Nawal wrong means
   wrong people are paid; Nawal captured means a captured economy.

**Where Nawal genuinely belongs:** as a **detector** that raises *challenges*, never as the
thing that decides payment. Detection is heuristic and probabilistic, which is exactly what
models are good at; verification must be mechanical. A flag opens an investigation, and
non-payment follows only from something checkable.

**There is one legitimate verifier role for Nawal** — and the test shows why. If Nawal
evaluates a model update against a **public, fixed benchmark**, then any third party can
re-run that benchmark and reach the same verdict. The judgment becomes falsifiable, and the
**benchmark becomes the trust root** rather than Nawal. That is sound. It also means the
real artefact to design is the benchmark, not the judge.

**One circularity to avoid:** if the federated task is to improve Nawal itself, then
Nawal judging the updates to Nawal is self-referential — the subject grading its own
examination. Any benchmark-based verification must be run by something independent of the
model being trained.

## Sequencing

**Do this before the validator set grows, not after.**

At one validator there is no adversary and no competition, which is precisely why none of
this has bitten and why it presently looks fine. The failure mode appears only once N
validators share a finite pool, at which point the rational strategy is fabrication.

It is not urgent-for-harm-today. It is **blocking-for-N-validators**.

**Order: make it verifiable → then pay it. Never pay first and hope verification arrives.**

## What not to do

- **Do not add a "compute reward" term now.** A reward for unverified compute is what
  items B and D removed; it would reopen the surface they closed.
- **Do not delete the heuristic** — reclassify it as the gate (B).
- **Do not let a model become the payment authority.** Detection yes; adjudication of
  payment no.
- **Do not fix this after onboarding validators.**
- **Do not unpick the `storage-proof` Groth16 guard** — its hard-error behaviour preserves
  a correct invariant and is the model to copy, not to relax.
- **Do not make a commitment payable on a caller-declared root.** The commitment is the
  anchor: a proof is only paid because somebody funded *that* content id. Wiring payment
  to a self-reported value would restore the free-money exploit that C removed.

## Verification commands

```bash
# Reward formula: which terms survive, and their weights
grep -n "calculate_validator_reward" -A 45 pallets/staking/src/lib.rs

# No unverifiable term is read in the reward path
grep -n "ValidatorQuantumStatsMap" pallets/staking/src/lib.rs

# honesty_score: field, seed, and the single decay writer
grep -n "honesty_score\|HONESTY_PENALTY_PER_OFFENCE" pallets/staking/src/lib.rs

# Slashing needs root; only consensus offences are automatic
grep -n "ensure_root" pallets/staking/src/lib.rs

# storage-proof: commitment gate + escrow payout, Groth16 still hard-errors
grep -n "NoCommitment\|Commitments\|repatriate_reserved\|Groth16NotYetAccepted" \
  pallets/storage-proof/src/lib.rs

# Nothing is minted on this path
grep -n "deposit_creating\|total_issuance" pallets/storage-proof/src/lib.rs

# The registry design, its limitations, and the bugs found building it
sed -n '1,40p' docs/technical-reference/concensus/STORAGE_COMMITMENT_REGISTRY_DESIGN.md

# Tests that pin both behaviour changes
cargo test -p pallet-storage-proof
cargo test -p pallet-belize-staking
```
