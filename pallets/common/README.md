# Common Pallet (Backend Library)

## Overview

The **Common Pallet** is a shared utility library providing reusable types, traits, and helper functions used across multiple BelizeChain pallets. It does **not** provide runtime extrinsics or storage—instead, it serves as a foundational module for consistent data structures and cross-pallet interfaces.

**Core Purpose**: Eliminate code duplication, enforce consistent patterns, and provide battle-tested cryptographic utilities for legal/financial operations.

### Key Features

- **Temporal Anchoring System**: Immutable audit trails for document evolution (land titles, licenses, proposals)
- **Zero Runtime Overhead**: Pure library crate (no storage, no extrinsics, no events)
- **Substrate-Compatible Types**: All types derive `Encode`, `Decode`, `TypeInfo`, `MaxEncodedLen` for storage compatibility
- **Cryptographic Helpers**: Merkle tree generation, hash verification, anchor chain validation
- **Cross-Pallet Traits**: Standard interfaces for document versioning, audit trails, temporal proofs

### Used By Pallets

- **LandLedger**: Temporal anchoring for land title modifications
- **Governance**: Proposal evolution tracking with verifiable history
- **Identity**: Credential amendment audit trails
- **Compliance**: Regulatory document versioning (FSC filings)
- **BNS**: Domain ownership transfer history (future)

## Architecture

### Module Structure
```
common/
├── lib.rs                     # Re-exports and module declarations
└── temporal_anchor.rs         # Temporal anchoring system
```

### Components

1. **TemporalAnchor Type**: Immutable snapshot of content at a specific block
2. **AnchorType Enum**: Categorizes anchored content (land titles, licenses, etc.)
3. **TemporalAnchoring Trait**: Standard interface for implementing anchoring in pallets
4. **Helper Functions**: Merkle root calculation, chain verification, hash utilities

## Core Types

### TemporalAnchor

**Purpose**: Represents a point-in-time snapshot of content, forming a blockchain-within-a-blockchain for document history.

**Structure**:
```rust
pub struct TemporalAnchor<BlockNumber> {
    /// DAG content hash (CID) of the current document version
    pub content_hash: [u8; 32],
    
    /// Hash of the previous anchor in the chain (None for genesis)
    pub previous_hash: Option<[u8; 32]>,
    
    /// Block number when this anchor was created
    pub block_number: BlockNumber,
    
    /// Unix timestamp when this anchor was created
    pub timestamp: u64,
    
    /// Type of content being anchored
    pub anchor_type: AnchorType,
    
    /// Merkle root of all previous anchors in the chain
    /// Allows efficient verification of entire history
    pub merkle_root: [u8; 32],
    
    /// Version number in the anchor chain (0 for genesis)
    pub version: u32,
}
```

**Usage Example** (in LandLedger pallet):
```rust
use pallet_belize_common::TemporalAnchor;

// Create genesis anchor for new land title
let anchor = TemporalAnchor {
    content_hash: title_ipfs_hash,
    previous_hash: None,  // First version
    block_number: current_block,
    timestamp: current_time,
    anchor_type: AnchorType::LandTitle,
    merkle_root: [0u8; 32],  // No history yet
    version: 0,
};
```

### AnchorType

**Purpose**: Categorizes the type of content being anchored for filtering and querying.

**Variants**:
```rust
pub enum AnchorType {
    LandTitle,              // Land title deed or ownership record
    BusinessLicense,        // Business operating license or permit
    GovernanceProposal,     // Governance proposal or referendum
    CourtRecord,            // Court judgment or legal decision
    EducationCredential,    // Educational credential or certificate
    RegulatoryDocument,     // FSC regulatory document
    LegalContract,          // Contract or legal agreement
    IdentityCredential,     // Identity credential or attestation
}
```

