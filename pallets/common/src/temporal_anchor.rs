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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// A temporal anchor record representing a point-in-time snapshot of content
///
/// Temporal anchors form a blockchain-within-a-blockchain, creating an immutable
/// history of content evolution. Each anchor links to its predecessor and contains
/// a merkle root of the entire history for cryptographic verification.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
        anchor: &TemporalAnchor<BlockNumber>
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
    use super::*;
    use super::helpers::*;
    
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
        
        // Should be blake2_256 of concatenated hashes
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
    fn verify_merkle_proof_works() {
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        let root = calculate_merkle_root(&[hash1, hash2]);
        
        // Proof for hash1 should be [hash2]
        assert!(verify_merkle_proof(hash1, root, &[hash2]));
        
        // Proof for hash2 should be [hash1]
        assert!(verify_merkle_proof(hash2, root, &[hash1]));
    }
}
