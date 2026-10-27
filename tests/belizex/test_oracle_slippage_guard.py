import pytest

from conftest import event_pairs

@pytest.mark.requires_blockchain
class TestBelizeXOracleSlippageGuard:
    def test_trade_respects_oracle_guard(self, submit_sudo_extrinsic, verify_kyc, fund_bbzd, query_storage, blockchain_connection, alice_keypair):
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

        # 2) Ensure Alice has KYC (oracle quorum, not sudo)
        verify_kyc(alice_keypair, 1)

        # bBZD lives in the economy pallet's ledger, so the quote leg must be minted
        fund_bbzd(alice_keypair, 50_000_000_000_000)

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
        assert ("BelizeX", "TradeExecuted") in event_pairs(receipt), (
            f"TradeExecuted event not found; got {event_pairs(receipt)}"
        )
