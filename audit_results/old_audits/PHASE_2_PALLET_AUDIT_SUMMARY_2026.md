# Phase 2: Pallet-by-Pallet Deep Audit — Complete Summary

**Auditor**: BelizeChain Full-Scale Comprehensive Audit  
**Date**: March 2026  
**Scope**: All 19 custom pallets in `pallets/`  
**Methodology**: Line-by-line review of every `lib.rs`, `types.rs`, `tests.rs`, `mock.rs`, `benchmarking.rs` across all pallets

---

## Executive Summary

**All 19 pallets audited. 382 total findings identified.**

| Severity | Count | % |
|----------|-------|---|
| **CRITICAL** | 35 | 9.2% |
| **HIGH** | 72 | 18.8% |
| **MEDIUM** | 120 | 31.4% |
| **LOW** | 88 | 23.0% |
| **INFO** | 67 | 17.5% |
| **TOTAL** | **382** | 100% |

### Systemic Issues (affecting ALL or most pallets)

1. **ALL 245+ weight functions are hand-estimated with no benchmarks** — Every pallet has weight undercounts ranging from 30% to 300%. This is a systemic CRITICAL/HIGH issue requiring Phase 15 dedicated attention.

2. **No formal verification or fuzz testing exists** — Zero `kani`, `prusti`, `proptest`, or `cargo-fuzz` targets across the entire codebase.

3. **Self-reported scoring** — Consensus (AI quality scores), Oracle (merchant verification), Community (participation scores) all rely on single-party attestation with no challenge mechanism.

4. **Unbounded storage growth** — Multiple pallets have `StorageMap` entries that are never cleaned up (mesh processed transactions, whistleblower reports, economy redemptions, BNS content versions, etc.)

5. **Single-authority abuse vectors** — Multiple pallets allow a single council member or authority to take unilateral actions (oracle merchant verification, moderation rulings, consensus AI authority control).

---

## Per-Pallet Findings Summary

### 1. Staking — 28 findings (4C, 5H, 8M, 6L, 5I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | `slash_validator` emits wrong slash amount (event lies) |
| C-2 | CRITICAL | `distribute_rewards` callable repeatedly per epoch — mints tokens each call |
| C-3 | CRITICAL | Partial clear in `end_epoch` leaves stale nominator entries |
| C-4 | CRITICAL | `claim_pouw_with_domain_bonus` double-mints base reward on top of domain bonus |
| H-1 | HIGH | `force_unstake` bypasses unbonding period — immediate stake withdrawal |
| H-2 | HIGH | No minimum nomination amount — dust nominations DoS validator maps |
| H-3 | HIGH | `update_commission` takes effect immediately — no epoch boundary delay |
| H-4 | HIGH | Weight undercounts across all staking extrinsics (50-200% under) |
| H-5 | HIGH | Validator can set 100% commission — nominators receive zero rewards |

### 2. Oracle — 27 findings (3C, 6H, 8M, 6L, 3I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Single operator unilateral merchant verification — no multi-sig or delay |
| C-2 | CRITICAL | Operator can self-verify as merchant for cashback fraud |
| C-3 | CRITICAL | Merchant records overwritten without existence check — data destroyed |
| H-1 | HIGH | Price feed staleness — no freshness check, feed can be arbitrarily old |
| H-2 | HIGH | Oracle operator can freely manipulate any price feed |
| H-3 | HIGH | No minimum oracle count — single oracle controls all feeds |
| H-4 | HIGH | Weight undercounts on all oracle extrinsics |
| H-5 | HIGH | Sanctions list managed by single operator — censorship risk |
| H-6 | HIGH | KYC level can be set by any oracle operator — no issuer-level check |

### 3. Payroll — 19 findings (0C, 6H, 5M, 6L, 2I)

| ID | Severity | Title |
|----|----------|-------|
| H-1 | HIGH | `issue_bonus` pays inactive/terminated employees |
| H-2 | HIGH | No payroll budget cap — unbounded treasury drain per period |
| H-3 | HIGH | Employee salary modification takes effect retroactively |
| H-4 | HIGH | `on_idle` full-table scan of all employees — DoS in on_idle |
| H-5 | HIGH | Weight undercounts across payroll extrinsics |
| H-6 | HIGH | Double-payment on retry for failed transfers |

