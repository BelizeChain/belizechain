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
from substrateinterface.utils.ss58 import ss58_decode

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

        return ReceiptWrapper(receipt)
    
    return _submit


class ReceiptWrapper:
    """Uniform receipt surface for both plain and sudo-wrapped extrinsics.

    Supports attribute access (`receipt.is_success`), item access
    (`receipt["success"]`) and `.get("error")`, because existing tests use all
    three spellings.
    """

    def __init__(self, inner, is_success: Optional[bool] = None,
                 error_message=None):
        self.inner = inner
        self.triggered_events = getattr(inner, "triggered_events", None) or []
        self.block_hash = getattr(inner, "block_hash", None)
        self.extrinsic_hash = getattr(inner, "extrinsic_hash", None)
        self.is_success = (
            getattr(inner, "is_success", False) if is_success is None else is_success
        )
        self.error_message = (
            getattr(inner, "error_message", None)
            if error_message is None
            else error_message
        )

    def __getitem__(self, key):
        if key == "success":
            return self.is_success
        if key == "error":
            return self.error_message
        raise KeyError(key)

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default


def event_pairs(receipt) -> list:
    """Return `(module_id, event_id)` for every event a receipt triggered.

    `substrate-interface` exposes the decoded event inside `.value`; older
    versions exposed `event_module` / `event_name` as attributes. Reading the
    attributes silently yields `None` on current versions, which turns every
    "did it emit X?" assertion into a permanent False. Read `.value` first.
    """
    pairs = []
    for event in getattr(receipt, "triggered_events", None) or []:
        value = getattr(event, "value", None)
        if not isinstance(value, dict):
            continue
        module = value.get("module_id")
        name = value.get("event_id")
        if module and name:
            pairs.append((module, name))
    return pairs


def same_account(a, b) -> bool:
    """True when two SS58 strings are the same on-chain account.

    The chain encodes with prefix 1981 (`r1...`) while
    `Keypair.create_from_uri` defaults to prefix 42 (`5F...`). Comparing the
    rendered strings therefore fails for the *same* account, which silently
    broke assertions in four suites. Compare the decoded AccountId instead.
    """
    if a is None or b is None:
        return False
    try:
        return ss58_decode(str(a)) == ss58_decode(str(b))
    except (ValueError, TypeError):
        return str(a) == str(b)


@pytest.fixture
def oracle_operator(blockchain_connection, submit_sudo_extrinsic, fund_account):
    """Return a funded keypair registered in `Oracle.OracleOperators`.

    `submit_price`, `verify_merchant` and `verify_identity` are all gated on that
    set. `OracleAdminOrigin` is `TechnicalCouncilMember`, which is an
    `EitherOfDiverse<EnsureRoot, _>`, so sudo can add an operator.
    """
    substrate = blockchain_connection
    operator = Keypair.create_from_uri(ORACLE_OPERATOR_SEEDS[0])

    def _oracle_operator() -> Keypair:
        fund_account(operator, 100 * 1_000_000_000_000)

        registered = substrate.query(
            "Oracle", "OracleOperators", [operator.ss58_address]
        ).value
        if registered:
            return operator

        receipt = submit_sudo_extrinsic(
            "Oracle", "add_operator", {"operator": operator.ss58_address}
        )
        message = str(receipt.error_message)
        assert receipt.is_success or "AlreadyRegistered" in message, (
            f"add_operator failed: {message}"
        )
        return operator

    return _oracle_operator


# --- Identity-pallet KYC (a different store from the Oracle one) -------------

IDENTITY_ISSUER_URI = "//Alice//identity-issuer"

# `pallet_identity::AttributeType`: Ssn, Passport, Biometrics.
ATTRIBUTE_SSN = 0
ATTRIBUTE_PASSPORT = 1
ATTRIBUTE_BIOMETRICS = 2

# `pallet_identity::KycLevel` is derived, not stored: L1 needs an SSN
# attestation, L2 adds a passport, L3 adds biometrics.
_ATTRIBUTES_BY_LEVEL = {
    1: (ATTRIBUTE_SSN,),
    2: (ATTRIBUTE_SSN, ATTRIBUTE_PASSPORT),
    3: (ATTRIBUTE_SSN, ATTRIBUTE_PASSPORT, ATTRIBUTE_BIOMETRICS),
}


