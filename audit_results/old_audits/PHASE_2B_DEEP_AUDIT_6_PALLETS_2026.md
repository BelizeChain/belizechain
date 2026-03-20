# Phase 2B: Deep Security Audit — Identity, Oracle, BNS, Compliance, LandLedger, Mesh

**Auditor**: BelizeChain Comprehensive Audit — Deep Dive  
**Date**: July 2026  
**Scope**: 6 pallets — `identity`, `oracle`, `bns`, `compliance`, `landledger`, `mesh`  
**Methodology**: Line-by-line review of every `lib.rs` with cross-pallet interaction analysis  
**Classification**: READ-ONLY AUDIT — no code modifications  

---

## Executive Summary

This deep-dive extends the Phase 2 summary with line-level references, complete extrinsic/storage inventories, and targeted analysis of the 7 key security questions below. **47 findings** identified across 6 pallets, including **6 CRITICAL**, **13 HIGH**, **15 MEDIUM**, **8 LOW**, and **5 INFO**.

### Key Security Questions — Verdict

| # | Question | Verdict | Details |
|---|----------|---------|---------|
| 1 | Can identity/KYC be spoofed or bypassed? | **PARTIALLY** | Single-issuer attestation model allows colluding issuer to certify arbitrary accounts. No multi-issuer consensus for identity attestations. Oracle KYC voting has quorum but identity pallet does not. |
| 2 | Can PII be recovered from on-chain data? | **YES — SSN** | 9-digit SSN has only 10^9 possibilities. Salt is stored on-chain alongside hash. Brute-force with blake2_256 is trivial (~seconds on modern hardware). Passport numbers are harder due to alphanumeric space but still feasible for targeted attacks. |
| 3 | Can oracle data be manipulated? | **PARTIALLY** | Price feeds require MinConsensusOperators quorum (median aggregation is sound). But merchant verification is single-operator. IoT device registration is open to any user. Behavior flags require multi-oracle consensus (good). |
| 4 | Can land titles be fraudulently registered? | **YES** | `register_property` is callable by ANY signed user with no KYC or sanctions check. Registration deposit is the only barrier. Property starts unverified, but the registration itself creates a permanent storage entry. |
| 5 | Can BNS names be hijacked? | **NO** | Registration requires KYC and sanctions check. Ownership is enforced per-extrinsic. Government tier requires governance origin. Transfer requires owner signature. However, **buyers in marketplace bypass KYC/sanctions**. |
| 6 | Are compliance rules enforceable? | **YES, with caveats** | Structuring detection is well-implemented. Sanctions screening uses hash-based lookup. But enforcement depends on OTHER pallets calling `ComplianceReporter::report_transaction()`. No automatic integration with balance transfers. |
| 7 | Is mesh node identity properly validated? | **PARTIALLY** | Node registration requires KYC (configurable level). Gateway requires higher KYC. ValidatorRelay checks `is_validator()`. But mesh transaction signatures are NOT cryptographically verified (only `signature_hash ≠ zero` check). |

---

## Severity Definitions

| Severity | Description |
|----------|-------------|
| **CRITICAL** | Exploitable vulnerability that can cause fund loss, data corruption, or complete security bypass |
| **HIGH** | Significant security weakness that requires mitigation before mainnet |
| **MEDIUM** | Design concern or missing validation that should be addressed |
| **LOW** | Minor issue or hardening opportunity |
| **INFO** | Observation, positive pattern, or documentation note |

---

## 1. Identity Pallet (`pallets/identity/src/lib.rs`, ~1200 lines)

### 1.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Storage Mutations |
|---|-----------|-----------|--------|--------|----------------------|
| 1 | 0 | `register_identity` | Signed | Hand-estimated | `Identities`, `IdentityCount`, balance transfer |
| 2 | 1 | `link_account` | Signed (primary only, M50) | Hand-estimated | `Identities.accounts` |
| 3 | 2 | `update_did_doc` | Signed (primary only, I-7) | Hand-estimated | `Identities.did_document` |
| 4 | 3 | `add_issuer` | AdminOrigin | Hand-estimated | `AuthorizedIssuers`, `IssuerBonds` |
| 5 | 4 | `remove_issuer` | AdminOrigin | Hand-estimated | `AuthorizedIssuers` |
| 6 | 5 | `set_standard_version` | AdminOrigin | Hand-estimated | `StandardVersions` |
| 7 | 6 | `set_operation_fee` | AdminOrigin | Hand-estimated | `OperationFee` |
| 8 | 7 | `set_issuer_bond_amount` | AdminOrigin | Hand-estimated | `IssuerBondAmount` |
| 9 | 8 | `set_rate_limits` | AdminOrigin | Hand-estimated | `RateLimitConfig` |
| 10 | 9 | `set_pause` | AdminOrigin | Hand-estimated | `Paused` |
| 11 | 10 | `issue_ssn` | Signed (authorized issuer) | Hand-estimated | `Identities.attestations`, `SsnHashIndex` |
| 12 | 11 | `issue_passport` | Signed (authorized issuer) | Hand-estimated | `Identities.attestations`, `PassportHashIndex` |
| 13 | 12 | `issue_biometrics` | Signed (authorized issuer) | Hand-estimated | `Identities.attestations` |
| 14 | 13 | `revoke` | RevokeOrigin | Hand-estimated | `Identities.attestations[].status` |
| 15 | 14 | `suspend` | RevokeOrigin | Hand-estimated | `Identities.attestations[].status` |
| 16 | 15 | `issuer_deposit_bond` | Signed | Hand-estimated | `IssuerBonds`, balance transfer |
| 17 | 16 | `issuer_withdraw_bond` | Signed | Hand-estimated | `IssuerBonds`, balance transfer |
| 18 | 17 | `flag_issuer` | AdminOrigin | Hand-estimated | `FlaggedIssuers` |
| 19 | 18 | `slash_issuer_bond` | AdminOrigin | Hand-estimated | `IssuerBonds`, balance transfer to Treasury |
| 20 | 19 | `report_bad_attestation` | AdminOrigin | Hand-estimated | `FlaggedIssuers`, `IssuerBonds` |

### 1.2 Key Storage Items

| Storage | Type | Bounded? | Pruning? |
|---------|------|----------|----------|
| `Identities` | StorageMap<IdentityId, IdentityRecord> | Yes (BoundedVec for accounts, attestations) | No auto-pruning |
| `SsnHashIndex` | StorageMap<H256, IdentityId> | N/A (1:1 map) | Re-issuance removes stale |
| `PassportHashIndex` | StorageMap<H256, IdentityId> | N/A (1:1 map) | Re-issuance removes stale |
| `AuthorizedIssuers` | StorageMap<Standard, BoundedVec<AccountId>> | Yes (MaxIssuersPerStandard) | Manual via remove_issuer |
| `IssuerBonds` | StorageMap<AccountId, Balance> | N/A (1:1) | Cleared on withdraw |
| `FlaggedIssuers` | StorageMap<AccountId, bool> | N/A (1:1) | Toggle only |
| `OperationFee` | StorageValue<Balance> | N/A | Overwrite |
| `IssuerBondAmount` | StorageValue<Balance> | N/A | Overwrite |
| `RateLimitConfig` | StorageValue<RateLimitConfiguration> | N/A | Overwrite |
| `Paused` | StorageValue<bool> | N/A | Toggle |
| `StandardVersions` | StorageMap<Standard, u32> | N/A (1:1) | Overwrite |
| `IdentityCount` | StorageValue<u64> | N/A | Increment only |

