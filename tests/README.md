# 🔗 BelizeChain Integration Tests

## Overview

Integration tests validate that all BelizeChain pallets and systems work together as one cohesive blockchain:
- **Blockchain Core**: 15 custom pallets + system pallets
- **Cross-Pallet Interactions**: Economy+Compliance, Identity+Governance, Staking+Oracle
- **Economic System**: DALLA, bBZD, multi-sig treasury, cashback mechanics
- **End-to-End Workflows**: Citizen onboarding, business payments, government operations

## Test Categories

### 1. **Blockchain Core Pallet Tests** (15 pallets)
- **Economy**: DALLA/bBZD minting, transfers, multi-sig treasury, account types
- **Identity**: BelizeID registration, KYC verification, SSN/Passport integration
- **Governance**: Proposals, voting, council elections, treasury spending, JaguarMode
- **Compliance**: KYC/AML enforcement, sanctions, FSC oversight, limits
- **Staking**: Validator registration, consensus participation, rewards distribution
- **Oracle**: Data feed registration, merchant verification, rate updates
- **Payroll**: Government/private payroll, automated salary distribution
- **Interoperability**: Cross-chain bridges (Ethereum, Polkadot), asset transfers
- **BelizeX**: DEX trading, liquidity pools, asset registry, Oracle guards
- **LandLedger**: Property registry, title transfers, document proofs
- **Consensus**: Proof of Useful Work, validator selection, block production
- **Quantum**: Quantum workload integration, PQW rewards
- **Community**: Community governance, local initiatives, participation tracking
- **BNS**: Domain registration (.bz), marketplace, IPFS hosting
- **Contracts**: Wasm smart contract execution (ink! platform)

### 2. **Cross-Pallet Integration Tests**
- Economy + Compliance: KYC enforcement on transactions, sanction blocking
- Identity + Governance: Verified voter requirements, council eligibility
- Staking + Oracle: Validator data feeds, reputation scoring
- BelizeX + Oracle: Exchange rate guards, slippage protection
- LandLedger + Identity: Property ownership verification, KYC requirements
- Payroll + Economy: Automated salary distribution, multi-sig approvals

### 3. **Economic System Tests**
- DALLA/bBZD dual-token mechanics
- Central Bank bBZD minting (1:1 BZD backing)
- Tourism cashback (5-8% rewards for verified merchants)
- Multi-signature treasury operations (4-of-7 governance)
- Account type limits (Citizen, Business, Tourism, Government)

### 4. **End-to-End Scenarios**
- **Citizen Onboarding**: BelizeID → KYC → DALLA wallet → First transaction
- **Business Payment**: Merchant verification → Customer payment → Cashback calculation → bBZD redemption
- **Government Operation**: Treasury proposal → Council voting → Multi-sig approval → Fund distribution
- **Property Transfer**: LandLedger lookup → Identity verification → Title transfer → Document storage
- **Emergency Activation**: JaguarMode trigger → Transaction pause → Council approval → System resume

## Architecture

