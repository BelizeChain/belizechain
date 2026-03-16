# BNS Pallet (pallet-belize-bns) — Complete Security Audit

**Auditor**: AI Security Audit Engine  
**Date**: 2026-03-15  
**Scope**: `/pallets/bns/src/` — lib.rs, types.rs, weights.rs, mock.rs, tests.rs, benchmarking.rs  
**Total Lines**: 2,949  
**Framework**: Substrate (Polkadot SDK stable2512)  
**Token**: DALLA (12 decimals, 1 DALLA = 1_000_000_000_000)

---

## 1. Executive Summary

The BNS pallet implements a permanent (non-expiring) domain name registry with marketplace, web hosting subscriptions, external domain linking, subdomain delegation, content versioning, and SSL certificate tracking. The overall architecture is **sound** with proper use of Substrate primitives, but several medium-severity issues and numerous low-severity design concerns were identified.

### Severity Counts

| Severity | Count | Description |
|----------|-------|-------------|
| **CRITICAL** | 2 | Exploitable bugs causing fund loss or state corruption |
| **HIGH** | 4 | Significant security or economic issues |
| **MEDIUM** | 8 | Logic flaws, missing validations, economic edge cases |
| **LOW / INFO** | 12 | Code quality, design concerns, missing tests |
| **TOTAL** | 26 | |

---

## 2. CRITICAL Findings

### CRITICAL-01: Marketplace Listing Seller Divergence — Stale Listing Allows Theft After Transfer

