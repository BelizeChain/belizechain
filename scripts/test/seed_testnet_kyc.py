#!/usr/bin/env python3
"""Seed Compliance + Identity KYC state for testnet operator accounts.

This script lifts the reusable KYC seeding flow out of `tests/conftest.py`
into a runnable operator utility.

Usage:
  BLOCKCHAIN_WS_URL=ws://127.0.0.1:9944 \
  ISSUER_SURI='//Alice' \
  SUDO_SURI='//Alice' \
  python3 scripts/test/seed_testnet_kyc.py --input scripts/test/seed_testnet_kyc.example.json

Environment:
  BLOCKCHAIN_WS_URL   WebSocket endpoint for the target chain.
  ISSUER_SURI         Account used to submit Identity.issue_ssn / issue_passport.
  SUDO_SURI          Optional. Preferred automation path for Compliance.verify_account
                     and Identity.add_issuer via Sudo.sudo(...).
  COUNCIL_SURI       Optional fallback when SUDO_SURI is unavailable. Automated
                     council mode only supports the single-member bootstrap case.

Notes:
  - This script is idempotent. It skips already-seeded compliance state,
    identities, issuer authorizations, and active attestations.
  - The manifest carries explicit attestation hashes/anchors. Do not rely on
    synthetic test data generation in the operator path.
  - If your TechnicalCouncil has more than one active member and you do not
    have SUDO_SURI, this script exits with a clear error instead of guessing
    the collective vote/close flow.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from substrateinterface import Keypair, SubstrateInterface


VERIFICATION_LEVEL_NAMES = {
    0: "None",
    1: "Basic",
    2: "Standard",
    3: "Enhanced",
    4: "Government",
}
VERIFICATION_LEVEL_VALUES = {name: value for value, name in VERIFICATION_LEVEL_NAMES.items()}

ATTRIBUTE_CONFIG = {
    "ssn": {
        "attr_id": 0,
        "issuer_storage": "SsnIssuers",
        "attestation_storage": "SsnAttestations",
        "issue_call": "issue_ssn",
        "hash_field": "ssn_hash",
        "anchor_field": "ssn_anchor",
    },
    "passport": {
        "attr_id": 1,
        "issuer_storage": "PassportIssuers",
        "attestation_storage": "PassportAttestations",
        "issue_call": "issue_passport",
        "hash_field": "passport_hash",
        "anchor_field": "passport_anchor",
    },
}


@dataclass
class Target:
    suri: str
    name: str
    verification_level: int
    risk_level: int
    ssn_hash: str
    ssn_anchor: str
    passport_hash: str
    passport_anchor: str

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "Target":
        required_fields = [
            "suri",
            "name",
            "ssn_hash",
            "ssn_anchor",
            "passport_hash",
            "passport_anchor",
        ]
        missing = [field for field in required_fields if field not in payload]
        if missing:
            raise ValueError(f"Target manifest missing required fields: {', '.join(missing)}")

        verification_level = int(payload.get("verification_level", 4))
        risk_level = int(payload.get("risk_level", 1))

        if verification_level not in VERIFICATION_LEVEL_NAMES:
            raise ValueError(f"Unsupported verification_level: {verification_level}")
        if not 0 <= risk_level <= 255:
            raise ValueError(f"risk_level must be between 0 and 255, got {risk_level}")

        return cls(
            suri=str(payload["suri"]),
            name=str(payload["name"]),
            verification_level=verification_level,
            risk_level=risk_level,
            ssn_hash=_normalize_h256(payload["ssn_hash"], "ssn_hash"),
            ssn_anchor=_normalize_anchor(payload["ssn_anchor"]),
            passport_hash=_normalize_h256(payload["passport_hash"], "passport_hash"),
            passport_anchor=_normalize_anchor(payload["passport_anchor"]),
        )


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input",
        required=True,
        type=Path,
        help="Path to the JSON target manifest.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Compose and print actions without submitting extrinsics.",
    )
    parser.add_argument(
        "--bootstrap-council",
        action="store_true",
        help=(
            "Append COUNCIL_SURI to the current TechnicalCouncil members via SUDO_SURI. "
            "Useful for local/bootstrap flows only."
        ),
    )
    return parser.parse_args()


def _load_targets(path: Path) -> list[Target]:
    payload = json.loads(path.read_text())
    if not isinstance(payload, dict) or not isinstance(payload.get("targets"), list):
        raise ValueError("Manifest must be a JSON object with a 'targets' array")
    targets = [Target.from_dict(entry) for entry in payload["targets"]]
    if not targets:
        raise ValueError("Manifest must contain at least one target")
    return targets


def _normalize_h256(value: Any, field_name: str) -> str:
    if not isinstance(value, str):
        raise ValueError(f"{field_name} must be a hex string")
    if not value.startswith("0x"):
        raise ValueError(f"{field_name} must start with 0x")
    hex_value = value[2:]
    if len(hex_value) != 64:
        raise ValueError(f"{field_name} must be 32 bytes (64 hex chars)")
    try:
        bytes.fromhex(hex_value)
    except ValueError as exc:
        raise ValueError(f"{field_name} is not valid hex") from exc
    return value.lower()


def _normalize_anchor(value: Any) -> str:
    if not isinstance(value, str):
        raise ValueError("anchor values must be strings")
    if value.startswith("0x"):
        try:
            raw = bytes.fromhex(value[2:])
        except ValueError as exc:
            raise ValueError(f"Invalid hex anchor: {value}") from exc
        if len(raw) > 128:
            raise ValueError("anchor exceeds Identity::MaxAnchorLen (128 bytes)")
        return value.lower()

    raw = value.encode("utf-8")
    if len(raw) > 128:
        raise ValueError("anchor exceeds Identity::MaxAnchorLen (128 bytes)")
    return f"0x{raw.hex()}"


def _query_value(
    substrate: SubstrateInterface,
    module: str,
    storage_function: str,
    params: list[Any] | None = None,
) -> Any:
    result = substrate.query(module=module, storage_function=storage_function, params=params or [])
    return result.value if hasattr(result, "value") else result


def _enum_name(value: Any) -> str | None:
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and value:
        return next(iter(value.keys()))
    return None


def _submit_extrinsic(
    substrate: SubstrateInterface,
    keypair: Keypair,
    call: Any,
    label: str,
    *,
    dry_run: bool,
) -> None:
    signer = getattr(keypair, "ss58_address", "unknown")
    print(f"  - {label} [signer={signer}]")
    if dry_run:
        return

    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)
    receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
    if not receipt.is_success:
        raise RuntimeError(f"{label} failed: {receipt.error_message}")


def _compose_sudo_call(substrate: SubstrateInterface, call: Any) -> Any:
    return substrate.compose_call(
        call_module="Sudo",
        call_function="sudo",
        call_params={"call": call},
    )


def _ensure_council_bootstrap(
    substrate: SubstrateInterface,
    sudo_keypair: Keypair,
    council_keypair: Keypair,
    *,
    dry_run: bool,
) -> None:
    members = _query_value(substrate, "TechnicalCouncil", "Members") or []
    if council_keypair.ss58_address in members:
        print(f"[skip] council member already present: {council_keypair.ss58_address}")
        return

    prime = _query_value(substrate, "TechnicalCouncil", "Prime")
    new_members = list(members) + [council_keypair.ss58_address]
    new_prime = prime if prime in new_members else council_keypair.ss58_address

    inner_call = substrate.compose_call(
        call_module="TechnicalCouncil",
        call_function="set_members",
        call_params={
            "new_members": new_members,
            "prime": new_prime,
            "old_count": len(members),
        },
    )
    sudo_call = _compose_sudo_call(substrate, inner_call)
    _submit_extrinsic(
        substrate,
        sudo_keypair,
        sudo_call,
        f"bootstrap TechnicalCouncil membership for {council_keypair.ss58_address}",
        dry_run=dry_run,
    )


def _submit_admin_call(
    substrate: SubstrateInterface,
    call: Any,
    label: str,
    *,
    sudo_keypair: Keypair | None,
    council_keypair: Keypair | None,
    dry_run: bool,
) -> None:
    if sudo_keypair is not None:
        sudo_call = _compose_sudo_call(substrate, call)
        _submit_extrinsic(substrate, sudo_keypair, sudo_call, label, dry_run=dry_run)
        return

    if council_keypair is None:
        raise RuntimeError(
            f"{label} requires either SUDO_SURI or COUNCIL_SURI for admin-origin calls"
        )

    members = _query_value(substrate, "TechnicalCouncil", "Members") or []
    if council_keypair.ss58_address not in members:
        raise RuntimeError(
            "COUNCIL_SURI is not an active TechnicalCouncil member; provide SUDO_SURI "
            "or bootstrap council membership first"
        )
    if len(members) != 1:
        raise RuntimeError(
            "Automated council mode only supports the single-member TechnicalCouncil bootstrap "
            "case. Provide SUDO_SURI or complete the collective vote manually."
        )

    proposal_len = len(call.encode())
    propose_call = substrate.compose_call(
        call_module="TechnicalCouncil",
        call_function="propose",
        call_params={
            "threshold": 1,
            "proposal": call,
            "length_bound": proposal_len + 100,
        },
    )
    _submit_extrinsic(substrate, council_keypair, propose_call, label, dry_run=dry_run)


def _current_verification_level(substrate: SubstrateInterface, address: str) -> int:
    status = _query_value(substrate, "Compliance", "ComplianceStatusOf", [address]) or {}
    level_name = _enum_name(status.get("verification_level")) if isinstance(status, dict) else None
    return VERIFICATION_LEVEL_VALUES.get(level_name or "None", 0)


def _ensure_compliance(
    substrate: SubstrateInterface,
    target: Target,
    target_keypair: Keypair,
    *,
    sudo_keypair: Keypair | None,
    council_keypair: Keypair | None,
    dry_run: bool,
) -> None:
    current_level = _current_verification_level(substrate, target_keypair.ss58_address)
    if current_level >= target.verification_level:
        print(
            f"[skip] compliance already seeded for {target_keypair.ss58_address} "
            f"({VERIFICATION_LEVEL_NAMES[current_level]})"
        )
        return

    call = substrate.compose_call(
        call_module="Compliance",
        call_function="verify_account",
        call_params={
            "account": target_keypair.ss58_address,
            "level": target.verification_level,
            "risk_level": target.risk_level,
        },
    )
    label = (
        f"verify_account({target_keypair.ss58_address}, "
        f"level={target.verification_level}, risk={target.risk_level})"
    )
    _submit_admin_call(
        substrate,
        call,
        label,
        sudo_keypair=sudo_keypair,
        council_keypair=council_keypair,
        dry_run=dry_run,
    )


def _ensure_identity_registered(
    substrate: SubstrateInterface,
    target: Target,
    target_keypair: Keypair,
    *,
    planned_identities: set[str],
    dry_run: bool,
) -> None:
    identity_id = _query_value(substrate, "Identity", "IdentityOf", [target_keypair.ss58_address])
    if identity_id is not None:
        planned_identities.add(target_keypair.ss58_address)
        print(f"[skip] identity already registered for {target_keypair.ss58_address} (id={identity_id})")
        return

    call = substrate.compose_call(
        call_module="Identity",
        call_function="register_identity",
        call_params={"name": target.name},
    )
    _submit_extrinsic(
        substrate,
        target_keypair,
        call,
        f"register_identity({target_keypair.ss58_address}, name={target.name!r})",
        dry_run=dry_run,
    )
    planned_identities.add(target_keypair.ss58_address)


def _ensure_issuer_authorized(
    substrate: SubstrateInterface,
    issuer_keypair: Keypair,
    *,
    attr_name: str,
    sudo_keypair: Keypair | None,
    council_keypair: Keypair | None,
    dry_run: bool,
) -> None:
    config = ATTRIBUTE_CONFIG[attr_name]
    issuers = _query_value(substrate, "Identity", config["issuer_storage"]) or []
    if issuer_keypair.ss58_address in issuers:
        print(f"[skip] issuer already authorized for {attr_name}: {issuer_keypair.ss58_address}")
        return

    call = substrate.compose_call(
        call_module="Identity",
        call_function="add_issuer",
        call_params={
            "attr": config["attr_id"],
            "issuer": issuer_keypair.ss58_address,
        },
    )
    _submit_admin_call(
        substrate,
        call,
        f"add_issuer(attr={attr_name}, issuer={issuer_keypair.ss58_address})",
        sudo_keypair=sudo_keypair,
        council_keypair=council_keypair,
        dry_run=dry_run,
    )


def _ensure_attestation(
    substrate: SubstrateInterface,
    issuer_keypair: Keypair,
    target: Target,
    target_keypair: Keypair,
    *,
    attr_name: str,
    planned_identities: set[str],
    dry_run: bool,
) -> None:
    config = ATTRIBUTE_CONFIG[attr_name]
    identity_id = _query_value(substrate, "Identity", "IdentityOf", [target_keypair.ss58_address])
    if identity_id is None:
        if target_keypair.ss58_address not in planned_identities:
            raise RuntimeError(
                f"Cannot issue {attr_name} for {target_keypair.ss58_address}: identity not registered"
            )
        existing = None
    else:
        existing = _query_value(substrate, "Identity", config["attestation_storage"], [identity_id])

    existing_status = _enum_name(existing.get("status")) if isinstance(existing, dict) else None
    if existing_status == "Active":
        print(
            f"[skip] {attr_name} attestation already active for {target_keypair.ss58_address} "
            f"(identity={identity_id})"
        )
        return

    call = substrate.compose_call(
        call_module="Identity",
        call_function=config["issue_call"],
        call_params={
            "target": target_keypair.ss58_address,
            "hash": getattr(target, config["hash_field"]),
            "anchor": getattr(target, config["anchor_field"]),
            "format_ok": True,
        },
    )
    _submit_extrinsic(
        substrate,
        issuer_keypair,
        call,
        f"{config['issue_call']}({target_keypair.ss58_address})",
        dry_run=dry_run,
    )


def main() -> int:
    args = _parse_args()

    try:
        targets = _load_targets(args.input)
    except Exception as exc:
        print(f"Manifest error: {exc}", file=sys.stderr)
        return 2

    ws_url = os.getenv("BLOCKCHAIN_WS_URL", "ws://127.0.0.1:9944")
    issuer_suri = os.getenv("ISSUER_SURI")
    sudo_suri = os.getenv("SUDO_SURI")
    council_suri = os.getenv("COUNCIL_SURI")

    if not issuer_suri:
        print("Missing required env var: ISSUER_SURI", file=sys.stderr)
        return 2
    if sudo_suri is None and council_suri is None:
        print(
            "Provide either SUDO_SURI (preferred) or COUNCIL_SURI for admin-origin calls",
            file=sys.stderr,
        )
        return 2
    if args.bootstrap_council and (sudo_suri is None or council_suri is None):
        print(
            "--bootstrap-council requires both SUDO_SURI and COUNCIL_SURI",
            file=sys.stderr,
        )
        return 2

    try:
        issuer_keypair = Keypair.create_from_uri(issuer_suri)
        sudo_keypair = Keypair.create_from_uri(sudo_suri) if sudo_suri else None
        council_keypair = Keypair.create_from_uri(council_suri) if council_suri else None
    except Exception as exc:
        print(f"Failed to create keypair(s): {exc}", file=sys.stderr)
        return 2

    substrate = SubstrateInterface(url=ws_url)
    print(f"Connected to {ws_url}")
    print(f"  chain    = {substrate.chain}")
    print(f"  runtime  = {substrate.runtime_version}")
    print(f"  issuer   = {issuer_keypair.ss58_address}")
    if sudo_keypair is not None:
        print(f"  admin    = root via {sudo_keypair.ss58_address}")
    elif council_keypair is not None:
        print(f"  admin    = TechnicalCouncil propose via {council_keypair.ss58_address}")
    print(f"  targets  = {len(targets)}")
    if args.dry_run:
        print("  mode     = dry-run")

    try:
        if args.bootstrap_council and sudo_keypair and council_keypair:
            print("[step] bootstrap TechnicalCouncil membership")
            _ensure_council_bootstrap(
                substrate,
                sudo_keypair,
                council_keypair,
                dry_run=args.dry_run,
            )

        print("[step] ensure compliance verification")
        target_keypairs: list[tuple[Target, Keypair]] = []
        planned_identities: set[str] = set()
        for target in targets:
            keypair = Keypair.create_from_uri(target.suri)
            target_keypairs.append((target, keypair))
            _ensure_compliance(
                substrate,
                target,
                keypair,
                sudo_keypair=sudo_keypair,
                council_keypair=council_keypair,
                dry_run=args.dry_run,
            )

        print("[step] ensure identity registration")
        for target, keypair in target_keypairs:
            _ensure_identity_registered(
                substrate,
                target,
                keypair,
                planned_identities=planned_identities,
                dry_run=args.dry_run,
            )

        print("[step] ensure issuer authorization")
        for attr_name in ATTRIBUTE_CONFIG:
            _ensure_issuer_authorized(
                substrate,
                issuer_keypair,
                attr_name=attr_name,
                sudo_keypair=sudo_keypair,
                council_keypair=council_keypair,
                dry_run=args.dry_run,
            )

        print("[step] ensure active identity attestations")
        for target, keypair in target_keypairs:
            for attr_name in ATTRIBUTE_CONFIG:
                _ensure_attestation(
                    substrate,
                    issuer_keypair,
                    target,
                    keypair,
                    attr_name=attr_name,
                    planned_identities=planned_identities,
                    dry_run=args.dry_run,
                )

        print("KYC seeding complete")
        return 0
    except Exception as exc:
        print(f"KYC seeding failed: {exc}", file=sys.stderr)
        return 1
    finally:
        try:
            substrate.close()
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())