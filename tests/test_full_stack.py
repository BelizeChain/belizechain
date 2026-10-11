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
import hashlib
import os
from pathlib import Path

# Import components
try:
    from nawal.server.aggregator import FederatedAggregator, ModelUpdate
    from nawal.storage.metrics_db import MetricsStore
    from nawal.blockchain.staking_connector import StakingConnector
    
    from kinich.core.quantum_node import QuantumNode, NodeConfig
    from kinich.queue.job_scheduler import QuantumJobQueue
    from kinich.blockchain.belizechain_adapter import BelizeChainAdapter
    
    from pakit.backends.ipfs_backend import IPFSBackend
    from pakit.blockchain.storage_proof_connector import StorageProofConnector
    from pakit.core.merkle_storage_proofs import build_proof_from_blocks
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
    
    # Submit the job to the on-chain Quantum pallet. Requires a running node —
    # skip rather than assert against a chain we cannot reach.
    adapter = BelizeChainAdapter(node_url=BLOCKCHAIN_URL)
    if not await adapter.connect():
        pytest.skip("BelizeChain node not reachable")

    tx_hash = await adapter.submit_quantum_job(
        job_id=job.job_id,
        backend="azure",
        circuit_hash=b"\xab\xcd" * 16,   # 32 bytes
        num_qubits=2,
        circuit_depth=2,
        num_shots=1024,
    )
    
    assert tx_hash, "Quantum job submission should return a tx hash"
    print(f"✅ Kinich → Blockchain Quantum job submitted: {tx_hash}")
    
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
    3. Proof submitted to the StorageProof pallet
    4. Blockchain stores proof on-chain
    """
    # Setup IPFS backend
    ipfs_backend = IPFSBackend(api_url=IPFS_URL)
    
    # Setup chain connector (mock mode — no live node required)
    connector = StorageProofConnector(node_url=BLOCKCHAIN_URL, mock_mode=True)
    await connector.connect()
    
    # Create test document
    test_data = b"Test land title document for property #12345"
    
    # Store the document
    content_id = hashlib.sha256(test_data).hexdigest()
    assert ipfs_backend.store(content_id, test_data), "Store should succeed"
    print(f"Stored: {content_id}")
    
    # Build a Merkle storage proof over the block
    built = build_proof_from_blocks(
        target_block_hash_hex=content_id,
        content_bytes=test_data,
        parent_block_hashes_hex=["0x" + "ab" * 32],
    )
    assert built is not None, "Proof build should succeed"
    onchain_content_id, proof = built

    # Submit proof to the StorageProof pallet
    result = await connector.submit_storage_zk_proof(
        content_id=onchain_content_id,
        zk_proof=proof.to_chain_args(),
        proof_type="merkle",
    )
    
    assert result, "Proof submission should succeed"
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
