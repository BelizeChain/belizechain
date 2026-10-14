#!/usr/bin/env bash
#
# check-doc-claims.sh — do the repos' docs and tooling agree with the code they describe?
#
# WHY THIS EXISTS
#   Docs and cross-repo references drift silently: a namespace is renamed, an
#   image pin moves, a referenced file is deleted. Between 2026-09 and 2026-10
#   every one of those was caught by hand, one conversation at a time. This
#   checker catches the MECHANICAL class automatically — deterministic greps
#   against real sources of truth. The SEMANTIC class ("is X actually live?")
#   still needs a live sweep: belizechain/.github/prompts/docs-alignment.prompt.md.
#
# USAGE
#   ./check-doc-claims.sh [path]        # path = a repo root, or a workspace root
#   ./check-doc-claims.sh --host        # also compare image pins against Ceiba's .env
#
#   Without arguments the script targets the repo it lives in. If that repo has
#   sibling BelizeChain repos next to it (belizechain/, kinich-quantum/, ...),
#   workspace mode turns on automatically and the cross-repo checks run too.
#
# EXIT
#   0 = all checks pass (or were skipped); 1 = at least one FAIL.
#
# IGNORES
#   A repo can exempt a failure by listing a substring per line (with a #reason)
#   in <repo>/.doc-claims-ignore. Exemptions print as "ignored" so they stay visible.
#
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOST_CHECK=0
TARGET=""
# Which host --host interrogates. This used to be a literal Ceiba IP baked into the ssh
# call, which after the 2026-10-06 migration is the wrong machine. From a workstation
# pass --host=<ip> or set HOST_TARGET=<ip>; on the host itself lib/host-env.sh resolves
# it. Unset and unresolvable is a loud skip, never a silent check of the wrong box.
HOST_TARGET="${HOST_TARGET:-}"

for arg in "$@"; do
  case "$arg" in
    --host) HOST_CHECK=1 ;;
    --host=*) HOST_CHECK=1; HOST_TARGET="${arg#--host=}" ;;
    *) TARGET="$arg" ;;
  esac
done

# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/host-env.sh"

# ── colors (disabled with NO_COLOR) ────────────────────────────────────────
if [ -n "${NO_COLOR:-}" ]; then
  C_RED=""; C_GRN=""; C_YLW=""; C_DIM=""; C_RST=""
else
  C_RED=$'\033[0;31m'; C_GRN=$'\033[0;32m'; C_YLW=$'\033[0;33m'
  C_DIM=$'\033[2m'; C_RST=$'\033[0m'
fi

FAILS=0
SKIPS=0
pass() { printf '  %s✅ PASS%s %s\n' "$C_GRN" "$C_RST" "$1"; }
fail() { printf '  %s❌ FAIL%s %s\n' "$C_RED" "$C_RST" "$1"; FAILS=$((FAILS + 1)); }
skip() { printf '  %s⏭️  SKIP%s %s\n' "$C_YLW" "$C_RST" "$1"; SKIPS=$((SKIPS + 1)); }
section() { printf '\n%s── %s%s\n' "$C_DIM" "$1" "$C_RST"; }

# ── locate target + workspace ──────────────────────────────────────────────
resolve_root() { # $1 = dir to resolve; prints the git toplevel, or the dir itself
  local out
  out="$(cd "$1" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)"
  if [ -n "$out" ]; then printf '%s\n' "$out"; else (cd "$1" && pwd); fi
}

if [ -n "$TARGET" ]; then
  ROOT="$(resolve_root "$TARGET")"
else
  ROOT="$(resolve_root "$SCRIPT_DIR/..")"
fi
PARENT="$(dirname "$ROOT")"

# Workspace detection: a dir counts as a workspace root when it directly
# contains BelizeChain repo folders — true for TARGET itself (pointed at the
# umbrella folder) or for its parent (pointed at one repo inside it).
has_repos() {
  local s
  for s in belizechain infra kinich-quantum nawal-ai pakit-storage gem ui; do
    [ -d "$1/$s/.git" ] || [ -d "$1/$s/.github" ] && return 0
  done
  return 1
}
WS=""
if has_repos "$ROOT"; then
  WS="$ROOT"
elif has_repos "$PARENT"; then
  WS="$PARENT"
fi

printf 'Doc-claims check — target: %s%s\n' "$ROOT" "${WS:+ (workspace mode: $WS)}"

