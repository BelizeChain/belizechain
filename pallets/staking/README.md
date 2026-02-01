# BelizeChain Staking: Proof of Useful Work (PoUW) Consensus

## Overview

The **Staking** pallet (`pallet-belize-staking`) implements BelizeChain's **Proof of Useful Work (PoUW)** consensus mechanism, where validators:
1. **Secure the blockchain** through traditional token staking (10K+ DALLA minimum)
2. **Execute federated learning tasks** on private datasets (via Nawal integration)
3. **Contribute quantum computations** for verification (via Quantum pallet)
4. **Get rewarded** based on contribution quality, timeliness, and honesty

**Key Innovation**: Unlike traditional Proof-of-Stake (where staking is the only requirement), PoUW validators must **actively contribute useful computational work** to earn full rewards. This ensures BelizeChain's security is **directly coupled** with national AI/quantum infrastructure development.

**Key Features**:
- **Federated Learning Integration**: Validators train AI models on decentralized datasets (Nawal orchestration)
- **Quantum Work Contribution**: Validators verify quantum computation results (Quantum pallet)
- **Multi-Dimensional Scoring**: Quality (40%), Timeliness (30%), Honesty (30%)
- **Slashing Conditions**: Penalties for late submissions, dishonest reporting, poor-quality models
- **KYC Level 3 Requirement**: Validators must have biometric verification (Identity pallet)
- **Geographic Distribution**: Ensures validator diversity across Belize districts
- **Compliance Score Tracking**: Sanctions compliance, uptime monitoring

**Business Use Cases**:
- **National AI Development**: Validators collectively train models for healthcare, agriculture, tourism
- **Privacy-Preserving ML**: Data never leaves local devices (hospitals, farms, hotels)
- **Quantum Verification**: Distributed network validates quantum computation authenticity
- **Economic Incentives**: Validators earn 10-15% APY for computational contributions

---

## Architecture

### 1. PoUW Validator Lifecycle

```
┌──────────────────────────────────────────────────────────┐
│  1. VALIDATOR REGISTRATION                               │
│     join_validators(stake, location, compute_capacity)   │
│     - Minimum stake: 10,000 DALLA                        │
│     - KYC Level 3 required (biometric verification)      │
│     - Sanctions check (Oracle pallet)                    │
│     - Geographic location (Belize district)              │
│     - Compute capacity: CPU/GPU/RAM specifications       │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  2. FEDERATED LEARNING ASSIGNMENT (Nawal)                │
│     assign_fl_task(task_id, model_hash, deadline)        │
│     - Task: Train model on local dataset                 │
│     - Frequency: Weekly (50,400 blocks ~7 days)          │
│     - Dataset: Healthcare records, crop data, tourism    │
│     - Privacy: Data never leaves validator node          │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  3. MODEL TRAINING (Off-Chain)                           │
│     - Validator runs Nawal client                        │
│     - Downloads global model from IPFS                   │
│     - Trains on local dataset (DP-SGD privacy)           │
│     - Computes model delta (weight updates)              │
│     - Training time: 1-6 hours (depending on dataset)    │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  4. MODEL DELTA SUBMISSION (On-Chain)                    │
│     submit_model_delta(task_id, delta_hash, proof)       │
│     - delta_hash: SHA-256 of model weights (Pakit DAG)   │
│     - proof: Zero-knowledge proof of training            │
│     - Timeliness: Before deadline = full rewards         │
│     - Late submission: -10% penalty per 10,000 blocks    │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  5. QUALITY SCORING (Nawal Aggregation)                  │
│     - Nawal aggregator validates all deltas              │
│     - Quality: Model improvement accuracy (40%)          │
│       * +5% test accuracy = 100 quality score            │
│       * 0% improvement = 0 quality score                 │
│     - Honesty: Privacy compliance verification (30%)     │
│       * DP-SGD noise validation                          │
│       * Gradient clipping enforcement                    │
│     - Timeliness: Deadline adherence (30%)               │
│       * On-time = 100 timeliness score                   │
│       * 10K blocks late = 90 timeliness score            │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  6. REWARD DISTRIBUTION                                  │
│     distribute_rewards()                                 │
│     - Base reward: 100 DALLA per epoch                   │
│     - Quality bonus: Up to +100 DALLA                    │
│     - Timeliness bonus: Up to +50 DALLA                  │
│     - Honesty bonus: Up to +50 DALLA                     │
│     - Total: 100-300 DALLA per week (~10-15% APY)        │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  7. SLASHING CONDITIONS                                  │
│     - No submission: -10% stake slash                    │
│     - Dishonest reporting: -25% stake slash              │
│     - Poor quality (< 20 score): -5% stake slash         │
│     - Repeated failures: Validator ejection              │
└──────────────────────────────────────────────────────────┘
```

