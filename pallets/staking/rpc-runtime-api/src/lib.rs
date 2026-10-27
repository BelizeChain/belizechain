//! Runtime API surface for `pallet-belize-staking`.
//!
//! Read-only projection of validator and epoch state. The validator set is the
//! one thing a client cannot reconstruct cheaply: `Validators` is keyed by
//! account, so "who is validating right now" requires a full map iteration that
//! only the runtime can do.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use pallet_belize_staking::ValidatorInfo;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// Read-only access to staking state.
    pub trait StakingApi<AccountId, Balance, BlockNumber>
    where
        AccountId: Codec,
        Balance: Codec,
        BlockNumber: Codec,
    {
        /// Every registered validator, in unspecified order.
        fn validators() -> Vec<ValidatorInfo<AccountId, Balance, BlockNumber>>;

        /// The active validator count.
        ///
        /// Read from the counter rather than `validators().len()`: the counter
        /// is what the pallet's own thresholds compare against, and the two can
        /// legitimately differ mid-update.
        fn validator_count() -> u32;

        /// The current PoUW epoch number.
        ///
        /// This is a caller-driven counter, not a clock — it only advances when
        /// `distribute_rewards` succeeds.
        fn current_epoch() -> u32;

        /// The block at which rewards were last distributed, or `None` if a
        /// chain has never distributed. This, not `current_epoch`, is what a
        /// client should use to decide whether rewards are due.
        fn last_reward_distribution() -> Option<BlockNumber>;
    }
}
