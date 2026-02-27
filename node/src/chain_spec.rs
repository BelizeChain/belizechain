use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Pair, Public, sr25519};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};
use sc_service::ChainType;
use frame_support::{BoundedVec, pallet_prelude::ConstU32};

use belizechain_runtime::{AccountId, WASM_BINARY};
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

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(s),
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
    .build())
}

/// Configure initial storage state for BelizeChain.
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
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
        "aura": {
            "authorities": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
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
    //   let aura_key = AuraId::from_slice(&hex!("...")).unwrap();
    //   let gran_key = GrandpaId::from_slice(&hex!("...")).unwrap();
    //
    // PLACEHOLDER keys (will be rejected at runtime by the guard above):
    let initial_authorities: Vec<(AuraId, GrandpaId)> = vec![
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
        "aura": {
            "authorities": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
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
    
    let authorities: Vec<(AuraId, GrandpaId)> = config.initial_authorities
        .iter()
        .map(|(aura_seed, grandpa_seed)| {
            authority_keys_from_seed(&format!("{}{}", aura_seed, grandpa_seed))
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
    
    let authorities: Vec<(AuraId, GrandpaId)> = config.initial_authorities
        .iter()
        .map(|(aura_seed, grandpa_seed)| {
            authority_keys_from_seed(&format!("{}{}", aura_seed, grandpa_seed))
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