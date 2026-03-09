// Chain specification configurations for different networks
// Enables easy switching between testnet/mainnet/devnet

use serde::{Deserialize, Serialize};
use belizechain_runtime::{Balance, DOLLARS as DALLA};

/// Network configuration presets (reserved for deployment automation)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub id: String,
    pub initial_authorities: Vec<(String, String)>, // (babe_seed, grandpa_seed)
    pub initial_allocation: Vec<(String, Balance)>, // (account_seed, balance)
    pub enable_sudo: bool,
    pub enable_faucet: bool,
    pub block_time_ms: u64,
}

#[allow(dead_code)]
impl NetworkConfig {
    /// Development network configuration
    pub fn development() -> Self {
        Self {
            name: "BelizeChain Development".to_string(),
            id: "belizechain_dev".to_string(),
            initial_authorities: vec![
                ("Alice".to_string(), "Alice".to_string()),
            ],
            initial_allocation: vec![
                ("Alice".to_string(), 1_000_000 * DALLA),
                ("Bob".to_string(), 1_000_000 * DALLA),
                ("Charlie".to_string(), 500_000 * DALLA),
            ],
            enable_sudo: true,
            enable_faucet: true,
            block_time_ms: 3000, // 3 second blocks for fast dev
        }
    }

    /// Local testnet configuration (multi-node local testing)
    pub fn local_testnet() -> Self {
        Self {
            name: "BelizeChain Local Testnet".to_string(),
            id: "belizechain_local_testnet".to_string(),
            initial_authorities: vec![
                ("Alice".to_string(), "Alice".to_string()),
                ("Bob".to_string(), "Bob".to_string()),
            ],
            initial_allocation: vec![
                ("Alice".to_string(), 10_000_000 * DALLA),
                ("Bob".to_string(), 10_000_000 * DALLA),
                ("Charlie".to_string(), 5_000_000 * DALLA),
                ("Dave".to_string(), 5_000_000 * DALLA),
                ("Eve".to_string(), 1_000_000 * DALLA),
                ("Ferdie".to_string(), 1_000_000 * DALLA),
            ],
            enable_sudo: true,
            enable_faucet: true,
            block_time_ms: 6000, // 6 second blocks
        }
    }

    /// Public testnet configuration (alpha/beta testing)
    pub fn public_testnet() -> Self {
        Self {
            name: "BelizeChain Public Testnet".to_string(),
            id: "belizechain_testnet".to_string(),
            initial_authorities: vec![
                // Production validator seeds (replace with real keys in deployment)
                ("validator1".to_string(), "validator1".to_string()),
                ("validator2".to_string(), "validator2".to_string()),
                ("validator3".to_string(), "validator3".to_string()),
            ],
            initial_allocation: vec![
                // Treasury allocation
                ("treasury".to_string(), 100_000_000 * DALLA),
                // Validator rewards pool
                ("rewards_pool".to_string(), 50_000_000 * DALLA),
                // Development fund
                ("dev_fund".to_string(), 25_000_000 * DALLA),
            ],
            enable_sudo: true, // Keep sudo for emergency upgrades during testnet
            enable_faucet: true,
            block_time_ms: 6000,
        }
    }

    /// Mainnet configuration (production)
    pub fn mainnet() -> Self {
        Self {
            name: "BelizeChain".to_string(),
            id: "belizechain".to_string(),
            initial_authorities: vec![
                // MUST be replaced with actual validator keys before mainnet launch
                // These are placeholder seeds - DO NOT use in production
                ("mainnet_validator_1".to_string(), "mainnet_validator_1".to_string()),
                ("mainnet_validator_2".to_string(), "mainnet_validator_2".to_string()),
                ("mainnet_validator_3".to_string(), "mainnet_validator_3".to_string()),
                ("mainnet_validator_4".to_string(), "mainnet_validator_4".to_string()),
                ("mainnet_validator_5".to_string(), "mainnet_validator_5".to_string()),
            ],
            initial_allocation: vec![
                // Central allocation (managed by multi-sig treasury)
                ("central_treasury".to_string(), 500_000_000 * DALLA),
                // Staking rewards pool (10-year emission schedule)
                ("staking_rewards".to_string(), 300_000_000 * DALLA),
                // Development & operations fund
                ("development_fund".to_string(), 100_000_000 * DALLA),
                // Public distribution reserve
                ("public_distribution".to_string(), 100_000_000 * DALLA),
            ],
            enable_sudo: false, // NO sudo on mainnet - governance only
            enable_faucet: false,
            block_time_ms: 6000, // 6 second blocks (10 blocks/minute)
        }
    }

