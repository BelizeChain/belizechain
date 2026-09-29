#!/usr/bin/env python3
"""Verify a running BelizeChain node's BABE epoch configuration.

Confirms the `testnet-fast-epoch` feature is actually baked into the running
binary (duration 300) rather than the shipped default (14_400), and reports
whether the chain's authority set is populated — the condition whose absence
stalled the previous chain.

Usage: verify_epoch_config.py [rpc_url] [expected_duration]
"""
import json
import sys
import urllib.request

URL = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:19944"
EXPECTED = int(sys.argv[2]) if len(sys.argv) > 2 else None

SLOT_MS = 6000


def call(method, params=None):
    params = params if params is not None else []
    body = json.dumps({"id": 1, "jsonrpc": "2.0", "method": method, "params": params}).encode()
    req = urllib.request.Request(URL, body, {"Content-Type": "application/json"})
    d = json.load(urllib.request.urlopen(req, timeout=25))
    if "error" in d:
        raise RuntimeError(f"{method}: {d['error']}")
    return d["result"]


def u64le(b, off):
    return int.from_bytes(b[off:off + 8], "little")


def compact(b, off):
    first = b[off]
    mode = first & 0b11
    if mode == 0:
        return first >> 2, off + 1
    if mode == 1:
        return int.from_bytes(b[off:off + 2], "little") >> 2, off + 2
    if mode == 2:
        return int.from_bytes(b[off:off + 4], "little") >> 2, off + 4
    n = (first >> 2) + 4
    return int.from_bytes(b[off + 1:off + 1 + n], "little"), off + 1 + n


header = call("chain_getHeader")
block = int(header["number"], 16)
print(f"head block        : #{block}")

raw = call("state_call", ["BabeApi_current_epoch", "0x"])
b = bytes.fromhex(raw[2:])
epoch_index = u64le(b, 0)
start_slot = u64le(b, 8)
duration = u64le(b, 16)

# authorities: Vec<(AuthorityId, weight)> follows
count, off = compact(b, 24)
authorities = []
for _ in range(count):
    aid = b[off:off + 32]
    off += 32
    weight = u64le(b, off)
    off += 8
    authorities.append((aid.hex(), weight))

print(f"epoch index       : {epoch_index}")
print(f"start slot        : {start_slot}")
print(f"duration          : {duration} slots  (~{duration * SLOT_MS / 3600000:.2f} h)")
print(f"epoch end slot    : {start_slot + duration}")
print(f"authorities       : {count}")
for aid, weight in authorities:
    print(f"    {aid}  weight={weight}")

print()
failures = []

if count == 0:
    failures.append("authority set is EMPTY — this is the exact condition that stalled #28791")
else:
    print(f"OK  authority set populated ({count} entr{'y' if count == 1 else 'ies'})")

if EXPECTED is not None:
    if duration == EXPECTED:
        print(f"OK  duration == expected {EXPECTED} (matches the expected build configuration)")
    else:
        failures.append(f"duration is {duration}, expected {EXPECTED} — the build feature did not take effect")

if failures:
    print()
    for f in failures:
        print(f"FAIL  {f}")
    sys.exit(1)

print()
print("PASS")