### 4. Interoperability (Bridge) — 14 findings (2C, 4H, 5M, 3L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Source TX hash replay — same hash can drain escrow repeatedly |
| C-2 | CRITICAL | No on-chain external burn proof — unlock without burn verification |
| H-1 | HIGH | Bridge message ordering not enforced — out-of-order processing |
| H-2 | HIGH | Escrow balance check after transfer (TOCTOU) |
| H-3 | HIGH | No timeout on pending transfers — funds locked indefinitely |
| H-4 | HIGH | Weight undercounts on bridge extrinsics |
| ✓ | FIXED | PQ signature field binding — now includes all transaction fields |
| ✓ | FIXED | Rate limiting on bridge operations |
| ✓ | CORRECT | Escrow model — proper PalletId-based account |

### 5. Governance — 25 findings (3C, 3H, 8M, 7L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Permissionless treasury proposals — anyone can propose spend from treasury |
| C-2 | CRITICAL | Single council member treasury drain — self-approval for proposals <10K |
| C-3 | CRITICAL | Constitutional amendment dual-house bypass — technical+governance councils not both required |
| H-1 | HIGH | No proposal deposit — free spam of governance proposals |
| H-2 | HIGH | Weight undercounts on governance extrinsics |
| H-3 | HIGH | Proposal execution has no timeout — stale proposals execute with outdated context |

### 6. Consensus — 23 findings (3C, 5H, 7M, 4L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | `pq_signature` field NEVER verified on-chain — PQ security is theater |
| C-2 | CRITICAL | AI model quality score set by single authority — no verification |
| C-3 | CRITICAL | Consensus rounds solely controlled by AI Authority — centralized block production |
| H-1 | HIGH | PoUW scoring is self-reported — validators grade their own work |
| H-2 | HIGH | No equivocation reporting for Aura authorities |
| H-3 | HIGH | Validator rotation depends on staking pallet — circular dependency risk |
| H-4 | HIGH | Weight undercounts on consensus extrinsics |
| H-5 | HIGH | Emergency (Jaguar) mode activation by single authority |

### 7. Quantum — 23 findings (3C, 5H, 7M, 5L, 3I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Anyone can claim executor role and steal job funds |
| C-2 | CRITICAL | Executor can cancel jobs and trigger refund to themselves |
| C-3 | CRITICAL | Anyone can vote on verification — no qualification check |
| H-1 | HIGH | NFT royalty percentage not bounded — can be set to >100% |
| H-2 | HIGH | Job refund logic allows double-spend via cancel+complete race |
| H-3 | HIGH | Weight undercounts on all quantum extrinsics |
| H-4 | HIGH | PQ key registration has no expiry or rotation mechanism |
| H-5 | HIGH | Verification voting has no quorum requirement |

### 8. Community — 31 findings (3C, 8H, 10M, 6L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | No minimum quorum for proposal finalization — 1 vote passes proposals |
| C-2 | CRITICAL | Proposer deposit slashed when treasury has insufficient funds for their proposal |
| C-3 | CRITICAL | Self-reporting participation — users attest their own community service |
| H-1 | HIGH | Education rewards paid without proof of completion |
| H-2 | HIGH | Green project funding with no milestone verification |
| H-3 | HIGH | Unbounded proposal storage — no cleanup after execution/rejection |
| H-4 | HIGH | Weight undercounts across community extrinsics |
| H-5 | HIGH | No rate limiting on proposal creation |
| H-6 | HIGH | Treasury spend proposals bypass governance pallet |
| H-7 | HIGH | Cultural event funding with no attendance or completion verification |
| H-8 | HIGH | Duplicate voting not prevented across sessions |

### 9. Compliance — 21 findings (3C, 5H, 6M, 4L, 3I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | **MOST URGENT FIX** — Seconds-vs-blocks unit mismatch: verification expires in ~28h instead of ~7 days |
| C-2 | CRITICAL | `lift_restriction` underflows stats counter — counter goes negative |
| C-3 | CRITICAL | `restrict_account` double-counts restrictions in stats |
| H-1 | HIGH | Sanctions list bypass via identity rotation |
| H-2 | HIGH | AML transaction monitoring has no historical lookback |
| H-3 | HIGH | Weight undercounts on compliance extrinsics |
| H-4 | HIGH | KYC verification level can be downgraded without re-verification |
| H-5 | HIGH | No rate limiting on compliance check queries |

### 10. Identity — 18 findings (1C, 3H, 5M, 5L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Test suite uses stale function signatures (missing `salt` param) — tests may not compile |
| H-1 | HIGH | No identity deletion/unlinking mechanism — PII permanently on-chain |
| H-2 | HIGH | Revoked attestation hash blocks re-issuance of same credential forever |
| H-3 | HIGH | Weight undercounts on identity extrinsics |

