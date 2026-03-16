# BelizeX DEX Pallet — Complete Security Audit Report

**Audit Date**: 2026-03-15  
**Auditor**: AI Security Auditor (Blockchain & DeFi Specialist)  
**Pallet**: `pallet-belize-belizex` (BelizeX Decentralized Exchange)  
**Files Audited**: 5 files, 3206 lines total  
**Substrate Version**: Polkadot SDK stable2512  
**Audit Priority**: HIGHEST — DEX handles user funds directly  

---

## 1. Executive Summary

The BelizeX pallet implements an AMM-based DEX with liquidity pools, limit order book, multi-hop routing, tourism fee discounts, and Oracle price guards. The pallet shows evidence of prior audit remediation (noted comments referencing X-1, X-2, X-8, X-9, X-10, M47, H-24, DOS-008, AR-10), but several **critical architectural and economic issues** remain.

### Severity Distribution

| Severity | Count | Description |
|----------|-------|-------------|
| **CRITICAL** | 3 | Fund loss risk, non-functional components, architectural flaws |
| **HIGH** | 4 | Weight underestimation, fee bypass, Oracle inconsistency |
| **MEDIUM** | 5 | Economic correctness, missing validations, dead code |
| **LOW/INFO** | 8 | Code quality, naming, unused fields |
| **Total** | **20** | |

### Overall Pallet Score: **38 / 100** (FAIL — Do Not Deploy)

The fundamental single-currency architecture means the DEX cannot actually exchange different assets, and the limit order book is non-functional. These two CRITICAL findings alone block production deployment.

---

## 2. CRITICAL Findings

### C-1: Single-Currency Architecture — DEX Cannot Exchange Different Assets

**Severity**: CRITICAL  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L76)  
**Lines**: 76, 730-735, 842-856, 1196-1201  
**Category**: State Inconsistency / Insecure Design  

**Description**: The entire DEX operates on a single `Currency` type (native DALLA). Despite defining four `AssetId` variants (DALLA, BBZD, TourismDALLA, WUSDC), all currency transfers use the same `T::Currency`. There are no separate token balances for BBZD, TourismDALLA, or WUSDC.

**Code Evidence** — `add_liquidity` transfers the combined amount as one token:
```rust
// lib.rs L730-735
let pool = Self::pool_account();
T::Currency::transfer(
    &who,
    &pool,
    base_amount.saturating_add(quote_amount),  // SINGLE token transfer for both "assets"
    ExistenceRequirement::KeepAlive,
)?;
```

`execute_trade` transfers `amount_in` in and `amount_out` out — same token:
```rust
// lib.rs L842-856
T::Currency::transfer(&who, &pool, amount_in.saturated_into(), ...)?;
T::Currency::transfer(&pool, &who, amount_out_balance, ...)?;
```

**Impact**: 
- There is no actual asset exchange. A "DALLA → BBZD swap" just sends DALLA to the pool and receives DALLA back.
- The AMM reserves (`base_reserve`, `quote_reserve`) are synthetic accounting entries, not backed by distinct token balances.
- LP providers deposit DALLA and receive back DALLA — the "pair ratio" is artificial.
- The entire concept of "price" between these assets is economically meaningless without distinct tokens.

**Attack Vector**: None directly (accounting is self-consistent within the single-currency model), but users expecting a real multi-asset exchange are fundamentally misled.

**Fix Recommendation**: 
1. Integrate `pallet-assets` or an equivalent multi-asset framework to represent BBZD, WUSDC, etc. as distinct on-chain tokens.
2. Modify all transfer logic to use asset-specific transfer functions.
3. Alternatively, if this is by design (e.g., single-token liquidity bootstrapping), document it prominently and restrict AssetId to only DALLA.

---

### C-2: Limit Order Book Is Non-Functional — Orders Can Never Execute

**Severity**: CRITICAL  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L908-L998)  
**Lines**: 908–998 (place_limit_order), 1230–1260 (cancel_order)
**Category**: Insecure Design / Dead Code  

**Description**: The pallet supports `place_limit_order` and `cancel_order`, but there is **no extrinsic or hook to match or execute limit orders**. Orders are stored in `OrderBook<T>` but can never be filled. Additionally, `place_limit_order` does **not reserve or lock any funds** from the order creator.

