#!/usr/bin/env python3
"""Submit an on-chain runtime upgrade (`sudo.sudo(system.set_code(wasm))`).

There is no dedicated `set_code` extrinsic on the governance pallets, so the
upgrade goes through `pallet_sudo`, which is ungated on testnet (AR-3). The
sudo key is read from chain and checked against the local keypair before
anything is signed, so a wrong key fails loudly instead of silently.

`system.set_code` rejects code whose `spec_version` is not greater than the
running one, so a stale artifact is refused by the chain rather than applied.

Dry run by default; pass ``--execute`` to actually submit.

Usage:
    upgrade-runtime.py <runtime.compact.compressed.wasm> --key //Alice
    upgrade-runtime.py <runtime.compact.compressed.wasm> --key //Alice --execute
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

from substrateinterface import Keypair, SubstrateInterface


def blake2b_256(data: bytes) -> str:
    return hashlib.blake2b(data, digest_size=32).hexdigest()


def _to_account_bytes(value: object) -> bytes:
    """Resolve a queried account value to its raw 32-byte AccountId."""
    if isinstance(value, (bytes, bytearray)):
        return bytes(value)
    if isinstance(value, str):
        text = value
        if text.startswith("0x"):
            return bytes.fromhex(text[2:])
        return Keypair(ss58_address=text).public_key
    raise TypeError(f"unsupported account value: {type(value)!r}")


def _as_hex(value: object) -> str:
    """Render an extrinsic hash (bytes or hex string) as 0x-prefixed hex."""
    if isinstance(value, (bytes, bytearray)):
        return "0x" + bytes(value).hex()
    return str(value)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wasm", type=Path, help="compressed runtime WASM to deploy")
    parser.add_argument("--rpc", default="ws://100.81.45.25:9944", help="node RPC endpoint")
    parser.add_argument(
        "--key",
        default="//Alice",
        help="sudo key URI (default: //Alice, the testnet genesis sudo key)",
    )
    parser.add_argument("--execute", action="store_true", help="actually submit the upgrade")
    args = parser.parse_args()

    wasm = args.wasm.read_bytes()
    code_hex = "0x" + wasm.hex()
    print(f"wasm        : {args.wasm}")
    print(f"wasm size   : {len(wasm)} bytes")
    print(f"wasm blake2b: {blake2b_256(wasm)}")
    print()

    with SubstrateInterface(url=args.rpc) as api:
        running = api.get_runtime_version() if hasattr(api, "get_runtime_version") else None
        if running is None:
            spec = api.rpc_request("state_getRuntimeVersion", [])["result"]
        else:
            spec = running
        current_spec = spec["specVersion"] if isinstance(spec, dict) else spec.spec_version
        print(f"on-chain spec_version : {current_spec}")

        sudo_key = api.query("Sudo", "Key")
        sudo_raw = _to_account_bytes(sudo_key.value)
        print(f"on-chain sudo key     : 0x{sudo_raw.hex()}")
        print(f"  (as ss58/42)        : {Keypair(public_key=sudo_raw, ss58_format=42).ss58_address}")

        keypair = Keypair.create_from_uri(args.key)
        print(f"local keypair         : 0x{keypair.public_key.hex()}")
        print(f"  (as ss58/42)        : {keypair.ss58_address}")
        # Compare raw public keys, never SS58 strings: the chain sets its own
        # SS58 prefix (BelizeChain uses 1981), so the same account renders
        # differently here than in a default-prefix tool.
        if keypair.public_key != sudo_raw:
            print("\nFATAL: local keypair is not the sudo key — refusing to submit.")
            return 1
        print()

        inner = api.compose_call(
            call_module="System",
            call_function="set_code",
            call_params={"code": code_hex},
        )
        outer = api.compose_call(
            call_module="Sudo",
            call_function="sudo",
            call_params={"call": inner},
        )
        extrinsic = api.create_signed_extrinsic(call=outer, keypair=keypair)
        tx_hash = _as_hex(extrinsic.extrinsic_hash)

        if not args.execute:
            print("DRY RUN — nothing submitted.")
            print(f"would submit   : sudo.sudo(System::set_code(<{len(wasm)} bytes>))")
            print(f"extrinsic hash : {tx_hash}")
            print("\nre-run with --execute to submit.")
            return 0

        print(f"submitting     : {tx_hash}")
        receipt = api.submit_extrinsic(extrinsic, wait_for_inclusion=True)
        print(f"included       : block {receipt.block_number}")
        print(f"extrinsic hash : {_as_hex(receipt.extrinsic_hash)}")
        if receipt.is_success:
            print("result         : SUCCESS")
        else:
            print("result         : FAILED")
            print(f"error          : {receipt.error_message}")
            for event in receipt.triggered_events:
                if "System" in str(event):
                    print(f"  event        : {event}")
            return 1

    # Re-read the version from a fresh connection so the result is not cached.
    with SubstrateInterface(url=args.rpc) as api:
        spec = api.rpc_request("state_getRuntimeVersion", [])["result"]
        new_spec = spec["specVersion"]
        code = api.rpc_request("state_getStorage", ["0x3a636f6465"])["result"]
        onchain = bytes.fromhex(code[2:])

    print(f"new spec_version: {new_spec}")
    print(f"on-chain :code  : {len(onchain)} bytes  blake2b {blake2b_256(onchain)}")
    if new_spec == current_spec:
        print("\nWARNING: spec_version did not change — the upgrade did not apply.")
        return 1
    if blake2b_256(onchain) != blake2b_256(wasm):
        print("\nWARNING: on-chain code hash differs from the submitted wasm.")
        return 1
    print("\nupgrade verified: spec_version advanced and :code matches the submitted wasm.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