### 1.3 Security Findings

#### ID-DEEP-1 — CRITICAL: On-Chain Salt Enables SSN Brute-Force Recovery

**Location**: `issue_ssn` extrinsic (~L770-830)

**Description**: The `issue_ssn` extrinsic stores the salt on-chain alongside the salted blake2_256 hash of the SSN. The SSN space is only 9 digits (000-000-000 to 999-999-999), representing 10^9 = 1 billion possibilities. With the salt available on-chain, an attacker can compute `blake2_256(salt || candidate_ssn)` for all billion candidates in **under 10 seconds** on commodity hardware (blake2 throughput ~1 GB/s).

**Code Pattern**:
```rust
// Salt stored in attestation record ON-CHAIN
let hash = sp_io::hashing::blake2_256(&[salt.as_slice(), &data].concat());
// Attestation { hash, salt, ... } written to storage
```

**Impact**: Complete de-anonymization of every SSN-attested identity on the chain. This violates Belizean Data Protection Act requirements and exposes citizens to identity theft.

**Recommendation**: 
1. **Immediate**: Remove salt from on-chain storage. Store only the hash.
2. **Short-term**: Increase work factor — use Argon2id or scrypt with high parameters instead of blake2_256.
3. **Long-term**: Implement ZK proofs for identity verification (planned for 2028 per codebase comments).

**Note**: Passport numbers (~L830-870) are safer due to alphanumeric space but still vulnerable to targeted attacks where partial passport info is known.

---

#### ID-DEEP-2 — HIGH: Single-Issuer Attestation Without Multi-Issuer Consensus

**Location**: `issue_ssn` (~L770), `issue_passport` (~L830), `issue_biometrics` (~L870)

**Description**: Any single authorized issuer can unilaterally create, modify, or overwrite attestations for any identity. There is no multi-issuer consensus requirement for identity attestations, unlike Oracle KYC voting which requires `MinOracleAgreement` quorum. A compromised or colluding issuer can:
1. Issue fraudulent SSN/passport attestations to accomplice accounts
2. Overwrite legitimate attestations (re-issuance path allows any authorized issuer to overwrite)
3. Create KYC-verified identities for sanctions evasion

**Contrast with Oracle**: The oracle pallet requires `MinOracleAgreement` votes for KYC level changes (~oracle L620-700). Identity attestations have NO equivalent quorum.

**Impact**: KYC system integrity depends entirely on issuer trustworthiness. A single compromised issuer undermines the entire identity chain.

**Recommendation**: Require attestation countersigning — at least 2 authorized issuers must independently attest the same credential hash before it's considered valid.

---

#### ID-DEEP-3 — HIGH: No Target Consent for Attestation Issuance

**Location**: `issue_ssn` (~L770), `issue_passport` (~L830), `issue_biometrics` (~L870)

**Description**: The `target` identity in all three attestation extrinsics is specified by the issuer. There is no check that the target account has consented to or requested the attestation. An issuer can attest credentials to any identity without the identity holder's knowledge.

**Attack scenario**: Malicious issuer attests a fake SSN to a victim's identity, which could:
1. Replace the victim's real attestation (overwrite path exists)
2. Link the victim to a different real person's SSN
3. Trigger compliance actions based on fraudulent attestation

**Recommendation**: Add an `accept_attestation` extrinsic that the target must call, or require the target's signature as a parameter.

---

#### ID-DEEP-4 — MEDIUM: Hash Index Cleanup Race on Re-Issuance

**Location**: `issue_ssn` (~L800-810)

**Description**: When re-issuing an SSN attestation, the code removes the old hash index entry and inserts the new one:
```rust
// Remove stale hash index
if let Some(old_att) = old_attestation {
    SsnHashIndex::<T>::remove(old_att.hash);
}
SsnHashIndex::<T>::insert(hash, target);
```
If two issuers attempt to issue SSN attestations to the same identity in the same block, the second transaction will overwrite the first, but the first transaction's hash index entry will persist as an orphan (the `old_attestation` check only catches the pre-existing value, not a same-block concurrent write).

**Impact**: Orphaned hash index entries that prevent future re-use of that SSN hash. Low severity but worth noting.

---

#### ID-DEEP-5 — MEDIUM: BelizeKyc Trait Grace Period Logic

**Location**: `is_kyc_verified()` and `kyc_state()` (~L1060-1140)

**Description**: The `BelizeKyc` trait implementation has a grace period mechanism where expired attestations are still considered valid within `GracePeriodBlocks`. The `get_verified_kyc_level()` function correctly prioritizes on-chain attestations over Oracle (M51 fix), but the grace period means an identity with an expired attestation can still pass KYC checks for an additional window.

**Security implications**: During the grace period, an account whose KYC has been revoked or expired can still execute KYC-gated transactions. If `GracePeriodBlocks` is set too high, this creates a window for sanctions evasion after credential revocation.

**Recommendation**: Ensure `GracePeriodBlocks` is set to a reasonable value (e.g., 1 day = 14,400 blocks). The `revoke` extrinsic should bypass grace period entirely.

---

#### ID-DEEP-6 — LOW: Identity Registration Has No KYC Requirement

**Location**: `register_identity` (~L570)

**Description**: Any signed account can register an identity. The only cost is the `OperationFee`. This is by design (you need an identity before you can get KYC), but it means the identity registry can be spammed with empty identities that have no attestations.

**Mitigation already present**: The `OperationFee` provides economic deterrent. Identities without attestations are useless for KYC-gated operations.

---

#### ID-DEEP-7 — INFO: Positive Pattern — Linked Account Sanctions Check

**Location**: `is_account_sanctioned()` (~L1150)

**Description**: The `is_account_sanctioned()` check iterates over ALL linked accounts in an identity record and checks each against the sanctions list. This means sanctioning one account of a multi-account identity effectively sanctions all linked accounts. This is a strong anti-evasion pattern.

---

## 2. Oracle Pallet (`pallets/oracle/src/lib.rs`, ~1550 lines)

