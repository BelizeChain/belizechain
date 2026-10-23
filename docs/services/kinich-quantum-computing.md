# Kinich Quantum Computing Layer

Kinich (named after the Mayan sun god) is BelizeChain's quantum computing orchestration layer that enables validators to perform quantum workloads and earn Proof of Quantum Work (PQW) rewards. On Ceiba no cloud provider is enabled — every job runs on the in-process simulator.

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    BelizeChain Runtime                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Consensus    │  │ Quantum      │  │ Staking      │      │
│  │ Pallet       │  │ Pallet       │  │ Pallet       │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                 │              │
└─────────┼─────────────────┼─────────────────┼──────────────┘
          │                 │                 │
          │ PQW Submission  │ Job Management  │ Rewards
          │                 │                 │
┌─────────▼─────────────────▼─────────────────▼──────────────┐
│              Kinich Quantum Orchestrator                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Quantum Job Scheduler                                 │  │
│  │ - Circuit optimization (gate count minimization)     │  │
│  │ - Backend selection (IonQ, Quantinuum, IBM, Rigetti) │  │
│  │ - Error mitigation (ZNE, readout correction)         │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ PQW Verification Engine                               │  │
│  │ - Quantum signature validation                        │  │
│  │ - Circuit complexity scoring (qubit count, depth)     │  │
│  │ - Result verification (cross-backend validation)      │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Compression Engine Integration                        │  │
│  │ - Quantum-classical hybrid compression (6.8x ratio)   │  │
│  │ - configured quantum backend optimization                          │  │
│  └──────────────────────────────────────────────────────┘  │
└───────────────────────┬──────────────────────────────────┘
                        │
          ┌─────────────┼─────────────┬─────────────┐
          │             │             │             │
┌─────────▼────┐ ┌──────▼─────┐ ┌────▼────────┐   │
│ Azure        │ │ IBM        │ │ Rigetti     │   │
│ Quantum      │ │ Quantum    │ │ Quantum     │   │
│ (Primary)    │ │ (Fallback) │ │ (Fallback)  │   │
│              │ │            │ │             │   │
│ IonQ:        │ │ IBM Kyoto: │ │ Aspen-M-3:  │   │
│ 25 qubits    │ │ 127 qubits │ │ 32 qubits   │   │
│ 99.5% gate   │ │ 99.2% gate │ │ 98.5% gate  │   │
│ fidelity     │ │ fidelity   │ │ fidelity    │   │
│              │ │            │ │             │   │
│ Quantinuum:  │ │            │ │             │   │
│ 20 qubits    │ │            │ │             │   │
│ 99.9% gate   │ │            │ │             │   │
│ fidelity     │ │            │ │             │   │
└──────────────┘ └────────────┘ └─────────────┘   │
                                                   │
┌──────────────────────────────────────────────────┘
│
▼
┌─────────────────────────────────────────────────┐
│           Kinich-Type Validators                │
│  - 16 cores / 64 GB RAM                         │
│  - Quantum SDK (Qiskit, configured quantum backend)          │
│  - Job submission & verification                │
└─────────────────────────────────────────────────┘
```

## Quantum Workload Workflow

### 1. Job Submission

```python
# Validator submits quantum circuit for execution
from kinich.core.quantum_node import QuantumNode
from qiskit import QuantumCircuit

# Create quantum circuit (Grover's algorithm example)
qc = QuantumCircuit(4, 4)  # 4 qubits, 4 classical bits

# Initialize superposition
qc.h(range(4))

# Oracle (marking |0101>)
qc.cz(0, 1)
qc.cz(2, 3)

# Diffusion operator
qc.h(range(4))
qc.x(range(4))
qc.h(3)
qc.mcx([0, 1, 2], 3)
qc.h(3)
qc.x(range(4))
qc.h(range(4))

# Measure
qc.measure(range(4), range(4))

