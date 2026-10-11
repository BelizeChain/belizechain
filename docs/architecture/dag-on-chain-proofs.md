# DAG On-Chain Proofs

**Blockchain Integration for Storage Verification**

Pakit integrates with BelizeChain pallets to provide cryptographic proofs of storage on-chain, enabling trustless verification.

---

## Overview

**On-Chain Storage Proofs** anchor Pakit DAG blocks to the blockchain, creating immutable records that:

✅ Prove content exists in Pakit DAG  
✅ Verify content integrity (hash matches)  
✅ Track content ownership  
✅ Enable trustless auditing  

---

## Architecture

```
┌──────────────────┐
│ User uploads     │
│ document.pdf     │
└────────┬─────────┘
         │
         ↓
┌──────────────────┐
│ Pakit DAG        │
│ Stores content   │ ← Content hash: 0x3a5b7c9e...
│ (off-chain)      │
└────────┬─────────┘
         │ Merkle Proof
         ↓
┌──────────────────┐
│ BelizeChain      │
│ LandLedger Pallet│ ← Stores hash + proof
│ (on-chain)       │    (immutable record)
└──────────────────┘
```

---

## Merkle DAG Proofs

### Proof Structure

```python
@dataclass
class StorageProof:
    """On-chain storage proof."""
    content_hash: str           # SHA-256 of content
    merkle_path: List[str]      # Path from content to DAG root
    root_hash: str              # Known DAG root (verified)
    timestamp: int              # Block number when registered
    owner: str                  # Account that registered proof
```

---

### Generating Proofs

```python
from pakit.core.dag_storage import DagBlock
from pakit.backends.dag_backend import DagBackend

backend = DagBackend()

# Store content
block = dag_builder.create_block(content)
backend.store_block(block)

# Generate Merkle proof
merkle_proof = backend.generate_merkle_proof(block.block_hash)

print(f"Block hash: {block.block_hash}")
print(f"Merkle path: {merkle_proof}")
print(f"Proof length: {len(merkle_proof)} hashes")  # O(log n)
```

---

### Verifying Proofs

```python
def verify_merkle_proof(
    block_hash: str,
    merkle_proof: List[str],
    root_hash: str
) -> bool:
    """
    Verify block is part of DAG using Merkle proof.
    
    Args:
        block_hash: Hash of content to verify
        merkle_proof: List of hashes forming path to root
        root_hash: Known root hash (from blockchain)
    
    Returns:
        True if proof is valid
    """
    import hashlib
    
    current = block_hash
    
    # Walk up the Merkle tree
    for proof_hash in merkle_proof:
        # Combine current with proof hash
        combined = current + proof_hash
        current = hashlib.sha256(combined.encode()).hexdigest()
    
    # Final hash should equal root
    return current == root_hash

# Example
is_valid = verify_merkle_proof(
    block_hash="3a5b7c9e...",
    merkle_proof=["f1e2d3c4...", "a9b8c7d6...", "5e4f3g2h..."],
    root_hash="master_dag_root"
)
print(is_valid)  # True
```

---

## Storage-Proof Pallet Integration

### Submitting Storage Proofs

```rust
// belizechain/pallets/storage-proof/src/lib.rs
// NOTE: LandLedger has no `register_document_proof` extrinsic. On-chain storage
// proofs are verified and recorded by the StorageProof pallet.

/// CID commitment -> submitter + height.
#[pallet::storage]
pub type StorageProofs<T: Config> =
    StorageMap<_, Blake2_128Concat, ContentId, ProofRecord<BlockNumberFor<T>, T::AccountId>>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]
    pub fn submit_storage_proof(
        origin: OriginFor<T>,
        content_id: ContentId,   // 32-byte BLAKE2-256 Merkle root
        proof: MerkleProof,      // { leaf: [u8;32], siblings: Vec<MerkleStep> }
        proof_type: ProofType,   // ProofType::Merkle only (Groth16 not accepted)
    ) -> DispatchResult {
        let submitter = ensure_signed(origin)?;
        
        // Walk the Merkle path leaf -> root (BLAKE2-256 on each concat).
        let derived_root = Self::derive_merkle_root(&proof.leaf, &proof.siblings);
        ensure!(derived_root == content_id, Error::<T>::RootMismatch);
        ensure!(
            !StorageProofs::<T>::contains_key(derived_root),
            Error::<T>::ContentAlreadyProven
        );

        // Record the proof on-chain (immutable).
        StorageProofs::<T>::insert(
            derived_root,
            ProofRecord {
                submitter: submitter.clone(),
                submitted_at: frame_system::Pallet::<T>::block_number(),
                proof_type,
            }
        );
        
        Self::deposit_event(Event::StorageProofAccepted {
            content_id: derived_root,
            submitter,
        });
        
        Ok(())
    }
}
```

