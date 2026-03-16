# BelizeChain Identity Pallet — Security Audit Report

**Date**: March 15, 2026  
**Auditor**: AI Security Auditor (Rust/Substrate specialist)  
**Scope**: `pallets/identity/src/` — all 5 `.rs` files (~1,200 LOC in `lib.rs`, ~200 LOC `tests.rs`, ~65 LOC `weights.rs`, ~160 LOC `benchmarking.rs`, ~200 LOC `mock.rs`)  
**Pallet Purpose**: Decentralized identity (DID), KYC attestations (SSN, Passport, Biometrics), issuer management, reputation, and cross-pallet KYC verification trait.

---

## Executive Summary

| Severity | Count |
|----------|-------|
| **CRITICAL** | 3 |
| **WARNING** | 10 |
| **INFO** | 8 |

**Overall Pallet Score: 72/100**

The identity pallet demonstrates competent Substrate development with important privacy-first design choices (salted hashes, no PII on-chain) and meaningful hardening (rate limiting, issuer bonds, flagging, hash deduplication). However, several critical gaps remain around identity lifecycle management, oracle trust boundaries, and weight accuracy that must be addressed before mainnet launch.

---

## Per-File Scores

| File | Score | Notes |
|------|-------|-------|
| `lib.rs` | 68/100 | Core logic; 3 CRITICAL, 7 WARNING |
| `weights.rs` | 55/100 | Hand-estimated weights, severe undercount on attestation ops |
| `benchmarking.rs` | 75/100 | Covers happy paths; missing edge-case benchmarks |
| `mock.rs` | 85/100 | Well-structured; oracle mock is trivially permissive |
| `tests.rs` | 80/100 | Good coverage (~40 tests); missing adversarial test vectors |

---

## CRITICAL Findings