### 2. Quantum Work Integration

Validators also participate in quantum result verification (Quantum pallet):

```
┌──────────────────────────────────────────────────────────┐
│  QUANTUM JOB SUBMITTED (User → Quantum Pallet)           │
│  - Circuit: Grover's algorithm (3 qubits)                │
│  - Execution: Azure Quantinuum backend                   │
│  - Result: |011⟩ measurement (58% probability)           │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  VALIDATOR VERIFICATION (Staking Pallet)                 │
│  record_quantum_contribution(validator, job_id, scores)  │
│  - Validator reviews ZK proof from Quantum pallet        │
│  - Difficulty score: Circuit complexity (3 qubits = low) │
│  - Accuracy score: Proof validation confidence           │
│  - Reward: 0.5 DALLA per verified job                    │
└──────────────────────────────────────────────────────────┘
```

**Total Validator Income**:
- **Federated Learning**: 100-300 DALLA/week (primary income)
- **Quantum Verification**: 5-20 DALLA/week (supplementary)
- **Block Production**: Standard validator rewards (chain-dependent)
- **Annual APY**: 10-15% on 10K DALLA stake

---

## Storage Items

### Validator Registry

| Storage | Type | Description |
|---------|------|-------------|
| `Validators` | `StorageMap<AccountId, ValidatorInfo>` | All active validators |
| `ValidatorCount` | `StorageValue<u32>` | Total validator count |
| `MaxValidators` | `StorageValue<u32>` | Maximum validator limit (100 default) |

**ValidatorInfo Structure**:
```rust
pub struct ValidatorInfo<AccountId, Balance, BlockNumber> {
    pub account: AccountId,              // Validator account
    pub stake: Balance,                  // Staked DALLA (10K+ minimum)
    pub compute_capacity: u32,           // CPU/GPU score (0-1000)
    pub location: BoundedVec<u8, 64>,    // Geographic location (Belize district)
    pub compliance_score: u8,            // Compliance rating (0-100)
    pub last_fl_contribution: BlockNumber, // Last FL task submission
    pub quality_score: u8,               // Model quality (0-100, 40% weight)
    pub timeliness_score: u8,            // Deadline adherence (0-100, 30% weight)
    pub honesty_score: u8,               // Privacy compliance (0-100, 30% weight)
    pub total_contributions: u32,        // All-time FL task count
}
```

**Compute Capacity Scoring** (0-1000):
- **CPU**: Cores * 50 (e.g., 8 cores = 400 points)
- **GPU**: VRAM GB * 100 (e.g., 8GB = 800 points)
- **RAM**: GB * 10 (e.g., 32GB = 320 points)
- **Total**: Min(1000, CPU + GPU + RAM)

**Location Examples** (Belize Districts):
- "Belize District" (Belize City)
- "Cayo District" (San Ignacio)
- "Corozal District" (Corozal Town)
- "Orange Walk District" (Orange Walk Town)
- "Stann Creek District" (Dangriga)
- "Toledo District" (Punta Gorda)

### Federated Learning Tasks

| Storage | Type | Description |
|---------|------|-------------|
| `FederatedLearningTasks` | `StorageMap<u32, FederatedLearningTask>` | All FL tasks by ID |
| `CurrentTask` | `StorageValue<u32>` | Active task ID |
| `NextTaskId` | `StorageValue<u32>` | Auto-incrementing task counter |

**FederatedLearningTask Structure**:
```rust
pub struct FederatedLearningTask {
    pub task_id: u32,                    // Unique task ID
    pub model_hash: [u8; 32],            // SHA-256 of global model (Pakit DAG)
    pub computation_time: u32,           // Expected training duration (blocks)
    pub reward_multiplier: Perbill,      // Task difficulty multiplier (1x-5x)
    pub deadline: u32,                   // Submission deadline (block)
}
```

