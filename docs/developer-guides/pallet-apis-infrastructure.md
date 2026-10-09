# Infrastructure Pallet APIs

**Oracle • Interoperability • Consensus • Quantum**

Comprehensive API reference for BelizeChain's infrastructure pallets.

---

## Oracle Pallet

**Merchant verification, price feeds, sanctions, KYC consensus, land mirror, and IoT feeds**

### Extrinsics

```rust
// call_index(0) — governance
pub fn add_operator(origin: OriginFor<T>, operator: T::AccountId) -> DispatchResult;

// call_index(1) — governance
pub fn remove_operator(origin: OriginFor<T>, operator: T::AccountId) -> DispatchResult;

// call_index(2) — authorised operator submits a price observation
pub fn submit_price(origin: OriginFor<T>, base_currency: u8, quote_currency: u8, price: u128) -> DispatchResult;

// call_index(3) — verify a merchant for a tourism category
pub fn verify_merchant(
    origin: OriginFor<T>,
    merchant: T::AccountId,
    category: u8,
    certification: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
    license: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
    location: Option<(i32, i32)>,
) -> DispatchResult;

// call_index(4) — add a sanctions entry
pub fn add_sanctioned_entity(
    origin: OriginFor<T>,
    account: T::AccountId,
    source: u8,
    reason: BoundedVec<u8, ConstU32<MAX_REASON_LEN>>,
    expires_at: Option<BlockNumberFor<T>>,
) -> DispatchResult;

// call_index(5) — remove a sanctions entry
pub fn remove_sanction(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult;

// call_index(6) — record a KYC verification
pub fn verify_identity(
    origin: OriginFor<T>,
    account: T::AccountId,
    kyc_level: u8,
    id_hash: [u8; 32],
    provider: BoundedVec<u8, ConstU32<64>>,
    biometric_verified: bool,
    address_verified: bool,
) -> DispatchResult;

// call_index(7) — mirror a land record
pub fn register_land(
    origin: OriginFor<T>,
    property_id: PropertyId,
    owner: T::AccountId,
    valuation: u128,
    has_encumbrances: bool,
    co_owner_count: u8,
) -> DispatchResult;

// call_index(8)
pub fn register_iot_device(origin: OriginFor<T>, device_id: [u8; 32], device_type_index: u8, location: Option<(i32, i32)>) -> DispatchResult;

// call_index(9)
pub fn submit_iot_data(
    origin: OriginFor<T>,
    device_id: [u8; 32],
    feed_type_index: u8,
    domain_index: Option<u8>,
    data: BoundedVec<u8, ConstU32<MAX_DATA_LEN>>,
    data_hash: [u8; 32],
    location: Option<(i32, i32)>,
    accuracy: u8,
) -> DispatchResult;

// call_index(10)
pub fn verify_iot_device(origin: OriginFor<T>, device_id: [u8; 32]) -> DispatchResult;

// call_index(11) — operator claims accumulated rewards
pub fn claim_oracle_rewards(origin: OriginFor<T>) -> DispatchResult;

// call_index(12) — governance sets a manual exchange rate
pub fn update_exchange_rate(origin: OriginFor<T>, base_currency: u8, quote_currency: u8, price: u128) -> DispatchResult;
```

> **Correction:** this pallet *does* carry price feeds (`PriceFeeds`, `PriceSubmissions`,
> `ManualExchangeRates`). It is not merchant-verification-only. bBZD is still pegged 1:1 to
> BZD by Central Bank governance — the price feeds are general-purpose oracle data, not the peg.

### Storage

