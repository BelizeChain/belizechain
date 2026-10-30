//! JSON-RPC server for [`BridgeApi`](pallet_belize_interoperability_rpc_runtime_api::BridgeApi).

use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};
use pallet_belize_interoperability::{BridgeChain, BridgeTransaction, ChainConfig};
use pallet_belize_interoperability_rpc_runtime_api::BridgeApi as BridgeRuntimeApi;
use serde::{de::DeserializeOwned, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait BridgeApi<BlockHash, AccountId, BlockNumber>
where
    AccountId: Serialize + DeserializeOwned,
    BlockNumber: Serialize + DeserializeOwned,
{
    #[method(name = "belizechain_bridgeChains")]
    fn chain_configurations(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(BridgeChain, ChainConfig)>>;

    #[method(name = "belizechain_bridgeTransactions")]
    fn bridge_transactions(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<BridgeTransaction<AccountId, BlockNumber>>>;
}

const RUNTIME_ERROR: i32 = 1;

pub struct Bridge<C, Block, AccountId, BlockNumber> {
    client: Arc<C>,
    _marker: PhantomData<(Block, AccountId, BlockNumber)>,
}

impl<C, Block, AccountId, BlockNumber> Bridge<C, Block, AccountId, BlockNumber> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<C, Block, AccountId, BlockNumber>
    BridgeApiServer<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Bridge<C, Block, AccountId, BlockNumber>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: BridgeRuntimeApi<Block, AccountId, BlockNumber>,
    AccountId: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
    BlockNumber: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn chain_configurations(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(BridgeChain, ChainConfig)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.chain_configurations(at)
            .map_err(runtime_error("chain configurations"))
    }

    fn bridge_transactions(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<BridgeTransaction<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.bridge_transactions(at)
            .map_err(runtime_error("bridge transactions"))
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
