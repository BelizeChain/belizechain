#!/usr/bin/env python3
"""
Doc reference scanner, negation-aware.

The first version of this scan reported 18 "fabricated references" in 9 files.
Every one turned out to be an HONEST NEGATION -- a line explicitly documenting
that the call does not exist, e.g.:

    // There is no `economy.transfer`.
    // There is no `compliance.submitKycApplication`.
    // no `staking.nominators` map exists.

Those lines are correct and must not be "fixed". The defect was in the detector,
so this version drops any match whose surrounding text negates it.

Usage:  python3 scan_doc_refs.py [repo_root]
Exit 0 always; it is a report, not a gate.
"""
import collections
import pathlib
import re
import sys

NEGATION = re.compile(
    r"(there\s+(?:is|are)\s+no\b"
    r"|there'?s\s+no\b"
    r"|\bno\s+such\b"
    r"|\bdoes\s+not\s+exist\b"
    r"|\bdo\s+not\s+exist\b"
    r"|\bdoesn'?t\s+exist\b"
    r"|\bdon'?t\s+exist\b"
    r"|\bnever\s+existed\b"
    r"|\bnever\s+created\b"
    r"|\bnot\s+self-service\b"
    r"|\bnot\s+published\b"
    r"|\bnot\s+on-chain\b"
    r"|\bno\s+longer\b"
    r"|\bhas\s+been\s+removed\b"
    r"|\bhave\s+been\s+removed\b"
    r")",
    re.IGNORECASE,
)

"""
Deliberately narrow. An earlier version also treated `is not`, `are not`, `has no`,
`cannot be` and a bare `removed` as negation, and that filtered genuine
fabrications: ordinary prose like "the domain is not registered" or "handles are
removed" was enough to excuse a bad API name on the same line, and the scan
reported 0 while real sites were still present.

Recall over precision: the cost of over-filtering is a silent false negative,
which is the failure this scan exists to prevent. Every pattern kept above states
that something does not exist, which is a much stronger signal than a bare
negation.
"""

# How many preceding lines count as context for a negation or annotation. Four is
# needed in practice: prose routinely puts the caveat on its own line and the
# example a couple of lines below it, e.g.
#   // Total marketplace revenue is NOT tracked on-chain: there is no
#   // `totalMarketplaceRevenue` storage item. Sum the fee fields from
#   // `DomainSold` events instead.
#   // const totalRevenue = await sdk.api.query.bns.totalMarketplaceRevenue();
NEGATION_WINDOW = 4

# Tokens that are never pallet members.
FALSE_POSITIVE_TOKENS = {
    "ts", "tsx", "js", "jsx", "git", "rs", "py", "md", "json", "toml", "yaml",
    "yml", "sh", "html", "css", "lock", "env", "org", "com", "net", "io", "bz",
    "dev", "app", "unwrap", "entries", "querymap", "issome", "isnone",
    "tostring", "tobigint", "map", "filter", "reduce", "foreach",
}

#
# A reference the prose marks as unimplemented is an honest annotation, not a
# fabrication. Matching one of these still counts, but separately.
#
ANNOTATED = re.compile(
    r"(NOT\s+IMPLEMENTED"
    r"|not\s+implemented"
    r"|planned"
    r"|coming\s+soon"
    r"|future\s+work"
    r")",
    re.IGNORECASE,
)

# Kept for reference: real namespaces that are NOT under pallets/. They never reach
# the flagging path (see the `pallet is None` guard below) but document why a
# reader should not expect them in the runtime alias map.
STANDARD_NAMESPACES = {
    "system", "balances", "timestamp", "session", "grandpa", "babe", "sudo",
    "utility", "multisig", "proxy", "scheduler", "contracts", "treasury",
    "xcm", "xcmpallet", "assets", "nfts",
    "chain", "state", "author", "payment", "engine", "offchain", "syncstate",
    "chainhead", "chainspec", "transaction", "transactionwatch", "archive",
    "pakit", "belizechain", "sdk",
}

# (pallet, member) pairs a human has confirmed are intentional references.
EXEMPT = {
    ("Consensus", "submit_quantum_work"),
    ("LandLedger", "query_map"),
    ("Identity", "initial"),
    ("Identity", "initialBiometricIssuers"),
}


def brace_match(text, start):
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return len(text)


def pallet_map(root):
    crate_to_dir = {}
    for manifest in sorted((root / "pallets").glob("*/Cargo.toml")):
        name = re.search(r'^name\s*=\s*"([^"]+)"', manifest.read_text(), re.M)
        if name:
            crate_to_dir[name.group(1).replace("-", "_")] = manifest.parent.name

    runtime = (root / "runtime/src/lib.rs").read_text()
    construct = runtime.find("construct_runtime!")
    body_start = runtime.find("{", runtime.find("pub struct Runtime", construct))
    block = runtime[body_start:brace_match(runtime, body_start)]

    aliases = {}
    for line in block.splitlines():
        line = re.sub(r"//.*", "", line)
        m = re.match(r"\s*(\w+)\s*:\s*([\w:]+)", line)
        if m and m.group(2).split("::")[0] in crate_to_dir:
            aliases[m.group(1)] = crate_to_dir[m.group(2).split("::")[0]]
    return aliases