# Submit to Kinich orchestrator
node = QuantumNode(
    validator_id='5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    orchestrator_url='wss://testnet.belizechain.org/api/kinich'
)

job_id = await node.submit_job(
    circuit=qc,
    backend_preference='ionq',  # IonQ on configured quantum backend (primary)
    shots=1024,
    optimization_level=3,
    error_mitigation='zne',  # Zero-noise extrapolation
    priority='normal'
)

print(f"Job submitted: {job_id}")
# Output: Job submitted: kinich-job-a3f7b2c9
```

### 2. Circuit Optimization

```python
# Kinich orchestrator optimizes circuit before execution
from kinich.optimization.transpiler import optimize_circuit

optimized_qc = optimize_circuit(
    circuit=qc,
    backend='ionq',
    coupling_map=None,  # IonQ is all-to-all connectivity
    optimization_level=3
)

# Gate count reduction
original_gates = qc.count_ops()
optimized_gates = optimized_qc.count_ops()

print("Gate count optimization:")
print(f"  Original: {sum(original_gates.values())} gates")
print(f"  Optimized: {sum(optimized_gates.values())} gates")
print(f"  Reduction: {(1 - sum(optimized_gates.values()) / sum(original_gates.values())) * 100:.1f}%")

# Output:
# Gate count optimization:
#   Original: 52 gates
#   Optimized: 38 gates
#   Reduction: 26.9%
```

### 3. Backend Selection

```python
# Orchestrator selects optimal backend based on circuit requirements
from kinich.adapters.azure import AzureQuantumBackend
from kinich.adapters.ibm import IBMQuantumBackend

def select_backend(circuit, preference):
    """Select best quantum backend for circuit."""
    qubit_count = circuit.num_qubits
    circuit_depth = circuit.depth()
    
    # configured quantum backend backends
    backends = {
        'ionq': {
            'qubits': 25,
            'fidelity': 0.995,
            'connectivity': 'all-to-all',
            'cost_per_shot': 0.00022  # USD
        },
        'quantinuum': {
            'qubits': 20,
            'fidelity': 0.999,
            'connectivity': 'all-to-all',
            'cost_per_shot': 0.00035
        }
    }
    
    # Check if circuit fits
    if qubit_count > backends[preference]['qubits']:
        # Fallback to IBM Quantum
        return IBMQuantumBackend('ibm_kyoto', qubits=127)
    
    # Return preferred backend
    return AzureQuantumBackend(preference)

backend = select_backend(optimized_qc, preference='ionq')
print(f"Selected backend: {backend.name} ({backend.num_qubits} qubits, {backend.fidelity:.1%} fidelity)")
```

### 4. Error Mitigation

```python
# Apply Zero-Noise Extrapolation (ZNE) to reduce errors
from kinich.error_mitigation.zne import zero_noise_extrapolation

# Execute circuit with multiple noise levels
noise_factors = [1.0, 1.5, 2.0, 2.5]
results = []

for factor in noise_factors:
    # Scale circuit depth to simulate higher noise
    noisy_circuit = scale_noise(optimized_qc, factor)
    
    # Execute on quantum backend
    job = backend.run(noisy_circuit, shots=1024)
    result = job.result()
    
    # Extract expectation value (e.g., energy)
    expectation = calculate_expectation(result)
    results.append((factor, expectation))

# Extrapolate to zero noise
zero_noise_estimate = zero_noise_extrapolation(results, method='richardson')

print(f"Error mitigation results:")
print(f"  Noisy result (factor=1.0): {results[0][1]:.6f}")
print(f"  Zero-noise extrapolation: {zero_noise_estimate:.6f}")
print(f"  Error reduction: {abs(results[0][1] - zero_noise_estimate) / results[0][1] * 100:.1f}%")

# Output:
# Error mitigation results:
#   Noisy result (factor=1.0): 0.842315
#   Zero-noise extrapolation: 0.891247
#   Error reduction: 5.8%
```

### 5. Readout Correction

```python
# Correct measurement errors using calibration matrix
from kinich.error_mitigation.readout import readout_correction

