#!/usr/bin/env python3
"""Attest that a learner completed a Maya Academy module, so they can be paid.

`community::complete_education_module` refuses to mint without a prior
attestation from the account seated in `Config::OracleAttestationOrigin`. On
this chain that is `TechnicalCouncilMember`: any single *signed* member of
TechnicalCouncil, **or Root**. The attestation is consumed by the claim, so one
attestation buys exactly one reward.

Two ways to satisfy it, and the choice matters:

* `--key <mnemonic>` — sign directly as a seated TechnicalCouncil member.
  Requires a collective motion to seat that account, so on a single-operator
  testnet it fails with `BadOrigin` until someone runs governance.
* `--via-sudo` — wrap the call in `sudo.sudo`, which dispatches it as Root and
  therefore passes. This is the path that actually works on Chain D today.

This is the operator side of that gate. It is deliberately a script and not a
Pallet call wrapper: the judgement ("did this account pass?") is made by the
learning provider off-chain, and this records the outcome.

Usage:
    attest-education.py --rpc ws://100.119.97.38:9944 --list
    attest-education.py --rpc ws://100.119.97.38:9944 --learner r1... --module 0
    attest-education.py --rpc ws://100.119.97.38:9944 --learner r1... --module 0 --via-sudo --execute

Dry run by default. Pass ``--execute`` to submit.

The RPC endpoint is required and has no default: a script that guesses which
chain to write to is a script that eventually attests on the wrong one.
"""

from __future__ import annotations

import argparse
import os
import sys

from substrateinterface import Keypair, SubstrateInterface

# `pallet_collective::Members` for TechnicalCouncil, read through the chain's
# storage rather than assumed from a config file.
COUNCIL_MEMBERS_STORAGE = "TechnicalCouncil"

MODULE_ENTRY_SIZE_HINT = None  # queried, not assumed — see read_modules()


def read_modules(api: SubstrateInterface) -> dict[int, dict]:
    """Return `{module_id: {title, reward, active, completions, max}}`."""
    modules: dict[int, dict] = {}
    for key, value in api.query_map("Community", "EducationModules"):
        # py-substrate-interface returns a single-key map's key as a scalecodec
        # object (U32 here), not raw bytes, so unwrap `.value` before use.
        module_id = key.value if hasattr(key, "value") else int(key)
        raw = value.value
        modules[module_id] = {
            "title": _decode_text(raw.get("title")),
            "reward": _planck_to_dalla(raw.get("reward_amount", 0)),
            "active": bool(raw.get("active")),
            "completions": raw.get("total_completions", 0),
            "max": raw.get("max_completions"),
        }
    return dict(sorted(modules.items()))


def _decode_text(value: object) -> str:
    """`BoundedVec<u8, _>` arrives as bytes; anything else is rendered as-is."""
    if isinstance(value, (bytes, bytearray)):
        return bytes(value).decode("utf-8", errors="replace")
    return str(value)


def _planck_to_dalla(value: int) -> str:
    """DALLA has 12 decimals; render without trailing zeros."""
    whole, frac = divmod(int(value), 10**12)
    if frac == 0:
        return str(whole)
    return f"{whole}.{frac:012d}".rstrip("0")


def council_members(api: SubstrateInterface) -> list[bytes]:
    """Raw 32-byte AccountIds currently seated in TechnicalCouncil."""
    result = api.query(COUNCIL_MEMBERS_STORAGE, "Members")
    members = result.value or []
    out = []
    for member in members:
        out.append(_to_account_bytes(member))
    return out


def _to_account_bytes(value: object) -> bytes:
    """Resolve an account value to its raw 32-byte AccountId."""
    if isinstance(value, (bytes, bytearray)):
        return bytes(value)
    if isinstance(value, str):
        if value.startswith("0x"):
            return bytes.fromhex(value[2:])
        return Keypair(ss58_address=value).public_key
    raise TypeError(f"unsupported account value: {type(value)!r}")


