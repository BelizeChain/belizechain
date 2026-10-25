# Groth16 Trusted Setup Ceremony — Pinned

**Status: NOT STARTED. Blocks any real use of `pallet-zk-attestation`.**

The pallet is built, tested and correct. The *claim* it makes is not yet earned, and this
document is the record of exactly which part is missing.

---

## The gap, stated plainly

`pallet-zk-attestation` verifies Groth16 proofs against a verifying key published through
governance. There is no such key on any chain, and there cannot be one until a ceremony
produces it.

`setup_for_testing` in `zk/attestation/src/circuit.rs` generates a complete CRS locally. That
is fine for tests and catastrophic for production. Groth16's security rests on a trapdoor
created during parameter generation: whoever knows it can produce proofs that verify for
statements that are false, and **nothing reveals that they did**. Not the chain, not the
verifier, not a later audit. A forged attestation is indistinguishable from an honest one.

So today, a proof accepted by the pallet would prove *"the submitter knows a witness"* — not
*"the witness is true."* Those are different claims and only the second one is useful.

The function is named `setup_for_testing` because of this, and it is the only setup function in
the crate.

---

## What is fixed and cannot be negotiated

These are measured from the current source, not estimated:

| Property | Value | Where |
|---|---|---|
| Curve | BLS12-381 | `zk/attestation/Cargo.toml`, `ark-bls12-381 0.5.0` |
| Constraint count | 8,870 | `circuit_stays_affordable` test |
| Public inputs | 3 (`set_root`, `threshold`, `nullifier`) | `verify::PUBLIC_INPUT_COUNT` |
| Verifying key | 536 bytes | CLI `setup` output |
| Proof | 192 bytes | CLI `prove` output |
| Verification | 8.0 ms (Ryzen 9 5900X, 2,000 samples) | `pallet-zk-attestation/src/weights.rs` |
| Proving key | 2,924,304 bytes | CLI `setup` output |

A ceremony must produce a **proving key** and **verifying key** for *exactly this circuit*.
Any change to the circuit invalidates both and requires a new ceremony. That is why
`RegistryRoot` carries a `circuit_version` and `VerifyingKeys` is keyed by it — rotation was
designed in, but rotation costs a ceremony.

`SessionKeys`-style undecodable state does not apply here: adding a verifying key is a plain
storage write. So unlike BEEFY, this **can** ship as a hot upgrade once a key exists. The
blocker is cryptographic, not structural.

---

## The uncomfortable question to settle first

This is upstream of the ceremony and it changes whether the ceremony is worth running at all.

Groth16-over-BLS12-381 carries **three** weaknesses, and one ceremony fixes only one:

| Weakness | Fixed by a ceremony? |
|---|---|
| Needs a trusted setup (trapdoor exists) | Yes |
| **Not post-quantum** — rests on discrete log | **No** |
| Submission is linkable to a signing account | No (separate; see pallet docs) |

A *transparent* proof system — a STARK, or a transparent SNARK — has **no setup at all** and is
hash-based, which makes it post-quantum-resistant in the same sense as the Merkle proofs already
in `pallet-storage-proof`.

The cost is real: proof size grows from 192 bytes to tens of kilobytes, and verification from
8 ms to something larger. At 6-second blocks and a national-scale registry, that trade may well
be worth taking — a government registry is a 20-year artefact, and "we will migrate the proof
system later" is the sentence every system that never migrated said.

**This document does not decide that.** It records that the decision exists and must be made
before ceremony effort is spent, because a ceremony for a proof system we then abandon is wasted
work. Recommendation below.

---

## Options

### Option 1 — Multi-party ceremony on BLS12-381 (keep the current circuit)

The textbook-correct answer for the curve we already use. Phase 1 (universal, curve-only) can be
reused from any prior BLS12-381 ceremony; only phase 2 (circuit-specific, 8,870 constraints) is
ours.

Sizes make phase 2 unusually cheap: a small circuit means a small proving key (2.9 MB) and a
short ceremony.

**Blocker to resolve before committing:** there is no turnkey, well-audited MPC ceremony tool for
arkworks 0.5 BLS12-381 circuits. `snarkjs` is BN254-only. Existing BLS12-381 ceremonies (Zcash
Sapling, Celo Plumo) shipped with their own bespoke tooling for their own circuits.

