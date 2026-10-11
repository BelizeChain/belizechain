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
    r"|\bdo(?:es)?\s+not\s+exist\b"
    r"|\bnever\s+existed\b"
    r"|\bnot\s+self-service\b"
    r"|\bnot\s+published\b"
    r"|\bno\s+longer\b"
    r"|\bhas\s+no\b"
    r"|\bhave\s+no\b"
    r"|\bnot\s+on-chain\b"
    r"|\bremoved\b"
    r"|\bdoesn'?t\s+exist\b"
    r"|\bcannot\s+be\b"
    r"|\bis\s+not\b"
    r"|\bare\s+not\b"
    r")",
    re.IGNORECASE,
)

# Tokens that are never pallet members.
FALSE_POSITIVE_TOKENS = {
    "ts", "tsx", "js", "jsx", "git", "rs", "py", "md", "json", "toml", "yaml",
    "yml", "sh", "html", "css", "lock", "env", "org", "com", "net", "io", "bz",
    "dev", "app", "unwrap", "entries", "querymap", "issome", "isnone",
    "tostring", "tobigint", "map", "filter", "reduce", "forEach".lower(),
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
    source = (root / "pallets" / directory / "src/lib.rs").read_text()
    calls = set()
    call_at = source.find("#[pallet::call]")
    if call_at >= 0:
        end = brace_match(source, source.find("{", call_at))
        calls = set(re.findall(r"pub fn (\w+)\s*\(", source[call_at:end]))
    storage = set(re.findall(r"pub type (\w+)<T: Config>", source))
    return calls | storage


def normalise(text):
    return re.sub(r"[^a-z0-9]", "", text.lower())


def main():
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    aliases = pallet_map(root)
    api = {name: pallet_api(root, directory) for name, directory in aliases.items()}
    normalised = {name: {normalise(x) for x in members} for name, members in api.items()}
    upper = {name.upper(): name for name in api}

    flagged = collections.defaultdict(list)
    negated = collections.Counter()

    for path in sorted((root / "docs").rglob("*.md")):
        lines = path.read_text(errors="replace").splitlines()
        for index, line in enumerate(lines, 1):
            for match in re.finditer(r"\b([A-Za-z][A-Za-z0-9_]*)\.([a-z][A-Za-z0-9_]{2,})\b", line):
                raw, member = match.group(1), match.group(2)
                pallet = raw if raw in api else upper.get(raw.upper())
                if pallet is None or (pallet, member) in EXEMPT:
                    continue
                if normalise(member) in normalised[pallet] or member.lower() in FALSE_POSITIVE_TOKENS:
                    continue
                if re.search(r"(://|\.belizechain\.|www\.)", line[max(0, match.start() - 25):match.end() + 8]):
                    continue

                # The fix: a reference the surrounding prose negates is correct.
                window = " ".join(lines[max(0, index - 3):index]) + " " + line
                if NEGATION.search(window):
                    negated[pallet] += 1
                    continue

                flagged[(pallet, member)].append(f"{path.relative_to(root)}:{index}")

    total = sum(len(v) for v in flagged.values())
    print(f"negation-aware scan: {len(flagged)} distinct / {total} sites flagged")
    print(f"  ({sum(negated.values())} references correctly documented as non-existent)")

    for (pallet, member), occurrences in sorted(flagged.items()):
        print(f"  {pallet}.{member:32s} -> {', '.join(occurrences)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
