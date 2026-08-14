#!/usr/bin/env python3
"""Merge a curated MCP stack into Cline's configuration.

Existing server entries are preserved unless --replace-existing is supplied.
The default target is the Cline CLI configuration; use --dry-run to inspect
the result without writing anything.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import sys
from datetime import datetime
from pathlib import Path
from typing import Any


def command_exists(name: str) -> bool:
    return shutil.which(name) is not None


def cline_config_path(target: str, editor: str) -> Path:
    home = Path.home()
    if target == "cli":
        return home / ".cline" / "data" / "settings" / "cline_mcp_settings.json"
    dirs = {"code": "Code", "cursor": "Cursor", "windsurf": "Windsurf"}
    editor_dir = dirs[editor]
    system = platform.system().lower()
    if system == "windows":
        appdata = os.environ.get("APPDATA")
        if not appdata:
            raise RuntimeError("APPDATA is not set")
        root = Path(appdata)
    elif system == "darwin":
        root = home / "Library" / "Application Support"
    else:
        root = home / ".config"
    return root / editor_dir / "User" / "globalStorage" / "saoudrizwan.claude-dev" / "settings" / "cline_mcp_settings.json"


def local_npx(package: str, *args: str, disabled: bool = False) -> dict[str, Any]:
    if platform.system().lower() == "windows":
        command, command_args = "cmd", ["/c", "npx", "-y", package, *args]
    else:
        command, command_args = "npx", ["-y", package, *args]
    return {"type": "stdio", "command": command, "args": command_args,
            "disabled": disabled or not command_exists("npx"), "autoApprove": []}


def local_uvx(package: str, *args: str, disabled: bool = False, env: dict[str, str] | None = None) -> dict[str, Any]:
    entry: dict[str, Any] = {"type": "stdio", "command": "uvx", "args": [package, *args],
                             "disabled": disabled or not command_exists("uvx"), "autoApprove": []}
    if env:
        entry["env"] = env
    return entry


def remote(url: str, *, disabled: bool = False, headers: dict[str, str] | None = None) -> dict[str, Any]:
    entry: dict[str, Any] = {"type": "streamableHttp", "url": url, "disabled": disabled, "autoApprove": []}
    if headers:
        entry["headers"] = headers
    return entry


def build_servers(project: Path) -> dict[str, dict[str, Any]]:
    project = project.resolve()
    neo4j_ready = all(os.environ.get(k) for k in ("NEO4J_URI", "NEO4J_USERNAME", "NEO4J_PASSWORD"))
    qdrant_ready, redis_ready = bool(os.environ.get("QDRANT_URL")), bool(os.environ.get("REDIS_URL"))
    aws_ready = bool(os.environ.get("AWS_PROFILE") or (os.environ.get("AWS_ACCESS_KEY_ID") and os.environ.get("AWS_SECRET_ACCESS_KEY")))
    return {
        "github": remote("https://api.githubcopilot.com/mcp/"),
        "filesystem": local_npx("@modelcontextprotocol/server-filesystem", str(project)),
        "git": local_uvx("mcp-server-git", "--repository", str(project)),
        "context7": remote("https://mcp.context7.com/mcp"),
        "playwright": local_npx("@playwright/mcp@latest"),
        "memory": local_npx("@modelcontextprotocol/server-memory"),
        "sequential-thinking": local_npx("@modelcontextprotocol/server-sequential-thinking"),
        "qdrant-memory": local_uvx("mcp-server-qdrant", disabled=not qdrant_ready, env={
            "QDRANT_URL": os.environ.get("QDRANT_URL", "http://localhost:6333"),
            "QDRANT_API_KEY": os.environ.get("QDRANT_API_KEY", ""),
            "COLLECTION_NAME": os.environ.get("QDRANT_COLLECTION", "cline-development-memory")}),
        "neo4j": {"type": "stdio", "command": "python", "args": ["-m", "neo4j_mcp_server"], "env": {
            "NEO4J_URI": os.environ.get("NEO4J_URI", "bolt://localhost:7687"), "NEO4J_USERNAME": os.environ.get("NEO4J_USERNAME", "neo4j"),
            "NEO4J_PASSWORD": os.environ.get("NEO4J_PASSWORD", ""), "NEO4J_DATABASE": os.environ.get("NEO4J_DATABASE", "neo4j"),
            "NEO4J_READ_ONLY": os.environ.get("NEO4J_READ_ONLY", "true"), "NEO4J_TELEMETRY": "false"}, "disabled": not neo4j_ready, "autoApprove": []},
        "redis": local_uvx("--from", "git+https://github.com/redis/mcp-redis.git", "redis-mcp-server", "--url", os.environ.get("REDIS_URL", "redis://localhost:6379/0"), disabled=not redis_ready),
        "docker-mcp-gateway": {"type": "stdio", "command": "docker", "args": ["mcp", "gateway", "run", "--profile", "dev-tools"], "disabled": not command_exists("docker"), "autoApprove": []},
        "supabase": remote("https://mcp.supabase.com/mcp?read_only=true"),
        "sentry": remote("https://mcp.sentry.dev/mcp"),
        "terraform": {"type": "stdio", "command": "docker", "args": ["run", "-i", "--rm", "hashicorp/terraform-mcp-server:latest"], "disabled": not command_exists("docker"), "autoApprove": []},
        "cloudflare": remote("https://mcp.cloudflare.com/mcp"),
        "aws-core": local_uvx("awslabs.core-mcp-server@latest", disabled=not aws_ready, env={"FASTMCP_LOG_LEVEL": "ERROR", **({"AWS_PROFILE": os.environ["AWS_PROFILE"]} if os.environ.get("AWS_PROFILE") else {})}),
        "fetch": local_uvx("mcp-server-fetch"),
        "notion": remote("https://mcp.notion.com/mcp"),
        "linear": remote("https://mcp.linear.app/mcp"),
        "mcp-docs": remote("https://modelcontextprotocol.io/mcp"),
    }


def load_existing(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"mcpServers": {}}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Existing config is invalid JSON: {path}\n{exc}") from exc
    if not isinstance(data, dict):
        raise RuntimeError(f"Existing config root must be a JSON object: {path}")
    if not isinstance(data.setdefault("mcpServers", {}), dict):
        raise RuntimeError("'mcpServers' in existing config must be a JSON object")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description="Install/merge a curated 20-server MCP stack into Cline.")
    parser.add_argument("--target", choices=["cli", "vscode"], default="cli")
    parser.add_argument("--editor", choices=["code", "cursor", "windsurf"], default="code")
    parser.add_argument("--project", default=".")
    parser.add_argument("--config")
    parser.add_argument("--replace-existing", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    project = Path(args.project).expanduser()
    if not project.exists():
        raise RuntimeError(f"Project path does not exist: {project}")
    config = Path(args.config).expanduser() if args.config else cline_config_path(args.target, args.editor)
    existing = load_existing(config)
    servers = build_servers(project)
    current = existing["mcpServers"]
    for name, entry in servers.items():
        if args.replace_existing or name not in current:
            current[name] = entry
    rendered = json.dumps(existing, indent=2) + "\n"
    print(f"Cline config: {config}\nProject root: {project.resolve()}")
    if args.dry_run:
        print(rendered)
        return 0
    config.parent.mkdir(parents=True, exist_ok=True)
    if config.exists():
        backup = config.with_suffix(config.suffix + f".backup-{datetime.now():%Y%m%d-%H%M%S}")
        shutil.copy2(config, backup)
        print(f"Backup: {backup}")
    config.write_text(rendered, encoding="utf-8")
    print(f"Wrote: {config}\nConfigured servers: {len(servers)}")
    disabled = [n for n, c in servers.items() if c.get("disabled") is True]
    if disabled:
        print("Installed but disabled: " + ", ".join(disabled))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)