# Financial Pallet APIs

**Staking • BelizeX • Treasury**

Comprehensive API reference for BelizeChain's financial infrastructure pallets.

---

## Staking Pallet

**Proof of Useful Work (PoUW): validator set, federated-learning tasks, model deltas, quantum/domain contributions**

> This is **not** Substrate's `pallet_staking`. There is no nominator concept, no `bond`/`nominate`/
> `validate`, and no `Ledger`/`Bonded` storage. Validators join with `join_validators`.

### Extrinsics

```rust
// call_index(0) — join the validator set with stake + compute capacity
pub fn join_validators(origin: OriginFor<T>, stake: <T::Currency as Currency<T::AccountId>>::Balance, compute_capacity: u32, location: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;

// call_index(1) — leave the set (starts unbonding)
pub fn leave_validators(origin: OriginFor<T>) -> DispatchResult;

// call_index(9) — withdraw after the unbonding period
pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResult;

// call_index(10) — slash a validator
pub fn report_validator_offense(origin: OriginFor<T>, validator: T::AccountId, slash_percent: u32, reason_code: u8) -> DispatchResult;

// call_index(2) — submit a federated-learning model delta (commitment-checked)
pub fn submit_model_delta(origin: OriginFor<T>, task_id: u32, encrypted_delta: BoundedVec<u8, ConstU32<1024>>, computation_commitment: [u8; 32], computation_log: [u8; 32]) -> DispatchResult;

// call_index(3) — governance assigns an FL task
pub fn assign_fl_task(origin: OriginFor<T>, task_id: u32, model_hash: [u8; 32], computation_time: u32, reward_multiplier: Perbill, deadline_blocks: BlockNumberFor<T>) -> DispatchResult;

// call_index(4) — distribute accumulated epoch rewards
pub fn distribute_rewards(origin: OriginFor<T>) -> DispatchResult;

// call_index(5) — record a quantum-work contribution
pub fn record_quantum_contribution(origin: OriginFor<T>, job_id: BoundedVec<u8, ConstU32<64>>, validator: T::AccountId, num_qubits: u16, circuit_depth: u32, num_shots: u32, accuracy_score: u8) -> DispatchResult;

// call_index(6) — governance force-joins a validator
pub fn force_join_validator(origin: OriginFor<T>, who: T::AccountId, stake: <T::Currency as Currency<T::AccountId>>::Balance, compute_capacity: u32, location: BoundedVec<u8, ConstU32<64>>) -> DispatchResult;

// call_index(7) — record a domain contribution
pub fn record_domain_contribution(origin: OriginFor<T>, operator: T::AccountId, domain: u8, quality_score: u8, volume_kb: u32) -> DispatchResult;

// call_index(8) — claim PoUW rewards with domain bonus
pub fn claim_pouw_with_domain_bonus(origin: OriginFor<T>) -> DispatchResult;
```

### Storage

| Storage | Purpose |
|---|---|
| `Validators` | `AccountId → { account, stake, computeCapacity, location, complianceScore, lastFlContribution, qualityScore, timelinessScore, honestyScore, totalContributions }` |
| `ValidatorCount` | Active validator count |
| `CurrentEpoch` · `EpochRewards` · `LastClaimedEpoch` | Epoch accounting |
| `ActiveFLTask` | Currently assigned FL task |
| `ModelSubmissions` | Submitted model deltas |
| `PendingUnbonds` | `AccountId → (amount, epoch)` |
| `SlashingSpans` | `AccountId → u32` |
| `QuantumContributions` · `ValidatorQuantumStatsMap` | Quantum work per validator |
| `EpochDomainContributions` · `OperatorDomainStatsMap` · `OperatorEpochContributions` | Domain contributions |

Exact value structs: see `pallets/staking/src/lib.rs`.

### Events

`ValidatorJoined`, `ValidatorLeft`, `FLTaskAssigned`, `ModelDeltaSubmitted`, `RewardsDistributed`,
`ValidatorSlashed`, `EpochCompleted`, `QuantumContributionRecorded`, `DomainContributionRecorded`,
`PouWRewardsClaimedWithBonus`, `UnbondingStarted`, `StakeWithdrawn`

> Weights: `pallets/staking/src/weights.rs` (`./scripts/bench_weights.sh pallet_belize_staking`).

---

## BelizeX Pallet

**On-chain AMM (constant-product with oracle guards) + limit-order book + tourism trader discounts**

> Asset identifiers are `u8` codes, not a `BoundedVec`/`AssetId` type. There is no generic
> `swap` extrinsic and no `AssetRegistry` storage — the trades are `execute_trade` (single pair)
> and `execute_multihop_trade` (path).

### Extrinsics

