import pytest


@pytest.mark.requires_blockchain
class TestEconomyCompliance:
    def test_mint_requires_kyc(self, blockchain_connection, submit_sudo_extrinsic, alice_keypair, bob_keypair):
        """
        Minting bBZD should require recipient KYC >= 1. Without KYC, expect KycVerificationRequired.
        After setting KYC via a signed Oracle call, mint should succeed.
        """
        substrate = blockchain_connection

        # 1) Authorize Alice as Central Bank minter (governance-only)
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="set_minter_authorization",
            params={
                "account": str(alice_keypair.ss58_address),
                "authorized": True,
            },
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"

        # 2) Set reserves high enough to allow minting
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="update_reserves",
            params={
                "new_reserves": 1_000_000_000_000,
            },
        )
        assert res["success"], f"Update reserves failed: {res.get('error')}"

        # 3) Attempt mint to Bob WITHOUT KYC -> expect failure
        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(bob_keypair.ss58_address),
                "amount": 1_000_000_000,
                "deposit_reference": "test-deposit",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Mint should fail when recipient lacks KYC"
        assert any(ev.event_module == "System" and ev.event_name == "ExtrinsicFailed" for ev in receipt.triggered_events), (
            f"Expected ExtrinsicFailed, got: {[ (e.event_module, e.event_name) for e in receipt.triggered_events ]}"
        )

        # 4) Set Bob's KYC Level 1 via signed Oracle call
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Set KYC failed: {getattr(receipt, 'error_message', None)}"

        # 5) Retry mint -> should succeed now
        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(bob_keypair.ss58_address),
                "amount": 2_000_000_000,
                "deposit_reference": "test-deposit-2",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Mint should succeed after KYC: {getattr(receipt, 'error_message', None)}"

    def test_mint_blocks_sanctioned(self, blockchain_connection, submit_sudo_extrinsic, alice_keypair, bob_keypair):
        """
        Minting bBZD should be blocked if recipient is sanctioned.
        """
        substrate = blockchain_connection

        # Ensure Alice is authorized minter and reserves set
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="set_minter_authorization",
            params={"account": str(alice_keypair.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="update_reserves",
            params={"new_reserves": 1_000_000_000_000},
        )
        assert res["success"], f"Update reserves failed: {res.get('error')}"

        # Sanction Bob via Oracle (admin/root)
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="add_sanctioned_entity",
            params={
                "account": str(bob_keypair.ss58_address),
                "source": 1,  # e.g., OFAC
                "reason": "test-sanction",
                "expires_at": None,
            },
        )
        assert res["success"], f"Add sanction failed: {res.get('error')}"

        # Attempt mint to Bob -> should fail with sanction
        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(bob_keypair.ss58_address),
                "amount": 1_000_000_000,
                "deposit_reference": "sanctioned-deposit",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Mint should fail for sanctioned recipient"
        assert any(ev.event_module == "System" and ev.event_name == "ExtrinsicFailed" for ev in receipt.triggered_events)

    def test_redeem_requires_kyc_and_not_sanctioned(self, blockchain_connection, submit_sudo_extrinsic, alice_keypair, bob_keypair):
        """
        Redeeming bBZD should require KYC >= 1 and non-sanctioned status.
        """
        substrate = blockchain_connection

        # Setup: authorize minter, set reserves, mint bBZD to Bob
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="set_minter_authorization",
            params={"account": str(alice_keypair.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"
        res = submit_sudo_extrinsic(
            pallet="BelizeEconomy",
            call="update_reserves",
            params={"new_reserves": 1_000_000_000_000},
        )
        assert res["success"], f"Update reserves failed: {res.get('error')}"
        # Mint some bBZD to Bob (use KYC first to allow mint)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(bob_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, "Setting KYC should succeed"
        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(bob_keypair.ss58_address),
                "amount": 5_000_000_000,
                "deposit_reference": "redeem-setup",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, "Mint for setup should succeed"

        # 1) Sanction Bob and attempt redeem -> expect failure
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="add_sanctioned_entity",
            params={
                "account": str(bob_keypair.ss58_address),
                "source": 1,
                "reason": "redeem-test",
                "expires_at": None,
            },
        )
        assert res["success"], f"Add sanction failed: {res.get('error')}"

        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="redeem_bbzd",
            call_params={
                "amount": 1_000_000_000,
                "bank_account": "test-bank-account",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Redeem should fail for sanctioned account"

        # 2) Remove sanction and redeem -> should succeed
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="remove_sanction",
            params={
                "account": str(bob_keypair.ss58_address),
            },
        )
        assert res["success"], f"Remove sanction failed: {res.get('error')}"

        call = substrate.compose_call(
            call_module="BelizeEconomy",
            call_function="redeem_bbzd",
            call_params={
                "amount": 1_000_000_000,
                "bank_account": "test-bank-account-2",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=bob_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Redeem should succeed after sanction removal: {getattr(receipt, 'error_message', None)}"
