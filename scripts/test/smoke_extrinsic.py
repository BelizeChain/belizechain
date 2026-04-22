#!/usr/bin/env python3
"""Step 1 smoke test: submit an Alice->Bob balance transfer to the Ceiba node.

Confirms: RPC reachable, metadata decodable, tx-pool accepts, block authorship
includes the extrinsic, and the expected Transfer event fires.
"""
from __future__ import annotations

import sys
import time

from substrateinterface import Keypair, SubstrateInterface

RPC_URL = "ws://100.81.45.25:9944"
TRANSFER_AMOUNT = 1_000_000_000_000  # 1 UNIT assuming 12 decimals; irrelevant in --dev


def main() -> int:
    print(f"[1/6] Connecting to {RPC_URL} ...")
    substrate = SubstrateInterface(url=RPC_URL)
    print(f"      chain      = {substrate.chain}")
    print(f"      runtime    = {substrate.runtime_version}")
    print(f"      ss58_fmt   = {substrate.ss58_format}")
    print(f"      token_sym  = {substrate.token_symbol}")

    alice = Keypair.create_from_uri("//Alice")
    bob = Keypair.create_from_uri("//Bob")
    print(f"[2/6] Alice  = {alice.ss58_address}")
    print(f"      Bob    = {bob.ss58_address}")

    def balance(who: str) -> int:
        info = substrate.query("System", "Account", [who])
        return int(info.value["data"]["free"])

    alice_before = balance(alice.ss58_address)
    bob_before = balance(bob.ss58_address)
    print(f"[3/6] pre:  Alice={alice_before}  Bob={bob_before}")

    print("[4/6] Composing balances.transfer_keep_alive ...")
    try:
        call = substrate.compose_call(
            call_module="Balances",
            call_function="transfer_keep_alive",
            call_params={"dest": bob.ss58_address, "value": TRANSFER_AMOUNT},
        )
    except Exception:
        call = substrate.compose_call(
            call_module="Balances",
            call_function="transfer_allow_death",
            call_params={"dest": bob.ss58_address, "value": TRANSFER_AMOUNT},
        )

    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
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
    alice_after = balance(alice.ss58_address)
    bob_after = balance(bob.ss58_address)
    delta_bob = bob_after - bob_before
    print(f"[6/6] post: Alice={alice_after}  Bob={bob_after}  bob_delta={delta_bob}")

    if delta_bob != TRANSFER_AMOUNT:
        print(f"      [FAIL] expected bob delta {TRANSFER_AMOUNT}, got {delta_bob}")
        return 3

    print("SMOKE OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
