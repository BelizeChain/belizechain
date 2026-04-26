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
//! - `belizechain_getChainInfo` — Summary of chain name, version, and feature flags

#![warn(missing_docs)]

use std::sync::Arc;

use belizechain_runtime::{opaque::Block, AccountId, Balance, Nonce};
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

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
        + BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool } = deps;

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client).into_rpc())?;

    // ── Custom BelizeChain RPC methods (§3.3) ────────────────────────────
    //
    // These lightweight methods expose chain metadata without requiring a
    // full runtime-API crate. For typed pallet queries (e.g., governance
    // proposals, staking validators, bridge transactions), create dedicated
    // `<pallet>-rpc` / `<pallet>-rpc-runtime-api` crates and register them
    // here following the Substrate custom-RPC pattern.

    module.register_method("belizechain_getChainInfo", |_, _, _| {
        serde_json::json!({
            "chain": "BelizeChain",
            "currency": "DALLA",
            "decimals": 12,
            "ss58_prefix": 1981,
            "block_time_secs": 6,
            "pallets": [
                "economy", "identity", "governance", "staking", "consensus",
                "oracle", "interoperability", "belizex", "bns", "landledger",
                "payroll", "community", "quantum", "mesh", "compliance",
                "justice", "whistleblower", "moderation"
            ],
            "features": [
                "federated_ai_consensus",
                "post_quantum_bridge",
                "gem_smart_contracts",
                "meshtastic_integration"
            ]
        })
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