def _as_hex(value: object) -> str:
    if isinstance(value, (bytes, bytearray)):
        return "0x" + bytes(value).hex()
    return str(value)


def parse_module_selection(text: str, known: dict[int, dict]) -> list[int]:
    """`all` or a comma-separated list of ids, validated against the chain."""
    if text.strip().lower() == "all":
        return list(known)

    ids = []
    for part in text.split(","):
        part = part.strip()
        if not part:
            continue
        if not part.isdigit():
            raise SystemExit(f"ERROR: {part!r} is not a module id")
        module_id = int(part)
        if module_id not in known:
            raise SystemExit(
                f"ERROR: module {module_id} does not exist on chain "
                f"(known: {', '.join(str(i) for i in known)})"
            )
        ids.append(module_id)
    if not ids:
        raise SystemExit("ERROR: no modules selected")
    return ids


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rpc",
        default=os.environ.get("CEIBA_RPC_URL"),
        help="node RPC endpoint (or set CEIBA_RPC_URL). Required.",
    )
    parser.add_argument(
        "--key",
        default="//Alice",
        help="attester key URI (default: //Alice). The account must be a seated "
        "TechnicalCouncil member, or Root.",
    )
    parser.add_argument(
        "--learner",
        help="SS58 address (prefix 1981) of the account that sat the module",
    )
    parser.add_argument(
        "--module",
        default=None,
        help="module id, comma-separated ids, or 'all'",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="print modules and their completion counts, then exit",
    )
    parser.add_argument(
        "--via-sudo",
        action="store_true",
        help="wrap the call in sudo.sudo so it dispatches as Root. Needed when the "
        "key is neither a seated TechnicalCouncil member nor otherwise authorised.",
    )
    parser.add_argument("--execute", action="store_true", help="actually submit")
    args = parser.parse_args()

    if not args.rpc:
        print(
            "ERROR: no RPC endpoint.\n"
            "  Pass --rpc <url>, or set CEIBA_RPC_URL.\n"
            "  Live testnet (Chain D on ceiba): ws://100.119.97.38:9944\n",
            file=sys.stderr,
        )
        return 2

    if not args.list and (not args.learner or args.module is None):
        print(
            "ERROR: nothing to do — pass --list, or --learner and --module.\n",
            file=sys.stderr,
        )
        return 2

    with SubstrateInterface(url=args.rpc) as api:
        modules = read_modules(api)
        if not modules:
            print("ERROR: the chain reports no education modules. Nothing to attest.")
            return 1

        print(f"education modules on chain: {len(modules)}")
        for module_id, module in modules.items():
            cap = "uncapped" if module["max"] is None else f"max {module['max']}"
            state = "active" if module["active"] else "INACTIVE"
            print(
                f"  [{module_id}] {module['title'][:44]:<44} "
                f"{module['reward']:>6} DALLA  {state:<8} "
                f"{module['completions']} completed, {cap}"
            )

        if args.list:
            return 0

        target_modules = parse_module_selection(args.module, modules)
        learner = args.learner

        keypair = Keypair.create_from_uri(args.key)
        seated = council_members(api)
        is_member = keypair.public_key in seated
        is_root_key = keypair.public_key == _root_account(api)

        print(f"\nattester              : {keypair.ss58_address}")
        print(f"  seated in council   : {'yes' if is_member else 'NO'}")
        print(f"  is sudo/root key    : {'yes' if is_root_key else 'NO'}")
        print(f"  dispatch            : {'sudo.sudo (Root)' if args.via_sudo else 'signed'}")

        # Preflight the origin, because the two ways of passing it are exclusive:
        # Root authority reaches the catch-all branch of
        # `SignedTechnicalCouncilMember::try_origin`, but a *signed* sudo key does
        # not — signing an extrinsic with the sudo key produces RawOrigin::Signed,
        # not Root, and is rejected with BadOrigin like any non-member.
        if args.via_sudo:
            if not is_root_key:
                print(
                    "\nFATAL: --via-sudo needs the sudo key. `sudo.sudo` is itself\n"
                    "checked against Sudo::Key, so any other account is rejected\n"
                    "before the education call is reached.",
                    file=sys.stderr,
                )
                return 1
        elif not is_member:
            print(
                "\nFATAL: this key is neither a seated TechnicalCouncil member nor\n"
                "Root, so `attest_education_completion` would be rejected with\n"
                "`BadOrigin` — seat the account via a collective motion, or pass\n"
                "--via-sudo with the sudo key.",
                file=sys.stderr,
            )
            return 1

        print(f"learner               : {learner}")
        print(f"modules               : {', '.join(str(i) for i in target_modules)}")

        # Attestation state is public, so a wrong learner or an already-claimed
        # module is visible before anything is signed.
        for module_id in target_modules:
            # `EducationAttestations` is a StorageMap keyed by the composite
            # `(AccountId, u32)`, so both parts travel as ONE parameter.
            entry = api.query(
                "Community", "EducationAttestations", [(learner, module_id)]
            )
            completed = api.query("Community", "CompletedEducation", [learner, module_id])
            if completed.value:
                print(f"  [{module_id}] already CLAIMED — attesting again is wasted")
            elif entry.value:
                print(f"  [{module_id}] already attested, not yet claimed")
            else:
                print(f"  [{module_id}] no attestation yet — this call creates one")

        if not args.execute:
            print("\nDRY RUN — nothing submitted.")
            for module_id in target_modules:
                call = _build_call(api, learner, module_id, via_sudo=args.via_sudo)
                extrinsic = api.create_signed_extrinsic(call=call, keypair=keypair)
                prefix = "sudo.sudo(Community.attest_education_completion" if args.via_sudo else "Community.attest_education_completion"
                suffix = "))" if args.via_sudo else ")"
                print(
                    f"would submit   : {prefix}({learner}, {module_id}{suffix}\n"
                    f"extrinsic hash : {_as_hex(extrinsic.extrinsic_hash)}"
                )
            print("\nre-run with --execute to submit.")
            return 0

        failures = 0
        for module_id in target_modules:
            call = _build_call(api, learner, module_id, via_sudo=args.via_sudo)
            extrinsic = api.create_signed_extrinsic(call=call, keypair=keypair)
            receipt = api.submit_extrinsic(extrinsic, wait_for_inclusion=True)

            if receipt.is_success:
                # `block_number` is None on this node's WS response even though the
                # extrinsic is finalised, so do not print a literal "block None".
                where = f"block {receipt.block_number}" if receipt.block_number else "a block"
                print(f"  [{module_id}] attested in {where}")
            else:
                failures += 1
                print(f"  [{module_id}] FAILED: {receipt.error_message}")

    if failures:
        print(f"\n{failures} of {len(target_modules)} attestation(s) failed.")
        return 1

    print(
        f"\n{len(target_modules)} attestation(s) submitted. The learner's claim in\n"
        "Maya Wallet will now be accepted; the attestation is consumed by it."
    )
    return 0


def _build_call(
    api: SubstrateInterface, learner: str, module_id: int, *, via_sudo: bool
):
    """The attestation call, optionally wrapped so it dispatches as Root."""
    inner = api.compose_call(
        call_module="Community",
        call_function="attest_education_completion",
        call_params={"learner": learner, "module_id": module_id},
    )
    if not via_sudo:
        return inner
    return api.compose_call(
        call_module="Sudo", call_function="sudo", call_params={"call": inner}
    )


def _root_account(api: SubstrateInterface) -> bytes:
    """The sudo key, used as a stand-in for "Root authority" in the preflight."""
    try:
        value = api.query("Sudo", "Key").value
    except Exception:
        return b""
    if value is None:
        return b""
    return _to_account_bytes(value)


if __name__ == "__main__":
    raise SystemExit(main())
