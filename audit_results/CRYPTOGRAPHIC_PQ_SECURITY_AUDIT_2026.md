# BelizeChain Cryptographic & Post-Quantum Security Audit

**Date**: 2026-07-14  
**Scope**: All cryptographic primitives, post-quantum signature schemes, hashing, key management, and identity privacy across pallets  
**Auditor**: AI Security Audit (comprehensive code review)  
**Severity Scale**: CRITICAL / HIGH / MEDIUM / LOW / INFORMATIONAL

---

## Executive Summary

BelizeChain implements ML-DSA-87 (NIST FIPS 204) via the `fips204` crate (v0.4.6) for post-quantum bridge security. The implementation in `pallets/interoperability` is **production-grade** with proper domain separation, canonical message binding, and exact-length guards. However, the `pallets/consensus` pallet has **critical gaps** where PQ signatures and public keys are accepted without any on-chain verification. The identity pallet uses salted blake2_256 for PII privacy — adequate for the current phase but **not true zero-knowledge**.

### Verdict by Pallet

| Pallet | PQ Crypto Status | Severity |
|--------|-----------------|----------|
| interoperability | **Production-ready** ML-DSA-87 | LOW residual risk |
| consensus | **CRITICAL**: PQ signatures accepted without verification | CRITICAL |
| quantum | No PQ crypto (by design) | INFORMATIONAL |
| identity | Salted hashing (not ZK) | MEDIUM |
| common | Standard blake2_256 | LOW |

---

## 1. Post-Quantum Signature System

### 1.1 Dependency: `fips204` Crate

**File**: [Cargo.toml](Cargo.toml#L121)  
```toml
fips204 = { version = "0.4.6", default-features = false, features = ["ml-dsa-87"] }
```

**Assessment**: 
- Uses ML-DSA-87 (NIST FIPS 204), the highest security level of the ML-DSA family
- `default-features = false` → `no_std` compatible for WASM runtimes ✓
- Only consumed by `pallets/interoperability/Cargo.toml` (line 26)
- **INFORMATIONAL**: The consensus pallet does NOT depend on `fips204`, which is why it cannot verify PQ signatures

### 1.2 PQSignatureVerifier Trait

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L55)

```rust
pub trait PQSignatureVerifier {
    fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}
```

**Assessment**: Clean trait abstraction. Pluggable via `Config::PQVerifier` associated type. ✓

### 1.3 PassthroughPQVerifier — TEST-ONLY BYPASS

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L80)

```rust
impl PQSignatureVerifier for PassthroughPQVerifier {
    fn verify(_public_key: &[u8], _message: &[u8], signature: &[u8]) -> bool {
        #[cfg(not(any(test, feature = "runtime-benchmarks")))]
        {
            panic!("PassthroughPQVerifier must NOT be used in production");
        }
        #[cfg(any(test, feature = "runtime-benchmarks"))]
        {
            signature.len() >= 64
        }
    }
}
```

| Finding | Severity |
|---------|----------|
| **PASS-01**: Production `panic!` guard prevents accidental deployment | LOW |
| **PASS-02**: Test mode accepts ANY signature ≥64 bytes (no cryptographic verification) | INFORMATIONAL (test-only) |

**Risk**: If the runtime is accidentally compiled with `runtime-benchmarks` feature in production, the passthrough would silently accept forged signatures. The `panic!` guard for non-test/non-benchmark builds mitigates this.

**Recommendation**: Add a compile-time `#[cfg_attr(not(test), deprecated)]` warning to make misuse more visible.

### 1.4 MLDsaVerifier — Production Verifier

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L120)

```rust
pub const PK_LEN: usize = 2592;   // ML-DSA-87 public key
pub const SIG_LEN: usize = 4627;  // ML-DSA-87 signature
pub const CTX: &[u8] = b"belizechain-bridge-v1"; // Domain separation

impl PQSignatureVerifier for MLDsaVerifier {
    fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        if public_key.len() != PK_LEN || signature.len() != SIG_LEN {
            return false;
        }
        let Ok(pk) = fips204::ml_dsa_87::PublicKey::try_from_bytes(...) else { return false };
        let Ok(sig) = fips204::ml_dsa_87::Signature::try_from_bytes(...) else { return false };
        pk.verify(message, &sig, CTX)
    }
}
```

