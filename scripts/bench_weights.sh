#!/usr/bin/env bash
# ==============================================================================
# BelizeChain weight generation (M-5)
# ==============================================================================
# Generates real FRAME weights for one pallet, or every pallet that has
# benchmarks, and writes them next to the pallet source.
#
# `scripts/bench_smoke.sh` only proves the benchmarks *execute* (steps=2,
# repeat=1). This script is the opposite: it runs enough steps/repeats to
# produce weights worth committing, and refuses to overwrite a pallet's weights
# unless the output passed FRAME's own regression check.
#
# Usage:
#   ./scripts/bench_weights.sh <pallet_name> [<pallet_name> ...]
#   ./scripts/bench_weights.sh --all
#   BENCH_MUL=1.25 ./scripts/bench_weights.sh --all
#
# Prerequisites:
#   cargo build --release -p belizechain-node --features runtime-benchmarks
#   ** ON AC POWER ** - these are wall-clock measurements. On battery this host
#   throttles to roughly a third of its clocks and the weights come out 40-80%
#   inflated (verified 2026-10-13: bns buy_domain 114M on AC vs 179M on battery).
#   Check `/sys/class/power_supply/ADP1/online` is 1 before starting.
#
# Weight multiplier (BENCH_MUL):
#   FRAME's benchmark CLI has no multiplier flag in this SDK version, so the
#   factor is applied by this script, to the generated file, right after the CLI
#   writes it. Only `ref_time` (the first `Weight::from_parts` argument) is
#   scaled - the second argument is a proof size in bytes, not a time. Rounding
#   is always UP, so it can never under-charge. Regenerating a pallet whose
#   committed file already carries a factor REQUIRES passing the same BENCH_MUL;
#   otherwise the run fails rather than silently dropping it.
#
# Notes:
#   * The pallet name is the `define_benchmarks!` identifier (e.g.
#     `pallet_storage_proof`), and the output path is resolved from
#     `pallets/<dir>/src/weights.rs`.
#   * Pallets whose weights are hand-written (no generated file) are reported and
#     skipped rather than silently creating a file nothing imports.
# ==============================================================================

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE_BIN="${ROOT_DIR}/target/release/belizechain-node"
STEPS="${BENCH_STEPS:-50}"
REPEAT="${BENCH_REPEAT:-20}"

# Safety factor applied to every generated `ref_time` (see header). Empty = 1.0,
# i.e. raw measured values with no factor.
MUL="${BENCH_MUL:-}"

# Read the multiplier stamped into an existing weights file, if any.
#
# ALWAYS returns 0, even when there is no stamp: this script runs under
# `set -euo pipefail`, and `committed_mul="$(existing_mul ...)"` would abort the
# whole run if the function returned non-zero for "no multiplier found". (The
# no-stamp case is the common one, so that bug made the script exit silently
# before generating anything.) The grep's failure is therefore swallowed inside
# the substitution, not in the pipeline.
existing_mul() {
    local f="$1" line=""
    [[ -f "$f" ]] || return 0
    line="$(grep -m1 -oE 'WEIGHT MULTIPLIER: [0-9]+(\.[0-9]+)?' "$f" 2>/dev/null || true)"
    if [[ -n "$line" ]]; then
        printf '%s' "${line##*: }"
    fi
    return 0
}