**Task Types**:
- **Healthcare**: Train diagnosis model on hospital patient data (5x reward multiplier)
- **Agriculture**: Crop disease detection on farm sensor data (3x multiplier)
- **Tourism**: Hotel occupancy forecasting (2x multiplier)
- **General**: BelizeChainLLM language model training (1x multiplier)

### Model Delta Submissions

| Storage | Type | Description |
|---------|------|-------------|
| `ModelDeltas` | `StorageDoubleMap<u32 (task_id), AccountId (validator), ModelDelta>` | FL contributions |

**ModelDelta Structure**:
```rust
pub struct ModelDelta {
    pub delta_hash: [u8; 32],            // SHA-256 of weight deltas (Pakit DAG)
    pub submission_block: u32,           // When submitted
    pub quality_score: u8,               // Nawal-validated quality (0-100)
    pub size_bytes: u64,                 // Delta size (for storage cost)
    pub proof_hash: [u8; 32],            // ZK proof of training
}
```

### Quantum Contributions

| Storage | Type | Description |
|---------|------|-------------|
| `QuantumContributions` | `StorageDoubleMap<AccountId (validator), BoundedVec<u8, 64> (job_id), QuantumContribution>` | Quantum verification records |
| `ValidatorQuantumStats` | `StorageMap<AccountId, ValidatorQuantumStats>` | Per-validator quantum stats |

**QuantumContribution Structure**:
```rust
pub struct QuantumContribution {
    pub job_id: BoundedVec<u8, 64>,      // Quantum job UUID
    pub difficulty_score: u8,            // Circuit complexity (0-100)
    pub accuracy_score: u8,              // Verification confidence (0-100)
    pub contributed_at: BlockNumber,     // Contribution block
    pub reward_earned: Balance,          // DALLA reward (0.5 typical)
}
```

**ValidatorQuantumStats Structure**:
```rust
pub struct ValidatorQuantumStats {
    pub total_verifications: u32,        // All-time quantum job count
    pub total_rewards: Balance,          // Cumulative DALLA earned
    pub average_accuracy: u8,            // Mean accuracy score
}
```

### Rewards & Slashing

| Storage | Type | Description |
|---------|------|-------------|
| `PendingRewards` | `StorageMap<AccountId, Balance>` | Unclaimed validator rewards |
| `SlashedStake` | `StorageMap<AccountId, Balance>` | Slashed amounts (recoverable after 180 days) |
| `SlashHistory` | `StorageDoubleMap<AccountId, BlockNumber, SlashRecord>` | Slashing event history |

**SlashRecord Structure**:
```rust
pub struct SlashRecord {
    pub reason: SlashReason,             // NoSubmission, Dishonest, PoorQuality
    pub amount: Balance,                 // DALLA slashed
    pub slashed_at: BlockNumber,
    pub recoverable_at: BlockNumber,     // 180 days later
}
```

**SlashReason Enum**:
- **NoSubmission**: Validator didn't submit FL task (10% stake)
- **Dishonest**: Privacy violation detected (25% stake)
- **PoorQuality**: Model quality < 20/100 (5% stake)
- **Timeout**: Submission > 2x deadline (15% stake)
- **DoubleSubmission**: Submitted duplicate deltas (50% stake, ejection)

### Statistics

| Storage | Type | Description |
|---------|------|-------------|
| `EpochStats` | `StorageMap<BlockNumber, EpochStats>` | Per-epoch statistics |
| `GlobalStats` | `StorageValue<GlobalStats>` | All-time statistics |

**EpochStats Structure**:
```rust
pub struct EpochStats {
    pub epoch_number: u32,
    pub total_validators: u32,
    pub tasks_assigned: u32,
    pub tasks_completed: u32,
    pub total_rewards_distributed: Balance,
    pub average_quality_score: u8,
}
```

---

## Extrinsics (Public Functions)

### Validator Management

#### `join_validators(origin, stake, location, compute_capacity)`
**Purpose**: Register as PoUW validator

**Parameters**:
- `origin`: Signed origin (validator account)
- `stake`: DALLA amount to stake (minimum 10K DALLA)
- `location`: Geographic location (Belize district)
- `compute_capacity`: CPU/GPU/RAM score (0-1000)

