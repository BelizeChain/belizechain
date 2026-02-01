# DAG Deduplication System

**Never Store the Same Content Twice**

Pakit's deduplication engine eliminates redundant storage by tracking content fingerprints and using reference counting.

---

## Core Concept

**Content-Addressable Storage** = Same content always produces the same hash:

```python
# Store content A
content_a = b"BelizeChain documentation"
hash_a = sha256(content_a).hexdigest()
# hash_a = "3a5b7c9e..."

# Store content B (identical to A)
content_b = b"BelizeChain documentation"
hash_b = sha256(content_b).hexdigest()
# hash_b = "3a5b7c9e..." ← SAME HASH!

# Result: Only store once, reference twice
```

---

## How It Works

### 1. Content ID Calculation

```python
from pakit.core.content_addressing import ContentID, calculate_content_id

content = b"Hello, Belize!"

# Calculate unique content ID (SHA-256)
content_id = calculate_content_id(content)

print(content_id.hex)     # "a1b2c3d4e5f6..."
print(content_id.base58)  # "QmX5..." (IPFS-compatible)
print(content_id.base32)  # "bafybeif..." (CIDv1)
```

---

### 2. Reference Counting

```python
from pakit.core.deduplication import DeduplicationEngine

engine = DeduplicationEngine()

# First reference: Store content
is_new = engine.add_reference(content_id, size_bytes=1024)
print(is_new)  # True

# Second reference: Don't store again!
is_new = engine.add_reference(content_id, size_bytes=1024)
print(is_new)  # False (duplicate detected)

# Check reference count
count = engine.get_reference_count(content_id)
print(count)  # 2 references
```

---

### 3. Garbage Collection

```python
# Remove reference
engine.remove_reference(content_id)
count = engine.get_reference_count(content_id)
print(count)  # 1 reference remaining

# Remove last reference
can_delete = engine.remove_reference(content_id)
print(can_delete)  # True (safe to delete from storage)
```

---

## Deduplication Statistics

Track how much space is saved:

```python
stats = engine.stats

print(f"Total stores: {stats.total_stores}")
print(f"Unique content: {stats.unique_content}")
print(f"Duplicate saves: {stats.duplicate_saves}")
print(f"Bytes saved: {stats.bytes_saved / 1e9:.2f} GB")
print(f"Dedup ratio: {stats.deduplication_ratio:.1%}")
print(f"Efficiency: {stats.efficiency_percent:.1f}%")
```

**Example Output**:
```
Total stores: 10,000
Unique content: 6,500
Duplicate saves: 3,500
Bytes saved: 12.45 GB
Dedup ratio: 35.0%
Efficiency: 35.0%
```

---

## Real-World Deduplication Rates

### Website Hosting

**Scenario**: 100 websites using same libraries (React, Bootstrap)

```
Total files: 50,000
Unique files: 12,000 (76% deduplicated!)
  
Common duplicates:
- react.min.js: 500 copies → 1 stored
- bootstrap.css: 300 copies → 1 stored
- jquery.min.js: 250 copies → 1 stored

Storage saved: 15.2 GB out of 18.6 GB (82%)
```

---

### Document Storage (LandLedger)

**Scenario**: Property documents with standard templates

```
Total documents: 1,000
Unique documents: 650 (35% deduplicated)

Common duplicates:
- Standard deed template: 200 copies
- Title insurance form: 150 copies
- Survey disclaimer: 100 copies

Storage saved: 3.8 GB out of 8.2 GB (46%)
```

---

### BNS Domain Content

**Scenario**: Domain websites using shared assets

```
Total domains: 5,000
Unique content blocks: 15,000 (67% deduplicated)

Common duplicates:
- Favicon.ico: 2,000 copies
- Logo images: 800 copies
- CSS frameworks: 1,500 copies

Storage saved: 42.1 GB out of 89.5 GB (47%)
```

---

## Chunking Strategies

### Fixed-Size Chunking

Simple but inefficient for small edits:

