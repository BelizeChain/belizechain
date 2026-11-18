# Running a BelizeChain node on the macOS desktop

Status: **plan — not yet executed.** No macOS build of this node has been produced or
verified. Every step below is derived from the repo and from the live chain, but the
build itself is unproven on Darwin. Treat the first run as a spike, not a rollout.

Scope: turning the owner's always-on office Mac into a BelizeChain node, and
(eventually) a second authority.

---

## 1. What this box actually costs

Measured on the live `ceiba-node` over 11.1 h of cgroup uptime (ticks ÷ uptime, not a
one-second sample):

| Resource | Steady state |
|---|---|
| CPU | **0.57 % of one core** |
| RAM | ~880 MiB |
| Disk | 605 MB at block 615,406 |
| Disk growth | ~14 MB/day ≈ **430 MB/month** |

That is the cost of *this* chain today — a near-idle testnet with one authority and no
dApp traffic. Do not quote these numbers for a mainnet with real load.

The one-off cost is **initial sync**: the Mac must execute ~615k blocks from genesis
before it can be considered caught up. Expect hours at full core usage, then silence.

---

## 2. Why it is built from source

There is no macOS artefact and there cannot be one from the current pipeline:

| Fact | Consequence |
|---|---|
| `belizechain-node` in the repo root is an **x86-64 Linux ELF** | unusable on Apple Silicon |
| `belizechain/ceiba-node:*` images are **`amd64` / linux** | Docker on Mac means QEMU emulation |
| No arm64 image is published | emulation is the only container path |
| The repo pins `channel = "1.90.0"` | build must use that exact toolchain |

QEMU-emulated consensus is not viable — a 6-second block deadline leaves no room for
instruction translation overhead. Build native.

---

## 3. Two flags that are not optional

Both of these have already caused a production incident.

### `--features testnet-fast-epoch`

`runtime/src/lib.rs`:

```rust
pub const BabeEpochDuration: u64 = if cfg!(feature = "testnet-fast-epoch") {
    300        // ~30 min — the live chain
} else {
    14_400     // ~24 h — the shipping default
};
```

A node built **without** the feature expects a 14,400-slot epoch while the live chain
rotates every 300. It will refuse the epoch change and stop importing blocks. This is
exactly how **Chain C died on 2026-10-04** — see
[`EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md`](EMPTY_EPOCH_STALL_AND_REGENESIS_2026-09-29.md).

There is a unit test pinning this (`babe_epoch_duration_respects_build_feature`), which
is why the value cannot drift silently — but the test only guards the *source*, not the
binary you happen to have compiled.

### `--bootnodes`

`testnet-spec.json` ships:

```json
"bootNodes": []
```

Empty. A node started from that spec has **no peer to ask** and will sit at genesis
forever while reporting healthy. Ceiba's peer ID must be passed explicitly:

```
/ip4/100.119.97.38/tcp/30333/p2p/12D3KooWMWYcGsBhe1NGX3mf4fMkoipi9U59KxEbJp3CmDqe7cP1
```

`100.119.97.38` is Ceiba's Tailscale address. If the Mac is not on the tailnet, use the
LAN address instead (`10.0.0.19`) or a publicly reachable address.

---

## 4. Prerequisites

```bash
xcode-select --install
brew install cmake protobuf openssl pkg-config

curl https://sh.rustup.rs -sSf | sh -s -- -y \
  --default-toolchain 1.90.0 \
  --profile minimal
```

Then, from inside the checkout, add the wasm targets the runtime needs
(`rust-toolchain.toml` requests them, so `rustup` will fetch them on first use, but
installing explicitly makes a failure obvious rather than mysterious):

```bash
rustup target add wasm32-unknown-unknown wasm32v1-none
```

Confirm you are on the pinned toolchain before building — a different Rust can produce a
different runtime blob, and a runtime blob that differs from Ceiba's will fail the
`set_code` hash comparison later:

```bash
rustc --version    # must print 1.90.0
```

---

## 5. Build

Pin the commit. Do not track `HEAD` — the node you run must match the runtime the chain
is executing.

```bash
git clone https://github.com/BelizeChain/belizechain.git
cd belizechain
git checkout 743bac8            # or the sha live on the chain at the time you build
cargo build --release -p belizechain-node --features testnet-fast-epoch
```

For reference, the same build takes **3m29s / 468 crates** on Ceiba with `-j 16`. A Mac
will be slower; a laptop will be much slower and may thermal-throttle.

**Verify the feature actually took effect** rather than trusting the command line:

```bash
# should report 300, not 14400
./target/release/belizechain-node build-spec --chain testnet-spec.json --raw 2>/dev/null \
  | grep -o 'BabeEpochDuration[^,]*' || echo "check runtime/src/lib.rs instead"
```

