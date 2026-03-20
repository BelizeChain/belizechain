# BelizeChain Interoperability Pallet — Security Audit Report

**Auditor**: AI Security Auditor (Principal Level)  
**Date**: 2026-03-15  
**Pallet**: `pallet-belize-interoperability`  
**Source**: `pallets/interoperability/src/` (5 files: lib.rs, mock.rs, tests.rs, weights.rs, benchmarking.rs)  
**Scope**: Line-by-line security audit of bridge operations, PQ signatures, cross-chain messaging, escrow model, access control, storage safety, and weight accuracy.

---

## Executive Summary

The interoperability pallet implements a multi-signature bridge with post-quantum (ML-DSA-87) cryptographic verification, an escrow-based lock/unlock model, a challenge-period dispute mechanism, and cross-chain messaging. The pallet has undergone significant hardening (referenced as CONS-*, AR-*, B-*, H-*, M5* fixes). The production runtime correctly wires `MLDsaVerifier` instead of `PassthroughPQVerifier`.

**Overall Score: 72/100**

| Severity | Count |
|----------|-------|
| CRITICAL | 3 |
| WARNING  | 10 |
| INFO     | 7 |

---

## CRITICAL Findings

### CRIT-1: `update_bridge_config` allows setting `fee_rate > 10_000` (100%+ fee) — no validation

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1277-L1297)  
**Lines**: 1277–1297  
**Severity**: CRITICAL  

The `update_bridge_config` extrinsic accepts an arbitrary `fee_rate: u32` parameter and writes it directly to `ChainConfigurations` without validating that it is ≤ 10,000 basis points. While `integrity_test()` on line 773 validates `BridgeFeeRate` (the global constant), it does **not** validate per-chain `fee_rate` values set via governance.

A governance proposal (or compromised GovernanceCouncilMajority) could set `fee_rate = 15_000` (150%), causing `initiate_bridge` to compute a fee *larger than the bridged amount*. Because `fee_amount = Perbill::from_rational(fee_rate, 10_000) * amount`, a fee_rate > 10_000 is silently clamped by `Perbill` (max 1 billion PPB), but `from_rational(15_000, 10_000)` evaluates to `Perbill(1_000_000_000)` (100%), meaning the **entire bridge amount becomes the fee** and `net_amount = 0`. The user's tokens are sent entirely to treasury with zero bridged.

```rust
// Line 1286 — no validation on fee_rate
pub fn update_bridge_config(
    origin: OriginFor<T>,
    chain_index: u8,
    enabled: bool,
    fee_rate: u32,      // ← No upper bound check
    max_amount: u128,
) -> DispatchResult {
```

**Impact**: Users could lose 100% of bridged funds to treasury fees.  
**Fix**: Add `ensure!(fee_rate <= 10_000, Error::<T>::InvalidConfiguration);` before the mutation.

---

### CRIT-2: `update_bridge_config` silently no-ops if chain config doesn't exist

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1289-L1296)  
**Lines**: 1289–1296  
**Severity**: CRITICAL  

```rust
ChainConfigurations::<T>::mutate(&chain, |maybe_config| {
    if let Some(config) = maybe_config {
        config.enabled = enabled;
        config.fee_rate = fee_rate;
        config.max_amount = max_amount;
    }
    // ← If `None`, silently does nothing — no error returned
});
```

If governance calls `update_bridge_config` with a `chain_index` that has no existing `ChainConfig`, the extrinsic **succeeds silently** (returning `Ok(())`) and emits a `BridgeConfigUpdated` event, misleading governance into thinking the config was applied. This is distinct from a revertible error — it's a silent success with no state change.

**Impact**: Governance may believe a chain's fee/max_amount was updated when it was not, leading to policy enforcement failures.  
**Fix**: Return `Error::<T>::UnsupportedChain` if `maybe_config.is_none()`, or use `try_mutate` and return an error.

---

### CRIT-3: Dispute bond calculated from `bridge_tx.fee`, which is 0 for BurnAndUnlock — zero-cost disputes

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1325-L1327)  
**Lines**: 1325–1327  
**Severity**: CRITICAL  

```rust
// CONS-012: dispute bond = 5% of transaction fee
let dispute_bond: ... = (bridge_tx.fee / 20u128).saturated_into();
T::Currency::reserve(&who, dispute_bond)?;
```