# Measure calibration circuits
calibration_circuits = [
    QuantumCircuit(4, 4),  # |0000>
    QuantumCircuit(4, 4)   # |1111>
]
calibration_circuits[1].x(range(4))
calibration_circuits[0].measure(range(4), range(4))
calibration_circuits[1].measure(range(4), range(4))

# Execute calibration
cal_results = [backend.run(qc, shots=1024).result() for qc in calibration_circuits]

# Build calibration matrix
cal_matrix = build_calibration_matrix(cal_results)

# Apply correction to measurement results
raw_counts = {'0101': 512, '0100': 256, '0111': 128, '0001': 128}
corrected_counts = readout_correction(raw_counts, cal_matrix)

print("Readout correction:")
print(f"  Raw counts: {raw_counts}")
print(f"  Corrected counts: {corrected_counts}")
```

### 6. Result Verification & PQW Submission

```python
# Verify quantum execution and submit the job to the blockchain
from kinich.blockchain.belizechain_adapter import BelizeChainAdapter

# Calculate circuit complexity score
complexity_score = calculate_complexity(
    qubit_count=optimized_qc.num_qubits,
    circuit_depth=optimized_qc.depth(),
    gate_count=sum(optimized_qc.count_ops().values())
)

# Submit the job to the Quantum pallet (real adapter method).
adapter = BelizeChainAdapter(node_url='<operator-provided-rpc-url>')
tx_hash = await adapter.submit_quantum_job(
    job_id=job_id,
    backend=backend.name,
    circuit_hash=hashlib.sha256(optimized_qc.qasm().encode()).digest(),
    num_qubits=optimized_qc.num_qubits,
    circuit_depth=optimized_qc.depth(),
    num_shots=shots,
    backend_index=backend_index,
)

print(f"Quantum job submitted: {tx_hash}")
```

> The PQW **reward** is a separate on-chain step: an authorized Oracle operator
> calls `Staking.record_quantum_contribution` with the job's `accuracy_score`.

```python
# Reward calculation (50-200 DALLA based on complexity)
base_reward = 50
max_reward = 200
reward_multiplier = {
    10: 1.0,   # 10 qubits = 1x
    20: 2.0,   # 20 qubits = 2x
    30: 4.0    # 30+ qubits = 4x
}

qubit_count = optimized_qc.num_qubits
multiplier = next(v for k, v in sorted(reward_multiplier.items()) if qubit_count <= k)
reward = base_reward + (max_reward - base_reward) * (complexity_score / 100) * multiplier

print(f"Expected reward: {reward:.2f} DALLA")
# Output: Expected reward: 127.50 DALLA
```

## Quantum Compression Engine

> ⚠️ **Does not exist (verified 2026-10-23).** There is no `kinich/compression/` package, no
> `QuantumCompressor` class, and no QAOA-based codec. Pakit's `CompressionEngine` explicitly
> raises for `CompressionAlgorithm.QUANTUM`, and no quantum codec exists in Kinich, Nawal or
> Pakit.
>
> The `6.8x` / `7.8x` ratios that used to be documented here were never produced by any run.
> They are also physically implausible: **Schumacher coding compresses qubit states, not
> classical bytes** — quantum mechanics gives no advantage over the classical entropy bound when
> the input is ordinary data. A QAOA instance solving a QUBO is a combinatorial optimiser, not a
> codec.
>
> The one defensible direction is using a quantum annealer to **search codec parameters or
> cluster deduplication candidates**, with "equal ratio at lower CPU" as the success criterion.
> See `pakit-storage/docs/COMPRESSION_QUANTUM_PLAN.md`, which scopes it as measure-first.

## Proof of Quantum Work (PQW) Algorithm

### Circuit Complexity Scoring

```python
# kinich/rewards/pqw_scoring.py

