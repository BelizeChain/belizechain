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
use std::path::PathBuf;

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
pub fn node_key_config(_base_path: &PathBuf, _network: &str) -> NodeKeyConfig {
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
pub fn keystore_config(base_path: &PathBuf, network: &str) -> KeystoreConfig {
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
    use std::path::PathBuf;

    /// Recommended database for validators (RocksDB with tuned settings)
    pub fn validator_database(base_path: &PathBuf) -> DatabaseSource {
        DatabaseSource::RocksDb {
            path: base_path.join("db"),
            cache_size: 512, // 512 MB cache for validators
        }
    }

    /// Recommended database for archive nodes (larger cache)
    pub fn archive_database(base_path: &PathBuf) -> DatabaseSource {
        DatabaseSource::RocksDb {
            path: base_path.join("db"),
            cache_size: 2048, // 2 GB cache for archive nodes
        }
    }

    /// Recommended database for light clients (minimal resources)
    pub fn light_database(base_path: &PathBuf) -> DatabaseSource {
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

    #[test]
    fn test_validator_configs() {
        let dev = ValidatorConfig::development();
        assert_eq!(dev.enable_mdns, true);

        let mainnet = ValidatorConfig::mainnet();
        assert_eq!(mainnet.enable_mdns, false);
        assert!(mainnet.min_peers >= 10);
    }

    #[test]
    fn test_bootstrap_nodes() {
        let mainnet_nodes = BootstrapNodes::mainnet();
        assert!(mainnet_nodes.len() >= 3);
        
        let testnet_nodes = BootstrapNodes::testnet();
        assert!(testnet_nodes.len() >= 2);
    }
}
