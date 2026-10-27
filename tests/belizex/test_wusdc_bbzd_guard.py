"""Oracle-guard behaviour for the WUSDC/BBZD pair on the real runtime.

These tests describe *policy*, so they state what the runtime actually does
rather than what an earlier design intended. Two verified facts drive them:

1. `BelizeXAssetLedger` settles WUSDC in `pallet_belize_belizex::WusdcBalances`
   (`runtime/src/lib.rs`). WUSDC enters circulation only through the
   governance-gated `BelizeX.issue_wusdc`, and the pair can therefore hold a
   real reserve. Until that ledger existed, `balance_of` returned 0, `transfer`
   returned `UnsupportedAsset`, and this pair was unseedable.
2. The pallet's oracle deviation guard is scoped to exactly this pair
   (`pallets/belizex/src/lib.rs`, `execute_trade` and the multihop hop). The
   scope is deliberate: WUSDC represents USD, so the Oracle's USD/BZD rate is
   the correct reference. DALLA floats, so applying this guard to DALLA/BBZD
   would reject every legitimate trade.

Because (1) is now fixed, the guard is reachable and these tests assert that it
actually fires — they are no longer `xfail`.
"""

import pytest

BBZD_ASSET = 1
DALLA_ASSET = 0
WUSDC_ASSET = 3

ONE_DALLA = 1_000_000_000_000  # 12 decimals

# `Oracle.Currency` discriminants used by `update_exchange_rate`.
CURRENCY_BZD = 0
CURRENCY_USD = 1


def _set_usd_bzd_rate(submit_sudo_extrinsic, price: int) -> None:
    res = submit_sudo_extrinsic(
        pallet="Oracle",
        call="update_exchange_rate",
        params={
            "base_currency": CURRENCY_USD,
            "quote_currency": CURRENCY_BZD,
            "price": price,
        },
    )
    assert res["success"], f"Oracle rate update failed: {res.get('error')}"


def _execute_trade(substrate, keypair, base_asset: int, quote_asset: int, amount_in: int):
    call = substrate.compose_call(
        call_module="BelizeX",
        call_function="execute_trade",
        call_params={
            "base_asset": base_asset,
            "quote_asset": quote_asset,
            "amount_in": amount_in,
            "min_amount_out": 1,
            "is_tourism_trade": False,
        },
    )
    return substrate.submit_extrinsic(
        substrate.create_signed_extrinsic(call=call, keypair=keypair),
        wait_for_inclusion=True,
    )


