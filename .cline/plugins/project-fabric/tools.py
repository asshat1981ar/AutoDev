#!/usr/bin/env python3
"""Dependency-free local tools for the Cline project plugin."""
from __future__ import annotations
import json, subprocess, sys
from pathlib import Path

def snapshot(root: Path) -> dict:
    files = [str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and ".git" not in p.parts]
    return {"root": str(root), "files": sorted(files)[:500], "file_count": len(files)}

def main() -> None:
    request = json.loads(sys.stdin.read() or "{}")
    root = Path(request.get("root", ".")).resolve()
    if request.get("tool") == "project_snapshot":
        print(json.dumps(snapshot(root)))
    elif request.get("tool") == "quality_gate_plan":
        print(json.dumps({"plan": ["format", "typecheck/build", "focused tests", "security review"], "executed": False}))
    else:
        raise SystemExit("unknown tool")

if __name__ == "__main__": main()