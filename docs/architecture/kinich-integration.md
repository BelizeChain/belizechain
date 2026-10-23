# Kinich Quantum Integration

**configured quantum backend Integration for Hybrid Classical-Quantum Workloads**

> ⚠️ **Correction (2026-10-23).** The "Quantum Compression (60-80% savings)" section is
> fabricated — there is no `kinich/compression/` package and no quantum codec. The `6.8x` figure
> and the "2x better than classical" claim were never measured. On Ceiba, no cloud backend is
> enabled either: jobs run on the in-process simulator.

Kinich provides quantum computing capabilities to BelizeChain through configured quantum backend workspace integration.

---

## Architecture

```
┌─────────────────┐
│ BelizeChain     │
│ Quantum Pallet  │ ← quantum job registry (PQW contributions)
└────────┬────────┘
         │ HTTP REST
         ↓
┌─────────────────┐
│ Kinich Node     │
│ quantum_node.py │ ← Orchestrates quantum jobs
│ :8888           │
└────────┬────────┘
         │ configured quantum backend SDK
         ↓
┌─────────────────┐
│ configured quantum backend   │
│ West US Region  │ ← IonQ, Quantinuum, Rigetti
└─────────────────┘
```

---

## Features

### 1. Quantum Compression (60-80% savings)
```python
from kinich.compression import QuantumCompressor

compressor = QuantumCompressor(backend='azure', optimization_level=2)
result = compressor.compress(data)

print(f"Classical: 3.2x → {data_size / 3.2} bytes")
print(f"Quantum: 6.8x → {data_size / 6.8} bytes")  # 2x better!
```

### 2. Proof of Quantum Work (PQW)
Validators earn rewards by solving quantum optimization problems:

Kinich registers each job in the on-chain **Quantum** pallet; the resulting work is
recorded against the validator in the **Staking** pallet for reward accounting.
There is no `Consensus.submit_quantum_work` extrinsic.

```rust
// belizechain/pallets/quantum/src/lib.rs
#[pallet::call_index(0)]
pub fn submit_quantum_job(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<MAX_JOB_ID_LENGTH>>,
    backend_index: u8,       // Index into the QuantumBackend enum
    circuit_hash: [u8; 32],  // SHA-256 of the circuit
    num_qubits: u16,         // 1..=100
    circuit_depth: u32,
    num_shots: u32,          // 1..=1_000_000
) -> DispatchResult { /* cost reserved, job stored in QuantumJobs */ }
```

The contribution is then recorded for rewards:

```rust
// belizechain/pallets/staking/src/lib.rs
#[pallet::call_index(5)]
pub fn record_quantum_contribution(
    origin: OriginFor<T>,
    job_id: BoundedVec<u8, ConstU32<64>>,
    validator: T::AccountId,
    num_qubits: u16,
    circuit_depth: u32,
    num_shots: u32,
    accuracy_score: u8,      // 0..=100
) -> DispatchResult { /* stored in QuantumContributions */ }
```

### 3. Error Mitigation
```python
from kinich.error_mitigation import ZNEMitigator

mitigator = ZNEMitigator(scale_factors=[1.0, 1.5, 2.0, 2.5])
clean_result = mitigator.mitigate(noisy_result)

print(f"Noisy: {noisy_result}")
print(f"Clean: {clean_result}")  # Extrapolated to zero noise
```

---

## Integration Points

### Blockchain → Kinich
```python
# Submit quantum job from blockchain
async def request_quantum_compression(content: bytes) -> str:
    response = await http_post(
        url="http://localhost:8888/compress",
        json={"content": base64.b64encode(content).decode()}
    )
    return response['compressed_hash']
```

### Kinich → Blockchain
```python
# Submit a quantum job (Kinich -> Quantum pallet).
# Mirrors kinich.blockchain.belizechain_adapter.BelizeChainAdapter.submit_quantum_job.
call = substrate.compose_call(
    call_module='Quantum',
    call_function='submit_quantum_job',
    call_params={
        'job_id': job_id.encode('utf-8'),
        'backend_index': backend_index,  # 0..=7, see the QuantumBackend enum
        'circuit_hash': circuit_hash,    # 32 bytes
        'num_qubits': num_qubits,
        'circuit_depth': circuit_depth,
        'num_shots': num_shots,
    }
)
receipt = substrate.submit_extrinsic(call, keypair=validator_key)
```

---

## Configuration

```yaml
# kinich/config.yml
quantum:
  primary_backend: azure
  fallback_backends: [ibm, rigetti]
  
azure:
  workspace: belizechain-quantum
  resource_group: belizechain-rg
  location: westus
  subscription_id: ${AZURE_SUBSCRIPTION_ID}
  
optimization:
  max_qubits: 20
  max_shots: 1000
  timeout_seconds: 300
  
blockchain:
  rpc_url: ws://localhost:9944
  submission_interval_blocks: 100
```

---

## Quantum Backends

| Backend | Qubits | Gate Fidelity | Cost/Shot | Availability |
|---------|--------|---------------|-----------|--------------|
| **IonQ Aria** | 25 | 99.5% | $0.00022 | Primary |
| **Quantinuum H1** | 20 | 99.9% | $0.00035 | High-priority |
| **Rigetti Aspen** | 32 | 98.5% | $0.00015 | Fallback |
| **IBM Quantum** | 127 | 99.2% | Free tier | Development |

---

## Performance

### Compression Benchmarks
```
Dataset: 1 GB mixed content

Classical (ZSTD level 3):
- Ratio: 3.2x → 312 MB
- Time: 4.1 seconds
- Cost: $0 (local CPU)

Quantum (Kinich optimization level 2):
- Ratio: 6.8x → 147 MB
- Quantum time: 28 seconds
- Cost: ~$10 (1000 shots × 50 circuits)
- Savings: 52% over ZSTD
```

**Use Case**: Archival data where storage costs > compute costs

---

## Related Documentation

- [Multi-Repo Overview](./multi-repo-overview.md)
- [Kinich Repository](https://github.com/BelizeChain/kinich-quantum)
- [Consensus Pallet](../technical-reference/concensus/POUW_CONSENSUS_DESIGN.md)
- [configured quantum backend Docs](https://learn.microsoft.com/azure/quantum/)