**File**: [lib.rs](pallets/bns/src/lib.rs#L623-L714)  
**Category**: 3 (TOCTOU) / 7 (State Inconsistency)

**Description**: When a domain is listed for sale and then the owner calls `transfer_domain()`, the transfer correctly removes the listing (line 519: `DomainListings::<T>::remove(&domain)`). However, the `buy_domain()` extrinsic validates the *listing's* seller against itself but pays the *listing's seller* — NOT the current domain owner. If the protected removal at line 519 were ever bypassed (e.g., via a runtime upgrade that touches `transfer_domain` without the M64 fix), the listing seller and domain owner would diverge.

**Current Status**: The M64 fix at line 519 **mitigates** this by eagerly removing listings on transfer. However:

1. The `buy_domain()` function reads the domain record **after** transferring funds (line 694). If the Currency::transfer fails partway through (between paying seller and paying treasury), the domain record read-and-update at line 694 would never execute, but the seller already received payment. This is mitigated by Substrate's transactional dispatch (all-or-nothing), so currently **safe** — but the code ordering is misleading.

2. The `buy_domain()` function does NOT check that `listing.seller == domain_record.owner` (line 694). It trusts that these are the same. If any code path creates a listing without ownership validation, the seller could sell someone else's domain.

**Impact**: If the M64 fix is removed or bypassed, domains can be sold from under a new owner.

**Recommendation**:
```rust
// In buy_domain(), after reading domain_record, add:
ensure!(listing.seller == domain_record.owner, Error::<T>::NotDomainOwner);
```

---

### CRITICAL-02: `ContentHistory` Unbounded Storage Growth — DoS via Version Flooding

**File**: [lib.rs](pallets/bns/src/lib.rs#L913-L934)  
**Category**: 5 (Unbounded Iteration / Storage Growth)

**Description**: `ContentHistory` is a `StorageDoubleMap` keyed by `(domain, u32_version)`. The version counter `CurrentContentVersion` increments monotonically via `saturating_add(1)` with **no upper bound**. An attacker who owns a domain with hosting can call `update_hosting_content()` repeatedly (each call inserts a new entry in `ContentHistory`), growing storage indefinitely.

- Each `ContentVersion` entry costs ~100 bytes on-chain.
- With 10,000 calls, this is ~1MB of permanent storage per domain.
- The `u32` counter allows up to 4 billion versions.
- There is **no cleanup mechanism** — content history is never pruned.

**Code**:
```rust
// lib.rs:926  
ContentHistory::<T>::insert(&domain, current_version, old_version);
// lib.rs:929
let new_version = current_version.saturating_add(1);
```

**Impact**: State bloat attack. An attacker with a cheap Free-tier hosting plan can flood storage.

**Attack Vector**:
1. Register domain (100 DALLA one-time)
2. Activate Free hosting (0 DALLA/month)
3. Call `update_hosting_content()` in a loop — each call only costs the extrinsic weight fee
4. Generate millions of `ContentHistory` entries

**Recommendation**:
```rust
// Add a constant for max versions
const MAX_CONTENT_VERSIONS: u32 = 100;

// In update_hosting_content(), before inserting:
ensure!(
    current_version < MAX_CONTENT_VERSIONS,
    Error::<T>::ArithmeticOverflow // or new error
);
```

---

## 3. HIGH Findings

### HIGH-01: No Sanctioned Account Check on `buy_domain()` and `transfer_domain()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L623-L714) (buy_domain), [lib.rs](pallets/bns/src/lib.rs#L502-L543) (transfer_domain)  
**Category**: 4 (Access-Control Bypass)

**Description**: `register_domain()` correctly checks `T::Identity::is_sanctioned(&who)` (line 440). However, neither `buy_domain()` nor `transfer_domain()` checks if the **buyer** or **new_owner** is sanctioned. A sanctioned entity can acquire domains through the marketplace or direct transfers.

**Impact**: Compliance bypass. Sanctioned accounts can circumvent KYC/sanctions by acquiring domains post-registration.

**Recommendation**:
```rust
// In buy_domain(), after let buyer = ensure_signed(origin)?;
ensure!(!T::Identity::is_sanctioned(&buyer), Error::<T>::AccountSanctioned);

// In transfer_domain(), add:
ensure!(!T::Identity::is_sanctioned(&new_owner), Error::<T>::AccountSanctioned);
```

---

### HIGH-02: `list_domain()` Allows Zero-Price Listing — Free Domain Theft via Frontrunning

**File**: [lib.rs](pallets/bns/src/lib.rs#L550-L618)  
**Category**: 6 (Incorrect Error Handling) / 14 (Cross-Pallet Trust)

**Description**: The `list_domain()` extrinsic accepts any `price: u128`, including zero. A domain owner could accidentally list a domain for price = 0. Anyone watching the mempool could then call `buy_domain()` and acquire the domain for free (paying only the 5% of 0 = 0 marketplace fee).

**Code**:
```rust
// lib.rs:554 — no validation on price
pub fn list_domain(
    origin: OriginFor<T>,
    domain_name: Vec<u8>,
    price: u128,        // <— can be 0
    min_offer: Option<u128>,
    duration_blocks: BlockNumberFor<T>,
) -> DispatchResult {
```

**Impact**: Domains can be listed for free and taken.

**Recommendation**:
```rust
ensure!(price > 0, Error::<T>::BidTooLow);
```

---

### HIGH-03: Permanent Domain Squatting — No Expiration Mechanism

**File**: [lib.rs](pallets/bns/src/lib.rs) (entire pallet design), [types.rs](pallets/bns/src/types.rs#L73-L90)  
**Category**: Name-Service-Specific (Name Squatting)

**Description**: The pallet is explicitly designed for "immutable, permanent ownership" (lib.rs line 3: "Immutable domain registry (.bz domains - permanent ownership like ENS)"). `DomainRecord` has no expiration field. Once registered, a domain cannot be reclaimed, even if:

- The owner's account is killed (goes below existential deposit)
- The owner loses their keys
- The domain is never used / never resolves anything

The only way to release a domain is for the owner to transfer it. There is no governance-override to reclaim abandoned domains.

**Impact**: Over time, the entire namespace could be squatted. Popular/short names will be taken early and held forever. This is a **fundamental design choice** but creates economic risk.

**Recommendation**: Consider adding:
1. Governance reclaim for demonstrably abandoned domains (e.g., no resolution set for 2+ years)
2. Annual renewal fees (even nominal) to prevent passive squatting
3. A grace period + auction mechanism for lost-key scenarios

---

### HIGH-04: `buy_domain()` Does Not Check Buyer's KYC Status

**File**: [lib.rs](pallets/bns/src/lib.rs#L623-L714)  
**Category**: 4 (Access Control Bypass)

**Description**: `register_domain()` requires `T::Identity::can_register_domain(&who)` (KYC check). However, `buy_domain()` performs no KYC check on the buyer. An unverified account can bypass KYC requirements by purchasing domains on the marketplace instead of registering them directly.

**Impact**: Complete KYC bypass for domain acquisition.

**Recommendation**:
```rust
// In buy_domain(), add after ensure_signed:
ensure!(T::Identity::can_register_domain(&buyer), Error::<T>::KycRequired);
```

---

## 4. MEDIUM Findings

### MED-01: Error Type Reuse — `InvalidTier` Used for "Invalid Version" in `rollback_content()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L1164-L1175)  
**Category**: 6 (Incorrect Error Handling)

**Code**:
```rust
// lib.rs:1164
let target_content = ContentHistory::<T>::get(&domain, target_version)
    .ok_or(Error::<T>::InvalidTier)?; // Reuse error for "invalid version"

// lib.rs:1168
ensure!(target_version < current_version, Error::<T>::InvalidTier);
```

**Impact**: Confusing error messages. Users receive "InvalidTier" when the version doesn't exist.

**Recommendation**: Add `InvalidVersion` and `VersionNotFound` errors.

---

### MED-02: `HostingExpired` Error Reused for "Listing Expired" in `buy_domain()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L648)  
**Category**: 6 (Incorrect Error Handling)

**Code**:
```rust
ensure!(current_block <= listing.expires_at, Error::<T>::HostingExpired);
```

**Impact**: A marketplace listing expiration returns `HostingExpired` — confusing for consumers.

**Recommendation**: Add `ListingExpired` error variant.

---

### MED-03: `update_hosting_content()` Description Truncation Uses Wrong Error

**File**: [lib.rs](pallets/bns/src/lib.rs#L920)  
**Category**: 6 (Incorrect Error Handling)

**Code**:
```rust
description: description.clone().try_into()
    .map_err(|_| Error::<T>::DomainTooLong)?,
```

The ContentVersion description field is bounded to 128 bytes (`ConstU32<128>`), but if it overflows, the error returned is `DomainTooLong` instead of something like `InvalidMetadata` or a proper error.

---

### MED-04: `DomainAlreadyExists` Error Reused for "Already Listed" in `list_domain()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L574)  
**Category**: 6 (Incorrect Error Handling)

**Code**:
```rust
ensure!(
    !DomainListings::<T>::contains_key(&domain),
    Error::<T>::DomainAlreadyExists  // Should be AlreadyListed
);
```

---

### MED-05: Subdomain Owner Gets Full Domain Rights Without Parent Control

**File**: [lib.rs](pallets/bns/src/lib.rs#L1095-L1145)  
**Category**: 4 (Access Control) / Name-Service-Specific (Subdomain Management)

**Description**: When `create_subdomain()` delegates to another account, the delegatee receives a `DomainRecord` with full ownership rights identical to a top-level domain. This means:

1. The delegatee can **transfer** the subdomain to anyone
2. The delegatee can **list and sell** the subdomain on the marketplace
3. The delegatee can create **sub-subdomains** under it
4. The parent domain owner has **no reclaim** capability

The parent owner cannot revoke the subdomain delegation.

**Impact**: Loss of control over delegated subdomains.

**Recommendation**: Add a `parent_domain` field to `DomainRecord` and enforce parent-owner override for reclaim operations.

---

### MED-06: `transfer_domain()` Does Not Transfer Associated Hosting/Resolution/SSL

**File**: [lib.rs](pallets/bns/src/lib.rs#L502-L543)  
**Category**: 7 (State Inconsistency)

**Description**: When a domain is transferred, only the `DomainRegistry` and `AccountDomains` are updated. The following associated records are **NOT** updated:

- `DomainResolution` — still points to the old owner's wallet address
- `HostedWebsites` — subscriber field still references old owner
- `SSLCertificates` — remains under old data
- `ContentHistory` — accessible by old version data

The new owner inherits the domain but the old owner's resolution (wallet address for payments) is still active. Payments routed via BNS resolution could go to the old owner.

**Impact**: Payments resolved through the domain go to the old owner until the new owner updates resolution.

**Recommendation**: Either:
1. Clear resolution records on transfer, or
2. Transfer them with the domain, or  
3. Document this behavior clearly and emit a warning event

---

### MED-07: `renew_hosting()` Accepts `months = 0` — No-Op Fee Collection

**File**: [lib.rs](pallets/bns/src/lib.rs#L826-L871)  
**Category**: 2 (Division/Rounding) / 6 (Error Handling)

**Description**: If `months = 0`, the fee calculation is `monthly_fee * 0 = 0`, which passes the `if total_fee > 0` check and becomes a no-op that emits a misleading `HostingRenewed` event with `blocks_extended = 0`.

**Recommendation**: `ensure!(months > 0, Error::<T>::InvalidTier);`

---

### MED-08: `ExternalDomains` Has No Format Validation

**File**: [lib.rs](pallets/bns/src/lib.rs#L948-L1026)  
**Category**: 9 (Cryptographic Weakness — input validation)

**Description**: `register_external_domain()` accepts any byte sequence up to 128 bytes as an external domain name. There is no validation that it:

- Contains a TLD (.com, .org, etc.)
- Contains valid DNS characters
- Is not a .bz domain (which should use register_domain)
- Is not empty

An attacker could register garbage bytes, emoji, or control characters as "external domains."

**Recommendation**: Add basic DNS format validation (alphanumeric + hyphen + dots, valid TLD pattern).

---

## 5. LOW / INFO Findings

### INFO-01: Weight Reuse for `rollback_content()` and `update_ssl_certificate()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L1147) and [lib.rs](pallets/bns/src/lib.rs#L1215)  
**Category**: 13 (Weight Underestimation)

Both `rollback_content` and `update_ssl_certificate` reuse `T::WeightInfo::update_hosting_content()` weight. While the actual storage operations differ:

- `rollback_content` reads `ContentHistory` (extra read) and inserts a rollback savepoint (extra write)
- `update_ssl_certificate` only does 1 read + 1 write (cheaper than update_hosting_content)

**Impact**: Weight inaccuracy. `rollback_content` may be undercharged; `update_ssl_certificate` is overcharged.

**Recommendation**: Add dedicated `fn rollback_content() -> Weight` and `fn update_ssl_certificate() -> Weight` to the `WeightInfo` trait with proper benchmarks.

---

### INFO-02: No `RuntimeEvent` Type in BNS Config Trait

**File**: [lib.rs](pallets/bns/src/lib.rs#L62-L100)  
**Category**: 12 (Determinism — Substrate convention)

The `Config` trait doesn't include `type RuntimeEvent: From<Event<Self>> + IsType<...>`. This is handled implicitly by the `#[pallet::event]` macro in newer Substrate versions, so this is just a convention note.

---

### INFO-03: `treasury_account()` Is Never Used

**File**: [lib.rs](pallets/bns/src/lib.rs#L1394-L1396)  
**Category**: Dead Code

```rust
pub fn treasury_account() -> T::AccountId {
    BNS_TREASURY_ID.into_account_truncating()
}
```

The pallet uses `T::Treasury::get()` for all fee collection. `treasury_account()` derives from `PalletId`, which is a different account. This is dead code that references a different treasury than the one actually used.

**Impact**: Potential confusion if someone uses this function expecting it to return the active treasury.

---

### INFO-04: `saturated_into()` Usage for Balance Conversion

**File**: [lib.rs](pallets/bns/src/lib.rs#L672-L684)  
**Category**: 1 (Integer Truncation)

Multiple uses of `.saturated_into()` to convert `u128` prices to Balance type. The code includes `// SAFETY` comments explaining that these are bounded by protocol constants. This is **acceptable** for the standard Substrate u128 Balance, but if the runtime ever uses a smaller Balance type, these would silently truncate.

**Impact**: No impact with u128 Balance. Potential future risk if Balance type changes.

---

### INFO-05: No Rate Limiting on Domain Registration

**File**: [lib.rs](pallets/bns/src/lib.rs#L402-L480)  
**Category**: 5 (DoS)

An account can register up to `MaxDomainsPerAccount` (100 in mock) domains in rapid succession. With sufficient funds, an attacker could front-run and squat hundreds of desirable names in a single block.

**Recommendation**: Consider per-block or per-era registration limits.

---

### INFO-06: `validate_domain_name()` Allows Leading/Trailing Hyphens and Dots

**File**: [lib.rs](pallets/bns/src/lib.rs#L1274-L1296)  
**Category**: 9 (Input Validation)

The validation only checks each byte individually. This allows malformed names like:
- `---.bz` (all hyphens)
- `.leading-dot`
- `trailing-.`
- `double..dot`

Standard DNS rules prohibit leading/trailing hyphens and consecutive dots.

---

### INFO-07: No Event Emitted on `unlist_domain()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L716-L737)

The `unlist_domain()` extrinsic does not emit an event, making it invisible to indexers and UIs.

---

### INFO-08: No Event Emitted on `deactivate_hosting()`

**File**: [lib.rs](pallets/bns/src/lib.rs#L876-L896)

Same as above — no event for hosting deactivation.

---

### INFO-09: `DomainListing.min_offer` Is Checked Redundantly

**File**: [lib.rs](pallets/bns/src/lib.rs#L646-L648)

```rust
ensure!(offer_price >= listing.price, Error::<T>::BidTooLow);
if let Some(min_offer) = listing.min_offer {
    ensure!(offer_price >= min_offer, Error::<T>::BidTooLow);
}
```

If `min_offer` > `price`, then `min_offer` is the effective floor. If `min_offer` <= `price`, it's a no-op. The contract semantics are unclear — should `min_offer` be a *below-asking-price* threshold for negotiation offers? Currently it only gates `buy_domain()` at full price.

---

### INFO-10: Test Coverage Gaps

**File**: [tests.rs](pallets/bns/src/tests.rs)  
**Category**: 11 (Testing)

Missing test coverage for:
- Zero-price listing scenario
- Expired listing purchase attempt
- `months = 0` hosting renewal
- Subdomain depth > 1 level (sub.sub.parent)
- Account at `MaxDomainsPerAccount` limit
- Concurrent listing + transfer race condition
- All error paths for `rollback_content`
- External domain format validation edge cases

---

### INFO-11: Benchmarking Helper `insert_listing` Uses `try_into().unwrap_or()`

**File**: [benchmarking.rs](pallets/bns/src/benchmarking.rs#L87)  
**Category**: 8 (Unsafe Pattern)

```rust
let expiry_val: u64 = TryInto::<u64>::try_into(current_block).unwrap_or(0) + 100_000u64;
```

Using `unwrap_or(0)` silently produces a listing that expires at block 100_000 if block number conversion fails. Benchmark-only code, so no runtime risk.

---

### INFO-12: `GovernanceOrigin` Type Is Defined But Never Used

**File**: [lib.rs](pallets/bns/src/lib.rs#L96)  
**Category**: Dead Code

```rust
type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
```

This is declared in Config but never used in any extrinsic. There's no governance-callable function (e.g., force-transfer, emergency freeze, domain reclaim). This was likely intended for the external domain verification approval flow but was never implemented.

**Recommendation**: Either implement governance extrinsics or remove the type.

---

## 6. Per-File Scores

| File | Lines | Score | Rationale |
|------|-------|-------|-----------|
| [lib.rs](pallets/bns/src/lib.rs) | 1,398 | **65/100** | Core logic is mostly sound. Uses `saturating_*` and checked arithmetic consistently. However: unbounded ContentHistory growth (CRITICAL-02), missing KYC/sanctions checks on transfers/purchases (HIGH-01/04), no price floor on listings (HIGH-02), error type reuse (MED-01/02/03/04), missing state transfer on domain transfer (MED-06), no events for unlist/deactivate. Domain validation allows malformed names. |
| [types.rs](pallets/bns/src/types.rs) | 240 | **90/100** | Clean type definitions. All types derive `MaxEncodedLen`. Bounded vecs used throughout. `BnsIdentityProvider` trait is well-designed. Minor: no `Default` impl for some types that would benefit from it. |
| [weights.rs](pallets/bns/src/weights.rs) | 144 | **70/100** | Hand-estimated weights without actual benchmark calibration. Missing `rollback_content` and `update_ssl_certificate` weight functions (reused from `update_hosting_content`). DB read/write counts may be inaccurate for some functions (e.g., `buy_domain` does more writes than declared). |
| [mock.rs](pallets/bns/src/mock.rs) | 159 | **85/100** | Standard mock runtime. `MockIdentityProvider` is simple but sufficient. All KYC bypasses are test-appropriate (account 666 = sanctioned, >= 100 = verified). |
| [tests.rs](pallets/bns/src/tests.rs) | 624 | **75/100** | Good coverage of happy paths and error paths. 30+ test functions covering registration, transfer, marketplace, hosting, subdomains, SSL, content versioning, and events. Missing: edge cases for zero-price, expired listings, renewal with months=0, max domains reached, and adversarial scenarios. |
| [benchmarking.rs](pallets/bns/src/benchmarking.rs) | 384 | **80/100** | All 13 WeightInfo functions benchmarked. Helper functions properly bypass KYC for benchmark setup. Minor: `unwrap_or` usage, no benchmark for `rollback_content` or `update_ssl_certificate`. |

---

## 7. Overall Pallet Score

### **72 / 100**

**Breakdown**:
- **Architecture**: 85/100 — Clean Substrate pallet structure, proper use of bounded types
- **Access Control**: 55/100 — KYC/sanctions bypass on marketplace and transfers is a significant gap  
- **Arithmetic Safety**: 90/100 — Consistent use of `checked_*` and `saturating_*`
- **Storage Safety**: 60/100 — ContentHistory unbounded growth is critical
- **Error Handling**: 60/100 — Extensive error reuse obscures failure reasons
- **State Consistency**: 65/100 — Domain transfer doesn't cascade to associated records
- **Testing**: 70/100 — Good coverage but missing critical edge cases
- **Weight Accuracy**: 65/100 — Hand-estimated, missing functions, some DB count mismatches

---

## 8. Priority Remediation Table

| Priority | ID | Finding | Effort | Impact |
|----------|-----|---------|--------|--------|
| **P0** | CRITICAL-02 | Cap ContentHistory versions to prevent storage DoS | Low | Blocks state bloat attack |
| **P0** | HIGH-01 | Add sanctions check to `buy_domain()` and `transfer_domain()` | Low | Compliance requirement |
| **P0** | HIGH-04 | Add KYC check to `buy_domain()` | Low | Closes KYC bypass |
| **P1** | CRITICAL-01 | Add `listing.seller == domain_record.owner` check in `buy_domain()` | Low | Defense-in-depth |
| **P1** | HIGH-02 | Add `price > 0` check in `list_domain()` | Low | Prevents free domain theft |
| **P1** | MED-06 | Clear or transfer resolution/hosting on domain transfer | Medium | Prevents stale payment routing |
| **P1** | MED-05 | Add parent-owner reclaim for delegated subdomains | Medium | Domain governance |
| **P2** | MED-07 | Reject `months = 0` in `renew_hosting()` | Low | Input validation |
| **P2** | MED-08 | Add DNS format validation for external domains | Medium | Input hygiene |
| **P2** | MED-01/02/03/04 | Add distinct error variants | Low | Developer experience |
| **P2** | INFO-01 | Add proper weight functions for rollback + SSL | Medium | Weight accuracy |
| **P2** | INFO-07/08 | Add events for unlist_domain and deactivate_hosting | Low | Observability |
| **P3** | HIGH-03 | Design domain renewal/reclaim mechanism | High | Long-term namespace health |
| **P3** | INFO-06 | Stricter domain name validation (no leading hyphens, etc.) | Low | Standards compliance |
| **P3** | INFO-12 | Implement governance extrinsics or remove `GovernanceOrigin` | Medium | Code cleanliness |
| **P3** | INFO-05 | Per-block registration rate limiting | Medium | Anti-squatting |
| **P3** | INFO-10 | Expand test coverage for edge cases | Medium | Quality assurance |

---

## 9. Detailed Category Analysis

### 9.1 Integer Overflow / Underflow / Truncation
- **Status**: GOOD. All arithmetic uses `saturating_add`, `checked_mul`, `checked_sub`, `checked_div`.
- `calculate_domain_price()` (line 1304) uses `checked_mul` — ✅
- `buy_domain()` fee calc (line 651) uses `checked_mul` + `checked_div` — ✅
- Counter updates all use `saturating_add` — ✅
- `saturated_into()` for u128→Balance is safe with u128 Balance — ✅ (with documented SAFETY notes)

### 9.2 Division by Zero / Rounding
- Marketplace fee: `price * 5 / 100` — no div-by-zero risk (constant divisor)
- Rounding: Integer division truncates. A price of 19 yields fee = 0 (19 * 5 / 100 = 0). This underpays the treasury for very small prices. Practically irrelevant since minimum domain prices are >50 DALLA.

### 9.3 Reentrancy / Double-Spend / TOCTOU
- Substrate's transactional dispatch provides atomicity — ✅
- `buy_domain()` transfers funds before updating ownership (lines 672-694). Safe due to atomicity, but ordering should prefer state-updates-before-transfers — **minor**.
- Transfer-while-listed: M64 fix handles this — ✅

### 9.4 Access-Control Bypass
- **Gap**: KYC + sanctions not enforced on marketplace buys and transfers (HIGH-01, HIGH-04)
- All extrinsics properly use `ensure_signed` — ✅
- Ownership checks present on all owner-gated operations — ✅

### 9.5 Unbounded Iteration / Storage Growth
- **Gap**: ContentHistory unbounded (CRITICAL-02)
- `AccountDomains` bounded by `MaxDomainsPerAccount` — ✅
- `DomainResolution.text_records` bounded by `MaxTextRecords` — ✅
- `DomainResolution.metadata` bounded to 256 bytes — ✅
- No iteration over storage maps in extrinsics — ✅

### 9.6 Incorrect Error Handling
- Multiple error type reuse issues (MED-01 through MED-04)
- No swallowed errors — all `?` propagation correct — ✅

### 9.7 State Inconsistency
- Domain transfer doesn't cascade (MED-06)
- Subdomain delegation gives irrevocable control (MED-05)
- M64 fix correctly clears listings on transfer — ✅

### 9.8 Unsafe Blocks / Raw Pointers
- **None found**. No `unsafe` code anywhere — ✅

### 9.9 Cryptographic Weakness
- Verification token generation uses `parent_hash` + `extrinsic_count` for entropy (BNS-3 fix) — ✅
- Blake2 hashing via Substrate's `Hashing` trait — ✅
- No custom crypto — ✅

### 9.10 Oracle / External Data Manipulation
- External domain verification is NOT auto-approved — requires governance — ✅
- No oracle dependency in core logic — ✅
- `verify_external_domain()` only records the attempt, doesn't change domain status — ✅

### 9.11 Concurrency / Race Conditions
- Substrate sequential extrinsic execution model prevents concurrency issues — ✅
- Potential mempool frontrunning of `buy_domain()` — inherent to any marketplace, not a bug

### 9.12 Determinism Violations
- **None found**. No floating point, no random number generation (verification token uses on-chain data only) — ✅
- `TimeProvider` is used but only in types, not in core logic — ✅

### 9.13 Weight / Gas Underestimation
- Weights are hand-estimated, not properly benchmarked — **medium risk**
- `buy_domain` weight claims 3 reads + 4 writes, but actual operations include 2 `Currency::transfer` calls (additional reads/writes inside the Currency pallet) — **underestimated**
- Missing weight functions for `rollback_content` and `update_ssl_certificate`

### 9.14 Cross-Pallet Trust Assumptions
- `T::Identity` trait is trusted for KYC — proper abstraction — ✅
- `T::Currency` is trusted for balance operations — standard — ✅
- No direct calls to other pallets beyond Currency/System — ✅
- `GovernanceOrigin` declared but unused — no governance operations

---

## 10. Name-Service-Specific Analysis

| Concern | Status | Notes |
|---------|--------|-------|
| **Name squatting** | ⚠️ HIGH-03 | Permanent ownership with no expiry enables indefinite squatting |
| **Name theft** | ✅ | Ownership checks on all transfer/sale operations |
| **Expiration** | N/A | Domains don't expire by design |
| **Resolution manipulation** | ⚠️ MED-06 | Stale resolution after transfer could route payments to old owner |
| **Registration fees** | ✅ | Properly calculated with checked arithmetic, length multiplier |
| **Subdomain management** | ⚠️ MED-05 | Delegated subdomains are irrevocable |
| **Renewal logic** | ⚠️ MED-07 | Hosting renewal allows months=0 no-op |
| **Reverse resolution** | ✅ | AccountDomains provides reverse lookup, properly maintained |
| **KYC requirements** | ⚠️ HIGH-04 | Enforced on registration but bypassed on marketplace buy |
| **Admin override** | ⚠️ INFO-12 | GovernanceOrigin defined but no govenance extrinsics exist |

---

*End of Audit Report*
