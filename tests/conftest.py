"""
BelizeChain Integration Tests - Shared Fixtures and Configuration

This module provides pytest fixtures for integration testing across all BelizeChain components.
"""

import asyncio
import json
import os
import time
from typing import Dict, Any, Optional

import pytest
import requests
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException

# Test configuration
BLOCKCHAIN_RPC_URL = os.getenv("BLOCKCHAIN_RPC_URL", "http://localhost:9944")
BLOCKCHAIN_WS_URL = os.getenv("BLOCKCHAIN_WS_URL", "ws://localhost:9944")
IPFS_API_URL = os.getenv("IPFS_API_URL", "http://localhost:5001")
FL_SERVER_URL = os.getenv("FL_SERVER_URL", "http://localhost:8080")
QUANTUM_API_URL = os.getenv("QUANTUM_API_URL", "http://localhost:8081")
REDIS_URL = os.getenv("REDIS_URL", "redis://localhost:6379")
POSTGRES_URL = os.getenv("POSTGRES_URL", "postgresql://belizechain:belize_chain_secure_2025@localhost:5432/belizechain")

# Test accounts (well-known development keys)
TEST_ACCOUNTS = {
    "alice": {
        "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "seed": "//Alice",
        "role": "validator"
    },
    "bob": {
        "address": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "seed": "//Bob",
        "role": "citizen"
    },
    "charlie": {
        "address": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
        "seed": "//Charlie",
        "role": "hospital"
    },
    "dave": {
        "address": "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy",
        "seed": "//Dave",
        "role": "business"
    },
}


# ============================================================================
# Service Health Checks
# ============================================================================

def wait_for_service(url: str, timeout: int = 30, interval: int = 1) -> bool:
    """
    Wait for a service to become available.
    
    Args:
        url: Service URL to check
        timeout: Maximum time to wait in seconds
        interval: Time between checks in seconds
        
    Returns:
        True if service is available, False otherwise
    """
    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            response = requests.get(url, timeout=2)
            if response.status_code < 500:  # Any non-5xx response is acceptable
                return True
        except requests.RequestException:
            pass
        time.sleep(interval)
    return False


def check_blockchain_health(substrate: SubstrateInterface) -> bool:
    """
    Check if blockchain node is healthy and producing blocks.
    
    Args:
        substrate: SubstrateInterface instance
        
    Returns:
        True if healthy, False otherwise
    """
    try:
        health = substrate.rpc_request("system_health", [])
        return health["result"]["peers"] >= 0  # In dev mode, 0 peers is OK
    except Exception:
        return False


def check_ipfs_health() -> bool:
    """
    Check if IPFS daemon is running.
    
    Returns:
        True if healthy, False otherwise
    """
    try:
        response = requests.post(f"{IPFS_API_URL}/api/v0/version", timeout=2)
        return response.status_code == 200
    except requests.RequestException:
        return False


# ============================================================================
# Pytest Fixtures - Blockchain
# ============================================================================

@pytest.fixture(scope="session")
def blockchain_connection():
    """
    Provide a connection to the BelizeChain blockchain node.
    
    This fixture establishes a WebSocket connection to the blockchain
    and verifies it's healthy before returning.
    """
    # Wait for blockchain to be ready
    if not wait_for_service(BLOCKCHAIN_RPC_URL, timeout=60):
        pytest.skip("Blockchain node not available")
    
    # Create substrate interface
    substrate = SubstrateInterface(
        url=BLOCKCHAIN_WS_URL,
        ss58_format=42,  # Generic Substrate format
        type_registry_preset="substrate-node-template"
    )
    
    # Verify health
    if not check_blockchain_health(substrate):
        pytest.skip("Blockchain node not healthy")
    
    yield substrate
    
    # Cleanup
    substrate.close()


@pytest.fixture
def alice_keypair():
    """Provide Alice's keypair (validator account)."""
    return Keypair.create_from_uri(TEST_ACCOUNTS["alice"]["seed"])


@pytest.fixture
def bob_keypair():
    """Provide Bob's keypair (citizen account)."""
    return Keypair.create_from_uri(TEST_ACCOUNTS["bob"]["seed"])


@pytest.fixture
def charlie_keypair():
    """Provide Charlie's keypair (hospital account)."""
    return Keypair.create_from_uri(TEST_ACCOUNTS["charlie"]["seed"])


@pytest.fixture
def dave_keypair():
    """Provide Dave's keypair (business account)."""
    return Keypair.create_from_uri(TEST_ACCOUNTS["dave"]["seed"])


@pytest.fixture
def sudo_keypair():
    """Provide sudo keypair (Alice in dev mode has sudo privileges)."""
    return Keypair.create_from_uri("//Alice")


# ============================================================================
# Pytest Fixtures - IPFS
# ============================================================================