| Storage | Key → Value | Purpose |
|---|---|---|
| `OracleOperators` | `AccountId → bool` | Authorised operators |
| `OperatorCount` | `u32` | Number of authorised operators |
| `PriceSubmissions` | double map | Per-operator price observations |
| `PriceFeeds` | `(u8, u8) → _` | Aggregated price feed per currency pair |
| `ManualExchangeRates` | `(u8, u8) → u128` | Governance-set rates |
| `MerchantCategories` | `AccountId → MerchantInfo` | Verified merchants, keyed by merchant account |
| `SanctionedEntities` | `AccountId → _` | Sanctions list |
| `IdentityVerifications` | `AccountId → _` | KYC records |
| `LandRegistryData` | `PropertyId → _` | Land mirror |
| `IoTDevices` | `[u8; 32] → _` | Registered IoT devices |
| `OracleOperatorStatsMap` | `AccountId → _` | Operator reputation and rewards |
| `PendingKycSubmissions` | `(AccountId, AccountId) → u8` | Pending KYC consensus votes |
| `KycLeadingVote` | `AccountId → (u8, u32)` | Leading KYC vote |
| `OracleDisputeFlag` | `AccountId → _` | KYC dispute flags |
| `BehaviorFlags` | `AccountId → _` | Behavioural flags |
| `BehaviorFlagCooldown` | `AccountId → BlockNumber` | Flag cooldowns |
| `PendingBehaviorFlags` | `(AccountId, AccountId) → u8` | Pending behaviour-flag votes |

Exact value structs: see `pallets/oracle/src/lib.rs`.

### Events

`OperatorAdded`, `OperatorRemoved`, `PriceSubmitted`, `PriceFeedUpdated`, `MerchantVerified`,
`MerchantExpired`, `SanctionAdded`, `SanctionRemoved`, `IdentityVerified`, `IdentityExpired`,
`LandRegistryUpdated`, `IoTDeviceRegistered`, `IoTDataSubmitted`, `IoTDeviceVerified`,
`OracleRewardsClaimed`, `ExchangeRateUpdated`, `ExchangeRateStale`, `KycVoteStaged`,
`KycConsensusReached`, `KycDisputeFlagged`, `KycDisputeResolved`, `BehaviorFlagVoteStaged`,
`BehaviorFlagged`, `BehaviorFlagCleared`

> Weights live in `pallets/oracle/src/weights.rs`. Regenerate with
> `./scripts/bench_weights.sh pallet_belize_oracle` — hand-written weight figures in docs are
> not authoritative.

---

## Interoperability Pallet

**Cross-chain bridges with post-quantum (PQ) signature verification and liquidity pools**

### Extrinsics

```rust
// call_index(0) — lock assets and start an outbound bridge transaction
pub fn initiate_bridge(
    origin: OriginFor<T>,
    target_chain_index: u8,
    target_address: Vec<u8>,
    amount: <T::Currency as Currency<T::AccountId>>::Balance,
    asset_index: u8,
) -> DispatchResult;

// call_index(1) — bridge validator supplies a PQ signature for a tx
pub fn provide_pq_signature(origin: OriginFor<T>, tx_id: u32, pq_signature: Vec<u8>) -> DispatchResult;

// call_index(2)
pub fn create_liquidity_pool(
    origin: OriginFor<T>,
    chain_index: u8,
    asset_index: u8,
    initial_liquidity: <T::Currency as Currency<T::AccountId>>::Balance,
) -> DispatchResult;

// call_index(3) — release locked assets after multi-sig + challenge period
pub fn process_unlock(origin: OriginFor<T>, tx_id: u32) -> DispatchResult;

// call_index(4)
pub fn send_cross_chain_message(origin: OriginFor<T>, target_chain_index: u8, payload: Vec<u8>) -> DispatchResult;

// call_index(5) — governance
pub fn update_bridge_config(origin: OriginFor<T>, chain_index: u8, enabled: bool, fee_rate: u32, max_amount: u128) -> DispatchResult;

// call_index(6)
pub fn dispute_bridge_transaction(origin: OriginFor<T>, tx_id: u32, reason: Vec<u8>) -> DispatchResult;

// call_index(7) — record an unlock originating on the source chain
pub fn submit_incoming_unlock(
    origin: OriginFor<T>,
    source_chain_index: u8,
    source_tx_hash: Vec<u8>,
    recipient: T::AccountId,
    amount: u128,
    asset_index: u8,
) -> DispatchResult;

// call_index(8)
pub fn register_bridge_validator(origin: OriginFor<T>, pq_public_key: Vec<u8>, supported_chain_indices: Vec<u8>) -> DispatchResult;

// call_index(9)
pub fn remove_bridge_validator(origin: OriginFor<T>, validator_account: T::AccountId) -> DispatchResult;

// call_index(10)
pub fn withdraw_liquidity(
    origin: OriginFor<T>,
    pool_id: u32,
    amount: <T::Currency as Currency<T::AccountId>>::Balance,
) -> DispatchResult;

// call_index(11) — confirm an oracle-attested burn proof
pub fn confirm_burn_proof(origin: OriginFor<T>, tx_id: u32) -> DispatchResult;
```