If that probe is inconclusive, the authoritative check is §7: does the node follow epoch
rotations, or does it stall at the first boundary?

---

## 6. First run — full node, no keys

```bash
./target/release/belizechain-node \
  --chain testnet-spec.json \
  --base-path "$HOME/belizechain-data" \
  --port 30333 \
  --rpc-port 9944 \
  --prometheus-port 9615 \
  --bootnodes /ip4/100.119.97.38/tcp/30333/p2p/12D3KooWMWYcGsBhe1NGX3mf4fMkoipi9U59KxEbJp3CmDqe7cP1
```

Expected on first start: it connects, begins syncing from genesis, and **appears frozen
at a low block number for a long time**. That is normal for a from-genesis sync; watch
`best` climb rather than assuming failure.

Stop macOS from sleeping the machine out of existence:

> System Settings → Lock Screen → **"Prevent automatic sleeping when the display is off"**

A sleeping node is an offline node. Do this before the first sync, not after.

---

## 7. Verification before any key is inserted

Do not add this node to the authority set until all three checks pass.

### 7.1 Same chain

```bash
curl -s http://localhost:9944 -H 'Content-Type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}'
```

Run the same against Ceiba (`http://100.119.97.38:9944`). Compare block numbers — they
should be close, and the gap should shrink, not grow.

### 7.2 Same genesis

Both nodes must report an identical genesis hash. If they differ, the Mac is on a
different chain and everything else is meaningless.

### 7.3 Survives an epoch boundary

An epoch is **300 slots ≈ 30 minutes** on this chain. Watch across at least one
boundary. This is the check that catches a build made without `testnet-fast-epoch`:

```bash
# note the block number, wait 35 minutes, note it again
# a stalled node reports the same number
```

A node that stops advancing at an epoch boundary was built wrong. Rebuild; do not
"fix" it with flags.

---

## 8. Promoting it to an authority

Only after §7 passes and the node has stayed synced for a full epoch.

The relevant live state at the time of writing was:

| Item | Value |
|---|---|
| `session.validators` | 1 |
| `babe.authorities` | 1 |
| `grandpa.authorities` | 1 |
| `staking.validatorCount` | 2 |

### What adding a second authority actually buys

GRANDPA's threshold is `total − (total−1)/3`. That yields:

| Authorities | Threshold | Tolerates |
|---|---|---|
| 1 | 1-of-1 | 0 — chain halts |
| **2** | **2-of-2** | **0 — finality freezes** |
| 3 | 3-of-3 | 0 |
| **4** | **3-of-4** | **1** |

So 1 → 2 is a **liveness** improvement, not a fault-tolerance one: with two authorities,
losing one keeps blocks being authored by the survivor but **stops finality**. Genuine
tolerance of one failure needs **four** authorities (3-of-4), not three.

That is a real gain over today (where losing the sole authority stops everything), but it
is not redundancy — and if both nodes share a router, a power feed, or an ISP, a single
event takes out both and 2-of-2 makes that event fatal to finality.

### Rotation mechanics

Authority changes go through the session pallet:

1. insert the session keys on the Mac (`author_insertKey` / `author_rotateKeys`)
2. register the new keys against the stash account
3. wait for the rotation at the next session/epoch boundary (~30 min)

Full command detail is in [`CEIBA_OPERATIONS_RUNBOOK.md`](CEIBA_OPERATIONS_RUNBOOK.md) and
`MAINNET_READINESS`; this document deliberately does not restate them.

---

## 9. Known gaps in this document

Stated plainly so nobody mistakes a plan for a result:

| Gap | Consequence |
|---|---|
| **No macOS build has been attempted** | link errors, missing `secp256k1`/`openssl` paths, or clang issues are all possible |
| Apple Silicon vs Intel untested | both should work; neither is proven here |
| `--bootnodes` assumed reachable from the Mac | if the Mac is off-tailnet, use the LAN IP |
| Sync duration unknown | the only measured figure is Ceiba's, which had no peers to sync from |
| Port-forwarding not addressed | inbound `30333` matters for being dialled; outbound-only peering works but is weaker |

When this is actually executed, **replace this section with what happened** — including
the failures. A doc that records only the successful path teaches nobody.

---

## 10. Why not other hosts

Recorded because these were each proposed and rejected on measurement, not preference.

| Host | Blocker |
|---|---|
| **Oracle edge proxy** (`100.80.208.21`) | `aarch64`; node image is `amd64`; 2 cores; also the only ingress, so co-locating an authority couples two critical roles |
| **Rented GPU** (vast.ai / HF / Modal) | ephemeral, NAT'd, hourly — unusable for keys |
| **Cloudflare Tunnel / new VPS** | viable for *ingress*, unrelated to validation |

The GPU boxes are for **training** Nawal/Kinich. Serving them stays on Ceiba; a rented
box cannot hold the always-on inference role either.