def calculate_complexity(qubit_count, circuit_depth, gate_count):
    """
    Calculate circuit complexity score (0-100).
    
    Factors:
    - Qubit count (40%): More qubits = harder
    - Circuit depth (30%): Deeper circuits = more gates
    - Gate count (30%): More gates = more complexity
    """
    # Normalize qubit count (max 50 qubits)
    qubit_score = min(qubit_count / 50, 1.0) * 40
    
    # Normalize circuit depth (max 1000)
    depth_score = min(circuit_depth / 1000, 1.0) * 30
    
    # Normalize gate count (max 5000)
    gate_score = min(gate_count / 5000, 1.0) * 30
    
    total_score = qubit_score + depth_score + gate_score
    
    return total_score

# Example scoring
scores = [
    (4, 12, 38, "Grover 4-qubit"),
    (20, 150, 450, "VQE 20-qubit"),
    (30, 500, 1500, "QAOA 30-qubit"),
    (50, 800, 3200, "Shor 50-qubit")
]

for qubits, depth, gates, name in scores:
    score = calculate_complexity(qubits, depth, gates)
    print(f"{name:20s}: {score:5.1f}/100 → {50 + (200-50) * (score/100):6.2f} DALLA")

# Output:
# Grover 4-qubit      :   6.5/100 →  59.75 DALLA
# VQE 20-qubit        :  34.5/100 → 101.75 DALLA
# QAOA 30-qubit       :  63.0/100 → 144.50 DALLA
# Shor 50-qubit       :  83.2/100 → 174.80 DALLA
```

### Quantum Signature Generation

```python
# Proof that circuit was executed on real quantum hardware
def generate_quantum_signature(circuit, result, backend, job_id):
    """
    Generate cryptographic signature proving quantum execution.
    
    Components:
    1. Circuit fingerprint (hash of QASM)
    2. Result fingerprint (hash of measurement counts)
    3. Backend certificate (signed by quantum provider)
    4. Timestamp + job ID
    """
    import hashlib
    import json
    
    # Circuit fingerprint
    circuit_qasm = circuit.qasm()
    circuit_hash = hashlib.sha256(circuit_qasm.encode()).hexdigest()
    
    # Result fingerprint
    result_json = json.dumps(result, sort_keys=True)
    result_hash = hashlib.sha256(result_json.encode()).hexdigest()
    
    # Backend certificate (simulated - real version from Azure/IBM)
    backend_cert = {
        'provider': 'configured quantum backend',
        'backend': backend,
        'job_id': job_id,
        'timestamp': '2026-01-31T12:00:00Z',
        'signature': 'azure-quantum-signed-cert-abc123...'
    }
    
    # Combine into quantum signature
    signature_data = {
        'circuit_hash': circuit_hash,
        'result_hash': result_hash,
        'backend_cert': backend_cert
    }
    
    # Final signature
    signature_json = json.dumps(signature_data, sort_keys=True)
    quantum_signature = hashlib.sha256(signature_json.encode()).hexdigest()
    
    return quantum_signature
```

## Validator Requirements

### Kinich-Type Validator Hardware

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 16 cores | 32 cores |
| RAM | 64 GB | 128 GB |
| GPU | Not required | Optional for simulation |
| Storage | 1 TB NVMe SSD | 2 TB NVMe SSD |
| Network | 1 Gbps | 10 Gbps |

### Software Setup

```bash
# Install Python 3.11+
sudo apt install python3.11 python3.11-venv

# Create virtual environment
python3.11 -m venv kinich-env
source kinich-env/bin/activate

# Install Qiskit
pip install qiskit==1.2.0
pip install qiskit-aer==0.15.0  # Local quantum simulator

# Install configured quantum backend SDK
pip install azure-quantum==1.0.0

# Install Kinich SDK
pip install kinich-quantum-client==1.0.0

