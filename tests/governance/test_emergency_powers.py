"""Jaguar Mode (national emergency) governance, against the real pallet.

Activation is two-phase, not a switch:

1. `declare_emergency(type_index, description, duration_hours)` writes
   `JaguarMode` with `active: false, is_pending: true` and a
   `veto_window_ends_at` of `now + EmergencyVetoWindow` (14_400 blocks, ~24 h).
2. `veto_emergency()` — council members only — records a veto in
   `EmergencyVetoes`. Once vetoes reach `council_count / 3 + 1`, `JaguarMode` is
   killed outright and the veto records are cleared.
3. If the window elapses with too few vetoes, `on_initialize` flips the record to
   `active: true, is_pending: false`.
4. `end_emergency()` — `CouncilOrigin` or root — clears it. Both an `active` and
   a `pending` record can be ended.

Phase 3 needs 14_400 blocks, so it is not reachable here; the pallet's unit
tests cover it.

`JaguarMode` is a single global record, so every test that declares one must end
it. Leaving it set makes all later declarations fail, which is exactly how the
previous version of this file produced eight skips instead of assertions — it
tested `activate_jaguarmode`, `EmergencyBoardMembers`, `pause_transactions` and
`whitelist_operations`, none of which exist.
"""

import contextlib

import pytest
from substrateinterface import Keypair

TYPE_HURRICANE = 0
TYPE_OTHER = 5

BLOCKS_PER_HOUR = 600

# `Governance` error index 72 (`NoActiveEmergency`), as `submit_sudo_extrinsic`
# reports inner errors still SCALE-encoded instead of decoded.
NO_ACTIVE_EMERGENCY = "0x48000000"



