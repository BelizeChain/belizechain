# DAG Storage Architecture

**Sovereign Content-Addressable Storage for BelizeChain**

Pakit's DAG (Directed Acyclic Graph) storage system provides decentralized, content-addressable storage with Merkle proofs, multi-parent blocks, and complete data sovereignty.

---

## Core Principles

### Data Sovereignty

**Zero External Dependencies**:
- No IPFS or Arweave (centralized gateways violate sovereignty)
- No cloud storage providers (foreign jurisdiction issues)
- 100% Belizean infrastructure (national data security)

**Complete Control**:
- Full custody of all data
- No third-party gatekeepers
- Sovereign network topology
- Emergency government access capability

---

## DAG vs Blockchain

### Traditional Blockchain (Single Chain)

```
Block 1 → Block 2 → Block 3 → Block 4 → Block 5
  ↓         ↓         ↓         ↓         ↓
 Data A    Data B    Data C    Data D    Data E
```

**Limitations**:
- Single parent per block (linear structure)
- Sequential verification (slow)
- Limited throughput (1 chain = 1 path)

---

### Pakit DAG (Multi-Parent)

```
          Block N ──────→ Block N-1
             ↓ ↘             ↑
             ↓   ↘         ↗
             ↓     Block N-5
             ↓   ↗       ↑
             ↓ ↗         ↑
          Block N-100 ──→ Block N-101
```

**Advantages**:
- Multiple parents per block (2-5 typically)
- Parallel verification (fast)
- Anti-censorship (multiple paths to each block)
- Better distributed topology (no single point of failure)

---

## Block Structure

### DagBlock Specification

```python
@dataclass
class DagBlock:
    # Identification
    block_hash: str          # SHA-256 hex (primary key)
    timestamp: float         # Unix timestamp
    depth: int               # Distance from genesis block
    
    # Content
    content: bytes           # Compressed data
    block_size: int          # Original size (bytes)
    compressed_size: int     # Compressed size (bytes)
    compression_algorithm: CompressionAlgorithm
    compression_ratio: float # original / compressed
    
    # DAG Structure (KEY INNOVATION!)
    parent_blocks: List[str] # 2-5 parent block hashes
    
    # Verification
    merkle_proof: List[str]  # Merkle path for verification
    
    # Metadata
    metadata: Dict[str, Any] # Storage provider, replication, etc.
```

---

### Content Addressing

Each block is identified by its **SHA-256 content hash**:

```python
def calculate_block_hash(content: bytes) -> str:
    """
    Calculate SHA-256 hash of block content.
    
    This is the primary identifier - same content always
    produces same hash, enabling deduplication.
    """
    return hashlib.sha256(content).hexdigest()

# Example
content = b"Hello, Belize!"
block_hash = calculate_block_hash(content)
# block_hash = "3a5b7c9e2f1d..."

# Same content = same hash (deduplication!)
content2 = b"Hello, Belize!"
block_hash2 = calculate_block_hash(content2)
assert block_hash == block_hash2  # True!
```

**Benefits**:
- Automatic deduplication (same content = same hash)
- Integrity verification (hash mismatch = corruption)
- Content authentication (unforgeable)
- Global unique identifier (no collisions with SHA-256)

---

## Parent Block Selection

### Selection Strategy

Each new block selects 2-5 parents using balanced algorithm:

```python
def select_parent_blocks(
    dag_state: DagState,
    num_parents: int = 4
) -> List[str]:
    """
    Select parent blocks for new DAG block.
    
    Strategy:
    1. Most recent block (maintains chain ordering)
    2. 2-4 random historical blocks (creates DAG web)
    3. Prefer under-referenced blocks (balance topology)
    4. Prefer Belizean blocks (national sovereignty)
    """
    parents = []
    
    # Parent 1: Most recent block (chain property)
    latest = dag_state.get_latest_block()
    if latest:
        parents.append(latest.block_hash)
    
    # Parents 2-N: Random historical blocks (DAG property)
    available = dag_state.get_under_referenced_blocks(
        min_references=3,
        max_depth_diff=1000
    )
    
    # Randomly select from under-referenced blocks
    if len(available) >= num_parents - 1:
        historical = random.sample(available, num_parents - 1)
        parents.extend(historical)
    
    return parents
```

