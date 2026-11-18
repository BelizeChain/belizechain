# Maya Academy — how a learner gets paid

**Status**: live on Jade (Chain D), spec 128, verified 2026-11-18.
**Audience**: whoever operates the education programme. No Rust knowledge needed.

---

## 1. The shape of the flow

```
learner studies the module in Maya Wallet
        │
        ▼
learner passes the quiz (75%)          ← UI only. The chain cannot check this.
        │
        ▼
provider attests completion ON CHAIN   ← scripts/attest-education.py
        │
        ▼
learner clicks "Claim"                 ← needs the attestation to exist
        │
        ▼
DALLA is minted to the learner
```

The middle step exists because **the pallet cannot read a quiz result**. Before spec
128, `complete_education_module` accepted any non-empty byte string as a "proof" and
minted 50 DALLA for it — `vec![0]` was worth 50 DALLA per module, four times. The
wallet's quiz was the only obstacle, and a wallet is not a security boundary.

Spec 128 replaced that with an on-chain attestation: a named account says "this
learner passed this module", the claim checks it, and the claim **consumes** it. One
attestation buys exactly one reward.

## 2. Where the money comes from

**New issuance.** Not the faucet, not the treasury, nothing to top up.

```rust
let capped_reward = module.reward_amount.min(T::EducationRewardAmount::get());
ensure!(issuance + capped_reward <= T::MaxSupply::get(), SupplyCapExceeded);
T::Currency::deposit_creating(&who, capped_reward);
```

| Bound | Value |
|---|---|
| `EducationRewardAmount` — per module | **50 DALLA** |
| `MaxDallaSupply` — global | 501,000,000 DALLA |

⚠️ **A module can advertise more than it pays.** Module 1 is seeded with 75 DALLA
against a 50 cap, so it credits 50. The events now report the minted amount, and the
wallet measures the balance delta rather than trusting any announcement — so a learner
sees the truth either way. If you want module 1 to actually pay 75, either raise
`EducationRewardAmount` or lower the module's `reward_amount`; both need a runtime
upgrade.

## 3. Attesting a learner — the operator path

```bash
python3 scripts/attest-education.py --rpc ws://100.119.97.38:9944 --list

python3 scripts/attest-education.py --rpc ws://100.119.97.38:9944 \
    --learner r1WwqnjLgrD5i55vX1D4U5Q4fSoQBoUDCWampx58b51Pwd9ce \
    --module 3 --via-sudo --execute
```

Dry run by default: it prints the modules, the learner, whether each module is already
attested or already claimed, and the extrinsic hash it *would* submit.

### ⚠️ `--via-sudo` is required on Chain D today

`Config::OracleAttestationOrigin` is `TechnicalCouncilMember`, which accepts:

* `RawOrigin::Root` — **works**
* `RawOrigin::Signed(who)` where `who` is a seated TechnicalCouncil member

Seating a council member needs a collective motion, and there is none on this chain. So
signing directly with the sudo key fails:

```
Community.attest_education_completion(...)  →  BadOrigin
```

**Signing with the sudo key is not Root.** `sudo.sudo(call)` dispatches the inner call
as Root, which is why the script offers `--via-sudo`. Measured 2026-11-18: the direct
signed path returns `BadOrigin`, the sudo path succeeds.

The proper fix, when the programme has real providers, is to seat each provider in
TechnicalCouncil (or repoint `OracleAttestationOrigin` at a dedicated provider set).
Until then `--via-sudo` is the only working route, and it means **every attestation is
a root action** — one more reason the sudo key is a pre-mainnet removal item.

## 4. What was verified on the live chain (2026-11-18)

| Step | Result |
|---|---|
| Claim with no attestation, module 3 | ❌ `AttestationRequired`, no reward minted |
| `attest_education_completion` signed directly | ❌ `BadOrigin` |
| `sudo.sudo(attest_education_completion)`, module 3 | ✅ attested |
| Claim module 3 (advertises **60**) | ✅ net **+49.999774 DALLA** = 50 minted − fee |
| Attestation after the claim | ✅ consumed (gone) |
| Second claim, same module | ❌ `AlreadyCompleted` |
| `EducationRewardClaimed.amount`, module 0 | ✅ `50000000000000` = **50 DALLA** |

Module 3 is the useful case: it advertises 60 and paid 50, and the balance moved by 50
— which is exactly the bug that made learners believe they had been short-changed.

## 5. Current state

| Module | Title | Advertised | Actually paid |
|---|---|---|---|
| 0 | Financial Literacy for Belizeans | 50 | 50 |
| 1 | Sustainable Farming Techniques | **75** | **50** ⚠️ |
| 2 | Participating in Digital Democracy | 50 | 50 |
| 3 | Blockchain and Web3 Fundamentals | 60 | **50** ⚠️ |

The sudo account completed modules 0 and 3 while verifying the gate — 100 DALLA minted
in total. That is real chain state, not a simulation.

## 6. Known gaps

* **The quiz is not enforced on chain.** The attestation is only as trustworthy as the
  provider that submits it. A provider that attests without checking anything has
  reintroduced the original problem with extra steps.
* **`completion_proof` is still unread.** The wallet sends a JSON blob and the pallet
  stores nothing; it is evidence for off-chain review only.
* **No bulk tooling.** `--module all` attests every module for one learner. There is no
  "attest this cohort" import yet.
* **Weights for the gate are derived, not measured.** `complete_education_module`
  gained one read and one write; `attest_education_completion` is modelled on
  `attest_participation`. Both are biased low for `ref_time`. Re-benchmark with
  `./scripts/bench_weights.sh pallet_belize_community` on `ceiba`.
