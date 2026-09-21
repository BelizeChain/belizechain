#!/usr/bin/env python3
"""One-time testnet repair: fund the founder key from the chain's sudo/dev authority.

Chain: THE Ceiba testnet (BelizeChain Testnet, spec 105). No resets, no spec edits.
Usage:  SUDO_SURI='<authority-suri>' python3 fund_founder.py <amount_dalla> [<recipient_ss58>]
Defaults: 100 DALLA to the founder address from mainnet-keys-inventory.json (same pubkey as r1SaBq6...).
Prints final balances of sender and recipient after the transfer (on-chain proof).

Environment:
  SUDO_SURI   SURI of the chain's current sudo authority. Required — the script never
              embeds the authority key. On the current testnet the authority is the
              documented dev seed (docs/operations/TESTNET_ONLY_RULE_2026-09-18.md).
"""
import os
import sys
from substrateinterface import Keypair, SubstrateInterface

URL = "ws://100.81.45.25:9944"
DEFAULT_RECIPIENT = "5Cg3Ez7Upm8caDfjonnMKPZ14B3H5daWM75DkYj7yEt4XSKt"  # founder key, generic SS58
DECIMALS = 12


def main() -> int:
    suri = os.environ.get("SUDO_SURI", "")
    if not suri:
        print("ERROR: SUDO_SURI is not set (SURI of the chain's sudo authority).", file=sys.stderr)
        return 2

    amount_dalla = float(sys.argv[1]) if len(sys.argv) > 1 else 100.0
    recipient = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_RECIPIENT
    planck = str(int(amount_dalla * 10 ** DECIMALS))

    substrate = SubstrateInterface(url=URL)
    authority = Keypair.create_from_uri(suri)
    # Decode any SS58 format (generic 42 or Belize r1/1981) to raw pubkey hex
    pubkey = bytes.fromhex(substrate.ss58_decode(recipient))
    founder = Keypair(ss58_format=substrate.ss58_format, public_key=pubkey)

    print(f"[1/5] Connected: {substrate.chain}")
    print(f"[2/5] Authority address: {authority.ss58_address}")
    print(f"[3/5] Founder address: {founder.ss58_address}")

    sender = substrate.query("System", "Account", [authority.public_key]).value
    target = substrate.query("System", "Account", [founder.public_key]).value
    print(
        f"      Pre-state: authority free={sender['data']['free']},"
        f" founder free={target['data']['free']}"
    )

    call = substrate.compose_call(
        call_module="Balances",
        call_function="transfer_keep_alive",
        call_params={"dest": founder.ss58_address, "value": planck},
    )
    extrinsic = substrate.create_signed_extrinsic(call, keypair=authority)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True, wait_for_finalization=True)
    print(f"[4/5] Transfer submitted: hash={receipt.extrinsic_hash} block={receipt.block_hash}")
    if not receipt.is_success:
        print(f"      TRANSFER FAILED on-chain: {receipt.error_message}")
        return 1

    sender_after = substrate.query("System", "Account", [authority.public_key]).value
    target_after = substrate.query("System", "Account", [founder.public_key]).value
    print(f"[5/5] Proof: authority free={sender_after['data']['free']}", flush=True)
    print(f"      Founder free={target_after['data']['free']} (target was {planck} planck = {amount_dalla} DALLA)")
    return 0 if target_after['data']['free'] >= int(planck) else 1


if __name__ == "__main__":
    sys.exit(main())