# ── ignore helpers ─────────────────────────────────────────────────────────
# returns 0 when the failure line is exempted by <repo>/.doc-claims-ignore
is_ignored() { # $1 = repo root, $2 = failure line
  local igfile="$1/.doc-claims-ignore" line
  [ -f "$igfile" ] || return 1
  while IFS= read -r line; do
    case "$line" in ''|'#'*) continue ;; esac
    case "$2" in *"$line"*) return 0 ;; esac
  done < "$igfile"
  return 1
}

emitted() { # $1 = repo root, $2 = failure line
  if is_ignored "$1" "$2"; then
    printf '  %s⏭️  IGNORED%s %s\n' "$C_YLW" "$C_RST" "$2"
  else
    fail "$2"
  fi
}

# ── find loops (shared excludes) ───────────────────────────────────────────
md_files() { # $1 = dir
  # Excludes archive dirs on purpose: old_audits/, audit/ + audits/ (dated audit
  # records), docs/archive/ and ui-docs-archive/ are frozen snapshots of a past
  # state — their links describe what existed when they were written, not what
  # exists now.
  find "$1" -name '*.md' \
    -not -path '*/.git/*' -not -path '*/node_modules/*' -not -path '*/.next/*' \
    -not -path '*/.venv/*' -not -path '*/venv/*' -not -path '*/site-packages/*' \
    -not -path '*/target/*' -not -path '*/dist/*' -not -path '*/build/*' \
    -not -path '*/old_audits/*' -not -path '*/audit/*' -not -path '*/audits/*' \
    -not -path '*/docs/archive/*' -not -path '*/ui-docs-archive/*' \
    2>/dev/null
}

strip_code_spans() { # stdin → stdout, with `inline code` removed
  # Without this, a doc that DOCUMENTS link syntax — e.g. the sentence explaining
  # that this check looks for a bracketed label followed by a parenthesised target
  # — reads as a dead link. That happened on 2026-10-14, in this repo's own state
  # doc. Code spans are not links, so they are removed before extraction. Only the
  # backtick-delimited part goes; anything else on the line is still checked.
  # The backticks must stay literal here — SC2016 assumes they were meant as
  # command substitution, which would defeat the whole point of this filter.
  # shellcheck disable=SC2016
  sed 's/`[^`]*`//g' 2>/dev/null
}

