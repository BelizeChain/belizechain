# Consensus Pallet

## Overview

The **Consensus pallet** implements BelizeChain's unique **Proof of Useful Work (PoUW)** consensus mechanism, which combines federated AI model training with blockchain consensus. Validators earn rewards by contributing to national AI initiatives (via Nawal) and quantum computing tasks (via Kinich), rather than solving arbitrary cryptographic puzzles.

## Purpose

This pallet transforms blockchain consensus from energy-intensive proof-of-work to **socially beneficial AI work**, enabling:
- **Federated AI Coordination**: Validators train privacy-preserving models on national datasets
- **Quality-Based Consensus**: Validator selection weighted by AI model quality scores
- **Economic Incentives**: Rewards tied to useful work, not arbitrary computation
- **Post-Quantum Security**: Quantum signature validation for future-proof consensus
- **National AI Development**: All consensus work contributes to Belize's AI capabilities

## Key Features

### 1. Federated Learning Integration
- Validators register AI models with the consensus system
- Each consensus round coordinates federated learning tasks
- Model quality scores (0-100) determine validator weights
- Privacy-preserving: No raw data leaves local nodes

### 2. Validator Selection
- **Economic Weight**: Staked DALLA amount (via Staking pallet)
- **Reputation Score**: Historical AI contribution quality (0-100)
- **Model Quality**: Recent federated learning performance (0-100)
- **Combined Score**: `(stake_weight × 0.3) + (reputation × 0.3) + (quality × 0.4)`

### 3. Consensus Rounds
- Fixed duration (e.g., 100 blocks per round)
- Coordinated AI training tasks
- Work submission and validation
- Reward distribution based on contribution quality

### 4. Work Types
- **FederatedLearning**: Model training on distributed datasets
- **QuantumComputation**: Quantum algorithm execution (Kinich integration)
- **ModelValidation**: Cross-validation of submitted models
- **DataQualityCheck**: Dataset integrity verification

### 5. Post-Quantum Cryptography
- Quantum-resistant signature schemes for validator attestations
- Future-proof security against quantum computing threats
- Integration with Kinich quantum backend for validation

## Architecture

### Integration with Staking Pallet
The Consensus pallet uses `ConsensusStakingProvider` trait to:
- Query validator stake amounts (economic weight)
- Retrieve reputation scores (historical quality)
- Update reputation based on AI work quality
- Lock stake during consensus participation

### Integration with Nawal AI
- Validators submit federated learning proof-of-work
- Nawal API validates model quality scores
- Consensus round results feed back to Nawal for model aggregation
- Privacy-preserving: Only gradients/weights shared, not raw data

### Integration with Kinich Quantum
- Quantum computation tasks submitted as useful work
- Kinich validates quantum job execution quality
- Post-quantum signatures validated via Kinich backend
- Quantum workload distribution across validators

### Storage Items
- **`RegisteredModels`**: Maps (AccountId, ModelId) → AIModel
- **`ConsensusValidators`**: Maps AccountId → ConsensusValidator
- **`CurrentRound`**: ConsensusRound (active round metadata)
- **`AIWorkSubmissions`**: Maps (RoundId, AccountId) → AIWorkSubmission
- **`ValidatorReputations`**: Maps AccountId → u8 (0-100 score)
- **`RoundHistory`**: Vec<ConsensusRound> (last 100 rounds)
- **`SystemMetrics`**: AISystemMetrics (global performance stats)

## Extrinsics

### `register_ai_model(model_id, model_type, metadata_hash, performance_metrics)`
Register an AI model for consensus participation.

**Parameters:**
- `model_id`: BoundedVec<u8, 64> - Unique identifier
- `model_type`: Enum (ImageClassification, TextGeneration, AnomalyDetection, ReinforcementLearning, Other)
- `metadata_hash`: H256 - IPFS/Arweave hash of model metadata
- `performance_metrics`: BoundedVec<(MetricType, Value)> - Initial benchmarks

**Caller:** Any account with minimum stake

**Events:**
- `AIModelRegistered(account, model_id, model_type)`

**Errors:**
- `InsufficientStake` - Below minimum consensus stake
- `ModelAlreadyExists` - Duplicate model_id

### `join_consensus_validator(model_ids)`
Join as a consensus validator using registered AI models.

**Parameters:**
- `model_ids`: Vec<BoundedVec<u8, 64>> - Models to use for consensus

**Requirements:**
- Minimum stake locked (configurable, e.g., 10,000 DALLA)
- At least one registered AI model
- Model quality score ≥ threshold (e.g., 60/100)
- Compliance: L3 Enhanced KYC verification (via Compliance pallet)

**Events:**
- `ValidatorJoined(account, stake_amount, model_count)`

**Errors:**
- `InsufficientStake`
- `NoRegisteredModels`
- `ModelQualityTooLow`
- `ComplianceCheckFailed`

