#!/usr/bin/env python3

import json
import re
import sys
from typing import Any


HIGH_IMPACT_PATTERNS = [
    r"\bsudo\b",
    r"\bsystemctl\b",
    r"\bservice\b",
    r"\bdocker\s+compose\s+(up|down|restart|stop|rm)\b",
    r"\bdocker\s+(restart|stop|rm|run)\b",
    r"\bshutdown\b",
    r"\breboot\b",
    r"\bpoweroff\b",
    r"\bmkfs\b",
    r"\bmount\b",
    r"\bumount\b",
    r"\brm\s+-rf\b",
    r"\bmv\b.+/data/chain",
    r"\btar\b.+/data/chain",
    r"\bssh\b.+(100\.81\.45\.25|10\.0\.0\.222|ceiba)",
]

RISK_MESSAGE = (
    "High-impact BelizeChain ops action detected. Before proceeding, state the target host or repo, "
    "blast radius, rollback path, and the preflight checks you will use to verify safety."
)


def _load_payload() -> dict[str, Any]:
    raw = sys.stdin.read().strip()
    if not raw:
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return {}


def _extract_tool_name(payload: dict[str, Any]) -> str:
    for key in ("tool_name", "toolName", "tool"):
        value = payload.get(key)
        if isinstance(value, str):
            return value
    tool_input = payload.get("toolInput")
    if isinstance(tool_input, dict):
        name = tool_input.get("tool_name") or tool_input.get("toolName")
        if isinstance(name, str):
            return name
    return ""


def _extract_command(payload: dict[str, Any]) -> str:
    direct_candidates = []
    for key in ("command", "input", "arguments"):
        value = payload.get(key)
        if isinstance(value, str):
            direct_candidates.append(value)

    for container_key in ("toolInput", "toolArguments", "parameters"):
        container = payload.get(container_key)
        if isinstance(container, dict):
            for key in ("command", "input", "args", "explanation", "goal"):
                value = container.get(key)
                if isinstance(value, str):
                    direct_candidates.append(value)
                elif isinstance(value, list):
                    direct_candidates.extend(item for item in value if isinstance(item, str))

    return "\n".join(direct_candidates)


def _is_high_impact(command_text: str) -> bool:
    lowered = command_text.lower()
    return any(re.search(pattern, lowered) for pattern in HIGH_IMPACT_PATTERNS)


def _allow() -> None:
    json.dump(
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow"
            }
        },
        sys.stdout,
    )


def _ask(reason: str) -> None:
    json.dump(
        {
            "systemMessage": reason,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "ask",
                "permissionDecisionReason": reason,
            },
        },
        sys.stdout,
    )


def main() -> int:
    payload = _load_payload()
    tool_name = _extract_tool_name(payload)
    command_text = _extract_command(payload)

    if tool_name and tool_name not in {"run_in_terminal", "execute"}:
        _allow()
        return 0

    if command_text and _is_high_impact(command_text):
        _ask(RISK_MESSAGE)
        return 0

    _allow()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())