```rust
// call_index(0) — create a pair. Assets are u8 codes; fee_rate in parts-per-million (or similar)
pub fn create_trading_pair(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, fee_rate: u32) -> DispatchResult;

// call_index(1) — add liquidity
pub fn add_liquidity(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, base_amount: <T::Currency as Currency<T::AccountId>>::Balance, quote_amount: <T::Currency as Currency<T::AccountId>>::Balance, min_lp_tokens: u128) -> DispatchResult;

// call_index(2) — AMM swap on a single pair (oracle-guarded)
pub fn execute_trade(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, amount_in: <T::Currency as Currency<T::AccountId>>::Balance, min_amount_out: u128, is_tourism_trade: bool) -> DispatchResult;

// call_index(3) — register a tourism trader (discount eligibility)
pub fn register_tourism_trader(origin: OriginFor<T>, trader: T::AccountId) -> DispatchResult;

// call_index(4) — place a limit order
pub fn place_limit_order(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, order_type: u8, amount: u128, price: u128, expires_in_blocks: BlockNumberFor<T>) -> DispatchResult;

// call_index(5) — multihop swap along a path
pub fn execute_multihop_trade(origin: OriginFor<T>, path: Vec<u8>, amount_in: <T::Currency as Currency<T::AccountId>>::Balance, min_amount_out: u128, is_tourism_trade: bool) -> DispatchResult;

// call_index(6)–(8) — governance controls
pub fn pause_global(origin: OriginFor<T>) -> DispatchResult;
pub fn resume_global(origin: OriginFor<T>) -> DispatchResult;
pub fn set_pair_status(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, active: bool) -> DispatchResult;

// call_index(9) — remove liquidity
pub fn remove_liquidity(origin: OriginFor<T>, base_asset: u8, quote_asset: u8, lp_tokens: u128, min_base_amount: u128, min_quote_amount: u128) -> DispatchResult;

// call_index(10) — cancel a resting order
pub fn cancel_order(origin: OriginFor<T>, order_id: u32) -> DispatchResult;
```

### Storage

| Storage | Purpose |
|---|---|
| `TradingPairs` | Pair reserves, fee rate, status |
| `OrderBook` | Resting limit orders |
| `NextOrderId` · `AccountOrderCount` | Order bookkeeping |
| `LPBalances` · `LiquidityProviders` | LP positions |
| `TourismTraders` | Registered tourism traders |
| `GlobalPaused` | DEX-wide pause flag |
| `DevSeedEnabled` · `DevSeedDone` | Dev/testnet seed gating |

Exact value structs: see `pallets/belizex/src/lib.rs`.

### Events

`TradingPairCreated`, `LiquidityAdded`, `OracleGuardRejected`, `OracleRateUnavailable`,
`LiquidityRemoved`, `TradeExecuted`, `OrderPlaced`, `OrderExecuted`, `OrderCancelled`,
`TourismTraderVerified`, `DexPaused`, `DexResumed`, `PairStatusUpdated`

> Weights: `pallets/belizex/src/weights.rs` (`./scripts/bench_weights.sh pallet_belize_belizex`).

---

## Treasury

**There is no separate Treasury pallet.** National treasury functions live in the
**Governance** pallet (`pallets/governance`):

| Need | Real surface |
|---|---|
| National reserve | `governance.NationalTreasuryReserve` |
| Propose a spend | `governance.propose_treasury_spend(recipient, amount, description, district_index)` — call_index 29 |
| Approve | `governance.approve_treasury_spend(proposal_id)` — call_index 30 |
| Execute | `governance.execute_treasury_proposal(proposal_id)` — call_index 31 |
| District budgets | `allocate_district_budget`, `transfer_district_budget`; storage `DistrictBudgets` |
| Department balances | `governance.DepartmentTreasuryBalances` |
| Spend proposals | `governance.TreasurySpendProposals` (+ `TreasurySpendTracker`, `NextTreasuryProposalId`) |

> **Corrections:** the previously documented `propose_spend`, `approve_proposal`,
> `reject_proposal` extrinsics and the `Proposals` / `TreasuryBalance` / `TreasuryCouncil`
> storage items **do not exist**. See the Governance section in
> [Core Pallet APIs](./pallet-apis-core.md) for the full treasury surface.

---

## Type Definitions

Enums and type aliases are defined in the pallets — read the source or query runtime metadata
rather than copying by hand:

- **`OrderType`** — `pallets/belizex`: limit-order side/type, passed as a `u8` index.
- **Asset identifiers** in BelizeX are `u8` codes (not a `BoundedVec`/`AssetId`).
- **Domain index** in `record_domain_contribution` is a `u8`.
- **`WorkerType`**, **`EmployerType`**, **`DeductionType`**, **`PaymentCategory`** — `pallets/payroll`.
- **`EncumbranceType`** — `pallets/landledger`.

> **Corrections:** `RewardDestination`, `OrderSide`, `AssetId = BoundedVec<u8, 8>`,
> `TradingPairId`, and `OrderId` as previously documented do not exist in the runtime. Staking
> has no reward-destination concept (no nominators); BelizeX order ids are `u32`.

---

## Performance Benchmarks

On-chain weights are generated, not hand-estimated:

```bash
cargo build --release -p belizechain-node --features runtime-benchmarks
./scripts/bench_weights.sh pallet_belize_staking
./scripts/bench_weights.sh pallet_belize_belizex
./scripts/bench_weights.sh pallet_belize_governance   # treasury lives here
```

Output lands in each `pallets/<name>/src/weights.rs`. For throughput, measure against the live
testnet — fixed "ops/second" and "gas" figures in documentation are not authoritative.

---

## Related Documentation

- [Core Pallet APIs](./pallet-apis-core.md)
- [Infrastructure Pallet APIs](./pallet-apis-infrastructure.md)
- [Services Pallet APIs](./pallet-apis-services.md)
- [Economics Overview](../economics/tokenomics.md)