**Code Evidence** — Order placement stores the order but doesn't touch funds:
```rust
// lib.rs L978-991
let order = OrderEntry {
    order_id, creator: who.clone(),
    pair: (base_asset_id.clone(), quote_asset_id.clone()),
    order_type: order_type_enum.clone(),
    amount, price, remaining: amount,
    expires_at, is_tourism_order,
};
OrderBook::<T>::insert(order_id, order);
NextOrderId::<T>::put(order_id.saturating_add(1));
// NO Currency::reserve() or Currency::transfer() call
```

**Impact**:
- Users can place unlimited fake orders (up to `MaxOrdersPerAccount`) with no economic backing.
- No order matching engine exists, so limit orders are permanently inert.
- The `AccountOrderCount` counter and `MaxOrdersPerAccount` limit manage state growth but serve no economic purpose.
- Storage bloat from unfillable orders accumulates indefinitely (orders have `expires_at` but no garbage collection).

**Attack Vector**: An attacker can place `MaxOrdersPerAccount` orders per account across unlimited accounts, consuming storage with zero economic cost.

**Fix Recommendation**:
1. Either implement a complete order matching engine with fund reservation (`Currency::reserve()`/`Currency::unreserve()`), expiry cleanup, and fill/partial-fill logic.
2. Or remove the order book entirely to eliminate dead code and storage attack surface.
3. If keeping orders, add on-chain `on_idle` garbage collection for expired orders.

---

### C-3: No Minimum LP Token Lock (First-Depositor Rounding Attack)

**Severity**: CRITICAL  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L693-L710)  
**Lines**: 693–728  
**Category**: Economic Attack / Integer Overflow-Truncation  

**Description**: When the first liquidity provider deposits to an empty pool, LP tokens are calculated as `sqrt(base * quote)`. There is no "dead shares" mechanism (like Uniswap V2's `MINIMUM_LIQUIDITY = 1000` permanently locked). This allows a first-depositor inflation attack:

1. Attacker deposits 1 wei each of base and quote → receives `sqrt(1*1) = 1` LP token.
2. Pool has reserves (1, 1) and 1 LP token.
3. Attacker trades to skew the pool ratio heavily (since reserves are tiny, small trades cause massive price impact).
4. Legitimate depositor adds proportional liquidity, but due to integer truncation in the `min(lp_from_base, lp_from_quote)` formula with extreme ratios, they receive 0 LP tokens while their funds are absorbed by the pool.

**Code Evidence**:
```rust
// lib.rs L693-710 - Initial deposit
let lp_tokens = if pair.total_lp_tokens == 0 {
    let product = U256::from(base_amount_u128)
        .checked_mul(U256::from(quote_amount_u128))
        .ok_or(Error::<T>::ArithmeticOverflow)?;
    let lp_u256 = product.integer_sqrt();
    let lp: u128 = lp_u256.try_into()
        .map_err(|_| Error::<T>::ArithmeticOverflow)?;
    lp  // No minimum lock! Can be 1
}
```

**Mitigating Factor**: In this single-currency architecture (C-1), the attacker would need to manipulate reserves via `execute_trade`, and the constant-product formula prevents draining reserves to exactly 0. However, the attack vector remains valid for extreme ratio manipulation.

**Fix Recommendation**:
1. Lock `MINIMUM_LIQUIDITY` (e.g., 1000) LP tokens to address zero on first deposit, similar to Uniswap V2.
2. Ensure LP minting returns error (not 0) when calculated LP tokens would be 0.
3. Enforce minimum initial deposit amounts (e.g., `ensure!(lp >= 1000, Error::BelowMinimumAmount)`).

---

## 3. HIGH Findings

### H-1: `remove_liquidity` Uses Wrong Weight Annotation

**Severity**: HIGH  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L1164)  
**Line**: 1164  
**Category**: Weight / Gas Underestimation  

**Description**: The `remove_liquidity` extrinsic uses `T::WeightInfo::add_liquidity()` instead of `T::WeightInfo::remove_liquidity()`.

```rust
// lib.rs L1164
#[pallet::weight(T::WeightInfo::add_liquidity())] // similar weight profile ← WRONG
pub fn remove_liquidity(...)
```

In the `SubstrateWeight` implementation:
- `add_liquidity()`: 45M ref_time, 2048 PoV, 4 reads, 4 writes
- `remove_liquidity()`: 50M ref_time, 2560 PoV, 5 reads, 5 writes

**Impact**: The extrinsic charges ~10% less weight than its actual cost. At scale, this enables block stuffing — more `remove_liquidity` calls fit in a block than the validator has capacity to execute, causing execution time overruns.

**Fix**:
```rust
#[pallet::weight(T::WeightInfo::remove_liquidity())]
```

