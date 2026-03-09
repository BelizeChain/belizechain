# BelizeChain Mainnet Key Generation Guide

## ⚠️ CRITICAL SECURITY NOTICE

**NEVER deploy to mainnet using development keys (Alice, Bob, Charlie, etc.)**

The current `chain_spec.rs` contains placeholder keys marked with `TODO` comments. These **MUST** be replaced with secure, production-generated keys before any mainnet deployment.

---

## Required Keys for Mainnet

### 1. **Validator Keys** (3+ nodes recommended)
Each validator requires two key types:
- **BABE (Sr25519)**: Block production (consensus)
- **Grandpa (Ed25519)**: Block finalization (GRANDPA consensus)

### 2. **Treasury/Root Account**
- **Sr25519**: Sudo account for initial governance
- Will be removed after on-chain governance is fully operational

### 3. **Government Issuer Accounts**
- **SSN Issuer**: Social Security Board account
- **Passport Issuer**: Immigration Department account  
- **Biometric Issuer**: National ID authority

### 4. **Council Members** (7+ recommended)
- **District Representatives**: One per district (Belize, Cayo, Corozal, Orange Walk, Stann Creek, Toledo)
- **At-large Members**: Additional community representatives

---

## Key Generation Methods

### Method 1: Using Subkey (Recommended)

```bash
# Install subkey
cargo install --force --git https://github.com/paritytech/polkadot-sdk subkey

# Generate BABE keys (Sr25519)
subkey generate --scheme Sr25519 --output-type json > validator1_babe.json

# Generate Grandpa keys (Ed25519)
subkey generate --scheme Ed25519 --output-type json > validator1_grandpa.json

# Repeat for each validator (validator2, validator3, etc.)
```

### Method 2: Using Polkadot.js Apps

1. Navigate to https://polkadot.js.org/apps/
2. Go to **Settings** → **Developer**
3. Use **Accounts** → **Add account** → **Advanced creation options**
4. Select **Sr25519** for BABE, **Ed25519** for Grandpa
5. **Save mnemonics SECURELY** (hardware wallet recommended)

### Method 3: Hardware Wallet (Most Secure)

For production mainnet, use hardware wallets (Ledger, Polkadot Vault):
1. Generate keys on air-gapped hardware device
2. Never expose private keys/mnemonics to networked computers
3. Use multi-signature schemes for treasury/sudo

---

## Updating chain_spec.rs

### Step 1: Generate All Keys

```bash
# Example: Generate 3 validators + 1 treasury
./scripts/generate_mainnet_keys.sh
```

### Step 2: Extract Public Keys

From your JSON files or hardware wallet, extract:
- BABE Public Key (Sr25519, starts with `0x...`)
- Grandpa Public Key (Ed25519, starts with `0x...`)
- Treasury Account ID (SS58 format: `5G...` or `5H...`)

### Step 3: Update `node/src/chain_spec.rs`

Replace line 181-186:

```rust
// ❌ BEFORE (INSECURE - development keys)
let initial_authorities: Vec<(BabeId, GrandpaId)> = vec![
    authority_keys_from_seed("ValidatorOne"),
    authority_keys_from_seed("ValidatorTwo"),
    authority_keys_from_seed("ValidatorThree"),
];

// ✅ AFTER (SECURE - production keys)
let initial_authorities: Vec<(BabeId, GrandpaId)> = vec![
    (
        // Validator 1 - Belize City Node
        hex!["YOUR_VALIDATOR1_BABE_PUBLIC_KEY"].unchecked_into(),
        hex!["YOUR_VALIDATOR1_GRANDPA_PUBLIC_KEY"].unchecked_into(),
    ),
    (
        // Validator 2 - Belmopan Node  
        hex!["YOUR_VALIDATOR2_BABE_PUBLIC_KEY"].unchecked_into(),
        hex!["YOUR_VALIDATOR2_GRANDPA_PUBLIC_KEY"].unchecked_into(),
    ),
    (
        // Validator 3 - San Ignacio Node
        hex!["YOUR_VALIDATOR3_BABE_PUBLIC_KEY"].unchecked_into(),
        hex!["YOUR_VALIDATOR3_GRANDPA_PUBLIC_KEY"].unchecked_into(),
    ),
];
```

Replace line 189:

```rust
// ❌ BEFORE
let root_key = get_account_id_from_seed::<sr25519::Public>("TreasuryAccount");

// ✅ AFTER
let root_key: AccountId = hex!["YOUR_TREASURY_ACCOUNT_ID"].into();
```

Replace lines 227-229 (Government issuers):

```rust
// ❌ BEFORE
"initialSsnIssuers": vec![ root_key.clone() ],
"initialPassportIssuers": vec![ root_key.clone() ],
"initialBiometricIssuers": vec![ root_key.clone() ],

// ✅ AFTER
"initialSsnIssuers": vec![ hex!["SSN_ISSUER_ACCOUNT_ID"].into() ],
"initialPassportIssuers": vec![ hex!["PASSPORT_ISSUER_ACCOUNT_ID"].into() ],
"initialBiometricIssuers": vec![ hex!["BIOMETRIC_ISSUER_ACCOUNT_ID"].into() ],
```

---

## Security Checklist

Before mainnet launch, verify:

- [ ] All validator keys generated on air-gapped/hardware devices
- [ ] Mnemonics stored in secure offline backup (NOT in code/git)
- [ ] Treasury account uses multi-signature (3-of-5 or 4-of-7)
- [ ] Government issuer accounts controlled by respective departments
- [ ] No development keys (`Alice`, `Bob`, `seed` references) in production config
- [ ] Council members are real elected/appointed officials
- [ ] Initial token distribution reviewed by governance
- [ ] Validator nodes deployed in geographically distributed locations
- [ ] Backup validator nodes ready (7+ total recommended)
- [ ] Key rotation procedures documented
- [ ] Emergency sudo removal timeline established (6-12 months post-launch)

---

## Key Storage Best Practices

1. **Never commit private keys to Git**
2. **Use encrypted storage** (KeePassXC, Bitwarden, hardware wallets)
3. **Implement key rotation** every 12-18 months
4. **Multi-signature for critical accounts** (treasury, sudo)
5. **Geographic distribution** of validator keys
6. **Legal custody** for government issuer keys (ministerial control)
7. **Disaster recovery plan** with secure key backup locations

---

## Post-Launch: Removing Sudo

After governance stabilizes (6-12 months):

```rust
// Remove sudo pallet from runtime/src/lib.rs
// Transfer all privileged operations to on-chain governance
// This makes BelizeChain fully decentralized
```

---

## Support & Security Audits

Before mainnet launch:
- [ ] External security audit of chain_spec.rs
- [ ] Validator key ceremony with witnesses
- [ ] Legal documentation for government key custody
- [ ] Insurance for validator infrastructure
- [ ] 24/7 monitoring and incident response team

---

## Quick Reference: Account Formats

| Type | Format | Example | Usage |
|------|--------|---------|-------|
| **BABE (Sr25519)** | Hex (66 chars) | `0x1234...abcd` | Block production |
| **Grandpa (Ed25519)** | Hex (66 chars) | `0x5678...ef01` | Block finalization |
| **Account ID** | Hex (66 chars) or SS58 | `0xabcd...`, `5GrwvaEF5...` | Treasury, issuers |

Convert between formats using:
```bash
subkey inspect "YOUR_MNEMONIC_PHRASE"
```

---

**Last Updated**: October 27, 2025  
**Status**: Development keys in use - **DO NOT DEPLOY TO MAINNET**
