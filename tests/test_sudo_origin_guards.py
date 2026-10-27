"""Sudo must not launder an origin check that a pallet performs itself.

`Oracle::verify_identity` is `ensure_signed` *and* requires an `OracleOperators`
entry, so `Sudo.sudo(Oracle.verify_identity(..))` returns `BadOrigin`. A wrapper
that reported success anyway would be the exact failure mode these tests exist to
catch: a call that looks like it worked and changed nothing.
"""

import pytest


class TestSudoOriginGuards:
    @pytest.mark.requires_blockchain
    def test_sudo_verify_identity_reports_failure(
        self, submit_sudo_extrinsic, alice_keypair, bob_keypair
    ):
        """`Sudo.sudo` cannot stand in for an Oracle operator."""
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(bob_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": "0x" + "07" * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        assert res["success"] is False, f"sudo must not launder BadOrigin: {res['error']}"
        assert "BadOrigin" in str(res["error"]), res["error"]

    @pytest.mark.requires_blockchain
    def test_verify_kyc_smoke(self, verify_kyc, charlie_keypair):
        """Smoke check only: the fixture raises on a bad quorum.

        It returns nothing, so there is no postcondition to assert here — the
        assertion lives inside `verify_kyc`. Kept as its own case so a regression
        there surfaces as a named failure instead of as collateral damage in
        whichever test happened to run first.
        """
        verify_kyc(charlie_keypair, 1)