@pytest.mark.requires_blockchain
class TestWusdcBbzdOracleGuard:
    def test_wusdc_pair_can_be_seeded_once_both_legs_are_funded(
        self,
        submit_sudo_extrinsic,
        verify_kyc,
        fund_bbzd,
        fund_wusdc,
        blockchain_connection,
        alice_keypair,
    ):
        """WUSDC now has a ledger, so a WUSDC/BBZD pool can hold a real reserve."""
        substrate = blockchain_connection

        submit_sudo_extrinsic(
            pallet="BelizeX",
            call="create_trading_pair",
            params={
                "base_asset": WUSDC_ASSET,
                "quote_asset": BBZD_ASSET,
                "fee_rate": 30,
            },
        )
        verify_kyc(alice_keypair, 1)
        # 10 WUSDC + 20 bBZD at a 2.0 price matches the Oracle's BZD peg.
        fund_wusdc(alice_keypair, 10 * ONE_DALLA)
        fund_bbzd(alice_keypair, 20 * ONE_DALLA)

        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": WUSDC_ASSET,
                "quote_asset": BBZD_ASSET,
                "base_amount": 10 * ONE_DALLA,
                "quote_amount": 20 * ONE_DALLA,
                "min_lp_tokens": 1,
            },
        )
        receipt = substrate.submit_extrinsic(
            substrate.create_signed_extrinsic(call=call, keypair=alice_keypair),
            wait_for_inclusion=True,
        )

        assert receipt.is_success, (
            "WUSDC/BBZD liquidity deposit must succeed now that WUSDC has a "
            f"ledger, got: {getattr(receipt, 'error_message', None)}"
        )

        # `TradingPairs` is keyed by a single `(base, quote)` tuple.
        pair = substrate.query(
            "BelizeX", "TradingPairs", [[WUSDC_ASSET, BBZD_ASSET]]
        )
        assert pair.value is not None, "the pair must exist after listing"
        assert int(pair.value["base_reserve"]) >= 10 * ONE_DALLA
        assert int(pair.value["quote_reserve"]) >= 20 * ONE_DALLA

    def test_trade_rejects_on_excessive_deviation(
        self,
        submit_sudo_extrinsic,
        verify_kyc,
        fund_bbzd,
        fund_wusdc,
        blockchain_connection,
        alice_keypair,
    ):
        """A grossly deviant USD/BZD rate must be rejected, not silently accepted."""
        substrate = blockchain_connection

        # 10 BZD per USD against a real pool priced at 2.0 is a 400% deviation,
        # far beyond `MaxOracleDeviationBps`.
        _set_usd_bzd_rate(submit_sudo_extrinsic, 10_000_000)

        verify_kyc(alice_keypair, 1)
        fund_wusdc(alice_keypair, 10 * ONE_DALLA)
        fund_bbzd(alice_keypair, 20 * ONE_DALLA)

        receipt = _execute_trade(
            substrate, alice_keypair, WUSDC_ASSET, BBZD_ASSET, ONE_DALLA
        )

        assert not receipt.is_success, (
            "A 400% oracle deviation was accepted — the deviation guard did not run"
        )
        assert "SlippageExceeded" in str(getattr(receipt, "error_message", "")), (
            f"Expected SlippageExceeded, got: {getattr(receipt, 'error_message', None)}"
        )

    @pytest.mark.pristine_oracle_rate
    def test_trade_blocks_when_oracle_rate_unavailable(
        self,
        verify_kyc,
        fund_bbzd,
        fund_wusdc,
        blockchain_connection,
        alice_keypair,
    ):
        """The guard fails closed: no USD/BZD rate means no WUSDC trade.

        `Oracle.update_exchange_rate` has no inverse, so this is only observable on
        a chain where no rate has been written yet — and every later suite that
        needs a rate writes the same global key. That makes the assertion
        order-dependent rather than chain-dependent, so the test carries
        `pristine_oracle_rate`: the CI job runs it as its own first step and the
        shared run excludes it, instead of it silently skipping there.
        """
        substrate = blockchain_connection

        existing = substrate.query(
            "Oracle",
            "ManualExchangeRates",
            [{"base": "USD", "quote": "BZD"}],
        )
        if existing.value:
            pytest.skip(
                "a USD/BZD rate is already set on this chain; the fail-closed "
                "path is covered by unit tests in pallets/belizex"
            )

        verify_kyc(alice_keypair, 1)
        fund_wusdc(alice_keypair, 10 * ONE_DALLA)
        fund_bbzd(alice_keypair, 20 * ONE_DALLA)

        receipt = _execute_trade(
            substrate, alice_keypair, WUSDC_ASSET, BBZD_ASSET, ONE_DALLA
        )

        assert not receipt.is_success, (
            "A trade executed with no Oracle rate — the guard did not fail closed"
        )
        assert "OracleRateUnavailable" in str(
            getattr(receipt, "error_message", "")
        ), f"Expected OracleRateUnavailable, got: {receipt.error_message}"

    def test_trade_requires_kyc(self, blockchain_connection, fund_account):
        """Basic KYC gates trading regardless of the oracle guard."""
        from substrateinterface import Keypair

        substrate = blockchain_connection
        # A fresh account with no KYC — `//Eve` is funded by dev genesis but
        # unverified; fund it again so fee payment cannot mask the KYC error.
        unverified = Keypair.create_from_uri("//Eve")
        fund_account(unverified, 100 * ONE_DALLA)

        receipt = _execute_trade(
            substrate, unverified, DALLA_ASSET, BBZD_ASSET, ONE_DALLA
        )
        assert not receipt.is_success, "an unverified account must not trade"
        assert "KycRequired" in str(getattr(receipt, "error_message", "")), (
            f"Expected KycRequired, got: {receipt.error_message}"
        )

    def test_pause_blocks_trades(
        self, submit_sudo_extrinsic, verify_kyc, blockchain_connection, alice_keypair
    ):
        """A global pause halts trading until resumed.

        KYC is asserted first because the pallet checks it *before* the pause
        flag — relying on an earlier test to have verified Alice makes this
        test pass or fail depending on run order.
        """
        substrate = blockchain_connection
        verify_kyc(alice_keypair, 1)

        res = submit_sudo_extrinsic(pallet="BelizeX", call="pause_global", params={})
        assert res["success"], f"pause_global failed: {res.get('error')}"

        try:
            receipt = _execute_trade(
                substrate, alice_keypair, DALLA_ASSET, BBZD_ASSET, ONE_DALLA
            )
            assert not receipt.is_success, "a paused DEX must not trade"
            assert "Paused" in str(getattr(receipt, "error_message", "")), (
                f"Expected Paused, got: {receipt.error_message}"
            )
        finally:
            resumed = submit_sudo_extrinsic(
                pallet="BelizeX", call="resume_global", params={}
            )
            assert resumed["success"], (
                f"resume_global failed, DEX left paused: {resumed.get('error')}"
            )
