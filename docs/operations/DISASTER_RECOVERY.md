# Disaster Recovery — Ceiba (single host)

**Rewritten**: 2026-11-22. This file previously described a multi-validator network with
staking, slashing, a datacenter failover and a mandatory quarterly drill. **None of that
exists.** There is one host, one authority, and no failover; the old text was wrong about
the architecture in every section, and its RTO table ("node crash: 5 minutes") was never
measured against anything. It is preserved in git history, not here.

Live values (host address, container pins, chain state) are not restated in this file — see
[`../ECOSYSTEM_STATE.md`](../ECOSYSTEM_STATE.md) §2. This file describes **procedure**.

---

## 1. What is actually deployed

| Property | Reality | Consequence for DR |
|---|---|---|
| Hosts | **One** (`ceiba`), owned hardware | Host loss = total outage. There is nothing to fail over to |
| Chain authorities | **One** | No quorum to survive. No slashing risk, because no stake is at risk |
| Containers | ~21 under one compose project | Recovery is "restore data, start compose" — not a cluster rebuild |
| Orchestration | Docker Compose, no k8s/Helm | `docker compose up -d` is the whole control plane |
| Chain data | `/data/chain` (bind mount), ~830 MB measured 2026-11-22 | Restorable from a daily archive |
| Admin access | Tailscale + SSH key | A lost Tailscale identity blocks recovery; keep a second admin key |
| Public entry | One hostname → nginx → containers | DNS + TLS are part of the recovery path, not decoration |

**The honest summary: this is a single-server deployment with backups.** Anything that
needs a second machine — failover, rolling upgrade safety, quorum loss — is out of scope
here and belongs in a pre-mainnet plan, not in a runbook.

---

## 2. Backup inventory — verified on the host 2026-11-22

Ceiba runs `belizechain-backup.timer` daily (~04:36–04:43 UTC). It executes five streams
in sequence and then verifies the chain archive:

| Stream | Contents | On-host path | Retention |
|---|---|---|---|
| `backup-postgres.sh` | `pg_dump --clean --if-exists`, gzipped | `/data/backups/postgres` | 7 days |
| `backup-chain.sh` | `tar.zst` of `/data/chain` **after stopping the node** | `/data/backups/chain` | 7 days |
| `backup-pakit.sh` | `tar.zst` of pakit storage | `/data/backups/pakit` | 7 days |
| `backup-nawal.sh` | `tar.zst` of `/data/nawal`, incl. `mesh/ed25519.key` | `/data/backups/nawal` | 7 days |
| `backup-secrets.sh` | `/opt/belizechain/.env`, **AES-256-CBC + PBKDF2 (600k iters)**, round-trip verified | `/data/backups/secrets` | 7 days |

The secrets stream was added 2026-11-22 and is the one that cannot be reconstructed from
anything else in this table. Measured that day: `.env` holds two **12-word signer mnemonics**
(`NAWAL_KEYPAIR_URI`, `PAKIT_SIGNER_URI`) which exist in no other file on the host and in no
other backup. Without them the stack starts but two on-chain identities do not exist.

> ⚠️ **Custody requirement — and a live gap as of 2026-11-23.** The passphrase is
> meant to live at `/etc/belizechain/backup-passphrase` (mode 600, root) so the
> nightly job can run unattended. **It does not exist yet**, so the secrets step
> currently exits 1 and the nightly unit reports failed; the four data streams and
> the chain verification run before it and are unaffected. That is deliberate: a
> green unit would claim the `.env` is backed up when it is not.
>
> A human must type the passphrase and hold the same value off this host — it
> cannot be generated for you, because anything an agent generates has passed
> through a transcript. Until it exists, **`.env` and the two signer mnemonics in
> it are not backed up at all.** If both the host and the passphrase are ever lost
> after that, every `env_*.env.enc` is noise — and so are the mnemonics inside them.

Two details that matter and are easy to get wrong:

