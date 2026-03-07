//! Production-grade validator and collator node configuration
//! 
//! Provides network presets for different deployment environments:
//! - Development: Permissive settings with mDNS discovery
//! - Testnet: Moderate peer limits with telemetry
//! - Mainnet: Strict production settings with high availability
//! - Bootnode: High connection limits for network bootstrap nodes
//!
//! **Status**: Reserved for production deployment automation.
//! These utilities will be activated during mainnet launch.

#![allow(dead_code)]

use sc_network::config::{NodeKeyConfig, SetConfig, TransportConfig};
use sc_service::config::{KeystoreConfig, PrometheusConfig};
use std::path::Path;

/// Validator node configuration presets (reserved for deployment automation)
#[allow(dead_code)]
pub struct ValidatorConfig {
    /// Minimum number of peers to maintain
    pub min_peers: u32,
    /// Maximum number of peers
    pub max_peers: u32,
    /// Enable mDNS peer discovery (disable in production)
    pub enable_mdns: bool,
    /// Enable Prometheus metrics endpoint
    pub prometheus_port: Option<u16>,
    /// Telemetry endpoints
    pub telemetry_endpoints: Vec<String>,
    /// Enable transaction pool gossip
    pub transaction_gossip: bool,
}

impl ValidatorConfig {
    /// Development node configuration (permissive, local discovery)
    pub fn development() -> Self {
        Self {
            min_peers: 0,
            max_peers: 25,
            enable_mdns: true,
            prometheus_port: Some(9615),
            telemetry_endpoints: vec![],
            transaction_gossip: true,
        }
    }

    /// Testnet validator configuration
    pub fn testnet() -> Self {
        Self {
            min_peers: 5,
            max_peers: 50,
            enable_mdns: false,
            prometheus_port: Some(9615),
            telemetry_endpoints: vec![
                "wss://telemetry.belizechain.org:443/submit/".to_string(),
            ],
            transaction_gossip: true,
        }
    }

    /// Mainnet validator configuration (strict, no local discovery)
    pub fn mainnet() -> Self {
        Self {
            min_peers: 10,
            max_peers: 100,
            enable_mdns: false, // Never use mDNS in production
            prometheus_port: Some(9615),
            telemetry_endpoints: vec![
                "wss://telemetry.belizechain.org:443/submit/".to_string(),
            ],
            transaction_gossip: true,
        }
    }

    /// Bootnode configuration (high connection limit)
    pub fn bootnode() -> Self {
        Self {
            min_peers: 0,
            max_peers: 500, // Bootnodes handle many connections
            enable_mdns: false,
            prometheus_port: Some(9615),
            telemetry_endpoints: vec![],
            transaction_gossip: true,
        }
    }
}

/// Network bootstrap nodes for peer discovery (reserved for mainnet launch)
#[allow(dead_code)]
pub struct BootstrapNodes;

impl BootstrapNodes {
    /// Mainnet bootnodes (replace with real addresses before launch)
    pub fn mainnet() -> Vec<String> {
        vec![
            // Belize City node
            "/dns4/bootnode1.belizechain.org/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".to_string(),
            // San Pedro node
            "/dns4/bootnode2.belizechain.org/tcp/30333/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD".to_string(),
            // Belmopan node (Central Bank datacenter)
            "/dns4/bootnode3.belizechain.org/tcp/30333/p2p/12D3KooWLmrYDLoNTyTYtRdDyZLWDe1paxzxTw5RgjmHLfzW96SX".to_string(),
        ]
    }

    /// Testnet bootnodes
    pub fn testnet() -> Vec<String> {
        vec![
            "/dns4/testnet-bootnode1.belizechain.org/tcp/30333/p2p/12D3KooWTestNode1".to_string(),
            "/dns4/testnet-bootnode2.belizechain.org/tcp/30333/p2p/12D3KooWTestNode2".to_string(),
        ]
    }

    /// Local testnet bootnodes (localhost for multi-node testing)
    pub fn local() -> Vec<String> {
        vec![
            "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".to_string(),
        ]
    }
}

/// Recommended node key storage configuration
pub fn node_key_config(_base_path: &Path, _network: &str) -> NodeKeyConfig {
    // Substrate NodeKeyConfig no longer has File variant in stable2512
    // Node keys are now managed through CLI --node-key or --node-key-file
    NodeKeyConfig::default()
}

