# PoUW Reward Verification Gap

**Status:** OPEN — finding documented, no fix applied
**Severity:** High for N-validator operation. Currently latent (1 validator, no adversaries).
**Raised:** 2026-09-30
**Related:** `POUW_CONSENSUS_DESIGN.md` (the design this audits), `pallets/staking`, `pallets/storage-proof`

## Summary

Every validator reward in `pallet_belize_staking` is paid against an input that either
verifies nothing, verifies a constant, or verifies only the *shape* of an opaque blob.
There is no reward term that depends on a checkable *result*.

Simultaneously, the one pallet that **does** perform real on-chain verification —
`pallet_storage_proof` — has **no payout path at all**.

The system therefore pays for claims and does not pay for proofs.

## Evidence

`pallets/staking/src/lib.rs:1644` — `calculate_validator_reward`:

| Weight | Input | How it is produced | Verifies |
|---|---|---|---|
| 25% | `quality_score` | `evaluate_model_quality` (`:1552`) | **shape of a blob** |
| 20% | `timeliness_score` | `calculate_timeliness_score` (`:1619`) | **when**, not what |
| 20% | `honesty_score` | **never computed** | **nothing** |
| 30% | `quantum_score` | counters in `record_quantum_contribution` (`:1162`) | **nothing** |
| 5% | stake bonus | stake ratio | **capital**, not work |

**75% of every reward is therefore unverified.**

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

### The one correct verifier is unpaid

`pallets/storage-proof/src/lib.rs` (354 lines) verifies Merkle proofs fully on-chain —
`derive_merkle_root` against the CID commitment — and hard-errors on the `Groth16` path
when the feature is off, explicitly preserving the invariant *"never accept a proof we
could not verify."*

Its **only** currency calls are `reserve` (`:253`) and `unreserve` (`:284`) for a
refundable anti-spam deposit. **There is no transfer, mint, or payout.**

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

Applied:

**A. `honesty_score` (20%) — do first, no new machinery.** Either derive it from
*already-adjudicated* faults (slashing events, revoked or failed proofs) starting at 100
and decaying on proven offence, or delete the term and redistribute the weight. Either
way: stop paying a fifth of every reward for existing.

**B. `quality_score` (25%) — reclassify as a gate, not a score.** The heuristic is not
worthless: it rejects empty, undersized, and random-garbage payloads. That is a legitimate
*anti-spam filter*, and a filter is cheap and reversible — a false accept is caught later,
a false reject is just resubmitted. Keep it as the gate it effectively is; remove it from
the reward calculation. This eliminates a quarter of unearned payout with no new code.

**C. Storage — wire the payout; the verifier already exists.** `pallet_storage_proof`
proves the work correctly and is the only component that got the ordering right:
verifier first, trust second. Add a reward for accepted proofs. **This is the template,
and it is the cheapest real win available.**

**D. `quantum_score` (30%) — verify the solution, not the computation.** For
optimisation-type jobs there is no need to re-run a quantum computer: score the *returned
solution* classically against the objective and constraints. Small circuits are also
classically simulable, so deterministic re-derivation covers a wide class. Sample and
re-execute the remainder. Until a verifier exists, treat this weight as a documented
interim provision that decays rather than a settled entitlement.

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

- **Do not add a "compute reward" term now.** A reward for unverified compute widens the
  surface that is already 75% open.
- **Do not delete the heuristic** — reclassify it as the gate (B).
- **Do not let a model become the payment authority.** Detection yes; adjudication of
  payment no.
- **Do not fix this after onboarding validators.**
- **Do not unpick the `storage-proof` Groth16 guard** — its hard-error behaviour preserves
  a correct invariant and is the model to copy, not to relax.

## Verification commands

```bash
# The reward formula and its weights
sed -n '1644,1700p' pallets/staking/src/lib.rs

# Quality is byte statistics, not model quality
sed -n '1552,1620p' pallets/staking/src/lib.rs

# honesty_score: field, two initialisations, one use — nothing computes it
grep -n "honesty_score" pallets/staking/src/lib.rs

# Slashing needs root; only consensus offences are automatic
grep -n "ensure_root" pallets/staking/src/lib.rs
sed -n '580,590p' runtime/src/lib.rs

# storage-proof: verification present, payout absent
grep -n "reserve\|unreserve\|transfer\|mint" pallets/storage-proof/src/lib.rs
wc -l pallets/storage-proof/src/lib.rs
```