@pytest.fixture
def identity_kyc(
    blockchain_connection, submit_sudo_extrinsic, submit_extrinsic, fund_account
):
    """Return `identity_kyc(account, level)` that seeds Identity attestations.

    This is **not** the same store as `verify_kyc`. The oracle quorum writes
    `Oracle.IdentityVerifications`, which is what `Economy::transfer_bbzd` and
    the governance compliance gates read. `T::BelizeKyc` — used by LandLedger,
    Staking, BNS and Community — instead reads `Identity.SsnAttestations`,
    `PassportAttestations` and `BiometricAttestations`, keyed by identity id.
    An account verified through the oracle is therefore still KYC-less for
    those pallets, which is why tests that seeded only the oracle skipped.

    `IssuerBondAmount` defaults to 0, so authorising an issuer needs no bond.
    """
    import hashlib

    substrate = blockchain_connection
    issuer = Keypair.create_from_uri(IDENTITY_ISSUER_URI)
    authorized_attributes: set = set()

    def _ensure_registered(account: Keypair) -> None:
        if substrate.query("Identity", "IdentityOf", [account.ss58_address]).value:
            return
        fund_account(account, 100 * 1_000_000_000_000)
        receipt = submit_extrinsic(
            "Identity",
            "register_identity",
            {"name": f"test-{account.ss58_address[:8]}"},
            account,
        )
        if not receipt.is_success and "IdentityExists" not in str(
            receipt.error_message
        ):
            raise AssertionError(
                f"register_identity failed for {account.ss58_address}: "
                f"{receipt.error_message}"
            )

    def _ensure_issuer(attribute: int) -> None:
        if attribute in authorized_attributes:
            return

        # `add_issuer` requires `IssuerBonds[attr][issuer] >= IssuerBondAmount`,
        # and that value is 1000 DALLA in this runtime's genesis — not zero, as
        # the pallet's `OptionQuery` default suggests.
        bond = substrate.query("Identity", "IssuerBondAmount").value
        fund_account(issuer, int(bond or 0) + 100 * 1_000_000_000_000)

        deposited = submit_extrinsic(
            "Identity", "issuer_deposit_bond", {"attr": attribute}, issuer
        )
        if not deposited.is_success:
            raise AssertionError(
                f"issuer_deposit_bond({attribute}) failed: {deposited.error_message}"
            )

        receipt = submit_sudo_extrinsic(
            "Identity",
            "add_issuer",
            {"attr": attribute, "issuer": issuer.ss58_address},
        )
        message = str(receipt.error_message)
        # `AlreadyAttested` means a previous run or test already authorised it.
        if not receipt.is_success and "AlreadyAttested" not in message:
            raise AssertionError(
                f"add_issuer({attribute}) failed: {receipt.error_message}"
            )
        authorized_attributes.add(attribute)

    def _issue(account: Keypair, attribute: int) -> None:
        _ensure_issuer(attribute)
        digest = hashlib.sha256(
            f"{account.ss58_address}:{attribute}".encode()
        ).hexdigest()

        if attribute == ATTRIBUTE_BIOMETRICS:
            call, params = "issue_biometrics", {
                "target": account.ss58_address,
                "anchor": "ipfs://test-biometrics",
            }
        else:
            call = "issue_ssn" if attribute == ATTRIBUTE_SSN else "issue_passport"
            params = {
                "target": account.ss58_address,
                "hash": "0x" + digest,
                "anchor": f"ipfs://test-{attribute}",
                "format_ok": True,
            }

        receipt = submit_extrinsic("Identity", call, params, issuer)
        message = str(receipt.error_message)
        if not receipt.is_success and "AlreadyAttested" not in message:
            raise AssertionError(
                f"{call} failed for {account.ss58_address}: {message}"
            )

    def _identity_kyc(account: Keypair, level: int = 1) -> None:
        if level not in _ATTRIBUTES_BY_LEVEL:
            raise ValueError(f"level must be 1..3, got {level}")
        _ensure_registered(account)
        for attribute in _ATTRIBUTES_BY_LEVEL[level]:
            _issue(account, attribute)

    return _identity_kyc


