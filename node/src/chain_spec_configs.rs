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
    pub initial_authorities: Vec<(String, String)>, // (aura_seed, grandpa_seed)
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
}