**Chain selection:** `target_chain_index` / `chain_index` index into the `BridgeChain` enum
(52 variants) used by `ChainConfigurations`, not a small `ChainId` list.

### Storage

| Storage | Purpose |
|---|---|
| `BridgeValidators` | Registered bridge validators + PQ keys |
| `LiquidityPools` | Per-pool liquidity (`pool_id → _`) |
| `BridgeTransactions` | Outbound/inbound bridge transactions (`tx_id → _`) |
| `CrossChainMessages` | Arbitrary cross-chain messages |
| `ChainConfigurations` | Per-`BridgeChain` config (enabled, fee rate, max amount, PQ requirements, endpoints) |
| `NextTxId` / `NextPoolId` / `NextMessageId` | Id counters |
| `TotalLockedAssets` | Locked assets per chain/asset |
| `PendingFinalizations` | Transactions awaiting finalization |
| `UserBridgeLocks` | Per-user locked amount |
| `BridgeCallsThisBlock` | Per-account rate-limit counter |
| `BurnConfirmations` | Per-(tx, oracle) burn confirmations |
| `BurnConfirmationCount` | Confirmation tally per tx |

Exact value structs: see `pallets/interoperability/src/lib.rs`.

### Events

`BridgeTransactionInitiated`, `BridgeTransactionExecuted`, `LiquidityPoolCreated`,
`AssetsLocked`, `AssetsUnlocked`, `BridgeValidatorRegistered`, `PQSignatureProvided`,
`CrossChainMessageSent`, `BridgeConfigUpdated`, `BridgeFeeCollected`,
`BridgeTransactionDisputed`, `BridgeTransactionFinalized`, `UnverifiedBurnProofWarning`,
`BurnProofConfirmed`

> Weights: `pallets/interoperability/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_interoperability`).

---

## Consensus Pallet

**Proof of Useful Work over AI models (register → validate → round → submit work → finalize)**

> This pallet does **not** expose `submit_quantum_work`. Quantum work is submitted through the
> Quantum pallet (`quantum.submit_quantum_job`); see the Quantum section below.

### Extrinsics

```rust
// call_index(0) — register an AI model with a PQ signature
pub fn register_ai_model(
    origin: OriginFor<T>,
    model_type_index: u8,
    parameters_hash: [u8; 32],
    training_data_size: u32,
    pq_signature: Vec<u8>,
) -> DispatchResult;

// call_index(1) — join the validator set (PoUW staking)
pub fn join_consensus_validator(
    origin: OriginFor<T>,
    stake_amount: <T::Currency as Currency<T::AccountId>>::Balance,
    pq_public_key: Vec<u8>,
) -> DispatchResult;

// call_index(2) — governance validates a model and scores its accuracy
pub fn validate_ai_model(origin: OriginFor<T>, model_id: u32, accuracy_score: u32) -> DispatchResult;

// call_index(3) — governance opens a consensus round
pub fn start_consensus_round(origin: OriginFor<T>, duration_blocks: BlockNumberFor<T>) -> DispatchResult;

// call_index(4) — validator submits useful work for a model
pub fn submit_ai_work(
    origin: OriginFor<T>,
    model_id: u32,
    work_type_index: u8,
    result_hash: [u8; 32],
    computation_time: u32,
    pq_signature: Vec<u8>,
) -> DispatchResult;

// call_index(5) — close the round and distribute rewards
pub fn finalize_consensus_round(origin: OriginFor<T>) -> DispatchResult;

// call_index(6) / call_index(7) — leave the set, then withdraw after unbonding
pub fn leave_validator(origin: OriginFor<T>) -> DispatchResult;
pub fn withdraw_validator_unbonded(origin: OriginFor<T>) -> DispatchResult;
```

