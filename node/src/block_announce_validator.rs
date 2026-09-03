//! Block Announce Validator for BelizeChain
//!
//! **P2P-FIX-002 (HIGH)**: Content-based block announcement validation for
//! defense-in-depth against eclipse and spam attacks.
//!
//! Substrate's `BlockAnnounceValidator` trait validates the announcement CONTENT
//! (header + associated data), not the peer identity. Peer-level filtering is
//! handled at the network layer via `--reserved-only` plus an operator-managed
//! reserved-nodes file (see P2P-FIX-003).
//!
//! This validator enforces:
//! - Block announcements must not carry unexpected associated data
//! - Block numbers must not be unreasonably far in the future
//! - Associated data payload must not exceed a sane size limit

use futures::FutureExt;
use sp_consensus::block_validation::{BlockAnnounceValidator, Validation};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, NumberFor};
use std::marker::PhantomData;
use std::pin::Pin;

/// Maximum number of blocks in the future we accept in announcements.
/// Anything further ahead is likely spam or a fork attack.
const MAX_BLOCKS_AHEAD: u32 = 64;

/// Maximum size of associated data in bytes.
/// Legitimate block announcements in BelizeChain carry empty data.
const MAX_ASSOCIATED_DATA_SIZE: usize = 256;

/// Content-based block announce validator for BelizeChain.
///
/// Validates block announcement headers and data payloads. Peer identity
/// filtering is handled separately by the network layer (`--reserved-only`).
pub struct BelizeBlockAnnounceValidator<B: BlockT> {
    best_number: std::sync::Arc<std::sync::atomic::AtomicU32>,
    _phantom: PhantomData<B>,
}

impl<B: BlockT> BelizeBlockAnnounceValidator<B> {
    pub fn new() -> Self {
        Self::new_with_best(0)
    }

    pub fn new_with_best(initial_best: u32) -> Self {
        log::info!(
            target: "belizechain::block_announce",
            "Initialized BelizeBlockAnnounceValidator with initial best #{} (content-based validation)",
            initial_best,
        );
        Self {
            best_number: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(initial_best)),
            _phantom: PhantomData,
        }
    }
}

impl<B: BlockT> BlockAnnounceValidator<B> for BelizeBlockAnnounceValidator<B>
where
    NumberFor<B>: Into<u64>,
{
    fn validate(
        &mut self,
        header: &B::Header,
        data: &[u8],
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Validation, Box<dyn std::error::Error + Send>>>
                + Send,
        >,
    > {
        let block_number: u64 = (*header.number()).into();
        let data_len = data.len();
        let data_is_empty = data.is_empty();
        let best = self.best_number.clone();

        async move {
            // Reject announcements with oversized associated data
            if data_len > MAX_ASSOCIATED_DATA_SIZE {
                log::warn!(
                    target: "belizechain::block_announce",
                    "Block #{} announce rejected: associated data too large ({} bytes, max {})",
                    block_number, data_len, MAX_ASSOCIATED_DATA_SIZE,
                );
                return Ok(Validation::Failure { disconnect: true });
            }

            // Reject non-empty associated data — BelizeChain announces should
            // carry no extra payload. Non-empty data is suspicious.
            if !data_is_empty {
                log::warn!(
                    target: "belizechain::block_announce",
                    "Block #{} announce rejected: unexpected non-empty associated data ({} bytes)",
                    block_number, data_len,
                );
                return Ok(Validation::Failure { disconnect: true });
            }

            // Track the highest known block to detect far-future announcements
            let current_best = best.load(std::sync::atomic::Ordering::Relaxed) as u64;
            let announced_number = block_number;

            if announced_number > current_best + MAX_BLOCKS_AHEAD as u64 {
                log::warn!(
                    target: "belizechain::block_announce",
                    "Block #{} announce rejected: too far ahead of best known #{} (max {} ahead)",
                    announced_number, current_best, MAX_BLOCKS_AHEAD,
                );
                return Ok(Validation::Failure { disconnect: false });
            }

            // Update best known block number if this announcement is ahead
            if announced_number > current_best {
                // Saturating cast is safe — block numbers won't exceed u32::MAX
                // for any realistic chain in the foreseeable future
                let new_best = announced_number.min(u32::MAX as u64) as u32;
                best.fetch_max(new_best, std::sync::atomic::Ordering::Relaxed);
            }

            log::trace!(
                target: "belizechain::block_announce",
                "Block #{} announce accepted",
                announced_number,
            );
            Ok(Validation::Success { is_new_best: false })
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;

    // Use the opaque block type from the runtime
    type Block = belizechain_runtime::opaque::Block;

    #[test]
    fn test_accepts_valid_announcement() {
        let mut validator = BelizeBlockAnnounceValidator::<Block>::new();
        let header = sp_runtime::generic::Header::<u32, sp_runtime::traits::BlakeTwo256> {
            parent_hash: Default::default(),
            number: 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Default::default(),
        };
        let result = validator
            .validate(&header, &[])
            .now_or_never()
            .unwrap()
            .unwrap();
        assert_eq!(result, Validation::Success { is_new_best: false });
    }

    #[test]
    fn test_rejects_nonempty_data() {
        let mut validator = BelizeBlockAnnounceValidator::<Block>::new();
        let header = sp_runtime::generic::Header::<u32, sp_runtime::traits::BlakeTwo256> {
            parent_hash: Default::default(),
            number: 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Default::default(),
        };
        let result = validator
            .validate(&header, &[0x42])
            .now_or_never()
            .unwrap()
            .unwrap();
        assert_eq!(result, Validation::Failure { disconnect: true });
    }

    #[test]
    fn test_rejects_far_future_block() {
        let mut validator = BelizeBlockAnnounceValidator::<Block>::new();
        // Block number far in the future (MAX_BLOCKS_AHEAD + 1)
        let header = sp_runtime::generic::Header::<u32, sp_runtime::traits::BlakeTwo256> {
            parent_hash: Default::default(),
            number: MAX_BLOCKS_AHEAD + 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Default::default(),
        };
        let result = validator
            .validate(&header, &[])
            .now_or_never()
            .unwrap()
            .unwrap();
        assert_eq!(result, Validation::Failure { disconnect: false });
    }

    #[test]
    fn test_accepts_near_future_block() {
        let mut validator = BelizeBlockAnnounceValidator::<Block>::new();
        // Block number exactly at MAX_BLOCKS_AHEAD should be accepted
        let header = sp_runtime::generic::Header::<u32, sp_runtime::traits::BlakeTwo256> {
            parent_hash: Default::default(),
            number: MAX_BLOCKS_AHEAD,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Default::default(),
        };
        let result = validator
            .validate(&header, &[])
            .now_or_never()
            .unwrap()
            .unwrap();
        assert_eq!(result, Validation::Success { is_new_best: false });
    }
}
