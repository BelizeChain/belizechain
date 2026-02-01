# 🎬 End-to-End Integration Tests

## Purpose
Full-stack scenario tests that validate complete workflows across all BelizeChain components.

## Architecture
```
┌─────────┐     ┌────────┐     ┌──────────┐     ┌────────────┐
│  Nawal  │ ──▶ │ Kinich │ ──▶ │  Pakit   │ ──▶ │ Blockchain │
│   AI    │     │Quantum │     │ Storage  │     │  (13 pallets)│
└─────────┘     └────────┘     └──────────┘     └────────────┘
     │               │               │                  │
     └───────────────┴───────────────┴──────────────────┘
                    Full E2E Flow
```

## Test Scenarios

### 1. Healthcare AI Workflow ✅ PASSING
**Flow**: Hospital FL → Model Aggregation → IPFS Storage → Blockchain Verification
```python
test_healthcare_ai_scenario()
```
**Steps**:
1. Hospital validator enrolls with KYC
2. FL training task assigned
3. Model delta submitted with privacy guarantees
4. Model aggregated and stored on IPFS
5. Model hash recorded on blockchain
6. PoUW rewards distributed based on quality

**Duration**: ~2 minutes  
**Status**: ✅ PASSING (11/11 assertions)

### 2. Hurricane Response Workflow 🚧 SKIPPED
**Flow**: Quantum Simulation → AI Prediction → Governance → Emergency Funds
```python
test_hurricane_response_scenario()
```
**Steps**:
1. Emergency quantum job submitted (priority=10)
2. Azure Quantum executes hurricane simulation
3. Results trigger AI prediction model
4. Governance fast-tracks emergency proposal
5. Treasury releases emergency funds
6. SMS/broadcast notification sent

**Duration**: ~4 minutes  
**Status**: 🚧 SKIPPED (requires full infrastructure)

### 3. Tourism Incentive Workflow 📝 TO BE IMPLEMENTED
**Flow**: Payment → Smart Contract → Cashback → Analytics
```python
test_tourism_incentive_scenario()
```
**Steps**:
1. Tourist makes payment at certified merchant
2. Oracle verifies merchant eligibility
3. Smart contract calculates cashback (5-8%)
4. bBZD cashback distributed automatically
5. Transaction recorded for analytics
6. Tourism dashboard updated

**Duration**: ~1 minute  
**Status**: 📝 NOT YET IMPLEMENTED

## Running Tests

```bash
# All E2E scenarios
pytest tests/integration/e2e/ -v

# Specific scenario
pytest tests/integration/e2e/test_scenarios_e2e.py::test_healthcare_ai_scenario -v

# With detailed output
pytest tests/integration/e2e/ -v -s

# Skip slow scenarios
pytest tests/integration/e2e/ -v -m "not slow"
```

## Prerequisites
```bash
# 1. Start ALL services
docker compose -f infra/docker-compose.yml up -d

# OR start manually:

# Blockchain
./target/release/belizechain-node --chain=testnet --tmp --alice

# IPFS
ipfs daemon

# Nawal FL server
cd nawal && python -m nawal.orchestrator server

# Kinich quantum node
cd kinich && python -m kinich.core.quantum_node

# Redis (for caching)
redis-server

# PostgreSQL (for metadata)
# Already running via docker-compose

# 2. Activate Python environment
source .venv/bin/activate
```

## Test Files
- `test_scenarios_e2e.py` - Main E2E scenarios (2 tests, 1 passing)

## Success Criteria

### Per Scenario
- ✅ All components interact correctly
- ✅ Data flows from start to finish
- ✅ Error handling works at each stage
- ✅ Performance meets expectations
- ✅ Events emitted correctly
- ✅ Storage proofs verified

### Overall
- ✅ At least 2/3 scenarios passing
- ✅ Healthcare workflow working (critical)
- ✅ No component crashes
- ✅ All services stay healthy during tests
- ✅ Logs show correct execution flow

## Troubleshooting

### "Connection refused" errors
```bash
# Check all services
docker compose -f infra/docker-compose.yml ps

# Check specific service
docker logs belizechain-node
docker logs belizechain-fl-aggregator
```

### Tests hang indefinitely
```bash
# Use timeout
pytest tests/integration/e2e/ -v --timeout=120

# Check for deadlocks
docker logs belizechain-fl-aggregator --tail=50
```

### IPFS timeouts
```bash
# Restart IPFS
docker restart belizechain-ipfs
curl http://localhost:5001/api/v0/version
```

## CI/CD Integration
E2E tests run on:
- ✅ Pull request to main
- ✅ Daily nightly builds
- ✅ Pre-release validation
- ❌ NOT on every commit (too slow)
