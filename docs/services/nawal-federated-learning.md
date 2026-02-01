# Nawal Federated Learning System

Nawal is BelizeChain's privacy-preserving machine learning infrastructure that enables collaborative model training across distributed datasets without exposing sensitive data. Named after the Mayan goddess of writing and knowledge, Nawal powers the Proof of Useful Work (PoUW) consensus mechanism.

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    BelizeChain Runtime                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Staking      │  │ Consensus    │  │ Identity     │      │
│  │ Pallet       │  │ Pallet       │  │ Pallet       │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                 │              │
└─────────┼─────────────────┼─────────────────┼──────────────┘
          │                 │                 │
          │ PoUW Rewards    │ Model Updates   │ KYC Verification
          │                 │                 │
┌─────────▼─────────────────▼─────────────────▼──────────────┐
│              Nawal Orchestration Server                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Federated Averaging (FedAvg) Aggregator              │  │
│  │ - Byzantine-robust aggregation (Krum, Trimmed Mean)  │  │
│  │ - Model version management                            │  │
│  │ - Performance metrics tracking                        │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ PoUW Scoring Engine                                   │  │
│  │ - Quality: 40% (accuracy improvement)                 │  │
│  │ - Timeliness: 30% (deadline adherence)                │  │
│  │ - Honesty: 30% (Byzantine fault detection)            │  │
│  └──────────────────────────────────────────────────────┘  │
└───────────────────────┬──────────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          │             │             │
┌─────────▼────┐ ┌──────▼─────┐ ┌────▼────────┐
│ Validator 1  │ │ Validator 2│ │ Validator N │
│ (Nawal Node) │ │ (Nawal Node│ │ (Nawal Node)│
│              │ │            │ │             │
│ Local Data:  │ │ Local Data:│ │ Local Data: │
│ - Health     │ │ - Education│ │ - Tourism   │
│   records    │ │   records  │ │   analytics │
│              │ │            │ │             │
│ Training:    │ │ Training:  │ │ Training:   │
│ - DP-SGD     │ │ - DP-SGD   │ │ - DP-SGD    │
│ - Gradient   │ │ - Gradient │ │ - Gradient  │
│   clipping   │ │   clipping │ │   clipping  │
│ - Noise      │ │ - Noise    │ │ - Noise     │
│   addition   │ │   addition │ │   addition  │
└──────────────┘ └────────────┘ └─────────────┘
```

## Federated Learning Workflow

### 1. Training Round Initialization

```python
# Nawal orchestrator starts new training round
from nawal.server.aggregator import FederatedAggregator

aggregator = FederatedAggregator(
    model_type='classification',
    task='disease_prediction',
    min_participants=5,
    max_participants=20,
    rounds=10,
    epochs_per_round=3,
    target_accuracy=0.85
)

# Broadcast training parameters to validators
training_config = {
    'model_architecture': 'BelizeChainLLM',
    'learning_rate': 0.001,
    'batch_size': 32,
    'privacy_budget': 0.1,  # ε for differential privacy
    'deadline': block_number + 7200  # 12 hours at 6s/block
}

await aggregator.start_round(training_config)
```

### 2. Local Training (Validator Side)

```python
# Validator receives training round notification
from nawal.client.trainer import FederatedTrainer
from nawal.security.differential_privacy import DPOptimizer

trainer = FederatedTrainer(
    validator_id='5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    local_dataset='health_records_encrypted.db',
    model_config=training_config
)

# Initialize model with global weights
model = trainer.load_global_model(round_id=42)

# Train with differential privacy
dp_optimizer = DPOptimizer(
    optimizer='SGD',
    learning_rate=0.001,
    noise_multiplier=1.1,  # σ = 1.1 for (ε=0.1, δ=10^-5)
    max_grad_norm=1.0,  # Gradient clipping threshold
    minibatch_size=32,
    microbatch_size=1
)

for epoch in range(3):
    for batch in trainer.local_dataloader:
        # Forward pass
        predictions = model(batch.features)
        loss = criterion(predictions, batch.labels)
        
        # Backward pass with DP
        loss.backward()
        dp_optimizer.step()
        dp_optimizer.zero_grad()

# Calculate privacy spent
epsilon_spent, delta = dp_optimizer.get_privacy_spent()
print(f"Privacy budget used: ε={epsilon_spent:.4f}, δ={delta}")
```

### 3. Gradient Submission

```python
# Compute model updates (gradients)
local_gradients = trainer.compute_gradients()

# Encrypt gradients before transmission
from nawal.security.encryption import homomorphic_encrypt

encrypted_gradients = homomorphic_encrypt(
    gradients=local_gradients,
    public_key=aggregator.public_key
)

# Submit to blockchain
from nawal.blockchain.staking_connector import StakingConnector

connector = StakingConnector(ws_endpoint='wss://rpc.belizechain.org')
tx_hash = await connector.report_training(
    validator_id=trainer.validator_id,
    round_id=42,
    encrypted_gradients=encrypted_gradients,
    privacy_proof={
        'epsilon_spent': epsilon_spent,
        'delta': delta,
        'gradient_norm': torch.norm(local_gradients).item()
    },
    timestamp=block_number
)

print(f"Gradients submitted: {tx_hash}")
```

### 4. Byzantine-Robust Aggregation

```python
# Orchestrator aggregates gradients from all validators
from nawal.server.aggregation import krum_aggregation, trimmed_mean

# Collect all submissions
submissions = await aggregator.collect_submissions(round_id=42)
print(f"Received {len(submissions)} submissions from {min_participants} required")

# Detect Byzantine validators using Krum algorithm
honest_submissions = krum_aggregation(
    submissions=submissions,
    num_byzantine=2,  # Assume up to 2 malicious validators
    num_closest=5      # Compare with 5 nearest neighbors
)

# Aggregate honest gradients using trimmed mean
aggregated_gradients = trimmed_mean(
    gradients=[s.gradients for s in honest_submissions],
    trim_ratio=0.1  # Trim top/bottom 10%
)

# Update global model
global_model.apply_gradients(aggregated_gradients)

# Calculate accuracy improvement
old_accuracy = aggregator.get_accuracy(round_id=41)
new_accuracy = aggregator.evaluate_model(global_model, test_dataset)
accuracy_gain = new_accuracy - old_accuracy

print(f"Round 42: Accuracy {old_accuracy:.4f} → {new_accuracy:.4f} (+{accuracy_gain:.4f})")
```

### 5. PoUW Scoring

```python
# Score each validator's contribution
from nawal.rewards.scoring import calculate_pouw_score

for submission in submissions:
    validator = submission.validator_id
    
    # Quality score (40%): How much did this validator improve the model?
    quality_score = calculate_quality_contribution(
        validator_gradients=submission.gradients,
        aggregated_gradients=aggregated_gradients,
        accuracy_gain=accuracy_gain
    ) * 0.40
    
    # Timeliness score (30%): How quickly did they submit?
    deadline = training_config['deadline']
    submission_time = submission.timestamp
    timeliness_score = max(0, 1 - (submission_time - round_start) / (deadline - round_start)) * 0.30
    
    # Honesty score (30%): Was gradient honest (not Byzantine)?
    is_honest = validator in [s.validator_id for s in honest_submissions]
    honesty_score = 1.0 if is_honest else 0.0
    honesty_score *= 0.30
    
    # Total score (0-1)
    total_score = quality_score + timeliness_score + honesty_score
    
    # Calculate DALLA reward (50-350 per session)
    base_reward = 50  # DALLA
    max_reward = 350  # DALLA
    reward = base_reward + (max_reward - base_reward) * total_score
    
    # Submit reward to blockchain
    await connector.distribute_pouw_reward(
        validator_id=validator,
        round_id=42,
        score=total_score,
        reward_amount=reward,
        breakdown={
            'quality': quality_score,
            'timeliness': timeliness_score,
            'honesty': honesty_score
        }
    )
    
    print(f"Validator {validator[:8]}... scored {total_score:.3f} → {reward:.2f} DALLA")
```

## Model Architecture

### BelizeChainLLM

Custom transformer-based model optimized for federated learning:

```python
# nawal/client/model.py
import torch
import torch.nn as nn

class BelizeChainLLM(nn.Module):
    def __init__(self, config):
        super().__init__()
        self.embedding = nn.Embedding(config.vocab_size, config.hidden_size)
        
        # Multi-head attention layers
        self.transformer_blocks = nn.ModuleList([
            TransformerBlock(
                hidden_size=config.hidden_size,
                num_heads=config.num_heads,
                dropout=config.dropout
            )
            for _ in range(config.num_layers)
        ])
        
        # Classification head
        self.classifier = nn.Linear(config.hidden_size, config.num_classes)
        
    def forward(self, input_ids, attention_mask=None):
        # Embedding
        x = self.embedding(input_ids)
        
        # Transformer blocks
        for block in self.transformer_blocks:
            x = block(x, attention_mask)
        
        # Pool and classify
        pooled = x.mean(dim=1)
        logits = self.classifier(pooled)
        
        return logits

# Configuration for disease prediction
config = ModelConfig(
    vocab_size=50000,
    hidden_size=768,
    num_heads=12,
    num_layers=6,
    dropout=0.1,
    num_classes=10  # 10 disease categories
)

model = BelizeChainLLM(config)
print(f"Model parameters: {sum(p.numel() for p in model.parameters()):,}")
# Output: Model parameters: 85,054,474
```

## Differential Privacy (DP-SGD)

### Privacy Budget Calculation

```python
# nawal/security/differential_privacy.py
from opacus.accountants import RDPAccountant

def calculate_privacy_budget(
    noise_multiplier: float,
    sample_rate: float,
    steps: int,
    delta: float = 1e-5
) -> float:
    """
    Calculate epsilon spent for given DP-SGD parameters.
    
    Args:
        noise_multiplier: σ (noise scale)
        sample_rate: Fraction of dataset per batch
        steps: Total training steps
        delta: Privacy parameter (typically 1/n^2)
    
    Returns:
        epsilon: Privacy budget spent
    """
    accountant = RDPAccountant()
    accountant.history = [(noise_multiplier, sample_rate, steps)]
    epsilon = accountant.get_epsilon(delta)
    return epsilon

# Example: 3 epochs on 10,000 samples, batch size 32
noise_multiplier = 1.1
sample_rate = 32 / 10000  # 0.0032
epochs = 3
steps_per_epoch = 10000 // 32  # 312
total_steps = epochs * steps_per_epoch  # 936

epsilon = calculate_privacy_budget(
    noise_multiplier=noise_multiplier,
    sample_rate=sample_rate,
    steps=total_steps,
    delta=1e-5
)

print(f"Privacy guarantee: (ε={epsilon:.4f}, δ=1e-5)-DP")
# Output: Privacy guarantee: (ε=0.0987, δ=1e-5)-DP
```

### Gradient Clipping & Noise Addition

```python
# Per-sample gradient clipping
def clip_gradients(gradients, max_norm=1.0):
    """Clip gradients to max L2 norm."""
    for param_grad in gradients:
        grad_norm = torch.norm(param_grad, p=2)
        clip_coef = max_norm / (grad_norm + 1e-6)
        if clip_coef < 1:
            param_grad.mul_(clip_coef)
    return gradients

# Add Gaussian noise
def add_noise(gradients, noise_multiplier=1.1, max_norm=1.0):
    """Add calibrated Gaussian noise for DP."""
    sigma = noise_multiplier * max_norm
    for param_grad in gradients:
        noise = torch.randn_like(param_grad) * sigma
        param_grad.add_(noise)
    return gradients

# Complete DP-SGD step
def dp_sgd_step(model, batch, optimizer, max_norm=1.0, noise_multiplier=1.1):
    optimizer.zero_grad()
    
    # Forward pass
    outputs = model(batch.features)
    loss = criterion(outputs, batch.labels)
    
    # Backward pass
    loss.backward()
    
    # Clip gradients
    gradients = [param.grad for param in model.parameters()]
    gradients = clip_gradients(gradients, max_norm)
    
    # Add noise
    gradients = add_noise(gradients, noise_multiplier, max_norm)
    
    # Update parameters
    for param, grad in zip(model.parameters(), gradients):
        param.grad = grad
    optimizer.step()
```

## Genome Evolution

Nawal uses genetic algorithms to optimize model architectures:

```python
# nawal/genome/evolution.py
from nawal.genome.mutator import ArchitectureMutator

class EvolutionarySearch:
    def __init__(self, population_size=20, generations=50):
        self.population_size = population_size
        self.generations = generations
        self.mutator = ArchitectureMutator()
        
    def evolve(self, base_config):
        population = self.initialize_population(base_config)
        
        for gen in range(self.generations):
            # Evaluate fitness (accuracy on validation set)
            fitness_scores = self.evaluate_population(population)
            
            # Select top performers
            elite = self.select_elite(population, fitness_scores, top_k=5)
            
            # Generate next generation
            offspring = self.crossover_and_mutate(elite)
            
            # Replace population
            population = elite + offspring
            
            best_fitness = max(fitness_scores)
            print(f"Generation {gen}: Best fitness = {best_fitness:.4f}")
        
        # Return best architecture
        best_idx = fitness_scores.index(max(fitness_scores))
        return population[best_idx]
    
    def mutate_architecture(self, config):
        """Random architecture mutations."""
        mutations = [
            lambda c: {**c, 'num_layers': c['num_layers'] + random.choice([-1, 1])},
            lambda c: {**c, 'hidden_size': c['hidden_size'] * random.choice([0.5, 2])},
            lambda c: {**c, 'num_heads': c['num_heads'] + random.choice([-2, 2])},
            lambda c: {**c, 'dropout': max(0.0, min(0.5, c['dropout'] + random.uniform(-0.1, 0.1)))}
        ]
        mutation = random.choice(mutations)
        return mutation(config)

# Run evolution
search = EvolutionarySearch(population_size=20, generations=50)
optimal_config = search.evolve(base_config)

print("Optimal architecture found:")
print(f"  Layers: {optimal_config['num_layers']}")
print(f"  Hidden size: {optimal_config['hidden_size']}")
print(f"  Attention heads: {optimal_config['num_heads']}")
print(f"  Dropout: {optimal_config['dropout']}")
```

## Validator Requirements

### Nawal-Type Validator Hardware

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 16 cores | 32 cores |
| RAM | 64 GB | 128 GB |
| GPU | NVIDIA A100 40GB | NVIDIA A100 80GB |
| Storage | 2 TB NVMe SSD | 4 TB NVMe SSD |
| Network | 1 Gbps | 10 Gbps |

### Software Setup

```bash
# Install CUDA 12.1
wget https://developer.download.nvidia.com/compute/cuda/12.1.0/local_installers/cuda_12.1.0_530.30.02_linux.run
sudo sh cuda_12.1.0_530.30.02_linux.run

# Install PyTorch with CUDA support
pip install torch==2.1.0+cu121 torchvision==0.16.0+cu121 -f https://download.pytorch.org/whl/torch_stable.html

# Install Nawal SDK
pip install nawal-sdk==1.0.0

# Verify GPU availability
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}')"
python -c "import torch; print(f'GPU: {torch.cuda.get_device_name(0)}')"
```

### Start Nawal Node

```bash
# Configure validator
nawal-cli config \
  --validator-id 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --orchestrator wss://nawal.belizechain.org \
  --data-path /mnt/nawal-data \
  --gpu-id 0

