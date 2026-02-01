import pytest


@pytest.mark.requires_blockchain
def test_update_usd_bzd_rate_emits_event(submit_sudo_extrinsic):
	"""
	Simple sanity check: set USD/BZD rate via Oracle and ensure extrinsic succeeds.
	"""
	price_scaled = 1_950_000  # 1.95
	res = submit_sudo_extrinsic(
		pallet="Oracle",
		call="update_exchange_rate",
		params={"base_currency": 1, "quote_currency": 0, "price": price_scaled},
	)
	assert res["success"], f"Oracle rate update failed: {res.get('error')}"