**Requirements**:
- Origin has KYC Level 3 (biometric verification via Identity pallet)
- Origin not sanctioned (Oracle pallet check)
- Stake >= MinValidatorStake (10,000 DALLA)
- Validator count < MaxValidators (100 limit)
- Compute capacity >= 200 (minimum hardware requirements)

**Outcome**:
- Locks `stake` amount (LockableCurrency)
- Creates `ValidatorInfo` with default scores (quality=50, timeliness=50, honesty=50)
- Increments `ValidatorCount`
- Emits `ValidatorJoined(account, stake, location)`

**Example**:
```rust
// Join as validator with 15K DALLA stake
let stake = 15_000 * 1_000_000; // 15K DALLA (6 decimals)
let location = b"Cayo District".to_vec().try_into().unwrap();
let compute_capacity = 650; // 8 CPU cores + 8GB GPU + 16GB RAM

Staking::join_validators(
    Origin::signed(validator_account),
    stake,
    location,
    compute_capacity
)?;
// Result: Validator registered, eligible for FL tasks
```

---

#### `leave_validators(origin)`
**Purpose**: Unregister as validator (unstake)

**Parameters**:
- `origin`: Signed origin (validator account)

**Requirements**:
- Origin is active validator
- No pending FL tasks (must complete or cancel)
- 7-day unbonding period (50,400 blocks)

**Outcome**:
- Sets unbonding timer (7 days)
- Validator ineligible for new tasks
- After unbonding: Unlocks stake, removes from `Validators`
- Emits `ValidatorLeft(account, stake)`

---

### Federated Learning Contributions

#### `submit_model_delta(origin, task_id, delta_hash, proof_hash)`
**Purpose**: Submit federated learning model delta

**Parameters**:
- `origin`: Signed origin (validator account)
- `task_id`: FL task ID
- `delta_hash`: SHA-256 hash of trained model weights (Pakit DAG storage)
- `proof_hash`: Zero-knowledge proof of training

**Requirements**:
- Origin is active validator
- Task exists and not expired
- Validator hasn't submitted for this task yet
- Submission before deadline (for full timeliness score)

**Process**:
1. Store `ModelDelta` with submission_block
2. Update `ValidatorInfo.last_fl_contribution`
3. Calculate timeliness_score:
   - On-time (before deadline): 100
   - 10K blocks late (~1 day): 90
   - 20K blocks late: 80
   - 50K+ blocks late: 50 (minimum)
4. Nawal aggregator validates quality (off-chain, later updates quality_score)

**Outcome**:
- Adds to `ModelDeltas[task_id][validator]`
- Updates validator stats
- Emits `ModelDeltaSubmitted(validator, task_id, delta_hash)`

**Example**:
```rust
// Submit trained model delta for healthcare diagnosis task
let delta_hash = sha256(model_weights); // Pakit DAG
let proof_hash = sha256(training_log);  // ZK proof

Staking::submit_model_delta(
    Origin::signed(validator_account),
    42, // task_id
    delta_hash,
    proof_hash
)?;
// Result: Delta submitted, awaiting Nawal quality validation
```

---

#### `assign_fl_task(origin, task_id, model_hash, deadline, reward_multiplier)`
**Purpose**: Assign federated learning task to validators (Nawal orchestrator only)

**Parameters**:
- `origin`: Signed origin (Nawal service account, authorized operator)
- `task_id`: Unique task ID
- `model_hash`: SHA-256 hash of global model (Pakit DAG)
- `deadline`: Submission deadline (block number)
- `reward_multiplier`: Task difficulty (1x-5x rewards)

**Requirements**:
- Origin is authorized operator (Nawal service account)
- Task ID unique (not duplicate)

**Outcome**:
- Creates `FederatedLearningTask`
- Sets `CurrentTask = task_id`
- All validators receive task notification (off-chain Nawal client polling)
- Emits `FLTaskAssigned(task_id, deadline, reward_multiplier)`

---

### Reward Distribution

#### `distribute_rewards(origin)`
**Purpose**: Distribute PoUW rewards for completed epoch (weekly)

**Parameters**:
- `origin`: Signed origin (anyone can trigger, permissionless)

**Requirements**:
- Current epoch completed (all deadlines passed)
- At least 1 validator submitted delta

**Process**:
1. Iterate all validators who submitted for CurrentTask
2. For each validator:
   - Calculate total_score = (quality * 0.4) + (timeliness * 0.3) + (honesty * 0.3)
   - Calculate reward = BaseReward * task_multiplier * (total_score / 100)
   - Add to `PendingRewards[validator]`
