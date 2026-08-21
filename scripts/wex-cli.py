#!/usr/bin/env python3
"""
wex-cli.py — Worktree Exchange Protocol CLI

Lightweight, stdlib-only coordination protocol for LLM CLI agents
operating in separate worktrees. Provides identity, heartbeat,
registry, handoff, and collision detection.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HEARTBEAT_INTERVAL = 300
WEX_VERSION = "1.0.0"
REGISTRY_DIR = ".worktrees/.registry"
WEX_DIR = ".wex"

def _get_worktree_root() -> Path:
    cwd = Path.cwd().resolve()
    for parent in [cwd] + list(cwd.parents):
        if (parent / ".git").is_dir() or (parent / ".git").is_file():
            return parent
        if (parent / ".worktrees").is_dir():
            return parent
    return cwd


def _get_git_worktree_id() -> str:
    """Detect worktree from CWD path relative to .worktrees/ dir."""
    cwd = Path.cwd().resolve()
    for i, part in enumerate(cwd.parts):
        if part == ".worktrees" and i + 1 < len(cwd.parts):
            return cwd.parts[i + 1]
    # Check git directory for worktree reference
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--git-dir"],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode == 0:
            git_dir = result.stdout.strip()
            if "/.worktrees/" in git_dir:
                return git_dir.split("/.worktrees/", 1)[1].split("/", 1)[0]
    except (subprocess.SubprocessError, FileNotFoundError):
        pass
    return "primary"


def _get_wex_path(root: Path, worktree_id: str) -> Path:
    if worktree_id == "primary":
        return root / WEX_DIR
    return root / ".worktrees" / worktree_id / WEX_DIR


def _get_registry_path(root: Path) -> Path:
    return root / REGISTRY_DIR


def _get_git_branch() -> str:
    try:
        r = subprocess.run(["git","rev-parse","--abbrev-ref","HEAD"],
                           capture_output=True,text=True,timeout=5)
        if r.returncode == 0:
            return r.stdout.strip()
    except: pass
    return "unknown"


def _get_git_revision() -> str:
    try:
        r = subprocess.run(["git","rev-parse","HEAD"],
                           capture_output=True,text=True,timeout=5)
        if r.returncode == 0:
            return r.stdout.strip()[:12]
    except: pass
    return "unknown"

def cmd_init(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    worktree_id = _get_git_worktree_id()
    wex_path = _get_wex_path(root, worktree_id)
    wex_path.mkdir(parents=True, exist_ok=True)

    identity = {
        "wex_version": WEX_VERSION,
        "worktree_id": worktree_id,
        "branch": _get_git_branch(),
        "revision": _get_git_revision(),
        "purpose": args.purpose or "development",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "last_updated": datetime.now(timezone.utc).isoformat(),
    }
    (wex_path / "identity.json").write_text(
        json.dumps(identity, indent=2), encoding="utf-8"
    )

    heartbeat = {
        "worktree_id": worktree_id,
        "last_seen": datetime.now(timezone.utc).isoformat(),
        "task": args.task or "",
        "status": "idle",
        "milestone": "",
        "revision": identity["revision"],
        "wex_version": WEX_VERSION,
    }
    (wex_path / "heartbeat.json").write_text(
        json.dumps(heartbeat, indent=2), encoding="utf-8"
    )

    (wex_path / "evidence-outbox").mkdir(parents=True, exist_ok=True)
    (wex_path / "handoff-inbox").mkdir(parents=True, exist_ok=True)

    registry_path = _get_registry_path(root)
    registry_path.mkdir(parents=True, exist_ok=True)
    (registry_path / "handoff-queue").mkdir(parents=True, exist_ok=True)
    (registry_path / "evidence-exchange").mkdir(parents=True, exist_ok=True)

    print(f"WEx initialized for worktree '{worktree_id}' at {wex_path}")
    return 0


def cmd_heartbeat(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    worktree_id = _get_git_worktree_id()
    wex_path = _get_wex_path(root, worktree_id)
    identity_path = wex_path / "identity.json"

    if not identity_path.exists():
        print("Error: WEx not initialized. Run 'wex-cli.py init' first.", file=sys.stderr)
        return 1

    try:
        identity = json.loads(identity_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as exc:
        print(f"Error reading identity: {exc}", file=sys.stderr)
        return 1

    heartbeat = {
        "worktree_id": worktree_id,
        "last_seen": datetime.now(timezone.utc).isoformat(),
        "task": args.task or identity.get("purpose", ""),
        "status": args.status or "active",
        "milestone": args.milestone or "",
        "revision": _get_git_revision(),
        "wex_version": WEX_VERSION,
    }
    (wex_path / "heartbeat.json").write_text(
        json.dumps(heartbeat, indent=2), encoding="utf-8"
    )

    registry_path = _get_registry_path(root)
    registry_path.mkdir(parents=True, exist_ok=True)
    manifest_path = registry_path / "manifest.json"
    if manifest_path.exists():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            manifest = {"worktrees": {}}
    else:
        manifest = {"worktrees": {}}

    manifest["worktrees"][worktree_id] = {
        "identity": identity,
        "heartbeat": heartbeat,
    }
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    print(f"Heartbeat: {worktree_id} — {heartbeat['status']} (task: {heartbeat['task']})")
    return 0

def cmd_list(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    registry_path = _get_registry_path(root)
    manifest_path = registry_path / "manifest.json"

    if not manifest_path.exists():
        print("No worktrees registered in this repository.")
        return 0

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        print("Corrupt registry manifest.", file=sys.stderr)
        return 1

    worktrees = manifest.get("worktrees", {})
    if not worktrees:
        print("No worktrees registered.")
        return 0

    now = datetime.now(timezone.utc)
    print(f"{'WORKTREE':25} {'STATUS':10} {'TASK':30} {'LAST SEEN':25} {'BRANCH':20}")
    print("-" * 110)
    for wid, info in worktrees.items():
        hb = info.get("heartbeat", {})
        status = hb.get("status", "unknown")
        task = (hb.get("task", "") or "")[:28]
        last_seen = hb.get("last_seen", "")
        branch = info.get("identity", {}).get("branch", "?")
        if last_seen:
            try:
                seen = datetime.fromisoformat(last_seen)
                if (now - seen).total_seconds() > HEARTBEAT_INTERVAL * 3:
                    status = "stale"
            except: pass
        print(f"{wid:25} {status:10} {task:30} {last_seen:25} {branch:20}")
    return 0


def cmd_reserve(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    worktree_id = _get_git_worktree_id()
    path = os.path.abspath(args.path)
    repo_root = str(root)
    if not path.startswith(repo_root):
        print(f"Error: Path must be inside repository: {repo_root}", file=sys.stderr)
        return 1

    rel_path = os.path.relpath(path, repo_root)
    registry_path = _get_registry_path(root)
    reservations_path = registry_path / "reservations.json"

    if reservations_path.exists():
        try:
            reservations = json.loads(reservations_path.read_text(encoding="utf-8"))
        except: reservations = {}
    else:
        reservations = {}

    for other_wid, reserved_paths in reservations.items():
        if other_wid == worktree_id: continue
        for rp in reserved_paths:
            if rel_path == rp or rel_path.startswith(rp + "/") or rp.startswith(rel_path + "/"):
                print(f"BLOCKED: '{rel_path}' reserved by worktree '{other_wid}' (path: '{rp}')", file=sys.stderr)
                return 1

    if worktree_id not in reservations:
        reservations[worktree_id] = []
    reservations[worktree_id].append(rel_path)
    reservations_path.write_text(json.dumps(reservations, indent=2), encoding="utf-8")
    print(f"Reserved: {rel_path} for worktree {worktree_id}")
    return 0


def cmd_check_collisions(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    registry_path = _get_registry_path(root)
    reservations_path = registry_path / "reservations.json"
    if not reservations_path.exists():
        print("No reservations found."); return 0
    try:
        reservations = json.loads(reservations_path.read_text(encoding="utf-8"))
    except: print("Corrupt reservations.", file=sys.stderr); return 1

    collisions = []
    for wid_a, paths_a in reservations.items():
        for wid_b, paths_b in reservations.items():
            if wid_a >= wid_b: continue
            for pa in paths_a:
                for pb in paths_b:
                    if pa == pb or pa.startswith(pb + "/") or pb.startswith(pa + "/"):
                        collisions.append((wid_a, pa, wid_b, pb))
    if collisions:
        print(f"{len(collisions)} collision(s) detected:")
        for a, pa, b, pb in collisions:
            print(f"  {a}:{pa}  <->  {b}:{pb}")
        return 1
    print("No collisions detected.")
    return 0

def cmd_push_evidence(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    worktree_id = _get_git_worktree_id()
    wex_path = _get_wex_path(root, worktree_id)
    evidence_file = Path(args.file)
    if not evidence_file.exists():
        print(f"Error: Evidence file not found: {evidence_file}", file=sys.stderr)
        return 1
    try:
        content = json.loads(evidence_file.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as exc:
        print(f"Error reading evidence: {exc}", file=sys.stderr)
        return 1
    content["_wex_source"] = worktree_id
    content["_wex_pushed_at"] = datetime.now(timezone.utc).isoformat()
    outbox_path = wex_path / "evidence-outbox" / evidence_file.name
    outbox_path.write_text(json.dumps(content, indent=2), encoding="utf-8")
    exchange_dir = _get_registry_path(root) / "evidence-exchange"
    exchange_dir.mkdir(parents=True, exist_ok=True)
    dest = exchange_dir / f"{worktree_id}--{evidence_file.name}"
    dest.write_text(json.dumps(content, indent=2), encoding="utf-8")
    print(f"Evidence pushed: {evidence_file.name} (→ registry)")
    return 0


def cmd_pull_evidence(args: argparse.Namespace) -> int:
    root = _get_worktree_root()
    exchange_dir = _get_registry_path(root) / "evidence-exchange"
    if not exchange_dir.is_dir():
        print("No evidence in registry."); return 0
    files = sorted(exchange_dir.glob("*.json"))
    if args.source:
        files = [f for f in files if f.name.startswith(args.source)]
    if not files:
        print(f"No evidence found{ ' from ' + args.source if args.source else ''}.")
        return 0
    for f in files:
        try:
            content = json.loads(f.read_text(encoding="utf-8"))
        except: continue
        eid = content.get("evidence_id", content.get("id", f.stem))
        result = content.get("result", "?")
        src = content.get("_wex_source", f.name.split("--")[0] if "--" in f.name else "?")
        print(f"  {eid:30} {result:10} from {src:20} [{f.name}]")
    return 0


def cmd_dashboard(args: argparse.Namespace) -> int:
    rv = cmd_list(args)
    if rv != 0: return rv
    print()
    print("=== Collision Check ===")
    cmd_check_collisions(args)
    print()
    print("=== Evidence Registry ===")
    exchange_dir = _get_registry_path(_get_worktree_root()) / "evidence-exchange"
    if exchange_dir.is_dir():
        files = list(exchange_dir.glob("*.json"))
        print(f"  {len(files)} evidence records")
    else:
        print("  No evidence registry found")
    print()
    print(f"  Protocol: {WEX_VERSION}")
    print(f"  Heartbeat interval: {HEARTBEAT_INTERVAL}s ({HEARTBEAT_INTERVAL//60}min)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Worktree Exchange Protocol CLI")
    sub = parser.add_subparsers(dest="command", help="sub-command")
    p_init = sub.add_parser("init", help="Initialize WEx in current worktree")
    p_init.add_argument("--purpose", "-p", default="development")
    p_init.add_argument("--task", "-t", default="")
    p_hb = sub.add_parser("heartbeat", help="Emit a heartbeat")
    p_hb.add_argument("--task", "-t", default="")
    p_hb.add_argument("--status", "-s", default="active",
                      choices=["active","idle","blocked","completed"])
    p_hb.add_argument("--milestone", "-m", default="")
    sub.add_parser("list", help="List all registered worktrees")
    p_res = sub.add_parser("reserve", help="Advisory hard-blocking path reservation")
    p_res.add_argument("path", help="File or directory path")
    sub.add_parser("check-collisions", help="Check overlapping reservations")
    p_pe = sub.add_parser("push-evidence", help="Push evidence to registry")
    p_pe.add_argument("file", help="Evidence JSON file")
    p_ple = sub.add_parser("pull-evidence", help="Pull evidence from registry")
    p_ple.add_argument("--source", "-s", default="", help="Filter by source worktree ID")
    sub.add_parser("dashboard", help="Show worktree dashboard")
    args = parser.parse_args()
    if not args.command:
        parser.print_help(); return 0
    cmd_map = {
        "init": cmd_init, "heartbeat": cmd_heartbeat, "list": cmd_list,
        "reserve": cmd_reserve, "check-collisions": cmd_check_collisions,
        "push-evidence": cmd_push_evidence, "pull-evidence": cmd_pull_evidence,
        "dashboard": cmd_dashboard,
    }
    return cmd_map[args.command](args)


if __name__ == "__main__":
    raise SystemExit(main())
