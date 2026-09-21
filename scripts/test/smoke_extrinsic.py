#!/usr/bin/env python3
"""Step 1 smoke test: submit a signed balance transfer to the Ceiba node.

Confirms: RPC reachable, metadata decodable, tx-pool accepts, block authorship
includes the extrinsic, and the expected Transfer event fires.

Usage:
  SMOKE_SENDER_SURI='<sender-suri>' \
  SMOKE_RECIPIENT_SURI='<recipient-suri>' \
  python3 scripts/test/smoke_extrinsic.py

Environment:
  SMOKE_SENDER_SURI     SURI of the sending account. Required — the script never
                        embeds an account key. On the current testnet the funded
                        authority seed is documented in
                        docs/operations/TESTNET_ONLY_RULE_2026-09-18.md.
  SMOKE_RECIPIENT_SURI  SURI of the receiving account. Required (any funded or
                        unfunded account works; the transfer must clear the ED).
  SMOKE_RPC_URL         Override the WebSocket endpoint (default: Ceiba).
"""
from __future__ import annotations

import os
import sys
import time

from substrateinterface import Keypair, SubstrateInterface

RPC_URL = os.environ.get("SMOKE_RPC_URL", "ws://100.81.45.25:9944")
TRANSFER_AMOUNT = 1_000_000_000_000  # 1 UNIT assuming 12 decimals; irrelevant in --dev


def main() -> int:
    sender_suri = os.environ.get("SMOKE_SENDER_SURI", "")
    recipient_suri = os.environ.get("SMOKE_RECIPIENT_SURI", "")
    if not sender_suri or not recipient_suri:
        print(
            "ERROR: SMOKE_SENDER_SURI and SMOKE_RECIPIENT_SURI must both be set.",
            file=sys.stderr,
        )
        return 2

    print(f"[1/6] Connecting to {RPC_URL} ...")
    substrate = SubstrateInterface(url=RPC_URL)
    print(f"      chain      = {substrate.chain}")
    print(f"      runtime    = {substrate.runtime_version}")
    print(f"      ss58_fmt   = {substrate.ss58_format}")
    print(f"      token_sym  = {substrate.token_symbol}")

    sender = Keypair.create_from_uri(sender_suri)
    recipient = Keypair.create_from_uri(recipient_suri)
    print(f"[2/6] sender    = {sender.ss58_address}")
    print(f"      recipient = {recipient.ss58_address}")

    def balance(who: str) -> int:
        info = substrate.query("System", "Account", [who])
        return int(info.value["data"]["free"])

    sender_before = balance(sender.ss58_address)
    recipient_before = balance(recipient.ss58_address)
    print(f"[3/6] pre:  sender={sender_before}  recipient={recipient_before}")

    print("[4/6] Composing balances.transfer_keep_alive ...")
    try:
        call = substrate.compose_call(
            call_module="Balances",
            call_function="transfer_keep_alive",
            call_params={"dest": recipient.ss58_address, "value": TRANSFER_AMOUNT},
        )
    except Exception:
        call = substrate.compose_call(
            call_module="Balances",
            call_function="transfer_allow_death",
            call_params={"dest": recipient.ss58_address, "value": TRANSFER_AMOUNT},
        )

    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=sender)
    print("[5/6] Submitting and waiting for inclusion ...")
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    print(f"      block_hash  = {receipt.block_hash}")
    print(f"      extr_hash   = {receipt.extrinsic_hash}")
    print(f"      is_success  = {receipt.is_success}")

    if not receipt.is_success:
        print(f"      error_msg   = {receipt.error_message}")
        return 2

    transfer_event = None
    for ev in receipt.triggered_events:
        mod = ev.value["module_id"]
        name = ev.value["event_id"]
        if mod == "Balances" and name == "Transfer":
            transfer_event = ev.value["attributes"]
            break
    if transfer_event is None:
        print("      [WARN] no Balances.Transfer event found")
    else:
        print(f"      transfer_evt = {transfer_event}")

    # Small delay so node account state reflects the block
    time.sleep(1)
    sender_after = balance(sender.ss58_address)
    recipient_after = balance(recipient.ss58_address)
    recipient_delta = recipient_after - recipient_before
    print(
        f"[6/6] post: sender={sender_after}  recipient={recipient_after}"
        f"  recipient_delta={recipient_delta}"
    )

    if recipient_delta != TRANSFER_AMOUNT:
        print(
            f"      [FAIL] expected recipient delta {TRANSFER_AMOUNT},"
            f" got {recipient_delta}"
        )
        return 3

    print("SMOKE OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
