#!/usr/bin/env python3
"""Submit an on-chain runtime upgrade (`sudo.sudo(system.set_code(wasm))`).

There is no dedicated `set_code` extrinsic on the governance pallets, so the
upgrade goes through `pallet_sudo`, which is ungated on testnet (AR-3). The
sudo key is read from chain and checked against the local keypair before
anything is signed, so a wrong key fails loudly instead of silently.

`system.set_code` rejects code whose `spec_version` is not greater than the
running one, so a stale artifact is refused by the chain rather than applied.
A **rollback** (spec down, e.g. 109 -> 108) therefore needs
``--without-checks``, which dispatches `system.set_code_without_checks`
instead; that variant is root-only and skips the version check.

**Epoch-duration guard.** `BabeEpochDuration` is a *compile-time* constant
(300 with the `testnet-fast-epoch` feature, 14400 without). Deploying a blob
built without that feature onto a chain that is running 300 changes the epoch
length under the live chain and every subsequent block import fails with
"Expected epoch change to happen at ...". This script therefore reads the
chain's duration from `BabeApi_current_epoch` before and after the upgrade and
**fails loudly if it changed** — so a wrongly-built artifact is caught
immediately instead of one epoch boundary later.

Dry run by default; pass ``--execute`` to actually submit.

The RPC endpoint is required (``--rpc``, or ``CEIBA_RPC_URL``). It used to
default to a literal host, which silently targeted whatever machine was live
when the default was written — for a runtime upgrade that means upgrading the
wrong chain. Live testnet (Chain D on ceiba): ``ws://100.119.97.38:9944``.

Usage:
    upgrade-runtime.py <runtime.compact.compressed.wasm> --rpc <url> --key //Alice
    upgrade-runtime.py <runtime.compact.compressed.wasm> --rpc <url> --key //Alice --execute
    upgrade-runtime.py <older.wasm> --rpc <url> --without-checks --execute   # rollback
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
from pathlib import Path

from substrateinterface import Keypair, SubstrateInterface


def blake2b_256(data: bytes) -> str:
    return hashlib.blake2b(data, digest_size=32).hexdigest()


def read_epoch_duration(api: SubstrateInterface) -> int:
    """Return the live BABE epoch duration, in slots.

    `BabeApi_current_epoch` encodes `CurrentEpoch` as
    `(index: u64, start_slot: u64, duration: u64, authorities: Vec<..>)`
    little-endian, so `duration` sits at byte offset 16.
    """
    raw = api.rpc_request("state_call", ["BabeApi_current_epoch", "0x"])["result"]
    data = bytes.fromhex(raw[2:] if raw.startswith("0x") else raw)
    return int.from_bytes(data[16:24], "little")


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
    parser.add_argument(
        "--rpc",
        default=os.environ.get("CEIBA_RPC_URL"),
        help="node RPC endpoint (or set CEIBA_RPC_URL). Required: the target chain must be "
        "stated, not defaulted.",
    )
    parser.add_argument(
        "--key",
        default="//Alice",
        help="sudo key URI (default: //Alice, the testnet genesis sudo key)",
    )
    parser.add_argument("--execute", action="store_true", help="actually submit the upgrade")
    parser.add_argument(
        "--without-checks",
        action="store_true",
        help="dispatch set_code_without_checks (root-only) — required for a spec DOWN rollback",
    )
    args = parser.parse_args()

    if not args.rpc:
        print(
            "ERROR: no RPC endpoint.\n"
            "  Pass --rpc <url>, or set CEIBA_RPC_URL.\n"
            "  Live testnet (Chain D on ceiba): ws://100.119.97.38:9944\n",
            file=sys.stderr,
        )
        return 2

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

        # Capture the live epoch length so a blob that changes it is caught here
        # rather than one epoch boundary later (2026-10-04 stall).
        try:
            epoch_duration_before = read_epoch_duration(api)
            print(f"epoch duration (pre)  : {epoch_duration_before} slots")
        except Exception as exc:  # metadata/runtime without BabeApi — non-fatal
            epoch_duration_before = None
            print(f"epoch duration (pre)  : unavailable ({exc})")
        print()
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

        call_function = "set_code_without_checks" if args.without_checks else "set_code"
        inner = api.compose_call(
            call_module="System",
            call_function=call_function,
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
            print(f"would submit   : sudo.sudo(System::{call_function}(<{len(wasm)} bytes>))")
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
    if blake2b_256(onchain) != blake2b_256(wasm):
        print("\nWARNING: on-chain code hash differs from the submitted wasm.")
        return 1
    if new_spec == current_spec:
        print("\nWARNING: spec_version did not change — the upgrade did not apply.")
        return 1

    # The defect that stalled the chain on 2026-10-04: a runtime built without
    # `testnet-fast-epoch` silently changed EpochDuration 300 -> 14400.
    #
    # A forward upgrade must NOT change the live epoch length. A rollback
    # (--without-checks) is expected to change it back, so it only reports.
    with SubstrateInterface(url=args.rpc) as api:
        try:
            epoch_duration_after = read_epoch_duration(api)
        except Exception as exc:
            epoch_duration_after = None
            print(f"epoch duration (post) : unavailable ({exc})")

    direction = "advanced" if new_spec > current_spec else "rolled back"
    if epoch_duration_after is not None:
        changed = epoch_duration_after != epoch_duration_before
        print(f"epoch duration (post) : {epoch_duration_after} slots")
        if changed and not args.without_checks:
            print(
                f"\n*** WARNING: EPOCH DURATION CHANGED "
                f"{epoch_duration_before} -> {epoch_duration_after} slots ***"
            )
            print(
                "This artifact was built with different features than the running runtime\n"
                "(almost certainly a missing `testnet-fast-epoch`). The chain WILL stop\n"
                "importing blocks at the next epoch boundary. Roll back immediately,\n"
                "then rebuild with: cargo build --release -p belizechain-runtime \\\n"
                "  --features testnet-fast-epoch"
            )
            return 1
        if changed:
            print(
                f"(expected on rollback: {epoch_duration_before} -> {epoch_duration_after} slots)"
            )
        elif args.without_checks:
            print(
                "NOTE: epoch duration unchanged by this rollback — if the chain was\n"
                "stalled on an epoch-boundary mismatch, this blob does NOT fix it."
            )

    print(f"\nupgrade verified: spec_version {direction} and :code matches the submitted wasm.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