escapes_repo() { # $1 = repo root, $2 = candidate path (the file need not exist)
  # A cross-repo link such as ../../belizechain/docs/X.md is CORRECT in a workspace
  # checkout but can never resolve in single-repo CI, where the sibling was not
  # cloned. Resolving both sides separates "the file is gone" (a defect) from "the
  # file is not in this checkout" (not checkable here, so it must not fail the run).
  local root cand
  root="$(realpath -m -- "$1" 2>/dev/null)" || return 1
  cand="$(realpath -m -- "$2" 2>/dev/null)" || return 1
  case "$cand" in "$root"/*) return 1 ;; *) return 0 ;; esac
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 1 — dead relative links in markdown (all repos)
#   Only markdown-link syntax [x](path) is checked. Bare backtick mentions and
#   absolute paths (/routes, /data/...) are deliberately ignored — too many
#   legitimate non-file targets. Catch: a link to a deleted/renamed file.
#   A link that resolves OUTSIDE the repo root is reported as a skip, not a
#   failure: it is valid in a workspace and simply not checkable in a single-repo
#   checkout. It stays visible so it never looks like it was verified.
# ═══════════════════════════════════════════════════════════════════════════
check_dead_links() { # $1 = repo root, $2 = repo label
  local repo="$1" label="$2" md target dir
  local findings
  findings="$(mktemp)"
  while IFS= read -r md; do
    dir="$(dirname "$md")"
    while IFS= read -r target; do
      # strip markdown wrapper pieces
      target="${target#](}"
      target="${target%)}"
      target="${target%% *}"      # drop "title" after a space
      target="${target#<}"; target="${target%>}"
      case "$target" in
        ''|'#'*|/*|*:*|*'{{'*|*'<'*|*'>'*) continue ;;   # anchors, absolute, schemes, templates
      esac
      target="${target%%#*}"       # drop in-file anchor
      [ -z "$target" ] && continue
      if [ ! -e "$dir/$target" ]; then
        if escapes_repo "$repo" "$dir/$target"; then
          printf 'OUTSIDE\t%s\t%s\t%s\n' "$label" "$md" "$target" >> "$findings"
        else
          printf 'DEAD\t%s\t%s\t%s\n' "$label" "$md" "$target" >> "$findings"
        fi
      fi
    done < <(strip_code_spans < "$md" | grep -oE '\]\([^)]+\)' 2>/dev/null)
  done < <(md_files "$repo")
  local n=0
  if [ -s "$findings" ]; then
    while IFS=$'\t' read -r kind lbl path tgt; do
      case "$kind" in
        OUTSIDE) skip "$lbl: cross-repo link escapes this checkout, not validated here: $path -> $tgt" ;;
        DEAD)    n=$((n + 1)); emitted "$repo" "$lbl: dead link: $path -> $tgt" ;;
      esac
    done < <(sort -u "$findings")
  fi
  rm -f "$findings"
  [ "$n" -eq 0 ] && pass "$label: all markdown links resolve"
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 2 — image pins: .env.example vs the live Ceiba host  (--host, infra)
#   This is the drift that shipped a wrong NAWAL_IMAGE for a week. Only the
#   *_IMAGE lines are compared — they are not secrets. Auto-skips when the
#   host is unreachable (CI) or --host was not given.
# ═══════════════════════════════════════════════════════════════════════════
check_host_pins() { # $1 = infra repo root
  local repo="$1" host_pins name target
  [ -f "$repo/.env.example" ] || { skip "host pins: no .env.example in $repo"; return; }
  if [ "$HOST_CHECK" -eq 0 ]; then
    skip "host pins: not requested (use --host=<ip> when the host is reachable)"
    return
  fi
  target="${HOST_TARGET}"
  if [ -z "$target" ]; then
    target="$(resolve_host_ip 2>/dev/null)" || {
      skip "host pins: no host target — pass --host=<ip> or set HOST_TARGET"; return; }
  fi
  host_pins="$(ssh -o ConnectTimeout=5 -o BatchMode=yes "wicked@${target}" \
    'grep -E "^[A-Z_]*_IMAGE=" /opt/belizechain/.env' 2>/dev/null)" || {
    skip "host pins: ${target} unreachable"; return; }
  local mismatches=0
  while IFS= read -r name; do
    local repo_val host_val
    repo_val="$(grep -E "^${name}=" "$repo/.env.example" | head -1)"
    host_val="$(printf '%s\n' "$host_pins" | grep -E "^${name}=" | head -1)"
    [ -z "$host_val" ] && { emitted "$repo" "host pins: $name missing on host"; mismatches=$((mismatches + 1)); continue; }
    if [ "$repo_val" != "$host_val" ]; then
      mismatches=$((mismatches + 1))
      emitted "$repo" "host pins: $name differs — repo: ${repo_val#*=} | host: ${host_val#*=}"
    fi
  done < <(grep -oE '^[A-Z_]*_IMAGE=' "$repo/.env.example" | tr -d '=')
  [ "$mismatches" -eq 0 ] && pass "host pins: all *_IMAGE pins match the live host .env"
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 3 — extrinsic calls from sibling repos vs the actual runtime
#   (workspace mode only). Module-aware: the call_module must exist in
#   construct_runtime!, and when it maps to a BelizeChain pallet the function
#   must be defined there. Framework pallets (System, Contracts, ...) are out
#   of scope — their call names belong to upstream Substrate, not this repo.
#   Caught: kinich's Consensus.submit_proof_of_* calls (nowhere defined).
# ═══════════════════════════════════════════════════════════════════════════
check_extrinsic_names() {
  local runtime="$WS/belizechain/runtime/src/lib.rs"
  [ -n "$WS" ] && [ -d "$WS/belizechain/pallets" ] && [ -f "$runtime" ] || {
    skip "extrinsic names: needs workspace mode with belizechain/runtime"; return; }

  # module name -> crate name, straight from construct_runtime!
  local modmap map_dir
  modmap="$(mktemp)"; map_dir="$(mktemp)"
  grep -oE '[A-Za-z][A-Za-z0-9]*: [a-z_0-9]+(::<[^>]*>|::[a-z_]+)? = [0-9]+' "$runtime" \
    | sed -E 's/: ([a-z_0-9]+)(::.*)? =.*/ \1/' > "$modmap"

  # crate name -> pallet dir (only BelizeChain's own pallets).
  # Cargo.toml names use dashes (pallet-belize-staking) while construct_runtime
  # and Rust module paths use underscores — normalize before matching.
  local d cname
  for d in "$WS/belizechain/pallets"/*/; do
    cname="$(grep -m1 -E '^name' "$d/Cargo.toml" 2>/dev/null | sed -E 's/.*"([^"]+)".*/\1/' | tr '-' '_')"
    [ -n "$cname" ] && printf '%s %s\n' "$cname" "${d%/}" >> "$map_dir"
  done

  # collect call sites: pair each call_module= with the next call_function=
  local sites bad=0 framework=0
  sites="$(mktemp)"

  # An array, not an unquoted $(find). awk needs the file list word-split, but an
  # unquoted expansion also splits on whitespace INSIDE a path, so a checkout under a
  # directory containing a space would silently drop files from the scan. Array elements
  # split only where find ends them. (shellcheck SC2046.)
  local -a py_files=()
  mapfile -t py_files < <(find "$WS/kinich-quantum" "$WS/nawal-ai" "$WS/pakit-storage" \
      -name '*.py' -not -path '*/.venv/*' -not -path '*/venv/*' \
      -not -path '*/site-packages/*' -not -path '*/tests/*' -not -name 'test_*' 2>/dev/null)

  # No sources means nothing to check. Guarded because awk with no file arguments reads
  # STDIN instead, and would block waiting for input that never arrives.
  if [ "${#py_files[@]}" -eq 0 ]; then
    rm -f "$modmap" "$map_dir" "$sites"
    skip "extrinsic names: no python sources found under the sibling repos"
    return
  fi

  awk '
    /call_module="[A-Za-z0-9_]+"/ {
      match($0, /call_module="[A-Za-z0-9_]+"/)
      m = substr($0, RSTART + 13, RLENGTH - 14)
    }
    /call_function="[a-z0-9_]+"/ {
      match($0, /call_function="[a-z0-9_]+"/)
      f = substr($0, RSTART + 15, RLENGTH - 16)
      if (m != "") print FILENAME ":" FNR " " m " " f
    }
  ' "${py_files[@]}" | sort -u > "$sites"

  local mod fname line crate_name dir fnlist owner_repo
  while read -r line mod fname; do
    # $line is "<path>:<lineno>" — resolve the owning repo so per-repo
    # .doc-claims-ignore files are consulted for the right repo.
    owner_repo="$WS/$(printf '%s' "$line" | sed -E "s|^$WS/([^/]+)/.*|\1|")"
    crate_name="$(awk -v m="$mod" '$1==m{print $2}' "$modmap")"
    if [ -z "$crate_name" ]; then
      bad=$((bad + 1))
      emitted "$owner_repo" "extrinsic names: module '$mod' (for '$fname' at $line) is not in the runtime"
      continue
    fi
    dir="$(awk -v c="$crate_name" '$1==c{print $2}' "$map_dir")"
    if [ -z "$dir" ]; then
      framework=$((framework + 1))   # upstream framework pallet — out of scope
      continue
    fi
    fnlist="$(grep -rhoE 'pub fn [a-z0-9_]+' --include='*.rs' "$dir" 2>/dev/null | awk '{print $3}' | sort -u)"
    if ! printf '%s\n' "$fnlist" | grep -qx "$fname"; then
      bad=$((bad + 1))
      emitted "$owner_repo" "extrinsic names: $mod.$fname (at $line) — '$fname' is not defined in the $mod pallet"
    fi
  done < "$sites"
  rm -f "$modmap" "$map_dir" "$sites"
  if [ "$bad" -eq 0 ]; then
    pass "extrinsic names: every custom-pallet call exists in the runtime ($framework framework call(s) skipped)"
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 4 — Node version alignment: .nvmrc vs Dockerfiles vs workflows
#   Any repo with an .nvmrc gets checked. Docs are excluded on purpose
#   (historical "Node 20" mentions are legitimate).
# ═══════════════════════════════════════════════════════════════════════════
check_node_alignment() { # $1 = repo root, $2 = repo label
  local repo="$1" label="$2" want bad=0
  [ -f "$repo/.nvmrc" ] || { skip "$label: no .nvmrc"; return; }
  want="$(tr -d 'v \n' < "$repo/.nvmrc" | cut -d. -f1)"
  while IFS=: read -r f ln _; do
    local ver
    ver="$(sed -n "${ln}p" "$f" | grep -oE 'node:?[0-9]+|node-version: .[0-9]+' | grep -oE '[0-9]+' | head -1)"
    [ -z "$ver" ] && continue
    [ "$ver" = "$want" ] && continue
    bad=$((bad + 1)); emitted "$repo" "$label: Node $ver at $f:$ln but .nvmrc=$want"
  done < <(grep -rnE 'FROM [^ ]*node:[0-9]+|node-version: .[0-9]+' \
             --include='Dockerfile*' --include='*.yml' "$repo" 2>/dev/null \
           | grep -vE '/node_modules/|/\.next/')
  [ "$bad" -eq 0 ] && pass "$label: Node $want aligned across Dockerfiles/workflows"
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 5 — every ${VAR} compose needs must be documented in .env.example
#   Only vars WITHOUT a :-default are checked: those are the ones a fresh
#   host deploy fails or silently mis-configures without.
# ═══════════════════════════════════════════════════════════════════════════
check_compose_env() { # $1 = repo root, $2 = repo label
  local repo="$1" label="$2" compose var file_bad
  for compose in "$repo/docker-compose.ceiba.yml" "$repo/docker-compose.yml"; do
    [ -f "$compose" ] || continue
    [ -f "$repo/.env.example" ] || { skip "$label: $(basename "$compose") has no .env.example"; continue; }
    file_bad=0
    while IFS= read -r var; do
      var="${var#\$\{}"; var="${var%%\}*}"
      case "$var" in *:-*|*:\?*|*:\+*) continue ;; esac     # has default → optional
      if ! grep -qE "^${var}=" "$repo/.env.example"; then
        file_bad=$((file_bad + 1))
        emitted "$repo" "$label: \${$var} in $(basename "$compose") is undocumented in .env.example"
      fi
    done < <(grep -oE '\$\{[A-Z0-9_]+(:-|:\?|:\+)?[^}]*\}' "$compose" | sort -u)
    [ "$file_bad" -eq 0 ] && pass "$label: $(basename "$compose") required env vars all documented"
  done
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK 6 — a `${VAR:?}` in compose must be supplied by the CI job that runs
#   `docker compose config`.
#   `config` interpolates EVERY file it is given and aborts on the FIRST
#   missing required variable, so adding one `${VAR:?}` to a compose file turns
#   the whole job red and NO file gets validated. It also hides any error in
#   the files it would have checked next.
#   This fired on 2026-10-12: 564f316 made AUTH_SESSION_SECRET required for the
#   UI services, the CI env block was not updated, and "Docker Compose
#   Validation" went red for a day — invisible because Actions was
#   billing-blocked at the time. A repo-local `.env` MASKS the problem, so it
#   reproduces only the way CI runs it.
# ═══════════════════════════════════════════════════════════════════════════

# Print "<job>|<file,file>|<KEY,KEY>" for every job in a workflow that invokes
# `docker compose -f <file>`. Only job-level `env:` keys are collected.
ci_compose_jobs() { # $1 = workflow file
  awk '
    function flush() {
      if (job != "" && nfiles > 0) {
        f = ""; for (i = 0; i < nfiles; i++) f = f (i ? "," : "") files[i]
        k = ""; for (x in keys) k = k (k ? "," : "") x
        print job "|" f "|" k
      }
      delete files; delete keys; nfiles = 0; job = ""; inenv = 0
    }
    BEGIN { injobs = 0 }
    /^jobs:[[:space:]]*$/            { injobs = 1; next }
    !injobs                          { next }
    /^[^[:space:]#]/                 { flush(); injobs = 0; next }
    /^  [A-Za-z0-9_-]+:[[:space:]]*$/ { flush(); job = $1; sub(/:$/, "", job); next }
    /^    env:[[:space:]]*$/         { inenv = 1; next }
    /^    [a-z]/                     { inenv = 0; next }
    inenv && /^      [A-Z][A-Z0-9_]*:/ { k = $1; sub(/:$/, "", k); keys[k] = 1; next }
    {
      if (match($0, /-f [A-Za-z0-9._-]+\.ya?ml/)) {
        files[nfiles++] = substr($0, RSTART + 3, RLENGTH - 3)
      }
    }
    END { flush() }
  ' "$1"
}

check_ci_compose_env() { # $1 = repo root, $2 = repo label
  local repo="$1" label="$2"
  local wf found=0 bad=0 job files keys f var
  local -a _files
  for wf in "$repo"/.github/workflows/*.yml; do
    [ -f "$wf" ] || continue
    grep -q 'docker compose' "$wf" 2>/dev/null || continue
    found=1
    while IFS='|' read -r job files keys; do
      [ -n "$job" ] || continue
      IFS=',' read -r -a _files <<< "$files"
      for f in "${_files[@]}"; do
        [ -n "$f" ] && [ -f "$repo/$f" ] || continue
        while IFS= read -r var; do
          var="${var#\$\{}"; var="${var%%:\?*}"
          case ",$keys," in *",$var,"*) continue ;; esac
          bad=$((bad + 1))
          emitted "$repo" "$label: \${$var:?} in $f is missing from the env: block of job '$job' in $(basename "$wf") - docker compose config will abort"
        done < <(grep -oE '\$\{[A-Z0-9_]+:\?[^}]*\}' "$repo/$f" | sort -u)
      done
    done < <(ci_compose_jobs "$wf")
  done
  if [ "$found" -eq 0 ]; then
    skip "$label: no workflow runs docker compose config"
  elif [ "$bad" -eq 0 ]; then
    pass "$label: compose required (VAR:?) vars all supplied by the CI job"
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# HONEST_RE — wording that admits a host is not live yet. A doc that says so is
#   telling the truth about the future, which is not the drift this check hunts.
# HONEST_WINDOW — lines of context around a hostname mention searched for that
#   wording. Deliberately narrow: an unrelated "coming soon" elsewhere in the
#   file must not excuse a bare link (file-wide matching is exactly how the old
#   `planned` allowlist masked real residue).
# ═══════════════════════════════════════════════════════════════════════════
readonly HONEST_RE='planned|not published|not resolve|does not exist|not live|NXDOMAIN|reserved|future|coming soon|never created|never existed'
readonly HONEST_WINDOW=10

host_is_flagged_honest() { # $1 = repo root, $2 = hostname
  local repo="$1" host="$2" file lineno first last

  md_files "$repo" | xargs -r grep -nE "$host" 2>/dev/null \
    | while IFS=: read -r file lineno _; do
        first=$(( lineno > HONEST_WINDOW ? lineno - HONEST_WINDOW : 1 ))
        last=$(( lineno + HONEST_WINDOW ))
        sed -n "${first},${last}p" "$file"
      done \
    | grep -qiE "$HONEST_RE"
}

# ═══════════════════════════════════════════════════════════════════════════
# CHECK — doc hostnames: every *.belizechain.org hostname a doc presents must
#   be live or explicitly marked as not yet published.
#
# WHY
#   The stack routes by PATH, not hostname (nginx `server_name _`). Only six
#   names exist: the apex, www, testnet, wallet, portal, explorer. Docs had
#   drifted into linking ~45 others (docs., forum., rpc., bns., faucet., scan.,
#   node., fl., gateway., ws., discord., ...) as if they worked, plus a few
#   belizechain.bz / .io hosts on domains the project does not own.
#
# WHAT PASSES
#   - a hostname that resolves today (verified 2026-10-08), or
#   - text near the link that says the host is planned / not published / coming
#     soon / reserved — an honest statement about the future is not a false claim.
#
#   Names the DNS plan reserves (infra/docs/DNS_AND_SUBDOMAIN_PLAN.md §3) are
#   NOT auto-approved: a bare link to one is still drift. An earlier version of
#   this check allowed them wholesale and masked real residue.
#
# WHAT FAILS
#   Anything else: invented names, typos, the SWEEP list (which is explicitly
#   "never create; fix the docs"), and any belizechain.bz / .io host.
#
#   A repo can still exempt a specific line via <repo>/.doc-claims-ignore.
# ═══════════════════════════════════════════════════════════════════════════
check_doc_hostnames() { # $1 = repo root, $2 = repo label
  local repo="$1" label="$2"

  # Resolve today (verified 2026-10-08). `api` is a CNAME to the apex, so it
  # serves the portal — it resolves, but is not an API endpoint; `send` is
  # email-only (MX). Both are listed so this check does not fire on them.
  local live=" belizechain.org www testnet wallet portal explorer api send "

  # Names the DNS plan reserves (P1/FUTURE, infra/docs/DNS_AND_SUBDOMAIN_PLAN.md §3)
  # are deliberately NOT auto-approved: a bare link to one is still drift and has
  # bitten us before. They pass only when the text near the link says the host is
  # planned or not-published — i.e. host_is_flagged_honest() below.

  local hosts bad=0 line host
  hosts="$(mktemp)"
  md_files "$repo" | xargs -r grep -hoE '[a-z0-9-]+\.belizechain\.(org|bz|io)' 2>/dev/null \
    | sort -u > "$hosts"

  [ -s "$hosts" ] || { rm -f "$hosts"; skip "$label: doc hostnames: no belizechain.org hosts found"; return; }

  while read -r host; do
    case "$host" in
      *.belizechain.bz|*.belizechain.io)
        bad=$((bad + 1))
        emitted "$repo" "$label: doc hostname '$host' — that domain is not the project's (use belizechain.org)"
        continue ;;
    esac
    local short="${host%.belizechain.org}"
    case "$live" in
      *" $short "*) continue ;;
    esac
    # Not live — but text near the link that says the host is planned or
    # not-published is honest.
    if host_is_flagged_honest "$repo" "$host"; then
      continue
    fi
    bad=$((bad + 1))
    emitted "$repo" "$label: doc hostname '$host' — not live and not marked as planned (see infra/docs/DNS_AND_SUBDOMAIN_PLAN.md §3)"
  done < "$hosts"
  rm -f "$hosts"

  if [ "$bad" -eq 0 ]; then
    pass "$label: every documented belizechain.org host is live or marked not-yet-published"
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# RUN
# ═══════════════════════════════════════════════════════════════════════════
label_for() { basename "$1"; }

section "Markdown links (all reachable repos)"
if [ -n "$WS" ]; then
  for r in belizechain ui infra kinich-quantum nawal-ai pakit-storage gem; do
    [ -d "$WS/$r" ] && check_dead_links "$WS/$r" "$r"
  done
else
  check_dead_links "$ROOT" "$(label_for "$ROOT")"
fi

section "Cross-repo references"
check_extrinsic_names

section "Documented hostnames"
if [ -n "$WS" ]; then
  for r in belizechain ui infra kinich-quantum nawal-ai pakit-storage gem belizechain.org; do
    [ -d "$WS/$r" ] && check_doc_hostnames "$WS/$r" "$r"
  done
else
  check_doc_hostnames "$ROOT" "$(label_for "$ROOT")"
fi

section "Version alignment"
if [ -n "$WS" ]; then
  for r in ui infra kinich-quantum nawal-ai pakit-storage gem belizechain; do
    [ -d "$WS/$r" ] && check_node_alignment "$WS/$r" "$r"
  done
else
  check_node_alignment "$ROOT" "$(label_for "$ROOT")"
fi

section "Compose ↔ .env.example coverage"
if [ -n "$WS" ] && [ -d "$WS/infra" ]; then
  check_compose_env "$WS/infra" "infra"
else
  check_compose_env "$ROOT" "$(label_for "$ROOT")"
fi

section "Compose required vars ↔ CI workflow env"
if [ -n "$WS" ] && [ -d "$WS/infra" ]; then
  check_ci_compose_env "$WS/infra" "infra"
else
  check_ci_compose_env "$ROOT" "$(label_for "$ROOT")"
fi

section "Image pins vs live Ceiba (optional)"
if [ -n "$WS" ] && [ -d "$WS/infra" ]; then
  check_host_pins "$WS/infra"
elif [ "$(label_for "$ROOT")" = "infra" ]; then
  check_host_pins "$ROOT"
else
  skip "host pins: not the infra repo"
fi

# ── summary ────────────────────────────────────────────────────────────────
printf '\n'
if [ "$FAILS" -eq 0 ]; then
  printf '%s✅ doc-claims check PASSED%s (%d skipped)\n' "$C_GRN" "$C_RST" "$SKIPS"
  exit 0
else
  printf '%s❌ doc-claims check FAILED: %d finding(s)%s (%d skipped)\n' "$C_RED" "$FAILS" "$C_RST" "$SKIPS"
  exit 1
fi
