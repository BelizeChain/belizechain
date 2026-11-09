# Ceiba host isolation — Intel I226-V PCIe detach, 2026-11-09

**Status: resolved by manual reboot. Root cause confirmed. Recurrence mitigation NOT yet applied.**

**Written 2026-11-09 20:55 UTC from the host's own journal, the edge monitor's log,
and the chain.** Every figure below was measured; nothing is inferred.

---

## 1. Summary

| | |
|---|---|
| What failed | The onboard **Intel I226-V** NIC (`igc`, `0000:06:00.0`, `enp6s0`) detached from the PCIe bus |
| Effect | Host lost all networking — no route, no DNS — for **2 h 41 m** |
| Chain impact | **None.** Block production continued the entire time |
| Detected | 2026-11-09 **18:07:12 UTC** by the edge uptime monitor → Telegram |
| Recovered | 2026-11-09 **20:47:13 UTC**, after a **manual reboot** |
| MTTR | 2 h 40 m — every minute of it waiting for a human, not for a fix |
| Recurrence | **Possible.** This is a known defect of this NIC. No mitigation applied yet (§5) |

## 2. Timeline (UTC)

| Time | Event | Evidence source |
|---|---|---|
| 18:00 | Log volume jumps 35 → 1,400 lines/15 min — DNS and connectivity start failing | `journalctl -b -1` density |
| **18:06:33** | `pcieport 0000:03:00.0: Unable to change power state from D3hot to D0, device inaccessible` | kernel |
| **18:06:34** | `igc 0000:06:00.0 enp6s0: PCIe link lost, device now detached` | kernel |
| 18:07:12 | Edge monitor: `FAIL public_https … HTTP 504`, `FAIL public_rpc`, `FAIL wallet_host`, `FAIL chain_advancing` | edge journal |
| 18:07:43 | `ALERT sent (first failure)` → Telegram | edge journal |
| 18:11:54 | Node still importing: `Imported #487353` — **host is healthy, only unreachable** | `docker logs ceiba-node` |
| 20:44:00 | Node still importing: `#488874` | `docker logs ceiba-node` |
| 20:45:46 | Final journal entry of the previous boot | `journalctl --list-boots` |
| 20:46:06 | Last block before restart: `#488895` | `docker logs ceiba-node` |
| 20:46:50 | New boot begins | `journalctl --list-boots` |
| 20:47:03 | `ceiba-node` container up (`RestartCount=0`) | `docker inspect` |
| 20:47:12 | Monitor: `PASS chain_advancing: block 488892 (+1599)`, all checks green | edge journal |
| 20:47:13 | `RECOVERY sent` | edge journal |

## 3. What did NOT happen

Ruling these out matters as much as the finding, because each would have implied a
different fix:

- **Not a crash or panic.** No kernel `BUG:`, `panic`, `hung task`, or `soft lockup` anywhere
  in the boot.
- **Not memory pressure.** No `oom-kill` or `Killed process` entry in ~34 days of journal.
  Host currently: 3,536 MB used of 30,974 MB.
- **Not disk.** `/` at 15 % (125 G of 915 G); no `I/O error`, `medium error`, `ata error`,
  `nvme error`, or `EXT4-fs error`.
- **Not thermal.** No `throttl` or thermal-trip messages.
- **Not a clean shutdown.** No `systemd-shutdown` / `Reached target Reboot` at the end of
  the boot — consistent with the NIC going away and the box simply becoming unreachable.
- **Not recurring.** This is the **first and only** occurrence in the retained journal
  (which reaches back to 2026-10-07): `grep -c "PCIe link lost"` → **1**.
- **Not a chain problem.** See §4.

## 4. The chain was never down

This is the most important operational fact, and it is counter-intuitive, so it is
stated with the numbers:

- 18:11:54 → `#487353`; 20:44:00 → `#488874`.
- 1,521 blocks / 9,921 s = **6.04 s per block** — nominal.
- `system_health` today: `peers: 0, shouldHavePeers: false` — single authority, so no peer
  connectivity is needed to produce blocks.

A network outage on this chain isolates the node from the outside world but **does not**
stop it. Anyone reading "site returns 504" as "the chain is down" would draw the wrong
conclusion, including about whether an emergency upgrade is safe mid-outage.

What **was** lost is 2 minutes of production at the reboot itself: `#488895` (20:46:06)
to the restart at 20:47:03. On restart the node resumed from a slightly earlier tip
(the monitor read `#488892` at 20:47:12), which is normal for an unclean stop and
harmless under single-authority BABE.

