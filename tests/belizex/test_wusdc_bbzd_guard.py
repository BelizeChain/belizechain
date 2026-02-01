import pytest

@pytest.mark.requires_blockchain
class TestWusdcBbzdOracleGuard:
    def test_wusdc_bbzd_trade_success_with_oracle(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair):
        """
        Create WUSDC/BBZD pair, set USD/BZD rate to ~2.0, add liquidity at ~2.0,
        then trade 1 WUSDC -> should succeed and emit TradeExecuted.
        """
        substrate = blockchain_connection

        # 1) Create WUSDC/BBZD pair via governance (sudo)
        res = submit_sudo_extrinsic(
            pallet="BelizeX",
            call="create_trading_pair",
            params={
                "base_asset": 3,  # WUSDC
                "quote_asset": 1,  # BBZD
                "fee_rate": 30,
            },
        )
        # Pair may already exist via genesis; ignore failure here

        # 2) Set USD/BZD exchange rate to 2.0 (scaled 1e6)
        price_scaled = 2_000_000
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="update_exchange_rate",
            params={"base_currency": 1, "quote_currency": 0, "price": price_scaled},
        )
        assert res["success"], f"Oracle rate update failed: {res.get('error')}"

        # 3) Ensure Alice KYC L1
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        assert res["success"], f"Set KYC failed: {res.get('error')}"

        # 4) Add liquidity WUSDC/BBZD ~ (10, 20)
        dalla_amount = 10_000_000_000_000  # Using chain balances for both sides (dev simplification)
        bbzd_amount = 20_000_000_000_000
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": 3,  # WUSDC
                "quote_asset": 1,  # BBZD
                "base_amount": dalla_amount,
                "quote_amount": bbzd_amount,
                "min_lp_tokens": 1,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Add liquidity failed: {getattr(receipt, 'error_message', None)}"

        # 5) Trade 1 WUSDC -> BBZD should pass under guard
        amount_in = 1_000_000_000_000
        min_amount_out = 1
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "amount_in": amount_in,
                "min_amount_out": min_amount_out,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Trade failed: {getattr(receipt, 'error_message', None)}"
        assert any(ev.event_module == "BelizeX" and ev.event_name == "TradeExecuted" for ev in receipt.triggered_events)

    def test_wusdc_bbzd_trade_rejected_on_deviation(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair):
        """
        Set Oracle USD/BZD=10.0 but pool ~2.0; guard should reject WUSDC->BBZD trade.
        """
        substrate = blockchain_connection

        # Ensure pair exists (idempotent create; ignore if already exists)
        res = submit_sudo_extrinsic(
            pallet="BelizeX",
            call="create_trading_pair",
            params={
                "base_asset": 3,
                "quote_asset": 1,
                "fee_rate": 30,
            },
        )
        # We don't assert here to allow re-runs where pair already exists

        # Set high Oracle rate 10.0
        price_scaled = 10_000_000
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="update_exchange_rate",
            params={"base_currency": 1, "quote_currency": 0, "price": price_scaled},
        )
        assert res["success"], f"Oracle rate update failed: {res.get('error')}"

        # KYC
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        assert res["success"], f"Set KYC failed: {res.get('error')}"

        # Seed pool ~2.0
        dalla_amount = 10_000_000_000_000
        bbzd_amount = 20_000_000_000_000
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "base_amount": dalla_amount,
                "quote_amount": bbzd_amount,
                "min_lp_tokens": 1,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Add liquidity failed: {getattr(receipt, 'error_message', None)}"

        # Attempt guarded trade -> expect rejection
        amount_in = 1_000_000_000_000
        min_amount_out = 1
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "amount_in": amount_in,
                "min_amount_out": min_amount_out,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success, "Trade should have been rejected by oracle guard"
        # Expect both ExtrinsicFailed and OracleGuardRejected diagnostics
        assert any(ev.event_module == "BelizeX" and ev.event_name == "OracleGuardRejected" for ev in receipt.triggered_events)
        assert any(ev.event_module == "System" and ev.event_name == "ExtrinsicFailed" for ev in receipt.triggered_events)

    def test_wusdc_bbzd_guard_boundary(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair):
        """
        Boundary tests around 5% guard: 2.1 (exactly +5%) should pass, 2.11 (>5%) should fail.
        """
        substrate = blockchain_connection

        # Ensure pair exists (genesis or create)
        submit_sudo_extrinsic(
            pallet="BelizeX",
            call="create_trading_pair",
            params={"base_asset": 3, "quote_asset": 1, "fee_rate": 30},
        )

        # KYC
        submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )

        # Seed pool ~2.0
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "base_amount": 10_000_000_000_000,
                "quote_amount": 20_000_000_000_000,
                "min_lp_tokens": 1,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success

        # Set Oracle to exactly +5% (2.1)
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="update_exchange_rate",
            params={"base_currency": 1, "quote_currency": 0, "price": 2_100_000},
        )
        assert res["success"]

        # Trade should pass
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "amount_in": 1_000_000_000_000,
                "min_amount_out": 1,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success

        # Now set Oracle to 2.11 (>5%)
        res = submit_sudo_extrinsic(
            pallet="Oracle",
            call="update_exchange_rate",
            params={"base_currency": 1, "quote_currency": 0, "price": 2_110_000},
        )
        assert res["success"]

        # Trade should fail with OracleGuardRejected
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success
        assert any(ev.event_module == "BelizeX" and ev.event_name == "OracleGuardRejected" for ev in receipt.triggered_events)

    def test_trade_requires_kyc(self, blockchain_connection, alice_keypair):
        """Trading without KYC should fail with ExtrinsicFailed."""
        substrate = blockchain_connection
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "amount_in": 1_000_000_000_000,
                "min_amount_out": 1,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success
        assert any(ev.event_module == "System" and ev.event_name == "ExtrinsicFailed" for ev in receipt.triggered_events)

    def test_pause_blocks_trades(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair):
        """pause_global should block trades until resume_global."""
        substrate = blockchain_connection
        # Sudo pause
        res = submit_sudo_extrinsic(pallet="BelizeX", call="pause_global", params={})
        assert res["success"]

        # Ensure KYC
        submit_sudo_extrinsic(
            pallet="Oracle",
            call="verify_identity",
            params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )

        # Attempt trade -> should fail (paused)
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="execute_trade",
            call_params={
                "base_asset": 3,
                "quote_asset": 1,
                "amount_in": 1_000_000_000_000,
                "min_amount_out": 1,
                "is_tourism_trade": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert not receipt.is_success
        assert any(ev.event_module == "System" and ev.event_name == "ExtrinsicFailed" for ev in receipt.triggered_events)

        # Resume and try again -> should still fail without liquidity but not due to pause
        res = submit_sudo_extrinsic(pallet="BelizeX", call="resume_global", params={})
        assert res["success"]