### Storage

| Storage | Purpose |
|---|---|
| `AIModels` | Registered models (`model_id → _`) |
| `ConsensusValidators` | Validator records (`validator_id → _`) |
| `ValidatorByAccount` | `AccountId → validator_id` |
| `ConsensusRounds` | Round records (`round_id → _`) |
| `ModelsByAccount` | Models owned per account |
| `CurrentConsensusRound` | Active round id |
| `NextModelId` / `NextValidatorId` / `NextRoundId` | Id counters |
| `GlobalAIMetrics` | Aggregate system metrics |
| `SubmitCallsThisBlock` | Per-account submit rate limit |
| `LastSubmitRateLimitBlock` | Block the counters were last reset |
| `PendingValidatorUnbonds` | Unbonding queue |

Exact value structs: see `pallets/consensus/src/lib.rs`.

### Events

`AIModelRegistered`, `ValidatorJoined`, `ConsensusRoundStarted`, `AIWorkSubmitted`,
`ConsensusRoundCompleted`, `ConsensusRewardsDistributed`, `ModelQualityUpdated`, `ValidatorLeft`

> Weights: `pallets/consensus/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_consensus`).

---

## Quantum Pallet

**On-chain registry for Kinich quantum jobs, results, verification consensus, and achievement NFTs**

> This pallet does **not** expose `request_quantum_compression` or `register_quantum_backend`.
> Backends are an enum compiled into the runtime (`QuantumBackend`), not registered via
> extrinsic, and quantum *compression* is an off-chain Kinich/Pakit concern with no on-chain
> extrinsic.

### Extrinsics

```rust
// call_index(0) — submit a quantum job (job_id comes from Kinich)
pub fn submit_quantum_job(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    backend_index: u8,          // index into QuantumBackend
    circuit_hash: [u8; 32],
    num_qubits: u16,
    circuit_depth: u32,
    num_shots: u32,
) -> DispatchResult;

// call_index(1) / call_index(2) — execution lifecycle
pub fn update_job_status(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    new_status_index: u8,
) -> DispatchResult;
pub fn record_quantum_result(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    result_data_hash: [u8; 32],
    verification_proof: BoundedVec<u8, ConstU32<MAX_PROOF_SIZE>>,
    accuracy_score: u8,
) -> DispatchResult;

// call_index(3) — verifier attests a result
pub fn verify_quantum_result(origin: OriginFor<T>, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, verification_passed: bool) -> DispatchResult;

// call_index(4) — mint an achievement NFT for completed work
pub fn mint_achievement_nft(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    achievement_type_index: u8,
    transferable: bool,
    _circuit_qubits: u16,
    _accuracy: u8,
) -> DispatchResult;

// call_index(5)–(8) — NFT transfer / marketplace
pub fn transfer_nft(origin: OriginFor<T>, nft_id: u64, to: T::AccountId) -> DispatchResult;
pub fn list_nft(origin: OriginFor<T>, nft_id: u64, price: <T::Currency as Currency<T::AccountId>>::Balance, duration: BlockNumberFor<T>) -> DispatchResult;
pub fn buy_nft(origin: OriginFor<T>, nft_id: u64) -> DispatchResult;
pub fn delist_nft(origin: OriginFor<T>, nft_id: u64) -> DispatchResult;

// call_index(9) / call_index(10) — multi-verifier consensus
pub fn request_verification(origin: OriginFor<T>, job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>, required_verifications: u8) -> DispatchResult;
pub fn submit_verification(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    vote_index: u8,      // index into VerificationVote
    confidence: u8,
) -> DispatchResult;

// call_index(11)–(13) — NFT bridging
pub fn bridge_to_ethereum(origin: OriginFor<T>, nft_id: u64, recipient: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;
pub fn bridge_to_parachain(origin: OriginFor<T>, nft_id: u64, parachain_id: u32, recipient: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;
pub fn cancel_bridge(origin: OriginFor<T>, nft_id: u64) -> DispatchResult;
```

