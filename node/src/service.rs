#![allow(clippy::result_large_err, clippy::type_complexity)]
//! Service and ServiceFactory implementation for BelizeChain.
//!
//! ## Customization Roadmap (Audit §3.7)
//!
//! The current node implementation closely follows the Substrate node template.
//! The following customizations should be implemented before mainnet:
//!
//! - **Custom RPC middleware**: Rate limiting, authentication, CORS policies
//! - **Telemetry hooks**: BelizeChain-specific metrics (PoUW scores, mesh node count)
//! - **Transaction pool logic**: Priority ordering for governance/emergency extrinsics
//! - **Networking layer**: Optimized peer selection for Belizean network topology

use belizechain_runtime::{self, opaque::Block, RuntimeApi};
use futures::FutureExt;
use sc_client_api::{Backend, HeaderBackend};
use sc_consensus_babe::{self, SlotProportion};
pub use sc_executor::WasmExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_consensus_babe::inherents::BabeCreateInherentDataProviders;
use std::{sync::Arc, time::Duration};

pub(crate) type FullClient =
    sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<sp_io::SubstrateHostFunctions>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
// NOTE (CONS-035): LongestChain is the standard Substrate pattern for BABE+GRANDPA.
// Finality safety is provided by GrandpaBlockImport (rejects reorganisation past
// finalized blocks) and BackoffAuthoringOnFinalizedHeadLagging (CONS-034).
// FinalityTrackingSelectChain does not exist in polkadot-sdk; do NOT replace.

/// CONS-036 FIX: Reduced from 512 (~51 min) to 32 (~3.2 min) for faster
/// finality signals to light clients and bridges.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 32;

pub fn new_partial(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
        (
            sc_consensus_babe::BabeBlockImport<
                Block,
                FullClient,
                sc_consensus_grandpa::GrandpaBlockImport<
                    FullBackend,
                    Block,
                    FullClient,
                    FullSelectChain,
                >,
                BabeCreateInherentDataProviders<Block>,
                FullSelectChain,
            >,
            sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            sc_consensus_babe::BabeLink<Block>,
            sc_consensus_babe::BabeWorkerHandle<Block>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);

    let (client, backend, keystore, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            vec![Arc::new(sc_consensus_grandpa::GrandpaPruningFilter)],
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let babe_config = sc_consensus_babe::configuration(&*client)?;
    let slot_duration = babe_config.slot_duration();
    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        babe_config,
        grandpa_block_import.clone(),
        client.clone(),
        Arc::new(move |_, _| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot =
                sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );
            Ok((slot, timestamp))
        }) as BabeCreateInherentDataProviders<Block>,
        select_chain.clone(),
        OffchainTransactionPoolFactory::new(transaction_pool.clone()),
    )?;

    let slot_duration = babe_link.config().slot_duration();
    let (import_queue, babe_worker_handle) =
        sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
            link: babe_link.clone(),
            block_import: babe_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            slot_duration,
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        })?;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container: keystore,
        select_chain,
        transaction_pool,
        other: (
            babe_block_import,
            grandpa_link,
            babe_link,
            babe_worker_handle,
            telemetry,
        ),
    })
}