- **The chain archive is taken with the node stopped.** A `tar` of a live RocksDB is not a
  usable backup, and a file that untars cleanly can still fail to open. That is why
  `verify-chain-backup.sh` boots the archive in isolation and waits for a new block
  (~17 s measured) instead of only checking the tarball. A backup that has never been
  opened is a hypothesis.
- **The Nawal mesh identity exists nowhere else.** `mesh/ed25519.key` is pinned by every
  peer that has seen it; regenerating it makes this node a stranger. That step runs as
  root (`ExecStart=+…`) because the key is mode 600 under a uid-999 tree — the only place
  in this stack where a backup needs privilege, and it is deliberate.

### Off-host copies

Data on the same host as the failure it protects is not a backup, so there are two
independent pulls:

| Custodian | Mechanism | Schedule |
|---|---|---|
| Oracle edge proxy | `belizechain-edge-backup-pull.timer` | daily |
| This workstation | `belizechain-backup-pull.timer` (**user** unit) | daily 05:30 UTC, `Persistent=true` |

Both keep the newest few of each archive kind. The pull runs after the on-host backup so
it copies that night's archive rather than yesterday's. **A stalled pull is worth checking
before trusting the off-host copy** — the 2026-11-09 measurement found the newest off-host
archive was 38 days old because the script existed and nothing called it.

---

## 3. What is NOT backed up

Measured against the five scripts above — none of them reads these paths:

| Path | Why it matters | Recoverable? |
|---|---|---|
| `/opt/belizechain/nginx/certs/privkey.pem` | TLS private key | Re-issuable, but only after DNS is pointed somewhere that can answer the ACME challenge |
| `/opt/belizechain/docker-compose.ceiba.yml` | The deployment itself | Yes — it lives in `infra/` in git |

The `.env` gap that used to head this table is **closed**. It held every service secret —
DB password, `AUTH_SESSION_SECRET`, image pins, and the two signer mnemonics — and the only
copies were `.env.bak-*` files on the same disk, which is not a backup. It is now streamed
nightly, encrypted, and pulled off-host with both custodians.

The remaining two rows are recoverable without a backup, which is why they are not streamed.
That is a judgement, not an oversight: re-issuing the certificate needs a DNS change nothing
can automate, and the compose file is the artifact under version control.

---

## 4. RTO and RPO — what is known, what is not

**RPO is known**: 24 hours for all five streams, because each runs nightly and is
verified. Chain data after that window is re-obtainable by re-syncing, at a time cost that
has not been measured on this hardware.

**RTO is not known, and no number is given here on purpose.** No restore has ever been
performed against a live target. The only drill (2026-09-17) restored into scratch and
disposable targets and explicitly performed *no* live restore, no service restart and no
volume replacement. Anything stated as minutes would be invented.

---

## 5. Restore procedures

### 5.1 PostgreSQL — the one path that is now tested

`infra/deploy/restore-postgres.sh` restores a dump into a **scratch** database and
verifies it. It never touches the live database: the default target is a throwaway
`postgres:15-alpine` container, and `--target host` creates a uniquely-named temporary
database on the real server and drops it afterwards. Restoring over production is not an
option it offers, on purpose.

```bash
cd infra
./deploy/restore-postgres.sh                     # newest dump in backups/postgres
./deploy/restore-postgres.sh --dump <file>       # a specific archive
./deploy/restore-postgres.sh --keep              # leave it up to inspect by hand
./deploy/restore-postgres.sh --target host --host-container ceiba-postgres
```

It refuses a corrupt archive (`gzip -t`, then the `.sha256` sidecar when present), and it
imports with `psql -v ON_ERROR_STOP=1` — without that, psql exits 0 after a failed
statement and the drill reports success on a broken restore.

**Measured 2026-11-22** against `belizechain_20261122_043420.sql.gz` (65.6 MB): **11/11
tables restored, 385 MB database, 8.7 s.** Row counts — `indexer.blocks` 463,123,
`indexer.transactions` 463,894, `indexer.network_snapshots` 463,123, `indexer.state` 1,
`public.kyc_verifications` 2; the rest empty.

