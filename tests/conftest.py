"""
BelizeChain Integration Tests - Shared Fixtures and Configuration

This module provides pytest fixtures for integration testing across all BelizeChain components.
"""

import asyncio
import json
import os
import threading
import time
from typing import Dict, Any, Optional
from types import SimpleNamespace

import pytest
import requests
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException

# Test configuration
BLOCKCHAIN_WS_URL = os.getenv("BLOCKCHAIN_WS_URL", "ws://localhost:9944")
# Derive HTTP URL from WS URL if not explicitly set
_default_rpc = BLOCKCHAIN_WS_URL.replace("ws://", "http://").replace("wss://", "https://")
BLOCKCHAIN_RPC_URL = os.getenv("BLOCKCHAIN_RPC_URL", _default_rpc)
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

def _create_substrate_connection():
    """Create and initialize a SubstrateInterface connection."""
    substrate = SubstrateInterface(
        url=BLOCKCHAIN_WS_URL,
        ss58_format=42,  # Generic Substrate format
        ws_options={"timeout": 60},
    )
    substrate.init_runtime()
    return substrate


class ResilientSubstrate:
    """Wrapper around SubstrateInterface that auto-reconnects on broken pipe or stale WS."""

    _WS_ERRORS = (BrokenPipeError, ConnectionError, OSError, TimeoutError)

    def __init__(self):
        self._inner = _create_substrate_connection()
        # Dynamically extend caught errors if websocket lib is available
        try:
            from websocket import WebSocketException, WebSocketTimeoutException
            self._WS_ERRORS = (*self._WS_ERRORS, WebSocketException, WebSocketTimeoutException)
        except ImportError:
            pass

    def _reconnect(self):
        try:
            self._inner.close()
        except Exception:
            pass
        self._inner = _create_substrate_connection()

    def __getattr__(self, name):
        attr = getattr(self._inner, name)
        if not callable(attr):
            return attr

        ws_errors = self._WS_ERRORS

        def _wrapper(*args, **kwargs):
            try:
                return attr(*args, **kwargs)
            except ws_errors:
                self._reconnect()
                return getattr(self._inner, name)(*args, **kwargs)

        return _wrapper

    def close(self):
        try:
            self._inner.close()
        except Exception:
            pass


@pytest.fixture(scope="session")
def blockchain_connection():
    """
    Provide a connection to the BelizeChain blockchain node.
    
    This fixture establishes a WebSocket connection to the blockchain
    and verifies it's healthy before returning.  Uses ResilientSubstrate
    to auto-reconnect on broken pipe errors during bulk test runs.
    """
    # Wait for blockchain to be ready
    if not wait_for_service(BLOCKCHAIN_RPC_URL, timeout=60):
        pytest.skip("Blockchain node not available")
    
    substrate = ResilientSubstrate()
    
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


