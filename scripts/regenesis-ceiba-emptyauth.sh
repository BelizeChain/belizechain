#!/usr/bin/env bash
#
# Re-genesis the Ceiba testnet onto a runtime carrying the empty-authority-set fix.
#
# Run this ON SIDEBAND (the workstation) only, or on Ceiba only with --assume-yes.
#
# Context: the previous chain (genesis 0x631fb936…) deadlocked at block #28791 on
# 2026-09-23 because block 14393 announced a NextEpochData digest with ZERO
# authorities. There is no rollback asset that predates that block — every
# /data/backups/chain snapshot was taken after it — so the chain cannot be
# recovered in place. This script wipes only the chain database. The keystore is
# preserved and matches the genesis authority.
#
# SAFETY: every mutating step is gated behind --assume-yes and prints what it
# will do first. Without the flag it is a dry run.
#
# shellcheck disable=SC2016
# SC2016 fires on the `'"${VAR}"'` idiom used below. Those expansions are
# deliberately deferred to the REMOTE host: the strings are arguments to
# `ssh_run`, so they must survive the local shell unexpanded. ShellCheck cannot
# see across the ssh boundary and reports them as unexpanded by mistake.
set -euo pipefail

# No defaults for the target: this script WIPES a chain database, so the host must
# be stated rather than inherited from whatever was live when the default was
# written. (It used to default to `wicked@ceiba`, which now runs a dead chain.)
# Live testnet (Chain D on ceiba):
#   CEIBA_HOST=wicked@ceiba CEIBA_RPC=http://100.119.97.38:9944
CEIBA_HOST="${CEIBA_HOST:-}"
CEIBA_RPC="${CEIBA_RPC:-}"
CEIBA_ROOT="${CEIBA_ROOT:-/opt/belizechain}"
CHAIN_DIR="/data/chain"
CHAIN_SUBDIR="${CHAIN_DIR}/chains/belizechain_testnet"
STAMP="$(date +%Y%m%d-%H%M%S)"

NEW_IMAGE_TAG="${NEW_IMAGE_TAG:-}"
NEW_SPEC="${NEW_SPEC:-}"
ASSUME_YES=0
SKIP_BACKUP=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --assume-yes)   ASSUME_YES=1; shift ;;
        --image-tag)    NEW_IMAGE_TAG="$2"; shift 2 ;;
        --spec)         NEW_SPEC="$2"; shift 2 ;;
        --skip-backup)  SKIP_BACKUP=1; shift ;;
        -h|--help) sed -n '2,20p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

fail() { echo "ERROR: $*" >&2; exit 1; }
step() { echo; echo "=== $* ==="; }

[[ -n "$NEW_IMAGE_TAG" ]] || fail "--image-tag is required (e.g. belizechain/ceiba-node:<sha>-emptyauth-<date>)"
[[ -n "$NEW_SPEC" ]]      || fail "--spec is required (path to the newly generated testnet spec)"

# Validate the TARGET here, with the other required inputs, and before the dry-run
# summary — so a dry run can never print an empty host and look like a valid plan.
[[ -n "$CEIBA_HOST" ]] || fail "CEIBA_HOST is required — this script wipes a chain database, so it will not guess the target.
  Live testnet (Chain D on ceiba): CEIBA_HOST=wicked@ceiba CEIBA_RPC=http://100.119.97.38:9944"
[[ -n "$CEIBA_RPC" ]]  || fail "CEIBA_RPC is required (the host's JSON-RPC URL, used to watch block production).
  Live testnet (Chain D on ceiba): http://100.119.97.38:9944"

if [[ "$ASSUME_YES" != "1" ]]; then
    cat <<EOF
DRY RUN — nothing will change. Re-run with --assume-yes to execute.

  host          : ${CEIBA_HOST}
  compose root  : ${CEIBA_ROOT}
  new image     : ${NEW_IMAGE_TAG}
  new spec      : ${NEW_SPEC}
  chain dir     : ${CHAIN_SUBDIR}
  wipe          : ${CHAIN_SUBDIR}/db   (chain database only)
  preserve      : ${CHAIN_SUBDIR}/keystore
  backup stamp  : ${STAMP}
EOF
    exit 0
fi

ssh_run() { ssh -o ConnectTimeout=15 "$CEIBA_HOST" "$@"; }

step "0. Preflight"
ssh_run 'set -e
  test -d '"${CHAIN_SUBDIR}"' || { echo "missing chain dir"; exit 1; }
  test -d '"${CHAIN_SUBDIR}"'/keystore || { echo "missing keystore"; exit 1; }
  echo "keystore files (must survive):"
  ls -1 '"${CHAIN_SUBDIR}"'/keystore | sed "s/^/  /"
  echo "chain dir size: $(du -sh '"${CHAIN_SUBDIR}"'/db 2>/dev/null | cut -f1)"
  echo "current block: $(curl -s -m 10 '"${CEIBA_RPC}"' -H "Content-Type: application/json" \
      -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\",\"params\":[]}" \
      | python3 -c "import sys,json;print(int(json.load(sys.stdin)[\"result\"][\"number\"],16))" 2>/dev/null || echo unreachable)"
  echo "free disk: $(df -h /data | tail -1 | awk "{print \$4}")"
'