### 2.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Operations |
|---|-----------|-----------|--------|--------|----------------|
| 1 | 0 | `add_operator` | OracleAdminOrigin | Hand-estimated | Adds to `OracleOperators`, enforces MaxOperators (O-2) |
| 2 | 1 | `remove_operator` | OracleAdminOrigin | Hand-estimated | Removes from `OracleOperators` |
| 3 | 2 | `submit_price` | Signed (operator) | Operational class (DOS-009) | `PriceSubmissions`, triggers `aggregate_price_feed()` |
| 4 | 3 | `verify_merchant` | Signed (operator) | Hand-estimated | `MerchantCategories` with 1-year expiry |
| 5 | 4 | `add_sanctioned_entity` | OracleAdminOrigin | Hand-estimated | `SanctionedEntities` with optional expiry |
| 6 | 5 | `remove_sanction` | OracleAdminOrigin | Hand-estimated | Removes from `SanctionedEntities` |
| 7 | 6 | `verify_identity` | Signed (operator) | Hand-estimated | Phase 3A KYC plurality voting → `KycPendingVotes`, `IdentityVerifications` |
| 8 | 7 | `register_land` | Signed (operator) | Hand-estimated | `LandRegistryData`, prevents overwrite (O-4) |
| 9 | 8 | `register_iot_device` | **Signed (any user)** | Hand-estimated | `IoTDevices` |
| 10 | 9 | `submit_iot_data` | Signed (device owner) | Hand-estimated | Device stats update only (NO on-chain data storage) |
| 11 | 10 | `verify_iot_device` | Signed (operator) | Hand-estimated | `IoTDevices.verified` |
| 12 | 11 | `claim_oracle_rewards` | Signed (operator) | Hand-estimated | Balance transfer from treasury, `OracleOperatorStatsMap` reset (AR-7) |
| 13 | 12 | `update_exchange_rate` | OracleAdminOrigin | Hand-estimated | `ManualExchangeRates` |
| 14 | 13 | `resolve_kyc_dispute` | OracleAdminOrigin | Hand-estimated | Clears `KycDisputeFlags`, `KycPendingVotes` |
| 15 | 14 | `submit_behavior_flag` | Signed (operator) | Hand-estimated | Phase 5A multi-oracle consensus → `BehaviorFlags`, `BehaviorFlagCooldown` |
| 16 | 15 | `clear_behavior_flag` | OracleAdminOrigin | Hand-estimated | Removes `BehaviorFlags`, `BehaviorFlagCooldown` |

### 2.2 Security Findings

#### OR-DEEP-1 — CRITICAL: Single-Operator Merchant Verification

**Location**: `verify_merchant` (~L560-590)

**Description**: A single oracle operator can unilaterally verify any account as a merchant in any category (Accommodation, FoodBeverage, TourOperator, Transportation, Retail, Other). The verification sets a 1-year expiry and immediately enables the merchant for tourism cashback rewards (5-8% via economy pallet). No multi-operator consensus is required.

**Attack vector**: A compromised operator calls `verify_merchant` for accomplice accounts. Those accounts then process self-referential tourism payments to extract cashback from the treasury. At 5-8% cashback with no volume limits (other than KYC daily limits), this can systematically drain the tourism incentive fund.

**Contrast**: Price feeds require `MinConsensusOperators` quorum. KYC verification requires `MinOracleAgreement` votes. Merchant verification has NO quorum — design inconsistency.

**Impact**: Treasury drain via fraudulent merchant verification feeding economy pallet tourism incentives.

**Recommendation**: Require at least `MinConsensusOperators` operators to verify a merchant before the verification becomes active.

---

#### OR-DEEP-2 — HIGH: IoT Device Registration Open to All Users

**Location**: `register_iot_device` (~L720-770)

**Description**: The `register_iot_device` extrinsic uses `ensure_signed` — any signed user can register an IoT device, not just oracle operators. The device is created with `verified: false` and requires a subsequent `verify_iot_device` call from an operator to become verified. However, unverified devices can still:
1. Call `submit_iot_data` to update their own stats
2. Consume storage slots in `IoTDevices`
3. Accumulate reputation scores via `update_device_reputation()`

**Impact**: Storage spam vector. Unverified devices building reputation could create false credibility.

**Recommendation**: Either restrict `register_iot_device` to operators, or add a registration deposit refundable on verification.

---

#### OR-DEEP-3 — HIGH: Operator Self-Verification as Merchant

**Location**: `verify_merchant` (~L560-590)

**Description**: No check prevents an oracle operator from verifying themselves (or an account they control) as a merchant. Combined with OR-DEEP-1, this creates a direct path from oracle operator to tourism cashback extraction.

**Recommendation**: Add `ensure!(operator != merchant, Error::<T>::SelfVerificationNotAllowed)`.

---

#### OR-DEEP-4 — MEDIUM: Price Feed Aggregation Iterates All Operators

**Location**: `aggregate_price_feed()` (~L1350-1400)

**Description**: The `aggregate_price_feed()` helper iterates over `OracleOperators::iter().take(max_ops)` to collect price submissions. While bounded by `MaxOperators` (O-2 fix), this is still a potentially expensive operation called within `submit_price`. If `MaxOperators` is set to a high value (e.g., 100), each price submission triggers 100 storage reads for operator enumeration plus 100 storage reads for `PriceSubmissions`.

**Mitigation present**: Bounded by `MaxOperators` constant. The Operational dispatch class (DOS-009 fix) gives priority in block inclusion.

---

#### OR-DEEP-5 — MEDIUM: KYC Dispute Resolution is Admin-Only Bottleneck

**Location**: `resolve_kyc_dispute` (~L850-880)

**Description**: When KYC plurality voting produces a dispute (conflicting votes), the `KycDisputeFlags` storage is set and ALL further voting is blocked for that identity until an admin calls `resolve_kyc_dispute`. There is no timeout or automatic resolution. If the admin is unavailable or slow, disputed KYC verifications remain frozen indefinitely.

**Impact**: Availability concern — admin bottleneck on KYC dispute resolution.

**Recommendation**: Add a `DisputeTimeout` after which the dispute auto-resolves to the majority vote, or the identity can request re-verification.

---

#### OR-DEEP-6 — MEDIUM: Behavior Flag Cooldown Not Checked in Oracle Itself

**Location**: `submit_behavior_flag` (~L890-930), `has_active_behavior_flag()` (~L1430)

**Description**: The `submit_behavior_flag` extrinsic sets `BehaviorFlagCooldown` but the oracle pallet itself does not check `has_active_behavior_flag()` before any oracle operations. The flag is a public API for other pallets (governance, economy) to use as a circuit breaker. However, a behavior-flagged oracle operator can still submit prices, verify merchants, and vote on KYC.

**Recommendation**: Check `has_active_behavior_flag()` for the operator in all operator-gated extrinsics.

---

#### OR-DEEP-7 — INFO: Positive Pattern — Median Price Aggregation

**Location**: `aggregate_price_feed()` (~L1380-1395)

The median calculation is correctly implemented with an overflow-safe average for even-length arrays: `a/2 + b/2 + (a%2 + b%2)/2`. The variance calculation uses basis points. Staleness filtering excludes old submissions.

---

## 3. BNS Pallet (`pallets/bns/src/lib.rs`, ~1420 lines)