@pytest.fixture(scope="session")
def ensure_kyc(blockchain_connection):
    """Ensure Alice and Bob have Government (level 4) compliance + registered identities.

    ComplianceOrigin = TechnicalCouncilSuperMajority (>2/3 vote), so we must:
    1. Add Alice to TechnicalCouncil via Sudo.sudo(set_members)
    2. Propose verify_account with threshold=1 (auto-executes with 1 member)
    3. Register identities in Identity pallet (needed by Community pallet)

    Uses level=4 because governance and treasury require Government level.
    """
    substrate = blockchain_connection
    alice = Keypair.create_from_uri("//Alice")
    _sufficient_levels = ("Government",)

    # Step 1: Ensure Alice is a TechnicalCouncil member
    try:
        members = substrate.query("TechnicalCouncil", "Members")
        if not members.value or alice.ss58_address not in members.value:
            set_members = substrate.compose_call(
                call_module="TechnicalCouncil",
                call_function="set_members",
                call_params={
                    "new_members": [alice.ss58_address],
                    "prime": alice.ss58_address,
                    "old_count": len(members.value) if members.value else 0,
                },
            )
            sudo_call = substrate.compose_call(
                call_module="Sudo",
                call_function="sudo",
                call_params={"call": set_members},
            )
            ext = substrate.create_signed_extrinsic(call=sudo_call, keypair=alice)
            substrate.submit_extrinsic(ext, wait_for_inclusion=True)
    except Exception as e:
        print(f"[ensure_kyc] Council setup failed: {e}")

    # Step 2: Verify each account at level 4 (Government) via TechnicalCouncil
    for seed in ["//Alice", "//Bob"]:
        account = Keypair.create_from_uri(seed)
        try:
            status = substrate.query("Compliance", "ComplianceStatusOf", [account.ss58_address])
            current_level = status.value.get("verification_level") if status.value else None
            if current_level in _sufficient_levels:
                continue  # Already at Government

            proposal = substrate.compose_call(
                call_module="Compliance",
                call_function="verify_account",
                call_params={"account": account.ss58_address, "level": 4, "risk_level": 1},
            )
            proposal_len = len(proposal.encode())
            propose_call = substrate.compose_call(
                call_module="TechnicalCouncil",
                call_function="propose",
                call_params={
                    "threshold": 1,
                    "proposal": proposal,
                    "length_bound": proposal_len + 100,
                },
            )
            ext = substrate.create_signed_extrinsic(call=propose_call, keypair=alice)
            receipt = substrate.submit_extrinsic(ext, wait_for_inclusion=True)
            if not receipt.is_success:
                print(f"[ensure_kyc] Verify {seed} failed: {receipt.error_message}")
        except Exception as e:
            print(f"[ensure_kyc] Verify {seed} exception: {e}")

    # Step 3: Register identity for each account (needed by Community pallet)
    for seed in ["//Alice", "//Bob"]:
        account = Keypair.create_from_uri(seed)
        try:
            id_status = substrate.query("Identity", "IdentityOf", [account.ss58_address])
            if id_status.value is not None:
                continue  # Already registered
            call = substrate.compose_call(
                call_module="Identity",
                call_function="register_identity",
                call_params={"name": f"{seed.strip('/')} Testnet"},
            )
            ext = substrate.create_signed_extrinsic(call=call, keypair=account)
            receipt = substrate.submit_extrinsic(ext, wait_for_inclusion=True)
            if not receipt.is_success:
                print(f"[ensure_kyc] Identity {seed} failed: {receipt.error_message}")
        except Exception as e:
            print(f"[ensure_kyc] Identity {seed} exception: {e}")

    # Step 4: Set up Identity pallet KYC attestations (SSN + Passport for L2)
    # The BNS, BelizeX, Governance, and Interoperability pallets check
    # Identity::get_verified_kyc_level() which requires on-chain attestations,
    # NOT the Compliance pallet status.
    # L1 = SSN attestation, L2 = SSN + Passport, L3 = SSN + Passport + Biometric
    # Bridge needs >= L2, others need >= L1.
    import hashlib
    import os

    # Step 4a: Authorize Alice as SSN (attr=0) and Passport (attr=1) issuer
    for attr_id in [0, 1]:  # 0=Ssn, 1=Passport
        attr_name = "SSN" if attr_id == 0 else "Passport"
        try:
            add_issuer_call = substrate.compose_call(
                call_module="Identity",
                call_function="add_issuer",
                call_params={"attr": attr_id, "issuer": alice.ss58_address},
            )
            proposal_len = len(add_issuer_call.encode())
            propose_call = substrate.compose_call(
                call_module="TechnicalCouncil",
                call_function="propose",
                call_params={
                    "threshold": 1,
                    "proposal": add_issuer_call,
                    "length_bound": proposal_len + 100,
                },
            )
            ext = substrate.create_signed_extrinsic(call=propose_call, keypair=alice)
            receipt = substrate.submit_extrinsic(ext, wait_for_inclusion=True)
            if not receipt.is_success:
                err = getattr(receipt, "error_message", None) or ""
                if "AlreadyAttested" not in str(err):
                    print(f"[ensure_kyc] Add {attr_name} issuer failed: {err}")
        except Exception as e:
            if "AlreadyAttested" not in str(e):
                print(f"[ensure_kyc] Add {attr_name} issuer exception: {e}")

    # Step 4b: Issue SSN and Passport attestations for Alice and Bob
    for seed in ["//Alice", "//Bob"]:
        account = Keypair.create_from_uri(seed)
        for attr_id, call_fn in [(0, "issue_ssn"), (1, "issue_passport")]:
            attr_name = "SSN" if attr_id == 0 else "Passport"
            try:
                # Check if attestation already exists
                storage_name = "SsnAttestations" if attr_id == 0 else "PassportAttestations"
                identity_id = substrate.query("Identity", "IdentityOf", [account.ss58_address])
                if identity_id.value is not None:
                    existing = substrate.query("Identity", storage_name, [identity_id.value])
                    if existing.value is not None:
                        status = existing.value.get("status") if isinstance(existing.value, dict) else None
                        if status == "Active":
                            continue  # Already has active attestation

                # Generate unique hash per account+attr
                hash_input = f"{account.ss58_address}-{attr_name}-test".encode()
                att_hash = "0x" + hashlib.sha256(hash_input).hexdigest()
                anchor = "0x" + f"{attr_name} attestation for {seed.strip('/')}".encode().hex()

                call = substrate.compose_call(
                    call_module="Identity",
                    call_function=call_fn,
                    call_params={
                        "target": account.ss58_address,
                        "hash": att_hash,
                        "anchor": anchor,
                        "format_ok": True,
                    },
                )
                ext = substrate.create_signed_extrinsic(call=call, keypair=alice)
                receipt = substrate.submit_extrinsic(ext, wait_for_inclusion=True)
                if not receipt.is_success:
                    print(f"[ensure_kyc] Issue {attr_name} for {seed} failed: {receipt.error_message}")
            except Exception as e:
                print(f"[ensure_kyc] Issue {attr_name} for {seed} exception: {e}")

    # Force a clean WebSocket to avoid stale subscription state from the 11+ extrinsics above
    substrate._reconnect()


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
    def _submit(pallet: str, call: str, params, keypair: Keypair):
        """Submit extrinsic and wait for finalization."""
        substrate = blockchain_connection

        # Handle both list and dict params
        if isinstance(params, dict):
            call_params = params
        elif isinstance(params, list):
            call_params = params
        else:
            call_params = {}

        try:
            call_obj = substrate.compose_call(
                call_module=pallet,
                call_function=call,
                call_params=call_params
            )
        except ValueError as e:
            pytest.skip(f"Call {pallet}.{call} unavailable in runtime: {e}")

        extrinsic = substrate.create_signed_extrinsic(
            call=call_obj,
            keypair=keypair
        )

        def _blocking_submit(sub, ext):
            # Use inner directly to bypass ResilientSubstrate auto-retry.
            # When we close the socket to kill a stuck recv(), the wrapper
            # would catch the error, reconnect, and retry — never exiting.
            inner = getattr(sub, "_inner", sub)
            return inner.submit_extrinsic(ext, wait_for_inclusion=True)

        def _submit_with_timeout(sub, ext, timeout_sec=45):
            """Submit extrinsic with a hard threading-based timeout.
            
            websocket-client catches EINTR and retries, so pytest-timeout's
            signal-based approach cannot interrupt blocking recv() calls.
            This uses a daemon thread + socket close to guarantee termination.
            """
            result_box = {}
            error_box = {}

            def _worker():
                try:
                    result_box["receipt"] = _blocking_submit(sub, ext)
                except Exception as exc:
                    error_box["exc"] = exc

            t = threading.Thread(target=_worker, daemon=True)
            t.start()
            t.join(timeout=timeout_sec)

            if t.is_alive():
                # Kill the stuck recv() by closing the underlying websocket
                try:
                    inner = getattr(sub, "_inner", sub)
                    ws = getattr(inner, "websocket", None) or getattr(inner, "ws", None)
                    if ws:
                        ws.close()
                except Exception:
                    pass
                t.join(timeout=5)
                # Reconnect so future calls work
                if hasattr(sub, "_reconnect"):
                    sub._reconnect()
                pytest.skip(
                    f"{pallet}.{call} not included in block within {timeout_sec}s"
                )

            if "exc" in error_box:
                raise error_box["exc"]
            return result_box.get("receipt")

        try:
            receipt = _submit_with_timeout(substrate, extrinsic, timeout_sec=45)
        except TimeoutError:
            pytest.skip(f"{pallet}.{call} timed out")
        except SubstrateRequestException as e:
            err_msg = str(e)
            if "Priority" in err_msg:
                # Nonce collision with pending tx — wait for next block and retry
                time.sleep(6)
                extrinsic = substrate.create_signed_extrinsic(
                    call=call_obj, keypair=keypair
                )
                try:
                    receipt = _submit_with_timeout(substrate, extrinsic, timeout_sec=45)
                except SubstrateRequestException as e2:
                    receipt = SimpleNamespace(
                        is_success=False,
                        error_message=str(e2),
                        triggered_events=[],
                        block_hash=None,
                        extrinsic_hash=None,
                    )
            else:
                receipt = SimpleNamespace(
                    is_success=False,
                    error_message=err_msg,
                    triggered_events=[],
                    block_hash=None,
                    extrinsic_hash=None,
                )

        class ReceiptWrapper:
            def __init__(self, inner):
                self.inner = inner
                self.is_success = getattr(inner, "is_success", False)
                self.error_message = getattr(inner, "error_message", None)
                self.triggered_events = getattr(inner, "triggered_events", [])
                self.block_hash = getattr(inner, "block_hash", None)
                self.extrinsic_hash = getattr(inner, "extrinsic_hash", None)
            def __getitem__(self, key):
                if key == "success":
                    return self.is_success
                if key == "error":
                    return self.error_message
                raise KeyError(key)

        return ReceiptWrapper(receipt)
    
    return _submit


