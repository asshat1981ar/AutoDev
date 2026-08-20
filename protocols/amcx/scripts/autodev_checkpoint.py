#!/usr/bin/env python3
import argparse, json, os, re, sys
from datetime import datetime, timezone
from pathlib import Path

SECRET_RE = re.compile(r"(?i)(api[_-]?key|secret|password|token|bearer\s+[A-Za-z0-9_.-]{20,}|ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,})")

def reject_unsafe(value):
    if SECRET_RE.search(value or ""):
        raise ValueError("checkpoint value resembles secret-bearing data")

def validate_worktree(value):
    p = Path(value)
    if p.is_absolute():
        raise ValueError("worktree_path must be repository-relative")
    parts = p.parts
    if len(parts) < 2 or parts[0] != ".worktrees" or ".." in parts:
        raise ValueError("worktree_path must be under .worktrees/<task-id>")

def atomic_write(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)

def init(args):
    for v in (args.task_id,args.plan,args.revision,args.branch,args.worktree,args.next_action):
        reject_unsafe(v)
    validate_worktree(args.worktree)
    data = {
        "schema_version": 1,
        "task_id": args.task_id,
        "plan_path": args.plan,
        "repository_revision": args.revision,
        "branch": args.branch,
        "worktree_path": args.worktree,
        "completed_tasks": [],
        "current_task": None,
        "verification": [],
        "effect_receipts": [],
        "rulings": [],
        "unresolved": [],
        "next_action": args.next_action,
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }
    atomic_write(args.output, data)

def main():
    ap=argparse.ArgumentParser()
    sub=ap.add_subparsers(dest="command", required=True)
    p=sub.add_parser("init")
    p.add_argument("--output", required=True)
    p.add_argument("--task-id", required=True)
    p.add_argument("--plan", required=True)
    p.add_argument("--revision", required=True)
    p.add_argument("--branch", required=True)
    p.add_argument("--worktree", required=True)
    p.add_argument("--next-action", required=True)
    args=ap.parse_args()
    try:
        init(args)
    except ValueError as e:
        print(str(e), file=sys.stderr)
        return 2
    return 0

if __name__=="__main__":
    raise SystemExit(main())