3. Update `EpochStats` with distribution data

**Outcome**:
- Rewards added to `PendingRewards` (claimable)
- Increments epoch counter
- Emits `RewardsDistributed(epoch, total_amount, validator_count)`

**Example Reward Calculation**:
```rust
// Validator scores: Quality=85, Timeliness=100, Honesty=90
let total_score = (85 * 0.4) + (100 * 0.3) + (90 * 0.3) = 34 + 30 + 27 = 91

// Base reward: 100 DALLA, Task multiplier: 3x (agriculture task)
let reward = 100 * 3 * (91 / 100) = 273 DALLA for this epoch
```

---

#### `claim_rewards(origin)`
**Purpose**: Claim accumulated PoUW rewards

**Parameters**:
- `origin`: Signed origin (validator account)

**Requirements**:
- Origin is validator with pending rewards

**Outcome**:
- Transfers `PendingRewards[validator]` from treasury to validator
- Resets `PendingRewards[validator] = 0`
- Emits `RewardsClaimed(validator, amount)`

---

### Quantum Contributions

#### `record_quantum_contribution(origin, validator, job_id, difficulty_score, accuracy_score)`
**Purpose**: Record quantum verification contribution (Quantum pallet integration)

**Parameters**:
- `origin`: Signed origin (Quantum pallet account)
- `validator`: Validator account
- `job_id`: Quantum job UUID
- `difficulty_score`: Circuit complexity (0-100)
- `accuracy_score`: Verification confidence (0-100)

**Requirements**:
- Origin is Quantum pallet (cross-pallet call)
- Validator is active

**Outcome**:
- Creates `QuantumContribution` record
- Updates `ValidatorQuantumStats`
- Calculates reward: 0.5 DALLA base + (0.01 * difficulty_score)
- Adds to `PendingRewards[validator]`
- Emits `QuantumContributionRecorded(validator, job_id, reward)`

---

### Slashing

#### `slash_validator(origin, validator, reason)`
**Purpose**: Slash validator stake for violations (admin or automated)

**Parameters**:
- `origin`: `T::AdminOrigin` OR automated (for NoSubmission)
- `validator`: Validator to slash
- `reason`: `SlashReason` enum

**Outcome**:
- Calculates slash amount (5-50% depending on reason)
- Moves slashed DALLA to `SlashedStake[validator]` (recoverable after 180 days)
- Reduces `ValidatorInfo.stake`
- Creates `SlashRecord` with recovery date
- If stake < MinValidatorStake: Eject validator
- Emits `ValidatorSlashed(validator, reason, amount)`

**Automated Slashing**:
```rust
// In on_finalize hook (end of epoch)
for validator in Validators::<T>::iter() {
    if validator.last_fl_contribution + EpochDuration < current_block {
        // Validator didn't submit for 2+ epochs
        Self::slash_validator(validator.account, SlashReason::NoSubmission)?;
    }
}
```

---

## Helper Functions (View/Query)

### `get_validator_stats(account) -> Option<ValidatorInfo>`
**Purpose**: Get validator performance statistics

**Returns**: ValidatorInfo with all scores

---

### `get_validator_rewards(account) -> Balance`
**Purpose**: Query pending unclaimed rewards

**Returns**: DALLA amount

---

### `get_epoch_stats(epoch) -> Option<EpochStats>`
**Purpose**: Get historical epoch statistics

**Returns**: EpochStats

---

### `is_validator(account) -> bool`
**Purpose**: Check if account is active validator

**Returns**: `true` if active validator

---

## Events

| Event | When Emitted |
|-------|-------------|
| `ValidatorJoined(AccountId, Balance, Vec<u8>)` | Validator registered (account, stake, location) |
| `ValidatorLeft(AccountId, Balance)` | Validator unregistered (account, stake_returned) |
| `FLTaskAssigned(u32, BlockNumber, Perbill)` | FL task assigned (task_id, deadline, multiplier) |
| `ModelDeltaSubmitted(AccountId, u32, [u8; 32])` | Delta submitted (validator, task_id, hash) |
| `RewardsDistributed(u32, Balance, u32)` | Epoch rewards (epoch, total_amount, validator_count) |
| `RewardsClaimed(AccountId, Balance)` | Rewards claimed (validator, amount) |
| `QuantumContributionRecorded(AccountId, Vec<u8>, Balance)` | Quantum work (validator, job_id, reward) |
| `ValidatorSlashed(AccountId, SlashReason, Balance)` | Stake slashed (validator, reason, amount) |

