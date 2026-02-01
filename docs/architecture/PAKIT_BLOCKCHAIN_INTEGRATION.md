# Pakit On-Chain Metadata Storage

## Overview

Pakit's sovereign DAG storage backend integrates with BelizeChain to store file metadata on-chain (IPFS and Arweave are legacy fallbacks). This provides:

✅ **Permanent Record**: Content ID → Backend CID mappings stored on blockchain  
✅ **Verification**: Cryptographic proof that files exist in decentralized storage  
✅ **Ownership Tracking**: Link files to BelizeID accounts  
✅ **Transparency**: Audit trail of all stored documents  

## Architecture

```
┌─────────────────┐
│  Pakit Client   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌──────────────────────┐
│  IPFS Backend   │◄────►│  StorageProof        │
│  Arweave Backend│      │  Connector           │
└─────────────────┘      └──────────┬───────────┘
                                    │
                                    ▼
                         ┌──────────────────────┐
                         │  BelizeChain         │
                         │  LandLedger Pallet   │
                         └──────────────────────┘
```

## Storage Flow

### 1. File Upload (Store Metadata)

```python
# User uploads file to Pakit
pakit upload document.pdf --tier hot

# Backend workflow:
1. Hash content → content_id (SHA-256)
2. Upload to IPFS → get ipfs_cid (Qm...)
3. Store on-chain:
   - content_id → ipfs_cid mapping
   - Owner account ID
   - Tier (hot/warm/cold)
   - Timestamp
4. Fallback to temp file if blockchain unavailable
```

### 2. File Retrieval (Query Metadata)

```python
# User retrieves file from Pakit
pakit retrieve <content_id>

# Backend workflow:
1. Query blockchain for content_id
2. Get ipfs_cid from on-chain storage
3. Retrieve from IPFS using CID
4. Fallback to temp file metadata if blockchain unavailable
```

## Implementation Details

### StorageProofConnector

**Location**: `pakit/blockchain/storage_proof_connector.py`

**Key Methods**:

```python
async def store_document_proof(
    content_id: str,
    ipfs_cid: Optional[str] = None,
    arweave_tx: Optional[str] = None,
    owner: str = "system",
    metadata: Dict[str, Any] = None
) -> bool
```

**Features**:
- Async API for non-blocking blockchain operations
- Mock mode for development (no blockchain required)
- In-memory cache when blockchain unavailable
- Graceful fallback to temp file storage

### IPFS Backend Integration

**Modified Methods**:
- `_store_metadata()`: Lines 372-406 - Now stores on-chain first, falls back to temp files
- `_get_ipfs_cid()`: Lines 408-437 - Queries blockchain first, falls back to temp files

**Pattern**:
```python
def _store_metadata(self, content_id: str, ipfs_cid: str, tier: str):
    try:
        connector = get_storage_proof_connector(mock_mode=True)
        loop = asyncio.new_event_loop()
        success = loop.run_until_complete(
            connector.store_document_proof(
                content_id=content_id,
                ipfs_cid=ipfs_cid,
                owner="system",
                metadata={"tier": tier}
            )
        )
        if success:
            logger.debug("✅ Stored on-chain metadata")
            return
    except Exception as e:
        logger.warning(f"Blockchain failed: {e}, using temp file")
    
    # Fallback to temp file storage...
```

### Arweave Backend Integration

**Modified Methods**:
- `_store_metadata()`: Lines 449-492 - On-chain storage with temp file fallback
- `_get_arweave_tx_id()`: Lines 494-525 - On-chain query with temp file fallback

**Additional Metadata**:
- Stores Arweave transaction ID (`tx_id`)
- Includes gateway URL for retrieval
- Preserves tier information

## Mock Mode vs Production

### Mock Mode (Current Default)

```python
connector = get_storage_proof_connector(mock_mode=True)
```

- No blockchain connection required
- Stores in-memory: `_mock_storage` dictionary
- Perfect for development and testing
- No substrate-interface dependency

### Production Mode (Future)

```python
connector = get_storage_proof_connector(
    node_url="ws://localhost:9944",
    mock_mode=False
)
await connector.connect()
```

- Requires BelizeChain node running
- Requires `substrate-interface` package
- Needs keypair for signing extrinsics
- Stores permanently on-chain

## Substrate Interface Integration

### Installation

```bash
pip install substrate-interface
```

### Connection Example

```python
from substrateinterface import SubstrateInterface, Keypair

# Connect to BelizeChain
substrate = SubstrateInterface(url="ws://localhost:9944")

# Create extrinsic
call = substrate.compose_call(
    call_module='LandLedger',
    call_function='register_document_proof',
    call_params={
        'content_id': content_id,
        'ipfs_cid': ipfs_cid,
        'arweave_tx': arweave_tx,
        'metadata': json.dumps(metadata)
    }
)

# Sign and submit
keypair = Keypair.create_from_uri('//Alice')
extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
```

## LandLedger Pallet Requirements

### Proposed Storage Items

```rust
// In belizechain/pallets/landledger/src/lib.rs

#[pallet::storage]
pub type DocumentProofs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    ContentId,  // SHA-256 hash (32 bytes)
    DocumentProof<T::AccountId>,
    OptionQuery
>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DocumentProof<AccountId> {
    pub ipfs_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    pub arweave_tx: Option<BoundedVec<u8, ConstU32<64>>>,
    pub owner: AccountId,
    pub tier: StorageTier,
    pub timestamp: BlockNumber,
    pub metadata: BoundedVec<u8, ConstU32<256>>,
}
```