For `BurnAndUnlock` transactions (incoming unlocks), `fee` is set to `0` on line 1195. Therefore `dispute_bond = 0 / 20 = 0`. A validator can file unlimited **zero-cost disputes** against all incoming unlock transactions, blocking every unlock indefinitely (since disputed transactions require governance intervention to resolve). There is no `resolve_dispute` extrinsic implemented, meaning disputed unlocks are permanently frozen.

**Impact**: Griefing attack — a single malicious validator can freeze all incoming bridge unlocks at zero cost with no on-chain resolution path.  
**Fix**:  
1. Calculate dispute bond from the transaction *amount* (e.g., `amount / 200` = 0.5%), not from `fee`.  
2. Implement a `resolve_dispute` governance extrinsic that can finalize or cancel disputed transactions.

---

## WARNING Findings

### WARN-1: No `resolve_dispute` extrinsic — disputed transactions are permanently frozen

**File**: [lib.rs](pallets/interoperability/src/lib.rs)  
**Severity**: WARNING (HIGH)  

The `dispute_bridge_transaction` extrinsic (call_index 6) sets `status = Disputed` and removes the transaction from `PendingFinalizations`. However, there is **no corresponding `resolve_dispute` extrinsic** to finalize or cancel disputed transactions. The comments at line 529 mention "governance can manually finalize or cancel stale entries via `resolve_dispute` / `cancel_bridge_transaction`" but these functions do not exist in the codebase.

**Impact**: Any successfully disputed transaction is **permanently stuck** in `Disputed` status. Locked funds in escrow can never be released or returned.  
**Fix**: Implement `resolve_dispute` (governance-only) that can either finalize (→ `Finalized`) or cancel (→ `Cancelled`, return escrow to initiator) a disputed transaction.

---

### WARN-2: `cancel_bridge_transaction` extrinsic does not exist

**File**: [lib.rs](pallets/interoperability/src/lib.rs)  
**Severity**: WARNING (HIGH)  

Referenced in comments (line 529) but not implemented. If a bridge transaction is fraudulent, governance has no on-chain path to cancel it and return escrowed funds.

**Fix**: Implement a governance-only `cancel_bridge_transaction` that reverts the escrow transfer and updates `TotalLockedAssets`.

---

### WARN-3: `submit_incoming_unlock` has no duplicate source_tx_hash protection

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1155-L1210)  
**Lines**: 1155–1210  
**Severity**: WARNING (HIGH)  

A validator can call `submit_incoming_unlock` multiple times with the **same** `source_tx_hash` from the same external chain, creating multiple independent `BridgeTransaction` entries. Each can then be signed by validators and executed, causing a **double-spend** on the unlock side.

The only guard is `TotalLockedAssets >= amount`, which is decremented upon `process_unlock` execution. However, if multiple unlock transactions for the same external burn are created and signed before the first is executed, they could all pass the `locked >= amount` check and drain the escrow beyond what was originally locked.

```rust
// No check like:
// ensure!(!SourceTxHashUsed::<T>::contains_key(&source_chain, &source_tx_hash), ...);
```

**Impact**: Double-spend of unlocked assets if multiple unlock requests for the same external burn event are submitted and finalized before execution.  
**Fix**: Add a `StorageDoubleMap<BridgeChain, BoundedVec<u8>, bool>` for `SourceTxHashUsed` and check uniqueness on submission.

---

### WARN-4: `PendingFinalizations` iteration uses `StorageMap::iter()` which is O(n)

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L749)  
**Lines**: 749  
**Severity**: WARNING (MEDIUM)  

```rust
for (tx_id, expiry_block) in PendingFinalizations::<T>::iter().take(20) {
```

`StorageMap::iter()` performs a prefix scan over all keys. While `.take(20)` bounds the *processing*, the iterator itself may scan significantly more entries to find the first 20. If there are thousands of pending finalizations (e.g., from a spam attack during a period when no blocks had leftover weight), this prefix scan could consume non-trivial PoV and time.

**Impact**: Weight underestimation in `on_idle` if `PendingFinalizations` grows large.  
**Fix**: Consider using `drain()` instead of `iter()` + separate remove loop, or use an `iter_from` approach with a stored cursor.

---

### WARN-5: `on_idle` finalization skips non-expired entries without processing order guarantee

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L747-L762)  
**Lines**: 747–762  
**Severity**: WARNING (MEDIUM)  