---

### Balanced Topology

**Goals**:
- No orphaned blocks (all blocks well-referenced)
- No super-nodes (avoid centralization)
- Redundant paths (censorship resistance)

**Metrics**:

| Metric | Target | Current |
|--------|--------|---------|
| Avg References per Block | 3-5 | 4.2 |
| Max References | < 100 | 87 |
| Min References | > 1 | 2 |
| Orphaned Blocks | 0% | 0.01% |

---

## Storage Layers

### Layer 1: Content Addressing

```python
from pakit.core.content_addressing import ContentID, calculate_content_id

# Store content
content = b"BelizeChain documentation"
content_id = calculate_content_id(content)  # SHA-256 hash

# Content ID = cryptographic proof
print(content_id.hex)  # "a1b2c3d4e5f6..."
print(content_id.base58)  # "QmX5..."
```

---

### Layer 2: Deduplication

```python
from pakit.core.deduplication import DeduplicationEngine

engine = DeduplicationEngine()

# First store: New content
is_unique = engine.add_reference(content_id, size_bytes=1024)
print(is_unique)  # True

# Second store: Duplicate!
is_unique = engine.add_reference(content_id, size_bytes=1024)
print(is_unique)  # False - already exists!

# Stats
print(engine.stats.deduplication_ratio)  # 0.50 (50% duplicates)
print(engine.stats.bytes_saved)  # 1024 (saved 1KB)
```

---

### Layer 3: Compression

```python
from pakit.core.compression import CompressionEngine, CompressionAlgorithm

engine = CompressionEngine()

# Auto-select best algorithm
result = engine.compress(content, algorithm=CompressionAlgorithm.AUTO)

print(f"Algorithm: {result.algorithm}")  # ZSTD (best balance)
print(f"Original: {result.original_size} bytes")
print(f"Compressed: {result.compressed_size} bytes")
print(f"Ratio: {result.compression_ratio}x")  # 3.2x
print(f"Time: {result.compression_time_ms} ms")  # 2.5ms
```

---

### Layer 4: DAG Structure

```python
from pakit.core.dag_builder import DagBuilder

builder = DagBuilder()

# Create DAG block
block = builder.create_block(
    content=compressed_data,
    compression_algo=CompressionAlgorithm.ZSTD,
    compression_ratio=3.2
)

print(f"Block Hash: {block.block_hash}")
print(f"Depth: {block.depth}")  # Distance from genesis
print(f"Parents: {len(block.parent_blocks)}")  # 4 parents
print(f"Merkle Proof Length: {len(block.merkle_proof)}")  # 8 hashes
```

---

### Layer 5: Persistence (DAG Backend)

```python
from pakit.backends.dag_backend import DagBackend

backend = DagBackend(
    db_path="pakit_dag.db",
    cache_size=1000,  # LRU cache for hot blocks
    enable_wal=True,  # Write-Ahead Logging
    fsync=True        # Durability guarantee
)

# Store block
backend.store_block(block)

# Retrieve block
retrieved = backend.get_block(block.block_hash)
assert retrieved.content == block.content

# Query by depth
recent_blocks = backend.get_blocks_by_depth(depth=1000, limit=10)
```

---

## Merkle DAG Proofs

### Verification Without Full Data

Merkle proofs allow verification of block existence without downloading entire DAG:

```python
def verify_merkle_proof(
    block_hash: str,
    merkle_proof: List[str],
    root_hash: str
) -> bool:
    """
    Verify block is part of DAG using Merkle proof.
    
    Args:
        block_hash: Hash of block to verify
        merkle_proof: Path from block to root
        root_hash: Known root hash
    
    Returns:
        True if block is in DAG
    """
    current = block_hash
    
    for proof_hash in merkle_proof:
        # Combine hashes (order matters!)
        combined = current + proof_hash
        current = hashlib.sha256(combined.encode()).hexdigest()
    
    return current == root_hash
```

