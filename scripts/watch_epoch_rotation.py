#!/usr/bin/env python3
"""Block until a BABE epoch rotation happens, then report the evidence.

An epoch rotation is the exact event that killed the previous chain, so this
polls `BabeApi_current_epoch` and exits as soon as the epoch index advances.
Also fails fast on a stall (no new blocks) so a regression is noticed in
minutes rather than days.

Usage: watch_epoch_rotation.py <rpc_url> [--max-minutes N]
"""
import json
import sys
import time
import urllib.request
from datetime import datetime, timezone

URL = sys.argv[1]
MAX_MIN = 60
if "--max-minutes" in sys.argv:
    MAX_MIN = int(sys.argv[sys.argv.index("--max-minutes") + 1])


def call(method, params=None):
    params = params if params is not None else []
    body = json.dumps({"id": 1, "jsonrpc": "2.0", "method": method, "params": params}).encode()
    req = urllib.request.Request(URL, body, {"Content-Type": "application/json"})
    d = json.load(urllib.request.urlopen(req, timeout=20))
    if "error" in d:
        raise RuntimeError(str(d["error"]))
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


def epoch_state():
    raw = call("state_call", ["BabeApi_current_epoch", "0x"])
    b = bytes.fromhex(raw[2:])
    idx, start, dur = u64le(b, 0), u64le(b, 8), u64le(b, 16)
    count, off = compact(b, 24)
    authorities = []
    for _ in range(count):
        authorities.append((b[off:off + 32].hex(), u64le(b, off + 32)))
        off += 40
    header = call("chain_getHeader")
    return {
        "block": int(header["number"], 16),
        "epoch": idx,
        "start": start,
        "duration": dur,
        "end": start + dur,
        "authorities": authorities,
    }


def log(msg):
    print(f"[{datetime.now(timezone.utc).strftime('%H:%M:%S')}Z] {msg}", flush=True)


st = epoch_state()
log(f"watching {URL}")
log(f"  block #{st['block']}  epoch {st['epoch']}  slots [{st['start']}, {st['end']})  "
    f"authorities {len(st['authorities'])}")
log(f"  rotation expected at slot {st['end']} (~{(st['end'] - st['start']) * 6 / 60:.0f} min epochs)")

start_epoch = st["epoch"]
deadline = time.time() + MAX_MIN * 60
last_block = st["block"]
last_block_at = time.time()
stall_warned = False

while time.time() < deadline:
    time.sleep(15)
    try:
        st = epoch_state()
    except Exception as exc:  # noqa: BLE001
        log(f"  poll error: {exc}")
        continue

    if st["block"] != last_block:
        last_block = st["block"]
        last_block_at = time.time()
        stall_warned = False

    if st["epoch"] != start_epoch:
        log("")
        log("*** EPOCH ROTATION OBSERVED ***")
        log(f"  epoch {start_epoch} -> {st['epoch']}")
        log(f"  block #{st['block']}")
        log(f"  new epoch slots [{st['start']}, {st['end']})  duration {st['duration']}")
        log(f"  authorities: {len(st['authorities'])}")
        for aid, weight in st["authorities"]:
            log(f"      {aid}  weight={weight}")
        log("")
        if not st["authorities"]:
            log("FAIL: post-rotation authority set is EMPTY")
            sys.exit(1)
        log("PASS: the chain rotated its epoch and kept a populated authority set")
        sys.exit(0)

    idle = time.time() - last_block_at
    if idle > 90 and not stall_warned:
        stall_warned = True
        log(f"!!! STALL WARNING: no new block for {idle:.0f}s at #{st['block']} (epoch {st['epoch']})")

log(f"TIMEOUT after {MAX_MIN} min without an epoch rotation (block #{st['block']}, epoch {st['epoch']})")
sys.exit(2)