/// Prometheus metrics configuration for monitoring
pub fn prometheus_config(port: u16) -> PrometheusConfig {
    PrometheusConfig::new_with_default_registry(
        format!("127.0.0.1:{}", port).parse().unwrap(),
        "belizechain".to_string(),
    )
}

/// Keystore configuration for validator keys
pub fn keystore_config(base_path: &Path, network: &str) -> KeystoreConfig {
    let keystore_path = base_path.join("keystore").join(network);
    KeystoreConfig::Path { path: keystore_path, password: None }
}

/// P2P network transport configuration
pub fn transport_config() -> TransportConfig {
    TransportConfig::Normal {
        enable_mdns: false, // Disable mDNS in production
        allow_private_ip: false, // Only public IPs in production
    }
}

/// Recommended reserved peer slots for critical nodes
pub fn reserved_peers_config() -> SetConfig {
    SetConfig {
        in_peers: 25,  // Inbound connections
        out_peers: 75, // Outbound connections (prioritize outbound)
        reserved_nodes: vec![], // Add reserved nodes here
        non_reserved_mode: sc_network::config::NonReservedPeerMode::Accept,
    }
}

/// Database backend configuration recommendations
pub mod database {
    use sc_service::config::DatabaseSource;
    use std::path::Path;

    /// Recommended database for validators (RocksDB with tuned settings)
    pub fn validator_database(base_path: &Path) -> DatabaseSource {
        DatabaseSource::RocksDb {
            path: base_path.join("db"),
            cache_size: 512, // 512 MB cache for validators
        }
    }

    /// Recommended database for archive nodes (larger cache)
    pub fn archive_database(base_path: &Path) -> DatabaseSource {
        DatabaseSource::RocksDb {
            path: base_path.join("db"),
            cache_size: 2048, // 2 GB cache for archive nodes
        }
    }

    /// Recommended database for light clients (minimal resources)
    pub fn light_database(base_path: &Path) -> DatabaseSource {
        DatabaseSource::RocksDb {
            path: base_path.join("db"),
            cache_size: 128, // 128 MB cache for light clients
        }
    }
}

/// Role-specific configuration recommendations
pub mod node_roles {
    use super::*;

    /// Full validator node (participates in consensus)
    pub fn validator() -> ValidatorConfig {
        ValidatorConfig::mainnet()
    }

    /// Archive node (stores full history)
    pub fn archive() -> ValidatorConfig {
        let mut config = ValidatorConfig::mainnet();
        config.max_peers = 200; // Archives can handle more peers
        config
    }

    /// RPC node (serves dApp requests)
    pub fn rpc() -> ValidatorConfig {
        let mut config = ValidatorConfig::mainnet();
        config.max_peers = 150;
        config.transaction_gossip = true;
        config
    }

