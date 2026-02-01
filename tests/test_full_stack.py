"""
BelizeChain Full Stack Integration Tests

Tests the complete end-to-end flow from Python services to blockchain.

Test Coverage:
- Nawal federated learning → Staking pallet (PoUW rewards)
- Kinich quantum jobs → Consensus pallet (PQW proofs)
- Pakit storage → LandLedger pallet (document proofs)

Requirements:
- Running BelizeChain node (ws://localhost:9944)
- Redis server (localhost:6379)
- IPFS daemon (localhost:5001)
- PostgreSQL/Cosmos DB (for metrics)

Author: BelizeChain Team
Date: January 2026
"""

import pytest
import asyncio
import os
from pathlib import Path

# Import components
try:
    from nawal.server.aggregator import FederatedAggregator, ModelUpdate
    from nawal.storage.metrics_db import MetricsStore
    from nawal.blockchain.staking_connector import StakingConnector
    
    from kinich.core.quantum_node import QuantumNode, NodeConfig
    from kinich.queue.job_scheduler import QuantumJobQueue
    from kinich.blockchain.consensus_connector import ConsensusConnector
    
    from pakit.storage.proof_manager import ProofManager
    from pakit.storage.ipfs_backend import IPFSBackend
except ImportError as e:
    pytest.skip(f"Missing dependencies: {e}", allow_module_level=True)


# =============================================================================
# Test Configuration
# =============================================================================

BLOCKCHAIN_URL = os.getenv("BLOCKCHAIN_URL", "ws://127.0.0.1:9944")
REDIS_URL = os.getenv("REDIS_URL", "redis://localhost:6379")
IPFS_URL = os.getenv("IPFS_URL", "http://127.0.0.1:5001")
COSMOS_DB_ENDPOINT = os.getenv("COSMOS_DB_ENDPOINT", "")
COSMOS_DB_KEY = os.getenv("COSMOS_DB_KEY", "")


@pytest.fixture
async def blockchain_connection():
    """Setup blockchain connection."""
    # In production, use polkadot-py or subxt
    # For now, mock connection
    class MockBlockchain:
        async def connect(self):
            pass
        
        async def disconnect(self):
            pass
        
        async def submit_extrinsic(self, pallet: str, method: str, args: list):
            print(f"Mock extrinsic: {pallet}.{method}({args})")
            return {"success": True, "hash": "0x1234..."}
    
    blockchain = MockBlockchain()
    await blockchain.connect()
    yield blockchain
    await blockchain.disconnect()


# =============================================================================
# Nawal Integration Tests
# =============================================================================

@pytest.mark.asyncio
async def test_nawal_to_blockchain_pouw_flow(blockchain_connection):
    """
    Test federated learning → PoUW rewards flow.
    
    Flow:
    1. Client trains model locally
    2. Submits update to aggregator
    3. Aggregator calculates fitness score
    4. Aggregator logs metrics to Cosmos DB
    5. Aggregator submits PoUW report to blockchain
    6. Staking pallet rewards validator
    """
    # Setup metrics store (use in-memory if no Cosmos DB)
    metrics_store = MetricsStore(
        endpoint=COSMOS_DB_ENDPOINT or "memory",
        key=COSMOS_DB_KEY or "test_key"
    )
    await metrics_store.initialize()
    
    # Setup aggregator with metrics
    aggregator = FederatedAggregator(
        min_participants=1,
        metrics_store=metrics_store
    )
    
    # Create mock model weights
    import torch
    mock_weights = {
        "layer1.weight": torch.randn(10, 5),
        "layer1.bias": torch.randn(10),
    }
    
    # Create mock genome
    from nawal.genome import Genome
    genome = Genome(genome_id="test-genome-001", generation=1)
    aggregator.set_genome(genome, mock_weights)
    
    # Simulate client update
    update = ModelUpdate(
        participant_id="alice",
        genome_id="test-genome-001",
        round_number=0,
        weights=mock_weights,
        samples_trained=1000,
        training_time=5.0,
        quality_score=85.0,
        timeliness_score=90.0,
        honesty_score=95.0,
        fitness_score=90.0,
    )
    
    # Submit update
    success = await aggregator.submit_update(update)
    assert success, "Update should be accepted"
    
    # Wait for aggregation
    await asyncio.sleep(2)
    
    # Verify metrics were logged
    stats = aggregator.get_statistics()
    assert stats["total_rounds"] >= 1
    assert stats["total_samples"] == 1000
    
    # Simulate blockchain submission (in production, use StakingConnector)
    staking_connector = StakingConnector(blockchain_url=BLOCKCHAIN_URL)
    
    # Submit PoUW report
    result = await staking_connector.submit_pouw_report(
        validator_address="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        quality_score=update.quality_score,
        timeliness_score=update.timeliness_score,
        honesty_score=update.honesty_score,
        fitness_score=update.fitness_score,
    )
    
    assert result["success"], "PoUW report submission should succeed"
    print(f"✅ Nawal → Blockchain PoUW flow complete: {result}")


# =============================================================================
# Kinich Integration Tests
# =============================================================================