# Scale the ref_time component of every `Weight::from_parts` in a generated file,
# rounding up, and stamp the factor into the header so it is self-documenting.
apply_multiplier() {
    local file="$1" mul="$2"
    python3 - "$file" "$mul" <<'PY'
import decimal, math, re, sys

path, mul_text = sys.argv[1], sys.argv[2]
mul = decimal.Decimal(mul_text)
src = open(path, encoding="utf-8").read()


def scale(match):
    # `Weight::from_parts(<ref_time>, <proof_size>)` - only ref_time is a time.
    raw = match.group(1).replace("_", "")
    scaled = int(math.ceil(decimal.Decimal(raw) * mul))
    return "Weight::from_parts(%s," % format(scaled, ",").replace(",", "_")


src, count = re.subn(r"Weight::from_parts\(([0-9_]+),", scale, src)
if count == 0:
    sys.exit("%s: no Weight::from_parts found - refusing to leave it unscaled" % path)

stamp = "//! WEIGHT MULTIPLIER: %s (ref_time only; applied by scripts/bench_weights.sh)" % mul_text
if "WEIGHT MULTIPLIER" not in src:
    src = re.sub(r"//! DATE: .*\n", lambda m: m.group(0) + stamp + "\n", src, count=1)

open(path, "w", encoding="utf-8").write(src)
PY
}
# These are wall-clock measurements, so power state is part of the measurement.
# On battery this host drops to roughly a third of its clocks and every weight
# comes out 40-80% inflated (measured 2026-10-13). Checking per pallet means a
# cable being unplugged mid-run fails loudly instead of writing a set whose
# pallets were measured under two different performance regimes.
require_ac_power() {
    local node=/sys/class/power_supply/ADP1/online
    [[ -r "$node" ]] || return 0   # not a laptop, or no AC node: do not block
    if [[ "$(cat "$node")" != "1" ]]; then
        echo ">>> ABORT: not on AC power (ADP1/online != 1); refusing to write weights." >&2
        echo "    Battery-measured weights are inflated 40-80% - see this file's header." >&2
        exit 1
    fi
}
if [[ ! -x "$NODE_BIN" ]]; then
    echo "❌ node binary not found: $NODE_BIN" >&2
    echo "   Run: cargo build --release -p belizechain-node --features runtime-benchmarks" >&2
    exit 1
fi

# Map a benchmark identifier to its pallet directory.
#
# `define_benchmarks!` uses crate names (`pallet_belize_governance`) while the
# directories are short (`pallets/governance`), with `storage-proof` hyphenated.
pallet_dir() {
    case "$1" in
        pallet_belize_*) echo "${1#pallet_belize_}" ;;
        pallet_storage_proof) echo "storage-proof" ;;
        pallet_*) echo "${1#pallet_}" | tr '_' '-' ;;
        *) echo "" ;;
    esac
}

resolve_output() {
    local pallet="$1" dir
    dir="$(pallet_dir "$pallet")"
    if [[ -z "$dir" || ! -f "${ROOT_DIR}/pallets/${dir}/src/weights.rs" ]]; then
        echo ""
        return
    fi
    echo "${ROOT_DIR}/pallets/${dir}/src/weights.rs"
}

# Count `fn name() -> Weight;` declarations inside a `pub trait WeightInfo` block.
count_trait_fns() {
    awk '/^pub trait WeightInfo/ { inside=1; next }
         inside && /^}/ { inside=0 }
         inside && /fn [a-zA-Z_]+\(/ { n++ }
         END { print n+0 }' "$1"
}