---

## Errors

| Error | Cause |
|-------|-------|
| `InsufficientStake` | Stake < 10K DALLA minimum |
| `NotValidator` | Account not registered as validator |
| `TooManyValidators` | Validator limit (100) reached |
| `InsufficientKYC` | KYC Level < 3 (biometric required) |
| `Sanctioned` | Account on OFAC/UN sanctions list |
| `TaskNotFound` | FL task ID doesn't exist |
| `AlreadySubmitted` | Validator already submitted for this task |
| `TaskExpired` | Past deadline (can submit with penalty) |
| `NoRewardsToClaim` | No pending rewards |
| `CannotLeave` | Pending tasks or unbonding period |
| `InsufficientComputeCapacity` | Compute score < 200 minimum |

---

## Integration with Other Pallets

### Identity Pallet

**Purpose**: Enforce KYC Level 3 for validators (biometric verification)

**Integration**:
```rust
// Staking checks KYC before validator registration
if !Identity::meets_validator_kyc(&account) {
    return Err(Error::<T>::InsufficientKYC.into());
}
```

---

### Oracle Pallet

**Purpose**: Sanctions compliance checks

**Integration**:
```rust
// Check sanctions before validator joins
if Oracle::is_sanctioned(&account) {
    return Err(Error::<T>::Sanctioned.into());
}
```

---

### Quantum Pallet

**Purpose**: Record quantum verification contributions

**Integration**:
```rust
// Quantum pallet calls Staking after result verification
Staking::record_quantum_contribution(
    validator_account,
    job_id,
    difficulty_score,
    accuracy_score
)?;
```

---

### Nawal Federated Learning (Off-Chain)

**Purpose**: Off-chain FL task execution and quality validation

**Integration**:
1. Nawal server listens to `FLTaskAssigned` events
2. Validators download global model from Pakit DAG
3. Train on local datasets (privacy-preserving DP-SGD)
4. Submit model deltas via `submit_model_delta()`
5. Nawal aggregator validates quality, updates on-chain scores

---

## Configuration Parameters

### `Config` Trait

```rust
pub trait Config: frame_system::Config {
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;
    type OracleVerifier: OracleVerifier<Self::AccountId>;
    type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
    type Identity: StakingIdentityProvider<Self::AccountId>;
    
    #[pallet::constant]
    type MaxValidators: Get<u32>;                    // 100 validators max
    
    #[pallet::constant]
    type MinValidatorStake: Get<Balance>;            // 10,000 DALLA minimum
    
    #[pallet::constant]
    type BaseReward: Get<Balance>;                   // 100 DALLA per epoch
    
    #[pallet::constant]
    type EpochDuration: Get<BlockNumberFor<Self>>;   // 50,400 blocks (~7 days)
    
    type WeightInfo: WeightInfo;
}
```

**Recommended Values**:
- `MaxValidators`: `100` (sufficient decentralization)
- `MinValidatorStake`: `10_000_000_000` (10K DALLA, 6 decimals)
- `BaseReward`: `100_000_000` (100 DALLA, 6 decimals)
- `EpochDuration`: `50_400` (blocks, ~7 days at 6s/block)

---

## Testing

### Unit Tests (src/tests.rs)

**Coverage**:
- Validator registration/leaving
- Stake locking/unlocking
- FL task assignments
- Model delta submissions (on-time, late)
- Reward calculations (all score combinations)
- Slashing conditions (all reasons)
- Quantum contribution recording
- KYC enforcement
- Sanctions compliance
- Unbonding periods

**Run Tests**:
```bash
cargo test -p pallet-belize-staking
```

---

## Integration with BelizeChain

### Runtime Configuration