**Measured again on the live host** (`--target host`, via
`DOCKER_HOST=ssh://wicked@ceiba`): **11/11 tables, identical row counts, 15.2 s.** The
temporary database was dropped afterwards and the live one was confirmed unaffected
(431 MB, 474,893 blocks, chain still authoring). That is the first time a restore of
current-host data has been exercised **on** the host that holds it.

Four things that measurement taught, all now encoded in the script:

- **There is no `postgres` role on Ceiba.** `psql -U postgres` fails with
  `FATAL: role "postgres" does not exist`; the only role is `belizechain` (superuser,
  createdb) and that is what the container's `POSTGRES_USER` names. The throwaway
  `postgres:15-alpine` container *does* create a `postgres` role, so the local drill
  passed while the host path was broken — the script now reads `POSTGRES_USER` from the
  target container instead of assuming.
- **The data is in the `indexer` schema, not `public`.** Four of the eleven tables
  (`blocks`, `gaps`, `network_snapshots`, `state`) live there, and `transactions` exists in
  **both** schemas with different contents (`indexer.transactions` 463,894 rows,
  `public.transactions` 0). Any check that assumes one schema, or that reduces to bare
  table names, will be wrong in both directions — the first version of this script reported
  4 false failures and miscounted 11 tables as 10.
- **The dump carries `\restrict`**, a 2025 security backport emitted by `pg_dump` 15.17. An
  older `psql` rejects it with `invalid command \restrict`, which reads like a corrupt dump
  but is a client-version problem. Restore with a 15.17-or-later client.
- **`pg_dump --clean --if-exists` is already in the archive**, so a restore is destructive
  to existing objects in the target database.

Restoring over the live database, when that day comes, is still a manual operation:

```bash
# On ceiba — DESTRUCTIVE to existing objects in ${DB_NAME}
gunzip -c /data/backups/postgres/belizechain_<timestamp>.sql.gz \
  | docker exec -i ceiba-postgres psql -U <DB_USER> -d <DB_NAME> -v ON_ERROR_STOP=1
```

### 5.2 Chain data

```bash
cd /opt/belizechain
docker compose -f docker-compose.ceiba.yml stop ceiba-node
mv /data/chain /data/chain.pre-restore-$(date +%Y%m%d-%H%M)
mkdir -p /data/chain
tar --zstd -xf /data/backups/chain/chain_<timestamp>.tar.zst -C /data/
docker compose -f docker-compose.ceiba.yml start ceiba-node
# then confirm a new block appears; verify-chain-backup.sh does this in isolation
```

**Do not restore a chain archive onto an older runtime than it requires.** The archive
contains the chain's own DB; a node binary that cannot speak its spec will fail at import.
Check the runtime version before starting.

### 5.3 Pakit and Nawal

These two are **not the same shape**, which is easy to get wrong:

- **Pakit** archives a **Docker named volume**, not a host directory — the script streams
  it out with `docker run -v "${VOLUME}:/data:ro" … tar -C /data -cf - .`. Restore means
  writing back into that volume (currently `belizechain_pakit-storage`), not into a path.
- **Nawal** archives the host tree `/data/nawal` with `tar -C "${NAWAL_ROOT}"`, so it
  restores to a path.

Both scripts carry their own restore example in a header comment. Nawal's tree is owned by
uid 999 under a mode-750 directory, so restore as root and leave ownership intact — a
restore that silently chowns is a restore that breaks the mesh identity lookup.

To prove a night's archives are actually restorable before you need them:

```bash
cd infra && ./deploy/restore-archives.sh            # newest of each, into a temp dir
cd infra && ./deploy/restore-archives.sh --keep     # same, but leaves the tree for inspection
```

It extracts to a scratch directory it creates and removes, checks sizes and the `.sha256`
sidecar, runs `PRAGMA integrity_check` on pakit's databases, and confirms nawal's
`mesh/ed25519.key` is intact (printing a fingerprint, never the key). Run without `root` it
states plainly that ownership was **not** verified. It is wired into `deploy/ci-local.sh`,
which runs it whenever `backups/pakit` and `backups/nawal` are present locally.

