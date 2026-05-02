# Ceiba Backup Restore Drill - 2026-05-02

Status: Phase 2 drill plan with read-only preflight completed
Host: `ceiba` at `100.81.45.25` over Tailscale
Compose path: `/opt/belizechain`

This drill covers the Phase 2 exit criterion for restore readiness across chain data, Postgres, service volumes, and compose/env state. The 2026-05-02 pass was intentionally non-destructive: no live restore, service restart, database overwrite, or volume replacement was performed.

## Scope

| Surface | Live Path Or Resource | Restore Target For Drill |
|---|---|---|
| Compose state | `/opt/belizechain/docker-compose.ceiba.yml` | copied config plus `docker compose config --quiet` |
| Environment state | `/opt/belizechain/.env` | copied file, never printed in full |
| Chain spec | `/data/chain/testnet-spec.json` | copied file plus checksum |
| Chain data | `/data/chain/chains/belizechain_testnet` | scratch extraction or maintenance restore |
| Postgres | `/data/postgres`, container `ceiba-postgres` | disposable restore database |
| Pakit volumes | `belizechain_pakit-storage`, `belizechain_pakit-cache` | disposable Docker volume or scratch extraction |
| Kinich volume | `belizechain_kinich-results` | disposable Docker volume or scratch extraction |
| Service data dirs | `/data/nawal`, `/data/ipfs`, `/data/prometheus`, `/data/grafana`, `/data/redis`, `/data/logs` | scratch extraction and manifest validation |

## Read-Only Preflight Results

Collected on 2026-05-02 after the consensus recovery baseline:

- Compose file: `/opt/belizechain/docker-compose.ceiba.yml`
- Env file: `/opt/belizechain/.env`
- Active chain data: `/data/chain/chains/belizechain_testnet`
- Docker volumes present:
  - `belizechain_pakit-storage`
  - `belizechain_pakit-cache`
  - `belizechain_kinich-results`
- Data directories observed under `/data`: `chain`, `postgres`, `backups/postgres`, `chain-backups`, `nawal`, `ipfs`, `prometheus`, `grafana`, `redis`, `logs`, `nginx`

Validation checks performed:

| Check | Result |
|---|---|
| Latest Postgres dump `/data/backups/postgres/belizechain_20260502_030001.sql.gz` | `gzip -t` passed |
| Existing chain backup `/data/chain-backups/chain-pre-node-rollout-20260429-014556.tar.gz` | `tar -tzf` listed expected `chain/chains/belizechain_testnet/...` entries |
| Compose render | `docker compose -f docker-compose.ceiba.yml --env-file .env config --quiet` passed |
| Volume inspect | Pakit and Kinich named volumes resolved to Docker volume mountpoints |

Gap: the readable chain tarball is a pre-node-rollout backup from 2026-04-29, not a current post-recovery backup. Create a fresh chain backup during the first scheduled maintenance drill.

## Blast Radius

| Drill Action | Blast Radius | Preferred Timing |
|---|---|---|
| Config/env copy and checksum | none, read-only | anytime |
| Postgres dump validation | low read IO | anytime |
| Restore into disposable Postgres database | low, touches Postgres but not live app database | quiet window |
| Chain backup while node runs | medium IO, backup may not be crash-consistent | quiet window only |
| Chain backup with node stopped | node stops producing/finalizing blocks | announced maintenance window |
| Chain restore to live path | node downtime and possible data rollback | explicit maintenance window only |
| Service volume restore to live volumes | affected service downtime and data replacement | explicit maintenance window only |

## Rollback Rules

- Never overwrite a live path without first moving it to a stamped `.pre-restore.<stamp>` path.
- Keep the pre-drill compose file, env file, chain spec, and live data paths until the restored service passes health checks.
- For live chain restore, rollback is: stop `ceiba-node`, move restored `/data/chain` aside, move `/data/chain.pre-restore.<stamp>` back to `/data/chain`, start `ceiba-node`, verify RPC and finality.
- For Postgres live restore, rollback is: stop dependent services, stop Postgres writes, restore the pre-drill database dump, restart services, verify service health.
- For service volume restore, rollback is: stop the affected service, swap the `.pre-restore.<stamp>` volume/data path back, start the service, verify health and data lookups.