`StorageMap::iter()` returns entries in **storage key order** (lexicographic on Blake2_128Concat hash), not by expiry block. This means:
1. The first 20 entries scanned may not include expired ones, causing expired transactions to be skipped.
2. Legitimate finalizations at the "end" of the iteration order may never be processed if earlier (non-expired) entries always fill the `.take(20)` window.

**Impact**: Starvation — some bridge transactions may never auto-finalize even after their challenge period expires.  
**Fix**: Change to `PendingFinalizations::<T>::iter().filter(|(_, expiry)| n >= *expiry).take(20)` — but this still scans all entries. A better approach is a sorted data structure or a deadline-indexed storage.

---

### WARN-6: `UserBridgeLocks` accounting is inconsistent — decremented for recipient, not initiator

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1112-L1114)  
**Lines**: 1112–1114  
**Severity**: WARNING (MEDIUM)  

In `process_unlock`:
```rust
UserBridgeLocks::<T>::mutate(&recipient, |locked| {
    *locked = locked.saturating_sub(amount);
});
```

But `UserBridgeLocks` is incremented for the **initiator** (the user who called `initiate_bridge`) on line 843. For `BurnAndUnlock`, the recipient may be a *different* account than whoever originally locked tokens. This means:
- The initiator's `UserBridgeLocks` counter is never decremented.
- The recipient's counter is decremented from a value that may be 0 (no prior bridge activity), which `saturating_sub` silently handles as a no-op.

**Impact**: `UserBridgeLocks` becomes an ever-growing phantom counter for bridge initiators, providing inaccurate accounting. Not exploitable for fund theft, but breaks any downstream logic depending on this storage.  
**Fix**: The `UserBridgeLocks` tracker should be redesigned to track per-transaction escrow amounts, or removed if it's purely informational.

---

### WARN-7: `register_bridge_validator` uses `set_lock` which can conflict with other locks

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1388-L1389)  
**Lines**: 1388–1389  
**Severity**: WARNING (LOW)  

```rust
T::Currency::set_lock(BRIDGE_LOCK_ID, &who, stake, ...);
```

`set_lock` uses a fixed `BRIDGE_LOCK_ID = *b"bzbridge"`. If the same account registers as a validator and also initiates bridge transactions (which previously used `set_lock` before the CONS-027 escrow migration), the locks would have shared the same ID and overwritten each other. While the escrow model (CONS-027) now avoids this for bridge initiators, the validator stake still uses `set_lock`. If a validator calls `register_bridge_validator` multiple times (blocked by the duplicate check) this is safe, but the lock amount is not proportional to actual stake — it's always `10 × MinBridgeAmount` regardless of the validator's actual balance.

**Impact**: Low. The duplicate check prevents re-registration, and the escrow migration eliminates the original conflict.

---

### WARN-8: Missing `pq_signatures_required` validation in `update_bridge_config`

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1277-L1297)  
**Lines**: 1277–1297  
**Severity**: WARNING (MEDIUM)  

`update_bridge_config` does not allow updating `pq_signatures_required`, `min_confirmations`, or `rpc_endpoint`. These fields are fixed at initial chain config creation (which has no dedicated extrinsic — configs must be set via storage migration or direct insertion). There is no extrinsic to **create** a new chain configuration, only to update existing ones.

**Impact**: No way to add new chain support post-genesis without a runtime upgrade or storage migration. No way to increase PQ signature requirements for an already-configured chain.  
**Fix**: Add an `add_chain_config` governance extrinsic and allow `update_bridge_config` to modify `pq_signatures_required`.

---

### WARN-9: `provide_pq_signature` does not verify validator supports the transaction's target chain

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L908-L960)  
**Lines**: 908–960  
**Severity**: WARNING (MEDIUM)  

The validator struct contains `supported_chains`, but `provide_pq_signature` never checks whether the signing validator actually supports the chain involved in the bridge transaction. A validator registered only for Bitcoin could sign Ethereum bridge transactions.

```rust
// Line 922 — validator fetched but supported_chains never checked
let _validator = Self::bridge_validators(&who)
    .ok_or(Error::<T>::ValidatorNotRegistered)?;
```

**Impact**: Reduces the security model from "N-of-M validators *who monitor that specific chain*" to "N-of-M *any registered validators*". A validator who doesn't run an Ethereum node could blindly sign Ethereum bridge transactions.  
**Fix**: Add `ensure!(validator.supported_chains.contains(&target_chain), Error::<T>::UnsupportedChain);`

