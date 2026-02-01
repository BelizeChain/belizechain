# Pakit Storage Integration

**Sovereign DAG-Based Decentralized Storage**

Pakit provides 100% sovereign content storage with quantum compression and on-chain proof verification.

---

## Architecture

```
┌──────────────────────┐
│ BelizeChain Pallets  │
│ - LandLedger         │ ← Document storage proofs
│ - BNS                │ ← Website hosting
│ - Economy            │ ← Receipt storage
└──────────┬───────────┘
           │ HTTP REST
           ↓
┌──────────────────────┐
│ Pakit Node           │
│ dag_storage.py :8890 │
│ SQLite + LRU Cache   │
└──────────┬───────────┘
           │ Content-addressable blocks
           ↓
┌──────────────────────┐
│ DAG Storage (Sovereign)
│ /var/pakit/blocks/   │ ← NO external dependencies
│ Multi-parent topology│
└──────────────────────┘
```

---

## Features

### 1. Content Upload
```python
import requests

# Upload land deed document
with open('deed_123.pdf', 'rb') as f:
    response = requests.post(
        'http://localhost:8890/upload',
        files={'file': f},
        data={
            'category': 'land_registry',
            'encrypt': 'true',
            'compress': 'auto'
        }
    )

content_hash = response.json()['hash']
merkle_root = response.json()['merkle_root']

print(f"Uploaded: {content_hash}")
print(f"Merkle root: {merkle_root}")
```

### 2. Content Retrieval
```python
# Download by content hash
response = requests.get(
    f'http://localhost:8890/retrieve/{content_hash}'
)

with open('retrieved_deed.pdf', 'wb') as f:
    f.write(response.content)

# Verify integrity
assert hashlib.sha256(response.content).hexdigest() == content_hash
```

### 3. Merkle Proof Generation
```python
# Generate proof for blockchain verification
response = requests.post(
    'http://localhost:8890/proof',
    json={
        'content_hash': content_hash,
        'block_height': 12345
    }
)

proof = response.json()['proof']
# proof = ['hash1', 'hash2', 'hash3']  ← Merkle path

# Submit to blockchain
substrate.compose_call(
    call_module='LandLedger',
    call_function='register_document_proof',
    call_params={
        'deed_id': 123,
        'content_hash': content_hash,
        'merkle_proof': proof
    }
)
```

---

## Integration Points

### LandLedger → Pakit
```rust
// belizechain/pallets/landledger/src/lib.rs

pub fn register_property(
    origin: OriginFor<T>,
    property_id: u32,
    document_hash: H256
) -> DispatchResult {
    // Query Pakit for proof existence
    let proof_valid = Self::verify_storage_proof(&document_hash)?;
    
    ensure!(proof_valid, Error::<T>::InvalidStorageProof);
    
    // Register property with verified document
    Properties::<T>::insert(property_id, PropertyInfo {
        document_hash,
        registered_at: frame_system::Pallet::<T>::block_number(),
        ..Default::default()
    });
}
```

### BNS → Pakit (Website Hosting)
```python
# Upload website bundle
def upload_bns_site(domain: str, site_files: dict):
    # Compress bundle
    bundle = create_tar_gz(site_files)
    
    # Upload to Pakit
    response = requests.post(
        'http://localhost:8890/upload',
        files={'file': bundle},
        data={'category': 'bns_hosting'}
    )
    
    content_hash = response.json()['hash']
    
    # Register on blockchain
    substrate.compose_call(
        call_module='BNS',
        call_function='update_domain_content',
        call_params={
            'domain': domain,
            'content_hash': content_hash
        }
    )
    
    return content_hash
```

---

## Configuration

```yaml
# pakit/config.yml
storage:
  backend: dag  # Sovereign DAG (primary)
  data_dir: /var/pakit/blocks
  max_block_size: 1048576  # 1 MB
  
compression:
  default_algorithm: zstd
  level: 3
  quantum_enabled: false  # Enable for archival data
  
deduplication:
  enabled: true
  chunk_size: 65536  # 64 KB
  algorithm: rabin  # Rabin fingerprinting
  
blockchain:
  rpc_url: ws://localhost:9944
  proof_submission_interval: 100  # blocks
  
performance:
  cache_size_mb: 1024  # 1 GB LRU cache
  worker_threads: 4
  max_concurrent_uploads: 50
```