*Not verified locally. Whoever picks this up must confirm current tooling before promising a
timeline — do not take this paragraph as an assessment of any specific project.*

### Option 2 — Migrate the circuit to BN254, use the mature ceremony tooling

BN254 + `snarkjs` + a public phase-1 `ptau` is the best-trodden path in the ecosystem: audited
tooling, many precedents, and phase 1 is free from an existing ceremony.

**Cost:** BN254 targets roughly 100-bit security, below the ~128-bit level of BLS12-381 and below
what a national identity-adjacent system should settle for. It is also still not post-quantum.

### Option 3 — Single-party setup with a public transcript

One operator runs the setup, publishes the transcript and hashes, and governance attests it was
honest. Cheap and fast.

**This is trust, not a ceremony.** It is acceptable only if the attestation is scoped to
low-value predicates where a forged proof is embarrassing rather than damaging, and only if that
scope is written down and enforced — which the pallet cannot currently enforce.

### Option 4 — Move to a transparent proof system

No ceremony, no trapdoor, hash-based and therefore post-quantum-resistant. Replaces `verify.rs`
and `circuit.rs`; the pallet's storage, nullifier logic, governance extrinsics and replay
protection all survive unchanged, because none of them depend on the proof system.

**Cost:** larger proofs, slower verification, and a rewrite of the two files that are most
sensitive to getting right.

---

## Recommendation

**Do Option 4's feasibility check before Option 1's ceremony.** Concretely: time a
transparent-system verifier in the same harness that produced the 8.0 ms number. If verification
stays inside a sane fraction of a 6-second block and proof size stays acceptable, take it — it
removes the ceremony *and* the post-quantum caveat in one change.

If it does not fit, fall back to Option 1 and run a genuine multiparty phase 2.

Option 3 should only be chosen with eyes open, and never for anything a citizen's benefit depends
on.

Do **not** choose Option 2. Giving up security level to reuse tooling is the wrong direction for
this system.

---

## If Option 1 is taken: what a ceremony must satisfy

1. **At least one participant is honest and independent of the others.** This is the entire
   security argument. Three participants run by the same team is a single participant with extra
   steps.
2. **Every participant destroys their toxic waste and the destruction is attestable.** Publish
   hashes at each step; the transcript must be independently reproducible.
3. **The contribution transcript is published and re-verified from scratch** by someone who did
   not participate.
4. **The circuit is frozen before the ceremony starts.** A one-constraint change voids the work.
   Tag the source commit and hash it.
5. **The verifying key that lands on chain is byte-identical to the one the transcript produces.**
   Compare hashes, not trust.
6. **A second, independent party reproduces the VK from the transcript** and confirms the hash
   before `set_verifying_key` is called.

---

## Ship checklist

Nothing here is optional.

- [ ] Proof-system decision recorded (Option 4 feasibility first, then 1 or 3)
- [ ] Circuit source tagged and hashed; constraint count re-measured and matches 8,870
- [ ] Ceremony run, or Option 3 scope written down and accepted by governance
- [ ] Transcript published; independent reproduction of the VK hash
- [ ] VK staged on a testnet first and exercised with a real proof end to end
- [ ] `set_verifying_key(circuit_version, vk)` through `RegistryAuthority`
- [ ] `set_registry_root` for a real set, with a real `leaf_count`
- [ ] One honest proof accepted; one inflated-threshold proof rejected
- [ ] `ECOSYSTEM_STATE.md` records the VK hash, the circuit version and the transcript location
- [ ] Pallet docs updated: remove the "trusted setup outstanding" caveat *only* after this
      checklist is complete

Until every box is ticked, the honest description of `pallet-zk-attestation` is:

> Correct and tested, but not yet able to prove anything true about the world, because no
> verifying key exists that a third party can trust.

---

## What must NOT be claimed in the meantime

Anyone writing docs, a demo, a website page or a funding application should not say any of:

- "privacy-preserving attestations are live" — no key, so nothing can be attested
- "zero-knowledge proofs are post-quantum on BelizeChain" — they are the one proof type that is
  not; ML-DSA-87 on identities and bridge deliveries is unaffected
- "trustless attestations" — today they are trust-*us*
- "citizens can prove income without revealing it" — the mechanism exists; the trust anchor does
  not

The mechanism is real and it works. Saying so precisely is what makes the eventual claim
credible.