@pytest.fixture(scope="session")
def ipfs_client():
    """
    Provide an IPFS client for content storage and retrieval.
    """
    # Wait for IPFS to be ready
    if not wait_for_service(f"{IPFS_API_URL}/api/v0/version", timeout=30):
        pytest.skip("IPFS node not available")
    
    if not check_ipfs_health():
        pytest.skip("IPFS node not healthy")
    
    class IPFSClient:
        """Simple IPFS client wrapper."""
        
        def add(self, data: bytes) -> str:
            """Upload data to IPFS and return CID."""
            files = {"file": data}
            response = requests.post(
                f"{IPFS_API_URL}/api/v0/add",
                files=files,
                timeout=10
            )
            response.raise_for_status()
            return response.json()["Hash"]
        
        def cat(self, cid: str) -> bytes:
            """Retrieve data from IPFS by CID."""
            response = requests.post(
                f"{IPFS_API_URL}/api/v0/cat",
                params={"arg": cid},
                timeout=10
            )
            response.raise_for_status()
            return response.content
        
        def pin_add(self, cid: str) -> bool:
            """Pin content to prevent garbage collection."""
            response = requests.post(
                f"{IPFS_API_URL}/api/v0/pin/add",
                params={"arg": cid},
                timeout=10
            )
            return response.status_code == 200
    
    yield IPFSClient()


# ============================================================================
# Pytest Fixtures - Federated Learning
# ============================================================================

@pytest.fixture(scope="session")
def fl_server():
    """
    Provide connection to Federated Learning aggregator server.
    """
    # Wait for FL server to be ready
    if not wait_for_service(FL_SERVER_URL, timeout=30):
        pytest.skip("Federated Learning server not available")
    
    class FLClient:
        """Simple FL server client wrapper."""
        
        def submit_model(self, model_data: Dict[str, Any], participant_id: str) -> Dict[str, Any]:
            """Submit model update to FL server."""
            response = requests.post(
                f"{FL_SERVER_URL}/submit_model",
                json={
                    "participant_id": participant_id,
                    "model_data": model_data
                },
                timeout=30
            )
            response.raise_for_status()
            return response.json()
        
        def get_global_model(self) -> Dict[str, Any]:
            """Retrieve current global model."""
            response = requests.get(
                f"{FL_SERVER_URL}/global_model",
                timeout=10
            )
            response.raise_for_status()
            return response.json()
        
        def get_round_status(self) -> Dict[str, Any]:
            """Get current FL round status."""
            response = requests.get(
                f"{FL_SERVER_URL}/status",
                timeout=10
            )
            response.raise_for_status()
            return response.json()
    
    yield FLClient()


# ============================================================================
# Pytest Fixtures - Quantum Computing
# ============================================================================

@pytest.fixture(scope="session")
def quantum_service():
    """
    Provide connection to Quantum Computing service.
    """
    # Wait for quantum service to be ready
    if not wait_for_service(QUANTUM_API_URL, timeout=30):
        pytest.skip("Quantum computing service not available")
    
    class QuantumClient:
        """Simple quantum service client wrapper."""
        
        def submit_job(self, job_type: str, circuit: str, priority: int = 5) -> str:
            """Submit quantum job and return job ID."""
            response = requests.post(
                f"{QUANTUM_API_URL}/jobs",
                json={
                    "job_type": job_type,
                    "circuit": circuit,
                    "priority": priority,
                    "shots": 1024
                },
                timeout=10
            )
            response.raise_for_status()
            return response.json()["job_id"]
        
        def get_job_status(self, job_id: str) -> Dict[str, Any]:
            """Get quantum job status."""
            response = requests.get(
                f"{QUANTUM_API_URL}/jobs/{job_id}",
                timeout=10
            )
            response.raise_for_status()
            return response.json()
        
        def get_job_result(self, job_id: str) -> Dict[str, Any]:
            """Get quantum job result (waits for completion)."""
            max_wait = 60
            start_time = time.time()
            
            while time.time() - start_time < max_wait:
                status = self.get_job_status(job_id)
                if status["status"] == "COMPLETED":
                    return status["result"]
                elif status["status"] == "FAILED":
                    raise RuntimeError(f"Job failed: {status.get('error')}")
                time.sleep(2)
            
            raise TimeoutError(f"Job {job_id} did not complete within {max_wait}s")
    
    yield QuantumClient()


# ============================================================================
# Pytest Fixtures - Database
# ============================================================================

@pytest.fixture(scope="session")
def postgres_connection():
    """
    Provide PostgreSQL database connection.
    """
    try:
        import psycopg2
        conn = psycopg2.connect(POSTGRES_URL)
        yield conn
        conn.close()
    except ImportError:
        pytest.skip("psycopg2 not installed")
    except Exception as e:
        pytest.skip(f"PostgreSQL not available: {e}")


@pytest.fixture(scope="session")
def redis_connection():
    """
    Provide Redis cache connection.
    """
    try:
        import redis
        client = redis.from_url(REDIS_URL)
        client.ping()  # Verify connection
        yield client
        client.close()
    except ImportError:
        pytest.skip("redis-py not installed")
    except Exception as e:
        pytest.skip(f"Redis not available: {e}")


# ============================================================================
# Pytest Fixtures - Integration Helpers
# ============================================================================

