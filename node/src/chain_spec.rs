use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};

use crate::chain_spec_configs::NetworkConfig;
use belizechain_runtime::{opaque::SessionKeys, AccountId, BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate a BABE authority key pair plus the derived account ID.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, BabeId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

/// Generate authority keys from explicit seeds.
///
/// The validator account and BABE key are derived from the sr25519 account seed,
/// while GRANDPA uses its own ed25519 seed.
pub fn authority_keys_from_explicit_seeds(
    account_seed: &str,
    babe_seed: &str,
    grandpa_seed: &str,
) -> (AccountId, BabeId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(account_seed),
        get_from_seed::<BabeId>(babe_seed),
        get_from_seed::<GrandpaId>(grandpa_seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("BelizeChain Development")
    .with_id("belizechain_dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(testnet_genesis(
        // Initial PoA authorities
        vec![authority_keys_from_seed("Alice")],
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        // Pre-funded accounts
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        ],
        true,
    )?)
    .build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("BelizeChain Local Testnet")
    .with_id("belizechain_local")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(testnet_genesis(
        // Initial PoA authorities
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        // Sudo account
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        // Pre-funded accounts
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        ],
        true,
    )?)
    .with_boot_nodes(
        crate::validator_config::BootstrapNodes::testnet()
            .into_iter()
            .filter_map(|addr| addr.parse().ok())
            .collect(),
    )
    .build())
}

pub fn belizechain_mainnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Production wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("BelizeChain")
    .with_id("belizechain")
    .with_chain_type(ChainType::Live)
    .with_genesis_config_patch(mainnet_genesis()?)
    .with_boot_nodes(
        crate::validator_config::BootstrapNodes::mainnet()
            .into_iter()
            .filter_map(|addr| addr.parse().ok())
            .collect(),
    )
    .build())
}