**Usage**:
- **LandLedger**: Uses `AnchorType::LandTitle` for property deed versions
- **Governance**: Uses `AnchorType::GovernanceProposal` for proposal amendments
- **Compliance**: Uses `AnchorType::RegulatoryDocument` for FSC filings

## TemporalAnchoring Trait

**Purpose**: Standard interface for pallets that implement temporal anchoring.

**Methods**:

### create_anchor
```rust
fn create_anchor(
    content_hash: [u8; 32],
    anchor_type: AnchorType,
) -> Result<[u8; 32], &'static str>
```

**Purpose**: Create the first anchor in a new chain (genesis anchor).

**Parameters**:
- `content_hash`: DAG content hash (CID) of the content
- `anchor_type`: Type of content being anchored

**Returns**: Hash of the created anchor (used to link future versions)

**Example** (LandLedger implementation):
```rust
impl<T: Config> TemporalAnchoring<BlockNumberFor<T>> for Pallet<T> {
    fn create_anchor(
        content_hash: [u8; 32],
        anchor_type: AnchorType,
    ) -> Result<[u8; 32], &'static str> {
        let anchor = TemporalAnchor {
            content_hash,
            previous_hash: None,
            block_number: frame_system::Pallet::<T>::block_number(),
            timestamp: <T::TimeProvider as Time>::now(),
            anchor_type,
            merkle_root: content_hash,  // Single-node merkle tree
            version: 0,
        };
        
        let hash = sp_io::hashing::blake2_256(&anchor.encode());
        TitleAnchors::<T>::insert(hash, anchor);
        
        Ok(hash)
    }
}
```

### update_anchor
```rust
fn update_anchor(
    previous_anchor_hash: [u8; 32],
    new_content_hash: [u8; 32],
) -> Result<[u8; 32], &'static str>
```

**Purpose**: Create a new anchor linked to the previous version (forms a chain).

**Parameters**:
- `previous_anchor_hash`: Hash of the previous anchor
- `new_content_hash`: DAG hash of the new content version

**Returns**: Hash of the new anchor

**Chain Formation**:
```
Genesis Anchor (v0) → Amendment 1 (v1) → Amendment 2 (v2) → ...
      ↓                    ↓                    ↓
previous_hash: None   previous_hash: v0    previous_hash: v1
```

**Example** (LandLedger implementation):
```rust
fn update_anchor(
    previous_anchor_hash: [u8; 32],
    new_content_hash: [u8; 32],
) -> Result<[u8; 32], &'static str> {
    let prev_anchor = TitleAnchors::<T>::get(previous_anchor_hash)
        .ok_or("Previous anchor not found")?;
    
    // Calculate new merkle root (combines prev_merkle_root + new_content_hash)
    let merkle_input = (prev_anchor.merkle_root, new_content_hash);
    let new_merkle_root = sp_io::hashing::blake2_256(&merkle_input.encode());
    
    let new_anchor = TemporalAnchor {
        content_hash: new_content_hash,
        previous_hash: Some(previous_anchor_hash),
        block_number: frame_system::Pallet::<T>::block_number(),
        timestamp: <T::TimeProvider as Time>::now(),
        anchor_type: prev_anchor.anchor_type,
        merkle_root: new_merkle_root,
        version: prev_anchor.version + 1,
    };
    
    let hash = sp_io::hashing::blake2_256(&new_anchor.encode());
    TitleAnchors::<T>::insert(hash, new_anchor);
    
    Ok(hash)
}
```

### get_anchor
```rust
fn get_anchor(hash: [u8; 32]) -> Option<TemporalAnchor<BlockNumber>>
```

**Purpose**: Retrieve an anchor by its hash.

**Returns**: `Some(anchor)` if found, `None` if not.

### verify_anchor_chain
```rust
fn verify_anchor_chain(hash: [u8; 32]) -> bool
```

**Purpose**: Verify the integrity of an anchor chain by walking backward through history.

