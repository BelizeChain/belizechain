//! # Temporal Anchor Module
//!
//! This module provides types and traits for temporal anchoring - a system for creating
//! immutable audit trails of document evolution over time.
//!
//! ## Overview
//!
//! Temporal anchoring solves the problem of tracking legal document changes in a verifiable,
//! tamper-proof way. Each anchor contains:
//! - Content hash (IPFS CID)
//! - Link to previous version
//! - Block number and timestamp
//! - Merkle root of entire history
//!
//! ## Use Cases
//!
//! - **Land Titles**: Track ownership transfers and modifications
//! - **Business Licenses**: Document license amendments and renewals
//! - **Governance Proposals**: Record proposal evolution and amendments
//! - **Court Records**: Maintain immutable court decision history
//! - **Education Credentials**: Track credential issuance and verification

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

/// A temporal anchor record representing a point-in-time snapshot of content
///
/// Temporal anchors form a blockchain-within-a-blockchain, creating an immutable
/// history of content evolution. Each anchor links to its predecessor and contains
/// a merkle root of the entire history for cryptographic verification.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct TemporalAnchor<BlockNumber> {
    /// IPFS content hash (CID) of the current document version
    pub content_hash: [u8; 32],

    /// Hash of the previous anchor in the chain (None for genesis anchor)
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

/// Types of content that can be temporally anchored
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum AnchorType {
    /// Land title deed or ownership record
    LandTitle,

    /// Business operating license or permit
    BusinessLicense,

    /// Governance proposal or referendum
    GovernanceProposal,

    /// Court judgment or legal decision
    CourtRecord,

    /// Educational credential or certificate
    EducationCredential,

    /// Financial Services Commission regulatory document
    RegulatoryDocument,

    /// Contract or legal agreement
    LegalContract,

    /// Identity credential or attestation
    IdentityCredential,
}

/// Trait for pallets that support temporal anchoring
///
/// Implementing this trait allows a pallet to create immutable audit trails
/// of document evolution over time.
pub trait TemporalAnchoring<BlockNumber> {
    /// Create a new anchor for content
    ///
    /// This creates the first anchor in a chain. For updating existing content,
    /// use `update_anchor` which links to the previous version.
    ///
    /// # Parameters
    /// - `content_hash`: IPFS CID of the content being anchored
    /// - `anchor_type`: Type of content (land title, license, etc.)
    ///
    /// # Returns
    /// - `Ok([u8; 32])`: Hash of the created anchor
    /// - `Err(&'static str)`: Error message if creation fails
    fn create_anchor(
        content_hash: [u8; 32],
        anchor_type: AnchorType,
    ) -> Result<[u8; 32], &'static str>;

    /// Update an existing anchor with new content version
    ///
    /// Creates a new anchor that links to the previous one, forming a chain
    /// of document versions.
    ///
    /// # Parameters
    /// - `previous_anchor_hash`: Hash of the previous anchor in the chain
    /// - `new_content_hash`: IPFS CID of the new content version
    ///
    /// # Returns
    /// - `Ok([u8; 32])`: Hash of the new anchor
    /// - `Err(&'static str)`: Error message if update fails
    fn update_anchor(
        previous_anchor_hash: [u8; 32],
        new_content_hash: [u8; 32],
    ) -> Result<[u8; 32], &'static str>;

    /// Retrieve an anchor by its hash
    ///
    /// # Parameters
    /// - `hash`: Hash of the anchor to retrieve
    ///
    /// # Returns
    /// - `Some(TemporalAnchor)`: The anchor if found
    /// - `None`: If no anchor exists with that hash
    fn get_anchor(hash: [u8; 32]) -> Option<TemporalAnchor<BlockNumber>>;

    /// Verify the integrity of an anchor chain
    ///
    /// Walks backward through the anchor chain, verifying that:
    /// 1. Each anchor's previous_hash points to a valid anchor
    /// 2. Merkle roots are correctly calculated
    /// 3. Version numbers increment properly
    ///
    /// # Parameters
    /// - `hash`: Hash of the anchor to start verification from
    ///
    /// # Returns
    /// - `true`: If the entire chain is valid
    /// - `false`: If any link in the chain is broken or invalid
    fn verify_anchor_chain(hash: [u8; 32]) -> bool;