```python
def fixed_chunk(data: bytes, chunk_size: int = 4096) -> List[bytes]:
    """Split data into fixed-size chunks."""
    chunks = []
    for i in range(0, len(data), chunk_size):
        chunks.append(data[i:i + chunk_size])
    return chunks

# Problem: Inserting 1 byte shifts all chunks!
original = b"AAAA" + b"BBBB" + b"CCCC"
modified = b"X" + b"AAAA" + b"BBBB" + b"CCCC"  # All chunks change!
```

---

### Content-Defined Chunking (Rabin Fingerprinting)

Smart chunking that adapts to content:

```python
from pakit.core.deduplication import ContentDefinedChunker

chunker = ContentDefinedChunker(
    min_chunk_size=2048,   # 2 KB minimum
    avg_chunk_size=8192,   # 8 KB average
    max_chunk_size=16384   # 16 KB maximum
)

chunks = chunker.chunk(data)

# Benefits:
# - Inserting content doesn't shift all chunks
# - Better deduplication for modified files
# - Adapts to content boundaries (e.g., line breaks)
```

---

## Deduplication Levels

### File-Level Deduplication

```python
# Entire file is deduplicated
file_hash = calculate_content_id(file_content)
if engine.check_exists(file_hash):
    print("File already stored!")
    engine.add_reference(file_hash, len(file_content))
else:
    print("New file, storing...")
    store_file(file_content)
    engine.add_reference(file_hash, len(file_content))
```

**Pros**: Simple, fast
**Cons**: No dedup for partially similar files

---

### Block-Level Deduplication

```python
# Split into blocks, deduplicate each
blocks = chunker.chunk(file_content)
block_hashes = []

for block in blocks:
    block_hash = calculate_content_id(block)
    
    if not engine.check_exists(block_hash):
        store_block(block)
    
    engine.add_reference(block_hash, len(block))
    block_hashes.append(block_hash)

# Store manifest linking blocks
store_manifest(file_path, block_hashes)
```

**Pros**: Dedup similar files, better for large files
**Cons**: More overhead, complex retrieval

---

## Storage Savings Examples

### Example 1: Code Repository

```python
# Initial commit
commit_1_size = 50 MB
stored = 50 MB
savings = 0%

# Second commit (10% changed)
commit_2_size = 50 MB
stored += 5 MB  # Only changed blocks
total_stored = 55 MB
savings = (100 MB - 55 MB) / 100 MB = 45%

# Third commit (5% changed)
commit_3_size = 50 MB
stored += 2.5 MB
total_stored = 57.5 MB
savings = (150 MB - 57.5 MB) / 150 MB = 62%
```

---

### Example 2: BNS Website Updates

```python
# Original website
website_v1 = 10 MB
stored = 10 MB

# Update homepage (1 MB changed)
website_v2 = 10 MB
stored += 1 MB  # Only changed chunks
total_stored = 11 MB
savings = (20 MB - 11 MB) / 20 MB = 45%

# Add blog post (500 KB new content)
website_v3 = 10.5 MB
stored += 0.5 MB
total_stored = 11.5 MB
savings = (30.5 MB - 11.5 MB) / 30.5 MB = 62%
```

---

## Reference Counting Implementation

```python
class DeduplicationEngine:
    def __init__(self):
        # Map: content_id → reference count
        self.reference_counts: Dict[str, int] = {}
        
        # Map: content_id → size in bytes
        self.content_sizes: Dict[str, int] = {}
        
        # Thread-safe lock
        self._lock = RLock()
    
    def add_reference(self, content_id: ContentID, size_bytes: int) -> bool:
        """
        Add reference to content.
        
        Returns:
            True if new content, False if duplicate
        """
        with self._lock:
            cid_hex = content_id.hex
            
            if cid_hex in self.reference_counts:
                # Duplicate! Increment reference
                self.reference_counts[cid_hex] += 1
                self.stats.duplicate_saves += 1
                self.stats.bytes_saved += size_bytes
                return False
            else:
                # New content
                self.reference_counts[cid_hex] = 1
                self.content_sizes[cid_hex] = size_bytes
                self.stats.unique_content += 1
                return True
    
    def remove_reference(self, content_id: ContentID) -> bool:
        """
        Remove reference to content.
        
        Returns:
            True if safe to delete (no more references)
        """
        with self._lock:
            cid_hex = content_id.hex
            
            if cid_hex not in self.reference_counts:
                return False
            
            self.reference_counts[cid_hex] -= 1
            
            if self.reference_counts[cid_hex] <= 0:
                # No more references, safe to delete
                del self.reference_counts[cid_hex]
                del self.content_sizes[cid_hex]
                return True
            
            return False
```

