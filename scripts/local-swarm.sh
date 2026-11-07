#!/usr/bin/env bash
# BelizeChain local validator swarm — Phase 3 rehearsal harness.
#
# Builds a throwaway N-authority BABE+GRANDPA chain from the `testnet-template`
# genesis, with real randomly-generated session keys (not dev seeds), and runs N
# native node processes on one host. Nothing here touches Ceiba or the live
# `belizechain_testnet` chain.
#
# Why a script: every network-parameter mistake below fails *silently* at
# runtime rather than at startup, and two of them cost real debugging time:
#
#   1. `key generate-node-key` loads a chain spec (upstream sc-cli derives the
#      default filename from the chain id). Our mainnet config fails closed on
#      an empty council, so the command dies unless `--chain <spec>` is passed.
#   2. `author_insertKey` against a *running* authority inserts the key into the
#      keystore but the GRANDPA voter has already started with `local_key: None`
#      and never notices. BABE picks the key up on its next slot, so the chain
#      produces blocks and looks healthy while finality stays pinned at genesis.
#      This script inserts keys with `key insert` *before* first start instead.
#
# Usage:
#   scripts/local-swarm.sh up      # generate keys, build spec, start N nodes
#   scripts/local-swarm.sh status  # peers / best / finalized per node
#   scripts/local-swarm.sh down    # stop nodes, keep data
#   scripts/local-swarm.sh clean   # stop nodes and delete WORK
#
# Env:
#   NODE        node binary         (default ./target/release/belizechain-node)
#   WORK        working dir         (default /tmp/belizechain-swarm)
#   N           authorities         (default 4)
#   CHAIN_ID    genesis chain id    (default belizechain_swarm)
#   BASE_P2P / BASE_RPC / BASE_PROM  first port, incremented per node
#   WASM        runtime wasm embedded as genesis :code. Must come from the same
#               build as NODE, or the wasm and native runtimes disagree about
#               constants such as BabeEpochDuration. Defaults to the wbuild
#               artifact next to NODE.
#   FUND        genesis balance per authority, in plancks

set -euo pipefail

NODE=${NODE:-./target/release/belizechain-node}
WORK=${WORK:-/tmp/belizechain-swarm}
N=${N:-4}
CHAIN_ID=${CHAIN_ID:-belizechain_swarm}
BASE_P2P=${BASE_P2P:-30333}
BASE_RPC=${BASE_RPC:-9944}
BASE_PROM=${BASE_PROM:-9615}
FUND=${FUND:-1000000000000000000000000}

die() { printf 'error: %s\n' "$*" >&2; exit 1; }
note() { printf '  %s\n' "$*"; }

resolve_wasm() {
    if [[ -n ${WASM:-} ]]; then
        [[ -f $WASM ]] || die "WASM not found: $WASM"
        printf '%s' "$WASM"
        return
    fi
    local dir guess
    dir=$(dirname "$NODE")
    guess="$dir/wbuild/belizechain-runtime/belizechain_runtime.compact.wasm"
    [[ -f $guess ]] || die "runtime wasm not found at $guess. Build the runtime too, or set WASM=."
    printf '%s' "$guess"
}

require_binary() {
    [[ -x $NODE ]] || die "node binary not executable: $NODE (set NODE=, or run cargo build --release)"
}

# build-spec echoes the entire genesis blob into its error text, so dump only the
# head of the log — otherwise the actionable message scrolls out of view.
fail_with_log() {
    printf 'error: %s\n' "$1" >&2
    head -c 1200 "$2" | sed 's/^/  /' >&2
    printf '\n  (full log: %s)\n' "$2" >&2
    exit 1
}

# ---------------------------------------------------------------- key material

generate_keys() {
    mkdir -p "$WORK/keys" "$WORK/nodes"
    : >"$WORK/keytable.tsv"
    local i seed sr ed
    local sr_hex sr_ss58 ed_hex ed_ss58
    for ((i = 1; i <= N; i++)); do
        [[ -f $WORK/keys/v$i.secret ]] || openssl rand -hex 32 >"$WORK/keys/v$i.secret"
        seed="0x$(cat "$WORK/keys/v$i.secret")"
        sr=$("$NODE" key inspect --scheme sr25519 "$seed")
        ed=$("$NODE" key inspect --scheme ed25519 "$seed")
        # `key inspect` right-pads its values, so take the last whitespace-delimited
        # field. Splitting on ': ' instead leaves the padding glued to the value and
        # produces an SS58 string that fails base58 decoding at build-spec time.
        sr_hex=$(awk '/Public key \(hex\)/ {print $NF}' <<<"$sr")
        sr_ss58=$(awk '/SS58 Address/ {print $NF}' <<<"$sr")
        ed_hex=$(awk '/Public key \(hex\)/ {print $NF}' <<<"$ed")
        ed_ss58=$(awk '/SS58 Address/ {print $NF}' <<<"$ed")
        [[ -n $sr_hex && -n $ed_hex ]] || die "key inspect produced no public key for validator $i"
        printf 'v%d\t%s\t%s\t%s\t%s\n' "$i" "$sr_hex" "$sr_ss58" "$ed_hex" "$ed_ss58" >>"$WORK/keytable.tsv"
    done
    local distinct
    distinct=$(cut -f2 "$WORK/keytable.tsv" | sort -u | wc -l)
    ((distinct == N)) || die "generated $distinct distinct sr25519 keys, expected $N"
    note "generated $N distinct authority keypairs"
}

