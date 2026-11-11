# Mainnet Readiness — verified status of every open security item

**Verified: 2026-11-06** · **Chain under test: Jade (Chain D), runtime spec 124**
**Scope: `belizechain` only.** Sibling repos are tracked in
[`../ECOSYSTEM_STATE.md`](../ECOSYSTEM_STATE.md) §4.1.

---

## 0. What this document is, and what it is not

This is the **single authority** on whether the core chain is ready for mainnet. It
supersedes the checklists in `audit_results/old_audits/COMPREHENSIVE_AUDIT_2026.md`
(dated 2026-03-20) and `docs/security/security-audit-results.md` as a *status* source.
Those documents remain the record of what was found; this one records what is true now.

**It is not a substitute for a third-party audit.** No external audit has been completed.
That is stated as a blocker in §3, not as a caveat, because it is the single largest one.

### The rule this document follows

Every row below was checked **against source**, not against a checklist. Two entries in
the inherited checklist proved to be wrong in opposite directions, which is why:

- **P0-05** was marked `FALSE FINDING — distribute_rewards() does not exist`. It had
  existed since the initial commit. A real churn loop was live in it for seven months.
- **P0-12** was reported as broken by an earlier agent pass that grepped
  `type PqVerifier` and landed on the `#[cfg(feature = "runtime-benchmarks")]` arm.
  The non-benchmark arm wires real ML-DSA-87. **A bare grep of a runtime associated
  type is not evidence — read the surrounding `cfg`.**

A ticked box is therefore not evidence. The *Evidence* column is.

---

## 1. P0 — all 22 verified

| ID | Finding | Verdict | Evidence (read 2026-11-06) |
|---|---|---|---|
| P0-01 | SSN brute-force via on-chain salt | ✅ **FIXED** | `Attestation.salt` is `Option<…>`, documented DEPRECATED, and set to `None` at all 4 construction sites. Hash is `blake2_256(salt ++ plaintext)` with the salt held off-chain. |
| P0-02 | Whistleblower anonymity broken | ✅ **FIXED, with a residual** | `Report` stores `commitment = blake2_256(DOMAIN_TAG ++ account ++ secret)`, not an `AccountId`. **Residual:** `bond_payer` is stored, so a self-funded report is linkable. The relayer/sponsor pattern is the documented mitigation — it must be used, or anonymity is procedural rather than guaranteed. |
| P0-03 | Land registration open to any signed user | ✅ **FIXED** | `register_property` requires `BelizeKyc::is_kyc_verified(…, KycLevel::L1)`. |
| P0-04 | Token decimal confusion (9 vs 12) | ✅ **FIXED** | `MaxDallaSupply = 501_000_000_000 * DOLLARS` with `DOLLARS = 1_000_000_000_000` → 501 billion DALLA at 12 decimals. |
| P0-05 | Dual reward minting path | ✅ **FIXED** (`3151665`) | `distribute_rewards` gates on `EpochDuration` + `LastRewardDistribution`, neither caller-influenced. |
| P0-06 | Uncapped `deposit_creating` | ✅ **FIXED, structurally fragile** | All 5 mint sites cap against `MaxDallaSupply` (community ×2 and whistleblower **reject**; staking ×2 **clamp to headroom**). **See §1.1** — the cap is per-site, not central. |
| P0-07 | `--alice` dev key in production | ⊘ **OBSOLETE** | Described the AKS deployment, removed. No AKS manifests remain. |
| P0-08 | CI `continue-on-error: true` | ⊘ **OBSOLETE** | Same. |
| P0-09 | Node key exposed in CLI args | ⊘ **OBSOLETE** | Same. |
| P0-10 | Mutable `:latest` image tag | ⊘ **OBSOLETE** | Compose pins explicit tags in `/opt/belizechain/.env`. |
| P0-11 | YAML manifest corruption | ⊘ **OBSOLETE** | Same. |
| P0-12 | PQ signatures never verified | ✅ **FIXED** | `RealPqVerifier` / `MLDsaVerifier` via `fips204::ml_dsa_87` in all non-benchmark builds; domain separation `belizechain-consensus-v1` / `belizechain-bridge-v1`; `PassthroughPQVerifier` fails closed. |
| P0-13 | PQ BoundedVec too small | ✅ **FIXED** | `pq_signature: BoundedVec<u8, ConstU32<5000>>`, `pq_public_key: BoundedVec<u8, ConstU32<3000>>` — ML-DSA-87 needs 4,627 / 2,592. |
| P0-14 | Implicit pallet indices | ✅ **FIXED** | 36 explicit indices in `construct_runtime!`. |
| P0-15 | `force_from()` truncates authorities | ✅ **FIXED** | `WeakBoundedVec::try_from` after an explicit `truncate(max_auths)` + `log::error!`, on **both** `Authorities` and `NextAuthorities`. |
| P0-16 | `execute_emergency_proposal` no-op | ✅ **FIXED** | Calls `Self::execute_action(action, proposal_id)?` then `ExecutedProposals::insert`. |
| P0-17 | Referendum quorum hardcoded 1000 | ✅ **FIXED** | `MinQuorumPercentage: Get<u8>`, runtime sets `ConstU8<10>`, validated in `create_referendum`. |
| P0-18 | Emergency override bypasses safeguards | ✅ **FIXED** | `council_override` requires `JaguarMode.active`; the self-declared `is_emergency` flag is explicitly insufficient. |
| P0-19 | Weights hand-estimated | ✅ **FIXED 2026-11-07 — all 20 regenerated** | Every pallet now carries one provenance: `HOSTNAME: ceiba`, `CPU: AMD Ryzen 9 5900X`, `DATE: 2026-11-07`, `WEIGHT MULTIPLIER: 1.25`. `check_weights_freshness.sh` went from 7 stale files to a pass. The hand-written `issue_wusdc` / `burn_wusdc` are now measured (1.25× / 1.58× their old flat 12,000,000 — they were under-charged, as suspected). `landledger`'s two new two-phase calls were measured for the first time and the provisional guesses were **2.3× and 1.9× too low**. See §1.2a for why the plan changed, §1.2b for the measured outcome and its caveats, and §1.2c for the recomputed safety margin. |
| P0-20 | Unbounded `iter_prefix().count()` | ✅ **FIXED** | Production hits are comments documenting replacement by O(1) counters. The only live `iter().count()` calls are in one-shot migrations (`belizex`, `mesh`). |
| P0-21 | Coverage gate on wrong package | ✅ **FIXED** | `.tarpaulin.toml`: `fail-under = 40.0` over 21 packages (20 pallets + node). Measured 56%. |
| P0-22 | Docs claim an unverifiable audit | ⚠️ **WAS STILL LIVE — fixed 2026-11-06** | `docs/technical-reference/whitepaper.md` §14.1 listed "Audit Partners" and `PENETRATION_TESTING_PROGRAM.md` §1 said "our formal security audit (Trail of Bits)". Both asserted an engagement that does not exist. Annotated with dated corrections; the dated documents themselves were not rewritten. |

### 1.1 P0-06 residual — the supply cap is not centralised

