import pytest
from substrateinterface import SubstrateInterface, Keypair

class TestBbzdMinimalIntegration:
    def test_authorized_mint_increases_balance_and_supply(
        self,
        submit_sudo_extrinsic,
        blockchain_connection: SubstrateInterface,
        alice_keypair: Keypair,
        dave_keypair: Keypair,
    ):
        substrate = blockchain_connection
        alice = alice_keypair
        user = dave_keypair

        # Authorize Alice as Central Bank minter
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="set_minter_authorization",
            params={"account": str(alice.ss58_address), "authorized": True},
        )
        assert res["success"], f"Authorize minter failed: {res.get('error')}"

        # Set reserves sufficiently high
        res = submit_sudo_extrinsic(
            pallet="Economy",
            call="update_reserves",
            params={"new_reserves": 10_000_000_000_000},
        )
        assert res["success"], f"Set reserves failed: {res.get('error')}"

        # Ensure Bob has KYC level 1
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
        xt = substrate.create_signed_extrinsic(call=call, keypair=user)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success

        # Read initial balance and supply
        init_balance = substrate.query(
            module="Economy", storage_function="BBZDBalances", params=[user.ss58_address]
        ).value or 0
        init_supply = substrate.query(
            module="Economy", storage_function="TotalBbzdSupply"
        ).value or 0

        # Mint 1000 bBZD to Bob
        mint_amount = 1_000_000_000_000
        call = substrate.compose_call(
            call_module="Economy",
            call_function="mint_bbzd",
            call_params={
                "recipient": str(user.ss58_address),
                "amount": mint_amount,
                "deposit_reference": f"0x{b'MINT'.hex()}",
            },
        )
        xt = substrate.create_signed_extrinsic(call=call, keypair=alice)
        receipt = substrate.submit_extrinsic(xt, wait_for_finalization=True)
        assert receipt.is_success, f"Mint failed: {getattr(receipt, 'error_message', None)}"

        # Verify balance and supply changed
        final_balance = substrate.query(
            module="Economy", storage_function="BBZDBalances", params=[user.ss58_address]
        ).value or 0
        final_supply = substrate.query(
            module="Economy", storage_function="TotalBbzdSupply"
        ).value or 0

        assert final_balance == init_balance + mint_amount
        assert final_supply >= init_supply + mint_amount