---

### H-2: Multihop Trade Weight Severely Underestimated

**Severity**: HIGH  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L1010)  
**Line**: 1010  
**Category**: Weight / Gas Underestimation  

**Description**: `execute_multihop_trade` is annotated with a single `execute_trade()` weight, but can perform up to 4 hops (path length 5). Each hop reads and writes `TradingPairs` storage.

```rust
// lib.rs L1010
#[pallet::weight(T::WeightInfo::execute_trade())] // 1x weight for up to 4x operations
pub fn execute_multihop_trade(...)
```

**Impact**: A 4-hop multihop trade consumes ~4x the DB operations of a single trade but is charged only 1x weight. This is exploitable for block-time attacks and DoS.

**Fix**: Weight should be `T::WeightInfo::execute_trade() * (path.len() - 1)`, or define a dedicated weight function parameterized by hop count.

---

### H-3: Multihop Tourism Fee Bypasses Minimum Fee Floor

**Severity**: HIGH  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L1051-L1058)  
**Lines**: 1051-1058  
**Category**: Economic Correctness  

**Description**: `execute_trade` uses `get_effective_fee_rate()` which enforces a `.max(10)` minimum floor (10 bps). But `execute_multihop_trade` calculates fees directly without this floor:

```rust
// lib.rs L1051-1058 (multihop)
let base_fee_rate = if is_tourism_trade {
    pair.fee_rate.saturating_sub(T::TourismDiscountRate::get())
} else {
    pair.fee_rate
};
// NO .max(10) floor applied!
```

**Impact**: For the TourismDALLA/BBZD pair (`fee_rate = 15`), a tourism trader via multihop gets `15 - 10 = 5 bps` fee, while via single `execute_trade` gets `max(15 - 10 - volume_discount, 10) = 10 bps`. Multihop bypasses the economic floor.

**Fix**: Apply the same minimum fee floor in multihop, or call `get_effective_fee_rate()` internally.

---

### H-4: Oracle Guard Inconsistency Between Trade Types

**Severity**: HIGH  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L823-L840) and [lib.rs](pallets/belizex/src/lib.rs#L1068-L1080)  
**Category**: Oracle / External Data Manipulation  

**Description**: For WUSDC/BBZD trades:

- `execute_trade` (L823): Uses `T::Oracle::get_crypto_exchange_rate()` directly. If `None` → **hard reject** with `OracleRateUnavailable`.
- `execute_multihop_trade` (L1068): Uses `Self::get_verified_exchange_rate()` which **falls back to on-chain AMM pricing** if Oracle returns `None`.

```rust
// execute_trade L823 - Hard reject if Oracle unavailable
} else {
    Self::deposit_event(Event::OracleRateUnavailable { ... });
    return Err(Error::<T>::OracleRateUnavailable.into());
}

// execute_multihop L1068 - Falls back to manipulable AMM price
if let Some(oracle_rate) = Self::get_verified_exchange_rate(a, b) {
    // Can use AMM price as "oracle" — attacker controls this
```

**Impact**: An attacker can bypass the Oracle guard on WUSDC/BBZD by routing through multihop. If Oracle is stale/unavailable, multihop silently falls back to the AMM price (which the attacker can manipulate), while direct trade correctly rejects.

**Fix**: Use the same Oracle lookup and rejection logic in both code paths. Never fall back to on-chain AMM price for guarded pairs.

---

## 4. MEDIUM Findings

### M-1: `add_liquidity` Doesn't Return Excess Tokens When Ratio Changes

**Severity**: MEDIUM  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L703-L730)  
**Category**: State Inconsistency  

**Description**: For subsequent deposits, LP tokens = `min(lp_from_base, lp_from_quote)`, but the full `base_amount + quote_amount` is transferred from the user. If the pool ratio shifted between submission and execution, one side's contribution is partially wasted (donated to existing LPs).

**Impact**: Users can lose value if pool ratio changes between tx creation and execution. In a high-MEV environment, this enables sandwich attacks on liquidity additions.

**Fix**: Calculate the actual proportional amounts, transfer only those, and return the remainder. Alternatively, accept only one deposit amount and calculate the other from the ratio.

---

### M-2: `DailyVolume` Never Resets — Accumulates Forever

**Severity**: MEDIUM  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L337-L342)  
**Category**: State Inconsistency / Unbounded Growth  

**Description**: `DailyVolume<T>` is incremented on every trade but never reset. Despite the name "daily", it's cumulative all-time volume.