### `validate_ai_model(account, model_id, quality_score, validation_proof)`
Validate another validator's AI model (cross-validation).

**Parameters:**
- `account`: Target validator
- `model_id`: Model being validated
- `quality_score`: u8 (0-100)
- `validation_proof`: BoundedVec<u8, 512> - Cryptographic proof

**Caller:** AI Authority or designated validators

**Events:**
- `ModelValidated(validator, model_id, quality_score)`

### `start_consensus_round(round_id, task_description, target_models)`
Initiate a new consensus round with specific AI tasks.

**Parameters:**
- `round_id`: u64
- `task_description`: BoundedVec<u8, 256>
- `target_models`: Vec<ModelId> - Models assigned to this round

**Authority:** AI Authority Origin

**Events:**
- `ConsensusRoundStarted(round_id, start_block, target_validators)`

### `submit_ai_work(round_id, work_type, quality_score, work_proof, metadata)`
Submit completed AI work for a consensus round.

**Parameters:**
- `round_id`: u64
- `work_type`: Enum (FederatedLearning, QuantumComputation, ModelValidation, DataQualityCheck)
- `quality_score`: u8 (0-100, self-reported, validated later)
- `work_proof`: BoundedVec<u8, 1024> - Cryptographic proof of work
- `metadata`: BoundedVec<u8, 512> - Training metrics, quantum results, etc.

**Caller:** Consensus validators only

**Events:**
- `AIWorkSubmitted(validator, round_id, work_type, quality_score)`

**Errors:**
- `NotConsensusValidator`
- `RoundNotActive`
- `WorkAlreadySubmitted`
- `InvalidWorkProof`

### `finalize_consensus_round(round_id)`
Complete consensus round and distribute rewards.

**Authority:** AI Authority or automated trigger (end block hook)

**Reward Distribution:**
- Base reward: Fixed amount (e.g., 100 DALLA per validator)
- Quality bonus: Scaled by `quality_score / 100`
- Reputation update: `new_reputation = (old_reputation × 0.7) + (quality_score × 0.3)`

**Events:**
- `ConsensusRoundFinalized(round_id, total_validators, total_rewards)`

## Helper Functions

### `get_validator_reputation(account) -> u8`
Retrieve validator's reputation score (0-100).

**Calculation:**
- Initial: 50 (neutral)
- Updated after each round: `(old × 0.7) + (quality_score × 0.3)`
- Decays slowly if inactive (0.95× per missed round)

### `get_model_quality_score(account) -> u8`
Get validator's most recent model quality score.

**Sources:**
- Latest federated learning round result
- Cross-validation by other validators
- Nawal API quality metrics

### `get_validator_stake(account) -> Balance`
Retrieve validator's locked stake amount (delegates to Staking pallet).

### `sync_validator_reputation(account, quality_score) -> bool`
Internal function to update reputation after work submission.

### `get_validator_contribution_score(account) -> u32`
Calculate composite score for validator selection:
- `score = (stake × 0.3) + (reputation × 0.3) + (quality × 0.4)`
- Used for weighted random selection in consensus rounds

### `account_id() -> AccountId`
Get the pallet's sovereign account (for reward distribution).

## Consensus Flow

### 1. Validator Registration
```
Validator → register_ai_model() → Model stored
         ↓
         join_consensus_validator() → Stake locked
         ↓
         Status: Active validator
```

### 2. Consensus Round Lifecycle
```
AI Authority → start_consensus_round()
            ↓
Validators assigned tasks (federated learning, quantum jobs)
            ↓
Validators train models locally (Nawal integration)
            ↓
Validators → submit_ai_work() with quality proofs
            ↓
AI Authority validates work quality
            ↓
finalize_consensus_round() → Rewards distributed
            ↓
Validator reputations updated
```

### 3. Reward Calculation Example
```rust
let base_reward = ConsensusReward::get(); // 100 DALLA
let quality_bonus = (quality_score as u128 * base_reward) / 100;
let total_reward = base_reward + quality_bonus;

// If quality_score = 85:
// total_reward = 100 + (85 * 100 / 100) = 185 DALLA
```

## Economic Model

### Staking Requirements
- **Minimum Stake**: 10,000 DALLA (configurable)
- **Lock Period**: Until validator exits consensus
- **Slash Conditions**: Invalid work proofs, malicious behavior
- **Unbonding Period**: 28 days (standard Substrate pattern)

### Reward Distribution
- **Base Reward**: 100 DALLA per round (configurable)
- **Quality Multiplier**: 0-2× based on AI work quality (0-100 score)
- **Frequency**: Per consensus round (e.g., every 100 blocks)
- **Annual Yield**: ~15-30% APY for high-quality validators

### Reputation System
- **Initial Score**: 50/100 (neutral)
- **Update Formula**: `(old × 0.7) + (quality × 0.3)` (exponential smoothing)
- **Inactivity Decay**: 0.95× per missed round
- **Minimum for Rewards**: 40/100 (below this, rewards reduced)

