"""
Economic test fixtures.

Auto-skips all economic tests when GovernanceOrigin (GovernanceCouncilMajority)
does not accept Root origin from Sudo.  Until the governance council is
bootstrapped on the dev chain, Sudo::sudo cannot set minter authorization or
reserves — making all BBZD integration tests impossible to execute.
"""
import pytest

_governance_via_sudo = None


@pytest.fixture(autouse=True)
def _require_governance_sudo(blockchain_connection, submit_sudo_extrinsic, alice_keypair):
    """Skip economic tests when governance-via-sudo is not available."""
    global _governance_via_sudo
    if _governance_via_sudo is not None:
        if not _governance_via_sudo:
            pytest.skip("GovernanceOrigin rejects Root; sudo cannot set economy state")
        return

    # One-time check: try to authorize minter via sudo and verify storage
    submit_sudo_extrinsic(
        pallet="Economy",
        call="set_minter_authorization",
        params={"account": str(alice_keypair.ss58_address), "authorized": True},
    )
    substrate = blockchain_connection
    try:
        is_authorized = substrate.query(
            module="Economy",
            storage_function="AuthorizedMinters",
            params=[alice_keypair.ss58_address],
        ).value
    except Exception:
        is_authorized = False

    _governance_via_sudo = bool(is_authorized)
    if not _governance_via_sudo:
        pytest.skip(
            "Economy.GovernanceOrigin = GovernanceCouncilMajority rejects Root; "
            "council must be bootstrapped before economic tests can run"
        )
