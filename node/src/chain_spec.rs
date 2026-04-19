use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Pair, Public, sr25519};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};
use sc_service::ChainType;
use frame_support::{BoundedVec, pallet_prelude::ConstU32};

use belizechain_runtime::{AccountId, BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY};
use crate::chain_spec_configs::NetworkConfig;

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

/// Generate a BABE authority key.
pub fn authority_keys_from_seed(s: &str) -> (BabeId, GrandpaId) {
    (
        get_from_seed::<BabeId>(s),
        get_from_seed::<GrandpaId>(s),
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
            .collect()
    )
    .build())
}

/// BelizeChain Testnet — single-validator chain for AKS deployment.
///
/// Uses real keys generated via `subkey` (not dev seeds).
/// Sr25519 (BABE/Account): 5Dk6wqPb2wzD1DscaBqNemv1xrETVQnmqXpjk2z6oHpQgjx8
/// Ed25519 (GRANDPA):      5FsJu5aQCFsz2kEB2pdYpXF7JD2Jfr1qWUXhzz81uMThSGcw
pub fn belizechain_testnet_config() -> Result<ChainSpec, String> {
    use sp_core::crypto::Ss58Codec;

    let babe_key = BabeId::from_ss58check(
        "5Dk6wqPb2wzD1DscaBqNemv1xrETVQnmqXpjk2z6oHpQgjx8",
    )
    .map_err(|e| format!("Invalid BABE key: {e:?}"))?;
    let grandpa_key = GrandpaId::from_ss58check(
        "5FsJu5aQCFsz2kEB2pdYpXF7JD2Jfr1qWUXhzz81uMThSGcw",
    )
    .map_err(|e| format!("Invalid GRANDPA key: {e:?}"))?;

    let initial_authorities: Vec<(BabeId, GrandpaId)> = vec![(babe_key, grandpa_key)];

    let root_key: AccountId = sp_runtime::AccountId32::from_ss58check(
        "5Dk6wqPb2wzD1DscaBqNemv1xrETVQnmqXpjk2z6oHpQgjx8",
    )
    .map_err(|e| format!("Invalid root key: {e:?}"))?;

    let endowment: u128 = 10_000_000 * 1_000_000_000_000; // 10M DALLA

    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Testnet wasm not available".to_string())?,
        Default::default(),
    )
    .with_name("BelizeChain Testnet")
    .with_id("belizechain_testnet")
    .with_chain_type(ChainType::Live)
    .with_genesis_config_patch(serde_json::json!({
        "balances": {
            "balances": vec![(root_key.clone(), endowment)],
        },
        "babe": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone(), 1u64)).collect::<Vec<_>>(),
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key.clone()),
        },
        "identity": {
            "operationFee": Some(10u128 * 1_000_000_000_000u128),
            "ssnStandardVersion": 1u8,
            "passportStandardVersion": 1u8,
            "initialSsnIssuers": vec![ root_key.clone() ],
            "initialPassportIssuers": vec![ root_key.clone() ],
            "initialBiometricIssuers": vec![ root_key.clone() ],
            "paused": false,
            "startIdentityId": 1u64,
            "issuerBondAmount": Some(1_000u128 * 1_000_000_000_000u128),
            "rateWindowBlocks": Some(14_400u32),
            "rateLimitSsn": Some(500u32),
            "rateLimitPassport": Some(200u32),
            "rateLimitBiometrics": Some(100u32),
        },
        "governance": {
            "councilMembers": vec![
                (root_key.clone(), 1u32, 0u32),
            ],
            "democracyLaunchPeriod": 28800u32,
            "democracyVotingPeriod": 43200u32,
            "democracyMinimumDeposit": 1000u128 * 1_000_000_000_000u128,
        },
    }))
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
            .collect()
    )
    .build())
}

/// Configure initial storage state for BelizeChain.
fn testnet_genesis(
    initial_authorities: Vec<(BabeId, GrandpaId)>,
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
        "babe": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone(), 1u64)).collect::<Vec<_>>(),
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
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
    // ── COMPILE-TIME SAFETY GUARD ──────────────────────────────────────────
    // Flip this to `true` ONLY after replacing the placeholder keys below
    // with real, securely-generated validator keys.
    const MAINNET_KEYS_CONFIGURED: bool = false;
    if !MAINNET_KEYS_CONFIGURED {
        return Err(
            "SECURITY: Mainnet genesis still uses placeholder keys. \
             Generate real validator keys with `subkey generate`, replace \
             the entries below, and set MAINNET_KEYS_CONFIGURED = true."
                .into(),
        );
    }

    // Production validator keys — replace with output of `subkey generate`
    // before setting MAINNET_KEYS_CONFIGURED = true.
    //
    // Example (DO NOT USE — generate your own):
    //   let babe_key = BabeId::from_slice(&hex!("...")).unwrap();
    //   let gran_key = GrandpaId::from_slice(&hex!("...")).unwrap();
    //
    // PLACEHOLDER keys (will be rejected at runtime by the guard above):
    let initial_authorities: Vec<(BabeId, GrandpaId)> = vec![
        authority_keys_from_seed("ValidatorOne"),
        authority_keys_from_seed("ValidatorTwo"),
        authority_keys_from_seed("ValidatorThree"),
    ];

    // Government of Belize treasury account (multi-signature 4-of-7)
    // PLACEHOLDER — replace with a real multi-sig account before mainnet.
    let root_key = get_account_id_from_seed::<sr25519::Public>("TreasuryAccount");

    // Initial token distribution per Step 9 Token Economics (100M DALLA genesis)
    let endowed_accounts: Vec<AccountId> = vec![
        root_key.clone(),
        // Additional accounts configured during deployment
    ];

    let endowment: u128 = 100_000_000 * 1_000_000_000_000; // 100M DALLA for treasury

    Ok(serde_json::json!({
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, endowment))
                .collect::<Vec<_>>(),
        },
        "babe": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone(), 1u64)).collect::<Vec<_>>(),
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
        // NOTE: Sudo pallet excluded — gated behind #[cfg(feature = "dev")] in runtime.
        
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