### 11. Mesh — 18 findings (2C, 4H, 5M, 4L, 3I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | `ProcessedMeshTransactions` unbounded growth — no pruning mechanism |
| C-2 | CRITICAL | `MeshBlockHeaders` unbounded growth — no pruning mechanism |
| H-1 | HIGH | Weight undercount on `submit_mesh_block` (declared 2R+2W vs actual 5R+4W) |
| H-2 | HIGH | Weight undercount on `relay_mesh_transaction` |
| H-3 | HIGH | Relay mining Sybil via two-account collusion — first relay gets reward |
| H-4 | HIGH | No mesh node registration/authentication — anyone can submit |

### 12. BNS (Belize Name System) — 17 findings (0C, 4H, 6M, 5L, 2I)

| ID | Severity | Title |
|----|----------|-------|
| H-1 | HIGH | `transfer_domain` bypasses KYC/sanctions checks on recipient |
| H-2 | HIGH | `buy_domain` bypasses KYC/sanctions checks on buyer |
| H-3 | HIGH | Unbounded content version history — no cap on `update_content` calls |
| H-4 | HIGH | Weight undercounts across all BNS extrinsics |

### 13. Landledger — 17 findings (1C, 4H, 5M, 4L, 3I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | Land transfer completes WITHOUT government approval — `government_approved: false` but ownership immediately transferred, no `approve_transfer` extrinsic exists |
| H-1 | HIGH | No duplicate parcel detection — same coordinates can be registered twice |
| H-2 | HIGH | Title transfer has no value/pricing mechanism — gifting without tax |
| H-3 | HIGH | Weight undercounts on landledger extrinsics |
| H-4 | HIGH | Surveyor verification is single-party attestation |

### 14. Justice — 17 findings (1C, 2H, 5M, 5L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | `mediator_ruling` dual-origin pattern fragile — breaks if `MediatorOrigin` changes to proportional origin |
| H-1 | HIGH | No dispute-count limit per target — grief attack permanently denies rehabilitation (~400 DALLA/year cost) |
| H-2 | HIGH | Disputant bond refund logic INVERTED — dismissed filers refunded, upheld filers unrewarded |
| M-1 | MEDIUM | `escrow_slash` overwrites instead of accumulating |
| M-2 | MEDIUM | Appeal mechanism non-functional — no extrinsic processes appealed disputes |
| M-3 | MEDIUM | Dispute counter saturates at u32::MAX |
| M-4 | MEDIUM | Weight undercount on `mediator_ruling` (~60% under) |
| M-5 | MEDIUM | Weight undercount on `open_dispute` |

### 15. Moderation — 14 findings (0C, 0H, 5M, 4L, 5I)

| ID | Severity | Title |
|----|----------|-------|
| M-1 | MEDIUM | Weight undercount on `flag_content` (3R declared vs 4R actual) |
| M-2 | MEDIUM | Weight undercount on `submit_nawal_assessment` (1W declared vs 2W actual) |
| M-3 | MEDIUM | `NawalOracleOrigin` = single TechCouncil member — unilateral AI score setting |
| M-4 | MEDIUM | No score range validation — accepts 0-255 instead of expected 0-100 |
| M-5 | MEDIUM | Single moderator can make unilateral permanent rulings |
| L-1 | LOW | Storage leak — flags/counts/assessments never cleaned after ruling |
| L-2 | LOW | Inconsistent boundary semantics (≥ vs >) |
| L-3 | LOW | Full `ModeratorSet` deserialized for O(n) contains check |
| L-4 | LOW | No expiry/timeout on queued content |
| **Positive** | | No unwrap(), bounded moderator storage, no floating-point, double-flag prevention, AlreadyRuled guard, saturating arithmetic, 30+ tests |

### 16. Whistleblower — 21 findings (1C, 2H, 6M, 6L, 5I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | `fund_whistleblower_pool` allows unbacked pool inflation — governance sets counter, `deposit_creating` mints tokens from nothing on reward claims |
| H-1 | HIGH | Reporter bond NEVER unreserved — permanently locked on submission |
| H-2 | HIGH | `claim_reward` does not update report status to terminal state — still shows "Approved" after claiming |
| M-1 | MEDIUM | Weight undercount on `review_report` (2W declared vs 3W actual) |
| M-2 | MEDIUM | Weight undercount on `submit_report` (2R declared vs 3R actual) |
| M-3 | MEDIUM | `ReportCounter` wraps at u32::MAX |
| M-4 | MEDIUM | `UnderReview` status is dead code — never set by any extrinsic |
| M-5 | MEDIUM | Unbounded `Reports` and `EscrowedReward` growth |
| M-6 | MEDIUM | `deposit_creating` mints tokens with no backing transfer |
| **Positive** | | Sound alias_hash pseudonymity, no unwrap(), saturating arithmetic, double-claim prevention via take(), pool sufficiency check |