**Positive Findings**:
- ✅ Exact-length guards for public key (2592) and signature (4627) — ML-DSA-87 correct sizes
- ✅ Domain separation context `b"belizechain-bridge-v1"` prevents cross-protocol replay
- ✅ `try_from_bytes` deserialization handles malformed inputs gracefully (returns `false`)
- ✅ Pure Rust, `no_std`/WASM-safe, zero C FFI
- ✅ No timing side channels (fips204 uses constant-time operations)

| Finding | Severity |
|---------|----------|
| **MLDSA-01**: Context string is hardcoded — no versioning mechanism for future upgrades | LOW |
| **MLDSA-02**: `fips204` v0.4.6 is not yet NIST-certified (implementation, not algorithm) | INFORMATIONAL |

---

## 2. Bridge PQ Signature Flow (Critical Path)

### 2.1 Canonical Message Construction (CONS-001 Fix)

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L960)

```rust
let canonical_message = match &bridge_tx.operation {
    BridgeOperation::LockAndMint { target_chain, target_address, amount, asset }
        => (tx_id, amount, target_chain, target_address.as_slice(), asset).encode(),
    BridgeOperation::BurnAndUnlock { source_chain, source_tx_hash, amount, asset, recipient }
        => (tx_id, amount, source_chain, source_tx_hash.as_slice(), asset, recipient.as_slice()).encode(),
    BridgeOperation::MessagePassing { target_chain, message_hash, .. }
        => (tx_id, target_chain, message_hash).encode(),
};
```

**Positive Findings**:
- ✅ **tx_id bound** prevents cross-transaction replay attacks
- ✅ **Amount + target_address + chain** bound prevents value substitution attacks
- ✅ SCALE encoding with `(tuple).encode()` provides unambiguous serialization
- ✅ Both LockAndMint and BurnAndUnlock include all value-critical fields

| Finding | Severity |
|---------|----------|
| **BRIDGE-01**: `MessagePassing` canonical message omits `payload` (only includes `message_hash`) — acceptable since hash commits to payload | INFORMATIONAL |
| **BRIDGE-02**: No nonce/timestamp in canonical message — replay within same tx_id is prevented by tx_id uniqueness, but the message itself is replayable if tx_id wraps (u32 overflow handled by `checked_add`) | LOW |

### 2.2 Signature Collection & Threshold

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L980)

```rust
ensure!(T::PQVerifier::verify(
    validator_record.pq_public_key.as_slice(),
    &canonical_message,
    &pq_signature,
), Error::<T>::InvalidPQSignature);

// M52 FIX: Prevent duplicate signatures from same validator
ensure!(!bridge_tx.pq_signatures.iter().any(|(v, _)| v == &who), Error::<T>::AlreadyExecuted);
```

**Positive Findings**:
- ✅ Each validator's PQ public key is loaded from on-chain storage (BridgeValidators)
- ✅ Duplicate signature prevention (M52)
- ✅ Signature bounded to `ConstU32<4627>` (exact ML-DSA-87 size)
- ✅ Maximum 32 validator signatures per transaction (`ConstU32<32>`)
- ✅ `pq_signatures_required` is per-chain configurable via `ChainConfig`

### 2.3 Challenge Period & Finalization

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1000)

Once threshold is met:
1. Status → `ReadyForExecution`
2. Registered in `PendingFinalizations` with expiry = current_block + `ChallengePeriod`
3. `on_idle` hook auto-finalizes after challenge period (max 20 per block)
4. Validators can `dispute_bridge_transaction` during challenge window

| Finding | Severity |
|---------|----------|
| **BRIDGE-03**: Dispute bond is 5% of `bridge_tx.fee` (not 5% of value) — for zero-fee incoming unlocks, the dispute bond is zero | MEDIUM |
| **BRIDGE-04**: `on_idle` finalization is bounded to 20 per block — if >20 transactions expire simultaneously, some are delayed (acceptable DoS mitigation) | LOW |

### 2.4 Validator Registration

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1389)

```rust
let pq_key: BoundedVec<u8, ConstU32<2592>> = pq_public_key
    .try_into()
    .map_err(|_| Error::<T>::InvalidPQSignature)?;
```