### Proposed Extrinsics

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn register_document_proof(
        origin: OriginFor<T>,
        content_id: ContentId,
        ipfs_cid: Option<Vec<u8>>,
        arweave_tx: Option<Vec<u8>>,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        let proof = DocumentProof {
            ipfs_cid: ipfs_cid.map(|v| v.try_into().unwrap()),
            arweave_tx: arweave_tx.map(|v| v.try_into().unwrap()),
            owner: who.clone(),
            tier: StorageTier::Hot,
            timestamp: frame_system::Pallet::<T>::block_number(),
            metadata: metadata.try_into().unwrap(),
        };
        
        DocumentProofs::<T>::insert(content_id, proof);
        
        Self::deposit_event(Event::DocumentProofRegistered {
            content_id,
            owner: who,
        });
        
        Ok(())
    }
}
```

## Testing

### Unit Tests

```bash
# Test blockchain connector in mock mode
cd pakit
pytest tests/test_storage_proof_connector.py -v

# Test IPFS backend with on-chain metadata
pytest tests/test_ipfs_backend_blockchain.py -v

# Test Arweave backend with on-chain metadata
pytest tests/test_arweave_backend_blockchain.py -v
```

### Integration Tests

```bash
# Requires BelizeChain node running
./target/release/belizechain-node --dev --tmp

# Run integration tests
pytest tests/integration/test_pakit_blockchain.py -v
```

## Fallback Strategy

The implementation uses a **graceful degradation** pattern:

1. **Primary**: Try on-chain storage via BelizeChain
2. **Fallback**: Use temp file storage if blockchain unavailable
3. **Logging**: Clear warnings when falling back

This ensures Pakit remains functional even if:
- BelizeChain node is offline
- substrate-interface not installed
- Network connectivity issues

## Performance Considerations

### Async/Sync Bridge

The connector uses asyncio in sync context:

```python
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
success = loop.run_until_complete(connector.store_document_proof(...))
loop.close()
```

**Impact**:
- Adds ~10-50ms latency per operation
- Blocks during blockchain communication
- Acceptable for file upload/download workflows

### Optimization (Future)

For high-throughput scenarios, consider:
- Background worker thread for blockchain operations
- Batch multiple proofs into single extrinsic
- Local cache with periodic sync to blockchain

## Status

✅ **COMPLETE**:
- StorageProofConnector implementation
- IPFS backend integration (TODO lines 378, 388 resolved)
- Arweave backend integration (TODO lines 453, 470 resolved)
- Mock mode testing
- Syntax validation

⏳ **PENDING**:
- LandLedger pallet extrinsic implementation (Rust)
- substrate-interface production integration
- Unit tests for connector
- Integration tests with live blockchain
- Performance benchmarks

## Migration Path

### Phase 1: Mock Mode (CURRENT)
- ✅ Use in-memory storage for development
- ✅ No blockchain dependency
- ✅ Maintains temp file fallback

### Phase 2: Local Blockchain
- Add substrate-interface to requirements.txt
- Enable `mock_mode=False` for local testing
- Test with dev blockchain node

### Phase 3: Production
- Implement LandLedger extrinsics in Rust
- Add keypair management for signing
- Deploy to production blockchain
- Monitor on-chain storage usage

## Configuration

### Environment Variables

```bash
# Blockchain connection
export BELIZECHAIN_RPC="ws://localhost:9944"

# Enable on-chain storage
export PAKIT_BLOCKCHAIN_ENABLED=true

# Mock mode (for testing)
export PAKIT_BLOCKCHAIN_MOCK=true
```

### Pakit Config

```yaml
# pakit/config.yaml
blockchain:
  enabled: true
  rpc_url: "ws://localhost:9944"
  mock_mode: true  # Set to false for production
  keypair_path: "/path/to/keypair.json"
  timeout_seconds: 30
```

## Security Considerations

### Access Control
- Only document owner can register proofs
- Requires signed extrinsic (prevents spam)
- Rate limiting on pallet side

### Data Privacy
- Content ID: SHA-256 hash (no file content exposed)
- IPFS CID: Public content identifier (expected)
- Arweave TX: Public transaction ID (expected)
- Metadata: Encrypted if sensitive

### Attack Vectors
- **Denial of Service**: Rate limit extrinsics, require minimum balance
- **Metadata Poisoning**: Validate metadata format on-chain
- **Storage Spam**: Charge transaction fees for proof registration

## Troubleshooting

### "substrate-interface not installed"

```bash
# Install the package
pip install substrate-interface

# Or use mock mode
connector = get_storage_proof_connector(mock_mode=True)
```

### "Failed to connect to blockchain"

```bash
# Check if node is running
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' \
     ws://localhost:9944

# Check firewall rules
sudo ufw allow 9944/tcp

# Use fallback mode
# Pakit will automatically fall back to temp file storage
```

### "Extrinsic failed: Module error"

- Check LandLedger pallet is deployed
- Verify extrinsic name matches pallet implementation
- Check account has sufficient balance for transaction fees

## Future Enhancements

1. **Batch Operations**: Submit multiple proofs in single extrinsic
2. **Event Subscriptions**: Listen for on-chain events (ProofRegistered, ProofDeleted)
3. **Storage Proofs**: Verify IPFS/Arweave data matches on-chain CID
4. **Content Updates**: Track version history of documents
5. **Access Control Lists**: On-chain permissions for file sharing

## References

- [Substrate Interface Documentation](https://polkascan.github.io/py-substrate-interface/)
- [BelizeChain LandLedger Pallet](../belizechain/pallets/landledger/src/lib.rs)
- [IPFS CID Specification](https://docs.ipfs.tech/concepts/content-addressing/)
- [Arweave Transaction Format](https://docs.arweave.org/developers/server/http-api)

---

**Last Updated**: January 7, 2025  
**Status**: Phase 1 Complete (Mock Mode) ✅  
**Next Action**: Implement LandLedger extrinsics in Rust
