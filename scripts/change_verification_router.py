#!/usr/bin/env python3
"""Map successful Cline file mutations to advisory repository verification commands.

This module is intentionally pure and unprivileged. It never executes commands, blocks
Cline tools, changes repository state, or grants execution authority. The candidate is
not wired into active hook configuration while it is being evaluated.
"""
from __future__ import annotations

import json
import sys
from collections.abc import Iterable, Mapping
from typing import Any

_MUTATING_TOOLS = {"apply_patch", "replace_in_file", "write_to_file"}


def _normalize(path: str) -> str:
    normalized = path.replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    return normalized


def verification_hints(paths: Iterable[str]) -> list[str]:
    """Return deterministic, non-mutating verification commands for changed paths."""
    normalized = {_normalize(path) for path in paths if isinstance(path, str) and path.strip()}
    commands: set[str] = set()

    if any(path.startswith("kotlin/android-command-center/") for path in normalized):
        commands.add(
            "cd kotlin && ./gradlew :android-command-center:ktlintCheck --no-daemon"
        )
    elif any(path.startswith("kotlin/") for path in normalized):
        commands.add("cd kotlin && ./gradlew ktlintCheck --no-daemon")

    if any(path.startswith(".github/workflows/") for path in normalized):
        commands.add("python scripts/check_harness_drift.py")

    return sorted(commands)


def hook_response(event: Mapping[str, Any]) -> dict[str, Any]:
    """Return an advisory Cline PostToolUse response without executing anything."""
    post = event.get("postToolUse", {}) if isinstance(event, Mapping) else {}
    if not isinstance(post, Mapping):
        post = {}

    tool = post.get("tool")
    success = post.get("success") is True
    parameters = post.get("parameters", {})
    if not isinstance(parameters, Mapping):
        parameters = {}

    path = parameters.get("path")
    if not success or tool not in _MUTATING_TOOLS or not isinstance(path, str):
        return {"cancel": False, "contextModification": "", "errorMessage": ""}

    commands = verification_hints([path])
    if not commands:
        return {"cancel": False, "contextModification": "", "errorMessage": ""}

    context = "Required advisory verification before push:\n" + "\n".join(
        f"- `{command}`" for command in commands
    )
    return {"cancel": False, "contextModification": context, "errorMessage": ""}


def main() -> int:
    raw = sys.stdin.read()
    try:
        event = json.loads(raw or "{}")
    except json.JSONDecodeError:
        event = {}
    if not isinstance(event, dict):
        event = {}
    print(json.dumps(hook_response(event), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
