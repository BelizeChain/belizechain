#!/usr/bin/env python3
"""Prove a node binary can instantiate a runtime blob, before it halts the chain.

A runtime reaches new host functions through `sp-io`. The client provides them
by *name*: `sp_wasm_interface` registers each one as a literal `ext_*` string,
and the WASM linker resolves the runtime's imports against that table. If the
runtime imports a name the client does not register, instantiation fails
outright — the chain does not fall back, it stops producing blocks.

That is a different failure class from a missing JSON-RPC method. A missing RPC
returns `-32601` and looks like a bug; a missing host function kills the chain.
It is also invisible to every static check we had, because it depends on the
*pairing* of a blob with a binary, and both are individually valid.

How it surfaced: Ceiba ran a binary built 2026-10-19 against a runtime deployed
2026-10-27. Nothing broke, because 121->124 happened to need no client support.
That was luck. This script makes it checkable instead of lucky.

What it reads
-------------
The blob's WASM import section (section id 2) for `env`-module `ext_*` names,
against the literal strings in the client binary. `strip` removes symbols, not
`.rodata` literals, so a stripped release binary still carries the names.

Soundness: a name absent from the binary cannot possibly be registered, so a
missing name is a real defect. A name present as a stray string would be a
false pass — vanishingly unlikely for `ext_*`, and the empirical rehearsal
(`rehearse-runtime-upgrade.py`) remains the end-to-end proof.

Usage
-----
    check-runtime-host-functions.py --wasm <blob.wasm> --binary ./belizechain-node
    check-runtime-host-functions.py --wasm <blob.wasm> --image belizechain/ceiba-node:<tag>
    check-runtime-host-functions.py --wasm <blob.wasm> --image <tag> --list-imports
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

WASM_MAGIC = b"\x00asm"
# `sp_maybe_compressed_blob` wraps a zstd payload as: 3-byte magic, one version
# byte, one flags byte, then a zstd frame. This is the exact artifact that lands
# in `System::code` (Ceiba's spec-124 `:code` is byte-identical to wasm-builder's
# `.compact.compressed.wasm`), so it is the form most likely to be passed in —
# and the one the parser cannot read.
COMPRESSED_BLOB_MAGIC = bytes([0x52, 0xBC, 0x53])

SECTION_IMPORT = 2
EXTERNAL_KIND_FUNC = 0x00
EXTERNAL_KIND_TABLE = 0x01
EXTERNAL_KIND_MEMORY = 0x02
EXTERNAL_KIND_GLOBAL = 0x03
EXTERNAL_KIND_TAG = 0x04

DEFAULT_BINARY_IN_IMAGE = "/usr/local/bin/belizechain-node"


def read_varuint(buf: bytes, pos: int) -> tuple[int, int]:
    """Decode an unsigned LEB128 value, returning (value, next_position)."""
    result = 0
    shift = 0
    while True:
        if pos >= len(buf):
            raise ValueError("truncated LEB128 value")
        byte = buf[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, pos
        shift += 7
        if shift > 63:
            raise ValueError("LEB128 value too long")


def read_name(buf: bytes, pos: int) -> tuple[str, int]:
    """Decode a WASM length-prefixed UTF-8 name."""
    length, pos = read_varuint(buf, pos)
    end = pos + length
    if end > len(buf):
        raise ValueError("truncated name")
    return buf[pos:end].decode("utf-8", errors="replace"), end


def skip_limits(buf: bytes, pos: int) -> int:
    flags, pos = read_varuint(buf, pos)
    _, pos = read_varuint(buf, pos)  # min
    if flags & 0x01:
        _, pos = read_varuint(buf, pos)  # max
    return pos


def skip_import_desc(buf: bytes, pos: int, kind: int) -> int:
    """Advance past one import descriptor of the given external kind."""
    if kind == EXTERNAL_KIND_FUNC:
        _, pos = read_varuint(buf, pos)
        return pos
    if kind == EXTERNAL_KIND_TABLE:
        return skip_limits(buf, pos + 1)  # reftype byte, then limits
    if kind == EXTERNAL_KIND_MEMORY:
        return skip_limits(buf, pos)
    if kind == EXTERNAL_KIND_GLOBAL:
        return pos + 2  # valtype + mutability
    if kind == EXTERNAL_KIND_TAG:
        _, pos = read_varuint(buf, pos + 1)  # attribute byte, then typeidx
        return pos
    raise ValueError(f"unknown external kind {kind:#x}")


def parse_host_fn_imports(blob: bytes) -> list[str]:
    """Return the `env`-module `ext_*` names the module imports, sorted."""
    if blob[:3] == COMPRESSED_BLOB_MAGIC:
        raise ValueError(
            f"this is a compressed blob envelope (version {blob[3]:#04x}), not a "
            "WASM module.\n"
            "  This is the artifact that goes on chain, so it is easy to reach for,\n"
            "  but the import section is zstd-compressed inside it. Pass the\n"
            "  uncompressed module built alongside it instead:\n"
            "    target/release/wbuild/belizechain-runtime/belizechain_runtime.wasm"
        )
    if blob[:4] != WASM_MAGIC:
        raise ValueError("not a WASM module (missing \\0asm magic)")

    pos = 8  # magic + version
    names: list[str] = []
    while pos < len(blob):
        section_id = blob[pos]
        pos += 1
        size, pos = read_varuint(blob, pos)
        end = pos + size
        if end > len(blob):
            raise ValueError("truncated section")
        if section_id != SECTION_IMPORT:
            pos = end
            continue

        count, pos = read_varuint(blob, pos)
        for _ in range(count):
            module, pos = read_name(blob, pos)
            field, pos = read_name(blob, pos)
            kind = blob[pos]
            pos = skip_import_desc(blob, pos + 1, kind)
            if module == "env" and field.startswith("ext_"):
                names.append(field)
        pos = end

    if not names:
        raise ValueError(
            "no `env` `ext_*` imports found — parsing may have gone wrong, or the "
            "module is not a Substrate runtime"
        )
    return sorted(set(names))


def binary_from_local_path(path: Path) -> bytes:
    return path.read_bytes()


def binary_from_docker(image: str, binary_in_image: str) -> bytes:
    """Extract the client binary from an image without leaving a container behind."""
    cid = subprocess.run(
        ["docker", "create", image],
        capture_output=True,
        text=True,
        check=False,
    )
    if cid.returncode != 0:
        raise RuntimeError(f"docker create failed for {image}: {cid.stderr.strip()}")
    container = cid.stdout.strip()
    try:
        result = subprocess.run(
            ["docker", "cp", f"{container}:{binary_in_image}", "-"],
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            raise RuntimeError(
                f"docker cp {binary_in_image} failed: "
                f"{result.stderr.decode(errors='replace').strip()}"
            )
        return result.stdout
    finally:
        subprocess.run(["docker", "rm", "-f", container], capture_output=True, check=False)


def missing_host_functions(binary: bytes, required: list[str]) -> list[str]:
    return [name for name in required if name.encode() not in binary]


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--wasm", type=Path, required=True, help="uncompressed runtime .wasm")
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--binary", type=Path, help="node binary on this machine")
    source.add_argument("--image", help="docker image to extract the node binary from")
    parser.add_argument(
        "--binary-in-image",
        default=DEFAULT_BINARY_IN_IMAGE,
        help=f"path to the node binary inside the image (default {DEFAULT_BINARY_IN_IMAGE})",
    )
    parser.add_argument(
        "--list-imports",
        action="store_true",
        help="print every imported host function, then the verdict",
    )
    args = parser.parse_args()

    try:
        required = parse_host_fn_imports(args.wasm.read_bytes())
    except (OSError, ValueError) as exc:
        print(f"❌ {args.wasm}: {exc}", file=sys.stderr)
        return 1

    print(f"blob    : {args.wasm} imports {len(required)} host function(s)")

    try:
        if args.binary is not None:
            binary = binary_from_local_path(args.binary)
            print(f"client  : {args.binary} ({len(binary):,} bytes)")
        else:
            binary = binary_from_docker(args.image, args.binary_in_image)
            print(f"client  : {args.image}:{args.binary_in_image} ({len(binary):,} bytes)")
    except (OSError, RuntimeError) as exc:
        print(f"❌ {exc}", file=sys.stderr)
        return 1

    if args.list_imports:
        for name in required:
            print(f"          {name}")

    missing = missing_host_functions(binary, required)
    if missing:
        print("", file=sys.stderr)
        print(
            f"❌ {len(missing)} host function(s) the runtime imports are NOT in the client:",
            file=sys.stderr,
        )
        for name in missing:
            print(f"     {name}", file=sys.stderr)
        print(
            "\n   These resolve at WASM instantiation. A client missing them does not\n"
            "   degrade — it fails to instantiate the runtime and stops producing blocks.\n"
            "   Rebuild the node image from the same tree as the blob (same pinned\n"
            "   polkadot-sdk rev), then re-run this check.",
            file=sys.stderr,
        )
        return 1

    print(f"✅ client provides all {len(required)} imported host function(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
