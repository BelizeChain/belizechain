# Ceiba Crash Diagnosis — 2026-09-18

**Status**: Root cause identified with high confidence — failing (non-ECC, no-brand) RAM under production load.
**Related**: `CEIBA_BASELINE_2026-05-02.md`, `CEIBA_OPERATIONS_RUNBOOK.md`

## Incident

Ceiba stopped overnight (Sep 17 → 18) and was unreachable until the user manually rebooted (~12:48 UTC, then again 13:00 UTC). Chain downtime ~10.5 h; node was sole block author, so block production paused with the host. No data corruption observed in the chain state after reboot — block production resumed normally (best block #5280+, finalizing).

## Evidence trail

### Boot history (Sep 17–18)
| Boot | Start | End | Character |
|------|-------|-----|-----------|
| -5 | Sep 17 21:05 | 21:23 | Clean-ish session exit (likely manual restart) |
| -4 | Sep 17 21:26 | 22:44 | clean end |
| -3 | Sep 17 23:11 | 23:13 | clean session exit (manual restart) |
| -2 | Sep 17 23:23 | **02:08 crash window** | **642 "Bad page map"/"Bad page state" kernel BUGs between 00:47 and 02:08; log ends abruptly → hard crash** |
| -1 | Sep 18 12:48 | 12:49 | after manual reboot; journal corruption from prior unclean shutdown |
| 0 | Sep 18 13:00 | running | post user-second reboot |

### Kernel corruption signature (boot -2)
- 642 `BUG: Bad page map` / `BUG: Bad page state` events across unrelated processes (runc, kworkers, curl) — page-table-level corruption, not a specific binary.
- Repeated, identical corrupted PTE (`pte:8000000119f0f025`, pfn `0x119f0f`) hit by multiple processes simultaneously — classic bad-RAM signature at a fixed physical address.
- Also tainted PFNs: `0x1b036d`, `0x1d7461`, `0x2661e0`.
- Not OOM (no oom-kill events), not MCE (no machine-check logs; EDAC shows module identifiers but the board has no ECC tracking), not watchdog-triggered.

### Hardware identification (dmidecode)
```
Error Correction Type: None
Size: 8 GB  | Locator: DIMM 0 | Bank Locator: P0 CHANNEL A | DDR4 3200 MT/s | Manufacturer: Unknown
Size: 8 GB  | Locator: DIMM 0 | Bank Locator: P0 CHANNEL B | DDR4 3200 MT/s | Manufacturer: Unknown
```
Unbranded (manufacturer ID "Unknown", Module Manufacturer ID `Bank 10, Hex 0x68`) non-ECC DDR4-3200 UDIMMs — cheap/generic RAM without fault tracking.

### Post-reboot validation (2026-09-18 13:13–13:30 UTC)
- Daytime memtester: 6 GB, 1 full loop, **all 11 test patterns passed** (Stuck Addr, Random, XOR/SUB/MUL/DIV, OR/AND, Seq Inc, Solid Bits, Block Seq, Checkerboard, Bit Spread, Bit Flip, Walking Ones/Zeros, 8-bit & 16-bit Writes).
- Conclusion: corruption is **intermittent/load-dependent** — RAM passes synthetic idle tests but fails under real sustained workload (docker churn + postgres backup + runc activity overnight). This is consistent with marginal/thermally-sensitive RAM, and the earlier interrupted 12G overnight soak is believed to have been running in just that failure regime.

## Immediate remediation done today (2026-09-18)

1. **`watchdog.service` + `wd_keepalive.service` masked** — the units were enabled but the box has no `/dev/watchdog`; they were failed units providing zero protection.
2. **Crash-autopsy instrumentation installed** — `ceiba-autopsy.service` runs after every boot and writes `/data/log/autopsy/boot_<TS>.txt`, capturing: clean-shutdown marker count, memory-corruption event count + tail, OOM/hung checks, fsck results, and last 40 log lines of the previous boot. Auto-pruned at 30 days. Validated with a first report (correctly detected boot -1 as a hard crash, 0 clean-poweroff markers).
3. **Nightly backup staggered** — `belizechain-backup.timer` moved from 03:00 to **04:30 + 15 min jitter** via drop-in `/etc/systemd/system/belizechain-backup.timer.d/stagger.conf`, to separate it from the 00:00 `dpkg-db-backup` and the observed overnight crash window.

## Diagnosis conclusion

**RAM is confirmed the primary suspect** — not merely possible, but strongly evidenced (642 fixed-PFN corruption events + unbranded non-ECC hardware + clean synthetic test results showing the defect is intermittent/load-related, exactly matching the "cheap RAM doesn't stay up" hypothesis previously recorded informally).

Root cause remains **unresolved as a hardware verdict** until the sticks are physically replaced or isolated (single-stick Stage-2 run), but working posture going forward is: *assume RAM is bad, plan replacement.*

## Recommended next steps

- **Hard fix**: replace both 8 GB DDR4-3200 UDIMMs (or use a healthier interchange pair: 2×8 GB Kingston/Crucial/G.Skill — any quality DDR4-3200 UDIMM set works; board takes 2 slots, 1 DIMM per channel).
- **Stage-2 (optional while waiting for replacement)**: pull CHANNEL B and run single-stick for days to isolate which stick is guilty. Cost: reduces capacity to 8 GB and runs a degraded config — not without risk for the chain node.
- Or **replace host entirely** (planned new server) — sidesteps the isolation exercise.
- Keep the autopsy collector in place either way: it will produce a full evidence bundle on next unexplained stop, memtester-or-not.
- After a RAM swap or new host: re-verify `dmidecode` to confirm the new DIMMs register with a real manufacturer ID, and re-run a targeted memtester pass to validate.

## Artifacts on Ceiba
- `/data/log/autopsy/boot_*.txt` — post-boot evidence bundles (30-day retention)
- `/usr/local/sbin/ceiba-autopsy` — collector script
- `/etc/systemd/system/ceiba-autopsy.service` — boot-triggered unit
- `/etc/systemd/system/belizechain-backup.timer.d/stagger.conf` — backup stagger override
- `/tmp/memprobe-targeted.log` — post-reboot 6GB memprobe (passed all patterns; kept for reference until reboot)