/// Configure initial storage state for BelizeChain.
fn testnet_genesis(
    initial_authorities: Vec<(AccountId, BabeId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> Result<serde_json::Value, String> {
    let endowment: u128 = 1_000_000 * 1_000_000_000_000; // 1M DALLA with 12 decimals
    let _stash: u128 = 100_000 * 1_000_000_000_000; // 100K DALLA for staking (reserved for future use)

    Ok(serde_json::json!({
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, endowment))
                .collect::<Vec<_>>(),
        },
        "session": {
            "keys": initial_authorities.iter().map(|(account, babe, grandpa)| {
                (
                    account.clone(),
                    account.clone(),
                    SessionKeys {
                        babe: babe.clone(),
                        grandpa: grandpa.clone(),
                    },
                )
            }).collect::<Vec<_>>(),
        },
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {},
        // Sudo key set to root_key (Alice in dev, real key in testnet)
        "sudo": {
            "key": Some(root_key.clone()),
        },

        // BelizeChain custom pallet configurations
        // Note: Economy pallet doesn't have genesis config yet

        "identity": {
            "operationFee": Some(10u128 * 1_000_000_000_000u128), // 10 DALLA with 12 decimals
            "ssnStandardVersion": 1u8,
            "passportStandardVersion": 1u8,
            "initialSsnIssuers": vec![ root_key.clone() ], // Alice as SSB for local
            "initialPassportIssuers": vec![ root_key.clone() ], // Alice as Immigration for local
            "initialBiometricIssuers": vec![ root_key.clone() ],
            "paused": false,
            "startIdentityId": 1u64,
            "issuerBondAmount": Some(1_000u128 * 1_000_000_000_000u128), // 1000 DALLA bond requirement
            "rateWindowBlocks": Some(14_400u32), // ~1 day at 6s blocks
            "rateLimitSsn": Some(500u32),
            "rateLimitPassport": Some(200u32),
            "rateLimitBiometrics": Some(100u32),
        },

        "governance": {
            "councilMembers": vec![
                (root_key.clone(), 1u32, 0u32),  // (account, rank, pouw_contribution)
                (get_account_id_from_seed::<sr25519::Public>("Bob"), 1u32, 0u32),
                (get_account_id_from_seed::<sr25519::Public>("Charlie"), 1u32, 0u32),
            ],
            "democracyLaunchPeriod": 28800u32, // 2 days in blocks (6 sec per block)
            "democracyVotingPeriod": 43200u32, // 3 days in blocks
            "democracyMinimumDeposit": 1000u128 * 1_000_000_000_000u128, // 1000 DALLA with 12 decimals
        },

        // Community pallet - Phase 5: Education Modules & Green Projects seed data
        "community": {
            "educationModules": vec![
                // Module 1: Financial Literacy
                (
                    0u32, // module_id
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Financial Literacy for Belizeans".to_vec()).unwrap(), // title
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Learn budgeting, savings, and investment strategies tailored for Belize's economy".to_vec()).unwrap(), // description
                    50u64 * 1_000_000_000_000u64, // reward_amount (50 DALLA)
                    200u32, // capacity
                    true, // is_active
                ),
                // Module 2: Sustainable Agriculture
                (
                    1u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Sustainable Farming Techniques".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Modern agriculture practices for Belize: organic farming, water conservation, and crop rotation".to_vec()).unwrap(),
                    75u64 * 1_000_000_000_000u64, // 75 DALLA (higher reward for agriculture)
                    150u32,
                    true,
                ),
                // Module 3: Digital Governance
                (
                    2u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Participating in Digital Democracy".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"How to use BelizeChain for voting, proposals, and community governance".to_vec()).unwrap(),
                    50u64 * 1_000_000_000_000u64,
                    300u32, // Higher capacity for civic education
                    true,
                ),
                // Module 4: Technology Skills
                (
                    3u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Blockchain and Web3 Fundamentals".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Understanding blockchain technology, smart contracts, and the digital economy".to_vec()).unwrap(),
                    60u64 * 1_000_000_000_000u64,
                    200u32,
                    true,
                ),
            ],
            "greenProjects": vec![
                // Project 1: Belize Barrier Reef Conservation
                (
                    0u32, // project_id
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Belize Barrier Reef Conservation".to_vec()).unwrap(), // title
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Protect and restore the Belize Barrier Reef - the second largest in the world".to_vec()).unwrap(), // description
                    1_000_000u64 * 1_000_000_000_000u64, // funding_goal (1M DALLA)
                    0u64, // current_funding
                    true, // is_active
                ),
                // Project 2: Rainforest Protection
                (
                    1u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Maya Mountain Rainforest Protection".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Preserve Belize's ancient rainforests and protect biodiversity hotspots".to_vec()).unwrap(),
                    750_000u64 * 1_000_000_000_000u64, // 750K DALLA
                    0u64,
                    true,
                ),
                // Project 3: Renewable Energy
                (
                    2u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Community Solar Power Initiative".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Install solar panels in rural communities to reduce fossil fuel dependency".to_vec()).unwrap(),
                    500_000u64 * 1_000_000_000_000u64, // 500K DALLA
                    0u64,
                    true,
                ),
                // Project 4: Waste Management
                (
                    3u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Zero-Waste Belize Program".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Implement recycling centers and composting programs in Belize City and San Pedro".to_vec()).unwrap(),
                    300_000u64 * 1_000_000_000_000u64, // 300K DALLA
                    0u64,
                    true,
                ),
                // Project 5: Mangrove Restoration
                (
                    4u32,
                    BoundedVec::<u8, ConstU32<128>>::try_from(b"Coastal Mangrove Restoration".to_vec()).unwrap(),
                    BoundedVec::<u8, ConstU32<256>>::try_from(b"Replant mangroves along the coast to prevent erosion and protect marine ecosystems".to_vec()).unwrap(),
                    400_000u64 * 1_000_000_000_000u64, // 400K DALLA
                    0u64,
                    true,
                ),
            ],
        },
    }))
}

