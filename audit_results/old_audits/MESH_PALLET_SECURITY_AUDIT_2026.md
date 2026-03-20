# MESH PALLET — LINE-BY-LINE SECURITY AUDIT 2026

**Auditor**: AI Security Auditor (Rust/Substrate)  
**Date**: 2026-03-15  
**Pallet**: `pallet-belize-mesh` (Meshtastic LoRa Mesh Network)  
**Scope**: `pallets/mesh/src/{lib.rs, types.rs, weights.rs, benchmarking.rs, mock.rs, tests.rs}`  
**Lines Audited**: ~2,700 (core logic in lib.rs ~1,515 lines)

---

## EXECUTIVE SUMMARY

| Metric | Value |
|--------|-------|
| **Overall Score** | **78/100** |
| CRITICAL findings | 3 |
| WARNING findings | 9 |
| INFO findings | 8 |
| Extrinsics audited | 14 |
| Storage items audited | 14 |
| Test coverage | Good (45+ tests, most error paths tested) |

The mesh pallet is well-structured with good access control patterns and saturating arithmetic throughout. However, it has **3 critical issues** involving unbounded storage growth, relay proof manipulation, and missing active-status validation. Several medium-severity issues exist around economic incentive misalignment, missing event data, and weight accuracy.

---

## FILE-BY-FILE ANALYSIS

### 1. `lib.rs` — Core Pallet Logic (Score: 74/100)

#### CRITICAL

**MESH-C1: Unbounded `ProcessedMeshTransactions` storage growth — no pruning mechanism**  
- **Location**: Storage declaration (L243–249) and usage across `submit_mesh_transaction` (L870) and `confirm_emergency_alert` (L1184–1190)
- **Issue**: `ProcessedMeshTransactions` is written to on every mesh transaction submission and every alert confirmation. The comment at L245 says "pruned periodically" but **no pruning logic exists** — not in `on_initialize`, not in any extrinsic, nowhere. Over time this storage map grows unboundedly, increasing state bloat and PoV costs.
- **Impact**: State bloat → increased PoV sizes → slower block production → potential chain halt on parachain if PoV budget is exceeded.
- **Severity**: **CRITICAL**
- **Recommendation**: Add a pruning `on_idle` hook or a bounded `BoundedBTreeMap` with expiry. Alternatively, add an `on_initialize` that removes entries older than N blocks.

**MESH-C2: `confirm_emergency_alert` dedup sentinel reuses `ProcessedMeshTransactions` — hash collision risk and semantic pollution**  
- **Location**: L1170–1192 (`confirm_emergency_alert`)
- **Issue**: The dedup mechanism for alert confirmations hashes `(alert_id, node_id)` into an `H256` and stores it in `ProcessedMeshTransactions`, which is semantically a map for *mesh transaction hashes*. This creates two problems:
  1. **Hash collision**: An attacker could craft a mesh transaction whose `tx_hash` collides with a dedup sentinel, causing that transaction to be rejected as "duplicate" by `submit_mesh_transaction` (L869–872). The collision space is 256-bit so practically unlikely, but the design is semantically wrong.
  2. **Pruning interference**: If pruning is ever added for `ProcessedMeshTransactions`, it would also prune alert dedup sentinels, re-enabling confirmation spam.
- **Severity**: **CRITICAL** (design flaw, not exploitable today but undermines future correctness)
- **Recommendation**: Create a separate `AlertConfirmations` storage map: `StorageDoubleMap<AlertId, MeshtasticNodeId, bool>`.

**MESH-C3: `heartbeat_reactivates_inactive_node` increments `active_nodes` without tracking whether the node was already counted**  
- **Location**: L818–826 (`node_heartbeat`)
- **Issue**: When a node's `active` field is `false`, the heartbeat sets it to `true` and increments `stats.active_nodes`. However, `active_nodes` is **never decremented** anywhere in the codebase — not when heartbeat timeout expires, not when a node is deregistered. This means:
  1. `active_nodes` only ever increases.
  2. After one reactivation, calling heartbeat again (node already `active=true`) doesn't increment, but every time a node goes inactive and comes back, it increments an already-stale counter.
  3. There is no mechanism that ever sets `node.active = false` (no `on_finalize` or activity-check hook), so this code path is currently unreachable from on-chain logic, but `active_nodes` is still stale at 0 forever.