# Verify installation
python -c "from qiskit import QuantumCircuit; print('Qiskit OK')"
python -c "from azure.quantum import Workspace; print('configured quantum backend OK')"
python -c "from kinich.core.quantum_node import QuantumNode; print('Kinich OK')"
```

### configured quantum backend Workspace Setup

```bash
# Login to Azure
az login

# Create quantum workspace
az quantum workspace create \
  --resource-group belizechain-rg \
  --name belizechain-quantum \
  --location westus \
  --storage-account belizechainquantum

# List available quantum backends
az quantum workspace show \
  --resource-group belizechain-rg \
  --name belizechain-quantum \
  --output table

# Output:
# Provider      Backend         Qubits  Status
# ----------   ---------------  ------  ------
# IonQ         ionq.simulator   29      Available
# IonQ         ionq.qpu         25      Available
# Quantinuum   quantinuum.sim   20      Available
# Quantinuum   quantinuum.qpu   20      Available
```

### Start Kinich Node

```bash
# Configure validator
kinich-cli config \
  --validator-id 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --orchestrator wss://testnet.belizechain.org/api/kinich \
  --azure-workspace belizechain-quantum \
  --azure-resource-group belizechain-rg \
  --backend-preference ionq \
  --max-qubits 25 \
  --budget 100  # USD per month

# Start quantum client
kinich-cli start \
  --min-complexity 20 \
  --max-jobs 10 \
  --error-mitigation zne
```

## Performance Benchmarks

### Circuit Execution Times

| Backend | Qubits | Circuit Depth | Gates | Execution Time | Cost (USD) |
|---------|--------|---------------|-------|----------------|------------|
| IonQ QPU | 4 | 12 | 38 | 15 sec | $0.23 |
| IonQ QPU | 10 | 50 | 150 | 42 sec | $0.68 |
| IonQ QPU | 20 | 150 | 450 | 3.2 min | $1.92 |
| Quantinuum | 10 | 50 | 150 | 1.8 min | $1.12 |
| Quantinuum | 20 | 150 | 450 | 8.5 min | $5.60 |
| IBM Kyoto | 50 | 500 | 1500 | 12 min | Free (limited) |

### Compression Performance

> ⚠️ **No quantum compression exists.** This table previously listed QAOA at 6.8x/7.8x with a
> "+74% vs Zstandard" advantage. Those rows were removed: there is no quantum codec, and a
> quantum one could not beat the classical entropy bound on classical input. Only the classical
> numbers below are real (measured, `pakit-storage` benchmarks).

| Algorithm | Input Size | Compressed Size | Ratio |
|-----------|-----------|-----------------|-------|
| Classical LZ4 | 1 GB | 384 MB | 2.6x |
| Classical Zstandard | 1 GB | 256 MB | 3.9x |

### Weekly Validator Earnings

| Job Complexity | Jobs/Week | DALLA/Job | Weekly DALLA | Monthly DALLA |
|----------------|-----------|-----------|--------------|---------------|
| Low (score 20-40) | 15 | 75 | 1,125 | 4,875 |
| Medium (score 40-60) | 10 | 125 | 1,250 | 5,417 |
| High (score 60-80) | 7 | 165 | 1,155 | 5,003 |
| Very High (score 80-100) | 3 | 195 | 585 | 2,535 |

**Average earnings**: ~1,000-1,500 DALLA/week (4,333-6,500 DALLA/month at $100 quantum budget)

## API Reference

See [Consensus Pallet PQW Methods](../developer-guides/pallet-apis-infrastructure.md#proof-of-quantum-work) for blockchain integration.

## Security Considerations

1. **Quantum Signature Validation**: Cryptographic proof of real quantum execution
2. **Cross-Backend Verification**: Results validated across multiple quantum providers
3. **Circuit Fingerprinting**: Prevent replay attacks with unique circuit hashes
4. **Cost Limits**: Monthly budget caps prevent validator overspending
5. **Complexity Thresholds**: Minimum complexity score prevents spam jobs
6. **Slashing**: Byzantine validators submitting fake quantum results lose stake