/// Builds a new service for a full client.
pub fn new_full<
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
    config: Configuration,
) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, babe_link, babe_worker_handle, mut telemetry),
    } = new_partial(&config)?;

    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());
    let metrics = sc_network::NotificationMetrics::new(config.prometheus_registry());
    let peer_store_handle = net_config.peer_store_handle();

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &backend.blockchain().info().genesis_hash,
        &config.chain_spec,
    );

    let (grandpa_protocol_config, grandpa_notification_service) =
        sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
            grandpa_protocol_name.clone(),
            metrics.clone(),
            peer_store_handle,
        );
    net_config.add_notification_protocol(grandpa_protocol_config);

    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(), // P2P-FIX-004: Warp sync hard forks (empty = no authority set hard forks)
                        // NOTE: For warp sync security with trusted checkpoints, use --warp-sync-checkpoint CLI flag
                        // See WARP_SYNC_CHECKPOINTS constant documentation above for checkpoint management strategy
    ));

    // P2P-FIX-002: Content-based block announce validation (defense-in-depth)
    // Peer identity filtering is handled by --reserved-only (P2P-FIX-003)

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            spawn_essential_handle: task_manager.spawn_essential_handle(),
            import_queue,
            block_announce_validator_builder: Some(Box::new(|_client| {
                Box::new(crate::block_announce_validator::BelizeBlockAnnounceValidator::new())
            })),
            warp_sync_config: Some(sc_service::WarpSyncConfig::WithProvider(warp_sync)),
            block_relay: None,
            metrics,
            net_config,
        })?;

    if config.offchain_worker.enabled {
        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                enable_http_requests: false, // P2P-FIX-005: Disabled - no pallets use OCW
                custom_extensions: |_| vec![],
            })?;
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            offchain_workers
                .run(client.clone(), task_manager.spawn_handle())
                .boxed(),
        );
    }

    let role = config.role;
    let force_authoring = config.force_authoring;
    // CONS-034 FIX: Enable BABE slot-skipping backoff when finality lags.
    // A value of 10 means the node will skip authoring after producing
    // 10 consecutive blocks without GRANDPA finality catching up.
    let backoff_authoring_blocks =
        Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |_| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    // Keep babe_worker_handle alive for the node lifetime.
    // Dropping it closes the channel to the essential babe-worker task, crashing the node.
    task_manager.keep_alive(babe_worker_handle);

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: telemetry.as_mut(),
        tracing_execute_block: None,
    })?;
    if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let slot_duration = babe_link.config().slot_duration();

        let babe = sc_consensus_babe::start_babe(sc_consensus_babe::BabeParams {
            keystore: keystore_container.keystore(),
            client: client.clone(),
            select_chain,
            block_import,
            env: proposer_factory,
            sync_oracle: sync_service.clone(),
            justification_sync_link: sync_service.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                Ok((slot, timestamp))
            },
            force_authoring,
            backoff_authoring_blocks,
            babe_link,
            block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        })?;

        // the BABE authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("babe", Some("block-authoring"), babe);
    }

    // if the node isn't running as a validator, run the grandpa observer instead.
    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = sc_consensus_grandpa::Config {
            // Standard GRANDPA gossip configuration (333ms gossip duration)
            gossip_duration: Duration::from_millis(333),
            // CONS-036 FIX: Reduced from 512 to 32 for faster finality signals.
            justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
            name: Some(name),
            observer_enabled: false,
            keystore: Some(keystore_container.keystore()),
            local_role: role,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            protocol_name: grandpa_protocol_name,
        };

        // start the full GRANDPA voter
        // NOTE: runs the embedded full voter instead of only running the observer.
        let grandpa_params = sc_consensus_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network,
            sync: sync_service.clone(),
            notification_service: grandpa_notification_service,
            voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry: prometheus_registry.clone(),
            shared_voter_state: sc_consensus_grandpa::SharedVoterState::empty(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
        };

        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            sc_consensus_grandpa::run_grandpa_voter(grandpa_params)?,
        );
    }

    Ok(task_manager)
}

#[cfg(feature = "runtime-benchmarks")]
/// Builds the PartialComponents for a benchmarking service.
///
/// Use this function if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_benchmark_partial(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
        (),
    >,
    sc_service::Error,
> {
    let sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: _,
    } = new_partial(config)?;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (),
    })
}

#[cfg(feature = "runtime-benchmarks")]
pub fn inherent_benchmark_data() -> Result<sp_inherents::InherentData, sc_service::Error> {
    let inherent_data = sp_inherents::InherentData::new();

    // InherentDataProvider now uses async provide_inherent_data
    // For benchmarking, we just return empty inherent data
    Ok(inherent_data)
}

#[cfg(feature = "runtime-benchmarks")]
pub struct RemarkBuilder {
    client: Arc<FullClient>,
}

#[cfg(feature = "runtime-benchmarks")]
impl RemarkBuilder {
    /// Create a new [`Self`] from the given client.
    pub fn new(client: Arc<FullClient>) -> Self {
        Self { client }
    }
}
//
// impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
//     fn pallet(&self) -> &str {
//         "system"
//     }
//
//     fn extrinsic(&self) -> &str {
//         "remark"
//     }
//
//     fn build(&self, nonce: u32) -> std::result::Result<belizechain_runtime::UncheckedExtrinsic, &'static str> {
//         let acc = AccountKeyring::Bob.pair();
//         let extrinsic: belizechain_runtime::UncheckedExtrinsic =
//             frame_system::Call::remark { remark: vec![] }.into();
//
//         Ok(extrinsic)
//     }
// }
//
// /// A [`ExtrinsicFactory`] which creates remark extrinsics.
// pub struct ExtrinsicFactory(pub std::collections::BTreeMap<&'static str, Box<dyn frame_benchmarking_cli::ExtrinsicBuilder>>);
//
// impl frame_benchmarking_cli::ExtrinsicFactory for ExtrinsicFactory {
//     fn try_get(&self, name: &str) -> Option<&dyn frame_benchmarking_cli::ExtrinsicBuilder> {
//         self.0.get(name).map(|builder| builder.as_ref())
//     }
// }