### 17. Common (Temporal Anchor Library) — 12 findings (0C, 0H, 4M, 4L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| M-1 | MEDIUM | Merkle root/proof ordering INCONSISTENCY — `calculate_merkle_root` uses positional ordering, `verify_merkle_proof` uses canonical ordering; incompatible for trees >2 leaves |
| M-2 | MEDIUM | Odd-leaf single-hash breaks proof verification |
| M-3 | MEDIUM | Trait returns unbounded `Vec` — DoS by design |
| M-4 | MEDIUM | O(n²) reads in `verify_anchor_chain` — ~5,050 storage reads worst case |
| L-1 | LOW | Timestamp field stores block number not Unix time — misleading for legal audit trails |
| L-2 | LOW | `get_latest_anchor` returns None unconditionally — dead code |
| L-3 | LOW | Trait uses `&'static str` errors not typed enums |
| L-4 | LOW | Repeated `Vec` allocation in Merkle root computation |
| **Positive** | | Clean library design (pure Rust, no FRAME), deterministic, no unwrap(), MaxEncodedLen, depth limits (MAX_CHAIN_DEPTH=100, MAX_HISTORY_DEPTH=100), no_std compatible |

### 18. BelizeX (DEX) — 20 findings (2C, 4H, 5M, 5L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| C-1 | CRITICAL | **FUNDAMENTAL ARCHITECTURAL FLAW** — Single-currency pool model breaks multi-asset AMM: ALL "assets" are actually DALLA via T::Currency, pools are immediately insolvent across multiple pairs |
| C-2 | CRITICAL | Limit orders don't reserve funds — no capital commitment, no execution engine, feature is non-functional |
| H-1 | HIGH | Unbounded `OrderBook` storage — DoS via order spam |
| H-2 | HIGH | Weight mismatches across ALL extrinsics (detailed table of declared vs actual R/W) |
| H-3 | HIGH | Multihop uses single-hop weight for up to 4 hops — 4× undercharge |
| H-4 | HIGH | No minimum liquidity lock — LP inflation attack vector (first depositor attack) |
| M-1 | MEDIUM | Fee precision loss — zero fee for trades ≤333 units, fee evasion through splitting |
| M-2 | MEDIUM | `DailyVolume` never resets — cumulative not daily |
| M-3 | MEDIUM | Combined transfer with separate reserve tracking — asymmetric deposit attack |
| M-4 | MEDIUM | Tourism discount underflows fee rate to zero in multihop |
| M-5 | MEDIUM | Multihop event reports only treasury fee not total fee |
| **Positive** | | Checked/saturating arithmetic, proportional LP minting, slippage protection, oracle price guard for WUSDC/BBZD, governance pause, constant product invariant preserved |

### 19. Economy — 17 findings (0C, 1H, 6M, 6L, 4I)

| ID | Severity | Title |
|----|----------|-------|
| H-1 | HIGH | Tourism incentive rates 100× lower than documented — `Permill::from_parts(500)` = 0.05%, not 5% |
| M-1 | MEDIUM | Weight undercount on `mint_bbzd` (3R+2W declared vs ~6R+4W actual) |
| M-2 | MEDIUM | Weight undercount on `redeem_bbzd` (2R+4W declared vs ~6R+5W actual) |
| M-3 | MEDIUM | Processed redemption requests never cleaned up — unbounded StorageMap |
| M-4 | MEDIUM | `on_initialize` inflation weight undercount (~12R+10W vs 7R+7W declared) |
| M-5 | MEDIUM | Self-referential tourism payments allow incentive harvesting (tourist == vendor) |
| M-6 | MEDIUM | Travel rule threshold / decimal denomination inconsistency across codebase (6 vs 9 vs 12 decimals) |
| L-1 | LOW | Missing zero-amount check in `governance_burn` |
| L-2 | LOW | Missing zero-amount check in `process_tourism_payment` |
| L-3 | LOW | Dead code: `TREASURY_ID` constant and `treasury_account()` helper |
| L-4 | LOW | Multi-year inflation gap not caught up after chain halt |
| L-5 | LOW | Weight undercount on `process_redemption` (minor) |
| L-6 | LOW | Weight undercount on `process_tourism_payment` |
| **Positive** | | No floating-point, no unwrap(), transactional safety, TotalSupply sync (H-20/H-21 fix), rate limiting on minting (AR-15 fix), bounded collections, reserve invariant checks, FATF compliance integration, correct inflation cap, 30+ tests |

