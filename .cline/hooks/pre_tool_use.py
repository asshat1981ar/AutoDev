#!/usr/bin/env python3
"""PreToolUse guard: reject a narrowly defined destructive command set."""
from __future__ import annotations
import json, re, sys

raw = sys.stdin.read()
try: event = json.loads(raw)
except json.JSONDecodeError: event = {"tool_input": {"command": raw}}
if not isinstance(event, dict): event = {"tool_input": {"command": raw}}
tool_input = event.get("tool_input", {})
if not isinstance(tool_input, dict): tool_input = {}
command = str(tool_input.get("command", event.get("command", "")))
blocked = re.search(
    # `git reset --hard` (any leading flags) | `git clean -f*` |
    # `git push -f`, `git push --force`, and argv forms split as `git push -- -f`
    # | `rm -rf`, `rm -fr`, `rm -r -f`, `rm --recursive --force`.
    r"\bgit\s+reset\s+--hard\b"
    r"|\bgit\s+clean\s+-[a-zA-Z]*f"
    r"|\bgit\s+push\b[^\n]*\s(?:-f\b|--force(?!-with-lease)\b)"
    r"|\brm\s+(?:-[a-zA-Z]*[rf][a-zA-Z]*|-[a-zA-Z]*r[a-zA-Z]*f|-[a-zA-Z]*f[a-zA-Z]*r|--recursive\b|--force\b)",
    command,
)
if blocked:
    print(json.dumps({"cancel": True, "reason": "Destructive command requires explicit approval."}))
    raise SystemExit(2)
print(json.dumps({"cancel": False}))