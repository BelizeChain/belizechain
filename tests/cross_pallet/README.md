# 🔗 Cross-Pallet Integration Tests

## Overview

Tests for interactions between BelizeChain pallets to ensure cohesive system behavior:
- **Economy + Compliance**: KYC enforcement on financial transactions
- **Identity + Governance**: Verified voter requirements
- **Staking + Oracle**: Validator reputation and data feeds
- **BelizeX + Oracle**: Exchange rate guards and slippage protection
- **LandLedger + Identity**: Property ownership verification

## Test Categories

### 1. Economy + Compliance (`test_economy_compliance.py`)
- `test_transfer_blocked_without_kyc` - Transfer fails for non-KYC accounts
- `test_transfer_blocked_for_sanctioned` - Sanctioned accounts cannot transact
- `test_daily_limit_enforced_by_kyc_level` - Account limits based on KYC tier
- `test_large_transaction_triggers_fsc_alert` - >10K DALLA flagged automatically
- `test_business_account_higher_limits` - Business accounts have 100K limit vs 25K citizen

### 2. Identity + Governance (`test_identity_governance.py`)
- `test_voting_requires_verified_identity` - Only KYC Level 2+ can vote
- `test_council_requires_enhanced_kyc` - Council candidates need Level 3 KYC
- `test_proposal_creation_requires_basic_kyc` - Proposals need Level 1 minimum
- `test_delegated_voting_verifies_both_parties` - Delegator and delegate must have KYC

### 3. Staking + Oracle (`test_staking_oracle.py`)
- `test_validator_provides_price_feed` - Validators submit Oracle data
- `test_oracle_quality_affects_rewards` - Accurate data = higher staking rewards
- `test_byzantine_oracle_data_slashes_validator` - Bad data triggers slashing
- `test_validator_reputation_tracked` - Oracle performance tracked on-chain

### 4. BelizeX + Oracle (`test_belizex_oracle.py`)
- `test_trade_rejected_on_oracle_deviation` - >5% deviation blocks trade
- `test_oracle_unavailable_blocks_guarded_trades` - No Oracle = trade fails
- `test_oracle_pause_affects_all_trading_pairs` - Paused Oracle stops DEX
- `test_manual_rate_update_after_oracle_failure` - Governance can override

## Running Tests

```bash
# All cross-pallet tests
pytest tests/integration/cross_pallet/ -v

# Specific interaction
pytest tests/integration/cross_pallet/test_economy_compliance.py -v

# Single test
pytest tests/integration/cross_pallet/test_identity_governance.py::TestIdentityGovernance::test_voting_requires_verified_identity -v
```

## Test Data

### Test Scenarios
- **Sanctioned Account**: Bob sanctioned by compliance pallet
- **Non-KYC Account**: Charlie has no KYC verification
- **Level 1 KYC**: Dave has basic KYC (view-only)
- **Level 2 KYC**: Eve has verified KYC (can transact <10K/day)
- **Level 3 KYC**: Ferdie has enhanced KYC (unlimited, can be validator)

## Success Criteria

- ✅ 20+ cross-pallet integration tests
- ✅ All critical interaction paths tested
- ✅ Compliance enforcement verified
- ✅ Oracle integration fully tested
- ✅ No pallet operates in isolation incorrectly

## Resources

- [BelizeChain Architecture Guide](https://docs.belizechain.org/architecture/)
- [Cross-Pallet Communication](https://docs.substrate.io/reference/how-to-guides/pallet-design/use-loose-coupling/)