## Quality Validation

### Model Quality Metrics
- **Accuracy**: Federated learning model accuracy improvement
- **Convergence**: Training convergence rate
- **Privacy**: Differential privacy guarantees met
- **Efficiency**: Computational resource usage

### Cross-Validation
- Random validators validate others' work
- Byzantine fault tolerance: Majority consensus required
- Malicious validators slashed for false validation

### Nawal Integration
- Nawal API provides ground truth quality scores
- Federated aggregation validates individual contributions
- Privacy-preserving: Only aggregated metrics on-chain

## Security Features

### Sybil Resistance
- Economic stake requirement (10,000 DALLA minimum)
- Reputation system (history-weighted)
- Quality validation by peers

### Work Validation
- Cryptographic proofs of AI training
- Cross-validation by random validators
- Nawal API verification for federated learning
- Kinich validation for quantum work

### Post-Quantum Security
- Quantum-resistant signatures for attestations
- Kinich integration for quantum validation
- Future-proof against quantum attacks

## Weight Information

### Benchmarked Operations
- `register_ai_model`: 30,000,000 + 2 reads + 1 write
- `join_consensus_validator`: 50,000,000 + 4 reads + 2 writes + stake lock
- `submit_ai_work`: 40,000,000 + 3 reads + 2 writes
- `finalize_consensus_round`: 100,000,000 + (validators × 10,000) + rewards
- `get_validator_reputation`: 15,000,000 + 1 read

## Testing

### Unit Tests (`tests.rs`)
- AI model registration and validation
- Validator join/exit workflows
- Consensus round lifecycle (start, submit, finalize)
- Reward distribution accuracy
- Reputation update calculations
- Quality score validation
- Stake locking/unlocking

### Mock Setup (`mock.rs`)
- Test runtime with Staking integration
- AI Authority origin configuration
- Sample federated learning scenarios
- Quantum work submission mocks

## Dependencies

### Pallets
- `pallet-belize-staking` - Validator stakes, reputation
- `pallet-belize-compliance` - L3 KYC verification
- `frame-system` - Block numbers, timestamps
- `frame-support` - Currency, traits, storage

### External Services
- **Nawal AI** - Federated learning coordination, quality validation
- **Kinich Quantum** - Quantum job execution, post-quantum crypto

## Integration Points

### Nawal AI Service
```rust
// After consensus round finalized:
1. Consensus pallet emits `AIWorkSubmitted` event
2. Nawal listens for events via blockchain RPC
3. Nawal aggregates federated learning results
4. Nawal reports quality scores back via `validate_ai_model()`
5. Reputation updated automatically
```

### Kinich Quantum Service
```rust
// Quantum work submission:
1. Validator submits quantum job to Kinich
2. Kinich executes on Azure Quantum backend
3. Validator calls `submit_ai_work(QuantumComputation, quality_score, proof)`
4. Kinich validates proof via `validate_ai_model()`
5. Rewards distributed based on quantum job quality
```

## Governance Parameters

### Configurable Constants
- `MaxValidators`: Maximum consensus validators (e.g., 100)
- `MinConsensusStake`: Minimum stake to join (10,000 DALLA)
- `MinModelQualityScore`: Minimum quality threshold (60/100)
- `ConsensusReward`: Base reward per round (100 DALLA)

### Runtime Configuration
```rust
impl pallet_belize_consensus::Config for Runtime {
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type UnixTime = Timestamp;
    type AIAuthorityOrigin = EnsureRoot<AccountId>;
    type MaxValidators = ConstU32<100>;
    type MinConsensusStake = ConstU128<10_000_000_000_000_000>; // 10K DALLA
    type MinModelQualityScore = ConstU32<60>;
    type ConsensusReward = ConstU128<100_000_000_000_000_000>; // 100 DALLA
    type WeightInfo = ();
    type Staking = StakingProvider;
}
```

## Future Enhancements
1. **Adaptive round length** - Based on network activity
2. **Multi-model consensus** - Validators run multiple models simultaneously
3. **Cross-chain AI work** - Proof-of-work shared across parachains
4. **Automated quality validation** - ML-based work verification
5. **Dynamic reward curves** - Inflation-adjusted based on total stake

## Files

### Core Implementation
- **`src/lib.rs`** (847 lines) - Main pallet logic, extrinsics, consensus flow
- **`src/mock.rs`** (210 lines) - Test runtime with Staking integration
- **`src/tests.rs`** (610 lines) - Comprehensive consensus round tests
- **`Cargo.toml`** - Dependencies and features

### Documentation
- **`README.md`** - This file

**Total Lines:** 1,667 lines of production code + tests

---

**Maintained by:** BelizeChain Core Team  
**Last Updated:** January 29, 2026  
**Status:** Production Ready (Polkadot SDK stable2512) - Integrated with Nawal AI and Kinich Quantum