- **Impact**: `NetworkStats.active_nodes` is always wrong, providing incorrect data to any consuming pallet or UI.
- **Severity**: **CRITICAL** (data integrity)
- **Recommendation**: Either implement `on_idle`/`on_finalize` to scan for timed-out nodes and decrement `active_nodes`, or remove the `active_nodes` counter and compute it on-demand from `is_node_active()`.

---

#### WARNING

**MESH-W1: `NextAlertId` can overflow `u32::MAX` — no wrapping guard**  
- **Location**: L1109 (`NextAlertId::<T>::put(alert_id + 1)`)
- **Issue**: After 4.29 billion alerts, `alert_id + 1` overflows and wraps to 0, potentially overwriting existing alerts in the `EmergencyAlerts` map.
- **Impact**: Practically low probability but violates determinism guarantees.
- **Recommendation**: Use `alert_id.checked_add(1).ok_or(Error::<T>::MaxActiveAlertsExceeded)?` or use `saturating_add`.

**MESH-W2: `claim_relay_rewards` clears ALL relay proofs for owned nodes regardless of confirmation status**  
- **Location**: L1275–1278
- **Issue**: When an account claims rewards, the code does `RelayProofs::<T>::remove(node_id)` for *all* owned nodes. This removes **unconfirmed** proofs too — proofs that haven't been confirmed by a second node yet. Those proofs and potential future rewards are permanently lost.
- **Impact**: Economic loss for node operators. A user who submits 50 proofs but only 10 are confirmed loses 40 pending proofs when they claim.
- **Recommendation**: Only remove confirmed proofs; retain unconfirmed ones. Or separate confirmed/pending into distinct storage.

**MESH-W3: Per-block rate limit `MeshTxThisBlock` is reset in `on_initialize`, but not enforced for inherents**  
- **Location**: L218–221 (`on_initialize`), L860 (`submit_mesh_transaction`)
- **Issue**: The rate limit counter is reset each block via `MeshTxThisBlock::<T>::kill()`. The limit is only checked for signed extrinsics (normal mesh tx submissions). This is correct for the current design but the weight for `on_initialize` only accounts for 1,000 ref_time and 0 PoV, which is too low for a `kill()` operation that does a DB write.
- **Impact**: Minor weight undercount.
- **Recommendation**: Update `on_initialize` weight to include at least `T::DbWeight::get().writes(1)`.

**MESH-W4: `submit_mesh_transaction` does not validate that `relay_path` node IDs are registered nodes**  
- **Location**: L855–885 (`submit_mesh_transaction`)
- **Issue**: The `relay_path` is stored as-is. Any arbitrary 4-byte values can be stuffed in. While the path is informational, it could be used to claim fake relay credit if a separate cross-referencing system trusts this data.
- **Impact**: Low — the relay path is currently only stored, not used for reward distribution. But it creates a false audit trail.
- **Recommendation**: Consider validating that each node in `relay_path` exists in `MeshNodes`, or document that this field is unverified.

**MESH-W5: `confirm_relay_proof` does not check if confirmer's node is active**  
- **Location**: L1349–1350
- **Issue**: The confirmer must own at least one node (`!confirmer_nodes.is_empty()`) but there's no check that any of their nodes are `active`. A deregistered-but-not-cleaned-up or inactive node's owner can still confirm proofs.
- **Impact**: Weakens the relay proof verification model. An inactive node confirming a relay is semantically invalid.
- **Recommendation**: Check that at least one of `confirmer_nodes` is active.