if [[ "$SKIP_BACKUP" != "1" ]]; then
    step "1. Back up .env, spec, and the stalled database"
    ssh_run 'set -e
      cd '"${CEIBA_ROOT}"'
      mkdir -p /data/regenesis-backup-'"${STAMP}"'
      cp -a .env /data/regenesis-backup-'"${STAMP}"'/.env
      cp -a '"${CHAIN_DIR}"'/testnet-spec.json /data/regenesis-backup-'"${STAMP}"'/testnet-spec.old.json
      cp -a '"${NEW_SPEC}"' '"${CHAIN_DIR}"'/testnet-spec.new.json
      echo "backing up chain db (forensic evidence of the stall)…"
      tar -czf /data/regenesis-backup-'"${STAMP}"'/db-before-regenesis.tar.gz \
        -C '"${CHAIN_DIR}"'/chains belizechain_testnet/db
      ls -la /data/regenesis-backup-'"${STAMP}"'/
      du -sh /data/regenesis-backup-'"${STAMP}"'
    '
else
    step "1. Backup SKIPPED (--skip-backup)"
    ssh_run "cp -a ${NEW_SPEC} ${CHAIN_DIR}/testnet-spec.new.json && echo 'new spec staged'"
fi

step "2. Install the new image tag and spec, stop the node"
ssh_run 'set -e
  cd '"${CEIBA_ROOT}"'
  cp -a .env .env.bak-pre-emptyauth-'"${STAMP}"'
  python3 - <<PY
import re, pathlib
p = pathlib.Path(".env")
s = p.read_text()
new = "CEIBA_NODE_IMAGE='"${NEW_IMAGE_TAG}"'"
s2, n = re.subn(r"^CEIBA_NODE_IMAGE=.*$", new, s, flags=re.M)
assert n == 1, f"expected exactly 1 CEIBA_NODE_IMAGE line, replaced {n}"
p.write_text(s2)
print("  .env updated:", new)
PY
  mv -f '"${CHAIN_DIR}"'/testnet-spec.new.json '"${CHAIN_DIR}"'/testnet-spec.json
  echo "  spec installed"
  docker compose -f docker-compose.ceiba.yml --env-file .env stop ceiba-node
  echo "  node stopped"
'

step "3. Wipe ONLY the chain database (keystore preserved)"
ssh_run 'set -e
  test -d '"${CHAIN_SUBDIR}"'/keystore || { echo "keystore vanished - aborting"; exit 1; }
  rm -rf '"${CHAIN_SUBDIR}"'/db '"${CHAIN_SUBDIR}"'/network
  echo "  db + network removed"
  echo "  keystore still present:"; ls -1 '"${CHAIN_SUBDIR}"'/keystore | sed "s/^/    /"
'

step "4. Start the node"
ssh_run 'set -e
  cd '"${CEIBA_ROOT}"'
  docker compose -f docker-compose.ceiba.yml --env-file .env up -d ceiba-node
  sleep 12
  docker ps --filter name=ceiba-node --format "{{.Names}}  {{.Status}}"
  echo
  echo "--- first log lines ---"
  docker logs ceiba-node --tail 25 2>&1
'

step "5. Verify block production"
ssh_run 'set -e
  echo "waiting for the chain to boot…"
  for i in $(seq 1 30); do
    sleep 4
    H=$(curl -s -m 5 '"${CEIBA_RPC}"' -H "Content-Type: application/json" \
        -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\",\"params\":[]}" 2>/dev/null || true)
    if [[ -n "$H" ]]; then
      N=$(echo "$H" | python3 -c "import sys,json;print(int(json.load(sys.stdin)[\"result\"][\"number\"],16))" 2>/dev/null || echo "")
      if [[ -n "$N" ]]; then
        echo "  head block: #$N"
        if [[ "$N" -gt 3 ]]; then break; fi
      fi
    fi
  done
  echo
  echo "--- blocks over 24s (expect ~6s cadence) ---"
  for i in 1 2 3 4; do
    curl -s -m 5 '"${CEIBA_RPC}"' -H "Content-Type: application/json" \
      -d "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\",\"params\":[]}" \
      | python3 -c "import sys,json,datetime;print(\"   \", datetime.datetime.now().strftime(\"%H:%M:%S\"), \"block\", int(json.load(sys.stdin)[\"result\"][\"number\"],16))"
    sleep 6
  done
  echo
  echo "--- authoring preflight ---"
  docker logs ceiba-node 2>&1 | grep -i "authoring preflight" | tail -2
'

cat <<EOF

Re-genesis complete. Next:
  - Watch the first epoch rotation (~30 min with testnet-fast-epoch):
      ssh ${CEIBA_HOST} 'docker logs -f ceiba-node 2>&1 | grep -Ei "epoch|session|NewSession"'
    Expect a session/epoch change WITHOUT the chain going idle.
  - Confirm no stall guard fires:
      ssh ${CEIBA_HOST} 'docker logs ceiba-node 2>&1 | grep "STALL DETECTED" | tail'
  - Re-register the Nawal AI operator. REQUIRED, and easy to miss: identity and
    staking state do NOT survive a re-genesis, so the operator silently drops out
    of Staking::Validators. Nothing breaks visibly - the chain keeps producing
    blocks - but the live-actions smoke test starts failing with
    "Nawal operator is not registered as a validator on chain".
      RPC_ENDPOINT=ws://<host>:9944 NODE_PATH=<repo>/ui/node_modules \\
        NAWAL_SEED="\$(ssh ${CEIBA_HOST} 'docker exec ceiba-nawal printenv NAWAL_KEYPAIR_URI')" \\
        node scripts/register-nawal-signer.js
    The signer holds NO session keys by design (it is PoUW-only). BelizeSessionManager
    filters it out of the queued authority set - that is what keeps BABE at one
    authority instead of announcing an empty next epoch. Expect a log line naming it
    as excluded; that is correct, not an error.
  - Backups: /data/regenesis-backup-${STAMP}/
EOF