| Finding | Severity |
|---------|----------|
| **VAL-01**: PQ public key is bounded to 2592 bytes (ML-DSA-87 correct) ✓ | PASS |
| **VAL-02**: No on-chain validation that the submitted bytes are a valid ML-DSA-87 public key — just length check via `BoundedVec` | MEDIUM |
| **VAL-03**: Validator registration requires Level 3 KYC + 10× MinBridgeAmount stake ✓ | PASS |
| **VAL-04**: Sanctions screening on registration ✓ | PASS |

**Recommendation (VAL-02)**: Add `fips204::ml_dsa_87::PublicKey::try_from_bytes()` validation during `register_bridge_validator` to reject malformed keys at registration time rather than at signature verification time.

---

## 3. CRITICAL: Consensus Pallet PQ Gaps

### 3.1 AI Model Registration — Unverified PQ Signature

**File**: [pallets/consensus/src/lib.rs](pallets/consensus/src/lib.rs#L573-L579)

```rust
pub fn register_ai_model(
    ...
    pq_signature: Vec<u8>,
) -> DispatchResult {
    // SECURITY TODO(CONS-007): Apply MLDsaVerifier PQ signature verification
    // on `pq_signature` before model registration. Currently accepted without
    // on-chain verification — relies on off-chain AI authority.
    ...
    pq_signature: pq_signature.try_into().map_err(|_| Error::<T>::InvalidPQSignature)?,
```

| Finding | Severity |
|---------|----------|
| **CONS-007a**: PQ signature is stored but **never verified on-chain** | **CRITICAL** |
| **CONS-007b**: `BoundedVec<u8, ConstU32<256>>` is too small for ML-DSA-87 signatures (4627 bytes) — accepts any data ≤256 bytes | HIGH |
| **CONS-007c**: No public key reference — even if verification were added, there's no corresponding PQ public key to verify against | HIGH |

**Impact**: Any user can register an AI model with a fabricated "PQ signature". The signature field provides **zero authenticity guarantee**.

### 3.2 Consensus Validator — Unverified PQ Public Key

**File**: [pallets/consensus/src/lib.rs](pallets/consensus/src/lib.rs#L651-L656)

```rust
pub fn join_consensus_validator(
    ...
    pq_public_key: Vec<u8>,
) -> DispatchResult {
    // SECURITY TODO(CONS-007): Apply MLDsaVerifier PQ signature verification
    // on `pq_public_key` before validator registration.
```

| Finding | Severity |
|---------|----------|
| **CONS-007d**: PQ public key stored as `BoundedVec<u8, ConstU32<256>>` — too small for ML-DSA-87 (2592 bytes) | HIGH |
| **CONS-007e**: No validation that bytes constitute a valid PQ public key | HIGH |
| **CONS-007f**: The pq_public_key is stored but **never used** for any verification | **CRITICAL** |

**Impact**: Consensus validators can register with arbitrary PQ key material that is never validated or used. The entire consensus PQ infrastructure is a placeholder.

**Recommendation**: 
1. Add `fips204` dependency to `pallets/consensus/Cargo.toml`
2. Increase BoundedVec sizes to match ML-DSA-87 (PK: 2592, SIG: 4627)
3. Verify PQ signatures on model registration using the trainer's registered PQ public key
4. Require validators to prove possession of the private key (challenge-response or self-signed attestation)

---

## 4. Hashing Analysis

### 4.1 Blake2_256 Usage

| Location | Purpose | Assessment |
|----------|---------|------------|
| [identity/src/lib.rs](pallets/identity/src/lib.rs#L194) | `blake2_256(salt ++ plaintext)` for PII hashing | See §5 |
| [common/src/temporal_anchor.rs](pallets/common/src/temporal_anchor.rs#L186) | `blake2_256(&anchor.encode())` for anchor hashes | ✓ Correct usage |
| [common/src/temporal_anchor.rs](pallets/common/src/temporal_anchor.rs#L217) | Merkle tree construction (paired hashing) | ✓ Correct |
| [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1320) | `sp_core::blake2_256(payload)` for cross-chain message hash | ✓ Correct |
| All pallets (storage maps) | `Blake2_128Concat` for storage key hashing | ✓ Standard Substrate practice |

### 4.2 SHA-256 References

| Location | Purpose | Assessment |
|----------|---------|------------|
| [quantum/src/lib.rs](pallets/quantum/src/lib.rs) `circuit_hash: [u8; 32]` | Pre-computed circuit hash, accepted as-is | ⚠ No on-chain verification |
| [quantum/src/lib.rs](pallets/quantum/src/lib.rs) `result_data_hash: [u8; 32]` | Pre-computed result hash, accepted as-is | ⚠ No on-chain verification |

**Note**: These are accepted as opaque 32-byte identifiers. No SHA-256 computation occurs on-chain. The hashes serve as commitment references for off-chain data.

### 4.3 No Keccak/SHA-3 Usage

Confirmed via grep: No keccak256 or SHA-3 operations found in production pallet code. Substrate internally uses keccak for Ethereum compatibility, but BelizeChain pallets exclusively use Blake2.

---

## 5. Identity PII Privacy Model

### 5.1 Salted Hash Construction

**File**: [pallets/identity/src/lib.rs](pallets/identity/src/lib.rs#L194)

```
hash: H256  // = blake2_256(salt ++ plaintext)
salt: Option<BoundedVec<u8, ConstU32<64>>>  // ≥32 bytes required
```

| Finding | Severity |
|---------|----------|
| **ID-01**: Salt is stored on-chain alongside the hash | MEDIUM |
| **ID-02**: Salt minimum 32 bytes enforced (`SaltTooShort` error) ✓ | PASS |
| **ID-03**: SSN has ~10^9 possible values (9 digits) — with known salt, brute-force is trivial (~seconds) | MEDIUM |
| **ID-04**: Passport numbers have bounded input space — similarly vulnerable with known salt | MEDIUM |
| **ID-05**: Biometrics attestation uses `hash: H256::zero(), salt: None` — no PII content stored | PASS |
| **ID-06**: Reverse hash indexes (`SsnHashIndex`, `PassportHashIndex`) prevent duplicate PII across identities ✓ | PASS |
| **ID-07**: Code explicitly documents this is NOT true ZK — "True ZK selective disclosure...roadmapped for 2028" | INFORMATIONAL |

**Attack Scenario (ID-03)**: An attacker who reads the chain state can enumerate all 10^9 possible SSN values, compute `blake2_256(known_salt ++ candidate)`, and match against stored hashes. This reveals the actual SSN.

**Mitigation**: The salt provides protection against rainbow tables across different identities (each identity has a unique salt), but does NOT protect against targeted brute-force. This is acknowledged in the codebase as a known limitation.

### 5.2 KYC Level Computation

**File**: [pallets/identity/src/lib.rs](pallets/identity/src/lib.rs#L1060)

```rust
pub fn get_verified_kyc_level(who: &T::AccountId) -> Option<u8> {
    // On-chain attestations take priority over Oracle (M51 FIX)
    ...
    if let Some(oracle_level) = T::Oracle::get_kyc_level(who) { return Some(oracle_level); }
    Some(0)
}
```

| Finding | Severity |
|---------|----------|
| **KYC-01**: On-chain attestations correctly take priority over Oracle (M51 fix) ✓ | PASS |
| **KYC-02**: Oracle fallback could return a higher level than on-chain attestations warrant | MEDIUM |
| **KYC-03**: Grace period handling: expired-but-in-grace attestations allow continued operation ✓ | PASS |
| **KYC-04**: Sanctions check traverses ALL linked accounts (MaxAccountsPerIdentity=5) ✓ | PASS |

---

## 6. Quantum Pallet Cryptographic Analysis

### 6.1 Verification Proof

**File**: [pallets/quantum/src/lib.rs](pallets/quantum/src/lib.rs) — `record_quantum_result` extrinsic

```rust
ensure!(verification_proof.len() >= 32, Error::<T>::InvalidVerificationProof);
ensure!(result_data_hash != [0u8; 32], Error::<T>::InvalidResultHash);
```

| Finding | Severity |
|---------|----------|
| **QV-01**: `verification_proof` is a "computation commitment hash" — NOT a ZK proof or cryptographic proof | INFORMATIONAL |
| **QV-02**: Only structural validation (≥32 bytes, non-zero hash) — no cryptographic verification | MEDIUM |
| **QV-03**: Multi-validator consensus (Phase 2.3) provides social/economic verification, not cryptographic | INFORMATIONAL |

### 6.2 NFT Bridge Security

**File**: [pallets/quantum/src/lib.rs](pallets/quantum/src/lib.rs#L1530) — `bridge_to_ethereum`

| Finding | Severity |
|---------|----------|
| **NFT-01**: Bridge locks NFT by setting `transferable = false` — prevents double-spend ✓ | PASS |
| **NFT-02**: No PQ signatures required for NFT bridge (unlike the interoperability bridge) | MEDIUM |
| **NFT-03**: Ethereum address validation: accepts 20 OR 42 bytes (raw or hex-encoded with `0x`) | LOW |
| **NFT-04**: Cancel bridge restores transferability ✓ | PASS |
| **NFT-05**: XCM bridge to parachains has a TODO comment — XCM message not actually sent | INFORMATIONAL |

---

## 7. Cross-Chain Security Controls

### 7.1 Rate Limiting

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1540)

```rust
fn check_bridge_rate_limit(who: &T::AccountId) -> DispatchResult {
    // CONS-025: Clear ALL per-account counters on new block
    let _ = BridgeCallsThisBlock::<T>::clear(u32::MAX, None);
    // Validate against MaxBridgePerBlock
    ensure!(count <= T::MaxBridgePerBlock::get(), Error::<T>::RateLimitExceeded);
}
```

| Finding | Severity |
|---------|----------|
| **RL-01**: Per-account, per-block rate limiting ✓ | PASS |
| **RL-02**: `BridgeCallsThisBlock::clear(u32::MAX, None)` clears ALL accounts on new block (CONS-025 fix) ✓ | PASS |

### 7.2 Escrow Model (CONS-027)

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L855)

```rust
let escrow = Self::escrow_account();
T::Currency::transfer(&who, &escrow, net_amount, ExistenceRequirement::KeepAlive)?;
```

| Finding | Severity |
|---------|----------|
| **ESC-01**: PalletId-derived escrow supports cross-user unlocks ✓ | PASS |
| **ESC-02**: `UserBridgeLocks` tracking is accounting only — actual funds are in escrow account | LOW |

### 7.3 Burn Proof Verification (CONS-003)

**File**: [pallets/interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1240)

```rust
Self::deposit_event(Event::UnverifiedBurnProofWarning { tx_id, submitted_by: who });
```

| Finding | Severity |
|---------|----------|
| **BURN-01**: No on-chain verification of external chain burn events — relies entirely on validator attestation + multi-sig + challenge period | HIGH |
| **BURN-02**: Warning event emitted (CONS-003 acknowledgment) | INFORMATIONAL |

**Mitigation**: The multi-sig threshold + challenge period + dispute mechanism provides economic security guarantees rather than cryptographic proof. This is the standard approach for cross-chain bridges without light client verification.

---

## 8. Key Management Summary

| Key Type | Size | Storage | Validation | Used For |
|----------|------|---------|------------|----------|
| ML-DSA-87 Public Key (bridge) | 2592 bytes | `BridgeValidators` map | Length check only (VAL-02) | Bridge PQ verification ✓ |
| ML-DSA-87 Signature (bridge) | 4627 bytes | `BridgeTransaction.pq_signatures` | Full MLDsaVerifier ✓ | Bridge multi-sig |
| PQ Signature (consensus) | ≤256 bytes | `AIModel.pq_signature` | **NONE** (CONS-007) | ❌ Never verified |
| PQ Public Key (consensus) | ≤256 bytes | `ConsensusValidator.pq_public_key` | **NONE** (CONS-007) | ❌ Never used |
| PII Hash Salt | 32-64 bytes | `Attestation.salt` | ≥32 bytes ✓ | Identity privacy |
| Circuit Hash | 32 bytes | `QuantumJob.circuit_hash` | None (opaque) | Off-chain reference |
| Result Hash | 32 bytes | `QuantumResult.result_data_hash` | Non-zero ✓ | Off-chain reference |

---

## 9. Findings Summary

### CRITICAL (2)

| ID | Location | Description |
|----|----------|-------------|
| CONS-007a | [consensus/src/lib.rs](pallets/consensus/src/lib.rs#L579) | AI model PQ signature stored but never verified on-chain |
| CONS-007f | [consensus/src/lib.rs](pallets/consensus/src/lib.rs#L656) | Consensus validator PQ public key stored but never used |

### HIGH (4)

| ID | Location | Description |
|----|----------|-------------|
| CONS-007b | [consensus/src/lib.rs](pallets/consensus/src/lib.rs#L177) | PQ signature BoundedVec ≤256 bytes vs ML-DSA-87's 4627 |
| CONS-007c | [consensus/src/lib.rs](pallets/consensus/src/lib.rs#L573) | No public key reference for model PQ signature verification |
| CONS-007d | [consensus/src/lib.rs](pallets/consensus/src/lib.rs#L225) | PQ public key BoundedVec ≤256 bytes vs ML-DSA-87's 2592 |
| BURN-01 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1240) | No on-chain verification of external chain burn events |

### MEDIUM (6)

| ID | Location | Description |
|----|----------|-------------|
| VAL-02 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1425) | No PK validity check at validator registration |
| BRIDGE-03 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L1355) | Zero dispute bond on zero-fee transactions |
| ID-01 | [identity/src/lib.rs](pallets/identity/src/lib.rs#L194) | Salt stored on-chain alongside hash |
| ID-03 | [identity/src/lib.rs](pallets/identity/src/lib.rs#L194) | SSN brute-force trivial with known salt (~10^9 space) |
| KYC-02 | [identity/src/lib.rs](pallets/identity/src/lib.rs#L1070) | Oracle can override on-chain KYC levels upward |
| NFT-02 | [quantum/src/lib.rs](pallets/quantum/src/lib.rs#L1530) | NFT bridge has no PQ signature requirement |

### LOW (6)

| ID | Location | Description |
|----|----------|-------------|
| PASS-01 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L80) | PassthroughPQVerifier prod `panic!` guard |
| MLDSA-01 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L120) | Hardcoded domain separation context |
| BRIDGE-02 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L960) | No nonce/timestamp in canonical message |
| BRIDGE-04 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs) | on_idle bounded to 20 finalizations/block |
| NFT-03 | [quantum/src/lib.rs](pallets/quantum/src/lib.rs#L1560) | Ethereum address accepts 20 OR 42 bytes |
| ESC-02 | [interoperability/src/lib.rs](pallets/interoperability/src/lib.rs#L870) | UserBridgeLocks is accounting-only |

---

## 10. Recommendations (Priority Order)

1. **[CRITICAL] Implement CONS-007**: Add `fips204` dependency to consensus pallet, increase BoundedVec sizes to ML-DSA-87 dimensions, and verify PQ signatures during model registration and validator join. This is the single highest-priority security gap.

2. **[HIGH] Light client or SPV verification for burn proofs**: Implement at minimum a Merkle proof verification for external chain transactions, rather than relying solely on validator attestation.

3. **[MEDIUM] Validate PQ public keys at registration**: Call `fips204::ml_dsa_87::PublicKey::try_from_bytes()` in `register_bridge_validator` to reject malformed keys early.

4. **[MEDIUM] Dispute bond for zero-fee transactions**: Set a minimum dispute bond floor (e.g., `MinBridgeAmount`) regardless of transaction fee to prevent costless dispute spam on incoming unlocks.

5. **[MEDIUM] Identity ZK migration**: The 2028 roadmap for true ZK selective disclosure should be accelerated given that SSN brute-force is trivially achievable with on-chain salt access.

6. **[LOW] Domain separation versioning**: Include a version byte in the PQ signature context to allow future upgrades without breaking existing signatures.

---

## 11. Positive Security Patterns

The codebase demonstrates several strong security practices:

- **Saturating arithmetic everywhere** — no integer overflow vulnerabilities
- **`checked_add` for ID counters** with `IdOverflow` error handling
- **`BoundedVec` for all user inputs** — prevents unbounded storage attacks
- **Escrow-based bridge model** (CONS-027) instead of balance locks
- **Canonical message binding** (CONS-001) prevents cross-transaction replay
- **Duplicate signature prevention** (M52) on bridge transactions
- **Multi-layer KYC** with on-chain priority over Oracle (M51)
- **Rate limiting** on bridge operations (per-account, per-block)
- **Challenge period + dispute mechanism** for bridge finalization
- **No `unwrap()` in production code** — consistent `Result<T, Error>` usage
- **Explicit TODO comments** for known security gaps (CONS-007)