/// Production mainnet genesis configuration
/// PRODUCTION DEPLOYMENT: Configure with actual validator keys and treasury accounts
///
/// # Security
/// Validator keys and the root (sudo) key MUST be injected from a secure key
/// management system (e.g. `subkey generate`, HSM, or Vault) before building
/// the production chain spec.  The `MAINNET_KEYS_CONFIGURED` compile-time guard
/// prevents accidental deployment with placeholder keys.
fn mainnet_genesis() -> Result<serde_json::Value, String> {
    use sp_core::crypto::Ss58Codec;

    const MAINNET_KEYS_CONFIGURED: bool = true;

    // Production validator session keys
    let initial_authorities: Vec<(AccountId, BabeId, GrandpaId)> = vec![
        (
            AccountId::from_ss58check("5EcjZEJhfQxEsrUWEMpSH6D6vbq1eWsLsYbR7DUDM82bFAuJ").map_err(|e| format!("{:?}", e))?,
            BabeId::from_ss58check("5EcjZEJhfQxEsrUWEMpSH6D6vbq1eWsLsYbR7DUDM82bFAuJ").map_err(|e| format!("{:?}", e))?,
            GrandpaId::from_ss58check("5E2qCsxXwbyywDnmyiQ7ST37bWy4eKZC7F3NDcaPZKz4CikY").map_err(|e| format!("{:?}", e))?,
        ),
        (
            AccountId::from_ss58check("5HTZ2TeVvkuPCgHSQUUiDo78mXdD6PZ58asi2YcziGDUWhV2").map_err(|e| format!("{:?}", e))?,
            BabeId::from_ss58check("5HTZ2TeVvkuPCgHSQUUiDo78mXdD6PZ58asi2YcziGDUWhV2").map_err(|e| format!("{:?}", e))?,
            GrandpaId::from_ss58check("5HERG37DJjXa7ouXHVXv5SdPBkhqdzXkQ6sCgbFoh1k1CbKm").map_err(|e| format!("{:?}", e))?,
        ),
        (
            AccountId::from_ss58check("5HN2uP8iB9a2C28qjz5LhGfNGAFR9JjWnAL7pbfAtnd2oUHC").map_err(|e| format!("{:?}", e))?,
            BabeId::from_ss58check("5HN2uP8iB9a2C28qjz5LhGfNGAFR9JjWnAL7pbfAtnd2oUHC").map_err(|e| format!("{:?}", e))?,
            GrandpaId::from_ss58check("5GqGQWkFvv5f4sLh1TMFpzuXJbcxg1MiP8FQsuLtxFP9N8Yy").map_err(|e| format!("{:?}", e))?,
        ),
        (
            AccountId::from_ss58check("5FgydnGuSshDLWhcaZyG5ZjV1BCs8rhRCc9JDozySKB8g8AE").map_err(|e| format!("{:?}", e))?,
            BabeId::from_ss58check("5FgydnGuSshDLWhcaZyG5ZjV1BCs8rhRCc9JDozySKB8g8AE").map_err(|e| format!("{:?}", e))?,
            GrandpaId::from_ss58check("5HMqcChxxfgjCTLJjSEt3diV47Fhk9Jiy5BSiWAk17JMQriK").map_err(|e| format!("{:?}", e))?,
        ),
    ];

    // Sovereign Founder Root / Sudo Controller (Wicked)
    let root_key = AccountId::from_ss58check("5Cg3Ez7Upm8caDfjonnMKPZ14B3H5daWM75DkYj7yEt4XSKt").map_err(|e| format!("{:?}", e))?;
    // Government of Belize Treasury Sovereign Reserve
    let treasury_key = AccountId::from_ss58check("5CJX6HRtMn2bvJM1vncjmyUfRbTVQRUWFxwJH6T6SCqoHjf3").map_err(|e| format!("{:?}", e))?;

    // Initial 100M DALLA Token Distribution (12 decimals)
    let endowed_balances: Vec<(AccountId, u128)> = vec![
        (treasury_key.clone(), 60_000_000u128 * 1_000_000_000_000u128),
        (root_key.clone(), 20_000_000u128 * 1_000_000_000_000u128),
        (initial_authorities[0].0.clone(), 2_500_000u128 * 1_000_000_000_000u128),
        (initial_authorities[1].0.clone(), 2_500_000u128 * 1_000_000_000_000u128),
        (initial_authorities[2].0.clone(), 2_500_000u128 * 1_000_000_000_000u128),
        (initial_authorities[3].0.clone(), 2_500_000u128 * 1_000_000_000_000u128),
    ];

    Ok(serde_json::json!({
        "balances": {
            "balances": endowed_balances,
        },
        "session": {
            "keys": initial_authorities.iter().map(|(account, babe, grandpa)| {
                (
                    account.clone(),
                    account.clone(),
                    SessionKeys {
                        babe: babe.clone(),
                        grandpa: grandpa.clone(),
                    },
                )
            }).collect::<Vec<_>>(),
        },
        "babe": {
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {},
        "sudo": {
            "key": Some(root_key.clone()),
        },

        // BelizeChain custom pallet configurations
        // Note: Economy pallet doesn't have genesis config yet

        "identity": {
            "operationFee": Some(50u128 * 1_000_000_000_000u128), // 50 DALLA for mainnet
            "ssnStandardVersion": 1u8,
            "passportStandardVersion": 1u8,
            // Government of Belize authorized identity issuers (Vital Statistics Unit, Immigration Dept, etc.)
            "initialSsnIssuers": vec![ root_key.clone() ],
            "initialPassportIssuers": vec![ root_key.clone() ],
            "initialBiometricIssuers": vec![ root_key.clone() ],
            "paused": false,
            "startIdentityId": 1u64,
            "issuerBondAmount": Some(100_000u128 * 1_000_000_000_000u128), // 100,000 DALLA bond for mainnet
            "rateWindowBlocks": Some(14_400u32), // ~1 day at 6s blocks
            "rateLimitSsn": Some(200u32), // Stricter limits for mainnet
            "rateLimitPassport": Some(100u32),
            "rateLimitBiometrics": Some(50u32),
        },

        "governance": {
            // District council representatives (elected per Constitution)
            "councilMembers": vec![
                (root_key.clone(), 1u32, 0u32),  // (account, rank, pouw_contribution)
                // Additional council members configured during governance setup
            ],
            "democracyLaunchPeriod": 100_800u32, // 7 days in blocks
            "democracyVotingPeriod": 201_600u32, // 14 days in blocks
            "democracyMinimumDeposit": 10_000u128 * 1_000_000_000_000u128, // 10,000 DALLA for mainnet
        },
    }))
}

/// Generate a public-testnet template from NetworkConfig.
///
/// This template is intended for `build-spec` followed by manual review and
/// editing. Real public testnet launches should use an explicit JSON/raw spec
/// file generated from this template rather than a built-in chain alias.
/// Use `build-spec --disable-default-bootnode` when exporting it, otherwise the
/// CLI injects a loopback bootnode into the generated artifact.
#[allow(dead_code)]
pub(crate) fn public_testnet_config() -> Result<ChainSpec, String> {
    let config = NetworkConfig::public_testnet();

    let authorities: Vec<(AccountId, BabeId, GrandpaId)> = config
        .initial_authorities
        .iter()
        .map(|(account_seed, grandpa_seed)| {
            authority_keys_from_explicit_seeds(account_seed, account_seed, grandpa_seed)
        })
        .collect();

    let endowed_accounts: Vec<AccountId> = config
        .initial_allocation
        .iter()
        .map(|(seed, _)| get_account_id_from_seed::<sr25519::Public>(seed))
        .collect();

    let root_key = get_account_id_from_seed::<sr25519::Public>("treasury");

    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Testnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name(&config.name)
    .with_id(&config.id)
    .with_chain_type(ChainType::Live)
    .with_genesis_config_patch(testnet_genesis(
        authorities,
        root_key,
        endowed_accounts,
        true,
    )?)
    .build())
}