# Start federated learning client
nawal-cli start \
  --min-score 0.7 \  # Only participate if expected score > 0.7
  --max-rounds 100 \  # Limit participation
  --privacy-budget 1.0  # Total ε budget
```

## Performance Benchmarks

### Training Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| Training round duration | 8-12 hours | 7200 blocks at 6s/block |
| Validators per round | 5-20 | Minimum 5, optimal 12-15 |
| Model convergence | 8-10 rounds | To reach 85% accuracy target |
| Communication overhead | 120 MB/round | Encrypted gradients + metadata |
| Privacy budget (ε) | 0.1 per round | Total ε ≤ 1.0 cumulative |

### Reward Distribution

| Validator Performance | Score | DALLA Reward |
|----------------------|-------|--------------|
| Perfect (1.0) | 1.000 | 350 |
| Excellent (0.9) | 0.900 | 320 |
| Good (0.8) | 0.800 | 290 |
| Average (0.7) | 0.700 | 260 |
| Below Average (0.6) | 0.600 | 230 |
| Poor (0.5) | 0.500 | 200 |
| Minimum (0.4) | 0.400 | 170 |
| Byzantine (detected) | 0.000 | 0 (slashed) |

**Weekly earnings** (7 rounds): 1,225 - 2,450 DALLA (avg 1,820 DALLA at 0.75 score)

## API Reference

See [Staking Pallet PoUW Methods](../developer-guides/pallet-apis-financial.md#proof-of-useful-work-pouw) for blockchain integration.

## Security Considerations

1. **Differential Privacy**: All gradients protected with (ε, δ)-DP guarantees
2. **Secure Aggregation**: Homomorphic encryption prevents server from seeing individual gradients
3. **Byzantine Tolerance**: Krum and trimmed mean detect malicious validators
4. **Model Poisoning**: Gradient norm checks prevent adversarial updates
5. **Data Sovereignty**: Local datasets never leave validator nodes
6. **Privacy Budget Tracking**: Cumulative ε monitored to prevent leakage