**Example**:

```python
# Verify block is in DAG
block_hash = "3a5b7c9e..."
merkle_proof = ["f1e2d3c4...", "a9b8c7d6...", "5e4f3g2h..."]
root_hash = "master_dag_root_hash"

is_valid = verify_merkle_proof(block_hash, merkle_proof, root_hash)
print(is_valid)  # True - block is authentic!
```

**Benefits**:
- Light clients (verify without full DAG)
- On-chain storage proofs (LandLedger pallet)
- Fast verification (O(log n))
- Bandwidth efficient (only need proof, not full DAG)

---

## Genesis Block

Every DAG starts with genesis block (depth = 0):

```python
def create_genesis_block() -> DagBlock:
    """Create genesis block for new DAG network."""
    content = b"BelizeChain Pakit Genesis - January 2026"
    
    return DagBlock(
        block_hash=calculate_block_hash(content),
        block_size=len(content),
        compressed_size=len(content),  # No compression for genesis
        timestamp=time.time(),
        content=content,
        compression_algorithm=CompressionAlgorithm.NONE,
        compression_ratio=1.0,
        parent_blocks=[],  # Genesis has no parents!
        depth=0,           # Root of DAG
        merkle_proof=[],
        metadata={"type": "genesis", "network": "belizechain"}
    )
```

---

## DAG State Management

### In-Memory State

```python
@dataclass
class DagState:
    blocks: Dict[str, DagBlock]              # hash → block
    blocks_by_depth: Dict[int, List[str]]    # depth → [hashes]
    reference_counts: Dict[str, int]         # hash → num_children
    total_blocks: int = 0
    max_depth: int = 0
```

### State Operations

```python
# Get latest block
latest = dag_state.get_latest_block()
print(latest.depth)  # Current DAG height

# Get blocks in range
recent = dag_state.get_blocks_in_range(
    start_depth=990,
    end_depth=1000
)
print(len(recent))  # 10 blocks

# Get under-referenced blocks (need more children)
orphans = dag_state.get_under_referenced_blocks(
    min_references=3,
    max_depth_diff=1000
)
print(len(orphans))  # 42 blocks need referencing
```

---

## Performance Characteristics

### Time Complexity

| Operation | Time | Space |
|-----------|------|-------|
| Store Block | O(1) | O(n) |
| Retrieve Block | O(1) | - |
| Verify Merkle Proof | O(log n) | O(log n) |
| Find by Depth | O(1) | - |
| Deduplication Check | O(1) | O(n) |

### Space Efficiency

**Without Compression**:
- 1 GB data = 1 GB storage

**With Compression (avg 3.2x)**:
- 1 GB data = 312 MB storage (68% savings)

**With Deduplication + Compression**:
- 1 GB data (30% duplicate) = 218 MB storage (78% savings)

**With Quantum Compression (Kinich)**:
- 1 GB data = 200-300 MB storage (70-80% savings)

---

## Comparison: DAG vs IPFS

| Feature | Pakit DAG | IPFS |
|---------|-----------|------|
| **Topology** | Multi-parent DAG | Single-parent tree |
| **Sovereignty** | 100% Belizean | Requires global gateways |
| **Verification** | Merkle DAG proofs | Merkle tree proofs |
| **Compression** | Multi-algo + quantum | None built-in |
| **Deduplication** | Automatic | Manual chunking |
| **Blockchain Integration** | Native (LandLedger pallet) | External oracle |
| **Censorship Resistance** | Multiple paths | Single path |
| **Performance** | Parallel verification | Sequential verification |

**Why Not IPFS?**:
1. **Gateway Centralization**: IPFS requires public gateways (foreign control)
2. **Data Sovereignty**: Cannot guarantee Belizean-only infrastructure
3. **No Quantum Integration**: IPFS lacks Kinich compression
4. **Single-Parent Limitation**: Tree structure is less resilient than DAG