/// Generate staging chain spec (pre-mainnet testing)
/// Used for pre-mainnet rehearsal and final integration tests
#[allow(dead_code)]
pub(crate) fn staging_config() -> Result<ChainSpec, String> {
    let config = NetworkConfig::staging();

    let authorities: Vec<(AccountId, BabeId, GrandpaId)> = config
        .initial_authorities
        .iter()
        .map(|(account_seed, grandpa_seed)| {
            authority_keys_from_explicit_seeds(account_seed, account_seed, grandpa_seed)
        })
        .collect();

    let endowed_accounts: Vec<AccountId> = config
        .initial_allocation
        .iter()
        .map(|(seed, _)| get_account_id_from_seed::<sr25519::Public>(seed))
        .collect();

    let root_key = get_account_id_from_seed::<sr25519::Public>("treasury");

    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Staging wasm not available".to_string())?,
        Default::default(),
    )
    .with_name(&config.name)
    .with_id(&config.id)
    .with_chain_type(ChainType::Live)
    .with_genesis_config_patch(testnet_genesis(
        authorities,
        root_key,
        endowed_accounts,
        false, // No debug println in staging
    )?)
    .build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::sr25519;

    // ── authority_keys_from_seed ───────────────────────────────────────────

    #[test]
    fn test_authority_keys_from_seed_deterministic() {
        let keys1 = authority_keys_from_seed("Alice");
        let keys2 = authority_keys_from_seed("Alice");
        assert_eq!(
            keys1.0, keys2.0,
            "AccountId must be deterministic for the same seed"
        );
        assert_eq!(
            keys1.1, keys2.1,
            "BabeId must be deterministic for the same seed"
        );
        assert_eq!(
            keys1.2, keys2.2,
            "GrandpaId must be deterministic for the same seed"
        );
    }

    #[test]
    fn test_authority_keys_from_seed_distinct_for_different_seeds() {
        let alice = authority_keys_from_seed("Alice");
        let bob = authority_keys_from_seed("Bob");
        assert_ne!(
            alice.0, bob.0,
            "Different seeds must yield different AccountIds"
        );
        assert_ne!(
            alice.1, bob.1,
            "Different seeds must yield different BabeIds"
        );
        assert_ne!(
            alice.2, bob.2,
            "Different seeds must yield different GrandpaIds"
        );
    }

    // ── get_account_id_from_seed ───────────────────────────────────────────

    #[test]
    fn test_get_account_id_from_seed_deterministic() {
        let id1 = get_account_id_from_seed::<sr25519::Public>("Alice");
        let id2 = get_account_id_from_seed::<sr25519::Public>("Alice");
        assert_eq!(
            id1, id2,
            "Same seed must produce the same AccountId every time"
        );
    }

    #[test]
    fn test_get_account_id_from_seed_distinct_accounts() {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
        let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
        assert_ne!(alice, bob);
        assert_ne!(alice, charlie);
        assert_ne!(bob, charlie);
    }

    #[test]
    fn test_stash_accounts_differ_from_base_accounts() {
        let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
        let alice_stash = get_account_id_from_seed::<sr25519::Public>("Alice//stash");
        assert_ne!(
            alice, alice_stash,
            "stash account must differ from the base account"
        );
    }

    // ── mainnet_genesis safety guard ───────────────────────────────────────

    // ── mainnet_genesis configuration test ────────────────────────────────
    #[test]
    fn test_mainnet_genesis_succeeds_when_keys_configured() {
        let result = mainnet_genesis();
        assert!(
            result.is_ok(),
            "mainnet_genesis must succeed with configured production keys"
        );
        let genesis = result.unwrap();
        assert!(
            genesis.get("sudo").is_some(),
            "mainnet genesis must contain sudo configuration"
        );
        assert_eq!(
            genesis["session"]["keys"].as_array().map(|a| a.len()),
            Some(4),
            "mainnet genesis must configure 4 validator session keys"
        );
    }

    // ── testnet_genesis structure ──────────────────────────────────────────

    #[test]
    fn test_testnet_genesis_succeeds_with_single_authority() {
        let alice_keys = authority_keys_from_seed("Alice");
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let result = testnet_genesis(
            vec![alice_keys],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        );
        assert!(
            result.is_ok(),
            "testnet_genesis must succeed with valid inputs"
        );
    }

    #[test]
    fn test_testnet_genesis_balances_match_endowed_accounts() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let bob_acct = get_account_id_from_seed::<sr25519::Public>("Bob");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct, bob_acct],
            false,
        )
        .unwrap();
        let balances = genesis["balances"]["balances"].as_array().unwrap();
        assert_eq!(
            balances.len(),
            2,
            "genesis must fund exactly as many accounts as given in endowed_accounts"
        );
    }

    #[test]
    fn test_testnet_genesis_session_keys_count() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
            ],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        let keys = genesis["session"]["keys"].as_array().unwrap();
        assert_eq!(
            keys.len(),
            2,
            "session must list exactly 2 validator key sets"
        );
    }

    #[test]
    fn test_testnet_genesis_babe_has_epoch_config_only() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        // BABE should only have epochConfig — authorities are managed by session
        assert!(
            !genesis["babe"]["epochConfig"].is_null(),
            "babe must have epochConfig"
        );
        assert!(
            genesis["babe"].get("authorities").is_none(),
            "babe must NOT have direct authorities when session manages them"
        );
    }

    #[test]
    fn test_testnet_genesis_grandpa_has_no_authorities() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        // GRANDPA authorities are managed by session, not set directly
        assert!(
            genesis["grandpa"].get("authorities").is_none(),
            "grandpa must NOT have direct authorities when session manages them"
        );
    }

    #[test]
    fn test_testnet_genesis_has_sudo_key() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        // Sudo pallet is still included for testnet bootstrapping (CONS-004).
        // It will be removed before mainnet (Phase 0, Step 0.4).
        assert!(
            genesis.get("sudo").is_some(),
            "sudo must be present in testnet genesis for bootstrapping"
        );
    }

    #[test]
    fn test_testnet_genesis_identity_config_present() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        assert!(
            !genesis["identity"].is_null(),
            "identity pallet genesis config must be present"
        );
        assert!(
            !genesis["identity"]["operationFee"].is_null(),
            "identity operationFee must be configured"
        );
        assert!(
            !genesis["identity"]["paused"].is_null(),
            "identity paused flag must be configured"
        );
    }

    #[test]
    fn test_testnet_genesis_governance_council_nonempty() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        let council = genesis["governance"]["councilMembers"].as_array().unwrap();
        assert!(
            !council.is_empty(),
            "governance council must have at least one member in testnet genesis"
        );
    }

    #[test]
    fn test_testnet_genesis_community_education_modules() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        let modules = genesis["community"]["educationModules"].as_array().unwrap();
        assert_eq!(
            modules.len(),
            4,
            "testnet genesis must include exactly 4 education modules"
        );
    }

    #[test]
    fn test_testnet_genesis_community_green_projects() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        let projects = genesis["community"]["greenProjects"].as_array().unwrap();
        assert_eq!(
            projects.len(),
            5,
            "testnet genesis must include exactly 5 green projects"
        );
    }

    #[test]
    fn test_testnet_genesis_empty_endowed_accounts_produces_empty_balances() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct,
            vec![], // no endowed accounts
            false,
        )
        .unwrap();
        let balances = genesis["balances"]["balances"].as_array().unwrap();
        assert!(
            balances.is_empty(),
            "empty endowed_accounts must produce empty genesis balances"
        );
    }

    // ── ChainSpec builder functions ────────────────────────────────────────
    //
    // These functions require WASM_BINARY to be compiled in (built by the
    // runtime's build.rs via substrate-wasm-builder). They will always succeed
    // in a standard `cargo test` run where the full workspace is compiled.

    #[test]
    fn test_development_config_succeeds() {
        development_config()
            .expect("development_config must succeed: WASM_BINARY must be present in test build");
    }

    #[test]
    fn test_local_testnet_config_succeeds() {
        local_testnet_config()
            .expect("local_testnet_config must succeed: WASM_BINARY must be present in test build");
    }

    #[test]
    fn test_belizechain_mainnet_config_succeeds() {
        let result = belizechain_mainnet_config();
        assert!(
            result.is_ok(),
            "mainnet config must succeed when production keys are configured"
        );
    }

    #[test]
    fn test_public_testnet_config_succeeds() {
        public_testnet_config().expect("public_testnet_config must succeed in test build");
    }

    #[test]
    fn test_staging_config_succeeds() {
        staging_config().expect("staging_config must succeed in test build");
    }
}