# ---------------------------------------------------------------------- spec

build_spec() {
    local wasm plain patched raw errlog
    wasm=$(resolve_wasm)
    plain=$WORK/spec-plain.json
    patched=$WORK/spec-patched.json
    raw=$WORK/spec-raw.json
    errlog=$WORK/spec.err

    # testnet-template refuses to build without the dev-seed acknowledgement;
    # the seeds are replaced below, so this only unlocks the template.
    BELIZECHAIN_ALLOW_DEV_SEEDS=1 "$NODE" build-spec --chain testnet-template \
        --disable-default-bootnode >"$plain" 2>"$errlog" \
        || fail_with_log "build-spec (testnet-template) failed" "$errlog"

    python3 - "$plain" "$patched" "$wasm" "$CHAIN_ID" "$FUND" "$WORK/keytable.tsv" <<'PY'
import json
import sys

plain, patched, wasm_path, chain_id, fund, keytable = sys.argv[1:7]
fund = int(fund)

with open(plain) as fh:
    spec = json.load(fh)
runtime_genesis = spec['genesis']['runtimeGenesis']
patch = runtime_genesis['patch']

rows = [line.rstrip('\n').split('\t') for line in open(keytable) if line.strip()]

# session.keys entries are [account, account, {babe, grandpa}]. BABE ids are
# sr25519, GRANDPA ids are ed25519 — passing the wrong scheme here yields a set
# no node can match, and the only symptom is finality never advancing.
patch['session']['keys'] = [
    [sr_ss58, sr_ss58, {'babe': sr_ss58, 'grandpa': ed_ss58}]
    for _, _, sr_ss58, _, ed_ss58 in rows
]
patch['sudo']['key'] = rows[0][2]

endowed = {account for account, _ in patch['balances']['balances']}
for _, _, sr_ss58, _, _ in rows:
    if sr_ss58 not in endowed:
        patch['balances']['balances'].append([sr_ss58, fund])

# A plain spec carries whatever :code blob it already had; build-spec copies it
# verbatim, so a stale blob silently wins over the binary's own runtime. Pin it
# to the artifact built alongside the node binary. It is a sibling of `patch`,
# not a patch field — passing it inside `patch` is rejected as an unknown field.
with open(wasm_path, 'rb') as fh:
    runtime_genesis['code'] = '0x' + fh.read().hex()

spec['id'] = chain_id
spec['name'] = 'BelizeChain Local Swarm'
spec['bootNodes'] = []

with open(patched, 'w') as fh:
    json.dump(spec, fh, indent=2)

print('  authorities: %d  sudo: %s  balances: %d'
      % (len(patch['session']['keys']), patch['sudo']['key'], len(patch['balances']['balances'])))
PY

    BELIZECHAIN_ALLOW_DEV_SEEDS=1 "$NODE" build-spec --chain "$patched" \
        --disable-default-bootnode --raw >"$raw" 2>"$errlog" \
        || fail_with_log "building the raw spec failed" "$errlog"
    note "raw spec: $raw ($(du -h "$raw" | cut -f1), chain id $CHAIN_ID)"
}

# --------------------------------------------------------------- key storage

# `key insert` accepts no --base-path: it takes --keystore-path and writes there
# verbatim (it does not append `chains/<id>/`). The node looks for keys in
# <base-path>/chains/<chain-id>/keystore, so that exact path has to be passed in.
# Keys must also be on disk before the node's first start (see header note 2).
insert_keys() {
    local i seed sr_ss58 ed_ss58 keystore
    for ((i = 1; i <= N; i++)); do
        seed="0x$(cat "$WORK/keys/v$i.secret")"
        sr_ss58=$(cut -f3 "$WORK/keytable.tsv" | sed -n "${i}p")
        ed_ss58=$(cut -f5 "$WORK/keytable.tsv" | sed -n "${i}p")
        keystore=$WORK/n$i/chains/$CHAIN_ID/keystore
        mkdir -p "$keystore"

        "$NODE" key insert --chain "$WORK/spec-raw.json" --keystore-path "$keystore" \
            --key-type babe --scheme sr25519 --suri "$seed" >/dev/null
        "$NODE" key insert --chain "$WORK/spec-raw.json" --keystore-path "$keystore" \
            --key-type gran --scheme ed25519 --suri "$seed" >/dev/null

        # key insert is silent on success, so verify rather than trust it.
        local nb ng
        nb=$(find "$keystore" -maxdepth 1 -name '62616265*' | wc -l)
        ng=$(find "$keystore" -maxdepth 1 -name '6772616e*' | wc -l)
        ((nb == 1 && ng == 1)) || die "v$i keystore has babe=$nb gran=$ng files in $keystore, expected 1 each"
        note "v$i keystore: babe ${sr_ss58:0:12}… gran ${ed_ss58:0:12}…"
    done
}