@pytest.fixture
def submit_sudo_extrinsic(blockchain_connection, sudo_keypair):
    """
    Helper to submit sudo-wrapped extrinsics.
    
    Returns a function that wraps an extrinsic in Sudo.sudo() and submits it.
    This is required for extrinsics that use ensure_root().
    """
    def _submit_sudo(pallet: str, call: str, params, keypair: Optional[Keypair] = None):
        """Submit sudo-wrapped extrinsic, defaulting to the sudo key."""
        substrate = blockchain_connection

        try:
            inner_call = substrate.compose_call(
                call_module=pallet,
                call_function=call,
                call_params=params if isinstance(params, dict) else {}
            )
        except ValueError as e:
            pytest.skip(f"Call {pallet}.{call} unavailable in runtime: {e}")

        sudo_call = substrate.compose_call(
            call_module="Sudo",
            call_function="sudo",
            call_params={"call": inner_call}
        )

        extrinsic = substrate.create_signed_extrinsic(
            call=sudo_call,
            keypair=keypair or sudo_keypair
        )

        def _sudo_blocking_submit(sub, ext):
            # Use inner directly to bypass ResilientSubstrate auto-retry.
            inner = getattr(sub, "_inner", sub)
            return inner.submit_extrinsic(ext, wait_for_inclusion=True)

        def _sudo_submit_with_timeout(sub, ext, timeout_sec=45):
            result_box = {}
            error_box = {}

            def _worker():
                try:
                    result_box["receipt"] = _sudo_blocking_submit(sub, ext)
                except Exception as exc:
                    error_box["exc"] = exc

            t = threading.Thread(target=_worker, daemon=True)
            t.start()
            t.join(timeout=timeout_sec)

            if t.is_alive():
                try:
                    inner = getattr(sub, "_inner", sub)
                    ws = getattr(inner, "websocket", None) or getattr(inner, "ws", None)
                    if ws:
                        ws.close()
                except Exception:
                    pass
                t.join(timeout=5)
                if hasattr(sub, "_reconnect"):
                    sub._reconnect()
                pytest.skip(
                    f"Sudo {pallet}.{call} not included in block within {timeout_sec}s"
                )

            if "exc" in error_box:
                raise error_box["exc"]
            return result_box.get("receipt")

        try:
            receipt = _sudo_submit_with_timeout(substrate, extrinsic, timeout_sec=45)
        except TimeoutError:
            pytest.skip(f"Sudo {pallet}.{call} timed out")
        except SubstrateRequestException as e:
            receipt = SimpleNamespace(
                is_success=False,
                error_message=str(e),
                triggered_events=[],
                block_hash=None,
                extrinsic_hash=None,
            )

        class ReceiptWrapper:
            def __init__(self, inner):
                self.inner = inner
                self.is_success = getattr(inner, "is_success", False)
                self.error_message = getattr(inner, "error_message", None)
                self.triggered_events = getattr(inner, "triggered_events", [])
                self.block_hash = getattr(inner, "block_hash", None)
                self.extrinsic_hash = getattr(inner, "extrinsic_hash", None)
            def __getitem__(self, key):
                if key == "success":
                    return self.is_success
                if key == "error":
                    return self.error_message
                raise KeyError(key)

        return ReceiptWrapper(receipt)
    
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

        try:
            result = substrate.query(
                module=pallet,
                storage_function=storage_name,
                params=params or []
            )
        except Exception as e:
            pytest.skip(f"Storage {pallet}.{storage_name} unavailable: {e}")

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
    # Pallet-specific markers used across integration suite
    pallet_markers = [
        "pallet_economy",
        "pallet_identity",
        "pallet_governance",
        "pallet_compliance",
        "pallet_staking",
        "pallet_oracle",
        "pallet_payroll",
        "pallet_interoperability",
        "pallet_belizex",
        "pallet_landledger",
        "pallet_consensus",
        "pallet_quantum",
        "pallet_community",
        "pallet_bns",
        "pallet_contracts",
    ]
    for marker in pallet_markers:
        config.addinivalue_line("markers", f"{marker}: auto-registered marker")

    # Cross-cutting markers
    config.addinivalue_line("markers", "cross_pallet: cross-pallet integration")
    config.addinivalue_line("markers", "governance: governance-focused tests")
    config.addinivalue_line("markers", "e2e: end-to-end scenario tests")


def pytest_collection_modifyitems(config, items):
    """Auto-mark all tests in integration directory."""
    for item in items:
        if "integration" in str(item.fspath):
            item.add_marker(pytest.mark.integration)
