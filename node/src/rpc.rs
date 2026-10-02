//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.
//!
//! ## Custom BelizeChain RPCs (§3.3)
//!
//! In addition to the standard System and TransactionPayment RPCs, we expose
//! chain-aware query methods for key pallets. For full custom typed RPCs,
//! create a `<pallet>-rpc` crate with a `decl_runtime_apis!` macro and
//! corresponding server implementation (see Substrate custom RPC guide).
//!
//! Currently available custom methods:
//! - `belizechain_getChainInfo` — node-observed chain/runtime metadata: spec
//!   name and versions, genesis/best/finalized pointers, and the runtime API
//!   surface actually compiled into the running runtime.

#![warn(missing_docs)]

use std::sync::Arc;

use belizechain_runtime::{opaque::Block, AccountId, Balance, Nonce};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Core, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::SaturatedConversion;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
        + pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
        + sp_api::Core<Block>
        + BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool } = deps;

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    let chain_info_client = client.clone();
    module.merge(TransactionPayment::new(client).into_rpc())?;

    // ── Custom BelizeChain RPC methods (§3.3) ────────────────────────────
    //
    // Reports only what the running node can observe. Static chain constants
    // (currency, decimals, SS58 prefix, block time) live in the chain spec and
    // runtime; duplicating them here would let the RPC drift from the chain.
    //
    // For typed pallet queries (e.g., governance proposals, staking
    // validators, bridge transactions), create dedicated `<pallet>-rpc` /
    // `<pallet>-rpc-runtime-api` crates and register them here following the
    // Substrate custom-RPC pattern.

    module.register_method("belizechain_getChainInfo", move |_, _, _| {
        let info = chain_info_client.info();
        let version = chain_info_client
            .runtime_api()
            .version(info.best_hash)
            .map_err(|err| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("runtime version unavailable at best block: {err}"),
                    None::<()>,
                )
            })?;

        // Runtime API ids are 8-byte blake2b-64 hashes of the trait name
        // (see sp-api's decl_runtime_apis), so they are rendered as hex — the
        // original names are not recoverable from the bytes.
        let runtime_apis: Vec<String> = version
            .apis
            .iter()
            .map(|(id, ver)| {
                let hex: String = id.iter().map(|byte| format!("{byte:02x}")).collect();
                format!("0x{hex}@{ver}")
            })
            .collect();

        Ok::<_, ErrorObjectOwned>(serde_json::json!({
            "specName": version.spec_name.to_string(),
            "implName": version.impl_name.to_string(),
            "specVersion": version.spec_version,
            "implVersion": version.impl_version,
            "authoringVersion": version.authoring_version,
            "transactionVersion": version.transaction_version,
            "stateVersion": format!("{:?}", version.state_version()),
            "genesisHash": info.genesis_hash.to_string(),
            "bestHash": info.best_hash.to_string(),
            "bestNumber": info.best_number.saturated_into::<u64>(),
            "finalizedHash": info.finalized_hash.to_string(),
            "finalizedNumber": info.finalized_number.saturated_into::<u64>(),
            "runtimeApis": runtime_apis,
        }))
    })?;

    // TODO(§3.3): Add typed custom RPCs for key pallets:
    // - belizechain_governanceProposals  (requires GovernanceApi runtime API)
    // - belizechain_stakingValidators    (requires StakingApi runtime API)
    // - belizechain_bridgeTransactions   (requires BridgeApi runtime API)
    // - belizechain_oracleFeeds          (requires OracleApi runtime API)
    //
    // Each requires:
    //   1. `decl_runtime_apis!` in a `<pallet>-rpc-runtime-api` crate
    //   2. `impl_runtime_apis!` in runtime/src/lib.rs
    //   3. Server struct in a `<pallet>-rpc` crate
    //   4. Registration via `module.merge(...)` here

    Ok(module)
}