### Backends

`QuantumBackend` is a runtime enum indexed by `backend_index`:
`AzureIonQ`, `AzureQuantinuum`, `AzureRigetti`, `IBMQuantum`, `Qiskit`, `SpinQGemini`,
`SpinQTriangulum`, `Other`.

### Storage

| Storage | Purpose |
|---|---|
| `QuantumJobs` | `JobId → QuantumJob` |
| `JobsByAccount` | Per-account job id index |
| `QuantumResults` | `JobId → QuantumResult` |
| `QuantumAchievements` | Achievement NFTs per account |
| `VerificationRequests` | Multi-verifier requests per job |
| `ValidatorReputation` | Verifier reputation scores |
| `AccountStats` | Per-account job/reward stats |
| `NFTCounter` / `ListingCounter` / `BridgeCounter` | Id counters |
| `TotalQuantumJobs` / `TotalDallaSpent` | Aggregate counters |
| `NFTListings` | Marketplace listings |
| `NFTAuctions` | Auctions |
| `BridgeRequests` | NFT bridge requests |

Exact value structs: see `pallets/quantum/src/lib.rs`.

### Events

`QuantumJobSubmitted`, `JobStatusUpdated`, `QuantumResultRecorded`, `ResultVerified`,
`AchievementNFTMinted`, `NFTTransferred`, `VerificationRequested`, `VerificationSubmitted`,
`VerificationConsensusReached`, `ReputationUpdated`, `NFTListed`, `NFTPurchased`, `NFTDelisted`,
`BridgeInitiated`, `BridgeCancelled`

> Weights: `pallets/quantum/src/weights.rs`
> (`./scripts/bench_weights.sh pallet_belize_quantum`).

---

## Type Definitions

Enums referenced above (all defined in the pallets, not here — this list is a pointer, not a copy):

- **`BridgeChain`** — `pallets/interoperability`: the bridge target-chain enum (~52 variants).
  `initiate_bridge` / `update_bridge_config` take an index into it.
- **`QuantumBackend`** — `pallets/quantum`: `AzureIonQ`, `AzureQuantinuum`, `AzureRigetti`,
  `IBMQuantum`, `Qiskit`, `SpinQGemini`, `SpinQTriangulum`, `Other`.
- **`RoundStatus`** — `pallets/consensus`: `InProgress`, `Completed`.
- **`JobStatus`**, **`VerificationVote`**, **`AchievementType`** — `pallets/quantum`.

Do not hand-copy struct/enum definitions into documentation; query the runtime metadata
(`@polkadot/api`) or read the pallet source for the authoritative shape.

---

## Performance Benchmarks

On-chain weights are generated by the FRAME benchmarking tooling, not estimated by hand:

```bash
cargo build --release -p belizechain-node --features runtime-benchmarks
./scripts/bench_weights.sh pallet_belize_oracle
./scripts/bench_weights.sh pallet_belize_interoperability
./scripts/bench_weights.sh pallet_belize_consensus
./scripts/bench_weights.sh pallet_belize_quantum
```

Output lands in each `pallets/<name>/src/weights.rs`. For chain-level throughput and
cross-chain latency, measure against the live testnet rather than quoting fixed figures —
values differ per hardware and per bridge endpoint.

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Financial Pallet APIs](./pallet-apis-financial.md)
- [Services Pallet APIs](./pallet-apis-services.md)
- Bridge Documentation
- [Kinich Quantum Integration](../architecture/kinich-integration.md)
