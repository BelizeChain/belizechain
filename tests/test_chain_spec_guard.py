"""Guards the committed testnet chain spec against accidental regeneration.

`testnet-spec.json` is not a generated artefact that happens to be checked in — it
is the spec the live Ceiba node boots from with `--chain /data/chain/testnet-spec.json`,
and its genesis section is what produced Chain D's genesis hash:

    0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef

A node refuses to open a database whose genesis hash does not match the spec it was
given, so replacing this file makes the running chain unopenable rather than
"up to date". `belizechain-node build-spec --chain testnet > testnet-spec.json` is
therefore a destructive command, not a refresh.

The file legitimately looks out of date: its genesis predates spec 122 and has no
`interoperability` key, because Ceiba started at spec 107 and received the bridge
registry from the V1 -> V2 storage migration. That is correct and must not be
"fixed" by regenerating.

These tests need no chain, so they run even when no node is available.
"""

import hashlib
import json
from pathlib import Path

import pytest

SPEC_PATH = Path(__file__).resolve().parent.parent / "testnet-spec.json"

#: sha256 of the canonicalised `genesis` section of `testnet-spec.json`.
#:
#: Only `genesis` is hashed, so editing `name`, `bootNodes` or telemetry endpoints
#: does not trip this. Any change here changes the genesis hash of a chain launched
#: from this file, which is why it needs a deliberate decision rather than a commit.
PINNED_GENESIS_DIGEST = "fab66b0cb5b94bf26b2c8a251709a5bb226e5888de91e1a71528b8ed72664084"

#: Genesis hash the live Chain D reports for block 0.
LIVE_CHAIN_D_GENESIS = (
    "0xb2664568b41503c0661d576c08198ef3152b04ea76fee2c8a59216e88830b5ef"
)


def _genesis_digest(spec: dict) -> str:
    """Stable digest of a spec's genesis section.

    `sort_keys` so key order in the file is irrelevant, and the separators are
    pinned so this does not change with a Python upgrade.
    """
    canonical = json.dumps(spec["genesis"], sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode()).hexdigest()


@pytest.fixture(scope="module")
def spec() -> dict:
    assert SPEC_PATH.exists(), f"chain spec is missing: {SPEC_PATH}"
    return json.loads(SPEC_PATH.read_text())


def test_genesis_matches_the_live_chain(spec):
    """The committed spec is Chain D's, not a regenerated one.

    If this fails, the file was rebuilt from current code. That is only acceptable
    as part of a deliberate re-genesis: keep the old file for rollback, expect the
    live node to reject its existing database, and update `LIVE_CHAIN_D_GENESIS`
    and `PINNED_GENESIS_DIGEST` in the same commit that announces the new chain.
    """
    assert _genesis_digest(spec) == PINNED_GENESIS_DIGEST, (
        "testnet-spec.json's genesis section changed. This file is what Ceiba boots "
        f"from and it must keep producing {LIVE_CHAIN_D_GENESIS}. Regenerating it "
        "makes the live node unable to open its own database. See this module's "
        "docstring before touching the pin."
    )


def test_genesis_has_no_interoperability_section(spec):
    """Ceiba's bridge registry came from the V1 -> V2 migration, not from genesis.

    Pinned because the absence looks like a defect and has already been
    "corrected" once in a report. Adding the section here would change the genesis
    hash, so a future chain that should seed the bridge at genesis needs a new spec
    file, not an edit to this one.
    """
    patch = spec["genesis"]["runtimeGenesis"]["patch"]
    assert "interoperability" not in patch, (
        "the interoperability genesis section was added to testnet-spec.json. That "
        "changes the genesis hash of the live chain — seed it in a spec for a *new* "
        "chain instead."
    )


def test_spec_identity_is_unchanged(spec):
    """Cheap sanity check on the fields the node and tooling read by name.

    `name` is `Jade` — Chain D's codename, per `docs/operations/CHAIN_LINEAGE.md`.
    It is not the `BelizeChain Testnet` that current code would generate, which is
    one more reason a regenerated spec is not a drop-in replacement.

    `chainType` is `Live`, not `Development`/`Local`: this is a real chain spec with
    real session keys, so a stray `build-spec --chain dev` overwriting the file is
    a different chain entirely.
    """
    assert spec["id"] == "belizechain_testnet"
    assert spec["name"] == "Jade"
    assert spec["chainType"] == "Live"