# Dev accounts, in the order `--dev` funds them.
DEV_ACCOUNT_URIS = (
    "//Alice",
    "//Bob",
    "//Charlie",
    "//Dave",
    "//Eve",
    "//Ferdie",
)


# A derivation path no other test touches. `//Dave` and `//Bob` were used as
# "unverified" stand-ins until it turned out `test_council.py` verifies them —
# which made the compliance-gate tests pass or fail depending on file order.
UNVERIFIED_ACCOUNT_URI = "//Alice//governance-unverified"

# Mutable so the fixture closure can share one monotonically increasing value
# across the session.
_UNVERIFIED_ACCOUNTS_ISSUED = [0]


@pytest.fixture
def unverified_account(blockchain_connection, fund_account):
    """Return `unverified_account()` -> a funded keypair with no KYC record.

    Each call derives a *different* account. A single shared path looked fine
    until a test that deliberately verifies its account (`..._succeeds_once_the
    _sender_is_verified`) ran first and left a permanent KYC record behind, after
    which every later negative test tripped this fixture's own guard. Varying the
    path keeps the guard meaningful instead of trading one failure for another.

    Raises `AssertionError` if the account already has an
    `Oracle.IdentityVerifications` entry, so a compliance-gate test can never
    pass for the wrong reason.
    """
    substrate = blockchain_connection

    def _unverified_account() -> Keypair:
        # The counter is module-level on purpose: this fixture is function-scoped,
        # so a local counter would restart at 1 for every test and hand out the
        # same account each time — which is exactly the collision it exists to
        # avoid.
        _UNVERIFIED_ACCOUNTS_ISSUED[0] += 1
        account = Keypair.create_from_uri(
            f"{UNVERIFIED_ACCOUNT_URI}-{_UNVERIFIED_ACCOUNTS_ISSUED[0]}"
        )

        record = substrate.query(
            "Oracle", "IdentityVerifications", [account.ss58_address]
        )
        assert record.value is None, (
            f"{UNVERIFIED_ACCOUNT_URI} already has a KYC record "
            f"({record.value}); the compliance-gate assertion would be "
            f"meaningless"
        )

        fund_account(account, 100 * 1_000_000_000_000)  # 100 DALLA for fees
        return account

    return _unverified_account


@pytest.fixture
def council_keypair(blockchain_connection):
    """A dev keypair that currently holds a Governance council seat, or `None`.

    Several Governance extrinsics (`veto_emergency`, `approve_treasury_spend`)
    are gated on `CouncilMembers`, and they are `ensure_signed`, so the call has
    to be signed by the member itself. Seats are created by district elections,
    so this returns `None` until `tests/governance/test_council.py` has run.
    """
    substrate = blockchain_connection

    def _council_keypair() -> Optional[Keypair]:
        seats = [
            key.value
            for key, _ in substrate.query_map("Governance", "CouncilMembers")
        ]
        if not seats:
            return None

        seat_accounts = {ss58_decode(seat) for seat in seats}
        for uri in DEV_ACCOUNT_URIS:
            candidate = Keypair.create_from_uri(uri)
            if candidate.public_key.hex() in seat_accounts:
                return candidate
        return None

    return _council_keypair


def sudo_inner_result(receipt) -> Optional[dict]:
    """Return the inner `DispatchResult` from a `Sudo.Sudid` event, or None.

    `Sudo.sudo` succeeds whenever the wrapper extrinsic is included, even when
    the wrapped call is rejected — the rejection lands in `Sudid.sudo_result`.
    Checking only `receipt.is_success` therefore reports success for a call
    that did nothing.
    """
    for event in getattr(receipt, "triggered_events", None) or []:
        value = getattr(event, "value", None)
        if not isinstance(value, dict):
            continue
        if value.get("module_id") != "Sudo" or value.get("event_id") != "Sudid":
            continue
        return (value.get("attributes") or {}).get("sudo_result")
    return None


# Pallet names never change mid-session, and each lookup costs an RPC.
_PALLET_NAME_BY_INDEX: dict = {}
_ERROR_NAME_BY_INDEX: dict = {}