**MESH-W6: `submit_mesh_transaction` checks `gateway.is_gateway` but error is `NodeNotFound` — misleading error**  
- **Location**: L847 (`ensure!(gateway.is_gateway, Error::<T>::NodeNotFound)`)
- **Issue**: When a registered node that isn't a gateway tries to submit a mesh transaction, it gets `NodeNotFound` instead of a descriptive error like `NotGatewayNode`.
- **Impact**: Poor developer/user experience, harder to debug.
- **Recommendation**: Add a dedicated `NotGatewayNode` error variant.

**MESH-W7: Relay reward for `Heartbeat`/`Confirmation` types uses integer division with potential reward of 0**  
- **Location**: L1380–1382 (`T::RelayRewardPerTransaction::get() / 10u32.saturated_into()`)
- **Issue**: If `RelayRewardPerTransaction` is less than 10 units, this division yields 0. In the mock config, the reward is `1_000_000_000`, so `/ 10 = 100_000_000` which is fine. But if a runtime configures a small reward, heartbeat relayers earn nothing.
- **Impact**: Economic misconfiguration risk.
- **Recommendation**: Add a minimum reward floor or use `max(result, 1)`.

**MESH-W8: `deregister_node` does not clean up `RelayProofs` for the deregistered node**  
- **Location**: L740–780 (`deregister_node`)
- **Issue**: When a node is deregistered, `MeshNodes`, `NodesByOwner`, and `NetworkStats` are updated. However, `RelayProofs::<T>` for that node_id are left orphaned. These can never be confirmed (node no longer exists) but still consume storage.
- **Impact**: Storage leak. Over time, orphaned proofs accumulate.
- **Recommendation**: Add `RelayProofs::<T>::remove(node_id);` to `deregister_node`.

**MESH-W9: No validation on `max_hops` in `update_mesh_config` — can be set to 0 or 255**  
- **Location**: L1294–1310 (`update_mesh_config`)
- **Issue**: `max_hops` is `u8` with no range check. Setting it to 0 would block all mesh transactions with any hops. Setting it to 255 effectively disables the hop limit.
- **Impact**: Governance misconfiguration can disable the mesh network or remove DoS protections.
- **Recommendation**: Enforce `1 <= max_hops <= 20` or similar reasonable bounds.

---

#### INFO

**MESH-I1: `register_node` does not validate GPS coordinates against Belize bounds**  
- **Location**: L693–696 — coordinates are NOT validated during registration (only during `update_node_location`)
- **Issue**: No coordinate validation in `register_node`, but it exists in `update_node_location`. Inconsistency.
- **Recommendation**: Apply the same coordinate validation from `update_node_location` to `register_node`.

**MESH-I2: `on_initialize` weight is underestimated**  
- **Location**: L220 — `Weight::from_parts(1_000, 0)`
- **Issue**: `MeshTxThisBlock::<T>::kill()` performs a storage write. The weight should include DB write cost.
- **Recommendation**: `T::DbWeight::get().writes(1)`.

**MESH-I3: `confirm_emergency_alert` doesn't check if the alert is resolved**  
- **Location**: L1163–1194
- **Issue**: Nodes can confirm a resolved alert. While this doesn't cause state corruption, it inflates `confirmations` on a dead alert.
- **Recommendation**: Add `ensure!(!alert.resolved, Error::<T>::AlertAlreadyResolved)`.

**MESH-I4: `relay_block_header` does not verify `parent_hash` against previously stored headers**  
- **Location**: L1226–1258
- **Issue**: The error `InvalidBlockHeader` exists (L623) but is never used. Block headers are stored without chain-of-custody verification. A malicious validator relay could submit a fabricated header.
- **Recommendation**: If a header for `block_number - 1` exists, verify `parent_hash` matches.

**MESH-I5: All arithmetic uses `saturating_add/sub` — GOOD**  
- **Location**: Throughout all extrinsics
- **Assessment**: No overflow bugs found. All counter increments use `.saturating_add(1)`, `.saturating_sub(1)`, and `.min(10000)` for reputation capping. This is correct.