### 5.4 Secrets — `.env`

`infra/deploy/backup-secrets.sh` encrypts the file and proves it decrypts back before the
artifact is kept; a ciphertext that cannot be read is deleted rather than stored, because a
backup nobody can open is worse than an honest absence. Restore during a rebuild:

```bash
cd infra
./deploy/backup-secrets.sh --decrypt backups/secrets/env_20261122_043500.env.enc \
                           --to /tmp/restored.env
```

It refuses to overwrite an existing file, writes the plaintext mode 600, and never prints
the passphrase. The passphrase comes from `PASSPHRASE_FILE` (default
`/etc/belizechain/backup-passphrase`).

**The passphrase is not recoverable from this repository, this host, or these backups.**
It is the one secret a human must hold. If it is lost, every `env_*.env.enc` on every
custodian is undecryptable, and the two signer mnemonics are gone with it.

---

## 6. Drills — performed and not performed

| Date | Scope | Result |
|---|---|---|
| 2026-09-17 | Non-destructive: every backup surface exercised, every restore path validated into scratch/disposable targets | Passed. **No live restore, no service restart, no volume replacement, no chain rollback**. Ran against `100.81.45.25`, decommissioned 2026-10-07 — so it validated the *old* host's data |
| **2026-11-22** | **PostgreSQL restore of the current host's newest dump**, into a throwaway container, via `restore-postgres.sh` | **Passed: 11/11 tables, 463,123 blocks, 385 MB, 8.7 s.** First restore validated against data Ceiba actually holds today |
| **2026-11-22** | Same, `--target host` — temporary database created **on the live server**, imported, verified, dropped | **Passed: 11/11 tables, identical row counts, 15.2 s.** Temp DB confirmed gone; live DB and chain confirmed unaffected |
| **2026-11-22** | **pakit + nawal archives**, via `restore-archives.sh` | **Passed.** Both open, match their `.sha256` sidecars, and every archived member is present at its recorded size. pakit's two SQLite files pass `PRAGMA integrity_check` and hold **the same row counts as the live database on all 8 tables** (5 `content_metadata`, 2 `content_roots`, 5 `dedup_references`, 2 `user_metadata`, 1 `storage_stats`, 0 in the three unused ones). nawal's `mesh/ed25519.key` is present at 64 bytes — the file that exists nowhere else |
| **nightly, continuous** | **Chain archive**, by `verify-chain-backup.sh` as the last step of `belizechain-backup.service` | **Passed every night.** Each archive is extracted to scratch, booted in a throwaway container from the same image the live node runs, and must **author a new block** before the unit succeeds. Last measured run 2026-11-22 04:35 UTC: opened at block **666330** (spec 128), authored **666330 → 666332**, unit `Result=success`. Whole step costs ~17 s |
| — | Live-restore drill (maintenance window, rollback owner present) | **Never performed** |
| — | Full host rebuild from the archives plus the encrypted `.env` | **Never performed** |

**All four data streams are now proven to restore.** Postgres was proven on 2026-11-22,
pakit and nawal on 2026-11-22, and the chain every night since the verifier was added to
the unit. That is a stronger position than this document claimed: an earlier revision of
this table said the chain archive was "only ever extracted into scratch", which was wrong —
`verify-chain-backup.sh` boots it and requires it to produce a block, and the host's journal
shows it doing so.

The gap that remains is not any single stream. It is the last row: no one has rebuilt the
stack from nothing, and no restore has been performed against a *live* target while the old
one is still running. Each stream is proven in isolation; their combination is not.

The pakit archive holds 9 files while the live volume holds 11. The two absent files are
`pakit_metadata.db-wal` and `pakit_metadata.db-shm` — SQLite write-ahead sidecars, which a
backup must not copy. That was checked rather than assumed: the restored `.db` carries the
same row counts as the live database with its WAL applied, so nothing committed was lost.

