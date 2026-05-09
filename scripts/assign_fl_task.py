#!/usr/bin/env python3
"""Assign a federated learning task on BelizeChain via Sudo.

Calls `Staking::assign_fl_task(task_id, model_hash, computation_time,
reward_multiplier, deadline_blocks)` wrapped in `Sudo::sudo(...)`.

Required env:
    SUDO_SURI                Sudo signer SURI (mnemonic, //suri, or hex seed).
    BLOCKCHAIN_WS_URL        WebSocket RPC, default ws://127.0.0.1:9944.

Usage:
    SUDO_SURI='//YourSudo' \\
    BLOCKCHAIN_WS_URL=ws://100.81.45.25:9944 \\
    python3 scripts/assign_fl_task.py \\
        --task-id 1 \\
        --model-hash 0x$(printf 'belizechain-fl-bootstrap' | sha256sum | cut -d' ' -f1) \\
        --computation-time 600 \\
        --reward-multiplier 100 \\
        --deadline-blocks 14400

Notes:
    * `--reward-multiplier` is parts-per-billion (Perbill). 100 == 0.0000001;
      use 1_000_000_000 for 100%. Default 1_000_000_000.
    * `--deadline-blocks` is added to the current head when the extrinsic runs.
    * Pass `--dry-run` to print the wrapped call data without submitting.
"""

from __future__ import annotations

import argparse
import os
import sys
import time

try:
    from substrateinterface import Keypair, SubstrateInterface
    from substrateinterface.exceptions import SubstrateRequestException
except ImportError:
    print("error: pip install 'substrate-interface>=1.8.0'", file=sys.stderr)
    raise


def _parse_model_hash(s: str) -> list[int]:
    h = s.lower().removeprefix("0x")
    if len(h) != 64:
        raise argparse.ArgumentTypeError("model-hash must be 32 bytes (64 hex chars)")
    try:
        return list(bytes.fromhex(h))
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"invalid hex: {exc}") from exc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Assign FL task on BelizeChain via Sudo")
    parser.add_argument("--task-id", type=int, required=True, help="u32 task id")
    parser.add_argument(
        "--model-hash",
        type=_parse_model_hash,
        required=True,
        help="32-byte model hash, hex (with or without 0x)",
    )
    parser.add_argument(
        "--computation-time", type=int, default=600, help="u32, seconds (informational)"
    )
    parser.add_argument(
        "--reward-multiplier",
        type=int,
        default=1_000_000_000,
        help="Perbill, parts-per-billion (1_000_000_000 = 100%%)",
    )
    parser.add_argument(
        "--deadline-blocks",
        type=int,
        default=14400,
        help="BlockNumber offset added to current head (default ~24h at 6s blocks)",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print call data and exit")
    args = parser.parse_args(argv)

    sudo_suri = os.getenv("SUDO_SURI")
    if not sudo_suri:
        print("error: SUDO_SURI must be set", file=sys.stderr)
        return 2

    ws_url = os.getenv("BLOCKCHAIN_WS_URL", "ws://127.0.0.1:9944")

    try:
        sudo_kp = Keypair.create_from_uri(sudo_suri)
    except Exception as exc:
        print(f"error: bad SUDO_SURI: {exc}", file=sys.stderr)
        return 2

    print(f"connecting to {ws_url}")
    substrate = SubstrateInterface(url=ws_url)
    print(f"  chain   = {substrate.chain}")
    print(f"  runtime = {substrate.runtime_version}")
    print(f"  signer  = {sudo_kp.ss58_address}")

    inner = substrate.compose_call(
        call_module="Staking",
        call_function="assign_fl_task",
        call_params={
            "task_id": args.task_id,
            "model_hash": args.model_hash,
            "computation_time": args.computation_time,
            "reward_multiplier": args.reward_multiplier,
            "deadline_blocks": args.deadline_blocks,
        },
    )
    sudo_call = substrate.compose_call(
        call_module="Sudo",
        call_function="sudo",
        call_params={"call": inner.value},
    )

    print(f"call = Sudo.sudo(Staking.assign_fl_task(task_id={args.task_id}, ...))")
    if args.dry_run:
        print(sudo_call)
        return 0

    extrinsic = substrate.create_signed_extrinsic(call=sudo_call, keypair=sudo_kp)
    try:
        receipt = substrate.submit_extrinsic(
            extrinsic, wait_for_inclusion=True, wait_for_finalization=False
        )
    except SubstrateRequestException as exc:
        print(f"submit failed: {exc}", file=sys.stderr)
        return 1

    print(f"included in block: {receipt.block_hash}")
    print(f"extrinsic hash:    {receipt.extrinsic_hash}")
    print(f"is_success:        {receipt.is_success}")
    if not receipt.is_success:
        print(f"error_message:     {receipt.error_message}", file=sys.stderr)
        return 1

    # Surface key events.
    for evt in receipt.triggered_events:
        ev = evt.value.get("event", evt.value)
        module = ev.get("module_id") or ev.get("event", {}).get("module_id")
        name = ev.get("event_id") or ev.get("event", {}).get("event_id")
        if module in ("Staking", "Sudo", "System"):
            print(f"  event: {module}.{name} attrs={ev.get('attributes', ev.get('event'))}")

    # Verify ActiveFLTask now matches.
    time.sleep(0.5)
    active = substrate.query(module="Staking", storage_function="ActiveFLTask")
    print(f"ActiveFLTask after assignment: {active.value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
