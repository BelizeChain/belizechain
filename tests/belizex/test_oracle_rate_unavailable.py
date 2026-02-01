import pytest

@pytest.mark.requires_blockchain
class TestOracleRateUnavailable:

    @staticmethod
    def _event_module_and_name(ev):
        """Extract (module, name) from a substrate-interface event record across versions.
        Falls back through known shapes to avoid attribute errors from scale-info wrappers.
        """
        # Newer substrate-interface may expose event details via ev.event.value
        try:
            if hasattr(ev, "event") and hasattr(ev.event, "value") and isinstance(ev.event.value, dict):
                val = ev.event.value
                # Common keys: 'module', 'event', or nested under 'name'
                module = val.get("module") or val.get("pallet") or val.get("section")
                name = val.get("event") or val.get("name")
                if module and name:
                    return str(module), str(name)
        except Exception:
            pass

        # Older substrate-interface exposed shortcuts
        try:
            module = getattr(ev, "event_module", None)
            name = getattr(ev, "event_name", None)
            if module and name:
                return str(module), str(name)
        except Exception:
            pass

        # Fallback to string parsing
        s = str(getattr(ev, "event", ev))
        # Try simple heuristics like "BelizeX.OracleRateUnavailable"
        if "." in s:
            parts = s.split(".")
            return parts[0], parts[-1]
        return s, s
    def test_trade_blocks_when_oracle_rate_unavailable(self, submit_sudo_extrinsic, blockchain_connection, alice_keypair):
        """
        When USD/BZD rate is not set (or stale so Oracle returns None),
        WUSDC→BBZD trades must be rejected and emit BelizeX::OracleRateUnavailable.
        """
        substrate = blockchain_connection

        # 1) Ensure WUSDC/BBZD pair exists (idempotent; ignore if already exists)
        submit_sudo_extrinsic(
            pallet="BelizeX",
            call="create_trading_pair",
            params={"base_asset": 3, "quote_asset": 1, "fee_rate": 30},
        )

        # 2) Ensure Alice has KYC Level 1 (signed call; Oracle requires signed origin)
        call = substrate.compose_call(
            call_module="Oracle",
            call_function="verify_identity",
            call_params={
                "account": str(alice_keypair.ss58_address),
                "kyc_level": 1,
                "id_hash": [0] * 32,
                "provider": "testnet",
                "biometric_verified": False,
                "address_verified": False,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Set KYC failed: {getattr(receipt, 'error_message', None)}"

        # 3) Add some liquidity so swap path exists
        call = substrate.compose_call(
            call_module="BelizeX",
            call_function="add_liquidity",
            call_params={
                "base_asset": 3,  # WUSDC
                "quote_asset": 1,  # BBZD
                "base_amount": 10_000_000_000_000,
                "quote_amount": 20_000_000_000_000,
                "min_lp_tokens": 1,
            },
        )
        extrinsic = substrate.create_signed_extrinsic(call=call, keypair=alice_keypair)
        receipt = substrate.submit_extrinsic(extrinsic, wait_for_finalization=True)
        assert receipt.is_success, f"Add liquidity failed: {getattr(receipt, 'error_message', None)}"

        # 4) Attempt guarded trade WITHOUT setting Oracle rate -> should reject
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
        assert not receipt.is_success, "Trade should have been rejected due to missing Oracle rate"

        # Expect diagnostics: OracleRateUnavailable (BelizeX) and ExtrinsicFailed (System)
        # Use error message string for robust matching across substrate-interface versions
        err_msg = getattr(receipt, "error_message", None)
        if isinstance(err_msg, dict):
            assert err_msg.get("name") == "OracleRateUnavailable" and err_msg.get("type") == "Module", (
                f"Expected BelizeX::OracleRateUnavailable Module error, got: {err_msg}"
            )
        else:
            err_str = str(err_msg or "")
            assert "OracleRateUnavailable" in err_str, f"Expected BelizeX::OracleRateUnavailable, got error: {err_str}"
        # System::ExtrinsicFailed is expected as the outer failure; event decoding varies across versions,
        # and we already asserted the module error above, so we skip strict System event matching here.
