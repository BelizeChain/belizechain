# BelizeChain — Economic & Tokenomics Security Audit 2026

**Scope**: `pallets/economy`, `pallets/belizex`, `pallets/payroll`, `runtime/src/lib.rs` economic config, codebase-wide minting‐operation search  
**Date**: 2026-07-14  
**Auditor**: AI Security Audit (Claude Opus 4.6)  
**Status**: Complete — all source files fully read  

---

## Executive Summary

The economic subsystem is **mature and well-hardened** after extensive prior remediation (H-19, H-20, CM-1, CM-2, X-1, X-2, COMP-CRIT-1/2, etc.). Supply-cap guards exist at every identified `deposit_creating` call site except one. The DEX uses U256 arithmetic to prevent overflow. Tourism cashback cannot be self-claimed. Payroll uses employer funds (no treasury drain). The fee model is congestion-responsive with bounded multipliers.

However, **7 findings** remain, including **1 HIGH**, **3 MEDIUM**, and **3 LOW/INFO** severity issues.

---

## Severity Legend

| Tag | Meaning |
|-----|---------|
| **H** | HIGH — exploitable or can cause unbounded supply inflation |
| **M** | MEDIUM — design weakness; limited exploitability but should be fixed |
| **L** | LOW — minor correctness issue |
| **I** | INFO — observation, documentation issue, or defense-in-depth recommendation |

---

## Findings

### H-ECON-1: Whistleblower `deposit_creating` Has NO Supply-Cap Check

**File**: `pallets/whistleblower/src/lib.rs`, lines 346–348  
**Severity**: HIGH

```rust
let reward = EscrowedReward::<T>::take(report_id)
    .ok_or(Error::<T>::NoEscrowedReward)?;
let _ = T::Currency::deposit_creating(&claimant, reward);
```

The `claim_reward` function mints DALLA via `deposit_creating` without any check against `MaxDallaSupply` or `total_issuance`. While the reward amount is drawn from a pre-funded pool (`WhistleblowerPool`), the `fund_whistleblower_pool` function (call_index 3) allows governance to increment the pool counter **without a corresponding balance transfer** when the origin is a pure collective (no signer):

```rust
let new_total = if let Ok(funder_account) = funder {
    // Transfer from funder account...
} else {
    // governance accounting entry — no balance transfer
```

This means:
1. Governance can inflate `WhistleblowerPool` to any value without locking real funds.
2. `review_report` escrows rewards from this phantom pool.
3. `claim_reward` then mints via `deposit_creating` with no supply-cap guard.

**Impact**: Unbounded DALLA inflation through governance-controlled whistleblower rewards that bypass the 501B cap.

**Remediation**: Add a supply-cap guard before `deposit_creating`:

```rust
let current = T::Currency::total_issuance();
ensure!(
    current.checked_add(&reward).is_some_and(|t| t <= T::MaxSupply::get()),
    Error::<T>::SupplyCapExceeded
);
let _ = T::Currency::deposit_creating(&claimant, reward);
```

---

### M-ECON-2: Limit Orders Do Not Reserve Funds at Placement

**File**: `pallets/belizex/src/lib.rs`, lines 910–975 (`place_limit_order`)  
**Severity**: MEDIUM

The `place_limit_order` function stores an `OrderEntry` in `OrderBook` storage but does **not** reserve or escrow the trader's funds. The order records `amount` and `remaining` but no balance is locked:

```rust
let order = OrderEntry {
    order_id,
    creator: who.clone(),
    // ...
    amount,
    remaining: amount,
    // ...
};
OrderBook::<T>::insert(order_id, order);
```

There is no `execute_order` extrinsic visible in the pallet — orders appear to be stored but never filled. This means:
1. A trader can place orders for arbitrary amounts they don't hold.
2. The orderbook becomes a misleading record of phantom liquidity.
3. If order execution is added later without balance reservation, it creates a double-spend vector.

**Impact**: Misleading orderbook depth. If execution logic is added, funds may not exist at fill time.

**Remediation**: Either (a) reserve funds at placement via `T::Currency::reserve(&who, amount)` and unreserve on cancel, or (b) remove limit orders until execution logic is implemented.

---

### M-ECON-3: Consensus `finalize_round` Supply-Cap Check Is Insufficient

**File**: `pallets/consensus/src/lib.rs`, lines 977–990  
**Severity**: MEDIUM

The `finalize_round` function checks that `total_issuance + total_reward` doesn't overflow, but it does **not** check against `MaxDallaSupply`:

```rust
// M56 FIX: Verify total issuance + reward won't overflow before minting
ensure!(
    T::Currency::total_issuance().checked_add(&total_reward).is_some(),
    Error::<T>::SupplyCapExceeded
);
```

This only prevents arithmetic overflow (which is extremely unlikely with u128), not exceeding the 501B supply cap. Compare with the staking pallet's correct implementation:

```rust
let headroom = max_supply.saturating_sub(current_issuance);
let capped_reward = reward.min(headroom);
```

