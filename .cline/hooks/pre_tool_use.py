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
blocked = re.search(r"\bgit\s+reset\s+--hard\b|\bgit\s+clean\s+-[a-zA-Z]*f|\bgit\s+push\b[^\n]*\s--force\b|\brm\s+-rf\b", command)
if blocked:
    print(json.dumps({"cancel": True, "reason": "Destructive command requires explicit approval."}))
    raise SystemExit(2)
print(json.dumps({"cancel": False}))