`restore-archives.sh` fails rather than skips when an archive is missing, truncated, or
fails its checksum — verified against all three cases. An earlier draft reported
`RESTORE OK` after verifying nothing, which is the exact failure mode a drill exists to
prevent.

---

## 7. Cold start — rebuilding the host from nothing

A full from-scratch build has been done **once**, on 2026-10-06/07, and the record is
`infra/docs/NEW_SERVER_MIGRATION.md`. Nothing since has repeated it, so read that as
history plus a procedure rather than a tested runbook for today's pins.

What a rebuild actually requires, in order:

1. **Host**: Ubuntu, Docker + Compose, Tailscale, the `wicked` user with passwordless
   sudo, and the port forwards for 80/443/30333.
2. **Repo**: `infra` onto the host at `/opt/belizechain` (it is **not** a git checkout in
   production — files are copied in).
3. **`.env`**: from `.env.example` plus the values that are deliberately blank —
   `AUTH_SESSION_SECRET` (`openssl rand -base64 48`) and the DB password. Verified
   2026-11-22: a pristine `git archive` of `infra` plus `.env.example` and that one
   generated secret produces a valid `docker compose config`. No other variable is
   missing, so the file set is genuinely self-contained.
4. **Images — the real barrier.** None of the ~10 `belizechain/*` images exists in a
   registry, and `docker-compose.ceiba.yml` has no `build:` sections, so it cannot build
   them either. They must be rebuilt from the sibling repos on another machine and moved
   with `docker save | docker load` (13 images, node image first — 24 s measured) or built
   in place. **This step, not data restore, dominates a cold-start RTO, and it has never
   been timed end to end.**
5. **Data**: restore the four data archives per §5, and `.env` per §5.4.
6. **TLS**: `deploy/init-letsencrypt.sh`; DNS must be pointed at the new host first or the
   ACME challenge cannot be answered.

Two traps that cost time in the one real migration and are easy to miss:

- **`/data/grafana`, `/data/prometheus`, `/data/alertmanager` must be owned by the
  container users** (`472:0`, `65534:65534`, `472:0`). Docker creates missing bind-mount
  sources as **root**, and both services then restart-loop. Ownership is host state, not
  repo state.
- **`maya` does not start under a plain `docker compose up -d`.** It carries
  `profiles: ["maya"]`, so it needs `--profile maya`. A restore that omits it restores
  everything except the wallet.

---

## 8. Single points of failure

1. **The host.** One machine, one PSU, one NIC (which has already needed a PCIe-detach
   recovery). No spare.
2. **The authority.** One node authors. Losing it stops the chain rather than degrading it.
3. **The workstation.** It is one of the two off-host custodians, and it is a laptop.
4. **The `.env`.** No off-host copy. See §3.

---

## 9. Before mainnet

These are prerequisites, not nice-to-haves, and none is satisfied today:

- A tested live restore, on a schedule, with a named owner.
- Off-host copies of `.env` and the TLS key, in a secret store rather than a `.bak` on the
  same disk.
- More than one authority, so a single host loss is a degraded chain rather than a stopped
  one.
- An RTO that was measured at least once instead of assumed.

---

## Related

- [`../ECOSYSTEM_STATE.md`](../ECOSYSTEM_STATE.md) — live values and open workstreams
- [`CEIBA_OPERATIONS_RUNBOOK.md`](CEIBA_OPERATIONS_RUNBOOK.md) — day-to-day operation
- [`CEIBA_BACKUP_RESTORE_DRILL_2026-09-17.md`](CEIBA_BACKUP_RESTORE_DRILL_2026-09-17.md) — the drill's full record
- [`RECOVERY_BUNDLE_CEIBA_20260424.md`](RECOVERY_BUNDLE_CEIBA_20260424.md) — host-identity bundle
- `infra/deploy/backup-*.sh`, `verify-chain-backup.sh`, `pull-ceiba-backups.sh` — the scripts above