Each validator share is then minted individually via `deposit_creating` without clamping to remaining headroom.

**Impact**: After total issuance approaches 501B DALLA, consensus rewards from `finalize_round` can push total supply past the cap. Each round mints `ConsensusReward` (split among validators) without cap enforcement.

**Remediation**: Add MaxSupply headroom clamping identical to the staking pallet's pattern:

```rust
let max_supply = T::MaxSupply::get();
let current_issuance = T::Currency::total_issuance();
let headroom = max_supply.saturating_sub(current_issuance);
if headroom.is_zero() { return Ok(()); }
let total_reward = T::ConsensusReward::get().min(headroom);
```

---

### M-ECON-4: Decimal Inconsistency Between Runtime and Economy Pallet

**File**: `runtime/src/lib.rs` line ~115, `pallets/economy/src/lib.rs` line ~1082 (burn_dalla docstring)  
**Severity**: MEDIUM

The runtime defines:
```rust
pub const DOLLARS: Balance = 1_000_000_000_000; // 10^12 — implying 12 decimals
pub const EXISTENTIAL_DEPOSIT: Balance = 1_000_000_000; // 0.001 DALLA
```

But the economy pallet's `burn_dalla` doc comment says:
```rust
/// // Burn 1000 DALLA (1000 * 10^6 with 6 decimals)
/// Economy::burn_dalla(Origin::signed(user), 1_000_000_000)?;
```

This claims 6 decimals, but `1_000_000_000` ÷ `DOLLARS (10^12)` = 0.001 DALLA, not 1000. This is a documentation bug that could mislead integrators.

Additionally, `MaxDallaSupply = 501_000_000_000_000_000`:
- With 12 decimals (DOLLARS = 10^12): max supply = **501,000** DALLA ← suspiciously low
- With 6 decimals (10^6): max supply = **501,000,000,000** (501B) DALLA ← matches intent

This suggests the codebase may actually intend 6 decimals despite `DOLLARS = 10^12`, or `MaxDallaSupply` is miscalibrated.

**Impact**: Off-chain integrators and block explorers may display incorrect token amounts. If the supply cap constant was set assuming 6 decimals but the runtime uses 12, the effective cap is only 501K DALLA — far below the intended 501B.

**Remediation**: Reconcile decimals across all documentation and constants. Verify `MaxDallaSupply` aligns with `DOLLARS`. If 12 decimals is correct, `MaxDallaSupply` should be `501_000_000_000 * 10^12 = 501_000_000_000_000_000_000_000`.

---

### L-ECON-5: Tourism Cashback Paid Via Transfer — No Per-Period Budget Cap

**File**: `pallets/economy/src/lib.rs`, lines 1038–1043  
**Severity**: LOW (economic, not exploit)

Tourism incentives are paid from the treasury via `Currency::transfer`:

```rust
let treasury = T::Treasury::get();
T::Currency::transfer(&treasury, &tourist, incentive_amount, ExistenceRequirement::KeepAlive)?;
```

This is correct — it will fail if the treasury lacks funds, and `KeepAlive` prevents draining below ED. However, there is **no per-period cap** on total tourism incentive payouts. A high volume of legitimate tourism transactions could deplete the treasury over time.

Tourism **cannot** be self-claimed: `is_merchant_verified(&vendor, category_id)` requires Oracle verification of the vendor, and the tourist pays the vendor (transferring funds), receiving a smaller cashback. Self-referring (tourist == vendor) requires the tourist to also be Oracle-verified as a merchant — this is an off-chain governance check.

**Impact**: Treasury depletion through sustained high-volume tourism, not through exploit. Defense-in-depth recommendation.

**Remediation**: Consider adding a per-epoch tourism incentive budget cap.

---

### L-ECON-6: DEX Dev Seed Liquidity Repeatable if `DevSeedDone` Is Reset

**File**: `pallets/belizex/src/lib.rs`, lines 435–490 (`on_idle`)  
**Severity**: LOW

The one-time dev seed in `on_idle` is gated by `DevSeedDone`. However, if governance (or a runtime upgrade) resets `DevSeedDone` to `false` while `DevSeedEnabled` remains `true`, the seed re-executes, adding phantom WUSDC/BBZD liquidity to the pool without real token backing.

```rust
if !Self::dev_seed_enabled() || Self::dev_seed_done() {
    return Weight::zero();
}
// ... adds 10T base + 20T quote to pool reserves ...
```

This inflates pool reserves without corresponding `Currency::transfer`, meaning the pool's accounting diverges from actual token balances held by the pool account.

**Impact**: Protocol-level reserve inflation if DevSeedDone is reset. Unlikely in production but represents a latent footgun.

**Remediation**: Remove the dev seed logic for mainnet builds or gate it behind `#[cfg(feature = "dev")]`.

---

### I-ECON-7: Inflation Applied Only Once Per ~365 Days — Timing Drift

**File**: `pallets/economy/src/lib.rs`, lines 440–454  
**Severity**: INFO

