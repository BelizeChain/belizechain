#!/usr/bin/env bash
# Shared host resolution for the BelizeChain deploy scripts.
#
# Why this file exists
# --------------------
# These scripts used to default to a literal `100.81.45.25`. That was correct only
# while Ceiba was the single host. After the 2026-10-06 migration to ceiba the same
# literal silently pointed ceiba's health timer and chain backup at the OLD machine,
# and a check that watches the wrong host reports success while the host it exists to
# watch is dead. The literal is gone.
#
# Source of truth
# ---------------
# `TAILSCALE_IP` in the compose `.env`, because that is the value docker-compose binds
# every admin port to (`${TAILSCALE_IP}:3003:3000` and friends). Reading it means the
# scripts and the port bindings cannot drift apart, and the fallback below mirrors
# compose's own `${TAILSCALE_IP:-127.0.0.1}` default.
#
# `tailscale ip -4` is deliberately NOT used. It answers "what is this machine's
# address", not "which machine am I supposed to check" - run from a workstation it
# returns the WORKSTATION's address, so a bare `health-check.sh` would quietly check
# the wrong host. That is the exact failure this file exists to remove.
#
# There is deliberately no built-in fallback address either. A loud failure is better
# than a silent check of the wrong machine.
#
# Usage:
#   . "$(dirname "$0")/lib/host-env.sh"
#   HOST_IP="$(resolve_host_ip)" || { echo "cannot resolve host" >&2; exit 1; }

# resolve_host_ip: print the host's Tailscale IPv4 on stdout. Return 1 with a reason
# on stderr when it cannot be determined.
resolve_host_ip() {
    if [[ -n "${TAILSCALE_IP:-}" ]]; then
        printf '%s\n' "${TAILSCALE_IP}"
        return 0
    fi

    local env_file="${COMPOSE_DIR:-/opt/belizechain}/.env"
    if [[ -r "${env_file}" ]]; then
        local from_env
        from_env="$(sed -n 's/^TAILSCALE_IP=//p' "${env_file}" | head -n 1)"
        from_env="${from_env%%#*}"           # drop a trailing comment
        from_env="${from_env//[[:space:]]/}" # drop all whitespace
        from_env="${from_env//\"/}"          # drop double quotes
        from_env="${from_env//\'/}"          # drop single quotes
        # An empty value mirrors compose: `${TAILSCALE_IP:-127.0.0.1}` binds the
        # admin ports to loopback, so loopback is the address worth checking.
        printf '%s\n' "${from_env:-127.0.0.1}"
        return 0
    fi

    echo "resolve_host_ip: no host address found." >&2
    echo "  Set TAILSCALE_IP, or add TAILSCALE_IP=<ip> to ${env_file}." >&2
    echo "  Off-host (e.g. from a workstation) set CEIBA_HOST=<tailscale-ip> instead." >&2
    return 1
}

# resolve_target_ip <ssh-target> <runner-fn>: print the Tailscale address of a REMOTE
# host, by asking that host.
#
# Needed by the checks whose curls run ON the target (via `tailscale ssh`), because the
# node binds RPC to its Tailscale address rather than loopback - so the address must be
# the target's, and 127.0.0.1 will not do. resolve_host_ip() is the wrong tool here: run
# from a workstation it answers with THIS machine's address.
#
#   TARGET_IP="$(resolve_target_ip "${SSH_TARGET}" run_remote)" || true
#
# <runner-fn> is called as `"$runner" '<command>'`, so it must accept a command string -
# e.g. run_remote() { tailscale ssh "$SSH_TARGET" "$1" < /dev/null; }. An explicit
# $TARGET_IP always wins, so a caller can override without editing this file.
resolve_target_ip() {
    if [[ -n "${TARGET_IP:-}" ]]; then
        printf '%s\n' "${TARGET_IP}"
        return 0
    fi

    local runner="$2" ip
    ip="$("$runner" 'tailscale ip -4 2>/dev/null | head -1' 2>/dev/null || true)"
    ip="${ip//[[:space:]]/}"
    if [[ -z "${ip}" ]]; then
        echo "resolve_target_ip: cannot determine ${1}'s Tailscale address." >&2
        echo "  Set TARGET_IP=<ip>, or check that ${1} is reachable over the tailnet." >&2
        return 1
    fi
    printf '%s\n' "${ip}"
}