@pytest.mark.asyncio
async def test_kinich_to_blockchain_pqw_flow(blockchain_connection):
    """
    Test quantum job → PQW proof flow.
    
    Flow:
    1. User submits quantum job
    2. Job added to Redis priority queue
    3. Quantum node dequeues and executes
    4. Results submitted to blockchain
    5. Consensus pallet validates PQW proof
    6. Validator receives reward
    """
    # Setup Redis job queue
    job_queue = QuantumJobQueue()
    await job_queue.connect()
    
    # Setup quantum node with queue
    config = NodeConfig(
        node_id="test-node-001",
        submit_results_to_chain=False,  # Mock for testing
    )
    node = QuantumNode(config=config, job_queue=job_queue)
    await node.start()
    
    # Create test quantum job
    from kinich.core.jobs import QuantumJob, JobPriority
    
    job = QuantumJob(
        job_id="job-001",
        circuit_qasm="OPENQASM 2.0; qreg q[2]; h q[0]; cx q[0],q[1];",
        backend="azure",
        priority=JobPriority.HIGH,
        user_id="test-user",
        shots=1024,
    )
    
    # Submit job to queue
    await job_queue.enqueue(job)
    
    # Dequeue job
    dequeued_job = await job_queue.dequeue(backend="azure", timeout=5)
    assert dequeued_job is not None, "Job should be dequeued"
    assert dequeued_job.job_id == job.job_id
    
    # Simulate job execution
    result = {
        "counts": {"00": 512, "11": 512},
        "shots": 1024,
        "execution_time": 2.5,
    }
    
    # Mark job complete
    await job_queue.complete_job(job.job_id, result)
    
    # Get job status
    status = await job_queue.get_job_status(job.job_id)
    assert status["status"] == "completed"
    
    # Simulate PQW proof submission (in production, use ConsensusConnector)
    consensus_connector = ConsensusConnector(blockchain_url=BLOCKCHAIN_URL)
    
    pqw_result = await consensus_connector.submit_quantum_work(
        validator_address="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        job_id=job.job_id,
        circuit_hash="0xabcd1234...",
        result_hash="0xef567890...",
        execution_time=2.5,
    )
    
    assert pqw_result["success"], "PQW submission should succeed"
    print(f"✅ Kinich → Blockchain PQW flow complete: {pqw_result}")
    
    # Cleanup
    await node.stop()
    await job_queue.disconnect()


# =============================================================================
# Pakit Integration Tests
# =============================================================================

@pytest.mark.asyncio
async def test_pakit_to_blockchain_proof_flow(blockchain_connection):
    """
    Test storage upload → blockchain proof flow.
    
    Flow:
    1. User uploads document to IPFS
    2. Pakit generates storage proof
    3. Proof submitted to LandLedger pallet
    4. Blockchain stores proof on-chain
    """
    # Setup IPFS backend
    ipfs_backend = IPFSBackend(api_url=IPFS_URL)
    
    # Setup proof manager
    proof_manager = ProofManager(blockchain_url=BLOCKCHAIN_URL)
    
    # Create test document
    test_data = b"Test land title document for property #12345"
    
    # Upload to IPFS
    cid = await ipfs_backend.upload(test_data)
    assert cid is not None, "Upload should return CID"
    print(f"Uploaded to IPFS: {cid}")
    
    # Generate proof
    proof = await proof_manager.generate_proof(
        cid=cid,
        document_type="land_title",
        owner_address="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        metadata={
            "property_id": "12345",
            "location": "Belize City",
            "size_acres": "2.5",
        }
    )
    
    assert proof["cid"] == cid
    assert "merkle_root" in proof
    
    # Submit proof to blockchain
    result = await proof_manager.register_proof(
        proof_id=proof["proof_id"],
        cid=cid,
        merkle_root=proof["merkle_root"],
        owner_address="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    )
    
    assert result["success"], "Proof registration should succeed"
    print(f"✅ Pakit → Blockchain proof flow complete: {result}")


# =============================================================================
# Full Stack Test
# =============================================================================

@pytest.mark.asyncio
async def test_full_stack_integration(blockchain_connection):
    """
    Test complete multi-component flow.
    
    Simulates:
    - Validator running Nawal FL client
    - Validator running Kinich quantum node
    - User uploading document via Pakit
    - All components reporting to blockchain
    """
    print("\n" + "="*70)
    print("FULL STACK INTEGRATION TEST")
    print("="*70)
    
    # Run all flows in parallel
    results = await asyncio.gather(
        test_nawal_to_blockchain_pouw_flow(blockchain_connection),
        test_kinich_to_blockchain_pqw_flow(blockchain_connection),
        test_pakit_to_blockchain_proof_flow(blockchain_connection),
        return_exceptions=True
    )
    
    # Check for failures
    failures = [r for r in results if isinstance(r, Exception)]
    if failures:
        print(f"❌ {len(failures)} tests failed:")
        for failure in failures:
            print(f"  - {failure}")
        pytest.fail("Integration tests failed")
    else:
        print("✅ All integration tests passed!")
        print("="*70)


# =============================================================================
# Run Tests
# =============================================================================

if __name__ == "__main__":
    # Run with: python -m pytest tests/integration/test_full_stack.py -v
    pytest.main([__file__, "-v", "-s"])