## Drill Procedure

Set a stamp and prepare an isolated backup root:

```bash
stamp=$(date +%Y%m%d-%H%M%S)
backup_root="/data/backups/phase2-restore-drill-$stamp"
sudo mkdir -p "$backup_root"
cd /opt/belizechain
```

Capture config and compose state:

```bash
sudo cp docker-compose.ceiba.yml "$backup_root/docker-compose.ceiba.yml"
sudo cp .env "$backup_root/env.source"
sudo cp /data/chain/testnet-spec.json "$backup_root/testnet-spec.json"
sudo sha256sum "$backup_root"/* > "$backup_root/SHA256SUMS"
docker compose -f docker-compose.ceiba.yml --env-file .env config --quiet
```

Create a current chain backup. For crash consistency, use the maintenance-window variant:

```bash
docker compose -f docker-compose.ceiba.yml --env-file .env stop ceiba-node
sudo tar -C /data -czf "$backup_root/chain-$stamp.tgz" chain
docker compose -f docker-compose.ceiba.yml --env-file .env up -d ceiba-node
```

Lower-impact alternative for inventory-only testing:

```bash
sudo tar -C /data -czf "$backup_root/chain-live-readonly-$stamp.tgz" chain
```

Capture Postgres:

```bash
docker exec ceiba-postgres sh -lc 'pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB"' \
  | gzip > "$backup_root/postgres-$stamp.sql.gz"
gzip -t "$backup_root/postgres-$stamp.sql.gz"
```

Capture service volumes into tarballs without writing to the volumes:

```bash
for volume in belizechain_pakit-storage belizechain_pakit-cache belizechain_kinich-results; do
  docker run --rm \
    -v "$volume:/src:ro" \
    -v "$backup_root:/backup" \
    alpine:3.20 sh -lc "tar -C /src -czf /backup/$volume-$stamp.tgz ."
done
```

Validate restore into scratch paths without replacing live data:

```bash
scratch="/tmp/ceiba-restore-drill-$stamp"
mkdir -p "$scratch/chain" "$scratch/volumes"
tar -xzf "$backup_root/chain-$stamp.tgz" -C "$scratch/chain"
test -d "$scratch/chain/chain/chains/belizechain_testnet"

for archive in "$backup_root"/belizechain_*.tgz; do
  name=$(basename "$archive" .tgz)
  mkdir -p "$scratch/volumes/$name"
  tar -xzf "$archive" -C "$scratch/volumes/$name"
done
```

Validate Postgres restore into a disposable database:

```bash
restore_db="belizechain_restore_drill_$stamp"
docker exec ceiba-postgres sh -lc 'createdb -U "$POSTGRES_USER" "'$restore_db'"'
gzip -dc "$backup_root/postgres-$stamp.sql.gz" \
  | docker exec -i ceiba-postgres sh -lc 'psql -U "$POSTGRES_USER" "'$restore_db'" >/tmp/restore-drill-psql.log'
docker exec ceiba-postgres sh -lc 'psql -U "$POSTGRES_USER" "'$restore_db'" -c "\\dt"'
docker exec ceiba-postgres sh -lc 'dropdb -U "$POSTGRES_USER" "'$restore_db'"'
```

## Success Criteria

- Fresh compose/env/spec artifacts exist under the drill backup root and checksums are recorded.
- A current chain backup tarball can be listed and extracted into scratch space.
- The latest Postgres dump passes `gzip -t` and restores into a disposable database.
- Pakit and Kinich service volumes can be archived and extracted without touching live volumes.
- After any maintenance-window node stop, `ceiba-node` returns to healthy state and best/finalized heads advance.
- Results are copied into the next dated Ceiba baseline or this drill document.

## Current Decision

Do not run the live restore portion until there is an explicit maintenance window. The safe next step is to create a fresh post-recovery chain backup and perform scratch extraction plus disposable Postgres restore. Live chain or volume replacement should be reserved for a scheduled drill with rollback owner present.