    /// Get the complete history of an anchor chain
    ///
    /// Returns all anchors in the chain, from the given anchor back to the genesis.
    /// Anchors are returned in reverse chronological order (newest first).
    ///
    /// # Parameters
    /// - `hash`: Hash of the latest anchor in the chain
    ///
    /// # Returns
    /// - Vector of all anchors in the chain (empty if hash not found)
    fn get_anchor_history(hash: [u8; 32]) -> Vec<TemporalAnchor<BlockNumber>>;

    /// Get the latest anchor for a specific content
    ///
    /// # Parameters
    /// - `content_hash`: Original content hash to find latest version of
    ///
    /// # Returns
    /// - `Some([u8; 32])`: Hash of the latest anchor in the chain
    /// - `None`: If no anchor exists for that content
    fn get_latest_anchor(content_hash: [u8; 32]) -> Option<[u8; 32]>;
}

/// Helper functions for temporal anchoring
pub mod helpers {
    use super::*;
    use sp_io::hashing::blake2_256;

    /// Calculate the hash of an anchor
    ///
    /// Used to generate the unique identifier for an anchor based on its contents.
    pub fn calculate_anchor_hash<BlockNumber: Encode>(
        anchor: &TemporalAnchor<BlockNumber>,
    ) -> [u8; 32] {
        blake2_256(&anchor.encode())
    }