**Verification Steps**:
1. **Previous Hash Validation**: Each anchor's `previous_hash` points to a valid anchor
2. **Merkle Root Verification**: Recalculate merkle root from history, compare to stored value
3. **Version Increment Check**: Version numbers increase by 1 each step
4. **Type Consistency**: All anchors in chain have same `AnchorType`

**Example** (simplified):
```rust
fn verify_anchor_chain(hash: [u8; 32]) -> bool {
    let mut current_hash = hash;
    let mut expected_version = None;
    
    loop {
        let anchor = match Self::get_anchor(current_hash) {
            Some(a) => a,
            None => return false,  // Broken chain
        };
        
        // Check version increments correctly
        if let Some(expected) = expected_version {
            if anchor.version != expected {
                return false;
            }
        }
        expected_version = Some(anchor.version.saturating_sub(1));
        
        // If genesis anchor (no previous), chain is valid
        if anchor.previous_hash.is_none() {
            return anchor.version == 0;
        }
        
        current_hash = anchor.previous_hash.unwrap();
    }
}
```

**Returns**: `true` if chain valid, `false` if any link broken

## Helper Functions

### temporal_helpers Module

**Purpose**: Utility functions for temporal anchoring operations.

**Functions** (from `temporal_anchor.rs`):

#### calculate_merkle_root
```rust
pub fn calculate_merkle_root(hashes: &[[u8; 32]]) -> [u8; 32]
```

**Purpose**: Calculate Merkle root from list of content hashes.

**Algorithm**: Standard binary Merkle tree construction
- If odd number of hashes, duplicate last hash
- Recursively hash pairs until single root remains

**Example**:
```rust
use pallet_belize_common::temporal_helpers;

let content_hashes = vec![
    anchor0.content_hash,
    anchor1.content_hash,
    anchor2.content_hash,
];

let merkle_root = temporal_helpers::calculate_merkle_root(&content_hashes);
```

#### verify_merkle_proof
```rust
pub fn verify_merkle_proof(
    leaf: [u8; 32],
    proof: &[[u8; 32]],
    root: [u8; 32],
) -> bool
```

**Purpose**: Verify a Merkle proof without reconstructing entire tree.

**Parameters**:
- `leaf`: Content hash to verify
- `proof`: Sibling hashes along path to root
- `root`: Expected Merkle root

**Returns**: `true` if proof valid

**Use Case**: Prove a specific document version exists in anchor chain without revealing entire history

## Integration Examples

### LandLedger Pallet

**Purpose**: Track land title modifications with immutable history.

**Implementation**:
```rust
use pallet_belize_common::{TemporalAnchor, AnchorType, TemporalAnchoring};

// Storage for title anchor chains
#[pallet::storage]
pub type TitleAnchors<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    [u8; 32],  // Anchor hash
    TemporalAnchor<BlockNumberFor<T>>,
>;

// Create genesis anchor when registering new title
fn register_land_title(...) {
    let anchor_hash = Self::create_anchor(
        title_ipfs_hash,
        AnchorType::LandTitle,
    )?;
    
    TitleRegistry::<T>::insert(title_id, anchor_hash);
}

// Update anchor when transferring ownership
fn transfer_title(...) {
    let current_anchor_hash = TitleRegistry::<T>::get(title_id)?;
    let new_title_hash = /* upload new deed to IPFS */;
    
    let new_anchor_hash = Self::update_anchor(
        current_anchor_hash,
        new_title_hash,
    )?;
    
    TitleRegistry::<T>::insert(title_id, new_anchor_hash);
}

// Verify title history for court cases
fn verify_title_history(title_id: u32) -> bool {
    let current_anchor_hash = TitleRegistry::<T>::get(title_id)?;
    Self::verify_anchor_chain(current_anchor_hash)
}
```

### Governance Pallet

**Purpose**: Track proposal amendments with verifiable history.