`pallet_balances` is configured with **no `MaxIssuance`**. The 501-billion cap is
enforced independently at each of the 5 mint sites. Every site is correct today, so this
is **not a live defect** — but it is a structural weakness worth recording:

> A future `deposit_creating` added without its own check silently breaks the supply cap,
> and nothing fails. The invariant is maintained by convention, not by the type system.

Two options if this is to be closed properly: set `MaxIssuance` on `pallet_balances`, or
route all minting through one checked helper. Neither is done.

### 1.2 P0-19 — attempted, measured, and REVERTED (throttled run)

> **Superseded 2026-11-07 — kept as the record of why the first attempt was discarded.**
> This section describes the *reverted laptop* run. Both `belizex` functions were since
> re-measured on Ceiba **1.25× and 1.58× above** their hand-written values, confirming the
> "genuinely under-charged" verdict below by measurement rather than by inspection.
> Current state: P0-19 is **fixed**, see §1.2b.

`fc8aae7` hand-wrote two weight functions into `pallets/belizex/src/weights.rs` **inside a
file whose header still read `AUTO-GENERATED … DATE: 2026-10-25`** — hand estimates
presented as measurements. Those two values are genuinely under-charged, which is the
unsafe direction: a block can accept more work than it budgets for.

A full regeneration of all 7 stale pallets was run on 2026-11-07 and then **discarded**.
Reason, measured rather than assumed:

| Signal | Value |
|---|---|
| `ADP1/online` | `1` — AC connected, so the script's guard passed |
| `scaling_governor` | **`powersave`**, not `performance` |
| Clock under full load (8 busy cores) | **~2,500 MHz** |
| `cpuinfo_max_freq` | **4,500 MHz** (`Intel Core Ultra 7 155H`) |
| Effect on output | **every extrinsic across 7 unrelated pallets rose ~1.70×** |

A uniform 1.70× across `belizex`, `governance`, `interoperability`, `landledger`, `quantum`,
`staking` and `zk-attestation` is not seven simultaneous code regressions — it is the clock.

**Why this is dangerous as well as wrong:** an inflated weight is the *safe* direction, so
nothing fails. CI stays green, the chain runs, and ~40 % of block capacity silently
disappears. It is only visible by comparing against a previous measurement regime.

**Fixed so it cannot recur:** `scripts/bench_weights.sh` now also refuses a non-
`performance` governor, reporting the measured clock against the ceiling, with an explicit
`ALLOW_NON_PERFORMANCE_GOVERNOR=1` override. Negative-tested: it aborts with exit 1 on this
host as it stands.

### 1.2a Correction, 2026-11-07: the reference host is this laptop, and it now thermal-throttles

Two facts were re-measured rather than inherited, and both change the plan.

1. **There is no 4.6–4.8 GHz reference machine.** All **20** committed weight files carry
   `HOSTNAME: wicked, CPU: Intel(R) Core(TM) Ultra 7 155H` — the development laptop, whose
   `cpuinfo_max_freq` is **4,500 MHz**. An earlier revision of this section claimed the
   reference set was "measured at 4.6–4.8 GHz"; that number cannot be true on a 4.5 GHz-max
   part and has been removed. **Do not quote it.**

2. **The laptop can no longer reproduce its own set.** With AC connected, EPP set to
   `performance`, and only 8 busy cores for ~6 s: package temperature reached **100 °C** and
   the loaded cores settled at **3.3–3.75 GHz**, not 4.5. It is thermally saturated. The same
   measurement run under the previous `EPP=power` regime gave ~2,500 MHz, which is why the
   1.70× inflation looked like a governor problem; the governor is only half of it.

**Consequence — the recovery plan changes.** Regenerating 7 pallets on this laptop would
produce a set measured at an unreproducible clock, and mixing those 7 with the other 13
(taken at a cooler moment) gives **two provenances inside one weight set**. That is worse
than staleness, because a stale file is at least internally consistent.

**Revised plan for P0-19:** run the whole 20-pallet regeneration on the **Ceiba host**
(`AMD Ryzen 9 5900X`, 24 threads, `cpuinfo_max_freq` 4.95 GHz, idle, and `NOPASSWD` sudo so
the governor is settable without an operator step), and regenerate **all 20** rather than
the stale 7, so the entire set shares one recorded provenance. Every file's header already
records `HOSTNAME`/`CPU`, so the new provenance is self-documenting.

**Note the direction of the change.** The expectation going in was that Ceiba at ~4.9 GHz
would time every extrinsic *faster* than the 4.5 GHz-max laptop and therefore produce
**smaller** weights — the *unsafe* direction for a slow validator. **That is not what
happened, and the measured result is in §1.2b.** The prediction is left here rather than
quietly deleted because it was reasonable and it was wrong: FRAME `ref_time` on these
pallets tracks storage I/O and harness overhead far more than core clock, so a 1.4× clock
difference moved the median weight by **nothing**.

### 1.2b Measured outcome of the 2026-11-07 regeneration (20 pallets, 262 functions)

Run on Ceiba with `BENCH_MUL=1.25`, `BENCH_STEPS=50`, `BENCH_REPEAT=20`, governor and EPP
both `performance`, CPU held at 4.91–4.94 GHz against a 4.95 GHz ceiling. All 20 pallets
reported `>>> OK`, no failures, in ~65 minutes.

**The host-speed hypothesis was falsified.** Comparing all 262 functions against the
previous laptop-generated set:

| bucket (by storage ops) | functions | median | mean | min | max |
|---|---|---|---|---|---|
| ≤2 ops (most compute-bound) | 31 | **1.009×** | 1.037× | 0.907× | 1.578× |
| 3–6 ops | 70 | 0.998× | 1.018× | 0.920× | 2.344× |
| ≥7 ops | 75 | 0.997× | 1.055× | 0.764× | 4.436× |

A faster host did **not** systematically shrink the set. The largest *decrease* is
`payroll::execute_payment` at 0.764×, and it is not a clock artifact: 30% of the function
disappeared between generations while its source did not change, which is benchmark
variance on a noisy function, not a speedup.

**Where the weights genuinely moved, the reasons are specific and checkable:**

| function | before | after | why |
|---|---|---|---|
| `landledger::approve_transfer` | 45,000,000 (hand-set) | **87,238,750** | First real measurement. Reads 9 / writes 9. The guess was **1.94× too low.** |
| `landledger::reject_transfer` | 30,000,000 (hand-set) | **70,331,250** | First real measurement. Reads 2 / writes 2. The guess was **2.34× too low.** |
| `landledger::transfer_property` | 173,811,250 (16r/10w) | **134,540,000** (14r/3w) | Expected and correct — P1-09 removed the ownership write. The two-phase flow now costs 221.8M combined, **27.6% more** than the old one-shot 173.8M, so the safer flow is also the more expensive one. |
| `belizex::issue_wusdc` / `burn_wusdc` | 12,000,000 (hand-set, `Measured: 3`) | **14,977,500 / 18,936,250** | Were never benchmarked. Proof size also corrected 2,527 → 3,529. Both were under-charged. |