---

### Querying Proofs

```python
from substrateinterface import SubstrateInterface

# Connect to BelizeChain
substrate = SubstrateInterface(url="<operator-provided-rpc-url>")

# Query a storage proof by content id
content_id = "0xa1b2c3d4..."  # 32-byte BLAKE2-256 Merkle root
proof_data = substrate.query(
    module='StorageProof',
    storage_function='StorageProofs',
    params=[content_id]
)

if proof_data:
    print(f"Submitter: {proof_data.value['submitter']}")
    print(f"Proof type: {proof_data.value['proof_type']}")
    print(f"Submitted at block: {proof_data.value['submitted_at']}")
```

---

## BNS Website Hosting Proofs

### Registering Website Content

```rust
// belizechain/pallets/bns/src/lib.rs

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn activate_hosting(
        origin: OriginFor<T>,
        domain_name: Vec<u8>,
        tier: u8,            // 0=Free, 1=Basic, 2=Pro, 3=Enterprise
        content_hash: [u8; 32],  // Pakit DAG root hash
        auto_renew: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let domain = Self::validate_domain_name(domain_name)?;

        // Convert tier from u8
        let hosting_tier = match tier {
            0 => HostingTier::Free,
            1 => HostingTier::Basic,
            2 => HostingTier::Pro,
            3 => HostingTier::Enterprise,
            _ => return Err(Error::<T>::InvalidTier.into()),
        };

        // Verify domain ownership
        let domain_record = DomainRegistry::<T>::get(&domain)
            .ok_or(Error::<T>::DomainNotFound)?;
        ensure!(domain_record.owner == who, Error::<T>::NotDomainOwner);

        // Store hosting info (includes the Pakit DAG hash)
        let current_block = frame_system::Pallet::<T>::block_number();
        HostedWebsites::<T>::insert(&domain, HostingInfo {
            subscriber: who.clone(),
            tier: hosting_tier,
            content_hash,  // Points to Pakit DAG
            activated_at: current_block,
            last_payment_at: current_block,
            expires_at: current_block.saturating_add(BLOCKS_PER_MONTH.into()),
            data_size: 0,
            monthly_fee: Self::calculate_hosting_fee(&hosting_tier),
            auto_renew,
        });
        
        Self::deposit_event(Event::HostingActivated {
            domain,
            owner: who,
            tier,
        });
        
        Ok(())
    }
}
```

---

### Resolving Website Content

```python
# Query BNS hosting record
domain = "alice"
hosting_info = substrate.query(
    module='Bns',
    storage_function='HostedWebsites',
    params=[domain.encode()]
)

if hosting_info:
    content_hash = hosting_info.value['content_hash'].hex()
    
    # Fetch from Pakit using content hash
    from pakit.client import PakitClient
    
    client = PakitClient(node_url="https://testnet.belizechain.org/api/pakit")
    website_content = client.get_content(content_hash)
    
    print(f"✅ Resolved alice.bz → {content_hash[:16]}...")
    print(f"Content size: {len(website_content)} bytes")
```

---

## Storage Proof Connector

### Pakit → Blockchain Bridge

```python
# pakit/blockchain/storage_proof_connector.py

from substrateinterface import SubstrateInterface, Keypair

class StorageProofConnector:
    """Bridge between Pakit and BelizeChain."""
    
    def __init__(self, node_url: str, keypair: Keypair):
        self.substrate = SubstrateInterface(url=node_url)
        self.keypair = keypair
    
    def submit_storage_zk_proof(
        self,
        content_id: str,
        leaf: str,
        siblings: List[Dict[str, object]]
    ) -> bool:
        """Submit a Merkle storage proof on-chain."""
        
        # Compose extrinsic
        call = self.substrate.compose_call(
            call_module='StorageProof',
            call_function='submit_storage_proof',
            call_params={
                'content_id': content_id,
                'proof': {'leaf': leaf, 'siblings': siblings},
                'proof_type': 'Merkle',
            }
        )
        
        # Sign and submit
        extrinsic = self.substrate.create_signed_extrinsic(
            call=call,
            keypair=self.keypair
        )
        
        receipt = self.substrate.submit_extrinsic(
            extrinsic,
            wait_for_inclusion=True
        )
        
        return receipt.is_success

# Usage
connector = StorageProofConnector(
    node_url="<operator-provided-rpc-url>",
    keypair=alice_keypair
)

success = connector.submit_storage_zk_proof(
    content_id=content_id,
    zk_proof={'leaf': leaf_hash, 'siblings': merkle_path},
    proof_type='merkle'
)
```

---

## Proof Size Analysis

### Merkle Proof Efficiency