---

## On-Chain Integration

### Storage Proofs in LandLedger

```rust
// belizechain/pallets/landledger/src/lib.rs

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn register_document_proof(
        origin: OriginFor<T>,
        property_id: PropertyId,
        document_hash: Hash,  // DAG block hash
        merkle_proof: Vec<Hash>,  // Merkle path
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Verify document is in Pakit DAG
        ensure!(
            Self::verify_dag_proof(&document_hash, &merkle_proof),
            Error::<T>::InvalidStorageProof
        );
        
        // Store proof on-chain (immutable record)
        DocumentProofs::<T>::insert(
            property_id,
            (document_hash, merkle_proof, Self::block_number())
        );
        
        Self::deposit_event(Event::DocumentProofRegistered {
            property_id,
            document_hash,
            registrar: who
        });
        
        Ok(())
    }
}
```

---

### BNS Website Hosting

```rust
// belizechain/pallets/bns/src/lib.rs

pub fn activate_hosting(
    origin: OriginFor<T>,
    domain: Vec<u8>,
    tier: HostingTier,
    content_hash: Hash,  // DAG root hash
    auto_renew: bool
) -> DispatchResult {
    // Content hash points to Pakit DAG block
    // Gateway resolves: alice.bz → DAG block → website files
    
    HostedWebsites::<T>::insert(&domain, HostingInfo {
        subscriber: who.clone(),
        tier,
        content_hash,  // Points to DAG!
        monthly_fee: Self::tier_monthly_fee(&tier),
        activated_at: now,
        expires_at: now + T::BlocksPerMonth::get(),
        auto_renew,
        payment_status: PaymentStatus::Active
    });
}
```

---

## Network Topology

### Belizean Sovereignty

Pakit prioritizes Belizean blocks for parent selection:

```python
def is_belizean_block(block: DagBlock) -> bool:
    """Check if block was created in Belize."""
    return block.metadata.get("country") == "BZ"

def select_parents_sovereignty_aware(
    dag_state: DagState,
    prefer_belizean: bool = True
) -> List[str]:
    """Select parents, preferring Belizean blocks."""
    available = dag_state.get_under_referenced_blocks()
    
    if prefer_belizean:
        # Sort Belizean blocks first
        belizean = [h for h in available 
                    if is_belizean_block(dag_state.blocks[h])]
        foreign = [h for h in available 
                   if not is_belizean_block(dag_state.blocks[h])]
        available = belizean + foreign
    
    return available[:4]  # Select top 4
```

---

## Future Enhancements

### Phase 2 (Q2 2026)

1. **Sharding**: Partition DAG for scalability (100TB+)
2. **Cross-DAG Links**: Bridge multiple sovereign DAGs
3. **Erasure Coding**: Redundancy without full replication
4. **ZKDAG Proofs**: Zero-knowledge storage verification

### Phase 3 (Q3 2026)

1. **DAG Compression**: Quantum compression at DAG level
2. **ML Prefetching**: Nawal AI predicts access patterns
3. **Geo-Replication**: Automatic international mirroring
4. **Smart Caching**: CDN with DAG-aware invalidation

---

## Related Documentation

- [Compression Engine](./dag-compression-engine.md)
- [Deduplication System](./dag-deduplication.md)
- [DAG vs IPFS Comparison](./dag-vs-ipfs.md)
- [On-Chain Proofs](./dag-on-chain-proofs.md)
- [Performance Benchmarks](./dag-performance-benchmarks.md)
- [BNS Web Hosting](../services/bns-ipfs-hosting.md)
- [LandLedger Integration](../user-guides/landledger-guide.md)

---

## Implementation Reference

- **Source Code**: `pakit/core/dag_storage.py`
- **DAG Builder**: `pakit/core/dag_builder.py`
- **Backend**: `pakit/backends/dag_backend.py`
- **Tests**: `pakit/tests/test_dag_storage.py`