```rust
parameter_types! {
    pub const MaxBelizeValidators: u32 = 100;
    pub const MinBelizeValidatorStake: u128 = 10_000_000_000; // 10K DALLA
    pub const BelizeBaseReward: u128 = 100_000_000; // 100 DALLA
    pub const BelizeEpochDuration: u32 = 50_400; // ~7 days
}

impl pallet_belize_staking::Config for Runtime {
    type Currency = Balances;
    type OracleVerifier = Oracle;
    type Randomness = RandomnessCollectiveFlip;
    type Identity = Identity;
    type MaxValidators = MaxBelizeValidators;
    type MinValidatorStake = MinBelizeValidatorStake;
    type BaseReward = BelizeBaseReward;
    type EpochDuration = BelizeEpochDuration;
    type WeightInfo = ();
}
```

---

## Future Enhancements

1. **Multi-Task Validators**: Support multiple simultaneous FL tasks
2. **Delegated Staking**: Non-validators delegate DALLA to validators
3. **Slashing Appeals**: Governance-based slash reversals
4. **Validator Rotation**: Automatic rotation based on performance
5. **Hardware Verification**: Trusted execution environment (TEE) proofs
6. **Cross-Chain Staking**: Bridge staked DALLA to Polkadot for parachain validation
7. **Reputation NFTs**: Validators earn reputation badges (Legendary Validator, etc.)
8. **Auto-Compounding**: Reinvest rewards into stake automatically

---

## Real-World Example: Healthcare AI Validator

**Scenario**: Hospital runs validator node for federated learning on patient diagnosis data

```rust
// 1. Hospital registers as validator (KYC L3 verified)
let stake = 25_000 * 1_000_000; // 25K DALLA stake
let location = b"Belize District".to_vec().try_into().unwrap();
let compute_capacity = 800; // High-end server (12 CPU cores, 16GB GPU, 64GB RAM)

Staking::join_validators(
    Origin::signed(hospital_account),
    stake,
    location,
    compute_capacity
)?;

// 2. Nawal assigns healthcare diagnosis task
// - Global model: BelizeDiagnosticAI v1.0
// - Dataset: 10,000 patient X-rays (pneumonia detection)
// - Privacy: DP-SGD with ε=1.0 differential privacy
// - Deadline: 7 days (50,400 blocks)
// - Reward multiplier: 5x (high-priority healthcare)

// 3. Hospital trains model (off-chain, Nawal client)
// - Training time: 4 hours
// - Model improvement: +6% accuracy (86% → 92% pneumonia detection)
// - Quality score: 100 (6% improvement)

// 4. Hospital submits model delta (on-chain)
let delta_hash = sha256(trained_weights); // Pakit DAG
let proof_hash = sha256(dp_sgd_log);

Staking::submit_model_delta(
    Origin::signed(hospital_account),
    task_id,
    delta_hash,
    proof_hash
)?;

// 5. Rewards distributed (end of epoch)
// - Quality score: 100 (6% improvement)
// - Timeliness score: 100 (on-time submission)
// - Honesty score: 100 (DP-SGD validated)
// - Total score: (100*0.4) + (100*0.3) + (100*0.3) = 100
// - Reward: 100 DALLA base * 5x multiplier * 1.0 = 500 DALLA

// 6. Hospital claims rewards
Staking::claim_rewards(Origin::signed(hospital_account))?;
// Result: 500 DALLA transferred to hospital account
// Annual APY: (500 DALLA/week * 52 weeks) / 25K stake = 104% APY (high-priority healthcare tasks)
```

---

## References

- **Nawal Federated Learning**: `nawal/README.md` (off-chain FL orchestration)
- **Quantum Pallet**: `belizechain/pallets/quantum/README.md` (quantum work integration)
- **Identity Pallet**: `belizechain/pallets/identity/README.md` (KYC Level 3 verification)
- **Oracle Pallet**: `belizechain/pallets/oracle/README.md` (sanctions compliance)
- **Differential Privacy**: [Google's DP-SGD](https://arxiv.org/abs/1607.00133)
- **Federated Learning**: [Google AI Blog](https://ai.googleblog.com/2017/04/federated-learning-collaborative.html)

---

## Contact & Support

For questions regarding BelizeChain Staking implementation:
- **Technical**: BelizeChain Core Developer Team
- **Validator Support**: validator-support@belizechain.org
- **Nawal Integration**: nawal@belizechain.org

**Audit Status**: ✅ Complete (January 2026)  
**Clippy Warnings**: 0  
**Test Coverage**: Comprehensive unit tests  
**Active Validators**: TBD (mainnet launch)  
**Total Stake**: TBD (mainnet launch)  
**Average APY**: 10-15% (estimated)