**MESH-I6: No `MaxPendingMeshTx` enforcement**  
- **Location**: Config type declared at L191 but never referenced in `submit_mesh_transaction`
- **Issue**: `MaxPendingMeshTx` is declared but the total count of `PendingMeshTransactions` is never checked against it. The per-block `MaxMeshTxPerBlock` is enforced, but the total pending queue can grow without limit.
- **Recommendation**: Track a `PendingMeshTxCount` and enforce `MaxPendingMeshTx`.

**MESH-I7: `MaxActiveAlerts` declared but never enforced**  
- **Location**: Config type at L194, never used in `issue_emergency_alert`
- **Issue**: Same pattern as I6. The constant exists but no check enforces it.
- **Recommendation**: Add `ensure!(NextAlertId::<T>::get() - resolved_count < T::MaxActiveAlerts::get(), ...)` or use `ActiveAlertCountPerDistrict` sum.

**MESH-I8: `fund_relay_rewards` reuses `NoRelayRewards` error for zero-amount funding**  
- **Location**: L1325
- **Issue**: Semantically incorrect error name for this context. "No relay rewards" doesn't describe "attempted to fund zero amount."
- **Recommendation**: Add `ZeroAmountFunding` error or similar.

---

### 2. `types.rs` — Type Definitions (Score: 95/100)

| Finding | Severity | Location | Description |
|---------|----------|----------|-------------|
| None | — | — | All types implement `MaxEncodedLen`, `Encode`, `Decode`, `DecodeWithMemTracking`. Correctly bounded. |

- All `BoundedVec` uses have appropriate bounds (`ConstU32<8>` for relay path, `ConstU32<128>` for messages).
- `MeshNode` struct correctly uses fixed-size types (no `Vec` or unbounded collections).
- `MeshTransaction` uses `BoundedVec` for relay_path — no unbounded storage.
- All enums derive `Default` where appropriate.
- No floating point. All values are integers (`i32` for GPS × 1e7, `u32` for counters).

**INFO: `MeshBlockHeader.block_number` is `u32`, which limits to ~4 billion blocks. At 6s blocks, this is ~812 years, acceptable.**

---

### 3. `weights.rs` — Weight Definitions (Score: 70/100)

| Finding | Severity | Location | Description |
|---------|----------|----------|-------------|
| MESH-W-W1 | WARNING | All functions | Weights are hand-estimated, not benchmarked. Header comment acknowledges this. |
| MESH-W-W2 | WARNING | `register_node()` | Claims 3 reads, 3 writes. Actual: reads `MeshNodes` + `NodesByOwner` + `NetworkStats` + `MeshConfig` + `Currency::reserve` = 5 reads, writes `MeshNodes` + `NodesByOwner` + `NetworkStats` + reserve = 4 writes. **Under-weighted.** |
| MESH-W-W3 | WARNING | `submit_mesh_transaction()` | Claims 5 reads, 4 writes. Actual reads: `MeshNodes` + `MeshConfig` + `MeshTxThisBlock` + `PendingMeshTransactions` + `ProcessedMeshTransactions` + `NetworkStats` = 6 reads. Writes: `PendingMeshTransactions` + `MeshNodes` + `MeshTxThisBlock` + `NetworkStats` = 4 writes. **One read under-counted.** |
| MESH-W-W4 | INFO | `confirm_relay_proof()` | Claims 2 reads, 1 write. Actual: reads `MeshNodes` (relayer) + `NodesByOwner` (confirmer) + `RelayProofs` = 3 reads, writes `RelayProofs` + `RelayRewards` = 2 writes. **Under-weighted.** |

**Recommendation**: Run `frame-benchmarking` to generate accurate weights. Current under-weighting could allow block stuffing attacks where an attacker submits many underpriced mesh extrinsics.

---

### 4. `benchmarking.rs` — Benchmark Implementations (Score: 88/100)