/// Generate testnet chain spec from NetworkConfig
/// Reserved for future public testnet deployment
#[allow(dead_code)]
pub(crate) fn public_testnet_config() -> Result<ChainSpec, String> {
    let config = NetworkConfig::public_testnet();
    
    let authorities: Vec<(BabeId, GrandpaId)> = config.initial_authorities
        .iter()
        .map(|(babe_seed, grandpa_seed)| {
            authority_keys_from_seed(&format!("{}{}", babe_seed, grandpa_seed))
        })
        .collect();
    
    let endowed_accounts: Vec<AccountId> = config.initial_allocation
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
    
    let authorities: Vec<(BabeId, GrandpaId)> = config.initial_authorities
        .iter()
        .map(|(babe_seed, grandpa_seed)| {
            authority_keys_from_seed(&format!("{}{}", babe_seed, grandpa_seed))
        })
        .collect();
    
    let endowed_accounts: Vec<AccountId> = config.initial_allocation
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
        assert_eq!(keys1.0, keys2.0, "BabeId must be deterministic for the same seed");
        assert_eq!(keys1.1, keys2.1, "GrandpaId must be deterministic for the same seed");
    }

    #[test]
    fn test_authority_keys_from_seed_distinct_for_different_seeds() {
        let alice = authority_keys_from_seed("Alice");
        let bob = authority_keys_from_seed("Bob");
        assert_ne!(alice.0, bob.0, "Different seeds must yield different BabeIds");
        assert_ne!(alice.1, bob.1, "Different seeds must yield different GrandpaIds");
    }

    // ── get_account_id_from_seed ───────────────────────────────────────────

    #[test]
    fn test_get_account_id_from_seed_deterministic() {
        let id1 = get_account_id_from_seed::<sr25519::Public>("Alice");
        let id2 = get_account_id_from_seed::<sr25519::Public>("Alice");
        assert_eq!(id1, id2, "Same seed must produce the same AccountId every time");
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
        assert_ne!(alice, alice_stash, "stash account must differ from the base account");
    }

    // ── mainnet_genesis safety guard ───────────────────────────────────────

    #[test]
    fn test_mainnet_genesis_blocked_when_keys_not_configured() {
        let result = mainnet_genesis();
        assert!(
            result.is_err(),
            "mainnet_genesis must return Err when MAINNET_KEYS_CONFIGURED = false"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("SECURITY"),
            "Error must contain 'SECURITY' keyword; got: {msg}"
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
        assert!(result.is_ok(), "testnet_genesis must succeed with valid inputs");
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
    fn test_testnet_genesis_babe_authority_count() {
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
        let authorities = genesis["babe"]["authorities"].as_array().unwrap();
        assert_eq!(authorities.len(), 2, "babe must list exactly 2 authorities");
    }

    #[test]
    fn test_testnet_genesis_grandpa_authority_count() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        let authorities = genesis["grandpa"]["authorities"].as_array().unwrap();
        assert_eq!(authorities.len(), 1, "grandpa must have 1 authority entry");
    }

    #[test]
    fn test_testnet_genesis_grandpa_weight_is_one() {
        let alice_acct = get_account_id_from_seed::<sr25519::Public>("Alice");
        let genesis = testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            alice_acct.clone(),
            vec![alice_acct],
            false,
        )
        .unwrap();
        // Each grandpa entry is [key, weight]. Weight must be 1 for uniform voting power.
        let entry = &genesis["grandpa"]["authorities"].as_array().unwrap()[0];
        let weight = entry.as_array().unwrap()[1].as_u64().unwrap();
        assert_eq!(weight, 1, "grandpa authority weight must be 1");
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
        assert!(!genesis["identity"].is_null(), "identity pallet genesis config must be present");
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
        assert_eq!(modules.len(), 4, "testnet genesis must include exactly 4 education modules");
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
        assert_eq!(projects.len(), 5, "testnet genesis must include exactly 5 green projects");
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
    fn test_belizechain_mainnet_config_fails_until_keys_configured() {
        // Even when WASM is available, mainnet_genesis() gate blocks the build
        // when MAINNET_KEYS_CONFIGURED = false, so this always returns Err.
        let result = belizechain_mainnet_config();
        assert!(
            result.is_err(),
            "mainnet config must fail until real validator keys are configured"
        );
    }

    #[test]
    fn test_public_testnet_config_succeeds() {
        public_testnet_config()
            .expect("public_testnet_config must succeed in test build");
    }

    #[test]
    fn test_staging_config_succeeds() {
        staging_config()
            .expect("staging_config must succeed in test build");
    }

    #[test]
    fn test_belizechain_testnet_config_succeeds() {
        let spec = belizechain_testnet_config()
            .expect("belizechain_testnet_config must succeed in test build");
        assert_eq!(spec.name(), "BelizeChain Testnet");
        assert_eq!(spec.id(), "belizechain_testnet");
    }
}