def pallet_api(root, directory):
    """
    Everything a pallet legitimately exposes: extrinsics, storage and events.

    Events matter as much as the other two. Without them the scan reports real
    event names (`Staking.FLTaskAssigned`, `Bns.DomainRegistered`) as fabricated,
    and a report full of false positives is one nobody acts on.
    """
    source = (root / "pallets" / directory / "src/lib.rs").read_text()

    calls = set()
    call_at = source.find("#[pallet::call]")
    if call_at >= 0:
        end = brace_match(source, source.find("{", call_at))
        calls = set(re.findall(r"pub fn (\w+)\s*\(", source[call_at:end]))

    storage = set(re.findall(r"pub type (\w+)<T: Config>", source))

    events = set()
    event_at = source.find("#[pallet::event]")
    if event_at >= 0:
        start = source.find("{", event_at)
        end = brace_match(source, start)
        # Variants are declared at a fixed indent inside the enum body.
        events = set(re.findall(r"^\s{8}(\w+)\s*[{(]", source[start:end], re.M))

    return calls | storage | events


def normalise(text):
    return re.sub(r"[^a-z0-9]", "", text.lower())


def strip_emphasis(text):
    """
    Remove markdown emphasis and code ticks before matching negation patterns.

    Prose writes ``There is **no `staking.bond`**``, which the pattern
    ``there\\s+is\\s+no`` cannot match because of the ``**``. Without this the
    scan reports its own corrections as new fabrications.
    """
    return re.sub(r"[*_`]", "", text)


# A dotted chain such as `sdk.api.tx.bns.purgeCdnCache`.
DOTTED_CHAIN = re.compile(r"[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)+")

# Chain heads that introduce a real API call. Inside these chains every adjacent
# pair is worth testing. Outside them, only the first pair of a chain is, which
# keeps module paths such as `pakit.quantum.compression` from being read as a
# reference to the `Quantum` pallet.
#
# `tx`/`query`/`rpc` are the polkadot-js-api CLI prefixes (`tx.staking.bond`,
# `query.interoperability.transactions`). Leaving them out silently suppressed
# every CLI-style finding in the docs.
API_ROOTS = {
    "api", "sdk", "blockchain", "client",
    "tx", "query", "rpc",
    "polkadot_js_api", "polkadotjsapi",
}


def dotted_pairs(line):
    """
    Yield every adjacent `(left, right, column)` pair worth testing on a line.

    Matching pairs with a plain `(\\w+)\\.(\\w+)` regex silently drops real
    references: the regex consumes characters, so in
    `sdk.api.tx.bns.purgeCdnCache` it matches `sdk.api`, then `tx.bns`, and the
    cursor is already past `bns` — so `bns.purgeCdnCache` is never examined. That
    is a false negative in exactly the case this scan exists to catch (memory:
    "never trust a scan whose recall you haven't probed by hand").

    Splitting each chain and pairing every neighbour has no such gap. Pairs other
    than the first are only tested when the chain is rooted at a known API head,
    because `pakit.quantum.compression` is a Python import path and pairing
    `quantum` with `compression` there is pure noise.
    """
    for chain in DOTTED_CHAIN.finditer(line):
        segments = chain.group(0).split(".")
        head = segments[0].lower()
        for position, (left, right) in enumerate(zip(segments, segments[1:])):
            if position > 0 and head not in API_ROOTS:
                break
            yield left, right, chain.start()


def main():
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    aliases = pallet_map(root)
    api = {name: pallet_api(root, directory) for name, directory in aliases.items()}
    normalised = {name: {normalise(x) for x in members} for name, members in api.items()}
    upper = {name.upper(): name for name in api}

    flagged = collections.defaultdict(list)
    negated = collections.Counter()
    annotated = collections.Counter()

    for path in sorted((root / "docs").rglob("*.md")):
        lines = path.read_text(errors="replace").splitlines()
        for index, line in enumerate(lines, 1):
            for raw, member, column in dotted_pairs(line):
                # Docs write pallets in lower case (`api.tx.economy.transfer`) while
                # the runtime alias is `Economy`, so resolution must be
                # case-insensitive. Skipping this silently matches nothing at all.
                pallet = raw if raw in api else upper.get(raw.upper())

                # Only a pair whose left side IS a pallet can be a bad pallet
                # reference. Flagging unknown left sides was tried and produced
                # ~4300 hits of pure noise (`ApiPromise.create`, `Cargo.toml`),
                # which buries the real findings.
                if pallet is None or (pallet, member) in EXEMPT:
                    continue
                if normalise(member) in normalised[pallet] or member.lower() in FALSE_POSITIVE_TOKENS:
                    continue
                if re.search(r"(://|\.belizechain\.|www\.)", line[max(0, column - 25):column + 40]):
                    continue

                window = strip_emphasis(
                    " ".join(lines[max(0, index - NEGATION_WINDOW):index]) + " " + line
                )

                # An annotated or negated reference is a documented absence, which
                # is the opposite of a fabrication. Counted, never flagged.
                if ANNOTATED.search(window):
                    annotated[pallet] += 1
                    continue
                if NEGATION.search(window):
                    negated[pallet] += 1
                    continue

                flagged[(pallet, member)].append(f"{path.relative_to(root)}:{index}")

    total = sum(len(v) for v in flagged.values())
    print(f"negation-aware scan: {len(flagged)} distinct / {total} sites flagged")
    print(f"  ({sum(negated.values())} references negated in prose, {sum(annotated.values())} marked not-implemented)")

    for (pallet, member), occurrences in sorted(flagged.items()):
        print(f"  {pallet}.{member:32s} -> {', '.join(occurrences)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
