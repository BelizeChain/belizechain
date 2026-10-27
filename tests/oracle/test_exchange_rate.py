"""Oracle USD/BZD rate updates.

`update_exchange_rate(base_currency, quote_currency, price)` takes
`Oracle::Currency` indices (0 = BZD, 1 = USD, 2 = EUR, …) and a `price` scaled by
1e6.

This file previously only asserted that the extrinsic was included, despite its
name promising an event check — an assertion that would pass even if the rate
were never stored.
"""

import pytest

CURRENCY_BZD = 0
CURRENCY_USD = 1

PRICE_SCALE = 1_000_000


def _set_rate(submit_sudo_extrinsic, price: int):
    return submit_sudo_extrinsic(
        pallet="Oracle",
        call="update_exchange_rate",
        params={
            "base_currency": CURRENCY_USD,
            "quote_currency": CURRENCY_BZD,
            "price": price,
        },
    )


def _read_rate(blockchain_connection):
    """The stored price, from `(price, updated_at)`.

    `ManualExchangeRates` is a `StorageMap<_, Blake2_128Concat, CurrencyPair,
    (u128, BlockNumber), OptionQuery>` — the tuple's second element is the block
    the rate was written at, not part of the price.

    `Currency` is a unit enum, so scalecodec needs the variant name, not the
    numeric discriminant the extrinsic takes.
    """
    entry = blockchain_connection.query(
        "Oracle", "ManualExchangeRates", [{"base": "USD", "quote": "BZD"}]
    ).value
    if entry is None:
        return None
    rate, _written_at = entry
    return rate


@pytest.mark.requires_blockchain
def test_update_usd_bzd_rate_is_stored(submit_sudo_extrinsic, blockchain_connection):
    """Setting USD/BZD actually persists the rate."""
    price = 1_950_000  # 1.95 BZD per USD

    res = _set_rate(submit_sudo_extrinsic, price)
    assert res["success"], f"Oracle rate update failed: {res.get('error')}"

    stored = _read_rate(blockchain_connection)
    assert stored is not None, (
        "update_exchange_rate succeeded but stored no ManualExchangeRates entry"
    )
    assert int(stored) == price, (
        f"stored rate {stored} does not match the submitted {price}"
    )


@pytest.mark.requires_blockchain
def test_rate_is_the_value_the_belizex_guard_reads(
    submit_sudo_extrinsic, blockchain_connection
):
    """This entry is the single input to BelizeX's deviation guard.

    `get_crypto_exchange_rate` in the runtime ignores the base asset and always
    returns USD/BZD for a bBZD-quoted pair, so a stale or missing entry here is
    exactly what makes the guard reject — or fail closed.
    """
    expected = 2_000_000

    res = _set_rate(submit_sudo_extrinsic, expected)
    assert res["success"], f"Oracle rate update failed: {res.get('error')}"

    stored = _read_rate(blockchain_connection)
    assert int(stored) == expected, (
        f"the guard would read {stored}, not the {expected} that was set"
    )