Inflation is triggered in `on_initialize` when `blocks_passed >= BLOCKS_PER_YEAR`:

```rust
let blocks_passed = current_block.saturating_sub(last_block);
if blocks_passed >= BLOCKS_PER_YEAR as u64 {
```

With 6-second blocks, `BLOCKS_PER_YEAR = 5,256,000`. If an inflation event is triggered at block `N`, the next one fires at block `N + 5,256,000`. But `LastInflationBlock` is set to the **current block** (not `last + BLOCKS_PER_YEAR`), so any delay in the first trigger pushes all subsequent inflation events forward, compounding timing drift.

**Impact**: Cosmetic — inflation timing drifts slightly over years but the rate remains 2%.

**Remediation**: Set `LastInflationBlock` to `last_inflation + BLOCKS_PER_YEAR` instead of `n` to eliminate drift.

---

## Supply Cap Analysis — All `deposit_creating` Call Sites

| Pallet | File:Line | Supply Cap Guard | Status |
|--------|-----------|-----------------|--------|
| economy | `economy/src/lib.rs:476,484,491` | `new_supply <= max_supply` check at L454 | ✅ Guarded |
| staking | `staking/src/lib.rs:976` | `headroom = max_supply - issuance; capped_reward = reward.min(headroom)` | ✅ Guarded |
| staking | `staking/src/lib.rs:1336` | Same headroom/min pattern (PoUW domain rewards) | ✅ Guarded |
| consensus | `consensus/src/lib.rs:990` | Only overflow check (`checked_add`), **not** MaxSupply | ⚠️ **M-ECON-3** |
| whistleblower | `whistleblower/src/lib.rs:346` | **NONE** | ❌ **H-ECON-1** |
| community | `community/src/lib.rs:1426` | `checked_add + <= MaxSupply` via ensure! | ✅ Guarded |
| community | `community/src/lib.rs:1558` | `checked_add + <= MaxSupply` via ensure! | ✅ Guarded |

**Answer: Can supply exceed 501B?** Yes — through whistleblower rewards (H-ECON-1) and consensus finalize_round rewards (M-ECON-3). All other minting paths are properly guarded.

---

## Key Security Questions — Answers

| Question | Answer |
|----------|--------|
| **Can supply exceed 501B through any path?** | **YES** — via H-ECON-1 (whistleblower) and M-ECON-3 (consensus finalize_round). See Supply Cap Analysis table. |
| **Can the treasury be drained?** | No single exploit path. Tourism cashback uses `transfer` (fails on insufficient funds) with `KeepAlive`. Inflation mints to treasury. L-ECON-5 notes sustained volume could deplete treasury over time (not an exploit). |
| **Can DEX math be exploited for profit extraction?** | No. Constant product formula uses U256 (overflow-safe). Oracle guard prevents WUSDC/BBZD price manipulation. Fee floor at 10 bps. Division by zero prevented by `ensure!(reserve > 0)`. |
| **Can tourism cashback be self-claimed?** | Practically no. Requires `is_merchant_verified(&vendor, category_id)` via Oracle. Tourist pays vendor first (real transfer), then receives smaller cashback. Self-referral requires the tourist to also be Oracle-verified as a merchant. |
| **Are all reward distributions capped and overflow-safe?** | All use `saturating_*` arithmetic. Supply cap: 5/7 sites guarded; consensus and whistleblower lack proper MaxSupply enforcement. |
| **Is the fee model resistant to manipulation?** | Yes. `TargetedFeeAdjustment` with `MinMultiplier(1/10)` and `MaxMultiplier(10/1)` prevents fee collapse. `OperationalFeeMultiplier = 5` prioritizes system extrinsics. Attacker cannot zero out fees. |

---

## Payroll Security Summary

The payroll pallet is **clean** from a supply-inflation perspective:

- Payments use `T::Currency::transfer` from employer to employee — **no minting**.
- Salary, bonuses, and deductions all use `saturating_sub` / `saturating_add`.
- Scheduled payments in `on_idle` are bounded by `MaxEmployees` per employer.
- Privacy design: salary commitments via blake2_256 hashes; events emit hashes not amounts (though extrinsic inputs remain visible on-chain — Substrate constraint).
- No `deposit_creating` anywhere in the payroll pallet.

**No findings** in the payroll pallet.

---

## Remediation Priority

| ID | Severity | Effort | Priority |
|----|----------|--------|----------|
| H-ECON-1 | HIGH | Low (3 lines) | **Immediate** |
| M-ECON-3 | MEDIUM | Low (5 lines) | **Next sprint** |
| M-ECON-2 | MEDIUM | Medium | Next sprint |
| M-ECON-4 | MEDIUM | Medium (audit all constants) | Next sprint |
| L-ECON-5 | LOW | Low | Backlog |
| L-ECON-6 | LOW | Low | Backlog |
| I-ECON-7 | INFO | Trivial | Backlog |

---

*End of Economic & Tokenomics Security Audit*