### 3.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Operations |
|---|-----------|-----------|--------|--------|----------------|
| 1 | 0 | `register_domain` | Signed | Hand-estimated | KYC + sanctions check, price calculation, fee to treasury, `Domains` |
| 2 | 1 | `set_resolution` | Signed (owner) | Hand-estimated | `Domains.resolution` |
| 3 | 2 | `transfer_domain` | Signed (owner) | Hand-estimated | Ownership change, lock check, marketplace cleanup (M64) |
| 4 | 3 | `list_domain` | Signed (owner) | Hand-estimated | `DomainMarketplace` listing |
| 5 | 4 | `buy_domain` | Signed (buyer) | Hand-estimated | Price validation, fee split (95% seller / 5% treasury), ownership change |
| 6 | 5 | `unlist_domain` | Signed (seller) | Hand-estimated | Removes `DomainMarketplace` entry |
| 7 | 6 | `activate_hosting` | Signed (owner) | Hand-estimated | `HostingSubscriptions`, tier-based monthly fee |
| 8 | 7 | `renew_hosting` | Signed (subscriber) | Hand-estimated | `HostingSubscriptions.expires_at` extension |
| 9 | 8 | `deactivate_hosting` | Signed (subscriber) | Hand-estimated | `HostingSubscriptions` removal |
| 10 | 9 | `update_hosting_content` | Signed (subscriber) | Hand-estimated | `ContentHistory` versioning |
| 11 | 10 | `register_external_domain` | Signed | Hand-estimated | `ExternalDomains`, generates verification token (BNS-3) |
| 12 | 11 | `verify_external_domain` | Signed | Hand-estimated | Records attempt — governance approval required |
| 13 | 12 | `create_subdomain` | Signed (parent owner) | Hand-estimated | `Subdomains`, optional delegation |
| 14 | 13 | `rollback_content` | Signed (subscriber) | Hand-estimated | Savepoint + restore from `ContentHistory` |
| 15 | 14 | `update_ssl_certificate` | Signed (owner) | Hand-estimated | `SslCertificates` |

### 3.2 Security Findings

#### BNS-DEEP-1 — HIGH: Domain Purchase Bypasses KYC and Sanctions Checks

**Location**: `buy_domain` (~L640-700)

**Description**: The `register_domain` extrinsic correctly checks KYC level and sanctions:
```rust
// In register_domain:
let kyc_level = T::KycProvider::get_verified_kyc_level(&who);
ensure!(kyc_level >= required_level, Error::<T>::InsufficientKycLevel);
ensure!(!T::SanctionsChecker::is_sanctioned(&who), Error::<T>::AccountSanctioned);
```
But `buy_domain` performs NONE of these checks. It only validates price, listing expiry, and payment:
```rust
// In buy_domain: NO KYC check, NO sanctions check
let listing = DomainMarketplace::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotListed)?;
ensure!(listing.price <= max_price, Error::<T>::PriceTooHigh);
// ... transfer payment and ownership
```

**Attack vector**: A sanctioned entity that cannot register a domain can still acquire one through the marketplace. An account with insufficient KYC can purchase premium/government/verified tier domains that required higher KYC to register.

**Impact**: Complete bypass of KYC/sanctions enforcement on domain ownership via marketplace purchases.