## 5. Root cause and why it will happen again

**The Intel I226-V has a well-known PCIe link-loss defect.** The kernel recorded the
exact signature: the upstream port failed to bring the device out of `D3hot`, and the
device detached. The I226-V is `rev 06`, firmware `2023:889d`, driver `igc` in
`7.0.0-42-generic`, on a **x1 PCIe link** (`4.000 Gb/s available`).

Current power-management state — **neither mitigation is in place**:

| Knob | Current value | Relevance |
|---|---|---|
| `/sys/module/pcie_aspm/parameters/policy` | `default` (i.e. ASPM enabled) | The D3hot failure is a power-management path |
| EEE (`ethtool --show-eee`) | `disabled` | Already off — not the trigger here |
| `/dev/watchdog` | **absent** | No hardware watchdog to reset the box |
| `node-exporter` | **not deployed** | Host health is entirely unmonitored (§6) |

Nothing on the host can recover from this state by itself: the NIC is gone until the
machine is power-cycled. That is why MTTR was 2 h 40 m.

## 6. The real gap this exposed

The failure was detected in **40 seconds** and still took 2 h 40 m to resolve. Detection
was never the problem — **recovery was**.

1. **No automatic recovery.** A device that detaches from PCIe cannot be re-enumerated
   reliably in software; a reboot is required. Nothing triggers one.
2. **No host-level metrics.** `deploy/prometheus.yml` scrapes containers only; there is no
   `node-exporter`, so NIC state, link errors, and PCIe events are invisible to Prometheus
   and Alerts. The only signal was the external 504.
3. **The alerting that did work, worked by design.** `edge-uptime-monitor.sh` runs on the
   edge host — a different machine, different provider — and posts straight to Telegram
   without touching Ceiba. Its own header says this exact reason. That design decision is
   the reason this outage was noticed at all.

## 7. Recommended remediation — NOT applied, awaiting approval

Requires an explicit decision because (a) it touches host power management and (b) the
durable form needs a reboot, which halts the chain for ~1–2 minutes.

| # | Action | Reboot? | Blast radius | Rollback |
|---|---|---|---|---|
| 1 | `echo performance > /sys/module/pcie_aspm/parameters/policy` — disables ASPM L0s/L1 on all PCIe devices | no | Higher idle power draw; no functional change | Echo `default` back |
| 2 | Persist `pcie_aspm=off` in the GRUB cmdline | **yes** | One ~1–2 min chain halt | Remove the parameter, `update-grub`, reboot |
| 3 | Local watchdog: if `enp6s0` is absent from `lspci` **or** has no carrier for ≥5 min, `systemctl reboot` | no | A bad rule could reboot-loop the host | Disable the timer — this is the one to test carefully |
| 4 | Add `node-exporter` + a `CeibaNicLinkDown` / `CeibaHostUnreachable` alert | no | One more container | Remove from compose |

**Ordering.** #3 is what converts a 2 h 40 m outage into ≤5 min and is the highest value.
It must key on **NIC absence or carrier loss**, never on internet reachability, or an ISP
outage would reboot a perfectly healthy chain host. #1 and #2 reduce the probability of the
event; #4 makes the next occurrence legible. #2's reboot should be batched with the next
runtime upgrade (125/126 are pending), not taken on its own.

## 8. Verification after recovery (2026-11-09 20:52 UTC)

| Check | Result |
|---|---|
| `tailscale status` | `ceiba` online |
| Host uptime | 4 min, load 0.60 |
| Containers | **20/20 up**; no restart-looping |
| `ceiba-node` | healthy, `spec 124`, `tx 3`, `peers 0` |
| Block production | `#488,920` → `#488,922` in 14 s (`#488,938` at 20:51:48) |
| `https://…/health` | 200 |
| `/api/nawal/health`, `/api/pakit/health`, `/api/kinich/health` | 200, 200, 200 |
| `/rpc` (GET) | 405 — correct, the endpoint is POST-only |
| Kernel errors since boot | 1 (`KHO: Failed to reserve lowmem scratch buffer` — kexec-over, benign) |
| PCIe link-lost / AER errors since boot | **0** |
| Memory | 3,536 MB / 30,974 MB |
| Disk | 125 G / 915 G (15 %) |
| Edge monitor state | `STATUS=ok`, all 5 checks PASS |
