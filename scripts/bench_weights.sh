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
#
# Prerequisites:
#   cargo build --release -p belizechain-node --features runtime-benchmarks
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
# This guard exists because a partially-benchmarked pallet is easy to overlook:
# `community` has 15 weight functions but only 4 benchmarks.
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

    echo "=== Generating weights: ${pallet} (steps=${STEPS} repeat=${REPEAT}) ==="
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

    echo ">>> OK: ${pallet} → ${output#"${ROOT_DIR}/"}"
done

echo ""
if [[ "$FAIL" -gt 0 ]]; then
    echo "FAILED: ${FAIL} pallet(s)" >&2
    exit 1
fi
echo "All requested pallets generated weights successfully."
echo "Next: re-run \`cargo test\` for the touched pallets and commit the weights."
