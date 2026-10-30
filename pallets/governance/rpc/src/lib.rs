//! JSON-RPC server for [`GovernanceApi`](pallet_belize_governance_rpc_runtime_api::GovernanceApi).
//!
//! Thin: every method reads the runtime API at `at` (or best block) and returns
//! it. There is no caching and no business logic here — anything that looked
//! like a rule would be a second copy of a rule the pallet already enforces.

use std::{marker::PhantomData, sync::Arc};

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};
use pallet_belize_governance::{BelizeDistrict, CouncilMember, Election};
use pallet_belize_governance_rpc_runtime_api::GovernanceApi as GovernanceRuntimeApi;
use serde::{de::DeserializeOwned, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait GovernanceApi<BlockHash, AccountId, BlockNumber>
where
    AccountId: Serialize + DeserializeOwned,
    BlockNumber: Serialize + DeserializeOwned,
{
    #[method(name = "belizechain_governanceCouncil")]
    fn council(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<CouncilMember<AccountId, BlockNumber>>>;

    #[method(name = "belizechain_governanceCouncilTerm")]
    fn council_term(&self, at: Option<BlockHash>) -> RpcResult<u32>;

    #[method(name = "belizechain_governanceElections")]
    fn district_elections(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(BelizeDistrict, Election<BlockNumber>)>>;

    #[method(name = "belizechain_governanceNextProposalId")]
    fn next_proposal_id(&self, at: Option<BlockHash>) -> RpcResult<u32>;
}

/// Error codes reported to clients. Distinct codes let a caller tell "the node
/// could not reach the runtime" from "the request was malformed" without
/// string-matching the message.
const RUNTIME_ERROR: i32 = 1;

pub struct Governance<C, Block, AccountId, BlockNumber> {
    client: Arc<C>,
    _marker: PhantomData<(Block, AccountId, BlockNumber)>,
}

impl<C, Block, AccountId, BlockNumber> Governance<C, Block, AccountId, BlockNumber> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<C, Block, AccountId, BlockNumber>
    GovernanceApiServer<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Governance<C, Block, AccountId, BlockNumber>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: GovernanceRuntimeApi<Block, AccountId, BlockNumber>,
    AccountId: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
    BlockNumber: codec::Codec + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn council(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<CouncilMember<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.council_members(at)
            .map_err(runtime_error("council members"))
    }

    fn council_term(&self, at: Option<Block::Hash>) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.council_term(at).map_err(runtime_error("council term"))
    }

    fn district_elections(
        &self,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(BelizeDistrict, Election<BlockNumber>)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.district_elections(at)
            .map_err(runtime_error("district elections"))
    }

    fn next_proposal_id(&self, at: Option<Block::Hash>) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.next_proposal_id(at)
            .map_err(runtime_error("next proposal id"))
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