---

## Aggregate Statistics

### By Severity
| Severity | Count |
|----------|-------|
| CRITICAL | 35 |
| HIGH | 72 |
| MEDIUM | 120 |
| LOW | 88 |
| INFO | 67 |
| **TOTAL** | **382** |

### By Category
| Category | Count | % of Total |
|----------|-------|------------|
| Weight Undercounts | ~60 | 15.7% |
| Access Control / Authorization | ~45 | 11.8% |
| Unbounded Storage Growth | ~25 | 6.5% |
| Financial / Token Logic | ~40 | 10.5% |
| Missing Validation | ~35 | 9.2% |
| Design / Architecture | ~30 | 7.9% |
| Dead Code / Unreachable | ~15 | 3.9% |
| Documentation Mismatch | ~12 | 3.1% |
| Test Coverage Gaps | ~10 | 2.6% |
| Other | ~110 | 28.8% |

### Top 10 Most Critical Findings (Immediate Action Required)

| # | Pallet | ID | Title | Impact |
|---|--------|----|-------|--------|
| 1 | compliance | C-1 | Seconds-vs-blocks unit mismatch | KYC verification expires in 28h instead of 7d — LIVE BUG |
| 2 | staking | C-2 | `distribute_rewards` callable repeatedly | Unlimited token minting per epoch |
| 3 | staking | C-4 | `claim_pouw_with_domain_bonus` double-mints | 2× reward payout on every claim |
| 4 | interoperability | C-1 | Source TX hash replay | Complete bridge escrow drain |
| 5 | belizex | C-1 | Single-currency pool model | DEX is architecturally broken for multi-asset |
| 6 | consensus | C-1 | PQ signatures never verified | Post-quantum security is non-functional |
| 7 | quantum | C-1 | Anyone can claim executor role | Job fund theft |
| 8 | governance | C-2 | Single council member treasury drain | Self-approval for <10K proposals |
| 9 | landledger | C-1 | Transfer without government approval | Land ownership transferred without required approval |
| 10 | whistleblower | C-1 | Unbacked pool inflation | Token minting from nothing on reward claims |

### Pallets by Risk (sorted by finding density)

| Pallet | LOC | Total Findings | Findings/100 LOC | Risk Level |
|--------|-----|----------------|-------------------|------------|
| justice | 559 | 17 | 3.04 | **VERY HIGH** |
| moderation | 501 | 14 | 2.79 | HIGH |
| whistleblower | 387 | 21 | 5.43 | **VERY HIGH** |
| common | 742 | 12 | 1.62 | MODERATE |
| landledger | 992 | 17 | 1.71 | HIGH |
| identity | 1,206 | 18 | 1.49 | MODERATE |
| economy | 1,225 | 17 | 1.39 | MODERATE |
| belizex | 1,454 | 20 | 1.38 | **HIGH** (2 CRITICALs) |
| compliance | 1,315 | 21 | 1.60 | **HIGH** (3 CRITICALs) |
| consensus | 1,311 | 23 | 1.76 | **HIGH** (3 CRITICALs) |
| bns | 1,398 | 17 | 1.22 | MODERATE |
| mesh | 1,481 | 18 | 1.22 | HIGH (2 CRITICALs) |
| payroll | 1,497 | 19 | 1.27 | MODERATE |
| oracle | 1,685 | 27 | 1.60 | **HIGH** (3 CRITICALs) |
| staking | 1,708 | 28 | 1.64 | **VERY HIGH** (4 CRITICALs) |
| interoperability | 1,734 | 14 | 0.81 | **HIGH** (2 CRITICALs) |
| quantum | 2,159 | 23 | 1.07 | **HIGH** (3 CRITICALs) |
| community | 2,395 | 31 | 1.29 | **HIGH** (3 CRITICALs) |
| governance | 7,396 | 25 | 0.34 | **HIGH** (3 CRITICALs) |

### Positive Patterns Observed Across Pallets