    /// Staging network (pre-mainnet rehearsal)
    pub fn staging() -> Self {
        Self {
            name: "BelizeChain Staging".to_string(),
            id: "belizechain_staging".to_string(),
            initial_authorities: vec![
                ("staging_val_1".to_string(), "staging_val_1".to_string()),
                ("staging_val_2".to_string(), "staging_val_2".to_string()),
                ("staging_val_3".to_string(), "staging_val_3".to_string()),
            ],
            initial_allocation: vec![
                ("treasury".to_string(), 1_000_000_000 * DALLA),
            ],
            enable_sudo: false, // Test without sudo to match mainnet
            enable_faucet: false,
            block_time_ms: 6000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_configs() {
        let dev = NetworkConfig::development();
        assert!(dev.enable_sudo);
        assert!(dev.enable_faucet);

        let mainnet = NetworkConfig::mainnet();
        assert!(!mainnet.enable_sudo);
        assert!(!mainnet.enable_faucet);
        assert!(mainnet.initial_authorities.len() >= 5);
    }

    // ── NetworkConfig presets ──────────────────────────────────────────────

    #[test]
    fn test_local_testnet_config() {
        let config = NetworkConfig::local_testnet();
        assert_eq!(config.id, "belizechain_local_testnet");
        assert!(config.enable_sudo);
        assert!(config.enable_faucet);
        assert_eq!(config.initial_authorities.len(), 2);
        assert_eq!(config.block_time_ms, 6000);
    }

    #[test]
    fn test_public_testnet_config() {
        let config = NetworkConfig::public_testnet();
        assert_eq!(config.id, "belizechain_testnet");
        assert!(config.enable_sudo, "public testnet retains sudo for emergency upgrades");
        assert!(config.enable_faucet);
        assert_eq!(config.initial_authorities.len(), 3);
        assert!(!config.initial_allocation.is_empty());
    }

    #[test]
    fn test_staging_config() {
        let config = NetworkConfig::staging();
        assert_eq!(config.id, "belizechain_staging");
        assert!(!config.enable_sudo, "staging must match mainnet: no sudo");
        assert!(!config.enable_faucet, "staging must match mainnet: no faucet");
        assert_eq!(config.initial_authorities.len(), 3);
        assert_eq!(config.block_time_ms, 6000);
    }

    // ── Block time values ──────────────────────────────────────────────────

    #[test]
    fn test_development_block_time_is_faster() {
        let dev = NetworkConfig::development();
        let mainnet = NetworkConfig::mainnet();
        assert_eq!(dev.block_time_ms, 3000);
        assert!(
            dev.block_time_ms < mainnet.block_time_ms,
            "dev blocks should be faster than mainnet for ergonomic testing"
        );
    }

    #[test]
    fn test_production_configs_use_six_second_blocks() {
        for config in [
            NetworkConfig::mainnet(),
            NetworkConfig::staging(),
            NetworkConfig::local_testnet(),
            NetworkConfig::public_testnet(),
        ] {
            assert_eq!(
                config.block_time_ms, 6000,
                "config '{}' must use 6 s blocks", config.id
            );
        }
    }

    // ── Initial allocation invariants ──────────────────────────────────────

    #[test]
    fn test_initial_allocation_all_nonzero() {
        for config in [
            NetworkConfig::development(),
            NetworkConfig::local_testnet(),
            NetworkConfig::public_testnet(),
            NetworkConfig::mainnet(),
        ] {
            assert!(
                !config.initial_allocation.is_empty(),
                "config '{}' must have initial_allocation entries", config.id
            );
            for (seed, balance) in &config.initial_allocation {
                assert!(
                    *balance > 0,
                    "allocation for '{}' in config '{}' must be > 0", seed, config.id
                );
            }
        }
    }

    #[test]
    fn test_mainnet_no_faucet() {
        let config = NetworkConfig::mainnet();
        assert!(!config.enable_faucet, "mainnet must not have a faucet");
    }

    // ── Network ID uniqueness ──────────────────────────────────────────────

    #[test]
    fn test_all_network_ids_are_unique() {
        let configs = [
            NetworkConfig::development(),
            NetworkConfig::local_testnet(),
            NetworkConfig::public_testnet(),
            NetworkConfig::mainnet(),
            NetworkConfig::staging(),
        ];
        let mut seen = std::collections::HashSet::new();
        for cfg in &configs {
            assert!(
                seen.insert(cfg.id.clone()),
                "Duplicate network ID detected: {}", cfg.id
            );
        }
    }

    // ── Authority seed tuple validity ──────────────────────────────────────

    #[test]
    fn test_authority_seeds_are_nonempty_strings() {
        for config in [
            NetworkConfig::development(),
            NetworkConfig::local_testnet(),
            NetworkConfig::mainnet(),
        ] {
            for (babe_seed, grandpa_seed) in &config.initial_authorities {
                assert!(
                    !babe_seed.is_empty(),
                    "babe seed must not be empty in config '{}'", config.id
                );
                assert!(
                    !grandpa_seed.is_empty(),
                    "grandpa seed must not be empty in config '{}'", config.id
                );
            }
        }
    }

    #[test]
    fn test_authority_count_minimum() {
        // All production/staging networks need at least 3 validators
        for config in [
            NetworkConfig::mainnet(),
            NetworkConfig::staging(),
            NetworkConfig::public_testnet(),
        ] {
            assert!(
                config.initial_authorities.len() >= 3,
                "config '{}' must have at least 3 initial authorities for BFT fault tolerance",
                config.id
            );
        }
    }

    // ── Serialization round-trip ───────────────────────────────────────────

    #[test]
    fn test_network_config_serializes_and_deserializes() {
        let original = NetworkConfig::local_testnet();
        let json = serde_json::to_string(&original)
            .expect("NetworkConfig must be serializable to JSON");
        let restored: NetworkConfig = serde_json::from_str(&json)
            .expect("NetworkConfig must be deserializable from JSON");
        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.enable_sudo, restored.enable_sudo);
        assert_eq!(original.block_time_ms, restored.block_time_ms);
        assert_eq!(original.initial_authorities.len(), restored.initial_authorities.len());
    }
}