| Finding | Severity | Location | Description |
|---------|----------|----------|-------------|
| MESH-B-I1 | INFO | `register_node` benchmark (L135) | Sets `min_kyc_for_registration: 0` to bypass KYC. This is correct for benchmarking but means the weight won't capture the cross-pallet call cost of `T::Identity::get_kyc_level`. |
| MESH-B-I2 | INFO | `resolve_emergency_alert` (L212) | Pre-creates alert with severity `Advisory`, which doesn't exercise the `CatastrophicAlertCount` mutation path. Should benchmark with `Catastrophic` severity too. |
| MESH-B-I3 | INFO | General | All 14 extrinsics are benchmarked — complete coverage. Good. |

---

### 5. `mock.rs` — Test Runtime (Score: 92/100)

| Finding | Severity | Location | Description |
|---------|----------|----------|-------------|
| MESH-M-I1 | INFO | L85–103 | `MockMeshIdentityProvider` hardcodes KYC levels per account. Default accounts get KYC level 1 (`_ => 1`). This is acceptable for testing but means unregistered accounts can pass KYC checks. |
| MESH-M-I2 | INFO | L115 | `EmergencyOrigin = EnsureRoot<u64>` — in production, this should be a more constrained origin. Mock is correct. |

---

### 6. `tests.rs` — Test Suite (Score: 85/100)

| Finding | Severity | Location | Description |
|---------|----------|----------|-------------|
| MESH-T-I1 | INFO | General | 45+ tests covering all 14 extrinsics. Error paths are well-tested. |
| MESH-T-W1 | WARNING | Missing | No test for `MaxMeshNodes` limit. The `MaxMeshNodesReached` error path is untested. |
| MESH-T-W2 | WARNING | Missing | No test verifying `on_initialize` resets `MeshTxThisBlock`. |
| MESH-T-W3 | WARNING | Missing | No test for per-block rate limit (`MeshTxRateLimitExceeded`). |
| MESH-T-I2 | INFO | Missing | No test for the `MeshTransactionBridge` and `EmergencyAlertProvider` trait implementations. |

---

## SECURITY CHECKLIST

| Category | Status | Notes |
|----------|--------|-------|
| **Access Control** | ✅ PASS | All extrinsics check `ensure_signed` + ownership. Governance/Emergency origins properly gated. |
| **Arithmetic Safety** | ✅ PASS | All operations use `saturating_add/sub/mul`. No `unwrap()` in production code. |
| **Storage Bounds** | ⚠️ PARTIAL | `BoundedVec` used for node-local collections, but `ProcessedMeshTransactions` and `PendingMeshTransactions` are unbounded `StorageMap`s. |
| **DoS Protection** | ⚠️ PARTIAL | Per-block rate limit exists, but `MaxPendingMeshTx` and `MaxActiveAlerts` are not enforced. |
| **Weight Correctness** | ⚠️ WARNING | Hand-estimated weights undercount reads/writes. Must benchmark. |
| **Determinism** | ✅ PASS | No floating point, no randomness, no off-chain dependencies. `H256` hashing for dedup is deterministic. |
| **Error Handling** | ✅ PASS | Comprehensive error enum. All paths return `Result`. No `unwrap()` or `expect()` in extrinsics. |
| **Event Emission** | ✅ PASS | All 14 extrinsics emit events. Events contain sufficient data for indexing. |
| **Cross-Pallet Safety** | ✅ PASS | `Currency::transfer` uses `KeepAlive`. `Currency::reserve` handles errors. `Identity` trait is read-only. |
| **Replay Protection** | ✅ PASS | Mesh transactions deduplicated by `tx_hash`. Alert confirmations deduplicated by `(alert_id, node_id)` hash. |
| **Token Economics** | ⚠️ PARTIAL | Relay rewards require 2-party confirmation (good anti-gaming). But reward claim clears unconfirmed proofs (W2). |

---

## FINDING PRIORITY MATRIX

