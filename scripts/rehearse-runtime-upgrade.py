#!/usr/bin/env python3
"""Dress-rehearse a runtime upgrade on a throwaway chain before touching Ceiba.

Reads spec version and BABE epoch length, dispatches `sudo.sudo(System.set_code)`
with the given blob, then reads both again. Exits non-zero if the blob does not
behave as the live chain requires.

Why this exists: the 2026-10-04 stall was a blob built without
`testnet-fast-epoch` that silently changed `BabeEpochDuration` 300 -> 14400 and
permanently stopped block import. `upgrade-runtime.py` catches that on the target
chain; this catches it before the target chain is involved at all.

Usage:
    rehearse-runtime-upgrade.py <wasm> [--rpc ws://127.0.0.1:9956] [--expect-spec 123]
                                 [--expect-epoch 300]
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from substrateinterface import SubstrateInterface, Keypair

CODE_KEY = "0x3a636f6465"


def epoch_length(si: SubstrateInterface) -> int:
    """Live BABE epoch length in slots, decoded from `BabeApi_configuration`."""
    raw = si.rpc_request("state_call", ["BabeApi_configuration", "0x"])["result"]
    blob = bytes.fromhex(raw[2:])
    # BabeConfiguration: slot_duration u64 | epoch_length u64 | ...
    return int.from_bytes(blob[8:16], "little")


def runtime_version(si: SubstrateInterface) -> dict:
    return si.rpc_request("state_getRuntimeVersion", [])["result"]


def spec_version(si: SubstrateInterface) -> int:
    return int(runtime_version(si)["specVersion"])


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("wasm", type=Path)
    p.add_argument("--rpc", default="ws://127.0.0.1:9956")
    p.add_argument("--key", default="//Alice")
    p.add_argument("--expect-spec", type=int, required=True)
    p.add_argument("--expect-epoch", type=int, required=True)
    p.add_argument(
        "--expect-tx-version",
        type=int,
        help="assert the post-upgrade transactionVersion (metadata accuracy check)",
    )
    args = p.parse_args()

    blob = args.wasm.read_bytes()
    print(f"blob        : {args.wasm} ({len(blob):,} bytes)")

    si = SubstrateInterface(url=args.rpc)
    spec_before, epoch_before = spec_version(si), epoch_length(si)
    print(f"spec (pre)  : {spec_before}")
    print(f"epoch (pre) : {epoch_before} slots")

    call = si.compose_call(
        call_module="Sudo",
        call_function="sudo",
        call_params={
            "call": si.compose_call(
                call_module="System",
                call_function="set_code",
                call_params={"code": "0x" + blob.hex()},
            )
        },
    )
    extrinsic = si.create_signed_extrinsic(call=call, keypair=Keypair.create_from_uri(args.key))
    receipt = si.submit_extrinsic(extrinsic, wait_for_inclusion=True)

    if not receipt.is_success:
        print(f"FAIL: upgrade rejected — {receipt.error_message}", file=sys.stderr)
        return 1
    print("set_code    : included and successful")

    spec_after, epoch_after = spec_version(si), epoch_length(si)
    print(f"spec (post) : {spec_after}")
    print(f"epoch (post): {epoch_after} slots")

    ok = True
    if args.expect_tx_version is not None:
        version = runtime_version(si)
        tx_after = int(version["transactionVersion"])
        print(f"tx (post)   : {tx_after}")
        if tx_after != args.expect_tx_version:
            print(
                f"FAIL: transactionVersion is {tx_after}, expected {args.expect_tx_version}",
                file=sys.stderr,
            )
            ok = False

    if spec_after != args.expect_spec:
        print(f"FAIL: spec is {spec_after}, expected {args.expect_spec}", file=sys.stderr)
        ok = False
    if epoch_after != args.expect_epoch:
        print(
            f"FAIL: epoch length is {epoch_after}, expected {args.expect_epoch}.\n"
            f"      This is the 2026-10-04 stall condition — the blob was almost\n"
            f"      certainly built without `--features testnet-fast-epoch`.\n"
            f"      DO NOT deploy it.",
            file=sys.stderr,
        )
        ok = False

    print("RESULT      :", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