```
tests/
├── conftest.py                  # Shared fixtures (blockchain connection, keypairs)
├── requirements.txt             # Python dependencies (substrateinterface, pytest)
├── README.md                    # This file
├── TEST_OVERVIEW.md             # Complete testing strategy
├── test_security.py             # Security-focused tests
│
├── blockchain/                  # Core pallet tests (15 custom + system pallets)
│   ├── __init__.py
│   ├── README.md
│   ├── test_economy.py          # Economy pallet (DALLA/bBZD)
│   ├── test_identity.py         # Identity pallet (BelizeID/KYC)
│   ├── test_governance.py       # Governance pallet
│   ├── test_compliance.py       # Compliance pallet (KYC/AML)
│   ├── test_staking.py          # Staking pallet (validators)
│   ├── test_oracle.py           # Oracle pallet
│   ├── test_payroll.py          # Payroll pallet
│   ├── test_interoperability.py # Bridges (ETH/DOT)
│   ├── test_belizex.py          # DEX pallet
│   ├── test_landledger.py       # Land registry
│   ├── test_consensus.py        # Consensus pallet
│   ├── test_quantum.py          # Quantum workload
│   ├── test_community.py        # Community governance
│   ├── test_bns.py              # BNS (.bz domains)
│   └── test_contracts.py        # Wasm contracts
│
├── cross_pallet/                # Cross-pallet interaction tests
│   ├── __init__.py
│   ├── README.md
│   ├── test_economy_compliance.py      # Economy + Compliance KYC
│   ├── test_identity_governance.py     # Identity + Governance voting
│   ├── test_staking_oracle.py          # Staking + Oracle feeds
│   ├── test_belizex_oracle.py          # BelizeX + Oracle guards
│   ├── test_landledger_identity.py     # LandLedger + Identity KYC
│   └── test_payroll_economy.py         # Payroll + Economy salary
│
├── economic/                    # Economic system tests (DALLA/bBZD)
│   ├── __init__.py
│   ├── README.md
│   ├── test_bbzd_compliance.py
│   ├── test_bbzd_integration.py
│   ├── test_bbzd_minimal_integration.py
│   ├── test_bbzd_redemption_e2e.py
│   └── test_economy_compliance.py
│
├── governance/                  # Governance system tests
│   ├── __init__.py
│   ├── README.md
│   ├── test_proposals.py
│   ├── test_voting.py
│   ├── test_treasury.py
│   ├── test_council.py
│   └── test_emergency_powers.py
│
├── belizex/                     # DEX tests with Oracle guards
│   ├── test_oracle_guard_rejection.py
│   ├── test_oracle_rate_unavailable.py
│   ├── test_oracle_slippage_guard.py
│   └── test_wusdc_bbzd_guard.py
│
├── oracle/                      # Oracle pallet tests
│   └── test_exchange_rate.py
│
└── e2e/                         # End-to-end scenario tests
    ├── __init__.py
    ├── README.md
    └── test_scenarios_e2e.py    # Complete user journeys
```

## Test Infrastructure

### Fixtures (`conftest.py`)
- `blockchain_connection`: BelizeChain testnet RPC/WebSocket connection
- `alice_keypair`: Sudo account keypair (test authority)
- `bob_keypair`: Standard test user account
- `charlie_keypair`: Secondary test user account
- `submit_sudo_extrinsic`: Helper to submit sudo-protected extrinsics
- `query_storage`: Helper to query on-chain storage
- `wait_for_block`: Wait for block finalization

### Test Utilities
- Blockchain interaction helpers (submit extrinsic, query storage, events)
- Mock data generators (accounts, proposals, transactions)
- Assertion helpers (on-chain state verification)
- Test account management (keypair generation, balance funding)

## Running Tests

### Prerequisites
```bash
# Start BelizeChain testnet node
./target/release/belizechain-node --chain=testnet --tmp --alice

# In another terminal, run tests
cd tests
pytest -v
```

### Running Specific Test Categories
```bash
# All blockchain core pallet tests
pytest tests/blockchain/ -v

# Cross-pallet interaction tests
pytest tests/cross_pallet/ -v

# Economic system tests (DALLA/bBZD)
pytest tests/economic/ -v

# Governance system tests
pytest tests/governance/ -v

# DEX and Oracle tests
pytest tests/belizex/ -v
pytest tests/oracle/ -v

# End-to-end scenarios
pytest tests/e2e/ -v

# Specific test file
pytest tests/economic/test_bbzd_compliance.py -v

# Single test
pytest tests/economic/test_bbzd_compliance.py::TestBBZDCompliance::test_mint_bbzd_requires_kyc -v
```

### Test Markers
```bash
# Run only fast tests (skip slow integration tests)
pytest -m "not slow" -v

# Run only tests requiring blockchain connection
pytest -m "requires_blockchain" -v

# Run only economic system tests
pytest -m "economic" -v

# Run only governance tests
pytest -m "governance" -v
```

## Expected Test Coverage