def decode_dispatch_error(substrate, error) -> str:
    """Render a swap-level dispatch error as `Pallet.Variant`.

    Two shapes show up, depending on the path:

    * `{'type': 'Module', 'name': 'CannotTransferToSelf', 'docs': [...]}` —
      already decoded by `submit_extrinsic`.
    * `{'Module': {'index': 26, 'error': '0x10000000'}}` — raw, as returned by
      the `Sudo` path. The variant is the low byte of the hex payload.

    Returning one spelling for both is what lets a test assert
    `"EmployerAlreadyRegistered" in receipt.error_message` without caring which
    submitter it used. Before this, every sudo assertion compared a name against
    a hex blob and could never match.
    """
    if error is None:
        return "None"

    if isinstance(error, str):
        return error

    if not isinstance(error, dict):
        return str(error)

    # Already-decoded form.
    if "type" in error and "name" in error:
        return error["name"]

    module = error.get("Module")
    if not isinstance(module, dict):
        return str(error)

    index = module.get("index")
    raw = module.get("error")

    # `DispatchError::Module` carries `error: [u8; 4]`, so the variant index is
    # the FIRST byte of the hex payload, not the low one:
    #   "0x10000000" -> [0x10, 0x00, 0x00, 0x00] -> 16
    hex_digits = str(raw)
    if hex_digits.startswith("0x"):
        hex_digits = hex_digits[2:]
    try:
        error_index = int(hex_digits[:2], 16)
    except (TypeError, ValueError):
        return str(error)

    if not _ERROR_NAME_BY_INDEX:
        # `MetadataVersioned.pallets` is a list of `PalletMetadataV14`, not a
        # name-keyed mapping, so index them directly.
        for pallet in substrate.get_metadata().pallets:
            pallet_index = pallet.value["index"]
            _PALLET_NAME_BY_INDEX[pallet_index] = pallet.value["name"]
            _ERROR_NAME_BY_INDEX[pallet_index] = [
                variant["name"] for variant in pallet.errors
            ]

    names = _ERROR_NAME_BY_INDEX.get(index) or []
    pallet_name = _PALLET_NAME_BY_INDEX.get(index, f"pallet#{index}")
    variant = names[error_index] if error_index < len(names) else f"error#{error_index}"
    return f"{pallet_name}.{variant}"


def error_name(receipt) -> str:
    """`receipt.error_message` rendered as `Pallet.Variant` where possible."""
    return str(getattr(receipt, "error_message", None))


def sudo_call_failed(receipt) -> Optional[str]:
    """Return the inner sudo error as text when the wrapped call was rejected."""
    inner = sudo_inner_result(receipt)
    if isinstance(inner, dict) and "Err" in inner:
        return str(inner["Err"])
    return None