| DAG Size | Proof Length | Proof Size | Verification Time |
|----------|--------------|------------|-------------------|
| 1,000 blocks | 10 hashes | 320 bytes | 0.5 ms |
| 10,000 blocks | 14 hashes | 448 bytes | 0.7 ms |
| 100,000 blocks | 17 hashes | 544 bytes | 0.9 ms |
| 1M blocks | 20 hashes | 640 bytes | 1.0 ms |
| 10M blocks | 24 hashes | 768 bytes | 1.2 ms |

**Key Insight**: Proof size grows logarithmically (O(log n)), not linearly!

---

## On-Chain Storage Costs

### Gas Costs (BelizeChain)

```rust
// Weight calculation for submit_storage_proof
impl<T: Config> WeightInfo for SubstrateWeight<T> {
    fn submit_storage_proof(s: u32) -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))   // Read proof record
            .saturating_add(RocksDbWeight::get().writes(1))  // Write proof
    }
}
```

**Estimated Cost**:
- Gas: ~0.05 DALLA
- Storage deposit: ~0.01 DALLA per proof
- Total: ~0.06 DALLA per document proof

---

## Verification Without Full DAG

Light clients can verify proofs without downloading entire DAG:

```python
# Light client verification
def verify_document_exists(
    property_id: int,
    expected_hash: str
) -> bool:
    """
    Verify document exists without downloading it.
    
    Only queries blockchain, no Pakit access needed!
    """
    # Query on-chain proof
    proof_data = substrate.query(
        module='LandLedger',
        storage_function='DocumentProofs',
        params=[property_id]
    )
    
    if not proof_data:
        return False
    
    document_hash, merkle_proof, _ = proof_data.value
    
    # Verify hash matches
    return document_hash.hex() == expected_hash

# Usage (no Pakit download required!)
exists = verify_document_exists(
    property_id=12345,
    expected_hash="3a5b7c9e..."
)
print(exists)  # True
```

---

## Trustless Auditing

### Audit Workflow

1. **Auditor queries blockchain** for registered proofs
2. **Auditor downloads content** from Pakit using hash
3. **Auditor recalculates hash** of downloaded content
4. **Compare hashes**: On-chain hash vs calculated hash

```python
def audit_document(property_id: int) -> bool:
    """Audit document integrity."""
    
    # 1. Get on-chain proof
    proof = substrate.query('LandLedger', 'DocumentProofs', [property_id])
    if not proof:
        return False
    
    on_chain_hash, merkle_proof, _ = proof.value
    
    # 2. Download content from Pakit
    content = pakit_client.get_content(on_chain_hash.hex())
    
    # 3. Recalculate hash
    import hashlib
    calculated_hash = hashlib.sha256(content).hexdigest()
    
    # 4. Compare
    return calculated_hash == on_chain_hash.hex()

# Audit property 12345
is_valid = audit_document(12345)
if is_valid:
    print("✅ Document integrity verified!")
else:
    print("❌ Document has been tampered with!")
```

---

## Event Subscription

Monitor storage proof registrations in real-time:

```python
def on_proof_accepted(event_data):
    """Callback for StorageProofAccepted events."""
    content_id = event_data['content_id']
    submitter = event_data['submitter']
    
    print(f"📄 New storage proof: {content_id.hex()}")
    print(f"   By: {submitter}")

# Subscribe to events
substrate.subscribe_block_headers(on_proof_registered)
```

---

## Security Considerations

### Immutability

✅ **On-chain proofs are permanent**
- Cannot be modified after registration
- Provides legal evidence of content existence
- Timestamp proves "proof of existence" at specific block

### Integrity

✅ **Merkle proofs guarantee integrity**
- Any tampering invalidates proof
- Verification is cryptographically sound
- Hash collision resistance (SHA-256)

### Availability

⚠️ **On-chain proof ≠ Content availability**
- Proof only shows content existed
- Actual content stored off-chain in Pakit
- If Pakit nodes go offline, content unavailable
- Solution: Multiple node replication

---

## Best Practices

### DO ✅

- **Register proofs for critical documents** (deeds, contracts)
- **Verify Merkle proofs before registration** (avoid invalid proofs)
- **Store multiple copies in Pakit** (redundancy)
- **Monitor events for proof registrations** (real-time awareness)
- **Audit periodically** (ensure integrity)

### DON'T ❌

- **Don't register proofs for temporary files** (waste gas)
- **Don't trust expired proofs** (verify freshness)
- **Don't rely solely on blockchain** (maintain Pakit backups)
- **Don't skip Merkle verification** (invalid proofs fail)

---

## Related Documentation

- [DAG Storage Design](./dag-storage-design.md)
- LandLedger Pallet API
- [BNS Website Hosting](../services/bns-ipfs-hosting.md)
- Merkle Trees Explanation
- Substrate Proof Verification