### Minimum Coverage Requirements for Testnet Deployment
- **Core Pallets**: 90%+ coverage (all extrinsics and storage queries)
- **Cross-Pallet Interactions**: 85%+ coverage (critical integration paths)
- **Economic System**: 95%+ coverage (DALLA/bBZD mechanics, treasury)
- **Governance**: 90%+ coverage (proposals, voting, emergency powers)
- **End-to-End Scenarios**: 100% coverage (complete user workflows)

### Critical Test Scenarios (Must Pass Before Testnet)
1. **Economy**: DALLA transfers, bBZD minting/redemption, treasury operations
2. **Identity**: BelizeID registration, KYC verification, sanctions enforcement
3. **Governance**: Proposal creation, voting, council elections, JaguarMode
4. **Compliance**: KYC limits, sanction blocking, FSC oversight
5. **BelizeX**: Oracle-guarded trades, liquidity management, slippage protection
6. **LandLedger**: Property registration, title transfers, document proofs
7. **Cross-Pallet**: Economy+Compliance KYC enforcement, Identity+Governance voter verification

## Test Environment

### Testnet Configuration
- **Network**: BelizeChain Testnet
- **Chain Spec**: `testnet-spec.json`
- **Genesis Accounts**: Alice (sudo), Bob, Charlie, Dave, Eve, Ferdie
- **Initial Balances**: 1,000,000 DALLA per test account
- **RPC Endpoint**: `ws://127.0.0.1:9944` (local) or `wss://testnet.belizechain.org:443` (remote)

### Test Data
- **Test Accounts**: 6 pre-funded accounts (Alice through Ferdie)
- **KYC Levels**: Basic (1), Verified (2), Enhanced (3)
- **Account Types**: Citizen, Business, Tourism, Government
- **Test Merchants**: 5 verified merchants for cashback testing
- **Test Properties**: 10 sample land parcels in LandLedger

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: Integration Tests
on: [push, pull_request]
jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - Checkout code
      - Build BelizeChain node
      - Start testnet node
      - Run integration tests
      - Upload coverage report
```

### Pre-Deployment Checklist
- [ ] All integration tests pass (100%)
- [ ] Coverage meets minimum requirements (90%+)
- [ ] No critical or high severity bugs
- [ ] Performance benchmarks within acceptable range
- [ ] Security audit passed
- [ ] Documentation updated
- [ ] Testnet deployment plan reviewed

## Debugging Failed Tests

### Common Issues
1. **Node Not Running**: Ensure `./target/release/belizechain-node --chain=testnet --tmp --alice` is running
2. **Connection Refused**: Check RPC endpoint in `conftest.py` matches node
3. **Insufficient Balance**: Test accounts may need funding via faucet
4. **KYC Not Set**: Some tests require accounts to have KYC level set
5. **Timeout**: Increase `wait_for_block` timeout for slow test environments

### Debugging Commands
```bash
# Check node is running and responsive
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "system_health"}' http://localhost:9933

# View test logs
pytest tests/integration/ -v --log-cli-level=DEBUG

# Run single test with full output
pytest tests/integration/economic/test_bbzd_compliance.py::TestBBZDCompliance::test_mint_bbzd_requires_kyc -vvs

# Check blockchain events
pytest tests/integration/ -v --capture=no
```

## Contributing

### Adding New Tests
1. Follow existing test structure (arrange-act-assert pattern)
2. Use descriptive test names: `test_<action>_<expected_outcome>`
3. Add docstrings explaining what is being tested
4. Use appropriate pytest markers (`@pytest.mark.requires_blockchain`, etc.)
5. Ensure tests are idempotent and can run in any order
6. Clean up test state after each test (use fixtures with cleanup)

### Test Naming Conventions
- `test_<pallet>_<extrinsic>_<scenario>`: Individual pallet functionality
- `test_<pallet_a>_<pallet_b>_interaction`: Cross-pallet tests
- `test_e2e_<scenario>`: End-to-end workflow tests

## Resources

- [BelizeChain Documentation](https://docs.belizechain.org)
- [Substrate Testing Guide](https://docs.substrate.io/test/)
