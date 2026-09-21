#!/usr/bin/env python3
"""Replace the runtime code embedded in a non-raw chain spec.

A non-raw Substrate chain spec carries the genesis runtime under
``genesis.runtimeGenesis.code``. Whatever WASM sits there is the runtime a
fresh chain is *born* with, so a spec pinning an old build forces every fresh
boot through a runtime upgrade before it reaches the current spec version.

Usage:
    set-spec-code.py <spec.json> <runtime.wasm> [-o <out.json>]

Without ``-o`` the spec is rewritten in place. Prints the old and new code
hashes so the change can be verified against the on-chain ``:code``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path


def code_hash(wasm: bytes) -> str:
    return hashlib.blake2b(wasm, digest_size=32).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", type=Path, help="non-raw chain spec JSON")
    parser.add_argument("wasm", type=Path, help="runtime WASM to embed")
    parser.add_argument("-o", "--out", type=Path, default=None, help="output path (default: in place)")
    args = parser.parse_args()

    spec = json.loads(args.spec.read_text())
    runtime_genesis = spec.get("genesis", {}).get("runtimeGenesis")
    if not isinstance(runtime_genesis, dict):
        print(f"error: {args.spec} has no genesis.runtimeGenesis (is it already raw?)", file=sys.stderr)
        return 1

    old_hex = runtime_genesis.get("code", "") or ""
    old = bytes.fromhex(old_hex[2:]) if old_hex.startswith("0x") else b""
    new = args.wasm.read_bytes()

    if not old:
        print("error: spec has no embedded runtime code to replace", file=sys.stderr)
        return 1

    runtime_genesis["code"] = "0x" + new.hex()

    out = args.out or args.spec
    out.write_text(json.dumps(spec, indent=2) + "\n")

    print(f"spec        : {args.spec}")
    print(f"runtime wasm: {args.wasm}")
    print(f"written     : {out}")
    print(f"old code    : {len(old):>9} bytes  blake2b-256 {code_hash(old)}")
    print(f"new code    : {len(new):>9} bytes  blake2b-256 {code_hash(new)}")
    if old == new:
        print("note: runtime code was already up to date")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