---

### WARN-10: Hand-estimated weights — not benchmarked for production

**File**: [weights.rs](pallets/interoperability/src/weights.rs#L1-L6)  
**Lines**: 1–6  
**Severity**: WARNING (MEDIUM)  

The weights file explicitly states: "THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates." While benchmarks exist in `benchmarking.rs`, the weights in `weights.rs` are hand-estimated and the runtime uses `SubstrateWeight<Runtime>` which references these estimates. The computed weights do not account for:
- ML-DSA-87 signature verification (~25ms per verify on WASM)
- Variable number of signatures scanned for duplicate check (linear in `pq_signatures.len()`)
- The `provide_pq_signature` weight is a flat 30M ref_time but does PQ crypto

**Impact**: Block weight underestimation could allow more bridge transactions per block than the chain can safely process, opening DoS vectors.  
**Fix**: Run `frame-benchmarking` and generate proper weights, especially for `provide_pq_signature` which performs real PQ cryptographic verification.

---

## INFO Findings

### INFO-1: Mock test uses `PassthroughPQVerifier` — tests don't validate real PQ crypto

**File**: [mock.rs](pallets/interoperability/src/mock.rs#L118)  
**Lines**: 118  
**Severity**: INFO  

The test mock wires `PassthroughPQVerifier` which accepts any signature ≥ 64 bytes. This means all signature-related tests pass without exercising the actual ML-DSA-87 verification path. The production runtime correctly uses `MLDsaVerifier`, but test coverage for the real verifier path is zero within this pallet's test suite.

**Fix**: Add integration tests that use `MLDsaVerifier` with real ML-DSA-87 key pairs and signatures.

---

### INFO-2: Mock test is missing `PalletId` configuration — tests may not compile or test escrow path correctly

**File**: [mock.rs](pallets/interoperability/src/mock.rs)  
**Severity**: INFO  

The mock `Config` implementation does not define `type PalletId`. This means either:
1. The mock doesn't compile (would be caught by `cargo test`), or
2. There's a default that isn't shown in the file excerpt.

Since tests appear to pass (based on the test file structure), a `PalletId` must be defined elsewhere or via a macro. However, the escrow account derivation in tests may not match production behavior.

---

### INFO-3: `BridgeChain::Polkadot` hardcoded as `source_chain` for all cross-chain messages

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1253)  
**Lines**: 1253  
**Severity**: INFO  

```rust
source_chain: BridgeChain::Polkadot, // BelizeChain as source
```

BelizeChain positions itself as sovereign and independent from Polkadot (line 9: "Independent sovereignty - NO Polkadot parachain association"), but cross-chain messages identify BelizeChain as `BridgeChain::Polkadot`. Should use a dedicated `BridgeChain::BelizeChain` variant.

---

### INFO-4: `PQSignatureThreshold` is defined in Config but not used in the pallet code

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L196-L198)  
**Lines**: 196–198  
**Severity**: INFO  

The runtime constant `PQSignatureThreshold` (3 in production, 5 in tests) is defined in the `Config` trait but is **never referenced** in the pallet code. Instead, the per-chain `pq_signatures_required` from `ChainConfig` is used. This makes the constant misleading — it suggests a global threshold but has no effect.

**Fix**: Either use `PQSignatureThreshold` as a minimum for `pq_signatures_required`, or remove it from `Config`.

---

### INFO-5: `decode_chain` casts `u8` to `u16` unnecessarily

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L1562)  
**Lines**: 1562  
**Severity**: INFO  

```rust
Some(match index as u16 {
```

The input is already `u8` (max 255) and all match arms are 0–50. The cast to `u16` is unnecessary and slightly misleading.

---

### INFO-6: `BridgeTransaction.pq_signatures` bounded to 32 but `MaxBridgeValidators` is 21 in production

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L380)  
**Lines**: 380  
**Severity**: INFO  

```rust
pub pq_signatures: BoundedVec<(AccountId, BoundedVec<u8, ConstU32<4627>>), ConstU32<32>>,
```

With `MaxBridgeValidators = 21` in production, the bound of 32 is safe but wasteful. Each entry is `AccountId (32 B) + 4627 B signature = ~4659 B`, so 32 entries ≈ 149 KB of storage per transaction struct. This is expensive on-chain.

