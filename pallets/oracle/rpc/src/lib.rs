//! JSON-RPC server for [`OracleApi`](pallet_belize_oracle_rpc_runtime_api::OracleApi).

use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};
use pallet_belize_oracle::types::{CurrencyPair, PriceFeedData};
use pallet_belize_oracle_rpc_runtime_api::OracleApi as OracleRuntimeApi;
use serde::{de::DeserializeOwned, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait OracleApi<BlockHash, BlockNumber>
where
    BlockNumber: Serialize + DeserializeOwned,
{
    #[method(name = "belizechain_oracleFeeds")]
    fn price_feeds(&self, at: Option<BlockHash>) -> RpcResult<Vec<PriceFeedData<BlockNumber>>>;

    #[method(name = "belizechain_oracleManualRates")]
    fn manual_exchange_rates(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CurrencyPair, u128, BlockNumber)>>;
}

const RUNTIME_ERROR: i32 = 1;

pub struct Oracle<C, Block, BlockNumber> {
    client: Arc<C>,
    _marker: PhantomData<(Block, BlockNumber)>,
}

impl<C, Block, BlockNumber> Oracle<C, Block, BlockNumber> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<C, Block, BlockNumber> OracleApiServer<<Block as BlockT>::Hash, BlockNumber>
    for Oracle<C, Block, BlockNumber>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: OracleRuntimeApi<Block, BlockNumber>,
    BlockNumber: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn price_feeds(&self, at: Option<Block::Hash>) -> RpcResult<Vec<PriceFeedData<BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.price_feeds(at).map_err(runtime_error("price feeds"))
    }

    fn manual_exchange_rates(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(CurrencyPair, u128, BlockNumber)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.manual_exchange_rates(at)
            .map_err(runtime_error("manual exchange rates"))
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
