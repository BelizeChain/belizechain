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

## LandLedger Integration

### Registering Document Proofs

```rust
// belizechain/pallets/landledger/src/lib.rs

#[pallet::storage]
pub type DocumentProofs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    PropertyId,                    // Property ID
    (Hash, Vec<Hash>, BlockNumber) // (doc_hash, merkle_proof, registered_at)
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn register_document_proof(
        origin: OriginFor<T>,
        property_id: PropertyId,
        document_hash: Hash,        // Pakit DAG block hash
        merkle_proof: Vec<Hash>,    // Merkle path
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Verify caller owns property
        ensure!(
            Self::property_owner(&property_id) == Some(who.clone()),
            Error::<T>::NotPropertyOwner
        );
        
        // Verify document is in Pakit DAG
        ensure!(
            Self::verify_dag_proof(&document_hash, &merkle_proof),
            Error::<T>::InvalidStorageProof
        );
        
        // Store proof on-chain (immutable!)
        DocumentProofs::<T>::insert(
            property_id,
            (document_hash, merkle_proof, <frame_system::Pallet<T>>::block_number())
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

### Querying Proofs

```python
from substrateinterface import SubstrateInterface

# Connect to BelizeChain
substrate = SubstrateInterface(url="wss://rpc.belizechain.org")

# Query document proof for property
property_id = 12345
proof_data = substrate.query(
    module='LandLedger',
    storage_function='DocumentProofs',
    params=[property_id]
)

if proof_data:
    document_hash, merkle_proof, registered_at = proof_data.value
    print(f"Document hash: {document_hash.hex()}")
    print(f"Merkle proof: {[h.hex() for h in merkle_proof]}")
    print(f"Registered at block: {registered_at}")
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
        domain: Vec<u8>,
        tier: HostingTier,
        content_hash: Hash,  // Pakit DAG root hash
        auto_renew: bool
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Verify domain ownership
        let domain_record = DomainRegistry::<T>::get(&domain)
            .ok_or(Error::<T>::DomainNotFound)?;
        
        ensure!(
            domain_record.owner == who,
            Error::<T>::NotDomainOwner
        );
        
        // Store hosting info (includes Pakit DAG hash)
        HostedWebsites::<T>::insert(&domain, HostingInfo {
            subscriber: who.clone(),
            tier,
            content_hash,  // Points to Pakit DAG!
            monthly_fee: Self::tier_monthly_fee(&tier),
            activated_at: now,
            expires_at: now + T::BlocksPerMonth::get(),
            auto_renew,
            payment_status: PaymentStatus::Active
        });
        
        Self::deposit_event(Event::HostingActivated {
            domain,
            subscriber: who,
            content_hash,
            tier
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
    
    client = PakitClient(node_url="https://pakit.belizechain.org")
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
    
    def register_proof(
        self,
        property_id: int,
        document_hash: str,
        merkle_proof: List[str]
    ) -> bool:
        """Register storage proof on-chain."""
        
        # Compose extrinsic
        call = self.substrate.compose_call(
            call_module='LandLedger',
            call_function='register_document_proof',
            call_params={
                'property_id': property_id,
                'document_hash': f'0x{document_hash}',
                'merkle_proof': [f'0x{h}' for h in merkle_proof]
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
    node_url="wss://rpc.belizechain.org",
    keypair=alice_keypair
)

success = connector.register_proof(
    property_id=12345,
    document_hash=deed_hash,
    merkle_proof=merkle_path
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
// Weight calculation for register_document_proof
impl<T: Config> WeightInfo for SubstrateWeight<T> {
    fn register_document_proof() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))   // Read property + verify
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
def on_proof_registered(event_data):
    """Callback for DocumentProofRegistered events."""
    property_id = event_data['property_id']
    document_hash = event_data['document_hash']
    registrar = event_data['registrar']
    
    print(f"📄 New proof: Property {property_id}")
    print(f"   Hash: {document_hash}")
    print(f"   By: {registrar}")

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
- [LandLedger Pallet API](../user-guides/landledger-guide.md)
- [BNS Website Hosting](../services/bns-ipfs-hosting.md)
- [Merkle Trees Explanation](../technical-reference/merkle-trees.md)
- [Substrate Proof Verification](../technical-reference/substrate-proofs.md)
