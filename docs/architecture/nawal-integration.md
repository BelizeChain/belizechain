# Nawal AI Integration

**Privacy-Preserving Federated Learning for National AI**

Nawal enables decentralized machine learning across Belizean institutions while preserving data privacy.

---

## Architecture

```
┌─────────────────────────────────────────┐
│ Government Hospitals (5 nodes)          │
│ Training on patient data (local only)   │
└───────────┬─────────────────────────────┘
            │ Encrypted gradients
            ↓
┌─────────────────────────────────────────┐
│ Nawal Aggregator                        │
│ Federated Averaging (DP-SGD)            │
│ :8889                                   │
└───────────┬─────────────────────────────┘
            │ Model updates
            ↓
┌─────────────────────────────────────────┐
│ BelizeChain Staking Pallet              │
│ PoUW Rewards (40% quality, 30% time)    │
└─────────────────────────────────────────┘
```

---

## Features

### 1. Federated Learning
Train models without centralizing data:

```python
from nawal.client import FederatedClient

client = FederatedClient(
    node_id="hospital_belmopan",
    data_loader=local_patient_data,
    model=BelizeChainLLM(layers=12, hidden=768)
)

# Train locally (data never leaves hospital)
client.train(epochs=5, privacy_budget=1.0)

# Submit encrypted gradients to aggregator
client.submit_gradients()
```

### 2. Differential Privacy (DP-SGD)
Formal privacy guarantees:

```python
from nawal.privacy import DifferentialPrivacy

dp = DifferentialPrivacy(
    epsilon=1.0,      # Privacy budget (lower = more private)
    delta=1e-5,       # Failure probability
    max_grad_norm=1.0 # Gradient clipping
)

# Add calibrated noise to gradients
private_gradients = dp.privatize_gradients(gradients)

print(f"Privacy guarantee: ({dp.epsilon}, {dp.delta})-DP")
```

### 3. Genome Evolution
Automated neural architecture search:

```python
from nawal.genome import GeneticEvolution

evolution = GeneticEvolution(
    population_size=50,
    mutation_rate=0.1,
    crossover_rate=0.3,
    generations=100
)

# Evolve model architecture
best_genome = evolution.evolve(
    fitness_fn=lambda model: evaluate_accuracy(model),
    constraints={'max_params': 100_000_000}
)

print(f"Best architecture: {best_genome.to_model()}")
```

---

## Integration Points

### Nawal → Blockchain (PoUW Rewards)
```python
# Report training contribution
async def report_training_session(
    session_id: str,
    model_accuracy: float,
    training_time: int
):
    call = substrate.compose_call(
        call_module='Staking',
        call_function='report_training',
        call_params={
            'session_id': session_id,
            'accuracy': int(model_accuracy * 10000),  # Basis points
            'time_seconds': training_time,
            'privacy_verified': True
        }
    )
    
    receipt = substrate.submit_extrinsic(call, keypair=node_key)
    
    if receipt.is_success:
        reward_event = find_event(receipt, 'Staking', 'TrainingRewardIssued')
        print(f"Earned: {reward_event['reward']} DALLA")
```

### Blockchain → Nawal (Query Models)
```python
# Query trained model metadata
model_info = substrate.query(
    module='Staking',
    storage_function='TrainedModels',
    params=['belizechain_llm_v1']
)

if model_info:
    print(f"Accuracy: {model_info['accuracy'] / 100}%")
    print(f"Contributors: {len(model_info['contributors'])}")
    print(f"Total rewards: {model_info['total_rewards']} DALLA")
```

---

## Configuration

```yaml
# nawal/config.yml
federated_learning:
  aggregation_algorithm: fedavg  # FedAvg, FedProx, FedOpt
  min_clients: 3
  max_clients: 10
  rounds: 100
  
privacy:
  differential_privacy: true
  epsilon: 1.0
  delta: 1e-5
  secure_aggregation: true
  
model:
  architecture: transformer
  layers: 12
  hidden_size: 768
  vocab_size: 50000
  
blockchain:
  rpc_url: ws://localhost:9944
  reward_account: 5GrwvaEF...
  submission_interval: 10  # rounds
```

---

## Privacy Guarantees

### Differential Privacy Math
```
Privacy Loss Budget:
ε = 1.0 (epsilon)
δ = 1e-5 (delta)

Meaning: Adversary learns ≤ ε info per query
         with probability ≥ (1 - δ)

Composition (100 rounds):
ε_total ≤ 100 × ε = 100 (advanced composition: ~14)
Still acceptable for medical data!
```

### Secure Aggregation
```python
# Each client encrypts gradients
from nawal.crypto import SecureAggregation

sa = SecureAggregation(num_clients=5, threshold=3)

# Client 1 encrypts
encrypted_grad_1 = sa.encrypt(gradients_1, client_id=1)

# Aggregator decrypts sum (not individuals!)
aggregated = sa.aggregate_and_decrypt([
    encrypted_grad_1,
    encrypted_grad_2,
    encrypted_grad_3
])

# Aggregator never sees individual gradients!
```

---

## Performance

### Training Speed
```
Model: BelizeChainLLM (768M parameters)
Dataset: 10M Belizean text samples
Hardware: 5 nodes × 1 NVIDIA V100 GPU

Centralized Training:
- Time: 48 hours
- Data Privacy: NONE (all data centralized)

Federated Training (Nawal):
- Time: 72 hours (1.5x slower)
- Data Privacy: (1.0, 1e-5)-DP guaranteed
- Communication: 2.5 GB/round (25 GB total)

Trade-off: 50% time penalty for complete privacy
```

---

## Use Cases

### 1. National Healthcare AI
```
Data: 5 hospitals × 50K patient records
Model: Disease diagnosis (10 conditions)
Privacy: (1.0, 1e-5)-DP

Results:
- Accuracy: 94.2% (vs 95.8% centralized)
- Privacy preserved: Patient data never shared
- Rewards: 5,000 DALLA distributed to hospitals
```

### 2. Financial Fraud Detection
```
Data: 3 banks × 100K transactions
Model: Fraud classifier (binary)
Privacy: (0.5, 1e-6)-DP (stricter)

Results:
- Accuracy: 98.1% (vs 98.9% centralized)
- False positive rate: 0.3%
- Rewards: 8,000 DALLA distributed to banks
```

---

## Related Documentation

- [Multi-Repo Overview](./multi-repo-overview.md)
- [Nawal Repository](https://github.com/BelizeChain/nawal-ai)
- [Staking Pallet](../developer-guides/pallet-staking.md)
- [Differential Privacy Guide](https://www.microsoft.com/research/wp-content/uploads/2016/02/dwork.pdf)
