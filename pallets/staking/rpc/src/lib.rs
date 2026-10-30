//! JSON-RPC server for [`StakingApi`](pallet_belize_staking_rpc_runtime_api::StakingApi).
//!
//! Thin wrapper over the runtime API: no caching, no derived values.

use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};
use pallet_belize_staking::ValidatorInfo;
use pallet_belize_staking_rpc_runtime_api::StakingApi as StakingRuntimeApi;
use serde::{de::DeserializeOwned, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait StakingApi<BlockHash, AccountId, Balance, BlockNumber>
where
    AccountId: Serialize + DeserializeOwned,
    Balance: Serialize + DeserializeOwned,
    BlockNumber: Serialize + DeserializeOwned,
{
    #[method(name = "belizechain_stakingValidators")]
    fn validators(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<ValidatorInfo<AccountId, Balance, BlockNumber>>>;

    #[method(name = "belizechain_stakingValidatorCount")]
    fn validator_count(&self, at: Option<BlockHash>) -> RpcResult<u32>;

    #[method(name = "belizechain_stakingCurrentEpoch")]
    fn current_epoch(&self, at: Option<BlockHash>) -> RpcResult<u32>;

    #[method(name = "belizechain_stakingLastRewardDistribution")]
    fn last_reward_distribution(&self, at: Option<BlockHash>) -> RpcResult<Option<BlockNumber>>;
}

const RUNTIME_ERROR: i32 = 1;

pub struct Staking<C, Block, AccountId, Balance, BlockNumber> {
    client: Arc<C>,
    _marker: PhantomData<(Block, AccountId, Balance, BlockNumber)>,
}

impl<C, Block, AccountId, Balance, BlockNumber> Staking<C, Block, AccountId, Balance, BlockNumber> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<C, Block, AccountId, Balance, BlockNumber>
    StakingApiServer<<Block as BlockT>::Hash, AccountId, Balance, BlockNumber>
    for Staking<C, Block, AccountId, Balance, BlockNumber>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: StakingRuntimeApi<Block, AccountId, Balance, BlockNumber>,
    AccountId: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
    Balance: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
    BlockNumber: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn validators(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<ValidatorInfo<AccountId, Balance, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.validators(at).map_err(runtime_error("validators"))
    }

    fn validator_count(&self, at: Option<Block::Hash>) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.validator_count(at)
            .map_err(runtime_error("validator count"))
    }

    fn current_epoch(&self, at: Option<Block::Hash>) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.current_epoch(at)
            .map_err(runtime_error("current epoch"))
    }

    fn last_reward_distribution(&self, at: Option<Block::Hash>) -> RpcResult<Option<BlockNumber>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.last_reward_distribution(at)
            .map_err(runtime_error("last reward distribution"))
    }
}

fn runtime_error(what: &'static str) -> impl Fn(sp_api::ApiError) -> ErrorObjectOwned {
    move |error| {
        ErrorObject::owned(
            RUNTIME_ERROR,
            format!("Unable to read {what}."),
            Some(error.to_string()),
        )
    }
}