**Two caveats that are real and are not hidden:**

1. **The benchmark host is the live production host.** All 20 containers were running,
   including the chain node, and `ceiba-pakit` was drawing ~7.5% CPU for part of the run.
   This CLI build reports no per-function standard deviation, so the noise cannot be
   quantified from the output. `BENCH_MUL=1.25` absorbs it and the pallets are short,
   single-threaded runs, but a same-box interference effect **cannot be ruled out** and is
   the most likely explanation for `execute_payment`'s 0.764×.
2. **`Sudo` is still at runtime index 6** (P1-02) and the chain is still pre-mainnet, so
   replacing the whole weight set is a single runtime upgrade rather than a coordinated
   multi-party change. Doing this after mainnet would need a governance-track upgrade.


### 1.2c The hardware-constraint margin, stated as arithmetic (recomputed against the real host)

`BlockWeights` allows **2 s of `ref_time` inside a 6 s block** (`runtime/src/lib.rs:837`,
`WEIGHT_REF_TIME_PER_SECOND × 2`), i.e. a 3× safety factor over the block time.

Against the documented minimum validator spec of **2.0 GHz**
(`docs/validators/VALIDATOR_REQUIREMENTS.md:52`) and the host that actually timed the
current weights — Ceiba, measured at **4,912–4,936 MHz** under load — a validator on the
minimum spec is **2.47× slower** than the host that produced the numbers.

- Ignoring `BENCH_MUL`: 3× ÷ 2.47× = **1.21×** margin.
- Crediting the committed `BENCH_MUL=1.25`: 3× × 1.25 ÷ 2.47× = **1.52×** margin.

The second figure is the true headroom, and it is thin. It is also *arithmetic about
block space*, not a guarantee that any given extrinsic lands: a 2.0 GHz validator can still
fall behind on peak load. Choosing a *slower* benchmark host is not the lever — it inflates
every weight and wastes block capacity. The two levers that actually express "runs on cheap
hardware" are:

- **`BENCH_MUL` above the default** — explicit, recorded in each file header, cheap.
  Currently 1.25; raising it is the cheapest way to widen the margin.
- **A benchmark-score minimum in `VALIDATOR_REQUIREMENTS.md`** — how established chains state
  this, and it makes the requirement checkable instead of aspirational.

---

## 2. P1 — triage after re-checking the source

The inherited list has 30 themes (`P1-01`…`P1-30`, spread across
`docs/security/security-audit-results.md` — 27 of them — and
`audit_results/old_audits/COMPREHENSIVE_AUDIT_2026.md`, which holds all 30). It is a
**stale** list, not a list of 30 live defects.

**Recount, every item checked against source on 2026-11-07 = 19 fixed, 2 open, 5 obsolete,
1 false positive, 3 partial/policy.**