@pytest.fixture
def wait_for_block(blockchain_connection):
    """
    Helper to wait for a new block to be produced.
    
    Returns a function that waits for N blocks.
    """
    def _wait(num_blocks: int = 1, timeout: int = 30) -> int:
        """Wait for N blocks to be produced."""
        substrate = blockchain_connection
        start_block = substrate.get_block_number(substrate.get_chain_head())
        target_block = start_block + num_blocks
        
        start_time = time.time()
        while time.time() - start_time < timeout:
            current_block = substrate.get_block_number(substrate.get_chain_head())
            if current_block >= target_block:
                return current_block
            time.sleep(1)
        
        raise TimeoutError(f"Waited {timeout}s for {num_blocks} blocks")
    
    return _wait


@pytest.fixture
def submit_extrinsic(blockchain_connection):
    """
    Helper to submit extrinsics to the blockchain.
    
    Returns a function that submits and waits for finalization.
    Accepts params as either list (positional) or dict (named).
    """
    def _submit(pallet: str, call: str, params, keypair: Keypair) -> Dict[str, Any]:
        """Submit extrinsic and wait for finalization."""
        substrate = blockchain_connection
        
        # Handle both list and dict params
        if isinstance(params, dict):
            # Convert dict to named params format
            call_params = params
        elif isinstance(params, list):
            # Use list as positional params
            call_params = params
        else:
            call_params = {}
        
        # Create call
        call_obj = substrate.compose_call(
            call_module=pallet,
            call_function=call,
            call_params=call_params
        )
        
        # Create and sign extrinsic
        extrinsic = substrate.create_signed_extrinsic(
            call=call_obj,
            keypair=keypair
        )
        
        # Submit and wait for finalization
        try:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
            
            # Extract error if extrinsic failed
            error_msg = None
            if not receipt.is_success:
                # Look for error events
                for event in receipt.triggered_events:
                    if event.event_module == "System" and event.event_name == "ExtrinsicFailed":
                        # Get the dispatch error
                        error_info = event.params
                        if error_info and len(error_info) > 0:
                            error_msg = str(error_info[0])
                        break
                # If no specific error found, try to get from error_message
                if not error_msg and hasattr(receipt, 'error_message'):
                    error_msg = str(receipt.error_message)
            
            return {
                "success": receipt.is_success,
                "block_hash": receipt.block_hash,
                "extrinsic_hash": receipt.extrinsic_hash,
                "events": receipt.triggered_events,
                "error": error_msg
            }
        except SubstrateRequestException as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    return _submit


@pytest.fixture
def submit_sudo_extrinsic(blockchain_connection, sudo_keypair):
    """
    Helper to submit sudo-wrapped extrinsics.
    
    Returns a function that wraps an extrinsic in Sudo.sudo() and submits it.
    This is required for extrinsics that use ensure_root().
    """
    def _submit_sudo(pallet: str, call: str, params) -> Dict[str, Any]:
        """Submit sudo-wrapped extrinsic."""
        substrate = blockchain_connection
        
        # Create the inner call
        inner_call = substrate.compose_call(
            call_module=pallet,
            call_function=call,
            call_params=params if isinstance(params, dict) else {}
        )
        
        # Wrap in Sudo.sudo
        sudo_call = substrate.compose_call(
            call_module="Sudo",
            call_function="sudo",
            call_params={"call": inner_call}
        )
        
        # Create and sign extrinsic
        extrinsic = substrate.create_signed_extrinsic(
            call=sudo_call,
            keypair=sudo_keypair
        )
        
        # Submit and wait for finalization
        try:
            receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
            
            # Extract error if extrinsic failed
            error_msg = None
            if not receipt.is_success:
                for event in receipt.triggered_events:
                    if event.event_module == "System" and event.event_name == "ExtrinsicFailed":
                        error_msg = str(event.params[0])
                        break
                if not error_msg and hasattr(receipt, 'error_message'):
                    error_msg = str(receipt.error_message)
            
            return {
                "success": receipt.is_success,
                "block_hash": receipt.block_hash,
                "extrinsic_hash": receipt.extrinsic_hash,
                "events": receipt.triggered_events,
                "error": error_msg
            }
        except SubstrateRequestException as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    return _submit_sudo


@pytest.fixture
def query_storage(blockchain_connection):
    """
    Helper to query blockchain storage.
    
    Returns a function that queries storage items.
    """
    def _query(pallet: str, storage_name: str, params: Optional[list] = None) -> Any:
        """Query blockchain storage."""
        substrate = blockchain_connection
        
        result = substrate.query(
            module=pallet,
            storage_function=storage_name,
            params=params or []
        )
        
        return result.value
    
    return _query


# ============================================================================
# Pytest Configuration
# ============================================================================

def pytest_configure(config):
    """Configure pytest for integration tests."""
    config.addinivalue_line(
        "markers", "integration: mark test as integration test"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow-running"
    )
    config.addinivalue_line(
        "markers", "requires_ipfs: mark test as requiring IPFS"
    )
    config.addinivalue_line(
        "markers", "requires_blockchain: mark test as requiring blockchain"
    )


def pytest_collection_modifyitems(config, items):
    """Auto-mark all tests in integration directory."""
    for item in items:
        if "integration" in str(item.fspath):
            item.add_marker(pytest.mark.integration)