---

## Integration with DAG Storage

```python
from pakit.core.dag_builder import DagBuilder
from pakit.core.deduplication import DeduplicationEngine

engine = DeduplicationEngine()
builder = DagBuilder()

def store_content(content: bytes) -> str:
    """Store content with deduplication."""
    
    # Calculate content ID
    content_id = calculate_content_id(content)
    
    # Check if already exists
    if engine.check_exists(content_id):
        print("Duplicate detected! Saving storage space.")
        engine.add_reference(content_id, len(content))
        return content_id.hex
    
    # New content: compress and store
    compressed = compress(content)
    block = builder.create_block(compressed)
    backend.store_block(block)
    
    # Track reference
    engine.add_reference(content_id, len(content))
    
    return content_id.hex
```

---

## Garbage Collection

Safe deletion when no more references:

```python
def garbage_collect():
    """Remove unreferenced content."""
    removed_count = 0
    removed_bytes = 0
    
    # Get all content IDs
    all_content = backend.list_all_content()
    
    for content_id in all_content:
        # Check if still referenced
        if engine.get_reference_count(content_id) == 0:
            # No references, safe to delete
            size = backend.delete_content(content_id)
            removed_count += 1
            removed_bytes += size
    
    print(f"GC: Removed {removed_count} blocks, freed {removed_bytes / 1e6:.2f} MB")
```

---

## Deduplication vs Compression

**Both are complementary!**

```python
# 1. Deduplication (eliminate duplicates)
unique_content = deduplication_engine.filter_duplicates(all_content)
# Savings: 35% (3.5 GB out of 10 GB)

# 2. Compression (compress unique content)
compressed_content = compression_engine.compress(unique_content)
# Savings: 3.2x (6.5 GB → 2.0 GB)

# Total savings: 80% (10 GB → 2.0 GB)
```

---

## Best Practices

### DO ✅

- **Enable deduplication by default** (automatic savings)
- **Use content-defined chunking** for large files
- **Track reference counts** (safe deletion)
- **Run periodic garbage collection** (free space)
- **Monitor deduplication stats** (measure effectiveness)
- **Combine with compression** (maximize savings)

### DON'T ❌

- **Don't delete without checking references** (data loss!)
- **Don't use fixed chunking for versioned files** (poor dedup)
- **Don't skip content ID calculation** (duplicates slip through)
- **Don't forget thread safety** (race conditions)
- **Don't disable dedup for "performance"** (false economy)

---

## Performance Considerations

### Hash Calculation Speed

```python
# SHA-256 is fast: ~400 MB/s on modern CPU
content = b"x" * 1_000_000  # 1 MB
start = time.time()
content_id = calculate_content_id(content)
elapsed = time.time() - start
print(f"Hashed 1 MB in {elapsed * 1000:.2f} ms")
# Output: Hashed 1 MB in 2.5 ms
```

### Lookup Speed

```python
# O(1) hash table lookup
exists = engine.check_exists(content_id)  # < 0.001 ms
```

### Memory Usage

```python
# Reference counts: ~100 bytes per unique content
# 1M unique files = ~100 MB memory
# 10M unique files = ~1 GB memory (acceptable)
```

---

## Related Documentation

- [DAG Storage Design](./dag-storage-design.md)
- [Compression Engine](./dag-compression-engine.md)
- [Content Addressing](../technical-reference/content-addressing.md)
- [Performance Benchmarks](./dag-performance-benchmarks.md)