| Item | Status | Note |
|---|---|---|
| **P1-02** | � **OPEN — owner decision: KEEP until mainnet** | `Sudo` at runtime index 6. **Decided 2026-11-07: it stays for now and is removed at the mainnet transition.** It remains a **mainnet blocker** — an unremoved sudo is unilateral upgrade authority — so it must not ship, but it is no longer blocked by missing machinery, and the runtime is already prepared for its removal. Verified 2026-11-07: governance can apply runtime upgrades without Root (`apply_pending_runtime_upgrade`, call 39, gated by `GovernanceCouncilSuperMajority` + dual-house hash ratification + on-chain hash check, proven by `non_root_apply_requires_both_house_ratifications`), and the four `EnsureRoot`-only origins that would have died with it were re-pointed to council supermajority aliases. **Why keep it:** deleting it also makes raw `System::set_code` / `set_storage` / `kill_prefix` unreachable, which costs testnet its break-glass recovery — and multi-validator bring-up is exactly the phase where a break-glass is most likely to be needed. Revisit when the validator set is seated and observed. |
| **P1-09** | ✅ **FIXED 2026-11-07** | Was a live defect: `transfer_property` rewrote `Properties::owner` in the same call and recorded `government_approved: false` afterwards, while `approve_transfer` **did not exist**, so the flag could never become `true` and `Error::TransferNotApproved` was unreachable. Now a two-call flow — `transfer_property` (call_index 1) takes the transfer tax and writes a `PendingTransfers` entry, changing nothing else; `approve_transfer` (10, `GovernmentOrigin`) re-checks ownership, verification, survey status and active encumbrances before moving title and appending the anchor; `reject_transfer` (11) refunds the tax. Regression test `transfer_property_does_not_move_title_until_approved` was **measured to fail on the pre-fix code** before being accepted. The provisional hand-set weights are gone: both calls were benchmarked on 2026-11-07 and came out **1.94× / 2.34× above** the guesses (87,238,750 and 70,331,250 ps), i.e. the guesses had been under-charging. |
| **P1-08** | ✅ **FIXED — both halves** | Earlier notes (including this project's own memory) said the matching half was open. **It is not.** The function is `match_order`, singular, called from `place_limit_order`; `match_orders` is only the `WeightInfo` signature, which is why grepping the plural looks like dead code. `plan_fill` / `apply_fill` / `settle_fill` / escrow account / `PRICE_SCALE` all present. |
| **P1-01** | ✅ **FIXED** | Bounds are now the exact FIPS 204 sizes: `pq_public_key: BoundedVec<u8, ConstU32<2592>>` = `ml_dsa_87::PK_LEN`, and `ConstU32<4627>` = `SIG_LEN`. |
| **P1-03** | ✅ **FIXED** | `limit_req zone=rpc/api` is applied in **7 locations** — but in `infra/nginx/snippets/stack-paths.conf`, *not* `nginx.conf`. Grepping `nginx.conf` alone makes this look unfixed; it is not. No wildcard CORS exists, which is correct here because both frontends derive their API origin from `window.location.origin` (same-origin). |
| **P1-05** | ✅ **FIXED** | `PassthroughPQVerifier` is `#[cfg(any(test, feature = "runtime-benchmarks"))]` on both the struct and its impl; `MLDsaVerifier` is the production one. |
| **P1-06** | ✅ **FIXED** | `verify_merchant` gates on `T::MerchantVerificationOrigin::ensure_origin`, not a single signed operator. |
| **P1-07** | ✅ **FIXED** | Real `sp_io::crypto::sr25519_verify` plus sender-key registration; the zero-hash check is gone. |
| **P1-10** | ✅ **FIXED** | Submissions are aggregated by **median** and rejected beyond `MaxPriceDeviation` from it (lib.rs:596). No TWAP, but median + deviation is a sound robust aggregator. |
| **P1-12** | ✅ **FIXED** | 0 hardcoded numeric `#[pallet::weight(N)]` annotations remain in any pallet. |
| **P1-13** | ✅ **FIXED** | `pallets/economy/src/weights.rs` exists (12.9 KB) and `type WeightInfo` is wired. |
| **P1-14** | ✅ **FIXED** | `calculate_voting_power` returns 1 + one per active delegation, and `place_vote` folds that bonus into `base_weight` (lib.rs:3683-3686). |
| **P1-15** | ✅ **FIXED** | `forfeit_deposit_to_treasury` uses `repatriate_reserved` (a real transfer), not `slash_reserved` (which burns), and logs any shortfall. |
| **P1-16** | ✅ **FIXED** | Both `ValidatorSlashed` sites report the amount actually removed (`actual_slashed` / the escrowed value), not the requested amount. |
| **P1-17** | ✅ **Not an exploit** | Self-trade is *skipped*, not filled, so a self-crossing order rests. `SelfTradeNotAllowed` is dead by design and annotated as such. |
| **P1-18** | ✅ **FIXED** | `TradingPair { base_asset, quote_asset, base_reserve, quote_reserve }` keyed by `(u8, u8)` — a real two-asset AMM, not a single-currency pool. |
| **P1-25** | ✅ **FIXED** | justice, moderation and whistleblower all have `src/benchmarking.rs`. |
| **P1-26** | ✅ **FIXED** | 170 `#[test]` in `pallets/governance/src/tests.rs`. |
| **P1-27** | ✅ **FIXED** | The CI step runs `scripts/bench_smoke.sh`, which covers all Belize pallets, not just `pallet_balances`. |
| **P1-28** | ✅ **FIXED** | `tests/cross_pallet/test_all_interactions.py` exists alongside `tests/e2e`, `tests/blockchain`, `tests/economic`. |
| **P1-30** | ✅ **FIXED** | The unsupported APY claims were removed; the doc now derives ~16% at full PoUW weight. |
| **P1-24** | 🟡 **Mitigated** | The forked 4-node swarm is stopped; `--force-authoring` is gated off by default. |
| **P1-19** | 🟡 **Partial — residual accepted** | The original complaint (major tags like `@v4`) is idiomatic and Dependabot-managed, so that half is a false positive. **Residual: 6 branch refs** — `dtolnay/rust-toolchain@stable` ×2 (its documented pattern, `toolchain:` input is pinned to 1.90.0) and `pypa/gh-action-pypi-publish@release/v1` ×4 (a mutable release branch). Not a blocker; pinning to SHAs is the hardening step. |
| **P1-29** | 🟡 **Policy, not a patch** | One identity per account is already enforced (`register_identity` requires `!IdentityOf::contains_key`). The vector is many funded accounts; a `MaxIdentities` cap is a governance choice. |
| P1-04, P1-20, P1-21, P1-22, P1-23 | ⊘ Obsolete | AKS-era: load-balancer exposure, ACR admin creds, K8s `securityContext`, `emptyDir` chain data, AKS `NetworkPolicy`. The whole deployment moved to Docker Compose on Ceiba. |
| P1-11 | ⊘ False positive | `LongestChain` select-chain is the standard BABE+GRANDPA pattern: finality is GRANDPA's job, not the chain-selection rule's. Changing it to a finality-tracking rule would stall block production after a GRANDPA lag. |

**The "20/30 closed" figure in `security-audit-results.md` is a 2026-10-23 snapshot and was
never recounted.** It is close, but it counted **P1-09 as fixed**, which it is not.

### 2.1 Why P1-09 survived three audits

It was reported as CRITICAL (C-1) in `audit_results/old_audits/PHASE_2_PALLET_AUDIT_SUMMARY_2026.md`,
described in full in `PHASE_2B_DEEP_AUDIT_6_PALLETS_2026.md`, and restated with the exact
two-phase fix in `PHASE_13_ADVERSARIAL_SCENARIOS_2026.md`. Nothing was ever changed. The
reason is visible in the folder name: `old_audits/` reads as superseded, so later sweeps
treated its findings as historical. **A finding is not closed by being filed under "old".**

---

## 3. Mainnet blockers — decision-bound, not code-bound

These are what actually stand between the current state and mainnet. None is resolved by
writing more tests.

| # | Blocker | State | Why it blocks |
|---|---|---|---|
| 1 | **No external professional audit** | none engaged | The entire P0/P1 remediation above is **self-assessed**. An independent review of consensus, economics, and the PQ path is the standard bar for a sovereign L1 holding citizen land titles. |
| 2 | **No penetration test** | program defined, never run | `PENETRATION_TESTING_PROGRAM.md` is a requirements document. Nothing in it has been executed. |
| 3 | **No economic modelling** | none | Tokenomics, staking yield ceilings, and DEX incentive design have never been simulated or reviewed. P1-30 was a *documentation* fix (removing a fabricated APY), not a model. |
| 4 | **No formal verification** | none | The whitepaper's §14.1 lists it as a phase. Nothing is verified. |
| 5 | **`Sudo` still mounted** (P1-02) | open by decision — **no longer blocked technically** | Prior wording said removing Sudo "also removes the upgrade path used for every hot upgrade so far". **That was wrong**, and it is corrected here. `Governance::apply_pending_runtime_upgrade` (call_index 39) applies upgrades with **no Root origin**: it needs `RuntimeUpgradeOrigin` (= `GovernanceCouncilSuperMajority`), both houses to ratify the exact hash, and an on-chain `blake2_256(&code)` match before calling `set_code`. That path is proven by `non_root_apply_requires_both_house_ratifications`, which applies an upgrade from a **signed** origin and asserts the pending state is cleared. It is strictly stronger than sudo: hash-pinned, two-house, auditable. The three `EnsureRoot`-only origins that *would* have become permanently dead (`pallet_collective::SetMembersOrigin` for both houses, `storage_proof::RevocationOrigin`, `zk_attestation::RegistryAuthority`) were re-pointed to council supermajority aliases on 2026-11-07 — additive changes, since every alias keeps its `EnsureRoot` branch. What remains is a **policy** choice, not an engineering one: deleting Sudo also makes raw `System::set_code` / `set_storage` / `kill_prefix` unreachable, so testnet loses its break-glass recovery. |
| 6 | **Single authority, 0 peers** | 🟡 **partially retired 2026-11-07** — a 4-authority set is now demonstrated, but only on one host | `shouldHavePeers: false` is correct for testnet and disqualifying for mainnet. **What is now proven** (`scripts/local-swarm.sh`, 4 native nodes on one box): four independently-generated BABE+GRANDPA authorities peer with each other, agree on every block hash (0 mismatches over heights 0–60), finalize continuously, and survive a **BABE epoch rotation** — the exact event that killed Chain C on 2026-10-04 — with the authority set still populated. Rotation was observed **twice**: at block 253 on the manual bring-up, and again at block 301 (`epoch_index` 0 → 1, `BabeEpochDuration` 300) on a clean `up` from an empty work dir, i.e. the scripted one-shot path with no manual intervention. Both runs reported 4 authorities at weight 1 in the new epoch and continued finalizing afterwards. **What is NOT proven:** those four nodes share a host, a clock, and a loopback network, so this exercises consensus logic, not real network partition behaviour, clock skew, or geographic latency. Seating four *independent operators* on separate hardware remains open. |
| 7 | **Bridge far side exists, is deployed and executing — but only against a local EVM** | 🟡 **re-scoped 2026-11-07, then largely retired the same day** — was "does not exist" | `confirm_external_delivery` (index 12) is the only writer of `external_confirmation` and is correct. The earlier wording — "no `BelizeBridgeReceiver` deployed on any foreign chain", implying one had to be written — was half right. The receiver **is** written: `infra/relayer/contracts/BelizeBridgeReceiver.sol` (455 lines) plus `BelizeWrappedToken.sol` (127 lines) and a relayer with 119 tests. Measured 2026-11-07: contracts compile clean under solc 0.8.38 (receiver bytecode 5901 B, 46 ABI entries) and **119/119 relayer tests pass**. The design is more careful than the blocker implied — the domain separator folds in `belizeGenesis` so a signature for one chain instance cannot replay after a re-genesis (where `NextTxId` restarts at zero), value and message digests carry disjoint kind tags, and signers must be submitted in ascending address order. **Now measured end to end against a live node** (Hardhat EVM + a 3-authority swarm, genesis `0x70a76ba6…`): receiver deployed at `0xe7f1725e…` (4910 bytes of code); `check-validator-sets.mjs` reports **ALIGNED** — the pallet's 3 registered ML-DSA validators (each `setEvmAddress`-ed) match the contract's 3 secp256k1 validators, and `PqSignatureThreshold = 3` matches the contract's on-chain `threshold()`; `deliverMessage` **executed** (tx `0x77c05371…`, 90166 gas) and `delivered[1]` flipped to true. The guards are not vacuous — they were each observed rejecting: 2-of-3 signatures on a 3-of-3 contract reverts `InsufficientSignatures(2, 3)`, a non-validator key reverts `NotValidator(0x1d3fc0f3…)`, and re-sending a delivered id is refused. The check also did its job on the first attempt, catching a real drift: the contract had been deployed with threshold 2 while the chain requires 3. **What is still missing:** no *foreign* chain — "local Hardhat" is not Sepolia, and nothing here proves behaviour across a real reorg, real gas markets, or a real bridge validator set operated by independent parties. So mainnet either deploys a receiver to an actual foreign chain, or drops the bridge claim; it does not need the contract written, and it no longer needs its execution demonstrated.<br><br>**2026-11-09 — the on-chain validator set now signs for real.** The three records registered on 2026-11-07 carried `randomAsU8a(2592)` as their PQ public key: the extrinsic path was proven, but no private key existed for those bytes, so `PqVerifier::verify` could never return true for them. Replaced via `scripts/rotate-bridge-pq-keys.js` with three ML-DSA-87 keypairs generated on Ceiba by `scripts/generate-bridge-pq-keys.sh` (OpenSSL 3.5, `openssl genpkey -algorithm ML-DSA-87`) and held 0600 in `/opt/belizechain/keys/bridge/`. Validators are `r1T2JCTaZw…`, `r1Ux2wHsp5…`, `r1SHELRhYz…`, each KYC L3 with 500 DALLA locked. **Proven, not assumed:** a signature produced from each `.pem` verifies under `fips204` 0.4.6 against the public key read back *from chain storage*, with the pallet's context `belizechain-bridge-v1`; the same signature is rejected under a wrong context string and against a tampered message, and one validator's signature is rejected by another's key. `author_insertKey` is **not** part of this path — the runtime's `SessionKeys` is `{ babe, grandpa }`, so there is no ML-DSA session key to insert.<br>**Two limitations remain, stated rather than hidden.** (a) The three bridge validators are the same three Oracle operator accounts: on a testnet with three funded trust anchors, role separation would need three more funded key holders. (b) Removing the three placeholder records released 1,500 DALLA of stake locks, but those accounts were created from seeds that were never retained, so the **2,767 DALLA** they hold (989 / 989 / 789) is unreachable. It is recoverable with root-only `balances.forceTransfer`, which has not been run. |
| 8 | ~~**DEX has no tradable market**~~ | ✅ **RESOLVED 2026-11-09** | The prior row was right that the pairs existed and wrong about why nothing could trade: `TotalBbzdSupply` had no issuance, so every bBZD reserve was bookkeeping with no ledger behind it. `scripts/bootstrap-dex.js` walked the production path in the only order the pallets allow — reserves → minter authorization → identity → KYC → mint bBZD → liquidity → a real swap — and settled against both ledgers, which is the check that actually retires the row. Measured after: `(0,1)` holds **100,199.52 DALLA / 199,801 bBZD**, `TotalBbzdSupply` **200,000 bBZD**, `CentralBankReserves` **2,000,000 DALLA**. Three oracle operators were seated (0 before) and the market maker reached Oracle KYC L2, because `Economy` reads the Oracle store, not Identity. Supply moved **3,001,009 → 3,102,266 DALLA** (+101,257) to fund the pool and the operator fee balances — that is real issuance, recorded here rather than implied. Bridge validators (row 7) were deliberately not registered at that time; they now are, with real ML-DSA-87 keys — see row 7. |
| 9 | ~~**Genesis stamps every council seat `Founder`**~~ | ✅ **RESOLVED 2026-11-07** | Two compounding defects. (a) The genesis build stamped *every* member `BoardRole::Founder` against `max_count() = 1`. (b) It never wrote `BoardComposition`, which `add_board_member` reads as the count — so the counter said 0 while three Founders sat, and a fourth could still be appointed. Roles now come from `GENESIS_SEAT_LADDER` and the counters are recomputed from the seats written. 175 governance tests pass; both halves falsified independently. Note this was never just descriptive metadata: the missing counter made `max_count` unenforceable.<br><br>**2026-11-09 — the ladder now matches the roster it has to seat.** `GENESIS_SEAT_LADDER` held 9 seats while `TESTNET_COUNCIL_BOOTSTRAP` supplies 12 (2 delegates × 6 districts), so genesis panicked with `12 council members exceeds the 9 seats`. The sum of `BoardRole::max_count` was only 11, so no ladder could have fitted a 12-seat roster either. Fixed by raising `CitizenDelegate` to 5 (total capacity 13) and extending the ladder to 12. The seat **order** matters as much as the length: seats are handed out from the front, so a three-seat genesis takes the first three and must get three distinct roles — a ladder starting `Founder, TechnicalSteward, TechnicalSteward` gave two members of one role and broke `genesis_seats_respect_each_roles_max_count`. `seat_ladder_respects_every_role_cap` now compares the ladder's length against the districts' own `seat_allocation()` sum and checks every role's count against its cap, so the two cannot drift apart silently. 176/176 governance tests pass.<br>**Live board state was still inconsistent after that fix, and is now repaired too.** On the running chain `governance.boardComposition` was **empty** while members sat as `Founder` (cap 1) — the chain was genesised before the counters were written, and no extrinsic wrote `BoardComposition` afterwards, so it needed a write rather than a migration. Measured 2026-11-09 that the two remaining board members were `//Bob` (`r1VBwEc9W86…`) and `//Charlie` (`r1VEbJnHYT6…`), i.e. the standard `DEV_PHRASE`; `testnet-spec.json` lists exactly `{//Alice, //Bob, //Charlie}`, so removing `//Alice` had left two of the three published genesis keys in place. All three were replaced: `sudo(remove_board_member)` × 2 then `sudo(add_board_member)` × 2, which is also the only path that writes the counter. Post-state read back from chain: **`r1UTSENDje4…` Founder, `r1T2JCTaZw…` TechnicalSteward**, both 2-year terms, `boardComposition = {Founder: 1, TechnicalSteward: 1}`, published dev keys on the board: **none**. Neither replacement is the sudo key, so board authority is not concentrated into the account that can already do anything. Value at risk throughout was small and measured — governance pallet account 0 DALLA, main treasury 2 DALLA, single approval under 10 DALLA — so this was a latent authority problem, not a live theft. |
| 10 | **Coverage 56 %** | measured | The CI gate is 40 %. That gate was set to make CI achievable, not to define a mainnet bar. Consensus-critical pallets should be argued individually. |
| 11 | ~~**7 weight files stale** (P0-19)~~ | ✅ **RESOLVED 2026-11-07** | All 20 pallets regenerated on Ceiba, one provenance, `check_weights_freshness.sh` passes. Residual, stated rather than hidden: the benchmark ran on the live host with all containers up, and one function (`payroll::execute_payment`) moved −24% with no source change — see §1.2b caveat 1. The effective safety margin against a 2.0 GHz validator is **1.52×** (§1.2c), which is thin. |
| 12 | ~~**Land ownership transfers without government approval** (P1-09)~~ | ✅ **RESOLVED 2026-11-07** | Two-phase transfer implemented: `transfer_property` only creates a pending request, `approve_transfer` moves title under `GovernmentOrigin`, `reject_transfer` refunds the tax. The two new extrinsics were benchmarked on 2026-11-07 and the provisional hand-set guesses were **1.94× and 2.34× too low** — replaced with measurements. |
| 13 | ~~**The live chain was writable by anyone on the internet**~~ | ✅ **RESOLVED 2026-11-07** | `Sudo.Key`, **both** consensus keys and **all three** identity-issuer lists belonged to one account: `0xd43593c7…da27d` / `r1Wm6WgK…`, the well-known `//Alice` dev key whose mnemonic is in every Substrate tutorial — on a chain whose RPC answers `state_getStorage` with no authentication. That was total compromise: `sudo.sudo(System::set_code(…))`, `set_storage`, `kill_prefix`, forged blocks, or forged KYC attestations. `ALICE_SUDO_RESET_2026-05-11.md` marked the key temporary and required the cutover **before opening to external peers**; the endpoint was opened anyway and the exposure survived roughly six months. **All three roles are now rotated and verified from the public endpoint** (`wss://testnet.belizechain.org/ws`): `Sudo.Key` → `0x32967ce9…3fb017`, `Session.Validators` → `r1UTSEND…`, `Babe.Authorities` → `0x6e451283…`, and `Ssn`/`Passport`/`Biometrics` issuers → `r1T7BYVAQ…`. A sweep of every non-map storage entry with privileged naming returns **zero** references to the old account, and `//Alice` calling `sudo.sudo(…)` is rejected `sudo.RequireSudo`. The authority switch landed at **block 461281, epoch 1538** with no stall: best/finalized held a 2-block gap throughout and the node logged no `failed to load session key`. Scripts: `rotate-sudo-key.js`, `rotate-validator.js`, `rotate-identity-issuer.js`. Three traps cost real retries and are recorded in each script and in `docs/operations/EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md` §11: `session.setKeys` needs a `POP_ ++ accountId` ownership proof per key; `pallet_session` delays a set change by one session by design; and `AttributeType` is a raw `u8` in extrinsics but an enum key in `IssuerBonds`. **Residual, stated rather than hidden:** `testnet-spec.json` (Ceiba copy *and* repo copy) still pins `//Alice` as `sudo.key` and as the sole session key, so **a re-genesis from that file restores the whole exposure.** That precondition is documented in the re-genesis runbook; the file was deliberately not edited because the genesis patch feeds the genesis hash and whether a node with an existing database re-reads genesis was not established. Also unchanged: the RPC still has no authentication, so key *secrets* are fixed but reads remain public. |
| 14 | ~~**Every deployed GEM contract was owned by the compromised `//Alice` key**~~ | ✅ **RESOLVED 2026-11-08** | The key rotation fixed *pallet* storage (`Sudo`, `Session`, `Identity` issuers), which an extrinsic can write. A contract's `owner`/`admin` lives **inside the instance** and only its own messages can change it, so all seven stayed on `//Alice` after blocker 13 was closed — no extrinsic reaches it. `scripts/transfer-contract-ownership.js` ran the two-step transfer for all seven roles (DALLA owner, faucet owner, BeliNFT owner, DAO admin, PSP37 owner, BelizeX factory admin, BelizeX fee setter) onto `r1T7BYVAQEF6GVZJFuvCqaHQt7tfdw8zdHyaNwjeHYmbdkYaP`. Verified by re-query, and by executing the two exploits against a caller with no rights: `mint` now returns `MissingRole`, `emergency_withdraw` returns `NotOwner`. The faucet's 10,000 DALLA was untouched throughout. `//Alice` no longer holds `ADMIN_ROLE` or `MINTER_ROLE`; the incoming key holds both. |
| 15 | **The boot spec still pins `//Alice` in genesis** | 🔴 **OPEN by decision — requires a re-genesis** | `chain-specs/genesis-live-0xb2664568.json` (byte-identical to `/data/chain/testnet-spec.json`, md5 `31171e10…`) carries four publicly-known dev keys across nine privileged slots. Measured 2026-11-08: its genesis hash — `0xb2664568…b5ef`, computed by booting the spec on a scratch base path — equals the live chain's `chain_getBlockHash(0)`, so the file **is** this chain's origin, not a stale copy. **Editing it would not harden anything: it would stop the node from starting**, because the genesis hash feeds the chain identity the database is validated against. Hardening genesis means starting a different chain — no history, no contracts. Blocked on that decision, not on work. |

---

## 4. What the test suite does and does not prove

Stated because "all tests pass" is repeatedly mistaken for readiness.

**It proves:** pallet logic is self-consistent against authored mocks; encoding,
arithmetic, origins and error paths behave as written; a real dev chain accepts the
transaction shapes (integration suites run against `belizechain-node --dev`).

**It does not prove:** that a live multi-validator network reaches consensus under load;
that the economic incentives are sound; that the PQ signatures resist a real adversary;
that storage proofs hold for adversarial inputs; or that any of this survives contact with
a professional auditor.

Two specific gaps, both real:

- **`ink_e2e` suites cannot run** in this repo's CI. Contract write paths are covered by
  `gem`'s own e2e script against a real node, not here.
- **`pallet_balances` has no `MaxIssuance`** (§1.1) — the property "supply can never exceed
  501 billion" is a convention, not an invariant.

---

## 5. Sign-off checklist

Nothing below is optional, and none of it can be satisfied by this repository alone.

- [ ] External audit completed; findings triaged with a written disposition each
- [ ] Penetration test executed against a production-like multi-node network
- [ ] Economic model reviewed independently; staking/DEX parameters justified
- [ ] `Sudo` removed at the mainnet transition. **Decision 2026-11-07: retained through the multi-validator phase, removed before mainnet.** It must not ship, but it is deliberately kept as break-glass while the validator set is being brought up. See P1-02 — the machinery to remove it safely already exists, so this is a scheduled step rather than outstanding engineering. **The retention decision assumed an unpublished Sudo key. It is not: see blocker #13 — the live `Sudo.Key` is `//Alice` and the RPC is public. Rotating it is now a prerequisite, not a mainnet step.**
- [x] **Live `Sudo` key rotated off `//Alice`, and the sole BABE authority rotated with it (blocker #13).** Done 2026-11-07 at block 461281, along with all three identity-issuer lists. Verified from the public endpoint; the DEX bootstrap is no longer gated on it.
- [ ] Validator set ≥ 4 authorities, run for ≥ 30 days without a stall or fork
- [ ] Genesis council roles corrected (§4.0a)
- [ ] Bridge far side deployed, **or** the bridge removed from public claims. **Progress 2026-11-07:** the receiver is deployed, reconciled and executing against a local EVM (blocker #7). What remains is a *real* foreign chain, not the contract.
- [ ] DEX pairs and liquidity listed, with the oracle guard exercised live
- [ ] `INK`/contract artefacts re-verified against the mainnet runtime, not testnet
- [ ] Rollback blob captured for the final pre-mainnet runtime and drill-restored once

---

## Change log

| Date | Change |
|---|---|
| 2026-11-06 | Created. All 22 P0 checked against source. P1-08 found already fixed (correcting prior notes). P0-22 found still live and corrected in two documents. P0-19 regeneration attempted, found throttled (governor `powersave`, 2.5 GHz of 4.8 GHz, uniform ~1.70× inflation) and reverted; the guard now detects the clock regime. |
| 2026-11-07 | **All 30 P1 items recounted individually against source** (the inherited list had only ever been spot-checked). Result: 19 fixed, 2 open, 5 obsolete, 1 false positive, 3 partial/policy. **P1-09 found live** — it had been counted as fixed; §2 now carries the full table and §3 gains blocker #12. Two near-miss false positives recorded: P1-14 (delegation *is* counted, via `calculate_voting_power`) and P1-03 (rate limiting *is* applied, but in `snippets/stack-paths.conf`, which a `nginx.conf`-only grep misses). |
| 2026-11-07 | **P1-09 FIXED.** `transfer_property` (call_index 1) now creates a `PendingTransfers` entry and takes the tax without touching ownership; `approve_transfer` (10, `GovernmentOrigin`) is the only writer of `Properties::owner` and re-checks ownership, verification, survey status and encumbrances before moving title; `reject_transfer` (11) refunds the tax. `Error::TransferNotApproved` remains declared but unused — left in place deliberately rather than removed, because removing an error variant shifts the indices of later variants and that is an ABI break on a live chain. Recount is now **20 fixed / 1 open / 3 partial / 6 obsolete-or-FP**, and blocker #12 is resolved. Two new tests were added per new call, and `transfer_property_does_not_move_title_until_approved` was confirmed to fail on the pre-fix code before acceptance. |
| 2026-11-07 | **Corrected a false claim in this document.** §1.2 said the reference weights were measured at "4.6–4.8 GHz"; all 20 weight files name an `Intel Core Ultra 7 155H`, which maxes at **4.5 GHz**, so that figure was impossible and is removed. Re-measured the host: 100 °C package temperature and 3.3–3.75 GHz loaded cores, i.e. it can no longer reproduce its own set. §1.2a now records this and §1.2b states the slow-validator margin as arithmetic (3× block safety ÷ ~2.4× speed gap ≈ **1.25×**). P0-19's plan changed from "regenerate the 7 stale files here" to "regenerate all 20 on Ceiba for one provenance". |
| 2026-11-07 | **P0-19 FIXED — all 20 weight sets regenerated on Ceiba.** 262 functions, one provenance (`ceiba`, Ryzen 9 5900X, `DATE: 2026-11-07`, `BENCH_MUL=1.25`), ~65 min, 0 failures, CPU held at 4.91–4.94 GHz. `check_weights_freshness.sh` moved from 7 stale errors to a pass; `cargo clippy --workspace --all-targets -- -D warnings` (the CI form) passes; runtime compiles; 97/97 landledger tests pass. **The pre-run prediction that a faster host would shrink the set was wrong** — median ratio 1.00× across all buckets, including the most compute-bound ones (§1.2c). The values that did move moved for checkable reasons: the two `wusdc` functions were hand-written and under-charged (1.25×/1.58×), and the two new landledger calls were benchmarked for the first time — the hand-set guesses were **1.94× and 2.34× too low**, i.e. unsafe. `transfer_property` fell 0.774× because P1-09 made it do strictly less (16r/10w → 14r/3w); the two-phase flow costs 221.8M combined, **27.6% more** than the old one-shot. §1.2b recomputed against the real 4.94 GHz host: **1.52×** effective margin, not 1.25×. Also fixed `scripts/bench_weights.sh` — `--all` derived its list from `define_benchmarks!` with a prefix regex, so `pallet_zk_attestation` was silently skipped by a run that reported success. |
| 2026-11-07 | **Prediction in §1.2a falsified by measurement — recorded rather than smoothed over.** §1.2a predicted the Ceiba set would come out *smaller* than the laptop set because the host is faster. It did not: median 1.00× across 262 functions, including the most compute-bound bucket (1.009×). The prediction was reasoned from "core clock dominates `ref_time`", which the data does not support. The paragraph is kept in place with a pointer to §1.2c, because a deleted wrong prediction teaches the next reader nothing. |
| 2026-11-07 | **P1-02 decided: Sudo is kept until mainnet.** Owner decision, recorded so it stops being re-litigated. Rationale: removing it also makes raw `System::set_code` / `set_storage` / `kill_prefix` unreachable, which costs testnet its break-glass recovery, and the multi-validator bring-up is precisely where that is most likely to be wanted. This is a deliberate deferral, not a blocker: the runtime is already prepared for removal (governance can apply upgrades with no Root, and the four `EnsureRoot`-only origins were re-pointed to council supermajority aliases), so the removal is a scheduled step at the mainnet transition rather than outstanding engineering. |
| 2026-11-07 | **Blocker #6 partially retired — 4-authority swarm reaches finality and survives an epoch rotation.** `scripts/local-swarm.sh` builds a throwaway chain from `testnet-template` with four real randomly-generated authority keypairs (not dev seeds), a distinct chain id (`belizechain_swarm`), and runs four native nodes on one host. Measured: 4/4 peers meshed, **0 block-hash mismatches across heights 0–60** on all four nodes, finality advancing continuously, and **`BabeEpochDuration` 300 rotation observed at block 253** (epoch 0 → 1) with the new authority set populated at 4 × weight 1 — i.e. the zero-authority epoch that killed Chain C on 2026-10-04 does **not** reproduce. Still open: all four nodes share a host, clock and loopback, so this proves consensus logic, not partition or clock-skew behaviour. Two silent-failure traps were found and encoded in the script rather than left as folklore: (1) `key generate-node-key` loads a chain spec, so the fail-closed mainnet config makes it die unless `--chain` is passed — upstream `sc-cli` behaviour, not ours; (2) `author_insertKey` against a *running* authority stores the key but the GRANDPA voter has already started with `local_key: None`, so blocks are authored while finality stays pinned at genesis — the node reports healthy throughout. Keys must be inserted with `key insert` before first start. |
| 2026-11-07 | **Same swarm, re-run cleanly from an empty work dir, plus fault-tolerance probes.** A second `up` with no manual intervention reproduced everything: 4/4 peers, finality from genesis, and **epoch 0 → 1 at block 301** with 4 authorities at weight 1, still finalizing afterwards (best 309 / finalized 307). Then two probes were run against the BFT threshold: **(a) 3 of 4** — validator 3 killed for 30 s, the survivors kept authoring *and* finalizing (height 80 → 84, `peers=2` on survivors), and v3 caught up to the tip immediately on restart (v1=87, v3=87 at t+3 s) with the mesh back to 3 peers each. **(b) 2 of 4** — v3 *and* v4 killed, below the 2/3 GRANDPA supermajority: **finality stalled at #314 exactly as predicted** while BABE kept authoring with backoff (best 321 → 324 → 330, so the lag grew 7 → 10 → 16 — block production continued, nothing finalized). Restarting both validators **recovered finality in under 6 s** (best 331 / finalized 329, lag 2), with no chain reset. That last point is the one with real weight: Chain C's stall required a re-genesis, whereas a below-threshold finality stall here is a *transient* condition that heals on its own once quorum returns. This is the first evidence in the repo for either behaviour. Still not tested: partitions/split-brain (two halves each believing they are the majority), clock skew, and geographic latency. |
| 2026-11-07 | **Blocker #8 re-scoped and mechanically fixed; blocker #7 re-scoped.** #8's stated cause ("0 pairs seeded, 0 pools", `LiquidityPools`) was wrong — Ceiba has 3 pairs and `WUSDC/bBZD` reports reserves. The actual defect is that bBZD has **never been minted** (`TotalBbzdSupply` unset, `BBZDBalances` empty), so every bBZD reserve is unbacked, plus two unsynchronised KYC stores (Economy reads oracle `IdentityVerifications`; BelizeX reads Identity attestations) and an oracle-operator gate (`MinOracleAgreement = 3`, seated by `TechnicalCouncilMember`) that no genesis bootstrap satisfies. `scripts/bootstrap-dex.js` walks the real production path and **proved settlement on a 4-validator swarm: 100 DALLA → 199.32 bBZD against a 1:2 pool**, converging on re-run. #7's "does not exist" was also wrong: `BelizeBridgeReceiver.sol` (455 lines) and a 119-test relayer already exist and compile clean under solc 0.8.38; what is missing is a deployment target, so the pallet↔contract validator key-set reconciliation has never run. |
| 2026-11-07 | **🔴 New blocker #13 — the live chain is writable by anyone on the internet.** Measured at #460257: `Sudo.Key` is `0xd43593c7…da27d` (`//Alice`, the public dev key), and the *same* key is the sole BABE authority and the sole SSN issuer, on a chain whose RPC answers `state_getStorage` unauthenticated. Anyone can call `sudo.sudo(System::set_code(…))`. This supersedes the framing of blockers #5 and #8 on testnet: keeping Sudo as "break-glass recovery" is only meaningful when the Sudo key is not published. `ALICE_SUDO_RESET_2026-05-11.md` required the cutover *before* opening the endpoint to external peers, and that precondition is breached. Recorded rather than fixed, deliberately: the fix is two owner-gated operations (rotate `sudo`, then rotate the BABE key and wait for an epoch), each capable of stalling the live chain, and rotating requires keys generated on Ceiba. **`bootstrap-dex.js` must not be pointed at Ceiba until this is done** — it would mint a stablecoin and write a reserve figure that anyone can overwrite. |
| 2026-11-07 | **✅ Blocker #13 resolved — all three privileged roles rotated off `//Alice`.** `Sudo.Key` → `r1T7BYVAQ…`; sole BABE authority → `r1UTSEND…` (enacted at block 461281, epoch 1538); `Ssn`/`Passport`/`Biometrics` issuers → `r1T7BYVAQ…`. Verified from the public endpoint, and a sweep of privileged storage returns zero references to the old account. The last two roles were not in the original finding: `pallet_belize_identity` keeps three separate issuer lists and `SsnIssuers` was only one of them, so the first fix left two forgery paths open that needed no sudo at all. Found by enumerating every privileged storage entry rather than fixing only what the report named — the lesson worth keeping. Residual: a re-genesis from `testnet-spec.json` still restores `//Alice`, documented in the re-genesis runbook. |
| 2026-11-07 | **Blocker #7 largely retired — the receiver now deploys, reconciles and executes.** `infra/relayer/scripts/deploy-receiver.mjs` deploys through the relayer's own tested signing primitives (no `ethers`, no second signing path); `scripts/deliver-message.mjs` + the existing `submitDelivery` drive a real delivery. Measured against a Hardhat EVM and a 3-authority swarm: deployed at `0xe7f1725e…`, `check-validator-sets.mjs` **ALIGNED** (3 pallet ML-DSA validators with EVM addresses ↔ 3 contract secp256k1 validators, `PqSignatureThreshold` 3 ↔ `threshold()` 3), `deliverMessage` executed in 90166 gas and `delivered[1]` became true. Each guard was observed *rejecting*, not just passing: `InsufficientSignatures(2, 3)`, `NotValidator(0x1d3fc0f3…)`, and replay refused. Encoding `to: ''` for CREATE was the one production-code gap this exposed — `signEip1559` rejected it, and it is a legal EIP-1559 transaction type. Three further defects were found by the run itself and fixed: `check-validator-sets.mjs` read pallet structs via decorated field access, but the relayer carries no type bundle so those fields are `undefined` and only `toJSON()` works; the CREATE address helper RLP-encoded the nonce as a zero-padded word instead of a minimal integer, so its prediction silently disagreed with the receipt; and the revert decoder initially rendered `address` words from the wrong end. The threshold drift the check caught on its first run (contract deployed at 2, chain requires 3) is the clearest evidence it is worth gating a deploy with. |
| 2026-11-11 | **Blocker #13 residual closed — the dev key's balances and session keys are gone.** The 2026-11-07 rotation removed `//Alice`'s privileges but not its DALLA: **957,513.011358374210** (31% of supply) was still spendable by anyone, and its BABE/GRANDPA `NextKeys` were still registered. Two genesis-funded accounts with no seed on record held 1,000,000 DALLA each (`nonce 0`, never used). Reclaimed with `session.purgeKeys()` and three `sudo(balances.forceTransfer)` at blocks 519,515 / 519,532 / 519,544 / 519,547; issuance moved only by the 0.000223003808 DALLA burned fee. Sources now hold 0.001 DALLA (ED); sudo holds 2,960,622.996649908069. **New residual: 95.4% of supply on the sudo key — distribute before external use.** |
