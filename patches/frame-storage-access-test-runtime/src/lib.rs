/// Stub replacement for `frame-storage-access-test-runtime`.
/// Provides the public API without the WASM build script that panics
/// outside the polkadot-sdk workspace.

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use sp_core::storage::ChildInfo;
use sp_runtime::traits;
use sp_trie::StorageProof;

/// No WASM binary — only needed by `benchmark storage`, not `benchmark pallet`.
pub const WASM_BINARY: Option<&'static [u8]> = None;
pub const WASM_BINARY_BLOATY: Option<&'static [u8]> = None;

#[derive(Decode, Clone, Encode)]
pub struct StorageAccessParams<B: traits::Block> {
    pub state_root: B::Hash,
    pub storage_proof: StorageProof,
    pub payload: StorageAccessPayload,
    pub is_dry_run: bool,
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum StorageAccessPayload {
    Read(Vec<(Vec<u8>, Option<ChildInfo>)>),
    Write((Vec<(Vec<u8>, Vec<u8>)>, Option<ChildInfo>)),
}

impl<B: traits::Block> StorageAccessParams<B> {
    pub fn new_read(
        state_root: B::Hash,
        storage_proof: StorageProof,
        payload: Vec<(Vec<u8>, Option<ChildInfo>)>,
    ) -> Self {
        Self {
            state_root,
            storage_proof,
            payload: StorageAccessPayload::Read(payload),
            is_dry_run: false,
        }
    }

    pub fn new_write(
        state_root: B::Hash,
        storage_proof: StorageProof,
        payload: (Vec<(Vec<u8>, Vec<u8>)>, Option<ChildInfo>),
    ) -> Self {
        Self {
            state_root,
            storage_proof,
            payload: StorageAccessPayload::Write(payload),
            is_dry_run: false,
        }
    }

    pub fn as_dry_run(&self) -> Self {
        Self {
            state_root: self.state_root,
            storage_proof: self.storage_proof.clone(),
            payload: self.payload.clone(),
            is_dry_run: true,
        }
    }
}

pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect("WASM binary not available in stub patch")
}