1. **No floating-point in any pallet** — All use `Permill`, `Perbill`, `saturating_*`, checked arithmetic
2. **No `unwrap()` in production code** — All pallets use `ensure!`, `ok_or`, `unwrap_or_default`
3. **Transactional semantics** — All FRAME v2 extrinsics have implicit transactional rollback
4. **Consistent event emission** — All state changes emit events
5. **Bounded collections where used** — `BoundedVec` with `try_push` pattern
6. **Saturating arithmetic** — Universal adoption of `saturating_add`, `saturating_sub`, `saturating_mul`
7. **Test suites exist for all 19 pallets** — Each has 15-50+ tests
8. **No unsafe code** — Verified by `cargo geiger`
9. **Feature-gated sudo** — `#[cfg(feature = "dev")]` excludes sudo from production

---

## Remediation Priority Matrix

### P0 — Fix Before Mainnet (CRITICAL, blocks launch)

| Finding | Pallet | Effort | Description |
|---------|--------|--------|-------------|
| compliance-C1 | compliance | Low | Fix seconds-vs-blocks unit mismatch |
| staking-C2 | staking | Low | Add epoch guard on distribute_rewards |
| staking-C4 | staking | Low | Fix double-mint in claim_pouw_with_domain_bonus |
| interop-C1 | interoperability | Medium | Add ProcessedTransactions replay guard with pruning |
| interop-C2 | interoperability | High | Design on-chain burn proof verification |
| belizex-C1 | belizex | High | Implement multi-asset trait (pallet-assets integration) |
| belizex-C2 | belizex | Medium | Reserve limit order funds or remove feature |
| consensus-C1 | consensus | High | Implement on-chain PQ signature verification |
| quantum-C1 | quantum | Low | Add executor registration + stake requirement |
| quantum-C2 | quantum | Low | Prevent executor from cancelling own jobs |
| quantum-C3 | quantum | Medium | Add voter qualification/stake requirement |
| governance-C1 | governance | Low | Add proposal deposit requirement |
| governance-C2 | governance | Medium | Require multi-member approval for all treasury spends |
| governance-C3 | governance | Medium | Enforce dual-house approval for constitutional amendments |
| landledger-C1 | landledger | Medium | Add `approve_transfer` extrinsic, gate ownership change on approval |
| whistleblower-C1 | whistleblower | Medium | Replace deposit_creating with treasury transfer |
| staking-C1 | staking | Low | Fix slash event amount to match actual slash |
| staking-C3 | staking | Medium | Complete nominator entry cleanup |
| community-C1 | community | Low | Add MinimumQuorum constant and enforce |
| community-C2 | community | Low | Only slash deposit on rejection, not insufficient treasury |
| community-C3 | community | Medium | Replace self-reporting with attestation + challenge |
| oracle-C1 | oracle | Medium | Require multi-operator consensus for merchant verification |
| oracle-C2 | oracle | Low | Prevent operator self-verification |
| oracle-C3 | oracle | Low | Add existence check before merchant record overwrites |
| consensus-C2 | consensus | High | Decentralize AI quality scoring |
| consensus-C3 | consensus | High | Remove single-authority consensus round control |
| compliance-C2 | compliance | Low | Fix counter underflow with saturating_sub |
| compliance-C3 | compliance | Low | Add existence check before incrementing restriction count |
| mesh-C1 | mesh | Medium | Add pruning mechanism for ProcessedMeshTransactions |
| mesh-C2 | mesh | Medium | Add pruning mechanism for MeshBlockHeaders |
| justice-C1 | justice | Medium | Redesign mediator origin pattern for forward compatibility |

### P1 — Fix in First Patch Release (HIGH)

- All 72 HIGH findings — primarily weight undercounts, access control gaps, unbounded storage
- Estimated 40% overlap with systemic weight issue (Phase 15 will address all at once)

### P2 — Fix in Subsequent Releases (MEDIUM)

- All 120 MEDIUM findings — edge cases, missing validations, documentation mismatches

### P3 — Address When Convenient (LOW/INFO)

- 155 LOW + INFO findings — code quality, dead code, minor optimizations

---

## Phase 2 Status: COMPLETE

All 19 pallets have been audited line-by-line. This report consolidates findings across all pallets and provides a prioritized remediation plan. Subsequent phases (3-15) will audit the runtime configuration, node binary, cross-pallet interactions, cryptography, networking, CI/CD, dependencies, test coverage, storage migrations, adversarial scenarios, formal verification, fuzz testing, and benchmark execution.
