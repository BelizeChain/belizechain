#![allow(clippy::result_large_err)]
//! BelizeChain Node Implementation
//!
//! The sovereign blockchain node for Belize, implementing:
//! - Substrate-based architecture
//! - Custom consensus with federated AI integration
//! - Post-quantum cryptography
//! - Cross-chain interoperability

use belizechain_runtime::{self, opaque::Block};
use sc_cli::{ChainSpec, SubstrateCli};
use sc_service::PartialComponents;

mod block_announce_validator;
mod chain_spec;
mod chain_spec_configs;
mod validator_config;
#[macro_use]
mod service;
mod cli;
mod rpc;

use cli::{Cli, Subcommand};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "BelizeChain Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/BelizeChain/belizechain/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    /// Load chain specification from ID or file path
    ///
    /// Supported chain IDs:
    /// - "dev" - Development mode with Alice as validator
    /// - "local" - Local testnet with Alice and Bob
    /// - "testnet-template" - Public testnet template for `build-spec` only
    /// - "staging" - Pre-mainnet rehearsal template
    /// - "testnet" - intentionally disabled as a built-in alias; use an explicit JSON/raw spec file
    /// - "belize" or "" - Mainnet configuration
    /// - Custom path - Load from JSON file
    fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_config()?),
            "local" => Box::new(chain_spec::local_testnet_config()?),
            "testnet-template" => Box::new(chain_spec::public_testnet_config()?),
            "staging" => Box::new(chain_spec::staging_config()?),
            "testnet" => {
                return Err(
                    "Built-in chain ID `testnet` is disabled. Generate a dedicated testnet spec with `build-spec --disable-default-bootnode --chain testnet-template`, convert it to raw with the same flag, and launch nodes with the resulting JSON/raw spec file path.".into()
                )
            }
            "" | "belize" => Box::new(chain_spec::belizechain_mainnet_config()?),
            path => {
                Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?)
            }
        })
    }
}

/// Parse and run command line arguments
///
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config)?;
                let aux_revert = Box::new(|client, _, blocks| {
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd.as_ref())?;
            runner.sync_run(|config| match cmd.as_ref() {
                frame_benchmarking_cli::BenchmarkCmd::Pallet(cmd) => cmd
                    .run_with_spec::<sp_runtime::traits::HashingFor<
                    Block,
                >, sp_io::SubstrateHostFunctions>(
                    Some(config.chain_spec)
                ),
                frame_benchmarking_cli::BenchmarkCmd::Machine(cmd) => cmd.run(
                    &config,
                    frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.clone(),
                ),
                _ => Err("This benchmark sub-command is not supported yet.".into()),
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(_cmd)) => Err("try-runtime is not implemented".into()),
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                match config.network.network_backend {
                    sc_network::config::NetworkBackendType::Libp2p => service::new_full::<
                        sc_network::NetworkWorker<
                            belizechain_runtime::opaque::Block,
                            <belizechain_runtime::opaque::Block as sp_runtime::traits::Block>::Hash,
                        >,
                    >(config)
                    .map_err(sc_cli::Error::Service),
                    sc_network::config::NetworkBackendType::Litep2p => {
                        service::new_full::<sc_network::Litep2pNetworkBackend>(config)
                            .map_err(sc_cli::Error::Service)
                    }
                }
            })
        }
    }
}

fn main() -> sc_cli::Result<()> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_cli::SubstrateCli;

    // ── SubstrateCli static metadata ─────────────────────────────────────

    #[test]
    fn test_impl_name_is_belizechain_node() {
        assert_eq!(Cli::impl_name(), "BelizeChain Node");
    }

    #[test]
    fn test_impl_version_is_nonempty() {
        // Set by node/build.rs from SUBSTRATE_CLI_IMPL_VERSION / CARGO_PKG_VERSION
        let v = Cli::impl_version();
        assert!(!v.is_empty(), "impl_version must be set by build.rs");
    }

    #[test]
    fn test_description_matches_cargo_toml() {
        let d = Cli::description();
        assert_eq!(d, "BelizeChain sovereign blockchain node implementation");
    }

    #[test]
    fn test_author_matches_cargo_toml() {
        let a = Cli::author();
        assert!(
            a.contains("BelizeChain"),
            "author must contain 'BelizeChain': {a}"
        );
    }

    #[test]
    fn test_support_url_points_to_github_issues() {
        let url = Cli::support_url();
        assert!(
            url.contains("github.com"),
            "support URL must be on GitHub: {url}"
        );
        assert!(
            url.contains("BelizeChain"),
            "support URL must reference the org: {url}"
        );
        assert!(
            url.ends_with("/issues/new"),
            "support URL must point to /issues/new: {url}"
        );
    }

    #[test]
    fn test_copyright_start_year() {
        assert_eq!(Cli::copyright_start_year(), 2024);
    }

    // ── load_spec match arms ──────────────────────────────────────────────

    fn default_cli() -> Cli {
        <Cli as clap::Parser>::parse_from(["belizechain-node"])
    }

    #[test]
    fn test_load_spec_dev_succeeds_when_wasm_built() {
        // "dev" arm calls development_config() — succeeds when WASM_BINARY is compiled in.
        let cli = default_cli();
        let result = cli.load_spec("dev");
        assert!(
            result.is_ok(),
            "load_spec(\"dev\") must succeed in test build: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_load_spec_local_succeeds_when_wasm_built() {
        let cli = default_cli();
        let result = cli.load_spec("local");
        assert!(
            result.is_ok(),
            "load_spec(\"local\") must succeed in test build: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_load_spec_testnet_template_succeeds_when_wasm_built() {
        let cli = default_cli();
        let result = cli.load_spec("testnet-template");
        assert!(
            result.is_ok(),
            "load_spec(\"testnet-template\") must succeed in test build: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_load_spec_staging_succeeds_when_wasm_built() {
        let cli = default_cli();
        let result = cli.load_spec("staging");
        assert!(
            result.is_ok(),
            "load_spec(\"staging\") must succeed in test build: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_load_spec_testnet_fails_closed_until_external_spec_is_provided() {
        let cli = default_cli();
        let result = cli.load_spec("testnet");
        assert!(
            result.is_err(),
            "load_spec(\"testnet\") must fail closed until an external spec is provided"
        );
    }

    #[test]
    fn test_load_spec_empty_string_succeeds() {
        let cli = default_cli();
        let result = cli.load_spec("");
        assert!(
            result.is_ok(),
            "load_spec(\"\") must succeed when mainnet keys are configured"
        );
    }

    #[test]
    fn test_load_spec_belize_succeeds() {
        let cli = default_cli();
        let result = cli.load_spec("belize");
        assert!(
            result.is_ok(),
            "load_spec(\"belize\") must succeed when mainnet keys are configured"
        );
    }

    #[test]
    fn test_load_spec_nonexistent_path_fails() {
        // `path` arm — tries to load from a JSON file that doesn't exist.
        let cli = default_cli();
        let result = cli.load_spec("/nonexistent/path/to/chainspec.json");
        assert!(
            result.is_err(),
            "load_spec with nonexistent path must return Err"
        );
    }
}