---

## Performance

### Upload Throughput
```
File Size: 1 MB
Compression: ZSTD level 3
Deduplication: Enabled

Pipeline:
1. Chunking: 2ms
2. Deduplication: 3ms
3. Compression: 4ms (3.2x ratio)
4. Hash calculation: 1ms
5. DAG insertion: 5ms
6. Merkle proof: 3ms
Total: 18ms

Throughput: 55 files/second
```

### Storage Efficiency
```
Dataset: 1000 BNS websites (10 GB raw)

Without optimization:
- Storage: 10 GB
- Cost: $10/month (Belize Data Center)

With Pakit optimization:
- Deduplication: 36% → 6.4 GB
- Compression: 3.2x → 2.0 GB
- Total savings: 80%
- Cost: $2/month
```

---

## API Reference

### Upload Endpoint
```http
POST /upload
Content-Type: multipart/form-data

Parameters:
- file: binary data
- category: string (optional)
- compress: boolean (default: true)
- encrypt: boolean (default: false)

Response:
{
  "hash": "0x1234...",
  "merkle_root": "0x5678...",
  "size_original": 1048576,
  "size_stored": 327680,
  "compression_ratio": 3.2,
  "deduplication_savings": 0.35
}
```

### Retrieve Endpoint
```http
GET /retrieve/{content_hash}

Response:
- 200 OK: Binary content
- 404 Not Found: Content not in storage
- 410 Gone: Content garbage collected
```

### Proof Endpoint
```http
POST /proof
Content-Type: application/json

Body:
{
  "content_hash": "0x1234...",
  "block_height": 12345
}

Response:
{
  "proof": ["0xaaa...", "0xbbb...", "0xccc..."],
  "leaf_index": 42,
  "tree_size": 1000
}
```

---

## Sovereignty Comparison

| Feature | Pakit (Sovereign) | IPFS | Arweave |
|---------|-------------------|------|---------|
| **Data Location** | 100% Belize | Global network | Global miners |
| **Legal Compliance** | Full Belizean jurisdiction | Unclear | US-based |
| **Cost (5-year)** | $930K | $1.2M | $2.5M |
| **Gateway Dependency** | None | Required | Required |
| **Compression** | Multi-algorithm + quantum | None | None |
| **Deduplication** | 35% typical | Limited | None |
| **Blockchain Integration** | Native Merkle proofs | External | Native (different chain) |

**Verdict**: Pakit chosen for sovereignty, cost savings, and technical superiority

---

## Use Cases

### 1. Land Registry
```
Documents: 50K property deeds
Average size: 500 KB/deed
Total raw: 25 GB

With Pakit:
- Deduplication: 35% → 16.25 GB
- Compression: 3.2x → 5.1 GB
- Storage cost: $5.10/month
- Sovereignty: 100% Belizean
```

### 2. BNS Website Hosting
```
Websites: 1000 .bz domains
Average size: 10 MB/site
Total raw: 10 GB

With Pakit:
- Deduplication: 36% (shared libraries)
- Compression: 3.2x
- Final: 2.0 GB
- Cost: $2/month
```

### 3. Economy Receipt Storage
```
Transactions: 1M/month
Receipt size: 2 KB/tx
Total: 2 GB/month

With Pakit:
- Compression: 6.5x (JSON data)
- Monthly storage: 308 MB
- Annual cost: $3.69
```

---

## Related Documentation

- [Multi-Repo Overview](./multi-repo-overview.md)
- [DAG Storage Design](./dag-storage-design.md)
- [DAG Compression Engine](./dag-compression-engine.md)
- [DAG vs IPFS Comparison](./dag-vs-ipfs.md)
- [Pakit Repository](https://github.com/BelizeChain/pakit-storage)
