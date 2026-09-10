# Recovery bundle: `recovery/ceiba-20260424/`

> Filed 2026-09-10 during the workspace-wide docs truthfulness audit — this bundle existed but was referenced by no doc anywhere.

**Location:** `/home/wicked/Projects/Belizechain/recovery/ceiba-20260424/` (workspace root, outside repo git for recovery/ — keep off public remotes; contains keys).

## Contents
- `chain-keys/` — chain key material snapshot
- `chain-paths-20260424.tgz` — archived chain-sync paths/state
- `systemd/` — Ceiba host systemd unit files
- `opt-belizechain/` — metadata mirror of host `/opt/belizechain`
- `metadata/` — bundle metadata

## Provenance
Captured 2026-04-24 as a Ceiba host recovery snapshot (chain keys + service definitions) during Phase 2 stabilization.

## Handling rules
- Never commit `chain-keys/` contents to any remote repo; treat as sensitive.
- If a fresh Ceiba host rebuild is needed, this bundle + `belizechain/docs/operations/DISASTER_RECOVERY.md` are the reference pair.
- Verify against `infra/docker-compose.ceiba.yml` service list before restoring: bundle predates the current 4-validator compose split.
