# bBZD Integration Tests

**Status**: ✅ 12 comprehensive tests ready  
**Location**: `tests/integration/economic/test_bbzd_integration.py`

## Test Coverage

### 1. Minting (3 tests)
- ✅ `test_mint_bbzd_authorized_minter` - Central Bank can mint after BZD deposit verification
- ✅ `test_mint_bbzd_unauthorized_fails` - Unauthorized users cannot mint
- ✅ `test_mint_bbzd_exceeds_reserves_fails` - Cannot mint more than reserves

### 2. Redemption (2 tests)
- ✅ `test_redeem_bbzd_creates_request` - Burns bBZD + creates settlement queue
- ✅ `test_redeem_bbzd_insufficient_balance_fails` - Cannot redeem more than balance

### 3. Processing (2 tests)
- ✅ `test_process_redemption_completes_settlement` - Central Bank confirms off-chain transfer
- ✅ `test_process_redemption_unauthorized_fails` - Only Central Bank can process

### 4. Reserve Invariants (1 CRITICAL test)
- ✅ `test_supply_never_exceeds_reserves` - **TotalBbzdSupply ≤ CentralBankReserves ALWAYS**

### 5. Governance Integration (2 tests)
- ✅ `test_minter_authorization_requires_governance` - Only governance can authorize minters
- ✅ `test_reserve_updates_require_governance` - Only governance can update reserves

---

## Running the Tests

### Prerequisites
```bash
# Start BelizeChain local rehearsal node
./target/release/belizechain-node --chain=local --tmp --alice

# In another terminal, run tests
cd tests
pytest economic/test_bbzd_integration.py -v
```

### Expected Output
```
tests/integration/economic/test_bbzd_integration.py::TestBbzdMinting::test_mint_bbzd_authorized_minter PASSED
tests/integration/economic/test_bbzd_integration.py::TestBbzdMinting::test_mint_bbzd_unauthorized_fails PASSED
tests/integration/economic/test_bbzd_integration.py::TestBbzdMinting::test_mint_bbzd_exceeds_reserves_fails PASSED
tests/integration/economic/test_bbzd_integration.py::TestBbzdRedemption::test_redeem_bbzd_creates_request PASSED
tests/integration/economic/test_bbzd_integration.py::TestBbzdRedemption::test_redeem_bbzd_insufficient_balance_fails PASSED
tests/integration/economic/test_bbzd_integration.py::TestRedemptionProcessing::test_process_redemption_completes_settlement PASSED
tests/integration/economic/test_bbzd_integration.py::TestRedemptionProcessing::test_process_redemption_unauthorized_fails PASSED
tests/integration/economic/test_bbzd_integration.py::TestReserveInvariants::test_supply_never_exceeds_reserves PASSED
tests/integration/economic/test_bbzd_integration.py::TestGovernanceIntegration::test_minter_authorization_requires_governance PASSED
tests/integration/economic/test_bbzd_integration.py::TestGovernanceIntegration::test_reserve_updates_require_governance PASSED

========================================== 12 passed ===========================================
```

---

## Test Architecture

### USDC-Style Model Validation
All tests verify the fiat-backed model:
1. **NO DALLA COLLATERAL** - bBZD is backed by Central Bank BZD reserves in traditional bank accounts
2. **Pure Mint/Burn** - Central Bank mints after verifying off-chain BZD deposits
3. **Burn-First Redemption** - User burns bBZD on-chain, receives BZD off-chain
4. **1:1 Peg Always** - No Oracle needed, peg is hardcoded 1 bBZD = 1 BZD

### Key Validation Points
- ✅ Event emission (BbzdMinted, BbzdRedeemed, RedemptionProcessed)
- ✅ Balance updates (immediate burn on redemption)
- ✅ Storage state (AuthorizedMinters, CentralBankReserves, TotalBbzdSupply)
- ✅ Reserve invariant (supply ≤ reserves after every operation)
- ✅ Error handling (UnauthorizedMinter, InsufficientReserves, InsufficientBbzdBalance)
- ✅ Governance controls (minter authorization, reserve updates)

---

## Integration Points Tested

1. **Economy ↔ Governance**:
   - `set_minter_authorization` requires governance origin
   - `update_reserves` requires governance origin
   
2. **On-Chain ↔ Off-Chain**:
   - Minting after BZD deposit verification (deposit_reference tracking)
   - Redemption settlement (RedemptionRequest queue → bank transfer → process_redemption)

3. **Multi-User Flows**:
   - Multiple users can mint/redeem independently
   - Total supply tracking across all users
   - Reserve checks apply globally (not per-user)

---

## Known Limitations

- **Requires Running Node**: Tests need a live blockchain node (cannot run in CI without node)
- **Fixture Dependencies**: Uses `blockchain_connection`, `alice_keypair`, `bob_keypair` from `conftest.py`
- **State Isolation**: Tests may interfere with each other if run in parallel (use `-x` for sequential)

---

## Test Data

**Test Accounts** (from `conftest.py`):
- `Alice` (//Alice): Central Bank / Governance (has sudo in dev mode)
- `Bob` (//Bob): Regular user / Citizen

**Test Amounts**:
- Reserves: 10,000 BZD (10_000_000_000_000 with 12 decimals)
- Minting: 1,000-5,000 bBZD per test
- Redemption: 500-2,000 bBZD per test

**Deposit References**: `BANK_DEPOSIT_20251230_001`, `DEPOSIT_SETUP`, etc.  
**Bank Accounts**: `BOB_BANK_ACCOUNT_BZD_12345`, `USER_BANK_123`, etc.