**Impact**: Misleads any external consumer (dashboard, Oracle, governance) relying on "daily" volume. The u128 value will saturate at u128::MAX after sufficient trading, though this is purely theoretical at 12 decimal places.

**Fix**: Implement daily reset via `on_initialize` or `on_idle` using block-based day boundaries, or rename to `CumulativeVolume`.

---

### M-3: No Maximum `fee_rate` Validation on `create_trading_pair`

**Severity**: MEDIUM  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L637-L660)  
**Category**: Access-Control (Governance Error)  

**Description**: `create_trading_pair` accepts any `fee_rate: u32` with no bounds check. A governance error or malicious proposal could set fee_rate to 10000 (100%) or higher.

```rust
// lib.rs L637 - No validation
fee_rate, // Accepted as-is, could be 10000+ bps
```

**Impact**: A pair with fee ≥ 10000 bps would charge 100%+ of trade input as fees, effectively stealing user funds.

**Fix**: `ensure!(fee_rate <= 1000, Error::<T>::InvalidFeeRate);` — cap at 10%.

---

### M-4: `SelfTradeNotAllowed` Error Declared But Never Used

**Severity**: MEDIUM  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L601)  
**Category**: Dead Code / Missing Validation  

**Description**: The error variant `SelfTradeNotAllowed` is declared but never checked in any extrinsic. Additionally, `base_asset == quote_asset` is never validated in `create_trading_pair`, `add_liquidity`, or `execute_trade`.

**Impact**: A pair like (DALLA, DALLA) could theoretically be created, allowing nonsensical self-swaps. In practice, governance would need to create it, limiting the attack surface.

**Fix**: Add `ensure!(base_asset != quote_asset, Error::<T>::SelfTradeNotAllowed);` to `create_trading_pair`.

---

### M-5: Oracle Guard Only Protects WUSDC/BBZD — Other Pairs Unguarded

