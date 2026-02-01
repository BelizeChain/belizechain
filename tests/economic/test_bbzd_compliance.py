import pytest

@pytest.mark.requires_blockchain
class TestBbzdCompliance:
    def test_mint_requires_kyc(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair, dave_keypair):
        substrate = blockchain_connection
        alice = alice_keypair
        recipient = dave_keypair

        # 1) Governance: authorize Alice as Central Bank minter
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"

        # 2) Governance: set reserves
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": 10_000_000_000_000},
        )
        assert res["success"], f"Set reserves failed: {res.get('error')}"

        # 3) Attempt mint to recipient WITHOUT KYC -> should fail
        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(recipient.ss58_address),
                "amount": 1_000_000_000_000,
                "deposit_reference": f"0x{b'MINT_TEST_NO_KYC'.hex()}",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Mint should fail when recipient lacks KYC"
        err = getattr(receipt, "error_message", None)
        if isinstance(err, dict):
            assert err.get("name") in ("KycVerificationRequired", "KycRequired", "SanctionedEntity"), f"Unexpected error: {err}"
        else:
            assert ("Kyc" in str(err or "")) or ("Sanction" in str(err or "")), f"Expected KYC or Sanction error, got: {err}"

        # 4) Fund recipient minimally to cover fee for signed call
        fund_call = substrate.compose_call(
            call_module="Balances",
            call_function="transfer_keep_alive",
            call_params={
                "dest": {"Id": str(recipient.ss58_address)},
                "value": 1_000_000_000_000,
            },
        )
        fund_xt = substrate.create_signed_extrinsic(call=fund_call, keypair=alice)
        fund_receipt = substrate.submit_extrinsic(fund_xt, wait_for_finalization=True)
        assert fund_receipt.is_success, f"Funding recipient failed: {getattr(fund_receipt, 'error_message', None)}"

        # 5) Set recipient's KYC Level 1 via signed Oracle call
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(recipient.ss58_address),
                "kyc_level": 1,
                "id_hash": "0x" + ("00" * 32),
                "provider": f"0x{b'dev'.hex()}",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=recipient)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Set KYC failed: {getattr(receipt, 'error_message', None)}"

        # 6) Mint to recipient WITH KYC -> should succeed
        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(recipient.ss58_address),
                "amount": 1_000_000_000_000,
                "deposit_reference": f"0x{b'MINT_TEST_WITH_KYC'.hex()}",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Mint with KYC failed: {getattr(receipt, 'error_message', None)}"

    def test_redeem_blocks_for_sanctioned(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair, bob_keypair):
        substrate = blockchain_connection
        alice = alice_keypair
        user = bob_keypair

        # Best-effort: clear any prior sanction on user
        try:
            res = submit_sudo_extrinsic(
                pallet="Oracle",
                call="remove_sanction",
                params={"account": str(user.ss58_address)},
            )
        except Exception:
            pass

        # Ensure Alice is authorized and reserves sufficient
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": 10_000_000_000_000},
        )
        assert res["success"], f"Set reserves failed: {res.get('error')}"

        # Fund user with minimal bBZD by minting (requires KYC first)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(user.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=user)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success

        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(user.ss58_address),
                "amount": 500_000_000_000,
                "deposit_reference": f"0x{b'SEED_FOR_REDEEM'.hex()}",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success

        # Governance (Oracle admin) marks user as sanctioned
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="add_sanctioned_entity",
            params={
                "account": str(user.ss58_address),
                "source": 0,  # OFAC
                "reason": f"0x{b'Test sanction'.hex()}",
                "expires_at": None,
            },
        )
        assert res["success"], f"Add sanction failed: {res.get('error')}"

        # User attempts to redeem -> should fail
        call = substrate.compose_call(
            call_module="Economy",
            call_function="redeem_bbzd",
            call_params={
                "amount": 100_000_000_000,
                "bank_account": f"0x{b'CHARLIE_BANK'.hex()}",
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=user)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Redeem should fail for sanctioned account"
        err = getattr(receipt, "error_message", None)
        if isinstance(err, dict):
            assert err.get("name") in ("SanctionedEntity", "AccountSanctioned"), f"Unexpected error: {err}"
        else:
            assert "Sanction" in str(err or ""), f"Expected sanction error, got: {err}"