# ------------------------------------------------------------------ processes

start_nodes() {
    mkdir -p "$WORK/logs"
    local bootpeer i p2p rpc prom

    # Node keys must exist before bootpeer is read, and `key generate-node-key`
    # needs an explicit --chain for the same reason as in the header note 1.
    for ((i = 1; i <= N; i++)); do
        if [[ ! -f $WORK/nodes/n$i.key ]]; then
            "$NODE" key generate-node-key --chain "$WORK/spec-raw.json" \
                --file "$WORK/nodes/n$i.key" >/dev/null 2>&1
        fi
        "$NODE" key inspect-node-key --file "$WORK/nodes/n$i.key" >"$WORK/nodes/n$i.peerid"
    done
    bootpeer=$(cat "$WORK/nodes/n1.peerid")

    for ((i = 1; i <= N; i++)); do
        p2p=$((BASE_P2P + i - 1))
        rpc=$((BASE_RPC + i - 1))
        prom=$((BASE_PROM + i - 1))

        local bootargs=()
        if ((i > 1)); then
            bootargs=(--bootnodes "/ip4/127.0.0.1/tcp/$BASE_P2P/p2p/$bootpeer")
        fi

        setsid nohup "$NODE" \
            --chain "$WORK/spec-raw.json" \
            --base-path "$WORK/n$i" \
            --node-key-file "$WORK/nodes/n$i.key" \
            --validator \
            --port "$p2p" --rpc-port "$rpc" --prometheus-port "$prom" \
            --rpc-methods unsafe --rpc-cors all \
            --name "swarm-v$i" --no-telemetry \
            "${bootargs[@]}" \
            >"$WORK/logs/v$i.log" 2>&1 &
        note "v$i started  p2p=$p2p rpc=$rpc prom=$prom"
    done
    disown -a 2>/dev/null || true
}

stop_nodes() {
    local pat="base-path $WORK/n"
    local pid
    while read -r pid; do
        kill "$pid" 2>/dev/null || true
    done < <(pgrep -f "$pat" || true)
    note "stopped nodes under $WORK"
}

# --------------------------------------------------------------------- status

rpc_call() {
    curl -s -H 'Content-Type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":${3:-[]}}" \
        "http://127.0.0.1:$1"
}

status() {
    local i rpc peers best finheight
    for ((i = 1; i <= N; i++)); do
        rpc=$((BASE_RPC + i - 1))
        peers=$(rpc_call "$rpc" system_health | python3 -c 'import json,sys;print(json.load(sys.stdin)["result"]["peers"])' 2>/dev/null || echo '?')
        best=$(rpc_call "$rpc" chain_getHeader | python3 -c 'import json,sys;print(int(json.load(sys.stdin)["result"]["number"],16))' 2>/dev/null || echo '?')
        finheight=$(rpc_call "$rpc" chain_getFinalizedHead | python3 -c '
import json, sys, urllib.request
head = json.load(sys.stdin)["result"]
req = urllib.request.Request("http://127.0.0.1:%d" % int(sys.argv[1]),
    data=json.dumps({"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[head]}).encode(),
    headers={"Content-Type":"application/json"})
print(int(json.load(urllib.request.urlopen(req, timeout=10))["result"]["number"], 16))
' "$rpc" 2>/dev/null || echo '?')
        printf '  v%d rpc=%s  peers=%s  best=%s  finalized=%s\n' "$i" "$rpc" "$peers" "$best" "$finheight"
    done
}

# ----------------------------------------------------------------------- main

[[ $# -ge 1 ]] || die "usage: $0 {up|status|down|clean}"
cmd=$1
require_binary

case $cmd in
up)
    printf 'BelizeChain local swarm: %s authorities, chain id %s\n' "$N" "$CHAIN_ID"
    # A second `up` would start another set on the same ports and the new nodes
    # would fail to bind. Refuse rather than half-start.
    if pgrep -f "base-path $WORK/n" >/dev/null 2>&1; then
        die "nodes are already running under $WORK — use '$0 down' first (or '$0 status')."
    fi
    mkdir -p "$WORK"
    generate_keys
    build_spec
    insert_keys
    start_nodes
    printf '\nNodes are starting. Check with: %s status  (finality lags best by ~2 blocks)\n' "$0"
    ;;
status) status ;;
down) stop_nodes ;;
clean)
    stop_nodes
    rm -rf "$WORK"
    note "removed $WORK"
    ;;
*) die "unknown command: $cmd (expected up|status|down|clean)" ;;
esac