| ID | Severity | Category | Extrinsic | Fix Effort |
|----|----------|----------|-----------|------------|
| MESH-C1 | CRITICAL | Storage Growth | N/A (systemic) | Medium |
| MESH-C2 | CRITICAL | Storage Design | `confirm_emergency_alert` | Medium |
| MESH-C3 | CRITICAL | Data Integrity | `node_heartbeat` | Medium |
| MESH-W1 | WARNING | Overflow | `issue_emergency_alert` | Low |
| MESH-W2 | WARNING | Economics | `claim_relay_rewards` | Medium |
| MESH-W3 | WARNING | Weight | `on_initialize` | Low |
| MESH-W4 | WARNING | Validation | `submit_mesh_transaction` | Low |
| MESH-W5 | WARNING | Access Control | `confirm_relay_proof` | Low |
| MESH-W6 | WARNING | Error UX | `submit_mesh_transaction` | Low |
| MESH-W7 | WARNING | Economics | `confirm_relay_proof` | Low |
| MESH-W8 | WARNING | Storage Leak | `deregister_node` | Low |
| MESH-W9 | WARNING | Validation | `update_mesh_config` | Low |

---

## PER-FILE SCORES

| File | Lines | Score | Key Issues |
|------|-------|-------|------------|
| `lib.rs` | ~1,515 | **74/100** | 3 CRITICAL (storage growth, dedup pollution, stale counter), 9 WARNING |
| `types.rs` | ~350 | **95/100** | Clean, well-bounded types. No issues. |
| `weights.rs` | ~120 | **70/100** | Hand-estimated, multiple under-weighted extrinsics |
| `benchmarking.rs` | ~310 | **88/100** | Complete coverage, minor KYC bypass concern |
| `mock.rs` | ~150 | **92/100** | Correct mock setup |
| `tests.rs` | ~1,000 | **85/100** | Good coverage, missing 3 critical error paths |

---

## RECOMMENDED FIXES (Priority Order)

### 1. [CRITICAL] Add `ProcessedMeshTransactions` pruning
```rust
// In on_idle or as a separate governance-callable extrinsic:
fn prune_processed_transactions(max_age_blocks: BlockNumberFor<T>) {
    let current = frame_system::Pallet::<T>::block_number();
    // Use a cursor-based iteration with bounded weight
}
```

### 2. [CRITICAL] Separate alert confirmation storage
```rust
#[pallet::storage]
pub type AlertConfirmations<T: Config> = StorageDoubleMap<
    _, Blake2_128Concat, u32, // alert_id
    Blake2_128Concat, MeshtasticNodeId,
    BlockNumberFor<T>, OptionQuery,
>;
```

### 3. [CRITICAL] Fix `active_nodes` tracking
Either remove `active_nodes` from `NetworkStats` and compute on-demand, or add an `on_idle` hook that scans nodes and updates the counter with bounded iteration.

### 4. [WARNING] Add missing bounds enforcement
```rust
// In issue_emergency_alert, add:
let active_count = ActiveAlertCountPerDistrict::<T>::get(&district);
ensure!(active_count < T::MaxActiveAlerts::get(), Error::<T>::MaxActiveAlertsExceeded);

// In submit_mesh_transaction, before inserting:
// Track PendingMeshTxCount and enforce MaxPendingMeshTx
```

### 5. [WARNING] Protect claim_relay_rewards from clearing unconfirmed proofs
```rust
// Replace blanket remove with selective cleanup:
for node_id in owned_nodes.iter() {
    RelayProofs::<T>::mutate(node_id, |proofs| {
        proofs.retain(|p| !p.confirmed);
    });
}
```

### 6. [WARNING] Run `frame-benchmarking` for accurate weights

---

## CONCLUSION

The mesh pallet demonstrates solid Substrate development practices: saturating arithmetic, bounded collections, comprehensive error handling, and proper access control. The 2-party relay proof confirmation model is a strong anti-gaming mechanism.

The three critical findings (unbounded storage growth, storage namespace pollution, and stale `active_nodes` counter) should be addressed before mainnet launch. The weight inaccuracies should be resolved by running benchmarks on reference hardware.

**Audit Status**: ⚠️ CONDITIONAL PASS — address 3 CRITICAL findings before production deployment.