**Implementation**:
```rust
use pallet_belize_common::{TemporalAnchor, AnchorType};

// When proposal is amended
fn amend_proposal(proposal_id: u32, new_content_hash: [u8; 32]) {
    let current_anchor = ProposalAnchors::<T>::get(proposal_id)?;
    
    let new_anchor_hash = Self::update_anchor(
        current_anchor,
        new_content_hash,
    )?;
    
    ProposalAnchors::<T>::insert(proposal_id, new_anchor_hash);
    
    Self::deposit_event(Event::ProposalAmended {
        proposal_id,
        version: current_anchor.version + 1,
        anchor_hash: new_anchor_hash,
    });
}
```

## Usage Guidelines

### When to Use Temporal Anchoring

**✅ Use Cases**:
- Legal documents that require audit trails (land titles, licenses)
- Governance proposals that can be amended
- Regulatory filings with version history
- Identity credentials with modification history
- Any document where "who changed what when" matters

**❌ Not Suitable For**:
- Frequently-changing data (block-by-block updates)
- Ephemeral content (chat messages, temporary files)
- Large binary files (use content-addressable storage instead)
- Performance-critical operations (anchoring has storage overhead)

### Best Practices

1. **Use DAG Storage for Content**: Store actual document content in Pakit DAG, only anchor hashes on-chain
2. **Batch Anchor Updates**: Don't create new anchors for minor typo fixes—batch logical changes
3. **Merkle Proof Optimization**: For long chains, use Merkle proofs to verify specific versions without loading entire history
4. **Event Emission**: Always emit events when creating/updating anchors for off-chain indexing
5. **Access Control**: Restrict who can create anchors (e.g., only property owners can update land titles)

## Testing

### Unit Tests
```bash
cargo test -p pallet-belize-common
```

**Test Coverage**:
- TemporalAnchor type serialization (Encode/Decode)
- Merkle root calculation correctness
- Anchor chain verification (valid chains return `true`)
- Broken chain detection (missing anchors, invalid hashes)
- Version number validation

### Example Test
```rust
#[test]
fn test_anchor_chain_integrity() {
    let genesis = TemporalAnchor {
        content_hash: [1u8; 32],
        previous_hash: None,
        version: 0,
        // ...
    };
    
    let amendment = TemporalAnchor {
        content_hash: [2u8; 32],
        previous_hash: Some(hash_of(genesis)),
        version: 1,
        // ...
    };
    
    assert!(verify_anchor_chain(hash_of(amendment)));
}
```

## Future Enhancements

### Phase 2: Merkle Proof Generation
- **Feature**: Helper functions to generate Merkle proofs for specific versions
- **Benefit**: Prove document authenticity without revealing entire chain
- **Timeline**: Q2 2026

### Phase 3: Compression
- **Feature**: Prune old anchor data (keep only recent versions + genesis)
- **Benefit**: Reduce storage costs for long-lived documents
- **Timeline**: Q3 2026

### Phase 4: Cross-Chain Anchors
- **Feature**: Anchor Belize documents to Ethereum/Polkadot for global verification
- **Benefit**: International legal recognition
- **Timeline**: Q1 2027

### Phase 5: Zero-Knowledge Proofs
- **Feature**: Prove document properties without revealing content (e.g., "title has never been mortgaged")
- **Benefit**: Privacy-preserving legal verification
- **Timeline**: Q4 2027

## References

- [Merkle Trees](https://en.wikipedia.org/wiki/Merkle_tree) - Cryptographic foundation
- [Certificate Transparency](https://certificate.transparency.dev/) - Similar audit trail system
- [Git Version Control](https://git-scm.com/) - Inspiration for content-addressable storage
- [LandLedger Pallet](../landledger/README.md) - Primary user of temporal anchoring
- [Governance Pallet](../governance/README.md) - Proposal versioning
- [Pakit Storage](../../pakit/README.md) - Content storage backend

## License
This pallet is part of BelizeChain and is licensed under GPL-3.0.