    /// Bootnode (helps new nodes discover peers)
    pub fn bootnode() -> ValidatorConfig {
        ValidatorConfig::bootnode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_network::config::{SetConfig, TransportConfig};
    use sc_service::config::{DatabaseSource, KeystoreConfig};

    #[test]
    fn test_validator_configs() {
        let dev = ValidatorConfig::development();
        assert!(dev.enable_mdns);

        let mainnet = ValidatorConfig::mainnet();
        assert!(!mainnet.enable_mdns);
        assert!(mainnet.min_peers >= 10);
    }

    #[test]
    fn test_bootstrap_nodes() {
        let mainnet_nodes = BootstrapNodes::mainnet();
        assert!(mainnet_nodes.len() >= 3);

        let testnet_nodes = BootstrapNodes::testnet();
        assert!(testnet_nodes.len() >= 2);
    }

    // ── ValidatorConfig presets ────────────────────────────────────────────

    #[test]
    fn test_testnet_validator_config() {
        let config = ValidatorConfig::testnet();
        assert!(!config.enable_mdns, "testnet must not use mDNS");
        assert_eq!(config.min_peers, 5);
        assert_eq!(config.max_peers, 50);
        assert_eq!(config.prometheus_port, Some(9615));
        assert!(!config.telemetry_endpoints.is_empty(), "testnet must expose telemetry");
        assert!(config.transaction_gossip);
    }

    #[test]
    fn test_bootnode_validator_config() {
        let config = ValidatorConfig::bootnode();
        assert!(!config.enable_mdns);
        assert_eq!(config.min_peers, 0);
        assert_eq!(config.max_peers, 500);
        assert_eq!(config.prometheus_port, Some(9615));
        assert!(config.transaction_gossip);
    }

    #[test]
    fn test_development_config_full() {
        let config = ValidatorConfig::development();
        assert_eq!(config.max_peers, 25);
        assert_eq!(config.min_peers, 0);
        assert_eq!(config.prometheus_port, Some(9615));
        assert!(config.telemetry_endpoints.is_empty(), "dev should have no telemetry");
        assert!(config.transaction_gossip);
    }

    #[test]
    fn test_mainnet_config_full() {
        let config = ValidatorConfig::mainnet();
        assert_eq!(config.max_peers, 100);
        assert_eq!(config.min_peers, 10);
        assert_eq!(config.prometheus_port, Some(9615));
        assert!(!config.telemetry_endpoints.is_empty(), "mainnet must expose telemetry");
        assert!(config.transaction_gossip);
    }

    // ── Peer limits ordering ───────────────────────────────────────────────

    #[test]
    fn test_mainnet_peer_limits_exceed_testnet() {
        let testnet = ValidatorConfig::testnet();
        let mainnet = ValidatorConfig::mainnet();
        assert!(
            mainnet.min_peers > testnet.min_peers,
            "mainnet must require more peers than testnet"
        );
        assert!(mainnet.max_peers > testnet.max_peers);
    }

    #[test]
    fn test_bootnode_max_peers_exceeds_all_others() {
        let bootnode = ValidatorConfig::bootnode();
        for config in [
            ValidatorConfig::development(),
            ValidatorConfig::testnet(),
            ValidatorConfig::mainnet(),
        ] {
            assert!(
                bootnode.max_peers > config.max_peers,
                "bootnode must accept more peers than any other role"
            );
        }
    }

    // ── BootstrapNodes ─────────────────────────────────────────────────────

    #[test]
    fn test_bootstrap_nodes_local() {
        let nodes = BootstrapNodes::local();
        assert!(!nodes.is_empty());
        assert!(
            nodes[0].starts_with("/ip4/127.0.0.1"),
            "local bootnode must be a loopback address"
        );
    }

    #[test]
    fn test_bootstrap_nodes_multiaddress_format() {
        for node in BootstrapNodes::mainnet() {
            assert!(node.starts_with("/dns4/"), "mainnet node must use /dns4/: {node}");
            assert!(
                node.contains("/tcp/30333/p2p/"),
                "mainnet node must include /tcp/30333/p2p/: {node}"
            );
        }
        for node in BootstrapNodes::testnet() {
            assert!(node.starts_with("/dns4/"), "testnet node must use /dns4/: {node}");
            assert!(
                node.contains("/tcp/30333/p2p/"),
                "testnet node must include /tcp/30333/p2p/: {node}"
            );
        }
        for node in BootstrapNodes::local() {
            assert!(node.contains("/p2p/"), "local node must include /p2p/ peer ID: {node}");
        }
    }

    // ── node_roles module ──────────────────────────────────────────────────

    #[test]
    fn test_node_roles_validator_matches_mainnet_config() {
        let role = node_roles::validator();
        let mainnet = ValidatorConfig::mainnet();
        assert_eq!(role.min_peers, mainnet.min_peers);
        assert_eq!(role.max_peers, mainnet.max_peers);
        assert!(!role.enable_mdns);
    }

    #[test]
    fn test_node_roles_archive_extended_peers() {
        let role = node_roles::archive();
        let mainnet = ValidatorConfig::mainnet();
        assert_eq!(role.max_peers, 200);
        assert!(role.max_peers > mainnet.max_peers, "archive nodes need more peer slots");
        assert!(!role.enable_mdns);
    }

    #[test]
    fn test_node_roles_rpc() {
        let role = node_roles::rpc();
        assert_eq!(role.max_peers, 150);
        assert!(role.transaction_gossip);
        assert!(!role.enable_mdns);
    }

    #[test]
    fn test_node_roles_bootnode_config() {
        let role = node_roles::bootnode();
        assert_eq!(role.max_peers, 500);
        assert_eq!(role.min_peers, 0);
        assert!(!role.enable_mdns);
    }

    // ── Utility functions ──────────────────────────────────────────────────

    #[test]
    fn test_node_key_config_returns_without_panic() {
        let base = std::path::Path::new("/tmp");
        let _cfg = node_key_config(base, "dev");
    }

    #[test]
    fn test_prometheus_config_builds_without_panic() {
        let _cfg = prometheus_config(9615);
        let _cfg2 = prometheus_config(9616);
    }

    #[test]
    fn test_keystore_config_path_contains_network() {
        let base = std::path::PathBuf::from("/tmp/belizechain_ks_test");
        let config = keystore_config(&base, "dev");
        match config {
            KeystoreConfig::Path { path, password } => {
                assert!(
                    path.to_string_lossy().contains("keystore"),
                    "keystore path must contain 'keystore' segment"
                );
                assert!(
                    path.to_string_lossy().contains("dev"),
                    "keystore path must contain the network name segment"
                );
                assert!(password.is_none(), "default keystore must have no password");
            }
            _ => panic!("Expected KeystoreConfig::Path variant"),
        }
    }

    #[test]
    fn test_keystore_config_different_networks_produce_different_paths() {
        let base = std::path::PathBuf::from("/tmp/belizechain_ks_test");
        let dev_ks = keystore_config(&base, "dev");
        let mainnet_ks = keystore_config(&base, "mainnet");
        match (dev_ks, mainnet_ks) {
            (
                KeystoreConfig::Path { path: p1, .. },
                KeystoreConfig::Path { path: p2, .. },
            ) => {
                assert_ne!(p1, p2, "different networks must produce different keystore paths");
            }
            _ => panic!("Expected KeystoreConfig::Path variants"),
        }
    }

    #[test]
    fn test_transport_config_disables_mdns_and_private_ips() {
        match transport_config() {
            TransportConfig::Normal { enable_mdns, allow_private_ip } => {
                assert!(!enable_mdns, "production transport must disable mDNS");
                assert!(!allow_private_ip, "production transport must disallow private IPs");
            }
            _ => panic!("Unexpected TransportConfig variant"),
        }
    }

    #[test]
    fn test_reserved_peers_config_out_exceeds_in() {
        let config = reserved_peers_config();
        assert_eq!(config.in_peers, 25);
        assert_eq!(config.out_peers, 75);
        assert!(config.reserved_nodes.is_empty());
        assert!(
            config.out_peers > config.in_peers,
            "outbound slots should exceed inbound slots"
        );
    }

    // ── database module ────────────────────────────────────────────────────

    #[test]
    fn test_database_validator_cache_size() {
        let base = std::path::PathBuf::from("/tmp/belizechain_db_test");
        match database::validator_database(&base) {
            DatabaseSource::RocksDb { cache_size, path } => {
                assert_eq!(cache_size, 512);
                assert!(path.to_string_lossy().ends_with("db"));
            }
            _ => panic!("Expected DatabaseSource::RocksDb for validator"),
        }
    }

    #[test]
    fn test_database_archive_cache_size() {
        let base = std::path::PathBuf::from("/tmp/belizechain_db_test");
        match database::archive_database(&base) {
            DatabaseSource::RocksDb { cache_size, path } => {
                assert_eq!(cache_size, 2048);
                assert!(path.to_string_lossy().ends_with("db"));
            }
            _ => panic!("Expected DatabaseSource::RocksDb for archive"),
        }
    }

    #[test]
    fn test_database_light_cache_size() {
        let base = std::path::PathBuf::from("/tmp/belizechain_db_test");
        match database::light_database(&base) {
            DatabaseSource::RocksDb { cache_size, path } => {
                assert_eq!(cache_size, 128);
                assert!(path.to_string_lossy().ends_with("db"));
            }
            _ => panic!("Expected DatabaseSource::RocksDb for light client"),
        }
    }

    #[test]
    fn test_database_cache_size_ordering() {
        let base = std::path::PathBuf::from("/tmp/belizechain_db_test");
        let light = match database::light_database(&base) {
            DatabaseSource::RocksDb { cache_size, .. } => cache_size,
            _ => panic!(),
        };
        let validator = match database::validator_database(&base) {
            DatabaseSource::RocksDb { cache_size, .. } => cache_size,
            _ => panic!(),
        };
        let archive = match database::archive_database(&base) {
            DatabaseSource::RocksDb { cache_size, .. } => cache_size,
            _ => panic!(),
        };
        assert!(light < validator, "light cache must be smaller than validator");
        assert!(validator < archive, "validator cache must be smaller than archive");
    }

    #[test]
    fn test_all_database_configs_use_rocksdb() {
        let base = std::path::PathBuf::from("/tmp/belizechain_db_test");
        let configs = [
            database::light_database(&base),
            database::validator_database(&base),
            database::archive_database(&base),
        ];
        for cfg in configs {
            assert!(
                matches!(cfg, DatabaseSource::RocksDb { .. }),
                "all node types must use RocksDb"
            );
        }
    }
}