    /// Calculate merkle root from a list of hashes
    ///
    /// Builds a merkle tree from the given hashes and returns the root.
    /// Used to verify the integrity of an entire anchor chain.
    pub fn calculate_merkle_root(hashes: &[[u8; 32]]) -> [u8; 32] {
        if hashes.is_empty() {
            return [0u8; 32];
        }

        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut current_level = hashes.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut combined = Vec::new();
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[1]);
                    combined
                } else {
                    chunk[0].to_vec()
                };

                next_level.push(blake2_256(&combined));
            }

            current_level = next_level;
        }

        current_level[0]
    }

    /// Verify that a hash is part of a merkle tree
    ///
    /// # Parameters
    /// - `leaf_hash`: The hash to verify
    /// - `merkle_root`: The root of the merkle tree
    /// - `proof`: List of sibling hashes forming the proof path
    ///
    /// # Returns
    /// - `true`: If the hash is part of the tree
    /// - `false`: If verification fails
    pub fn verify_merkle_proof(
        leaf_hash: [u8; 32],
        merkle_root: [u8; 32],
        proof: &[[u8; 32]],
    ) -> bool {
        let mut current_hash = leaf_hash;

        for sibling in proof {
            let mut combined = Vec::new();

            // Order matters for merkle trees
            if current_hash < *sibling {
                combined.extend_from_slice(&current_hash);
                combined.extend_from_slice(sibling);
            } else {
                combined.extend_from_slice(sibling);
                combined.extend_from_slice(&current_hash);
            }

            current_hash = blake2_256(&combined);
        }

        current_hash == merkle_root
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;
    use codec::{Decode, Encode};

    // ── calculate_merkle_root ────────────────────────────────────────────────

    #[test]
    fn calculate_merkle_root_single_hash() {
        let hash = [1u8; 32];
        let root = calculate_merkle_root(&[hash]);
        assert_eq!(root, hash);
    }

    #[test]
    fn calculate_merkle_root_two_hashes() {
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        let root = calculate_merkle_root(&[hash1, hash2]);

        let mut combined = Vec::new();
        combined.extend_from_slice(&hash1);
        combined.extend_from_slice(&hash2);
        let expected = sp_io::hashing::blake2_256(&combined);

        assert_eq!(root, expected);
    }

    #[test]
    fn calculate_merkle_root_empty() {
        let root = calculate_merkle_root(&[]);
        assert_eq!(root, [0u8; 32]);
    }

    #[test]
    fn calculate_merkle_root_three_hashes() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let h3 = [3u8; 32];
        let root = calculate_merkle_root(&[h1, h2, h3]);

        // Level 1: pair(h1,h2) → blake2(h1++h2), odd h3 → blake2(h3)
        let mut c12 = Vec::new();
        c12.extend_from_slice(&h1);
        c12.extend_from_slice(&h2);
        let h12 = sp_io::hashing::blake2_256(&c12);
        let h3_hashed = sp_io::hashing::blake2_256(&h3);

        // Level 2: pair(h12, h3_hashed) → blake2(h12++h3_hashed)
        let mut c_top = Vec::new();
        c_top.extend_from_slice(&h12);
        c_top.extend_from_slice(&h3_hashed);
        let expected = sp_io::hashing::blake2_256(&c_top);

        assert_eq!(root, expected);
    }

    #[test]
    fn calculate_merkle_root_four_hashes() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let h3 = [3u8; 32];
        let h4 = [4u8; 32];
        let root = calculate_merkle_root(&[h1, h2, h3, h4]);

        // Level 1: pair(h1,h2), pair(h3,h4)
        let mut c12 = Vec::new();
        c12.extend_from_slice(&h1);
        c12.extend_from_slice(&h2);
        let h12 = sp_io::hashing::blake2_256(&c12);

        let mut c34 = Vec::new();
        c34.extend_from_slice(&h3);
        c34.extend_from_slice(&h4);
        let h34 = sp_io::hashing::blake2_256(&c34);

        // Level 2: pair(h12, h34)
        let mut c_top = Vec::new();
        c_top.extend_from_slice(&h12);
        c_top.extend_from_slice(&h34);
        let expected = sp_io::hashing::blake2_256(&c_top);

        assert_eq!(root, expected);
    }

    #[test]
    fn calculate_merkle_root_five_hashes() {
        let hashes: Vec<[u8; 32]> = (1u8..=5).map(|i| [i; 32]).collect();
        let root = calculate_merkle_root(&hashes);
        // Just verify it's deterministic and non-zero
        assert_ne!(root, [0u8; 32]);
        assert_eq!(root, calculate_merkle_root(&hashes));
    }

    #[test]
    fn calculate_merkle_root_is_deterministic() {
        let hashes: Vec<[u8; 32]> = (10u8..=15).map(|i| [i; 32]).collect();
        let root1 = calculate_merkle_root(&hashes);
        let root2 = calculate_merkle_root(&hashes);
        assert_eq!(root1, root2);
    }

    #[test]
    fn calculate_merkle_root_order_matters() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let root_12 = calculate_merkle_root(&[h1, h2]);
        let root_21 = calculate_merkle_root(&[h2, h1]);
        assert_ne!(root_12, root_21);
    }

    #[test]
    fn calculate_merkle_root_different_inputs_different_roots() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let h3 = [3u8; 32];
        let root_a = calculate_merkle_root(&[h1, h2]);
        let root_b = calculate_merkle_root(&[h1, h3]);
        assert_ne!(root_a, root_b);
    }

    #[test]
    fn calculate_merkle_root_identical_leaves() {
        let h = [42u8; 32];
        let root = calculate_merkle_root(&[h, h]);
        // blake2_256(h ++ h) — valid even with identical leaves
        let mut combined = Vec::new();
        combined.extend_from_slice(&h);
        combined.extend_from_slice(&h);
        let expected = sp_io::hashing::blake2_256(&combined);
        assert_eq!(root, expected);
    }

    #[test]
    fn calculate_merkle_root_all_zeros() {
        let z = [0u8; 32];
        let root = calculate_merkle_root(&[z, z]);
        // Should still produce a valid (non-zero input) hash
        let mut combined = Vec::new();
        combined.extend_from_slice(&z);
        combined.extend_from_slice(&z);
        let expected = sp_io::hashing::blake2_256(&combined);
        assert_eq!(root, expected);
    }

    // ── verify_merkle_proof ─────────────────────────────────────────────────

    #[test]
    fn verify_merkle_proof_works() {
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        let root = calculate_merkle_root(&[hash1, hash2]);

        assert!(verify_merkle_proof(hash1, root, &[hash2]));
        assert!(verify_merkle_proof(hash2, root, &[hash1]));
    }

    #[test]
    fn verify_merkle_proof_fails_wrong_sibling() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let h3 = [3u8; 32];
        let root = calculate_merkle_root(&[h1, h2]);
        // h3 is not a sibling of h1 in this tree
        assert!(!verify_merkle_proof(h1, root, &[h3]));
    }

    #[test]
    fn verify_merkle_proof_fails_wrong_root() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let wrong_root = [99u8; 32];
        assert!(!verify_merkle_proof(h1, wrong_root, &[h2]));
    }

    #[test]
    fn verify_merkle_proof_empty_proof_leaf_equals_root() {
        // Edge case: empty proof means leaf should equal root
        let h = [42u8; 32];
        assert!(verify_merkle_proof(h, h, &[]));
    }

    #[test]
    fn verify_merkle_proof_empty_proof_leaf_not_root() {
        let h = [42u8; 32];
        let root = [99u8; 32];
        assert!(!verify_merkle_proof(h, root, &[]));
    }

    #[test]
    fn verify_merkle_proof_canonical_ordering() {
        // verify_merkle_proof uses canonical ordering (smaller hash first)
        // so it should work regardless of which leaf we verify
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];

        // Build root using verify_merkle_proof's canonical ordering
        let mut combined = Vec::new();
        if h1 < h2 {
            combined.extend_from_slice(&h1);
            combined.extend_from_slice(&h2);
        } else {
            combined.extend_from_slice(&h2);
            combined.extend_from_slice(&h1);
        }
        let canonical_root = sp_io::hashing::blake2_256(&combined);

        // Both leaves should verify against the canonical root
        assert!(verify_merkle_proof(h1, canonical_root, &[h2]));
        assert!(verify_merkle_proof(h2, canonical_root, &[h1]));
    }

    // ── calculate_anchor_hash ───────────────────────────────────────────────

    #[test]
    fn calculate_anchor_hash_is_deterministic() {
        let anchor = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let hash1 = calculate_anchor_hash(&anchor);
        let hash2 = calculate_anchor_hash(&anchor);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn calculate_anchor_hash_different_content_different_hash() {
        let anchor1 = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let anchor2 = TemporalAnchor::<u64> {
            content_hash: [2u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        assert_ne!(
            calculate_anchor_hash(&anchor1),
            calculate_anchor_hash(&anchor2)
        );
    }

    #[test]
    fn calculate_anchor_hash_different_block_different_hash() {
        let base = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let mut modified = base.clone();
        modified.block_number = 200u64;
        assert_ne!(
            calculate_anchor_hash(&base),
            calculate_anchor_hash(&modified)
        );
    }

    #[test]
    fn calculate_anchor_hash_different_type_different_hash() {
        let base = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let mut modified = base.clone();
        modified.anchor_type = AnchorType::CourtRecord;
        assert_ne!(
            calculate_anchor_hash(&base),
            calculate_anchor_hash(&modified)
        );
    }

    #[test]
    fn calculate_anchor_hash_with_previous_hash() {
        let anchor = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: Some([99u8; 32]),
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [42u8; 32],
            version: 1,
        };
        let hash = calculate_anchor_hash(&anchor);
        assert_ne!(hash, [0u8; 32]);
    }

    // ── TemporalAnchor codec roundtrip ──────────────────────────────────────

    #[test]
    fn temporal_anchor_encode_decode_roundtrip_genesis() {
        let anchor = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 42u64,
            timestamp: 1700000000,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let encoded = anchor.encode();
        let decoded = TemporalAnchor::<u64>::decode(&mut &encoded[..]).expect("decode should work");
        assert_eq!(anchor, decoded);
    }

    #[test]
    fn temporal_anchor_encode_decode_roundtrip_with_previous() {
        let anchor = TemporalAnchor::<u64> {
            content_hash: [10u8; 32],
            previous_hash: Some([20u8; 32]),
            block_number: 500u64,
            timestamp: 1700001000,
            anchor_type: AnchorType::GovernanceProposal,
            merkle_root: [30u8; 32],
            version: 5,
        };
        let encoded = anchor.encode();
        let decoded = TemporalAnchor::<u64>::decode(&mut &encoded[..]).expect("decode should work");
        assert_eq!(anchor, decoded);
    }

    // ── AnchorType encoding ─────────────────────────────────────────────────

    #[test]
    fn anchor_type_encode_decode_all_variants() {
        let variants = [
            AnchorType::LandTitle,
            AnchorType::BusinessLicense,
            AnchorType::GovernanceProposal,
            AnchorType::CourtRecord,
            AnchorType::EducationCredential,
            AnchorType::RegulatoryDocument,
            AnchorType::LegalContract,
            AnchorType::IdentityCredential,
        ];
        for variant in &variants {
            let encoded = variant.encode();
            let decoded = AnchorType::decode(&mut &encoded[..]).expect("decode should work");
            assert_eq!(variant, &decoded);
        }
    }

    #[test]
    fn anchor_type_variants_have_unique_encoding() {
        let variants = [
            AnchorType::LandTitle,
            AnchorType::BusinessLicense,
            AnchorType::GovernanceProposal,
            AnchorType::CourtRecord,
            AnchorType::EducationCredential,
            AnchorType::RegulatoryDocument,
            AnchorType::LegalContract,
            AnchorType::IdentityCredential,
        ];
        let mut encodings = Vec::new();
        for variant in &variants {
            let encoded = variant.encode();
            assert!(!encodings.contains(&encoded), "Duplicate encoding found");
            encodings.push(encoded);
        }
    }

    // ── TemporalAnchor equality ─────────────────────────────────────────────

    #[test]
    fn temporal_anchor_equality() {
        let anchor1 = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let anchor2 = anchor1.clone();
        assert_eq!(anchor1, anchor2);
    }

    #[test]
    fn temporal_anchor_inequality_on_version() {
        let anchor1 = TemporalAnchor::<u64> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 100u64,
            timestamp: 123456,
            anchor_type: AnchorType::LandTitle,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let mut anchor2 = anchor1.clone();
        anchor2.version = 1;
        assert_ne!(anchor1, anchor2);
    }

    // ── Merkle tree with larger inputs ──────────────────────────────────────

    #[test]
    fn calculate_merkle_root_eight_hashes() {
        let hashes: Vec<[u8; 32]> = (1u8..=8).map(|i| [i; 32]).collect();
        let root = calculate_merkle_root(&hashes);
        assert_ne!(root, [0u8; 32]);
        // Verify determinism
        assert_eq!(root, calculate_merkle_root(&hashes));
        // Different set → different root
        let mut different = hashes.clone();
        different[7] = [99u8; 32];
        assert_ne!(root, calculate_merkle_root(&different));
    }

    #[test]
    fn calculate_merkle_root_power_of_two_vs_non_power() {
        let hashes_4: Vec<[u8; 32]> = (1u8..=4).map(|i| [i; 32]).collect();
        let hashes_5: Vec<[u8; 32]> = (1u8..=5).map(|i| [i; 32]).collect();
        // Adding one extra leaf changes the root
        assert_ne!(
            calculate_merkle_root(&hashes_4),
            calculate_merkle_root(&hashes_5)
        );
    }

    // ── AnchorType clone ────────────────────────────────────────────────────

    #[test]
    fn anchor_type_clone_works() {
        let t = AnchorType::LegalContract;
        let t2 = t.clone();
        assert_eq!(t, t2);
    }

    // ── TemporalAnchor with u32 block number ────────────────────────────────

    #[test]
    fn temporal_anchor_works_with_u32_block_number() {
        let anchor = TemporalAnchor::<u32> {
            content_hash: [1u8; 32],
            previous_hash: None,
            block_number: 42u32,
            timestamp: 1000,
            anchor_type: AnchorType::BusinessLicense,
            merkle_root: [0u8; 32],
            version: 0,
        };
        let hash = calculate_anchor_hash(&anchor);
        assert_ne!(hash, [0u8; 32]);
        let encoded = anchor.encode();
        let decoded = TemporalAnchor::<u32>::decode(&mut &encoded[..]).expect("decode");
        assert_eq!(anchor, decoded);
    }
}