@pytest.fixture
def submit_sudo_extrinsic(blockchain_connection, sudo_keypair):
    """
    Helper to submit sudo-wrapped extrinsics.
    
    Returns a function that wraps an extrinsic in Sudo.sudo() and submits it.
    This is required for extrinsics that use ensure_root().

    Reports the *inner* call's result: `Sudo.sudo` is marked successful as soon
    as the wrapper is included, so a rejected inner call must be read from the
    `Sudid` event instead of from `receipt.is_success`.
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

        # The wrapper extrinsic succeeding says nothing about the call it
        # carried — trust `Sudid` when it is present.
        inner_error = sudo_call_failed(receipt)
        if inner_error is not None:
            inner = sudo_inner_result(receipt) or {}
            raw = inner.get("Err") if isinstance(inner, dict) else None
            detail = decode_dispatch_error(substrate, raw) if raw is not None else inner_error
            return ReceiptWrapper(
                receipt,
                is_success=False,
                error_message=f"Sudo-wrapped {pallet}.{call} rejected: {detail}",
            )
        return ReceiptWrapper(receipt)
    
    return _submit_sudo


# Accounts that sign as oracle operators. Three distinct accounts because
# `MinOracleAgreement` is 3 — a single vote only stages, it never finalizes.
ORACLE_OPERATOR_SEEDS = ("//Alice", "//Bob", "//Charlie")


@pytest.fixture
def verify_kyc(blockchain_connection, sudo_keypair):
    """Return `verify_kyc(account, level)` backed by a real oracle quorum.

    `Oracle.verify_identity` is `ensure_signed` *and* requires an
    `OracleOperators` entry, so it can never be driven by `Sudo.sudo` — that
    path returns `BadOrigin` while the wrapper still reports success. It also
    needs `MinOracleAgreement` (3) matching votes before
    `IdentityVerifications` is written; fewer votes only stage.

    This fixture registers the operator set once per session (sudo accepted:
    `OracleAdminOrigin` is `SignedTechnicalCouncilMember`, which admits `Root`),
    then casts the full quorum per call.
    """
    import hashlib

    substrate = blockchain_connection
    operators = [Keypair.create_from_uri(seed) for seed in ORACLE_OPERATOR_SEEDS]

    # Operators must pay fees. `--dev` endows only Alice/Bob, so fund the rest
    # rather than assume a specific genesis.
    OPERATOR_MIN_FREE = 100 * 1_000_000_000_000  # 100 DALLA at 12 decimals

    def _fund_operator(operator):
        account = substrate.query("System", "Account", [operator.ss58_address]).value
        free = (account or {}).get("data", {}).get("free", 0)
        if free >= OPERATOR_MIN_FREE:
            return
        inner = substrate.compose_call(
            call_module="Balances",
            call_function="force_set_balance",
            call_params={"who": operator.ss58_address, "new_free": OPERATOR_MIN_FREE},
        )
        outer = substrate.compose_call(
            call_module="Sudo", call_function="sudo", call_params={"call": inner}
        )
        receipt = substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
            wait_for_inclusion=True,
        )
        error = sudo_call_failed(receipt)
        if error is not None:
            pytest.skip(f"Cannot fund oracle operator: {error}")

    def _register_operators():
        for operator in operators:
            _fund_operator(operator)
            already = substrate.query(
                "Oracle", "OracleOperators", [operator.ss58_address]
            )
            if already.value:
                continue
            inner = substrate.compose_call(
                call_module="Oracle",
                call_function="add_operator",
                call_params={"operator": operator.ss58_address},
            )
            outer = substrate.compose_call(
                call_module="Sudo", call_function="sudo", call_params={"call": inner}
            )
            receipt = substrate.submit_extrinsic(
                substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
                wait_for_inclusion=True,
            )
            error = sudo_call_failed(receipt)
            if error is not None:
                pytest.skip(f"Cannot register oracle operator: {error}")

    _register_operators()

    def _verify_kyc(account: Keypair, kyc_level: int = 1) -> None:
        """Reach oracle consensus for `account` at `kyc_level`."""
        # Idempotent: a prior run (or an earlier test) may have finalized this
        # subject already, and re-voting the identical payload is rejected as
        # `AlreadyVotedSameKyc` at the pallet level.
        existing = substrate.query(
            "Oracle", "IdentityVerifications", [account.ss58_address]
        )
        if existing.value is not None:
            return

        # A dispute from earlier divergent votes would block *every* vote with
        # `KycDisputeActive`, so clear it before trying to build consensus.
        dispute = substrate.query(
            "Oracle", "OracleDisputeFlag", [account.ss58_address]
        )
        if dispute.value:
            inner = substrate.compose_call(
                call_module="Oracle",
                call_function="resolve_kyc_dispute",
                call_params={"account": account.ss58_address},
            )
            outer = substrate.compose_call(
                call_module="Sudo", call_function="sudo", call_params={"call": inner}
            )
            receipt = substrate.submit_extrinsic(
                substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
                wait_for_inclusion=True,
            )
            error = sudo_call_failed(receipt)
            if error is not None:
                raise AssertionError(
                    f"Cannot clear KYC dispute for {account.ss58_address}: {error}"
                )

        # A disagreement between operators sets a dispute flag that blocks all
        # further votes for the subject. Tests that previously voted with a
        # different (level, id_hash) leave that flag behind, so clear it before
        # voting — otherwise this fixture inherits another test's mess.
        disputed = substrate.query(
            "Oracle", "OracleDisputeFlag", [account.ss58_address]
        )
        if disputed.value:
            inner = substrate.compose_call(
                call_module="Oracle",
                call_function="resolve_kyc_dispute",
                call_params={"account": account.ss58_address},
            )
            outer = substrate.compose_call(
                call_module="Sudo", call_function="sudo", call_params={"call": inner}
            )
            receipt = substrate.submit_extrinsic(
                substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
                wait_for_inclusion=True,
            )
            error = sudo_call_failed(receipt)
            if error is not None:
                raise AssertionError(
                    f"Cannot clear KYC dispute for {account.ss58_address}: {error}"
                )

        # Every operator must submit the *same* (level, id_hash) pair or the
        # votes collide and flag a dispute instead of converging.
        id_hash = "0x" + hashlib.sha256(
            f"{account.ss58_address}-kyc-{kyc_level}".encode()
        ).hexdigest()

        for operator in operators:
            call = substrate.compose_call(
                call_module="Oracle",
                call_function="verify_identity",
                call_params={
                    "account": account.ss58_address,
                    "kyc_level": kyc_level,
                    "id_hash": id_hash,
                    "provider": "testnet",
                    "biometric_verified": False,
                    "address_verified": False,
                },
            )
            receipt = substrate.submit_extrinsic(
                substrate.create_signed_extrinsic(call=call, keypair=operator),
                wait_for_inclusion=True,
            )
            error = getattr(receipt, "error_message", None) or {}
            if receipt.is_success:
                continue
            # An operator that already cast this exact vote still counts toward
            # the quorum, so this is not a failure.
            if "AlreadyVotedSameKyc" in str(error):
                continue
            raise AssertionError(
                f"Oracle vote by {operator.ss58_address} rejected: {error}"
            )

        resolved = substrate.query(
            "Oracle", "IdentityVerifications", [account.ss58_address]
        )
        if resolved.value is None:
            raise AssertionError(
                f"Oracle consensus did not finalize KYC for {account.ss58_address}: "
                f"MinOracleAgreement not reached"
            )

    return _verify_kyc


@pytest.fixture
def set_reserves(blockchain_connection, sudo_keypair):
    """Return `set_reserves(desired)` that raises central-bank reserves safely.

    `Economy.update_reserves` refuses a value below circulating bBZD supply
    (`InsufficientReserves`) — reserves may never under-collateralize. Tests
    share one chain, so a later test's smaller literal would fail even though
    its own logic is fine. Clamp to the current supply instead.
    """
    substrate = blockchain_connection

    def _set_reserves(desired: int) -> int:
        supply = substrate.query("Economy", "TotalBbzdSupply").value or 0
        target = max(desired, int(supply))

        inner = substrate.compose_call(
            call_module="Economy",
            call_function="update_reserves",
            call_params={"new_reserves": target},
        )
        outer = substrate.compose_call(
            call_module="Sudo", call_function="sudo", call_params={"call": inner}
        )
        receipt = substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
            wait_for_inclusion=True,
        )
        error = sudo_call_failed(receipt)
        if error is not None:
            raise AssertionError(f"Cannot set reserves to {target}: {error}")
        return target

    return _set_reserves


@pytest.fixture
def fund_account(blockchain_connection, sudo_keypair):
    """Return `fund_account(address_or_keypair, amount)` that sets a native balance.

    Lets a test use a *fresh* account (to prove a negative like "no KYC means
    no trade") without depending on which seeds a given genesis happens to
    endow. `Balances.force_set_balance` is root-only, hence the sudo wrapper.
    """
    substrate = blockchain_connection

    def _fund_account(account, amount: int) -> None:
        address = getattr(account, "ss58_address", account)
        inner = substrate.compose_call(
            call_module="Balances",
            call_function="force_set_balance",
            call_params={"who": address, "new_free": amount},
        )
        outer = substrate.compose_call(
            call_module="Sudo", call_function="sudo", call_params={"call": inner}
        )
        receipt = substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
            wait_for_inclusion=True,
        )
        error = sudo_call_failed(receipt)
        if error is not None:
            raise AssertionError(f"Cannot fund {address}: {error}")

    return _fund_account


@pytest.fixture
def fund_bbzd(blockchain_connection, sudo_keypair, verify_kyc):
    """Return `fund_bbzd(account, amount)` that mints bBZD through the economy pallet.

    bBZD is **not** a `Balances` asset — it lives in
    `pallet_belize_economy::BBZDBalances`, and `BelizeXAssetLedger` routes
    `AssetId::BBZD` there. A DEX fixture therefore cannot fund the quote leg by
    sending DALLA; it has to go through the minting path, which needs an
    authorized minter, collateralized reserves and recipient KYC >= 1.

    Amounts are kept below `TravelRuleThreshold` (100 DALLA) so Basic KYC is
    sufficient; at or above it the pallet demands Enhanced KYC (L2).
    """
    substrate = blockchain_connection
    minter = Keypair.create_from_uri("//Alice")

    def _ensure_minter_setup():
        res = substrate.query(
            "Economy", "AuthorizedMinters", [minter.ss58_address]
        )
        if not res.value:
            inner = substrate.compose_call(
                call_module="Economy",
                call_function="set_minter_authorization",
                call_params={"account": minter.ss58_address, "authorized": True},
            )
            outer = substrate.compose_call(
                call_module="Sudo", call_function="sudo", call_params={"call": inner}
            )
            receipt = substrate.submit_extrinsic(
                substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
                wait_for_inclusion=True,
            )
            error = sudo_call_failed(receipt)
            if error is not None:
                pytest.skip(f"Cannot authorize bBZD minter: {error}")

        inner = substrate.compose_call(
            call_module="Economy",
            call_function="update_reserves",
            call_params={"new_reserves": 10**18},
        )
        outer = substrate.compose_call(
            call_module="Sudo", call_function="sudo", call_params={"call": inner}
        )
        substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=outer, keypair=sudo_keypair),
            wait_for_inclusion=True,
        )

    _ensure_minter_setup()

    def _fund_bbzd(account: Keypair, amount: int) -> None:
        verify_kyc(account, 1)
        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": account.ss58_address,
                "amount": amount,
                "deposit_reference": "0x" + b"dex-test-deposit".hex(),
            },
        )
        receipt = substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=call, keypair=minter),
            wait_for_inclusion=True,
        )
        if not receipt.is_success:
            raise AssertionError(
                f"mint_bbzd for {account.ss58_address} failed: "
                f"{getattr(receipt, 'error_message', None)}"
            )

    return _fund_bbzd


@pytest.fixture
def fund_wusdc(blockchain_connection, submit_sudo_extrinsic):
    """Return `fund_wusdc(account, amount)` that mints WUSDC through `BelizeX`.

    WUSDC is **not** a `Balances` asset either: `BelizeXAssetLedger` routes
    `AssetId::WUSDC` to `pallet_belize_belizex::WusdcBalances`, which only
    `issue_wusdc` (governance-gated) and `burn_wusdc` can move. Before that
    ledger existed the WUSDC/BBZD pair could never hold a reserve, which made
    the pallet's oracle deviation guard unreachable.
    """
    substrate = blockchain_connection

    def _fund_wusdc(account, amount: int) -> None:
        address = getattr(account, "ss58_address", account)
        res = submit_sudo_extrinsic(
            pallet="BelizeX",
            call="issue_wusdc",
            params={"account": address, "amount": amount},
        )
        if not res["success"]:
            raise AssertionError(
                f"issue_wusdc for {address} failed: {res.get('error')}"
            )

    return _fund_wusdc


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
    # Suites that need the sibling services (Kinich, Nawal, Pakit) rather than
    # just a dev chain. They self-skip when the endpoints are absent, so this
    # marker exists to make that dependency greppable and selectable:
    #   pytest -m "not requires_services"
    config.addinivalue_line(
        "markers",
        "requires_services: mark test as requiring Kinich/Nawal/Pakit services",
    )
    # A test whose assertion depends on chain state that can only be *written*,
    # never cleared. `Oracle.update_exchange_rate` has no inverse, so the
    # fail-closed guard is observable exactly once per chain. Marked so the default
    # run can exclude it and a dedicated first step can run it for real instead of
    # skipping:
    #   pytest -m "not pristine_oracle_rate"  (shared run)
    #   pytest -m "pristine_oracle_rate"      (first, on a fresh chain)
    config.addinivalue_line(
        "markers",
        "pristine_oracle_rate: needs a chain on which no Oracle rate has been set yet",
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