---

### INFO-7: Events use numeric encoding (`u8`) for chain/asset instead of the enum variant

**File**: [lib.rs](pallets/interoperability/src/lib.rs#L611-L640)  
**Lines**: 611–640  
**Severity**: INFO  

All events encode `BridgeChain` and `BridgeAsset` as `u8` via `encode_chain()`/`encode_asset()` rather than including the enum variants directly. This makes event decoding by UI/indexer clients dependent on matching the pallet's encoding table exactly.

---

## Per-File Scores

| File | Lines | Score | Notes |
|------|-------|-------|-------|
| [lib.rs](pallets/interoperability/src/lib.rs) | ~1720 | **68/100** | 3 CRITICAL + 8 WARNING findings. Core logic is solid (escrow model, canonical PQ signing, challenge period, rate limiting) but missing dispute resolution, fee_rate validation, and duplicate source_tx_hash protection. |
| [mock.rs](pallets/interoperability/src/mock.rs) | ~180 | **75/100** | Reasonable mock with KYC/sanctions simulation. Missing PalletId config and MLDsaVerifier integration tests. |
| [tests.rs](pallets/interoperability/src/tests.rs) | ~1300+ | **80/100** | Good coverage of happy paths, KYC enforcement, sanctions, rate limiting, dispute flow. Missing: double-spend test for submit_incoming_unlock, fee_rate > 10K test, dispute resolution path test. |
| [weights.rs](pallets/interoperability/src/weights.rs) | ~58 | **55/100** | Hand-estimated, not benchmarked. PQ verification weight severely underestimated. |
| [benchmarking.rs](pallets/interoperability/src/benchmarking.rs) | ~230 | **72/100** | Covers all 6 weight functions. KYC bypass documented. setup_bridge_tx uses `saturating_add` not `checked_add`. |

---

## Security Architecture Assessment

### What's Done Well
1. **Escrow model (CONS-027)**: Replacing `set_lock` with treasury transfers to a PalletId-derived escrow account correctly enables cross-user unlocks without lock conflicts.
2. **Canonical PQ signing (CONS-001)**: The signature message binds `tx_id + amount + chain + address/recipient + asset`, preventing cross-transaction replay.
3. **Domain separation**: ML-DSA-87 verification uses context `b"belizechain-bridge-v1"` to prevent cross-protocol replay.
4. **Challenge period (§4.4b)**: The `on_idle` finalization with bounded iteration (max 20) and weight gating is well-designed.
5. **Rate limiting (AR-15)**: Per-account, per-block bridge initiation limits with block-boundary reset.
6. **KYC/sanctions integration**: Level 2 for users, Level 3 for operators, sanctions screening at every entry point.
7. **PassthroughPQVerifier safety**: Panics in non-test/non-benchmark builds, preventing accidental production use.
8. **ID overflow protection (B-9)**: `checked_add` with `IdOverflow` error on all ID counters.
9. **Duplicate signature prevention (M52)**: `pq_signatures.iter().any(|(v, _)| v == &who)` prevents same validator signing twice.

### Critical Gaps
1. **No dispute resolution path** — disputes permanently freeze funds.
2. **No duplicate burn hash protection** — double-spend on unlock side.
3. **No fee_rate validation** — governance can set confiscatory fees.
4. **Weights not benchmarked** — PQ crypto weight severely underestimated.

---

## Recommended Priority Actions

| Priority | Action | Effort |
|----------|--------|--------|
| P0 | Add `fee_rate <= 10_000` validation in `update_bridge_config` | 1 line |
| P0 | Return error when chain config doesn't exist in `update_bridge_config` | 3 lines |
| P0 | Implement `resolve_dispute` governance extrinsic | ~50 lines |
| P0 | Add `SourceTxHashUsed` storage to prevent duplicate unlock submissions | ~15 lines |
| P1 | Calculate dispute bond from transaction amount, not fee | 1 line |
| P1 | Check validator `supported_chains` in `provide_pq_signature` | 3 lines |
| P1 | Run frame-benchmarking and generate real weights | DevOps task |
| P2 | Add `add_chain_config` governance extrinsic | ~30 lines |
| P2 | Fix `UserBridgeLocks` accounting or remove tracker | ~10 lines |
| P2 | Add integration tests with real ML-DSA-87 signatures | ~100 lines |

---

*End of audit report. Generated 2026-03-15.*