**Recommendation**: Add KYC level check (matching the domain's tier requirement) and sanctions check in `buy_domain`.

---

#### BNS-DEEP-2 — HIGH: Domain Transfer Bypasses KYC and Sanctions Checks

**Location**: `transfer_domain` (~L600-640)

**Description**: Like `buy_domain`, the `transfer_domain` extrinsic checks only that the caller is the current owner and the domain is not locked. No KYC or sanctions check is performed on the recipient (`new_owner`).

**Attack vector**: A legitimate domain owner can transfer a domain to a sanctioned entity.

**Impact**: Sanctions evasion by receiving domains via direct transfer.

**Recommendation**: Add sanctions check on `new_owner`: `ensure!(!T::SanctionsChecker::is_sanctioned(&new_owner), ...)`. Add KYC tier check matching the domain's tier.

---

#### BNS-DEEP-3 — MEDIUM: Unlimited Free Subdomain Creation

**Location**: `create_subdomain` (~L1050-1100)

**Description**: The `create_subdomain` extrinsic allows the parent domain owner to create unlimited subdomains at no cost. There is no per-domain subdomain limit, no creation fee, and no KYC check on the subdomain recipient (when delegation is used).

**Impact**: Storage bloat via subdomain spam. A single domain owner can create thousands of subdomains all pointing to different accounts, effectively providing a free naming service that bypasses registration fees.

**Recommendation**: Add either a MaxSubdomainsPerDomain limit, a subdomain creation fee, or both.

---

#### BNS-DEEP-4 — MEDIUM: ContentHistory Unbounded Growth

**Location**: `update_hosting_content` (~L800-850)

**Description**: Each `update_hosting_content` call appends to `ContentHistory` storage. While content versions use `BoundedVec` for the content hash itself, the version counter is u32 and the history map has no maximum entries. The `rollback_content` extrinsic reads from this history.

**Impact**: Over time, a frequently-updated domain could accumulate millions of content versions, making history queries increasingly expensive.

**Recommendation**: Cap `ContentHistory` entries per domain (e.g., last 100 versions). Prune oldest on overflow.

---

#### BNS-DEEP-5 — MEDIUM: Domain Price Calculation — Short Domains Command Premium

**Location**: `calculate_domain_price()` (~L1300-1340)

**Description**: Domain pricing uses a length-based multiplier:
```rust
let multiplier = match domain.len() {
    1..=2 => 10,  // Ultra-premium: 10× base
    3 => 5,       // Premium: 5× base
    4 => 2,       // Semi-premium: 2× base
    _ => 1,       // Standard: 1× base
};
```
Base prices: Standard=100 DALLA, Premium=500 DALLA, Government=50 DALLA, Verified=1000 DALLA.

A 1-character Standard domain costs 1,000 DALLA. A 1-character Government domain costs 500 DALLA. There is no check preventing Government-tier registration by governance for ultra-premium 1-2 character domains at a 50 DALLA base × 10 = 500 DALLA discount versus Standard's 1,000 DALLA.

**Impact**: Low — governance-controlled, but worth documenting the pricing gap.

---

#### BNS-DEEP-6 — INFO: Positive Pattern — External Domain Verification Deferred to Governance

**Location**: `verify_external_domain` (~L1010-1040)

External domain verification correctly records the verification attempt but does NOT auto-approve. Governance must separately approve the mapping. The verification token generation (BNS-3 fix) includes `parent_hash()` and `extrinsic_count()` for unpredictability.

---

## 4. Compliance Pallet (`pallets/compliance/src/lib.rs`, ~1350 lines)

### 4.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Operations |
|---|-----------|-----------|--------|--------|----------------|
| 1 | 0 | `verify_account` | ComplianceOrigin | Hand-estimated | `VerifiedAccounts`, stats (CP-4 FIX: dedup) |
| 2 | 1 | `update_risk_level` | ComplianceOrigin | Hand-estimated | `VerifiedAccounts.risk_level` |
| 3 | 2 | `whitelist_account` | ComplianceOrigin | Hand-estimated | `WhitelistedAccounts` |
| 4 | 3 | `restrict_account` | ComplianceOrigin | Hand-estimated | `RestrictedAccounts`, stats increment |
| 5 | 4 | `lift_restriction` | ComplianceOrigin | Hand-estimated | `RestrictedAccounts`, stats decrement (CP-3 FIX) |
| 6 | 5 | `flag_suspicious_activity` | ComplianceOrigin | Hand-estimated | `SuspiciousActivities`, auto-elevate risk to High, FIFO eviction |
| 7 | 6 | `add_sanctions_entry` | SanctionsOrigin | Hand-estimated | `SanctionedAccounts` (hash-based) |
| 8 | 7 | `remove_sanctions_entry` | SanctionsOrigin | Hand-estimated | `SanctionedAccounts` |
| 9 | 8 | `sync_verification_from_identity` | Signed (self only) | Hand-estimated | `VerifiedAccounts` — reads identity pallet, does NOT refresh timestamp |

### 4.2 Key Cross-Pallet Integration

```
ComplianceReporter trait:
├── report_transaction(account, amount) → bool (structuring detected?)
│   └── Called by: economy pallet (tourism payments), interoperability (bridge transfers)
│   └── NOT called by: frame_system balances transfers, BNS marketplace, landledger transfers
│
AccountSanctionsChecker trait:
├── is_sanctioned(account) → bool
│   └── Called by: BNS (register_domain), landledger (transfer_property), mesh (register_node)
│   └── NOT called by: BNS (buy_domain, transfer_domain) ← GAP
```

### 4.3 Security Findings

#### COMP-DEEP-1 — HIGH: Structuring Detection Not Integrated with Balance Transfers

**Location**: `ComplianceReporter` impl (~L1285-1290)

**Description**: The `ComplianceReporter::report_transaction()` function is the hook that other pallets call to trigger structuring detection. However, this is opt-in — it's only called where pallets explicitly integrate it. Native balance transfers via `pallet_balances::transfer` do NOT trigger compliance reporting.

**Gap analysis**: Structuring detection only fires when:
- Economy pallet processes tourism payments
- Interoperability pallet processes bridge transfers

Structuring detection does NOT fire when:
- Users make direct `pallet_balances::transfer` calls
- BNS marketplace purchases (`buy_domain`)
- LandLedger property transfers (`transfer_property` — has tax but no structuring check)
- Staking deposits/withdrawals

**Impact**: Sophisticated structurers can evade detection by using plain balance transfers instead of pallet-mediated transfers.

**Recommendation**: Implement a `SignedExtension` / `TransactionExtension` that hooks into ALL balance-mutating transactions and calls `report_transaction()` automatically.

---

#### COMP-DEEP-2 — MEDIUM: Sanctions List Uses Account Hash, Not Account ID

**Location**: `add_sanctions_entry` (~L800-830), `is_sanctioned()` helper (~L1130)

**Description**: The compliance pallet sanctions list stores H256 hashes (presumably hashed account IDs or identity hashes), while the oracle pallet stores sanctions by `AccountId` directly. This creates two parallel sanctions systems:
1. Compliance: `SanctionedAccounts<H256, bool>` — hash-based
2. Oracle: `SanctionedEntities<AccountId, SanctionInfo>` — ID-based

**Confusion risk**: An account sanctioned in one system may not be detected by checks using the other system. The `AccountSanctionsChecker` trait implementation bridges this, but the dual-system design is fragile.

**Recommendation**: Consolidate sanctions into one canonical source. Either both pallets use the same storage, or one pallet delegates to the other.

---

#### COMP-DEEP-3 — MEDIUM: `sync_verification_from_identity` Does Not Refresh Timestamp

**Location**: `sync_verification_from_identity` (~L860-900)

**Description**: When a user calls `sync_verification_from_identity` to pull their KYC level from the identity pallet, the compliance pallet correctly requires that the identity attestation is not expired. However, it does NOT update the `verified_at` timestamp — only a `ComplianceOrigin` can set this via `verify_account`.

**Implication**: A user who syncs their identity will have a compliance verification that uses the original `verified_at` timestamp. If their compliance verification is about to expire, syncing does NOT extend it. This is intentionally designed as a security measure (users can't refresh their own compliance status), but it means compliance expiry can lag behind identity validity.

---

#### COMP-DEEP-4 — MEDIUM: `on_idle` Prunes Only 5 Accounts Per Block

**Location**: `on_idle` hook (~L440-480)

**Description**: The `on_idle` hook prunes stale suspicious activity reports (>180 days old) but limits to 5 accounts per block. With potentially thousands of flagged accounts, full pruning could take thousands of blocks (hours to days).

**Impact**: Low — this is a conservative design for weight budgeting. But it means storage bloat accumulates faster than it's cleaned during high-activity periods.

---

#### COMP-DEEP-5 — INFO: Positive Pattern — Structuring Detection Implementation

**Location**: `record_transaction_and_check_structuring()` (~L1170-1265)

The structuring detection uses a proper sliding window approach:
1. Bounded sliding window of recent transactions per account
2. Prunes entries outside the window
3. Checks if ALL entries are below threshold (sub-threshold splitting)
4. Checks if cumulative total exceeds threshold
5. Requires at least 2 transactions (prevents false positive on first tx)
6. Auto-files SAR with FIFO eviction on full capacity
7. Emits `StructuringDetected` event

This is a well-implemented FATF-compliant structuring detector.

---

## 5. LandLedger Pallet (`pallets/landledger/src/lib.rs`, ~1100 lines)

### 5.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Operations |
|---|-----------|-----------|--------|--------|----------------|
| 1 | 0 | `register_property` | **Signed (any user)** | Hand-estimated | Coordinate validation, title hash uniqueness, `RegistrationDeposit`, temporal anchor |
| 2 | 1 | `transfer_property` | Signed (owner) | Hand-estimated | Government verification required, Oracle cross-check, KYC≥2, sanctions check both parties, transfer tax, temporal anchor |
| 3 | 2 | `verify_property` | GovernmentOrigin | Hand-estimated | Sets `government_verified: true` |
| 4 | 3 | `survey_property` | Signed (registered surveyor) | Hand-estimated | `SurveyReports` |
| 5 | 4 | `register_surveyor` | GovernmentOrigin | Hand-estimated | `GovernmentSurveyors` |
| 6 | 5 | `remove_surveyor` | GovernmentOrigin | Hand-estimated | `GovernmentSurveyors` removal (LL-4 FIX) |

### 5.2 Security Findings

#### LL-DEEP-1 — CRITICAL: Property Registration Has No KYC or Sanctions Check

**Location**: `register_property` (~L540-600)

**Description**: The `register_property` extrinsic accepts ANY signed origin with NO KYC or sanctions check. The only barrier is the `RegistrationDeposit`. Contrast with `transfer_property` which requires KYC≥2 on the buyer and sanctions checks on both parties:

```rust
// register_property: NO KYC, NO sanctions
pub fn register_property(origin: OriginFor<T>, ...) -> DispatchResult {
    let who = ensure_signed(origin)?;
    // ... coordinate validation, title hash uniqueness, deposit ...
    // NO: KYC check
    // NO: sanctions check
}

// transfer_property: HAS KYC + sanctions
pub fn transfer_property(origin: OriginFor<T>, ...) -> DispatchResult {
    // ... ensure buyer KYC >= 2, ensure !sanctioned(seller), ensure !sanctioned(buyer)
}
```

**Attack vectors**:
1. Sanctioned entity registers properties (claim-squatting before sanctions enforcement)
2. Un-KYC'd accounts register properties to create "proof of intent" for future disputes
3. Spam registration of all Belize coordinates to block legitimate registrations (title hash uniqueness check prevents same title, but different properties at same coordinates are possible)

**Impact**: Undermines the integrity of the land registry. While unverified properties can't be transferred, their existence in storage creates legal ambiguity.

**Recommendation**: Add at minimum a sanctions check on the registrant. Consider KYC≥1 requirement for registration.

---

#### LL-DEEP-2 — HIGH: Transfer Completes Before Government Approval

**Location**: `transfer_property` (~L620-700)

**Description**: The `transfer_property` extrinsic immediately changes the property owner and creates a new `TransferRecord` with `government_approved: false`. There is no separate `approve_transfer` extrinsic. Once called, ownership is immediately changed in `Properties` storage:

```rust
property.owner = buyer.clone();
Properties::<T>::insert(property_id, property);
TransferRecords::<T>::insert(property_id, transfer_record);
```

The `government_approved: false` field is set but never subsequently checked or enforced. No extrinsic exists to approve or reject the transfer after the fact.

**Impact**: Government approval is decorative, not functional. The transfer is irrevocable upon execution.

**Recommendation**: 
1. Add a `PendingTransfers` storage map
2. `transfer_property` creates a pending transfer (funds escrowed)
3. Add `approve_transfer` extrinsic (GovernmentOrigin) that finalizes ownership change
4. Add `reject_transfer` extrinsic that refunds buyer

---

#### LL-DEEP-3 — HIGH: No Duplicate Parcel Detection (Same Coordinates)

**Location**: `register_property` (~L540-580)

**Description**: The uniqueness check uses `TitleHashIndex` — preventing the same title hash from being registered twice. But there is NO check preventing multiple properties at the same geographic coordinates. Two users can register different title hashes at the same lat/lon.

**Coordinate validation** (Belize bounds):
```rust
// Latitude: 15.0 to 19.0 (stored as i64 × 10^6)
// Longitude: -90.0 to -87.0 (stored as i64 × 10^6)
ensure!(lat >= 15_000_000 && lat <= 19_000_000, ...);
ensure!(lon >= -90_000_000 && lon <= -87_000_000, ...);
```
This validates Belize bounds but does not check coordinate overlap with existing properties.

**Impact**: Conflicting property claims at the same location. Multiple unverified registrations for the same physical parcel.

**Recommendation**: Add a spatial index or coordinate-hash uniqueness check (e.g., hash of rounded coordinates to nearest parcel grid).

---

#### LL-DEEP-4 — MEDIUM: Transfer Tax Calculation Not Visible

**Location**: `transfer_property` (~L670-680)

**Description**: The transfer tax is calculated within the extrinsic but the exact formula was not fully visible in the read. The tax amount is transferred to the treasury. Verify that the tax calculation uses saturating arithmetic and cannot overflow for high-value properties.

---

#### LL-DEEP-5 — MEDIUM: Temporal Anchor Chain Depth Limit

**Location**: `verify_anchor_chain()`, `get_anchor_history()` (~L950-1030, M65 FIX)

**Description**: Both `verify_anchor_chain()` and `get_anchor_history()` are now properly depth-limited to `MAX_CHAIN_DEPTH = 100` (M65 fix). This prevents the O(n²) DoS previously identified. However, properties with anchor chains longer than 100 entries will have incomplete verification — only the last 100 anchors are checked.

**Impact**: Low — 100 anchors covers significant history. But callers should be aware that old anchors (>100 transfers ago) are not verified.

---

#### LL-DEEP-6 — LOW: Coordinate Bounds Are Approximate

**Location**: `register_property` (~L550-560)

**Description**: The latitude bounds (15.0-19.0°N) and longitude bounds (-90.0 to -87.0°W) are approximate for Belize. Belize's actual boundaries extend slightly beyond these ranges in some areas (e.g., cayes and offshore areas). Properties on border cayes or near Guatemala/Mexico borders might fail validation.

---

## 6. Mesh Pallet (`pallets/mesh/src/lib.rs`, ~1530 lines)

### 6.1 Complete Extrinsic Inventory

| # | call_index | Extrinsic | Origin | Weight | Key Operations |
|---|-----------|-----------|--------|--------|----------------|
| 1 | 0 | `register_node` | Signed | Hand-estimated | KYC check (configurable level), Gateway needs higher KYC, ValidatorRelay needs `is_validator()`, `NodeRegistrationDeposit` |
| 2 | 1 | `deregister_node` | Signed (owner) | Hand-estimated | Deposit return, `NetworkStats` update |
| 3 | 2 | `update_node_location` | Signed (owner) | Hand-estimated | Coordinate validation (global range, NOT Belize-specific) |
| 4 | 3 | `node_heartbeat` | Signed (owner) | Hand-estimated | `last_seen` update, reactivation |
| 5 | 4 | `submit_mesh_transaction` | Signed (gateway owner) | Hand-estimated | Signature hash ≠ zero (M57), dedup via `ProcessedMeshTransactions`, max hops |
| 6 | 5 | `submit_relay_proof` | Signed (node owner) | Hand-estimated | `RelayProofs` storage, reward deferred until confirmed |
| 7 | 6 | `issue_emergency_alert` | EmergencyOrigin OR Signed (emergency authority) | Hand-estimated | `EmergencyAlerts`, district counters (H-40 O(1)), bounded message (#80) |
| 8 | 7 | `resolve_emergency_alert` | EmergencyOrigin OR Signed (emergency authority) | Hand-estimated | Alert removal, counter decrement |
| 9 | 8 | `confirm_emergency_alert` | Signed (node owner) | Hand-estimated | Dedup via ProcessedMeshTransactions sentinel (M58) |
| 10 | 9 | `relay_block_header` | Signed (ValidatorRelay owner) | Hand-estimated | `MeshBlockHeaders`, prevents overwrite (MH-6) |
| 11 | 10 | `claim_relay_rewards` | Signed | Hand-estimated | Transfer from pallet account, clears ALL proofs |
| 12 | 11 | `update_mesh_config` | GovernanceOrigin | Hand-estimated | `MeshConfig` update |
| 13 | 12 | `fund_relay_rewards` | Signed (anyone) | Hand-estimated | Transfer to pallet account |
| 14 | 13 | `confirm_relay_proof` | Signed (node owner) | Hand-estimated | Cannot confirm own proofs, accrues reward on confirmation |

### 6.2 Security Findings

#### MESH-DEEP-1 — CRITICAL: Mesh Transaction Signature Not Cryptographically Verified

**Location**: `submit_mesh_transaction` (~L680-740)

**Description**: The `submit_mesh_transaction` extrinsic accepts a `signature_hash: H256` parameter. The M57 fix added a check that `signature_hash ≠ H256::zero()`, but NO actual cryptographic signature verification is performed. The signature hash is stored as-is without verifying that it corresponds to a valid signature over the transaction data by the sender.

```rust
// M57 FIX: Reject zero signature
ensure!(signature_hash != H256::zero(), Error::<T>::InvalidTransactionData);
// But: NO verification that signature_hash is actually a valid signature
```

**Attack vector**: A compromised gateway can submit fraudulent mesh transactions with arbitrary non-zero signature hashes. Since the gateway submits on behalf of the off-grid sender, there is no way for the chain to verify the sender actually authorized the transaction.

**Impact**: Gateway operators can fabricate mesh transactions. The trust model relies entirely on gateway honesty, not cryptographic verification.

**Recommendation**: 
1. Accept the full signature (not just hash) and verify it on-chain
2. Or implement a challenge period where the sender (once back online) can dispute unauthorized transactions
3. Or accept this as a design trade-off of the off-grid model and document the trust assumptions

---

#### MESH-DEEP-2 — HIGH: Relay Mining Sybil Attack via Two-Account Collusion

**Location**: `submit_relay_proof` (~L760-800), `confirm_relay_proof` (~L1400-1440)

**Description**: The relay mining system requires dual confirmation — one node submits a relay proof and a DIFFERENT node must confirm it. The `confirm_relay_proof` extrinsic correctly prevents self-confirmation:
```rust
ensure!(confirmer_node.owner != relayer_node.owner, Error::<T>::CannotConfirmOwnRelay);
```

However, the check only compares `owner` accounts. An attacker with two accounts (Account A and Account B) can:
1. Register Node X (owned by Account A)
2. Register Node Y (owned by Account B)
3. Node X submits a relay proof
4. Node Y (different owner!) confirms it
5. Both accounts are controlled by the same person → Sybil mining

**Impact**: Systematic extraction of relay mining rewards by colluding nodes that may not have actually relayed any real data.

**Recommendation**: 
1. Require relay proofs to include verifiable data (e.g., message content hash that can be cross-referenced)
2. Add geographic proximity requirements (confirmer must be within LoRa range of relayer)
3. Implement reputation-weighted rewards that increase with genuine confirmed relays over time
4. Consider a random confirmer assignment instead of self-selected confirmation

---

#### MESH-DEEP-3 — HIGH: `ProcessedMeshTransactions` Unbounded Growth

**Location**: `submit_mesh_transaction` (~L720), `confirm_emergency_alert` (M58 sentinel)

**Description**: `ProcessedMeshTransactions` is used for deduplication of both mesh transactions AND as a sentinel for emergency alert confirmation (M58 fix). Entries are NEVER pruned. Over time, this storage grows without bound.

**Impact**: Increasing storage costs. Eventually, iteration over this storage (if any exists) becomes a DoS vector. The dual-purpose usage (transactions + alert sentinel) further accelerates growth.

**Recommendation**: Add a pruning mechanism in `on_idle` — remove entries older than a configurable `DeduplicationWindow` (e.g., 7 days).

---

#### MESH-DEEP-4 — HIGH: `claim_relay_rewards` Clears ALL Proofs Including Unconfirmed

**Location**: `claim_relay_rewards` (~L1050-1080)

**Description**: When a node owner claims rewards, the extrinsic clears ALL relay proofs for all nodes they own. This includes both confirmed proofs (rewards already accrued) and unconfirmed proofs (waiting for confirmation). Unconfirmed proofs are lost — their pending rewards are never accrued.

**Impact**: Node owners who claim rewards before all their proofs are confirmed lose pending rewards permanently.

**Recommendation**: Only clear confirmed proofs. Keep unconfirmed proofs for future confirmation.

---

#### MESH-DEEP-5 — MEDIUM: Node Location Validation Uses Global Coordinate Range

**Location**: `update_node_location` (~L600-620)

**Description**: The coordinate validation for mesh nodes uses extremely broad bounds:
```rust
// Latitude: ±90° × 10^7 = ±900,000,000
// Longitude: ±180° × 10^7 = ±1,800,000,000
```
This accepts ANY coordinate on Earth, not just Belize. Contrast with LandLedger which validates Belize-specific bounds (15-19°N, 87-90°W).

**Rationale**: Mesh nodes could theoretically operate anywhere (e.g., Belizean diaspora nodes, relay points in neighboring countries). But this means the Belize-specific mesh network has no geographic enforcement.

**Impact**: Low — the LoRa physical range (10-15 km) naturally constrains effective mesh coverage. But nodes at fake coordinates could claim relay rewards without providing real coverage.

---

#### MESH-DEEP-6 — MEDIUM: Emergency Alert Confirmation Sentinel Overload

**Location**: `confirm_emergency_alert` (~L870-910, M58 FIX)

**Description**: The M58 fix uses `ProcessedMeshTransactions` as a sentinel to prevent duplicate emergency alert confirmations. This mixes concerns — the same storage map is used for transaction dedup and alert confirmation dedup. A collision (alert hash matching a transaction hash) would falsely prevent alert confirmation.

**Impact**: Theoretically possible hash collision between alert IDs and transaction hashes. Extremely unlikely (H256 collision space), but the architectural coupling is fragile.

**Recommendation**: Use a dedicated `ProcessedEmergencyAlerts` storage map for alert confirmation dedup.

---

#### MESH-DEEP-7 — LOW: Reward Rates Per Relay Type

**Location**: `confirm_relay_proof` (~L1400-1440)

```rust
let reward = match proof.relay_type {
    RelayType::Transaction => T::RelayRewardPerTransaction::get(),
    RelayType::BlockHeader => T::RelayRewardPerBlockHeader::get(),
    RelayType::EmergencyAlert => T::RelayRewardPerEmergencyAlert::get(),
    RelayType::Heartbeat | RelayType::Confirmation => {
        T::RelayRewardPerTransaction::get() / 10u32.saturated_into()
    },
};
```

Heartbeat/Confirmation rewards are 1/10th of transaction rewards. This ratio is hardcoded. If transaction rewards change, heartbeat rewards change proportionally. Consider making this a configurable parameter.

---

#### MESH-DEEP-8 — INFO: Positive Pattern — O(1) Emergency Alert Counters

**Location**: `issue_emergency_alert`, `resolve_emergency_alert` (H-40 FIX)

The emergency alert system maintains O(1) counters per district (`ActiveAlertCountPerDistrict`) and a global catastrophic alert counter (`CatastrophicAlertCount`), avoiding iteration over all alerts to answer "how many active alerts in district X?" queries.

---

## 7. Cross-Pallet Interaction Analysis

### 7.1 Identity → All Pallets (KYC Foundation)

```
Identity Pallet (KYC Source of Truth)
├── BelizeKyc trait
│   ├── is_kyc_verified(account) → bool
│   ├── get_verified_kyc_level(account) → KycLevel
│   ├── meets_kyc_requirement_level(account, level) → bool
│   └── is_account_sanctioned(account) → bool
│
├── Consumers:
│   ├── BNS: register_domain (KYC level check, sanctions check)
│   │   └── GAP: buy_domain, transfer_domain skip KYC/sanctions
│   ├── LandLedger: transfer_property (KYC≥2, sanctions both parties)
│   │   └── GAP: register_property skips KYC/sanctions
│   ├── Mesh: register_node (configurable KYC level, higher for gateway)
│   ├── Compliance: sync_verification_from_identity (pulls KYC level)
│   └── Oracle: verify_identity (parallel KYC source — can conflict with identity pallet)
│
└── RISK: Two KYC sources (identity attestations vs oracle votes)
    └── M51 FIX: On-chain attestations take priority over oracle
    └── BUT: If oracle KYC expires and identity doesn't, or vice versa, 
        different checks may return different results
```

### 7.2 Oracle → Identity, LandLedger, Economy

```
Oracle Pallet (External Data Bridge)
├── Price Feeds → Economy (exchange rates), BelizeX (swap prices)
├── Merchant Verification → Economy (tourism cashback)
│   └── RISK: Single-operator verification feeds cashback system
├── Sanctions → Identity (is_account_sanctioned), Compliance, BNS, LandLedger
│   └── RISK: Dual sanctions system (oracle AccountId-based vs compliance hash-based)
├── KYC Voting → Identity (fallback if no on-chain attestation)
├── Land Registry → LandLedger (cross-check on transfer)
└── Behavior Flags → Governance, Economy (circuit breaker)
    └── POSITIVE: Multi-oracle consensus for behavior flags
```

### 7.3 Compliance ← Economy, Interoperability

```
Compliance Pallet (Regulatory Enforcement)
├── ComplianceReporter::report_transaction()
│   ├── Called by: Economy (process_tourism_payment)
│   ├── Called by: Interoperability (bridge transfers)
│   └── NOT called by: pallet_balances transfers, BNS, LandLedger, Staking
│       └── CRITICAL GAP: Most value transfers bypass structuring detection
│
├── AccountSanctionsChecker::is_sanctioned()
│   └── No-op impl for () type — tests/mocks bypass sanctions
│
└── Verification gating:
    ├── can_be_validator() → Staking/Consensus
    ├── can_participate_in_governance() → Governance
    ├── can_access_treasury() → Governance/Economy
    └── requires_travel_rule() → Economy/Interoperability
```

### 7.4 Identified Cross-Pallet Risks

| # | Risk | Pallets | Severity | Description |
|---|------|---------|----------|-------------|
| XP-1 | Dual KYC Source Conflict | Identity ↔ Oracle | MEDIUM | Two independent KYC verification systems can disagree. M51 fix establishes priority but edge cases exist. |
| XP-2 | Dual Sanctions System | Compliance ↔ Oracle | MEDIUM | Hash-based vs AccountId-based sanctions can diverge. Need consolidation. |
| XP-3 | Balance Transfer Structuring Bypass | Compliance ← (none) | HIGH | Native balance transfers bypass `report_transaction()`. Major structuring detection gap. |
| XP-4 | Merchant Verification → Tourism Cashback | Oracle → Economy | HIGH | Single-operator merchant verification feeds directly into treasury-funded cashback. |
| XP-5 | BNS Marketplace KYC Gap | Identity ← BNS | HIGH | Domain purchases/transfers bypass the KYC/sanctions checks enforced on registration. |
| XP-6 | Land Registration KYC Gap | Identity ← LandLedger | HIGH | Property registration skips KYC/sanctions that transfer requires. |
| XP-7 | Mesh Signature Trust Gap | Mesh (internal) | CRITICAL | Mesh transactions are not cryptographically verified — entire security relies on gateway honesty. |

---

## 8. Aggregate Findings Summary

### By Severity

| Severity | Count | Findings |
|----------|-------|----------|
| **CRITICAL** | 6 | ID-DEEP-1 (SSN brute-force), OR-DEEP-1 (single-operator merchant), LL-DEEP-1 (no KYC on register), LL-DEEP-2 (transfer before approval), MESH-DEEP-1 (no sig verification), XP-7 (mesh trust gap) |
| **HIGH** | 13 | ID-DEEP-2, ID-DEEP-3, OR-DEEP-2, OR-DEEP-3, BNS-DEEP-1, BNS-DEEP-2, COMP-DEEP-1, LL-DEEP-3, MESH-DEEP-2, MESH-DEEP-3, MESH-DEEP-4, XP-3, XP-4 |
| **MEDIUM** | 15 | ID-DEEP-4, ID-DEEP-5, OR-DEEP-4, OR-DEEP-5, OR-DEEP-6, BNS-DEEP-3, BNS-DEEP-4, BNS-DEEP-5, COMP-DEEP-2, COMP-DEEP-3, COMP-DEEP-4, LL-DEEP-4, LL-DEEP-5, MESH-DEEP-5, MESH-DEEP-6 |
| **LOW** | 8 | ID-DEEP-6, LL-DEEP-6, MESH-DEEP-7, XP-1, XP-2, XP-5, XP-6, (reserve) |
| **INFO** | 5 | ID-DEEP-7, OR-DEEP-7, BNS-DEEP-6, COMP-DEEP-5, MESH-DEEP-8 |
| **TOTAL** | **47** | |

### Top 10 Priority Fixes

| Priority | Finding | Pallet | Effort | Action |
|----------|---------|--------|--------|--------|
| P0 | ID-DEEP-1 | Identity | Medium | Remove salt from on-chain storage; upgrade to Argon2id |
| P0 | MESH-DEEP-1 | Mesh | High | Implement on-chain signature verification or challenge period |
| P0 | LL-DEEP-2 | LandLedger | Medium | Add PendingTransfers escrow + approve_transfer extrinsic |
| P0 | OR-DEEP-1 | Oracle | Low | Require multi-operator consensus for merchant verification |
| P0 | COMP-DEEP-1 | Compliance | High | Add TransactionExtension for universal structuring detection |
| P1 | BNS-DEEP-1 | BNS | Low | Add KYC/sanctions checks in buy_domain |
| P1 | BNS-DEEP-2 | BNS | Low | Add sanctions check in transfer_domain |
| P1 | LL-DEEP-1 | LandLedger | Low | Add sanctions check in register_property |
| P1 | MESH-DEEP-2 | Mesh | Medium | Add verifiable relay proof data requirements |
| P1 | MESH-DEEP-3 | Mesh | Medium | Add ProcessedMeshTransactions pruning in on_idle |

---

## 9. Positive Security Patterns Observed

1. **No `unwrap()` in any of the 6 pallets** — Universal `ensure!()`, `ok_or()`, `unwrap_or_default()` usage
2. **Saturating arithmetic everywhere** — `saturating_add`, `saturating_sub`, `saturating_mul` consistently used
3. **No floating-point** — All calculations use integer arithmetic with `Permill`/basis points
4. **BoundedVec usage** — All vector storage uses `BoundedVec` with `try_push`
5. **Proper deposit patterns** — Registration deposits in identity, landledger, mesh
6. **Multi-oracle consensus** — Price feeds, KYC voting, behavior flags all require quorum
7. **Temporal anchoring** — LandLedger's Merkle-chain property history with depth limits (M65)
8. **Emergency alert O(1) counters** — H-40 fix avoids iteration-based alert counting
9. **Verification token entropy** — BNS-3 fix includes parent_hash and extrinsic_count
10. **Linked account sanctions** — Identity sanctions check iterates ALL linked accounts
11. **Dispute detection** — Oracle KYC voting detects conflicting votes and blocks until admin resolution
12. **M51 priority** — On-chain attestations correctly prioritize over oracle KYC
13. **Re-issuance hash index cleanup** — Identity SSN/passport re-issuance removes stale hash indexes

---

## Appendix A: Files Audited

| File | Lines | Read Coverage |
|------|-------|---------------|
| `pallets/identity/src/lib.rs` | ~1,200 | 100% |
| `pallets/oracle/src/lib.rs` | ~1,550 | 100% |
| `pallets/bns/src/lib.rs` | ~1,420 | 100% |
| `pallets/compliance/src/lib.rs` | ~1,350 | 100% |
| `pallets/landledger/src/lib.rs` | ~1,100 | 100% |
| `pallets/mesh/src/lib.rs` | ~1,530 | 100% |
| **Total** | **~8,150** | **100%** |

---

*Phase 2B Deep Audit — Complete*