### C-01: No Identity Deregistration or Account Unlinking — Permanent Storage Bloat & Identity Lock-in
**File**: [lib.rs](pallets/identity/src/lib.rs#L561-L1011)  
**Lines**: Entire `#[pallet::call]` block (call_index 0–19)  
**Severity**: CRITICAL  

There is **no extrinsic to delete an identity or unlink an account**. Once registered, an identity record, all its attestations, and all linked account→identity mappings persist forever.

**Impact**:
- **Storage bloat**: Every identity ever created remains in `Identities`, `IdentityOf`, all attestation maps, and all history maps permanently. With no cleanup, storage grows monotonically.
- **Identity lock-in**: If a private key is compromised, the victim cannot deregister and re-register. The compromised key's `IdentityOf` mapping remains, and the attacker retains a linked account to the identity.
- **Account recycling impossible**: A linked account cannot be removed from one identity and registered under another, even by governance.
- **Architectural doc mismatch**: The architectural reconstruction report at line 242 references `unlink_account` (call_index 7) as a planned extrinsic, but it does not exist in the pallet.

**Recommendation**:
```rust
// Add call_index(20): deregister_identity (governance-only)
// - Removes Identities, IdentityOf for all linked accounts
// - Removes all attestation records and hash indexes
// - Clears attribute history

// Add call_index(21): unlink_account (owner-only)
// - Removes IdentityOf for the target account
// - Removes from identity record's accounts vec
// - Must not allow unlinking the primary (owner) account
```

---

### C-02: Oracle Can Bypass On-Chain KYC — Trust Boundary Violation
**File**: [lib.rs](pallets/identity/src/lib.rs#L1076-L1095)  
**Lines**: 1076–1095 (`meets_kyc_requirement_level`)  

Despite the `M51 FIX` comment claiming on-chain verification takes priority, the function **falls through to the Oracle** when on-chain KYC is invalid (expired, suspended, or revoked):

```rust
// Line 1089: Fallback to Oracle as supplementary verification
if T::Oracle::meets_kyc_requirement(who, required_level) {
    return true;
}
```

**Impact**:
- A compromised or malicious oracle can grant KYC status to **any account**, completely bypassing the legitimately revoked or expired on-chain attestation.
- If governance revokes an identity's SSN attestation for fraud, the oracle can still declare them KYC-compliant.
- The `is_kyc_verified` trait method (line 1020–1042) correctly checks **only** on-chain state, but cross-pallet consumers calling `meets_kyc_requirement_level` instead get the oracle fallback — inconsistent security guarantees.

**Recommendation**:
```rust
pub fn meets_kyc_requirement_level(who: &T::AccountId, required_level: u8) -> bool {
    let now = frame_system::Pallet::<T>::block_number();
    let kyc_level = match required_level { ... };
    // On-chain check only — no oracle fallback
    matches!(Self::kyc_state(who, kyc_level, now), KycState::Valid | KycState::Grace)
    // Oracle should only be used for INITIAL attestation issuance, never runtime bypass
}
```

---

### C-03: Biometric Attestation Hash is Always `H256::zero()` — No Uniqueness Enforcement
**File**: [lib.rs](pallets/identity/src/lib.rs#L868-L890)  
**Lines**: 868–890 (`issue_biometrics`)

```rust
// Line 880
hash: H256::zero(),
```

Unlike SSN and Passport, biometric attestations:
1. Store `H256::zero()` as the hash — all biometric attestations have the same hash
2. Have **no hash deduplication index** (no `BiometricHashIndex`)
3. Accept only an anchor CID — the pallet cannot verify uniqueness

**Impact**:
- The same biometric data could be claimed by multiple identities
- An issuer can issue biometric attestations to unlimited identities with no proof of uniqueness
- KYC L3 (which requires biometrics) provides weaker guarantees than L2

**Recommendation**:
- Add a `hash: H256` parameter to `issue_biometrics` (like SSN/Passport)
- Add `BiometricHashIndex` storage map for uniqueness enforcement
- Require issuers to compute `blake2_256(biometric_template)` off-chain

---

## WARNING Findings

### W-01: `AttributeType::from(u8)` Maps All Unknown Values to `Biometrics`
**File**: [lib.rs](pallets/identity/src/lib.rs#L139-L143)  
**Lines**: 139–143

```rust
impl From<u8> for AttributeType {
    fn from(v: u8) -> Self {
        match v { 0 => AttributeType::Ssn, 1 => AttributeType::Passport, _ => AttributeType::Biometrics }
    }
}
```

Any `attr` value ≥ 2 is silently treated as `Biometrics`. This affects `revoke`, `suspend`, `add_issuer`, `remove_issuer`, `set_standard_version`, `flag_issuer`, `slash_issuer_bond`, and `report_bad_attestation`.

**Impact**: A governance call to `revoke(account, attr=255)` would revoke the biometric attestation, not error. An admin calling `add_issuer(attr=50, issuer)` would unknowingly add a biometric issuer.

**Recommendation**: Return `Result<AttributeType, Error>` or add validation:
```rust
fn try_from(v: u8) -> Result<Self, ()> {
    match v { 0 => Ok(Ssn), 1 => Ok(Passport), 2 => Ok(Biometrics), _ => Err(()) }
}
```

---

### W-02: `add_blocks` Helper Uses `unwrap_or` Fallback — Silent Truncation
**File**: [lib.rs](pallets/identity/src/lib.rs#L497-L506)  
**Lines**: 497–506

```rust
fn add_blocks(a: BlockNumberFor<T>, b: BlockNumberFor<T>) -> BlockNumberFor<T> {
    let a_u64: u64 = TryInto::<u64>::try_into(a).unwrap_or(0);
    let b_u64: u64 = TryInto::<u64>::try_into(b).unwrap_or(0);
    let sum = a_u64.saturating_add(b_u64);
    sum.try_into().unwrap_or(a)  // Falls back to `a` if conversion fails
}
```

If `BlockNumberFor<T>` is larger than u64 (e.g., u128), the conversion silently returns 0, producing incorrect validity windows. The `unwrap_or(a)` on the return path silently ignores conversion failure.

**Impact**: On runtimes with non-u64 block numbers, attestation validity windows will be incorrect, potentially making attestations immediately valid-forever or immediately invalid.

**Recommendation**: Use Substrate's `saturating_add` directly on `BlockNumberFor<T>` if it implements `sp_runtime::traits::Saturating`, or bound the config.

---

### W-03: Weights Are Hand-Estimated — Severely Undercount Attestation Operations
**File**: [weights.rs](pallets/identity/src/weights.rs#L1-L65)  
**Lines**: All

The file explicitly states: `THIS FILE WAS NOT AUTO-GENERATED. These are conservative estimates.`

`issue_attestation()` claims 3 reads and 3 writes, but actual `issue_ssn` performs:
- **Reads**: `FlaggedIssuers`, `GlobalPaused`, `RateWindowBlocks`, `RateMaxPerWindowSsn`, `IssuerRate`, `IdentityOf`, `SsnHashIndex`, `SsnStandardVersion`, `KycValidityBlocks`, `KycGraceBlocks`, `SsnAttestations` (for stale hash cleanup), `OperationFee` (if fee check precedes) — **up to 12 reads**.
- **Writes**: `IssuerRate`, `SsnAttestations`, `SsnHashIndex`, `SsnHashIndex::remove` (stale), `AttributeHistory` — **up to 5 writes**.

**Impact**: Block weight accounting is off by ~4x on reads, enabling block stuffing attacks where an attacker fills blocks with attestation extrinsics that cost less weight than they actually consume.

**Recommendation**: Run `frame-benchmarking` and generate accurate weights. This is a pre-mainnet blocker.

---

### W-04: No Re-attestation (Update) Flow — Issuer Must Overwrite Silently
**File**: [lib.rs](pallets/identity/src/lib.rs#L744-L810)  
**Lines**: 744–810 (`issue_ssn`)

When a new SSN attestation is issued for an identity that already has one, the old attestation is silently overwritten:
```rust
// Line 798-801: Remove stale hash index
if let Some(old_att) = SsnAttestations::<T>::get(id) {
    if old_att.hash != hash {
        SsnHashIndex::<T>::remove(old_att.hash);
    }
}
SsnAttestations::<T>::insert(id, att);
```

There is no `AlreadyAttested` check for re-issuance, unlike what the error variant `AlreadyAttested` (line 434) suggests. This means **any authorized issuer can silently replace another issuer's attestation**.

**Impact**: Issuer A attests Alice's SSN. Issuer B (also authorized) can overwrite it with a different hash, anchor, and format_ok, without Alice's consent and without audit trail beyond the generic history append.

**Recommendation**:
- Add an explicit `update_ssn` extrinsic with separate call_index 
- Require the original issuer OR governance to update
- Or require explicit revocation-then-reissue flow

---

### W-05: `issuer_withdraw_bond` — Pallet Account May Die (KeepAlive Transfer)
**File**: [lib.rs](pallets/identity/src/lib.rs#L925-L938)  
**Lines**: 925–938

```rust
T::Currency::transfer(&Self::account_id(), &issuer, amount, ExistenceRequirement::KeepAlive)?;
```

If the pallet's sovereign account balance drops below `ExistentialDeposit` after the transfer, the `KeepAlive` flag causes the transaction to fail. But the code has no fallback — the issuer's bond becomes permanently locked.

Additionally, if multiple issuers' bonds are the only funds in the pallet account, withdrawals will fail for the last issuer because the account would be reaped.

**Impact**: Bond recovery failure for the last issuer to withdraw. The test file (`tests.rs` line 892) works around this by seeding the pallet account with `deposit_creating`, confirming awareness of the issue but not fixing it in production.

**Recommendation**: Seed the pallet account during genesis with a minimum buffer above existential deposit, or use `AllowDeath` and handle account reaping gracefully.

---

### W-06: No Pause Check on `update_did_doc`
**File**: [lib.rs](pallets/identity/src/lib.rs#L614-L628)  
**Lines**: 614–628

`register_identity` and all `issue_*` extrinsics check `ensure_not_paused()`, but `update_did_doc` does not. During an emergency pause (e.g., active exploit), DID documents can still be modified.

**Impact**: An attacker could update DID document anchors during a security incident, potentially redirecting off-chain DID resolution to malicious documents.

**Recommendation**: Add `Self::ensure_not_paused()?;` at the start of `update_did_doc`.

---

### W-07: No Sanctioned-Account Check on Identity Registration or Attestation Issuance
**File**: [lib.rs](pallets/identity/src/lib.rs#L561-L575)  
**Lines**: 561–575 (`register_identity`), 744–810 (`issue_ssn`)

`is_account_sanctioned` exists as a read-only helper but is never enforced:
- A sanctioned account can register an identity
- An issuer can issue attestations to a sanctioned account
- No attestation extrinsic checks `is_account_sanctioned(&target)`

**Impact**: Sanctions enforcement is entirely dependent on cross-pallet consumers calling `is_account_sanctioned` — the identity pallet itself permits full lifecycle operations for sanctioned accounts.

**Recommendation**: Add `ensure!(!Self::is_account_sanctioned(&who), Error::<T>::Sanctioned)` to `register_identity`, and `ensure!(!Self::is_account_sanctioned(&target), ...)` to all `issue_*` extrinsics.

---

### W-08: `report_bad_attestation` Performs Flag + Slash but Not Revocation
**File**: [lib.rs](pallets/identity/src/lib.rs#L970-L1011)  
**Lines**: 970–1011

When governance reports a bad attestation, the issuer is flagged and optionally slashed, but the **attestation itself remains Active**. Victims of the bad attestation retain valid KYC status.

**Impact**: A fraudulent SSN attestation continues to grant the target account valid KYC L1+ status even after the issuer is flagged and slashed.

**Recommendation**: `report_bad_attestation` should accept an identity/account parameter and automatically revoke all attestations issued by the flagged issuer, or at minimum revoke the specific attestation referenced.

---

### W-09: `append_history` O(n) Shift on BoundedVec Overflow
**File**: [lib.rs](pallets/identity/src/lib.rs#L518-L530)  
**Lines**: 518–530

```rust
if hist.try_push(evt.clone()).is_err() {
    if !hist.is_empty() {
        let _ = hist.remove(0); // O(n) shift
        let _ = hist.try_push(evt);
    }
}
```

When history reaches `MaxHistoryLen`, every new event triggers `remove(0)` which shifts all remaining elements left — `O(n)` operation on each attestation lifecycle event.

**Impact**: With `MaxHistoryLen=50`, this is 50 element copies per history append at capacity. Not a DoS vector at current bounds, but inefficient.

**Recommendation**: Use a ring buffer approach (track head index) instead of shifting, or use `swap_remove(0)` + sort, or accept the bounded O(n) cost with a comment.

---

### W-10: `get_verified_kyc_level` Always Returns `Some(0)` — Never `None`
**File**: [lib.rs](pallets/identity/src/lib.rs#L1055-L1074)  
**Lines**: 1055–1074

```rust
pub fn get_verified_kyc_level(who: &T::AccountId) -> Option<u8> {
    // ... checks ...
    Some(0) // L0 - no verification
}
```

The return type is `Option<u8>` but `None` is never returned. An unregistered account without any identity still returns `Some(0)`.

**Impact**: Callers cannot distinguish "no identity exists" from "identity exists but has no KYC." Any cross-pallet code pattern-matching on `None` for "unknown account" will never trigger.

**Recommendation**: Return `None` for accounts with no `IdentityOf` mapping.

---

## INFO Findings

### I-01: `salt` Field Deprecated but Still in Storage Schema
**File**: [lib.rs](pallets/identity/src/lib.rs#L180-L184)  
**Lines**: 180–184

```rust
/// DEPRECATED: Salt removed (P0-01 fix). ...
/// Kept as Option for storage compatibility; always None for new attestations.
pub salt: Option<BoundedVec<u8, ConstU32<64>>>,
```

The field occupies codec space in every attestation record. Consider a storage migration to remove it and reduce per-attestation storage cost by ~66 bytes.

---

### I-02: `SaltTooShort` Error Variant is Dead Code
**File**: [lib.rs](pallets/identity/src/lib.rs#L449)  
**Line**: 449

The `SaltTooShort` error is defined but never used anywhere in the pallet. It was part of the pre-P0-01 flow that accepted on-chain salts.

---

### I-03: Benchmark `admin_simple` Uses `RawOrigin::Root` — May Not Reflect Real Governance Weight
**File**: [benchmarking.rs](pallets/identity/src/benchmarking.rs#L75-L82)  
**Lines**: 75–82

`AdminOrigin` in production may be a multi-sig council or a democracy pallet, not `Root`. The benchmark measures the extrinsic weight only with root origin, which may differ from real governance dispatch overhead.

---

### I-04: `is_authorized_issuer` Calls `.into_inner()` Creating Unnecessary Allocation  
**File**: [lib.rs](pallets/identity/src/lib.rs#L465-L471)  
**Lines**: 465–471

```rust
fn is_authorized_issuer(attr: AttributeType, who: &T::AccountId) -> bool {
    match attr {
        AttributeType::Ssn => SsnIssuers::<T>::get().into_inner().contains(who),
```

`.into_inner()` converts `BoundedVec` to `Vec`, allocating on the heap. Use `.as_slice().contains(who)` or `.iter().any(|x| x == who)` to avoid the allocation.

---

### I-05: `issue_ssn` Contains Dead Code Check
**File**: [lib.rs](pallets/identity/src/lib.rs#L796)  
**Line**: 796

```rust
if PassportAttestations::<T>::contains_key(id) { /* no-op */ }
```

This line reads `PassportAttestations` storage but does nothing with the result. Likely a leftover from development. Costs one unnecessary DB read per SSN attestation.

---

### I-06: No `#[pallet::compact]` on Balance/BlockNumber Extrinsic Parameters
**File**: [lib.rs](pallets/identity/src/lib.rs)  
**Lines**: Various extrinsics

Extrinsics like `set_operation_fee(fee: BalanceOf<T>)`, `set_rate_limits(window, ssn, passport, biometrics)`, `slash_issuer_bond(amount)` don't use `#[pallet::compact]` encoding on numeric parameters. This wastes a few bytes per extrinsic call in block encoding.

---

### I-07: Test Coverage Gaps — Missing Adversarial Vectors
**File**: [tests.rs](pallets/identity/src/tests.rs)

Missing test vectors:
- Linked account attempting `link_account` (tests only primary owner)
- Re-attestation (overwriting existing attestation with new issuer)
- Revoked attestation + re-issuance flow
- Suspended → Active transition (no `unsuspend` extrinsic exists!)
- `get_verified_kyc_level` with oracle returning conflicting level
- `is_account_sanctioned` with linked account sanctions
- DID document update from linked (non-primary) account

---

### I-08: No `unsuspend` Extrinsic — Suspended Attestations Are Permanent
**File**: [lib.rs](pallets/identity/src/lib.rs#L848-L860)  
**Lines**: 848–860

Once an attestation is suspended via `suspend()`, there is no extrinsic to lift the suspension. The only recovery path is re-issuance by an authorized issuer (which overwrites the attestation per W-04), making suspension functionally equivalent to revocation.

**Recommendation**: Add a `reinstate` or `unsuspend` extrinsic gated by `RevokeOrigin`.

---

## Cross-Pallet Security Analysis

### Consumers:
1. **LandLedger pallet** — Uses `BelizeKyc::is_kyc_verified` trait (safe — on-chain only, no oracle fallback)
2. **Compliance pallet** — Expected to use `meets_kyc_requirement_level` (WARNING: includes oracle fallback per C-02)
3. **All pallets** via runtime config — `BelizeKyc` trait is the safe interface; direct `Pallet::<T>::` calls may hit the oracle-fallback functions

### Trust Boundary:
```
Safe:       BelizeKyc::is_kyc_verified()      → on-chain only
DANGEROUS:  Pallet::meets_kyc_requirement_level() → oracle fallback
DANGEROUS:  Pallet::get_verified_kyc_level()      → oracle fallback
Safe:       Pallet::is_account_sanctioned()      → oracle, but read-only advisory
```

---

## Summary of Recommendations (Priority Order)

| # | Finding | Priority | Effort |
|---|---------|----------|--------|
| C-01 | Add `deregister_identity` + `unlink_account` extrinsics | P0 | High |
| C-02 | Remove oracle fallback from `meets_kyc_requirement_level` | P0 | Low |
| C-03 | Add hash + uniqueness index for biometric attestations | P0 | Medium |
| W-03 | Generate benchmarked weights via `frame-benchmarking` | P0 | Medium |
| W-07 | Add sanctions check to registration + attestation issuance | P1 | Low |
| W-01 | Replace `From<u8>` with fallible `TryFrom` + validation | P1 | Low |
| W-04 | Add explicit re-attestation flow with issuer authorization | P1 | Medium |
| W-06 | Add pause check to `update_did_doc` | P1 | Low |
| W-08 | Auto-revoke attestations on `report_bad_attestation` | P1 | Medium |
| W-05 | Seed pallet account or handle KeepAlive edge case | P2 | Low |
| W-10 | Return `None` for unregistered accounts in `get_verified_kyc_level` | P2 | Low |
| I-08 | Add `unsuspend`/`reinstate` extrinsic | P2 | Low |
| W-02 | Use native `saturating_add` on block numbers | P2 | Low |
| W-09 | Optimize `append_history` ring buffer | P3 | Low |
| I-05 | Remove dead `PassportAttestations` read in `issue_ssn` | P3 | Trivial |
| I-02 | Remove dead `SaltTooShort` error variant | P3 | Trivial |
| I-04 | Avoid `.into_inner()` allocation in `is_authorized_issuer` | P3 | Trivial |

---

*Audit complete. 3 CRITICAL, 10 WARNING, 8 INFO findings across 5 files. Overall score: 72/100.*