**Severity**: MEDIUM  
**File**: [lib.rs](pallets/belizex/src/lib.rs#L815-L840)  
**Category**: Oracle / Price Manipulation  

**Description**: The Oracle price deviation guard is hardcoded to only trigger for `WUSDC == base_asset && BBZD == quote_asset`. DALLA/BBZD and TourismDALLA/BBZD trades have no Oracle protection.

**Impact**: Pool prices for unguarded pairs can be manipulated freely via large trades, enabling sandwich attacks and arbitrage extraction from LPs.

**Fix**: Generalize Oracle guard to any pair where Oracle rates are available. Check `if T::Oracle::get_crypto_exchange_rate(base_asset, quote_asset).is_some()` rather than hardcoded pair ID comparison.

---

## 5. LOW / INFO Findings

### I-1: `AssetId::from(u8)` and `OrderType::from(u8)` Silently Default Unknown Values

**File**: [lib.rs](pallets/belizex/src/lib.rs#L178) and [lib.rs](pallets/belizex/src/lib.rs#L271)  
**Category**: Incorrect Error Handling  

`AssetId::from(255)` silently becomes `DALLA`. `OrderType::from(200)` silently becomes `Buy`. These should return `Result` or at least be documented as intentional fallbacks. A user calling `execute_trade(base=99, quote=100, ...)` would silently resolve to `(DALLA, DALLA)`.

---

### I-2: `LiquidityProvider::rewards_earned` Field Never Incremented

**File**: [lib.rs](pallets/belizex/src/lib.rs#L200)  
**Category**: Dead Code  

The field always remains 0. No LP reward distribution mechanism exists. Either implement LP rewards or remove the field to reduce storage overhead.

---

### I-3: Dev-Only Storage Items in Production

**File**: [lib.rs](pallets/belizex/src/lib.rs#L364-L371)  
**Category**: Determinism / Cleanup  

`DevSeedEnabled` and `DevSeedDone` are genesis-only bootstrapping flags. After first block, they're permanently set and consume storage. Should be removed after initial deployment via a migration.

---

### I-4: `Randomness` Type Included But Explicitly Unused

**File**: [lib.rs](pallets/belizex/src/lib.rs#L84)  
**Category**: Dead Code  

The comment acknowledges this is reserved for future use. No security concern, but it adds unnecessary trait bounds.

---

### I-5: Multihop Event Reports Only Treasury Fee as `fee_paid`

**File**: [lib.rs](pallets/belizex/src/lib.rs#L1107)
**Category**: Incorrect Error Handling  

```rust
fee_paid: total_treasury_fee, // Only treasury portion, not total fee
```

External indexers expecting `fee_paid` to represent total fees will under-report. Single `execute_trade` correctly reports total `fee` (L880).

---

### I-6: Wrong Error Types in Multihop Path Validation

**File**: [lib.rs](pallets/belizex/src/lib.rs#L1028-L1029)  
**Category**: Incorrect Error Handling  

```rust
ensure!(path.len() >= 2, Error::<T>::PairNotFound);      // Should be a path/validation error
ensure!(path.len() <= 5, Error::<T>::SlippageExceeded);   // Should be PathTooLong or similar
```

---

### I-7: All Weights Are Hand-Estimated — Not Benchmark-Verified

**File**: [weights.rs](pallets/belizex/src/weights.rs#L1-L5)  
**Category**: Weight / Gas Underestimation  

The file header explicitly states: "THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates." For a fund-handling DEX pallet, benchmark-generated weights are essential before mainnet.

---

### I-8: `get_lp_balance` Helper Iterates All Pairs (Unbounded)

**File**: [lib.rs](pallets/belizex/src/lib.rs#L1375-L1378)  
**Category**: Unbounded Iteration  

```rust
pub fn get_lp_balance(account: &T::AccountId) -> u128 {
    LPBalances::<T>::iter_prefix(account)
        .fold(0u128, |acc, (_, bal)| acc.saturating_add(bal))
}
```

This is a read-only helper not called in any extrinsic, so it cannot cause on-chain DoS. However, it could be expensive for RPC queries if many pairs exist.

---

## 6. Per-File Scores

| File | Lines | Score | Rationale |
|------|-------|-------|-----------|
| [lib.rs](pallets/belizex/src/lib.rs) | 1480 | **32/100** | Contains 3 CRITICAL, 4 HIGH, 5 MEDIUM findings. Single-currency DEX is architecturally unsound. Dead order book. No minimum LP lock. Weight bugs. Fee bypass. Oracle inconsistency. Core AMM math (constant-product, U256 checked arithmetic, slippage checks) is correctly implemented. |
| [tests.rs](pallets/belizex/src/tests.rs) | 1263 | **62/100** | 47 tests covering happy paths and error paths. Good coverage of access control, pausing, slippage, events. Missing tests: first-depositor attacks, extreme reserves, weight correctness, multihop fee floor bypass, Oracle fallback differences. No fuzz testing. |
| [mock.rs](pallets/belizex/src/mock.rs) | 172 | **70/100** | Clean mock setup. `MockKyc` always passes (noted). `MockOracle` returns static 1:1 rates (limits Oracle guard testing). No mock variant for KYC denial runtime. ExistentialDeposit=1 (differs from mainnet 0.001 DALLA). |
| [benchmarking.rs](pallets/belizex/src/benchmarking.rs) | 205 | **55/100** | Covers 9 of 11 extrinsics. Missing: `cancel_order`, `execute_multihop_trade`. `setup_trading_pair_with_liquidity` inserts reserves without backing funds in pool account — benchmarks may not reflect real-world DB access patterns. |
| [weights.rs](pallets/belizex/src/weights.rs) | 86 | **45/100** | Hand-estimated only. No production benchmark data. `remove_liquidity` correctly defined but unused (H-1). Read/write counts appear roughly correct but ref_time values are unverified. |

---

## 7. Overall Pallet Score: 38 / 100

| Category | Score | Weight | Weighted |
|----------|-------|--------|----------|
| Fund Safety | 30/100 | 25% | 7.5 |
| AMM Math Correctness | 75/100 | 15% | 11.25 |
| Access Control | 80/100 | 10% | 8.0 |
| Error Handling | 55/100 | 10% | 5.5 |
| Weight Accuracy | 25/100 | 10% | 2.5 |
| State Consistency | 35/100 | 10% | 3.5 |
| Test Coverage | 55/100 | 10% | 5.5 |
| Code Quality | 60/100 | 10% | 6.0 |
| **Overall** | | | **49.75 → 38*** |

*Adjusted down from 49.75 due to C-1 (single-currency) being a blocking architectural defect that invalidates the entire DEX premise.

---

## 8. Priority Remediation Table

| Priority | ID | Finding | Effort | Impact |
|----------|-----|---------|--------|--------|
| **P0** | C-1 | Integrate multi-asset framework (pallet-assets) | HIGH | Blocks all real DEX functionality |
| **P0** | C-2 | Remove or fully implement order book w/ fund reservation | MEDIUM | Dead code + storage attack surface |
| **P0** | C-3 | Add MINIMUM_LIQUIDITY lock on first deposit | LOW | Prevents first-depositor attack |
| **P1** | H-1 | Fix `remove_liquidity` weight annotation | TRIVIAL | Weight undercharge |
| **P1** | H-2 | Fix multihop weight to scale with path length | LOW | Block-stuffing DoS vector |
| **P1** | H-3 | Apply min fee floor in multihop fee calculation | LOW | Fee bypass |
| **P1** | H-4 | Unify Oracle guard logic (hard reject if unavailable) | LOW | Oracle bypass via multihop |
| **P2** | M-1 | Return excess tokens in add_liquidity | MEDIUM | Value loss on ratio shift |
| **P2** | M-3 | Cap fee_rate in create_trading_pair (≤1000 bps) | TRIVIAL | Governance safety |
| **P2** | M-4 | Enforce base_asset ≠ quote_asset | TRIVIAL | Self-pair prevention |
| **P2** | M-5 | Generalize Oracle guard to all pairs with Oracle data | LOW | Price manipulation |
| **P3** | M-2 | Implement DailyVolume reset or rename | LOW | Data accuracy |
| **P3** | I-1–I-8 | Code quality improvements | LOW | Maintainability |
| **P3** | — | Run `frame-benchmarking` to generate real weights | MEDIUM | Weight accuracy |
| **P3** | — | Add fuzz tests and adversarial LP scenarios | MEDIUM | Test coverage |

---

## 9. What Works Well

Despite the critical findings, several aspects demonstrate competent implementation:

1. **AMM constant-product formula** — correctly implemented using `U256` checked arithmetic with proper error propagation.
2. **Slippage protection** — `min_amount_out` on trades and `min_lp_tokens` / `min_base_amount` / `min_quote_amount` on liquidity operations.
3. **Access control** — all governance functions properly gated with `EnsureOrigin`. KYC checks on user-facing extrinsics. Creator-only order cancellation.
4. **Global and per-pair pausing** — comprehensive emergency stop mechanism covering all user flows.
5. **Oracle price guard** (for WUSDC/BBZD) — deviation-based guard with configurable threshold and event logging.
6. **U256 overflow protection** — all financial calculations use `checked_mul`/`checked_div` with explicit error handling.
7. **PalletId-based pool account** — deterministic escrow derivation; no admin key can access funds.
8. **Per-account order count limit** — `MaxOrdersPerAccount` prevents storage flood DoS.
9. **Saturating arithmetic** — prevents silent wrapping in balance operations.
10. **Event emission** — comprehensive event coverage for all state mutations.

---

## 10. Appendix: Audit Checklist Summary by Category

| # | Category | Status | Notes |
|---|----------|--------|-------|
| 1 | Integer overflow/underflow | ✅ PASS | U256 checked math throughout |
| 2 | Division by zero | ✅ PASS | `.max(1)` guards, `ensure!(reserve > 0)` |
| 3 | Reentrancy/double-spend/TOCTOU | ✅ PASS | Substrate dispatchable model prevents reentrancy; state updated atomically |
| 4 | Access-control bypass | ✅ PASS | Proper `EnsureOrigin` usage, KYC gating |
| 5 | Unbounded iteration/storage DoS | ⚠️ WARN | `get_lp_balance` unbounded (helper only); order book GC missing |
| 6 | Incorrect error handling | ⚠️ WARN | Wrong error types in multihop; silent defaults in `From<u8>` |
| 7 | State inconsistency | ❌ FAIL | Single-currency architecture; excess tokens not returned; DailyVolume never resets |
| 8 | Unsafe blocks | ✅ PASS | No unsafe code |
| 9 | Cryptographic weakness | ✅ PASS | No custom crypto; Randomness unused |
| 10 | Oracle manipulation | ⚠️ WARN | Guard only on one pair; inconsistent fallback behavior |
| 11 | Concurrency/race | ✅ PASS | Substrate's single-threaded executor prevents races |
| 12 | Determinism violations | ✅ PASS | No floating-point; no host-dependent ops |
| 13 | Weight underestimation | ❌ FAIL | remove_liquidity wrong weight; multihop severely undercharged; hand-estimated only |
| 14 | Cross-pallet trust | ✅ PASS | KYC trait properly abstracted; Oracle trait with sensible fallback |

---

*End of audit. This report covers 3206 lines across 5 files with 20 findings (3 CRITICAL, 4 HIGH, 5 MEDIUM, 8 INFO). Pallet score: 38/100 — not suitable for mainnet deployment without remediation of P0 items.*
