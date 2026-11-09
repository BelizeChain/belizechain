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
| Detected | 2026-11-09 **18:07:12 UTC** by the edge uptime monitor (38 s after the NIC detached); Telegram alert sent 18:07:43 |
| Recovered | 2026-11-09 **20:47:13 UTC**, after a **manual reboot** |
| MTTR | 2 h 40 m — every minute of it waiting for a human, not for a fix |
| Recurrence | **Possible.** This is a known defect of this NIC. No mitigation applied yet (§5) |

## 2. Timeline (UTC)

| Time | Event | Evidence source |
|---|---|---|
| 18:04:30 | Last scheduled `ceiba-health-log.service` run completes normally | journal |
| **18:06:33** | `pcieport 0000:03:00.0: Unable to change power state from D3hot to D0, device inaccessible` | kernel |
| **18:06:34** | `igc 0000:06:00.0 enp6s0: PCIe link lost, device now detached` | kernel |
| 18:07:11 | First DNS failure (`dockerd` resolver: i/o timeout) — *after* the detach | `dockerd` |
| 18:07:12 | Edge monitor: `FAIL public_https … HTTP 504`, `FAIL public_rpc`, `FAIL wallet_host`, `FAIL chain_advancing` | edge journal |
| 18:07:35 | `tailscaled`: log upload failed, `context deadline exceeded` | `tailscaled` |
| 18:07:43 | `ALERT sent (first failure)` → Telegram | edge journal |
| 18:08:07 | `tailscaled`: `no-derp-connection`, "Internet connection might be down" | `tailscaled` |
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

The failure was detected in **38 seconds** and still took 2 h 40 m to resolve. Detection
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

## 7. Remediation

Applied and verified 2026-11-09 20:55 UTC, **without a reboot**. #4 was not applied.

| # | Action | Status | Reboot? | Rollback |
|---|---|---|---|---|
| 1 | Pin every PCIe **bridge** to `power/control=on`, removing D3hot as a reachable state | ✅ applied, persisted via udev | no | `echo auto > /sys/bus/pci/devices/<addr>/power/control`, delete the rule |
| 2 | Persist `pcie_aspm=off` in the GRUB cmdline | ⬜ **not applied** — needs a reboot | **yes** | — |
| 3 | Local watchdog: reboot when the NIC leaves the PCI bus or has no carrier | ✅ applied, timer armed | no | `systemctl disable --now ceiba-nic-watchdog.timer` |
| 4 | `node-exporter` + NIC/host alerts in Prometheus | ⬜ **not applied** | no | — |

### Why #1 is `power/control` and not ASPM

The kernel's own message names the mechanism: `pcieport 0000:03:00.0: Unable to change
power state from **D3hot** to D0, device inaccessible`. That is a **device power state**,
not an ASPM link state, so disabling ASPM would have addressed a neighbouring problem.
`pcie_aspm=off` is the commonly-cited fix for *i225/i226 link loss*, and it may well help,
but the recorded failure was D3hot specifically.

Nine bridges were runtime-suspendable. All nine — including `00:01.2`, `02:00.2` and
`03:09.0`, the three on the NIC's own path (`00:01.2 → 02:00.2 → 03:09.0 → 06:00.0`) — are
now pinned to D0 by `/etc/udev/rules.d/99-belizechain-pci-d0.rules`, so the setting survives
a reboot without a kernel cmdline change. The NIC itself was already `on`; it was the
bridge above it that could sleep.

### The watchdog, and the line it will not cross

`ceiba-nic-watchdog.timer` runs every 5 minutes and reboots only when the NIC has been
**absent from the PCI bus**, or **carrier-down**, for **two consecutive checks** (10 min).
It never consults internet reachability: an ISP outage or a switch reboot leaves the PCI
device present and the chain healthy, and rebooting for that would be worse than the outage
it exists to fix.

Rails: 2 strikes before acting, at most one reboot per 2 h, and it defers if a backup is
running. Worst case is therefore one extra reboot per 2 h, and only while the host is
already disconnected and unable to serve anyone.

**Proven, not assumed** — every branch was exercised against a throwaway state directory
with the address pointed at a non-existent device, so the real NIC was never involved:

| Scenario | Result |
|---|---|
| Device absent, 1st check | `strike 1/2`, no action |
| Device absent, 2nd check | `would reboot now` |
| Device returns | `recovered … clearing 3 strike(s)`, counter → 0 |
| Device absent, rebooted 60 s ago | `would reboot but rate-limited (60s since last, limit 7200s)` |
| Device absent, rebooted 3 h ago | `would reboot now` |
| Interface missing (carrier path) | `unhealthy (carrier down on …)`, escalates the same way |

Artifacts live in `infra/deploy/` (`host-sbin/ceiba-nic-watchdog.sh`,
`ceiba-nic-watchdog.{service,timer}`, `99-belizechain-pci-d0.rules`) and are byte-identical
to the host — `check-host-drift.sh` maps all four, so a host rebuild cannot lose them
silently.

### Still open

- **#2** — until the cmdline is set, the mitigation survives reboots only because of udev.
  That is a real mechanism, not a placeholder, but `pcie_aspm=off` is the belt to that
  braces. Batch it with the pending 125/126 runtime upgrade so the chain halts once.
- **#4** — the host still has no `node-exporter`, so the next detach will again present as
  a 504 rather than as "the NIC went away". Worth doing; not done here because it edits the
  shared compose stack and CI-validated config for no gain in recovery time.

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
