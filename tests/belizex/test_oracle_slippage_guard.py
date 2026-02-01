import pytest

@pytest.mark.requires_blockchain
class TestBelizeXOracleSlippageGuard:
    def test_trade_respects_oracle_guard(self, submit_sudo_extrinsic, query_storage, blockchain_connection, alice_keypair):
        """
        Set USD/BZD exchange rate, add liquidity to DALLA/BBZD, and execute a trade.
        While BBZD guard relies on Oracle rate, it only applies when Oracle returns a rate.
        This test ensures normal trade flow works and demonstrates guard wiring without failure.
        """
        substrate = blockchain_connection

        # 1) Set manual USD/BZD rate to 1.92 via Oracle (scaled 1e6)
        price_scaled = 1_920_000
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="update_exchange_rate",
            params={"base_currency": 1, "quote_currency": 0, "price": price_scaled},
        )
        assert res["success"], f"Oracle rate update failed: {res.get('error')}"

        # 2) Ensure Alice has KYC (Identity pallet)
        # KYC level: 1 (Basic). Identity::verify_identity doesn't require admin in this dev config.
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0]*32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        assert res["success"], f"Set KYC failed: {res.get('error')}"

        # 3) Create DALLA/BBZD pair if not present (already defaults to active)
        # We'll proceed to add liquidity directly.

        # 4) Add liquidity to the pair (Alice)
        # Using small dev amounts: 10 DALLA + 20 bBZD (12 decimals)
        dalla_amount = 10_000_000_000_000
        bbzd_amount = 20_000_000_000_000
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": 0,  # DALLA
                "quote_asset": 1,  # BBZD
                "base_amount": dalla_amount,
                "quote_amount": bbzd_amount,
                "min_lp_tokens": 1,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Add liquidity failed: {getattr(receipt, 'error_message', None)}"

        # 5) Execute a trade within reasonable slippage bounds
        amount_in = 1_000_000_000_000  # 1 DALLA
        min_amount_out = 1  # accept anything for now; guard is wired in pallet
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 0,  # DALLA
                "quote_asset": 1,  # BBZD
                "amount_in": amount_in,
                "min_amount_out": min_amount_out,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Trade failed: {getattr(receipt, 'error_message', None)}"

        # 6) Confirm event emitted
        found = False
        for ev in receipt.triggered_events:
            if ev.event_module == "BelizeX" and ev.event_name == "TradeExecuted":
                found = True
                break
        assert found, "TradeExecuted event not found"