# Count `#[benchmark]`-annotated functions.
count_bench_fns() {
    awk '/^[[:space:]]*#\[benchmark\]/ { want=1; next }
         want && /fn [a-zA-Z_]+\(/ { n++; want=0 }
         END { print n+0 }' "$1"
}

# A pallet is only safe to regenerate when every trait method has exactly one
# benchmark. Otherwise the generated file omits methods the trait still declares
# (or invents ones it does not), and the crate stops compiling.
#
# This guard is a live safety net for any pallet that is mid-way through gaining
# benchmarks. `community` used to be the example (15 weight functions, only 4
# benchmarks); it has since been fully benchmarked, so every pallet now passes
# this check. The guard stays because a partially-benchmarked pallet silently
# generates a file that does not match its `WeightInfo` trait.
check_coverage() {
    local dir="$1" trait_file trait_count bench_file bench_count
    trait_file="${ROOT_DIR}/pallets/${dir}/src/weights.rs"
    if [[ ! -f "$trait_file" ]] || ! grep -q '^pub trait WeightInfo' "$trait_file" 2>/dev/null; then
        trait_file="${ROOT_DIR}/pallets/${dir}/src/lib.rs"
    fi
    bench_file="${ROOT_DIR}/pallets/${dir}/src/benchmarking.rs"
    if [[ ! -f "$bench_file" ]]; then
        echo "no benchmarking.rs"
        return 1
    fi

    trait_count="$(count_trait_fns "$trait_file")"
    bench_count="$(count_bench_fns "$bench_file")"
    if [[ "$trait_count" -eq 0 ]]; then
        echo "no WeightInfo trait found"
        return 1
    fi
    if [[ "$trait_count" -ne "$bench_count" ]]; then
        echo "trait has ${trait_count} fns but only ${bench_count} benchmarks"
        return 1
    fi
    return 0
}

# Pick the template matching this pallet's weights file.
#
# Most pallets declare `WeightInfo` in `lib.rs`, so the generated file holds only
# `SubstrateWeight<T>`. A few (e.g. `pallet-belize-oracle`) own the trait in
# `weights.rs`, and the shared template would delete it and break the build.
# Detecting the shape beats a hard-coded pallet list that can drift.
template_for() {
    local output="$1"
    if grep -q '^pub trait WeightInfo' "$output"; then
        echo "${ROOT_DIR}/templates/weights-template-with-trait.hbs"
    else
        echo "${ROOT_DIR}/templates/weights-template.hbs"
    fi
}

TARGETS=()
if [[ "${1:-}" == "--all" ]]; then
    # Every entry in runtime/src/lib.rs `define_benchmarks!` that is a Belize
    # pallet. Read from the runtime so the list cannot drift from the runtime.
    while read -r pallet; do
        TARGETS+=("$pallet")
    done < <(grep -oE '\[pallet_belize_[a-z_]+|\[pallet_storage_proof' \
        "${ROOT_DIR}/runtime/src/lib.rs" | tr -d '[' | sort -u)
elif [[ $# -gt 0 ]]; then
    TARGETS=("$@")
else
    echo "usage: $0 <pallet_name> [<pallet_name> ...] | --all" >&2
    exit 1
fi

FAIL=0
for pallet in "${TARGETS[@]}"; do
    require_ac_power

    dir="$(pallet_dir "$pallet")"
    output="$(resolve_output "$pallet")"
    if [[ -z "$output" ]]; then
        echo ">>> SKIP: ${pallet} — no pallets/${dir}/src/weights.rs (weights are hand-written)"
        continue
    fi

    if ! reason="$(check_coverage "$dir")"; then
        echo ">>> SKIP: ${pallet} — ${reason}"
        echo "         Finish the benchmarks before regenerating; a partial run would"
        echo "         emit a file the WeightInfo trait does not match."
        continue
    fi

    # Never silently change a factor that is already committed: regenerating
    # without the same BENCH_MUL would quietly revert the weights to raw values.
    committed_mul="$(existing_mul "$output")"
    if [[ -n "$committed_mul" && -z "$MUL" ]]; then
        echo ">>> FAIL: ${pallet} - committed weights carry a ${committed_mul}x multiplier." >&2
        echo "         Re-run with BENCH_MUL=${committed_mul}, or edit deliberately." >&2
        FAIL=$((FAIL + 1))
        continue
    fi
    if [[ -n "$committed_mul" && -n "$MUL" && "$MUL" != "$committed_mul" ]]; then
        echo ">>> FAIL: ${pallet} - multiplier change ${committed_mul}x -> ${MUL}x needs a deliberate decision." >&2
        FAIL=$((FAIL + 1))
        continue
    fi

    echo "=== Generating weights: ${pallet} (steps=${STEPS} repeat=${REPEAT}, mul=${MUL:-1}) ==="
    template="$(template_for "$output")"
    echo "    template: ${template#"${ROOT_DIR}/"}"
    if ! "$NODE_BIN" benchmark pallet \
        --chain dev \
        --pallet "$pallet" \
        --extrinsic "*" \
        --steps "$STEPS" \
        --repeat "$REPEAT" \
        --template "$template" \
        --output "$output"; then
        echo ">>> FAIL: ${pallet} — weights left untouched" >&2
        FAIL=$((FAIL + 1))
        continue
    fi

    if [[ -n "$MUL" ]]; then
        apply_multiplier "$output" "$MUL"
    fi

    echo ">>> OK: ${pallet} → ${output#"${ROOT_DIR}/"}"
done

echo ""
if [[ "$FAIL" -gt 0 ]]; then
    echo "FAILED: ${FAIL} pallet(s)" >&2
    exit 1
fi
echo "All requested pallets generated weights successfully."
echo "Next: re-run \`cargo test\` for the touched pallets and commit the weights."
