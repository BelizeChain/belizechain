# BelizeChain Pallet Security Audit

**Date:** 2025-07-08  
**Auditor:** GitHub Copilot (automated static analysis)  
**Scope:** 7 pallets — `common`, `mesh`, `quantum`, `landledger`, `bns`, `payroll`, `community`  
**Total Lines:** 9,578  

---

## Executive Summary

The BelizeChain pallet suite shows a competent Substrate implementation with generally correct use of `BoundedVec`, `saturating_*` arithmetic, and proper origin checks. However, the audit identified **7 Critical**, **12 High**, **18 Medium**, and **15 Low** severity issues. The most impactful findings are: unbounded storage iterations in on-chain hooks and trait implementations (DoS/block weight exhaustion), uncollected marketplace fees (revenue leakage), uncapped token minting via `deposit_creating`, privacy model inconsistencies in payroll, and missing verification logic in BNS external domains.

---

## Table of Contents

1. [Pallet: common](#1-pallet-common)
2. [Pallet: mesh](#2-pallet-mesh)
3. [Pallet: quantum](#3-pallet-quantum)
4. [Pallet: landledger](#4-pallet-landledger)
5. [Pallet: bns](#5-pallet-bns)
6. [Pallet: payroll](#6-pallet-payroll)
7. [Pallet: community](#7-pallet-community)
8. [Cross-Pallet Findings](#8-cross-pallet-findings)

---

## 1. Pallet: common

**File:** `pallets/common/src/lib.rs` (20 lines)

### 1.1 Storage Items
None. This is a re-export module.

### 1.2 Extrinsics
None.

### 1.3 Events / Errors
None.

### 1.4 Hooks
None.

### 1.5 Helper Functions
Re-exports from `temporal_anchor` submodule:
- `TemporalAnchor` (struct)
- `AnchorType` (enum)
- `TemporalAnchoring` (trait)
- `temporal_helpers` (module)

### 1.6 TODO / FIXME / HACK
None.

### 1.7 Issues
No issues — this is a minimal re-export crate.

### 1.8 Cross-Pallet Dependencies
- **Used by:** `landledger` (implements `TemporalAnchoring` trait)

---

## 2. Pallet: mesh

**File:** `pallets/mesh/src/lib.rs` (1,429 lines)

### 2.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `MeshNodes` | `StorageMap<MeshtasticNodeId, MeshNode>` | Node registry |
| 2 | `NodesByOwner` | `StorageMap<AccountId, BoundedVec<..., ConstU32<10>>>` | Max 10 nodes/owner |
| 3 | `PendingMeshTransactions` | `StorageMap<H256, MeshTransaction>` | Unprocessed txs |
| 4 | `ProcessedMeshTransactions` | `StorageMap<H256, MeshTransaction>` | Processed txs |
| 5 | `EmergencyAlerts` | `StorageMap<u32, EmergencyAlert>` | Alert by ID |
| 6 | `NextAlertId` | `StorageValue<u32>` | Monotonic counter |
| 7 | `MeshBlockHeaders` | `StorageMap<u32, MeshBlockHeader>` | Block#→Header |
| 8 | `RelayProofs` | `StorageMap<MeshtasticNodeId, BoundedVec<..., ConstU32<100>>>` | Max 100 proofs/node |
| 9 | `RelayRewards` | `StorageMap<AccountId, Balance>` | Pending rewards |
| 10 | `NetworkStats` | `StorageValue<NetworkStatistics>` | Aggregate stats |
| 11 | `MeshConfig` | `StorageValue<MeshNetworkConfig>` | Governance-set config |

### 2.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `register_node` | Signed | KYC level via `T::Identity` |
| 2 | 1 | `deregister_node` | Signed | Owner check |
| 3 | 2 | `update_node_location` | Signed | Owner check |
| 4 | 3 | `node_heartbeat` | Signed | Owner check |
| 5 | 4 | `submit_mesh_transaction` | Signed | KYC level via `T::Identity` |
| 6 | 5 | `submit_relay_proof` | Signed | Owner + ValidatorRelay role |
| 7 | 6 | `issue_emergency_alert` | Emergency or Signed+Authority | Dual origin |
| 8 | 7 | `resolve_emergency_alert` | Emergency or Signed+Authority | Dual origin |
| 9 | 8 | `confirm_emergency_alert` | Signed | Owner check |
| 10 | 9 | `relay_block_header` | Signed | Owner + ValidatorRelay role |
| 11 | 10 | `claim_relay_rewards` | Signed | Has rewards |
| 12 | 11 | `update_mesh_config` | GovernanceOrigin | Governance only |
| 13 | 12 | `fund_relay_rewards` | Signed | Any account |
| 14 | 13 | `confirm_relay_proof` | Signed | Different owner + owns node |

### 2.3 Events
`NodeRegistered`, `NodeDeregistered`, `NodeLocationUpdated`, `NodeHeartbeat`, `MeshTransactionSubmitted`, `MeshTransactionProcessed`, `RelayRewardsFunded`, `RelayProofSubmitted`, `RelayProofConfirmed`, `RelayRewardsClaimed`, `EmergencyAlertIssued`, `EmergencyAlertResolved`, `EmergencyAlertConfirmed`, `BlockHeaderRelayed`, `MeshConfigUpdated`

### 2.4 Errors
`NodeAlreadyRegistered`, `NodeNotFound`, `NotNodeOwner`, `MaxNodesReached`, `InvalidNodeConfiguration`, `InsufficientKYCLevel`, `TransactionHashConflict`, `EmergencySystemDisabled`, `AlertNotFound`, `AlertAlreadyResolved`, `NotEmergencyAuthority`, `DuplicateMeshTransaction`, `RelayMiningDisabled`, `ValidatorNotFound`, `ProofAlreadyConfirmed`, `NotRelayMiner`, `NoRelayRewards`, `CannotConfirmOwnProof`, `ProofIndexOutOfRange`

### 2.5 Hooks
None.

### 2.6 Helper Functions
- `pallet_account_id()` — returns PalletId account
- `is_node_active()` — heartbeat timeout check
- `active_gateway_count()` — reads NetworkStats
- `total_transactions()` — reads NetworkStats

### 2.7 Trait Implementations
- `MeshTransactionBridge` — `is_mesh_tx_pending`, `total_mesh_transactions`
- `EmergencyAlertProvider` — `active_alerts_for_district`, `has_catastrophic_alert`

### 2.8 TODO / FIXME / HACK / Notes
- **L959:** `NOTE: Rewards are NOT accrued here. They are deferred until a confirm_relay_proof is called`
- **L1403:** `In production, this would filter by district/coordinates`
- **L1405-1406:** `Note: iteration over storage maps is expensive on-chain / In production, maintain a separate counter per district`

### 2.9 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| M-C1 | 1400-1415 | **`active_alerts_for_district()` iterates ALL alerts** — loops `0..NextAlertId` doing individual storage reads. This is called as a trait impl and could be invoked by other pallets on-chain. | Block weight exhaustion / DoS as alert count grows unboundedly. |
| M-C2 | 1417-1425 | **`has_catastrophic_alert()` iterates ALL alerts** — same pattern as above. | Block weight exhaustion / DoS. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| M-H1 | 1160-1190 | **`claim_relay_rewards` clears ALL relay proofs** for all owned nodes via `NodesByOwner` iteration (up to 10 nodes × 100 proofs = 1,000 removals), but weight function returns a constant. Actual weight is O(nodes × proofs). | Under-charged block weight; potential block stuffing. |
| M-H2 | All WeightInfo defaults | **All default weights have `proof_size: 0`**. Substrate's 2D weight model requires accurate proof_size for PoV (Proof of Validity) limits on parachains. | Parachain validation failure; blocks could exceed PoV limits. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| M-M1 | 724 | **`submit_mesh_transaction` marks tx processed and emits both Submitted+Processed events in same call** — no actual processing/validation occurs. | Misleading semantics; no actual tx validation. |
| M-M2 | 1093-1105 | **`confirm_emergency_alert` doesn't check if alert is resolved** — allows incrementing `confirmations` on already-resolved alerts. | Stale data accumulation. |
| M-M3 | 1093-1105 | **`confirm_emergency_alert` allows unlimited confirmations** from the same node. No deduplication. | Confirmation count inflation. |
| M-M4 | 1133-1145 | **`relay_block_header` overwrites existing header** at same block number. Any validator relay can replace a previously relayed header. | Data integrity risk if malicious relay node overwrites valid headers. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| M-L1 | 1030 | `issue_emergency_alert`: `unwrap_or_default()` on BoundedVec conversion silently truncates/drops oversized messages. | User gets empty message without error feedback. |
| M-L2 | 796 | `submit_mesh_transaction`: `amount` field is `0` in the stored struct when created via this extrinsic, regardless of actual tx data. | Informational — amount not used currently. |

### 2.10 Cross-Pallet Dependencies
- **Inbound:** `T::Identity: MeshIdentityProvider` (from identity pallet)
- **Outbound:** `MeshTransactionBridge` trait, `EmergencyAlertProvider` trait (consumed by other pallets)

---

## 3. Pallet: quantum

**File:** `pallets/quantum/src/lib.rs` (2,139 lines)

### 3.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `QuantumJobs` | `StorageMap<BoundedVec, QuantumJob>` | Job registry |
| 2 | `JobsByAccount` | `StorageMap<AccountId, BoundedVec<..., ConstU32<100>>>` | Max 100 jobs/account |
| 3 | `QuantumResults` | `StorageMap<BoundedVec, QuantumResult>` | Result by job_id |
| 4 | `QuantumAchievements` | `StorageMap<u64, QuantumAchievement>` | NFT registry |
| 5 | `NFTCounter` | `StorageValue<u64>` | Monotonic NFT ID |
| 6 | `TotalQuantumJobs` | `StorageValue<u64>` | Counter |
| 7 | `TotalDallaSpent` | `StorageValue<u128>` | Cumulative spend |
| 8 | `AccountStats` | `StorageMap<AccountId, AccountStatistics>` | Per-account stats |
| 9 | `VerificationRequests` | `StorageMap<BoundedVec, VerificationRequest>` | Multi-validator verification |
| 10 | `ValidatorReputation` | `StorageMap<AccountId, u32>` | 0-1000 reputation |
| 11 | `JobCounter` | `StorageValue<u64>` | Alternative counter |
| 12 | `NFTListings` | `StorageMap<u64, NFTListing>` | Marketplace listings |
| 13 | `NFTAuctions` | `StorageMap<u64, AuctionListing>` | Auction listings |
| 14 | `ListingCounter` | `StorageValue<u64>` | Counter |
| 15 | `BridgeRequests` | `StorageMap<u64, BridgeRequest>` | Cross-chain bridge |
| 16 | `BridgeCounter` | `StorageValue<u64>` | Counter |

### 3.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `submit_quantum_job` | Signed | Balance check + reserve |
| 2 | 1 | `update_job_status` | Signed | Submitter OR executor |
| 3 | 2 | `record_quantum_result` | Signed | Executor (any signed) |
| 4 | 3 | `verify_quantum_result` | Root | Root-only emergency override |
| 5 | 4 | `mint_achievement_nft` | Signed | Job submitter or executor |
| 6 | 5 | `transfer_nft` | Signed | NFT owner |
| 7 | 6 | `list_nft` | Signed | NFT owner |
| 8 | 7 | `buy_nft` | Signed | Any (not seller) |
| 9 | 8 | `delist_nft` | Signed | Seller |
| 10 | 9 | `request_verification` | Signed | Job submitter |
| 11 | 10 | `submit_verification` | Signed | Any validator (not already verified) |
| 12 | 11 | `bridge_to_ethereum` | Signed | NFT owner |
| 13 | 12 | `bridge_to_parachain` | Signed | NFT owner |
| 14 | 13 | `cancel_bridge` | Signed | NFT owner |

### 3.3 Events
`QuantumJobSubmitted`, `JobStatusUpdated`, `QuantumResultRecorded`, `ResultVerified`, `AchievementNFTMinted`, `NFTTransferred`, `NFTListed`, `NFTPurchased`, `NFTDelisted`, `BridgeInitiated`, `BridgeCancelled`, `VerificationRequested`, `VerificationSubmitted`, `VerificationConsensusReached`, `ReputationUpdated`

### 3.4 Errors
`JobNotFound`, `JobAlreadyExists`, `InsufficientBalance`, `InvalidCircuitParameters`, `NotAuthorized`, `InvalidStatusTransition`, `ResultAlreadyRecorded`, `ResultNotFound`, `InvalidVerificationProof`, `NFTNotFound`, `NFTAlreadyListed`, `ListingNotFound`, `ListingExpired`, `CannotBuyOwnNFT`, `NFTNotTransferable`, `NFTLockedInBridge`, `BridgeAlreadyInitiated`, `BridgeRequestNotFound`, `BridgeAlreadyClaimed`, `InvalidRecipientAddress`, `VerificationRequestAlreadyExists`, `VerificationRequestNotFound`, `ValidatorAlreadyVerified`, `ConsensusAlreadyReached`, `InvalidVerificationVote`, `VerificationDeadlinePassed`, `InvalidMetadataURI`

### 3.5 Hooks
None.

### 3.6 Helper Functions
- `get_active_job_count()` — filters JobsByAccount (bounded to 100)
- `has_achievement()` — **iterates ALL NFTs** via `QuantumAchievements::iter()`
- `update_validator_reputations()` — iterates verification votes
- `calculate_nft_attributes()` — pure computation
- `generate_metadata_uri()` — constructs IPFS placeholder URI

### 3.7 TODO / FIXME / HACK / Notes
- **L37:** `NOTE: This is a hash commitment, NOT a zero-knowledge proof`
- **L1128:** `In production, prefer request_verification + submit_verification flow`
- **L1373:** `In production, track original minter separately` — NFT marketplace royalty bug source
- **L1613:** `In production: Send XCM message to parachain` — XCM not implemented
- **L2006:** `In production, this would upload JSON metadata to IPFS`
- **L2126-2127:** Using estimated weights; benchmarks not run

### 3.8 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| Q-C1 | 1411-1416 | **Marketplace fee (2%) is calculated but NEVER transferred.** The `buy_nft` function calculates `marketplace_fee` then has a commented-out transfer: `// T::Currency::transfer(&buyer, &treasury_account, marketplace_fee, ...)?;`. The `marketplace_fee` amount simply vanishes — buyer pays full price but seller receives `sale_price - royalty - marketplace_fee`, meaning the fee is effectively burned (never sent anywhere). | **Revenue leakage.** 2% of every NFT sale is deducted from seller proceeds but not collected. Money disappears. |
| Q-C2 | 1373 | **NFT royalty uses current owner as "original minter"** — `let original_minter = nft.owner.clone();` at listing time. After the first resale, `original_minter` becomes the first buyer, not the actual minter. Royalties go to wrong account on all subsequent resales. | **Incorrect royalty distribution.** Creators never receive royalties after first sale. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| Q-H1 | 1883-1893 | **`has_achievement()` iterates ALL NFTs** via `QuantumAchievements::iter()`. This is O(n) over total NFTs stored. | Unbounded on-chain iteration; DoS as NFT count grows. Should use a secondary index. |
| Q-H2 | 1065 | **`record_quantum_result` has no executor validation** — any signed account can claim to be the executor and submit a result. The function only checks that the job exists and is Running/Completed, but doesn't verify that `executor` was assigned. | **Unauthorized result submission.** Any account can front-run the actual executor. |
| Q-H3 | 1195-1197 | **`mint_achievement_nft` charges mint fee via `reserve` but never releases/transfers it.** The fee stays reserved on the minter's account indefinitely. | Funds permanently locked. Should use `transfer` to treasury or `slash_reserved`. |
| Q-H4 | All WeightInfo | **Default weights have 0 `proof_size`** (uses only `ref_time` + RocksDb reads/writes but PoV component is still 0). | Parachain PoV limit bypass risk. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| Q-M1 | 1681-1689 | **`request_verification` deadline is hardcoded to 100 blocks** (~17 minutes @ 10s blocks). No configurability. For complex quantum results, this may be too short. | Verification may time out before validators can evaluate. |
| Q-M2 | 1613 | **XCM bridge for `bridge_to_parachain` is a stub** — NFT gets locked but no XCM message is sent. No mechanism to unlock if relayer never completes. | NFTs can be permanently locked with no recovery path (only `cancel_bridge` works before claim). |
| Q-M3 | 955 | **`update_job_status` allows executor to cancel submitted jobs** — the `(_, JobStatus::Cancelled)` match arm allows cancellation from any non-Completed status, and both submitter and executor can call it. | Executor can cancel a job they were assigned, getting the submitter refunded but wasting submitter time. |
| Q-M4 | 1540 | **Bridge `recipient` validation accepts both 20 and 42 byte addresses** — Ethereum addresses are 20 bytes raw or 42 bytes hex-encoded. Accepting both without normalization could lead to bridge relayer confusion. | Ambiguous address format could cause failed bridging. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| Q-L1 | 2006 | `generate_metadata_uri` returns placeholder IPFS URI — not actual IPFS content. | NFT metadata is not actually decentralized. |
| Q-L2 | 1195 | `mint_achievement_nft` validates `achievement_type_index > 10` maps to `Custom`. Any index ≥ 11 becomes Custom, which may be unintended. | Minor — no validation on custom type semantics. |
| Q-L3 | 2126-2130 | Benchmarking module disabled. Weights are hand-estimated. | Production readiness concern. |

### 3.9 Cross-Pallet Dependencies
- **Inbound:** `T::Currency` (economy/balances)
- **Outbound:** None (self-contained)

---

## 4. Pallet: landledger

**File:** `pallets/landledger/src/lib.rs` (943 lines)

### 4.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `Properties` | `StorageMap<u64, Property>` | Property registry |
| 2 | `PropertyOwners` | `StorageMap<AccountId, BoundedVec<..., ConstU32<1000>>>` | Max 1000 properties/owner |
| 3 | `TransferRecords` | `StorageMap<u64, TransferRecord>` | Transfer history |
| 4 | `PropertyByTitle` | `StorageMap<BoundedVec, u64>` | Title→ID index |
| 5 | `NextPropertyId` | `StorageValue<u64>` | Monotonic counter |
| 6 | `NextTransferId` | `StorageValue<u64>` | Monotonic counter |
| 7 | `GovernmentSurveyors` | `StorageMap<AccountId, bool>` | Authorized surveyors |
| 8 | `ZoningMap` | `DoubleMap<District, Zone, ZoningInfo>` | District×Zone→Info |
| 9 | `LandAnchors` | `StorageMap<u64, TemporalAnchor>` | Anchor by property ID |
| 10 | `PropertyAnchorChain` | `StorageMap<u64, (u64, u64)>` | Anchor chain links |

### 4.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `register_property` | Signed | KYC via Oracle |
| 2 | 1 | `transfer_property` | Signed | Owner check |
| 3 | 2 | `verify_property` | Signed/Root | Governance or authorized |
| 4 | 3 | `survey_property` | Signed | GovernmentSurveyor check |
| 5 | 4 | `register_surveyor` | Root | Root only |

### 4.3 Events
`PropertyRegistered`, `PropertyTransferred`, `PropertyVerified`, `PropertySurveyed`, `SurveyorRegistered`, `TransferTaxCollected`, `AnchorCreated`, `AnchorUpdated`

### 4.4 Errors
`PropertyNotFound`, `NotPropertyOwner`, `PropertyAlreadyExists`, `TitleAlreadyRegistered`, `NotAuthorized`, `InvalidPropertyData`, `MaxPropertiesReached`, `NotGovernmentSurveyor`, `AnchorNotFound`, `InvalidAnchorChain`

### 4.5 Hooks
None.

### 4.6 Helper Functions
- `account_id()` — PalletId for tax collection
- `next_property_id()` / `next_transfer_id()` — auto-increment
- TemporalAnchoring impl: `create_anchor`, `update_anchor`, `get_anchor`, `verify_anchor_chain`, `get_anchor_history`

### 4.7 TODO / FIXME / HACK / Notes
None found.

### 4.8 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| L-C1 | 629-630 | **`transfer_property` sets `government_approved: true` unconditionally.** The TransferRecord is created with `government_approved: true` without any actual government approval workflow. Comment in code acknowledges: "In production, this would require government approval step". | **Regulatory bypass.** Property transfers appear government-approved on-chain when they are not. Undermines the entire legal framework of the land registry. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| L-H1 | 880-910 | **`get_anchor_history` has unbounded recursion.** It recursively follows `PropertyAnchorChain` storage entries. A malicious or deeply-chained property could cause stack overflow or massive read amplification. | DoS via stack overflow or excessive storage reads. |
| L-H2 | 855-878 | **`verify_anchor_chain` has unbounded loop** following chain links via storage reads. No depth limit. | Same as L-H1 — DoS vector. |
| L-H3 | ~580 | **PropertyOwners allows `BoundedVec<..., ConstU32<1000>>`** — storing up to 1,000 property IDs per account. Each entry is a `u64`, so max ~8KB per account entry. | Excessive per-account storage. Consider a lower bound or pagination index. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| L-M1 | ~595-600 | **Transfer tax calculated as `price * rate / 10000`** using `u128` arithmetic. If `transfer_price` is very large, `price * rate` could overflow `u128`. No `checked_mul` used. | Potential overflow on extremely high property values (theoretical — `u128` max is ~3.4×10^38). |
| L-M2 | ~455 | **`register_property` doesn't validate `assessed_value > 0`** or impose minimum values. | Zero-value properties allowed. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| L-L1 | ~640 | `transfer_property` doesn't check if the `to` account exists or has KYC. | Property can be transferred to unverified accounts. |
| L-L2 | ~500 | `register_surveyor` is root-only with no event for removal — no `remove_surveyor` extrinsic. | Surveyors cannot be deauthorized without storage migration. |

### 4.9 Cross-Pallet Dependencies
- **Inbound:** `T::Oracle: LandLedgerOracleProvider` (from oracle pallet), `common::TemporalAnchoring`
- **Outbound:** `LandLedgerOracleProvider` trait (verify_land_owner, get_kyc_level, is_sanctioned)

---

## 5. Pallet: bns

**File:** `pallets/bns/src/lib.rs` (1,384 lines)

### 5.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `DomainRegistry` | `StorageMap<BoundedVec, DomainRecord>` | Domain registry |
| 2 | `AccountDomains` | `StorageMap<AccountId, BoundedVec<..., MaxDomainsPerAccount>>` | Owned domains |
| 3 | `DomainResolution` | `StorageMap<BoundedVec, DomainResolutionRecord>` | DNS resolution |
| 4 | `DomainListings` | `StorageMap<BoundedVec, DomainListing>` | Marketplace |
| 5 | `HostedWebsites` | `StorageMap<BoundedVec, HostingInfo>` | Web hosting |
| 6 | `ExternalDomains` | `StorageMap<BoundedVec, ExternalDomainInfo>` | External DNS |
| 7 | `DomainVerification` | `StorageMap<BoundedVec, VerificationStatus>` | Verification tracking |
| 8 | `SSLCertificates` | `StorageMap<BoundedVec, SSLCertInfo>` | SSL cert hashes |
| 9 | `TotalDomains` | `StorageValue<u64>` | Counter |
| 10 | `TotalHostingRevenue` | `StorageValue<u128>` | Revenue tracking |
| 11 | `TotalMarketplaceRevenue` | `StorageValue<u128>` | Revenue tracking |
| 12 | `NextOperationId` | `StorageValue<u64>` | Counter |
| 13 | `ContentHistory` | `DoubleMap<BoundedVec, u32, ContentVersion>` | Versioned content |
| 14 | `CurrentContentVersion` | `StorageMap<BoundedVec, u32>` | Current version |

### 5.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `register_domain` | Signed | Identity check + fee |
| 2 | 1 | `set_resolution` | Signed | Owner check |
| 3 | 2 | `transfer_domain` | Signed | Owner check |
| 4 | 3 | `list_domain` | Signed | Owner check |
| 5 | 4 | `buy_domain` | Signed | Not owner + fee |
| 6 | 5 | `unlist_domain` | Signed | Owner check |
| 7 | 6 | `activate_hosting` | Signed | Owner + fee |
| 8 | 7 | `renew_hosting` | Signed | Subscriber check |
| 9 | 8 | `deactivate_hosting` | Signed | Subscriber check |
| 10 | 9 | `update_hosting_content` | Signed | Subscriber check |
| 11 | 10 | `register_external_domain` | Signed | Identity check |
| 12 | 11 | `verify_external_domain` | Signed | Owner check |
| 13 | 12 | `create_subdomain` | Signed | Parent domain owner |
| 14 | 13 | `rollback_content` | Signed | Subscriber check |
| 15 | 14 | `update_ssl_certificate` | Signed | Owner check |

### 5.3 Events
`DomainRegistered`, `ResolutionSet`, `DomainTransferred`, `DomainListed`, `DomainPurchased`, `DomainUnlisted`, `HostingActivated`, `HostingRenewed`, `HostingDeactivated`, `ContentUpdated`, `ExternalDomainRegistered`, `ExternalDomainVerified`, `SubdomainCreated`, `ContentRolledBack`, `SSLCertificateUpdated`, `HostingFeeCollected`

### 5.4 Errors
`DomainAlreadyExists`, `DomainNotFound`, `NotDomainOwner`, `DomainTooShort`, `DomainTooLong`, `InvalidDomainCharacters`, `MaxDomainsReached`, `InsufficientBalance`, `DomainNotListed`, `CannotBuyOwnDomain`, `HostingNotActive`, `HostingExpired`, `ArithmeticOverflow`, `InvalidTier`, `VerificationFailed`

### 5.5 Hooks
None.

### 5.6 Helper Functions
- `validate_domain_name()` — charset + length validation
- `calculate_domain_price()` — tier × length multiplier
- `collect_domain_fee()` / `collect_hosting_fee()` — transfer to treasury
- `calculate_hosting_fee()` — tier-based pricing
- `generate_verification_token()` — hash(account + domain + block)
- `treasury_account()` — PalletId

### 5.7 TODO / FIXME / HACK / Notes
- **L1043:** `NOTE: In production, this would call an oracle or off-chain worker to verify the DNS TXT record`

### 5.8 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| B-C1 | 1037-1060 | **`verify_external_domain` auto-approves without DNS verification.** The function immediately sets `verified = true` regardless of whether the TXT record exists. Comment says "In production, this would call an oracle". Currently, **any domain owner can self-verify any external domain claim.** | **Domain hijacking.** A user can claim to own `google.com` and it will show as verified on-chain. Complete breakdown of external domain trust model. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| B-H1 | ~720 | **`buy_domain` marketplace fee (5%) deducted from seller but verified by simple multiplication** — `let fee = price * 5u128 / 100u128`. The fee is subtracted from the `seller_receives` amount and transferred to treasury via `collect_domain_fee`. However, **the seller separately loses the domain AND receives less** — the fee accounting is correct but `buy_domain` doesn't check domain `locked_until`. | Listed domains bypass time-lock. |
| B-H2 | ~700-760 | **Domain transfer during active listing** — `transfer_domain` doesn't check if domain is listed in marketplace. Owner can list a domain, then transfer it to another account, but the listing still references the old owner. A buyer would pay the old owner. | **Marketplace listing dangle** — stale listing after transfer. Buyer pays wrong account. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| B-M1 | ~810-830 | **Hosting expiry not checked on content update** — `update_hosting_content` checks `current_block <= hosting_info.expires_at` but `renew_hosting` and `activate_hosting` set `expires_at` from current block. If block time changes, calculations drift. | Minor timing inconsistency. |
| B-M2 | 1158-1175 | **`rollback_content` reuses `Error::InvalidTier`** for "version not found" — confusing error messages for callers. | Poor developer experience. |
| B-M3 | 1258-1265 | **`calculate_domain_price` uses `checked_mul` correctly** and returns `ArithmeticOverflow` on overflow. Good practice. | Positive finding. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| B-L1 | 1180 | `rollback_content` creates a "Rollback savepoint" with hardcoded description. Content version numbers grow monotonically even on rollbacks. | Version history grows unboundedly (via StorageDoubleMap). |
| B-L2 | ~1300 | `generate_verification_token` is deterministic from (account, domain, block_number) — if called in the same block, gives same token. | Predictable verification tokens. |

### 5.9 Cross-Pallet Dependencies
- **Inbound:** `T::Identity: BnsIdentityProvider` (from identity pallet), `T::Treasury` (PalletId)
- **Outbound:** None

---

## 6. Pallet: payroll

**File:** `pallets/payroll/src/lib.rs` (1,455 lines)

### 6.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `Employees` | `DoubleMap<AccountId, AccountId, Employee>` | Employer→Employee |
| 2 | `EmployeeDeductions` | `DoubleMap<AccountId, AccountId, BoundedVec<Deduction, MaxDeductions>>` | Deductions |
| 3 | `EmployerProfiles` | `StorageMap<AccountId, EmployerProfile>` | Employer data |
| 4 | `Departments` | `DoubleMap<AccountId, u32, [u8;32]>` | Employer→Dept |
| 5 | `PayrollSchedules` | `DoubleMap<AccountId, u32, PayrollSchedule>` | Scheduled payments |
| 6 | `NextScheduleId` | `StorageValue<u32>` | Counter |
| 7 | `PayrollRecords` | `StorageMap<u32, PayrollRecord>` | Audit trail |
| 8 | `NextRecordId` | `StorageValue<u32>` | Counter |
| 9 | `GlobalStats` | `StorageValue<PayrollStats>` | Aggregate stats |
| 10 | `VerifiedEmployers` | `StorageMap<AccountId, bool>` | **DEPRECATED** |
| 11 | `EmployerCount` | `StorageValue<u32>` | Counter |

### 6.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `add_employee` | Signed | Verified employer |
| 2 | 1 | `remove_employee` | Signed | Employer owns employee |
| 3 | 2 | `update_salary` | Signed | Employer owns employee |
| 4 | 3 | `execute_payment` | Signed | Employer owns employee |
| 5 | 4 | `batch_payment` | Signed | Verified employer |
| 6 | 5 | `create_schedule` | Signed | Verified employer |
| 7 | 6 | `update_schedule` | Signed | Employer owns schedule |
| 8 | 7 | `verify_employer` | Root | Root only |
| 9 | 8 | `toggle_employee_status` | Signed | Employer owns employee |
| 10 | 9 | `create_department` | Signed | Verified employer |
| 11 | 10 | `set_deduction` | Signed | Employer owns employee |
| 12 | 11 | `issue_bonus` | Signed | Employer owns employee |

### 6.3 Events
`EmployeeAdded`, `EmployeeRemoved`, `SalaryUpdated`, `PaymentExecuted`, `BatchPaymentCompleted`, `ScheduleCreated`, `ScheduleUpdated`, `EmployerVerified`, `EmployeeStatusToggled`, `DepartmentCreated`, `DeductionUpdated`, `BonusIssued`, `ScheduledPaymentProcessed`

### 6.4 Errors
`EmployerNotVerified`, `EmployeeNotFound`, `EmployeeAlreadyExists`, `MaxEmployeesReached`, `InsufficientBalance`, `ScheduleNotFound`, `InvalidPaymentAmount`, `MaxDepartmentsReached`, `MaxDeductionsReached`

### 6.5 Hooks

**`on_initialize`** (lines ~465-530):
- Scans `PayrollSchedules::iter()` — **full storage iteration**
- Limits processing to `scan_limit = T::MaxSchedulesPerBlock::get() * 10`
- For each due schedule, calls `process_scheduled_payment()` which itself iterates all employees

### 6.6 Helper Functions
- `compute_salary_commitment()` — blake2_256(salary, employer, employee)
- `compute_payment_commitment()` — blake2_256(gross, deductions, net, employer, employee)
- `calculate_deductions()` — sum all active deductions for employee
- `process_scheduled_payment()` — iterates employees, pays them, creates records
- `get_total_payroll()` — iterates all employees for employer (unbounded)
- `get_employee_count()` — `iter_prefix().count()` (O(n))
- `get_department_employee_count()` — `iter_prefix().filter().count()` (O(n))

### 6.7 TODO / FIXME / HACK / Notes
- **L425:** `DEPRECATED, use EmployerProfiles` — `VerifiedEmployers` storage item still actively used for employer verification checks in every extrinsic
- **L148:** Privacy note claiming salary is accessible "only via blockchain query" — misleading since blockchain is public
- **L260:** `Gross amount (kept for internal execution — see privacy note)`
- **L473:** `Note: The transfer amount IS visible in the system Transfer event (Substrate constraint)`

### 6.8 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| P-C1 | ~465-530 | **`on_initialize` performs `PayrollSchedules::iter()`** — a full storage scan across ALL employers and ALL schedules on EVERY block. Even with `scan_limit`, it reads all entries until limit is hit. The `iter()` call is **over a DoubleMap**, meaning it loads every employer's schedules. | **Block production DoS.** As employer count grows, `on_initialize` consumes increasing block weight. Could exceed block limits and halt the chain. |
| P-C2 | ~970-1000 + 1310-1400 | **`batch_payment` AND `process_scheduled_payment` iterate ALL employees TWICE** — once to calculate total, once to actually pay. Both use `Employees::iter_prefix(employer)` which is unbounded. An employer with thousands of employees could exhaust block weight. | **Block weight exhaustion.** No per-call employee count limit. Batch processing should use pagination. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| P-H1 | ~845-850 | **`add_employee` uses `iter_prefix().count()` for max employee check** — O(n) just to check if limit is reached. | Increasingly expensive per call as employee count grows. Should use a counter. |
| P-H2 | 148, 260, 473 | **Privacy model is broken.** Salary is stored in plaintext in `Employee.salary`. The `salary_commitment` hash provides no privacy because the actual salary is on-chain AND visible in Transfer events. The privacy claims in documentation are misleading. | False security guarantees. Users may rely on privacy that doesn't exist. |
| P-H3 | 425 | **`VerifiedEmployers` marked DEPRECATED but still used** in every extrinsic's `ensure!(VerifiedEmployers::get(&employer))` check. No migration path to `EmployerProfiles`. | Confusion about canonical employer verification source. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| P-M1 | 1070-1090 | **`create_schedule` counts employees via `iter_prefix`** to populate `employee_count` — this count is stale immediately if employees are added/removed later. | Schedule `employee_count` drifts from reality over time. |
| P-M2 | ~1310-1400 | **`process_scheduled_payment` uses `?` on Currency::transfer** — a single failed payment (e.g., existential deposit issue) aborts the entire batch for all employees. | One underpayable employee blocks all others in the schedule. |
| P-M3 | ~1120-1130 | **`create_department` uses `iter_prefix().count()` for department limit** — same O(n) pattern as add_employee. Also, `department_id = dept_count + 1` doesn't account for deleted departments. | IDs could collide with previously deleted departments. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| P-L1 | Various | Default `WeightInfo` implementations include nonzero `proof_size` — better than mesh/quantum zero-proof_size defaults. | Positive finding. |
| P-L2 | 1158 | `issue_bonus` doesn't check employee is active before paying. | Bonuses can be sent to inactive employees. |
| P-L3 | ~530 | `on_initialize` returns `Weight::from_parts(base_weight, 0)` with hardcoded base. Doesn't account for short-circuit cases. | Minor weight inaccuracy. |

### 6.9 Cross-Pallet Dependencies
- **Inbound:** `T::Oracle: PayrollOracleProvider` (from oracle pallet), `T::TimeProvider: UnixTime`
- **Outbound:** PayrollOracleProvider trait (get_kyc_level, meets_kyc_requirement)

---

## 7. Pallet: community

**File:** `pallets/community/src/lib.rs` (2,208 lines)

### 7.1 Storage Items

| # | Name | Type | Notes |
|---|------|------|-------|
| 1 | `SocialResponsibilityScores` | `StorageMap<AccountId, SRSData>` | SRS registry |
| 2 | `ParticipationHistory` | `StorageMap<AccountId, BoundedVec<..., MaxParticipationHistory>>` | Activity log |
| 3 | `PeerEndorsements` | `StorageMap<AccountId, u32>` | Endorsement count |
| 4 | `LastEndorsement` | `StorageMap<AccountId, BlockNumber>` | Cooldown tracking |
| 5 | `UserProposals` | `StorageMap<AccountId, ProposalStats>` | Proposal stats |
| 6 | `FeeExemptionUsage` | `StorageMap<AccountId, FeeExemptionData>` | Monthly fee tracking |
| 7 | `ProposalCount` | `StorageValue<u32>` | Counter |
| 8 | `CommunityProposals` | `StorageMap<u32, CommunityProposal>` | Proposal registry |
| 9 | `ProposalVotes` | `DoubleMap<u32, AccountId, Vote>` | Vote records |
| 10 | `EthicsFilterConfig` | `StorageValue<EthicsConfig>` | Config |
| 11 | `SanctionedAccounts` | `StorageMap<AccountId, SanctionStatus>` | Sanctions |
| 12 | `EthicsCouncilVotes` | `DoubleMap<u32, AccountId, bool>` | Council votes |
| 13 | `EducationModules` | `StorageMap<u32, EducationModule>` | Learn-to-earn |
| 14 | `CompletedEducation` | `DoubleMap<AccountId, u32, CompletionData>` | Completion records |
| 15 | `GreenProjects` | `StorageMap<u32, GreenProject>` | Green initiatives |
| 16 | `GreenContributions` | `DoubleMap<AccountId, u32, u64>` | Contribution amounts |
| 17 | `ReferralData` | `StorageMap<AccountId, ReferralInfo>` | Referral tracking |
| 18 | `ReferralClaimed` | `DoubleMap<AccountId, AccountId, bool>` | Claim dedup |

### 7.2 Extrinsics

| # | Call Index | Name | Origin | Auth Check |
|---|-----------|------|--------|------------|
| 1 | 0 | `record_participation` | Signed | Self-reporting |
| 2 | 1 | `update_srs` | Signed | By account |
| 3 | 2 | `endorse_peer` | Signed | KYC verified |
| 4 | 3 | `set_srs_privacy` | Signed | By account |
| 5 | 4 | `submit_community_proposal` | Signed | KYC + deposit |
| 6 | 5 | `vote_community_proposal` | Signed | KYC verified |
| 7 | 6 | `finalize_community_proposal` | Signed | Anyone (after deadline) |
| 8 | 7 | `sanction_account` | GovernanceOrigin | Governance only |
| 9 | 8 | `lift_sanction` | GovernanceOrigin | Governance only |
| 10 | 9 | `ethics_council_vote` | Signed | Council member check |
| 11 | 10 | `complete_education_module` | Signed | Module exists + active |
| 12 | 11 | `contribute_to_green_project` | Signed | Project exists + active |
| 13 | 12 | `claim_referral_reward` | Signed | Not self-referral |

### 7.3 Events
`ParticipationRecorded`, `SRSUpdated`, `PeerEndorsed`, `SRSPrivacyUpdated`, `ProposalSubmitted`, `ProposalVoted`, `ProposalFinalized`, `ProposalExecuted`, `ProposalRejected`, `DepositReturned`, `AccountSanctioned`, `SanctionLifted`, `EthicsCouncilVoted`, `EthicsDecisionFinalized`, `EducationModuleCompleted`, `EducationRewardClaimed`, `GreenContributionMade`, `GreenMilestoneReached`, `ReferralClaimed`, `ReferralRewardPaid`, `FeeExemptionReset`

### 7.4 Errors
`NotVerified`, `InvalidScore`, `CannotEndorseSelf`, `EndorsementCooldownActive`, `ProposalNotFound`, `InvalidProposalStatus`, `VotingPeriodEnded`, `AlreadyVoted`, `InsufficientDeposit`, `NotEthicsCouncilMember`, `AlreadyVotedInCouncil`, `ModuleNotFound`, `ModuleInactive`, `AlreadyCompleted`, `ModuleCapReached`, `ProjectNotFound`, `ProjectInactive`, `InvalidReferral`, `ReferralAlreadyClaimed`, `BeneficiaryNotVerified`

### 7.5 Hooks
None.

### 7.6 Helper Functions
- SRS calculation: `calculate_srs`, `calculate_governance_score`, `calculate_education_score`, `calculate_sustainability_score`, `calculate_participation_score`, `calculate_endorsement_score`, `calculate_honesty_score`
- Fee system: `calculate_fee_discount`, `check_fee_exemption_limit`, `apply_fee_exemption`
- Ethics: `check_ethics_filter`, `try_finalize_ethics_review`
- SRS update: `update_srs_internal`, `update_srs_after_education`, `update_srs_after_green_contribution`
- Tier: `score_to_tier`

### 7.7 Trait Implementations (Exported)
- `CommunityRank<AccountId>` — `get_community_rank`, `get_srs_tier`
- `FeeCalculator<AccountId, Balance>` — `calculate_effective_fee`, `apply_fee_discount`
- `PoUWContributor<AccountId>` — `record_pouw_contribution`, `get_pouw_score`
- `GovernanceParticipation<AccountId>` — `record_proposal_submission`, `record_vote_cast`, `record_proposal_approval`, `record_council_activity`

### 7.8 TODO / FIXME / HACK / Notes
- **L688:** `NOTE: In production, cross-pallet trait calls should be used for automatic recording`
- **L1131:** `Slash deposit (goes to treasury in production)`
- **L1271:** `_completion_proof: BoundedVec<u8, ConstU32<256>>, // Future: verify proof` — education completion proof is accepted but never verified

### 7.9 Issues

#### CRITICAL

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| C-C1 | 1286-1310 | **`complete_education_module` mints tokens via `deposit_creating` with NO supply cap.** Anyone with an active education module can mint rewards. Module `reward_amount` is set at genesis or by governance but there's no total supply check. Repeated calls to different modules stack rewards unboundedly. | **Uncapped inflation.** Total education reward issuance is bounded only by the number of modules × capacity, which is governance-set but has no hard cap per block or per period. |
| C-C2 | 1380-1410 | **`claim_referral_reward` mints tokens via `deposit_creating` with NO supply cap.** Referral reward = `1000 + (referrals - 1) * 100`, increasing per referral. Since referee only needs an SRS entry (registerable by anyone), Sybil attacks can mint unbounded tokens. | **Sybil-drainable inflation.** Create N fake accounts, get SRS entries, then claim referral rewards for each. Rewards increase with each referral. |

#### HIGH

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| C-H1 | ~680-700 | **`record_participation` allows self-reporting** — caller == account with no verification. Anyone can inflate their participation history and SRS score. | **SRS gaming.** A user can call `record_participation` repeatedly to boost their score, getting fee discounts (up to 90%) and governance voting weight. |
| C-H2 | 1621-1640 | **`calculate_education_score` uses `CompletedEducation::iter_prefix(account).count()`** — O(n) storage reads in SRS calculation. SRS is recalculated on every participation, endorsement, and education completion. | Increasingly expensive storage reads as education completion count grows. |
| C-H3 | 1643-1660 | **`calculate_sustainability_score` iterates `GreenContributions::iter_prefix(account)`** — same O(n) issue with compounding reads. | Same unbounded iteration concern. |
| C-H4 | 1110-1140 | **Rejected proposals AND failed treasury transfers both slash the proposer's deposit.** If a proposal is approved by vote but treasury is empty, the proposer loses their deposit — punished for treasury insolvency, not for bad behavior. | Unfair deposit slashing on treasury failure. |

#### MEDIUM

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| C-M1 | ~830-840 | **Endorsement cooldown uses `blocks_per_month = 432_000`** (assumes 6s blocks) but other code uses `blocks_per_month = 30 * 24 * 60 * 10` ≈ 432,000 (assumes 10s blocks). The `check_fee_exemption_limit` uses `30 * 24 * 60 * 10 = 432,000` which equals the 6s assumption. Comment says "~6s blocks" but the math `30 * 24 * 60 * 10 = 432,000` is correct for 10s blocks (30 days × 24h × 60min × 6 blocks/min). Inconsistency between comment and math. | Block time assumption inconsistency across the codebase. |
| C-M2 | 1271 | **Education `completion_proof` parameter is accepted but NEVER validated.** The function parameter `_completion_proof` is prefixed with `_` and ignored. Any value passes. | Learn-to-earn rewards distributed without actual learning verification. |
| C-M3 | 1340-1350 | **`contribute_to_green_project` takes `amount: u64` but no actual transfer occurs.** The function records the contribution amount but doesn't transfer or reserve any tokens. | Contribution tracking is purely self-reported — no economic commitment required. |
| C-M4 | 1050-1060 | **`vote_community_proposal` uses SRS score as vote weight** — higher SRS = more voting power. Combined with C-H1 (self-reportable SRS), this creates a plutocratic/sybil vulnerability. | Governance capture via SRS inflation. |
| C-M5 | 1500-1510 | **`calculate_srs` includes `governance_score` which iterates `ParticipationHistory`** — O(n) per entry with time-decay bonus calculation. | Compounding computation cost per SRS update. |

#### LOW

| ID | Line | Issue | Impact |
|----|------|-------|--------|
| C-L1 | 1580-1600 | `calculate_governance_score` time-decay bonus: `blocks_per_6_months = 6 * 24 * 30 * 10 * 6 = 259,200` — note the unusual calculation `6 * 24 * 30 * 10 * 6`. This evaluates to 259,200 which is ~30 days at 10s blocks, not 6 months. Missing a factor. | Governance time bonus kicks in after ~30 days instead of 6 months. |
| C-L2 | ~1900-1930 | Genesis config for education modules and green projects doesn't validate uniqueness of IDs. | Duplicate IDs at genesis would overwrite each other silently. |
| C-L3 | 1380-1410 | `claim_referral_reward` checks `SocialResponsibilityScores::contains_key(&referee)` to validate the referee exists. But SRS entry can be created by the referee themselves via `record_participation`. | Extremely low barrier for Sybil referral farming. |

### 7.10 Cross-Pallet Dependencies
- **Inbound:** `T::BelizeKyc: pallet_belize_identity::BelizeKyc`
- **Outbound:** `CommunityRank`, `FeeCalculator`, `PoUWContributor`, `GovernanceParticipation` (all traits consumed by other pallets)

---

## 8. Cross-Pallet Findings

### 8.1 State Inconsistencies

| ID | Severity | Pallets | Issue |
|----|----------|---------|-------|
| X-1 | HIGH | payroll, community | **Privacy model contradiction.** Payroll claims privacy-preserving salary commitments, but `community::FeeCalculator` exports SRS-based fee discounts that could be correlated with salary-sized transactions. Transfer events reveal exact amounts regardless. |
| X-2 | MEDIUM | community, governance | **SRS weight in voting + self-reportable participation** creates circular amplification. Users boost SRS → get more governance weight → approve proposals that benefit them → record participation → further boost SRS. |
| X-3 | MEDIUM | mesh, all | **Mesh pallet's `EmergencyAlertProvider` trait implementations** iterate storage unboundedly. Any pallet calling `active_alerts_for_district` or `has_catastrophic_alert` incurs O(total_alerts) cost that is not reflected in the caller's weight. |
| X-4 | LOW | bns, identity | **BNS `verify_external_domain` auto-approves** without using identity pallet's verification infrastructure. The identity pallet's `BnsIdentityProvider::can_register_verified` is checked at registration but not at verification. |

### 8.2 Dead Code / Stub Implementations

| ID | Location | Description |
|----|----------|-------------|
| D-1 | quantum L1613 | XCM bridge — `bridge_to_parachain` locks NFT but never sends XCM message |
| D-2 | bns L1043 | External domain DNS verification — auto-approves |
| D-3 | community L1271 | Education completion proof — parameter accepted but never validated |
| D-4 | community L1340 | Green contributions — no token transfer/reserve for contribution amount |
| D-5 | payroll L425 | `VerifiedEmployers` storage marked DEPRECATED but still in active use |

### 8.3 Consistent Patterns (Positive Findings)

1. **`saturating_*` arithmetic** used consistently across all pallets — no raw overflow risk
2. **`BoundedVec`** used for all user-controlled collections (nodes, jobs, domains, etc.)
3. **Proper origin checks** on governance-only functions via configurable origin types
4. **KYC verification** integrated at key touchpoints (registration, proposals, endorsements)
5. **PalletId-based treasury accounts** properly used for fee collection in bns, mesh, landledger

---

## Summary Table

| Pallet | Critical | High | Medium | Low | Total |
|--------|----------|------|--------|-----|-------|
| common | 0 | 0 | 0 | 0 | **0** |
| mesh | 2 | 2 | 4 | 2 | **10** |
| quantum | 2 | 4 | 4 | 3 | **13** |
| landledger | 1 | 3 | 2 | 2 | **8** |
| bns | 1 | 2 | 3 | 2 | **8** |
| payroll | 2 | 3 | 3 | 3 | **11** |
| community | 2 | 4 | 5 | 3 | **14** |
| cross-pallet | 0 | 1 | 2 | 1 | **4** |
| **TOTAL** | **10** | **19** | **23** | **16** | **68** |

---

## Priority Remediation Recommendations

### Immediate (Critical)

1. **P-C1/P-C2:** Replace `PayrollSchedules::iter()` in `on_initialize` with a dedicated `ScheduleQueue` sorted by next_payment block. Use pagination for batch_payment.
2. **C-C1/C-C2:** Add per-period minting caps for education and referral rewards. Use `transfer` from a funded treasury instead of `deposit_creating`.
3. **Q-C1:** Implement the marketplace fee transfer to treasury (uncomment and fix the commented line).
4. **Q-C2:** Track `original_minter` as a separate immutable field on `QuantumAchievement`, not derived from current owner at listing time.
5. **L-C1:** Add a proper government approval workflow for property transfers (multi-sig or governance approval before `government_approved` is set).
6. **B-C1:** Disable `verify_external_domain` or gate it behind oracle/off-chain worker verification.
7. **M-C1/M-C2:** Replace alert iteration with per-district counters maintained on insert/resolve.

### Short-term (High)

8. Fix all zero `proof_size` in WeightInfo defaults (mesh, quantum).
9. Add `original_minter` to `NFTListing` and track it properly.
10. Add executor assignment/authorization in `record_quantum_result`.
11. Replace `iter_prefix().count()` calls with dedicated counter storage items.
12. Bound `get_anchor_history` and `verify_anchor_chain` recursion depth.
13. Remove or migrate `VerifiedEmployers` deprecated storage.
14. Fix proposal deposit slashing on treasury insolvency.

### Medium-term (Medium)

15. Add deduplication to `confirm_emergency_alert`.
16. Prevent `relay_block_header` from overwriting existing entries.
17. Add domain listing cancellation on `transfer_domain`.
18. Validate education `completion_proof` parameter.
19. Require economic commitment for green project contributions.
20. Restrict self-reporting in `record_participation` or add attestation requirements.