@contextlib.contextmanager
def declared_emergency(
    submit_sudo_extrinsic,
    emergency_type_index=TYPE_HURRICANE,
    duration_hours=1,
    description=b"test emergency",
):
    """Declare an emergency, yielding inside the window, and always end it."""
    declared = submit_sudo_extrinsic(
        pallet="Governance",
        call="declare_emergency",
        params={
            "emergency_type_index": emergency_type_index,
            "description": "0x" + description.hex(),
            "duration_hours": duration_hours,
        },
    )
    if not declared["success"]:
        pytest.skip(
            "declare_emergency refused — an emergency is already pending or "
            f"active: {declared.get('error')}"
        )
    try:
        yield declared
    finally:
        submit_sudo_extrinsic(pallet="Governance", call="end_emergency", params={})


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyDeclaration:
    """What a declaration writes, and what it deliberately does not."""

    def test_declaration_is_pending_not_active(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """A declaration must not activate emergency powers on its own."""
        substrate = blockchain_connection

        with declared_emergency(
            submit_sudo_extrinsic,
            TYPE_HURRICANE,
            duration_hours=24,
            description=b"Category 4 landfall, Belize District",
        ):
            status = substrate.query("Governance", "JaguarMode").value
            assert status is not None, (
                "declare_emergency succeeded but wrote no JaguarMode entry"
            )
            assert status["is_pending"] is True, (
                "a declared emergency starts in the veto window, so is_pending "
                f"must be true, got {status['is_pending']}"
            )
            assert status["active"] is False, (
                "a declaration must not activate emergency powers on its own"
            )
            assert status["emergency_type"] == "Hurricane"
            assert int(status["veto_window_ends_at"]) > int(status["declared_at"]), (
                "the veto window must end after the declaration block"
            )

            expected_duration = BLOCKS_PER_HOUR * 24
            actual_duration = int(status["expires_at"]) - int(status["declared_at"])
            assert actual_duration == expected_duration, (
                f"24h should be {expected_duration} blocks, got {actual_duration}"
            )

    def test_unknown_type_index_falls_back_to_other(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """Indices above 4 are not an error — they map to `EmergencyType::Other`.

        Worth pinning down because it is inconsistent with the rest of the
        pallet: every other index argument here is rejected when out of range.
        """
        substrate = blockchain_connection

        with declared_emergency(
            submit_sudo_extrinsic, 200, description=b"unmapped type"
        ):
            status = substrate.query("Governance", "JaguarMode").value
            assert status["emergency_type"] == "Other", (
                f"index 200 should map to Other, got {status['emergency_type']}"
            )

    def test_root_declaration_records_no_declarer(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """A root-authorised declaration leaves `declared_by` empty."""
        substrate = blockchain_connection

        with declared_emergency(
            submit_sudo_extrinsic, TYPE_OTHER, description=b"attribution check"
        ):
            status = substrate.query("Governance", "JaguarMode").value
            # `declare_emergency` accepts `CouncilOrigin` *or* root, and sudo
            # presents root, so no signed declarer is attributed. That is the
            # correct behaviour for a root-authorised declaration — the field
            # only carries an account when a council member signs directly.
            assert status["declared_by"] is None, (
                "a root-authorised declaration must not claim an account, got "
                f"{status['declared_by']}"
            )


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyVeto:
    """The veto window and its threshold."""

    def test_veto_requires_council_membership(
        self, submit_sudo_extrinsic, submit_extrinsic, fund_account, verify_kyc,
        blockchain_connection,
    ):
        """A non-council account cannot veto."""
        substrate = blockchain_connection

        outsider = Keypair.create_from_uri("//Bob")
        fund_account(outsider, 100 * 1_000_000_000_000)
        verify_kyc(outsider, 1)

        with declared_emergency(submit_sudo_extrinsic, description=b"membership"):
            receipt = submit_extrinsic(
                "Governance", "veto_emergency", {}, outsider
            )
            assert not receipt.is_success, "a non-council account must not veto"
            message = str(receipt.error_message)
            assert (
                "NotCouncilMember" in message or "AlreadyVetoed" in message
            ), f"expected NotCouncilMember, got: {receipt.error_message}"

    def test_veto_records_or_cancels_the_emergency(
        self, submit_sudo_extrinsic, submit_extrinsic, council_keypair,
        blockchain_connection,
    ):
        """A council veto either records itself or cancels the declaration.

        With fewer than three council members, `council_count / 3 + 1` is 1, so
        a single veto immediately kills `JaguarMode` and clears
        `EmergencyVetoes`. Both outcomes are correct; which one happens depends
        on the council size on this chain.
        """
        substrate = blockchain_connection

        member = council_keypair()
        if member is None:
            pytest.skip(
                "no dev account holds a council seat on this chain, so no veto "
                "can be signed; run tests/governance/test_council.py first"
            )

        with declared_emergency(
            submit_sudo_extrinsic, description=b"veto target"
        ):
            # Signed by the member: sudo presents Root and would be rejected.
            res = submit_extrinsic("Governance", "veto_emergency", {}, member)
            assert res.is_success, (
                f"veto_emergency failed: {res.error_message}"
            )

            vetoes = [
                key.value
                for key, _ in substrate.query_map("Governance", "EmergencyVetoes")
            ]
            still_declared = substrate.query("Governance", "JaguarMode").value

            if still_declared is None:
                # The veto met the threshold and cancelled the declaration.
                assert not vetoes, (
                    "cancelling an emergency must also clear the veto records"
                )
            else:
                assert vetoes, (
                    "a successful veto on a surviving emergency must be recorded"
                )
                assert still_declared["is_pending"] is True


@pytest.mark.requires_blockchain
@pytest.mark.governance
class TestEmergencyEnd:
    """`end_emergency` clears the record and resumes normal governance."""

    def test_end_clears_a_pending_emergency(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """A *pending* emergency can be ended, not only an active one."""
        substrate = blockchain_connection

        declared = submit_sudo_extrinsic(
            pallet="Governance",
            call="declare_emergency",
            params={
                "emergency_type_index": TYPE_HURRICANE,
                "description": "0x" + b"to be ended".hex(),
                "duration_hours": 1,
            },
        )
        if not declared["success"]:
            pytest.skip("an emergency is already pending or active on this chain")

        assert substrate.query("Governance", "JaguarMode").value is not None

        ended = submit_sudo_extrinsic(
            pallet="Governance", call="end_emergency", params={}
        )
        assert ended["success"], f"end_emergency failed: {ended.get('error')}"
        assert substrate.query("Governance", "JaguarMode").value is None, (
            "end_emergency must remove the JaguarMode record so normal "
            "governance resumes"
        )

    def test_end_without_an_emergency_is_rejected(
        self, submit_sudo_extrinsic, blockchain_connection
    ):
        """Ending when nothing is declared is an error, not a silent no-op."""
        substrate = blockchain_connection
        if substrate.query("Governance", "JaguarMode").value is not None:
            pytest.skip("an emergency is currently pending or active")

        res = submit_sudo_extrinsic(
            pallet="Governance", call="end_emergency", params={}
        )
        assert not res["success"], "ending a non-existent emergency must fail"
        assert (
            "NoActiveEmergency" in str(res.get("error"))
            or NO_ACTIVE_EMERGENCY in str(res.get("error"))
        ), f"expected NoActiveEmergency (index 72), got: {res.get('error')}"
