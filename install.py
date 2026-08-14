#!/usr/bin/env python3
"""Install the Cline-native development fabric into a repository."""
from __future__ import annotations
import argparse, json, os, shutil, sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE = ROOT / ".cline"
REQUIRED_PACKAGE_FILES = (
    ".cline/config/capabilities.json",
    ".cline/config/permissions.json",
    ".cline/config/policies/default.yaml",
    ".cline/config/policies/production.yaml",
    ".cline/hooks/hooks.json",
    ".cline/plugins/project-fabric/plugin.json",
)


def validate_package() -> None:
    """Fail early when the checked-in fabric is incomplete or malformed."""
    missing = [path for path in REQUIRED_PACKAGE_FILES if not (ROOT / path).is_file()]
    if missing:
        raise ValueError("fabric package is missing: " + ", ".join(missing))
    manifests: dict[str, object] = {}
    for relative in (
        ".cline/config/capabilities.json",
        ".cline/config/permissions.json",
        ".cline/hooks/hooks.json",
        ".cline/plugins/project-fabric/plugin.json",
    ):
        path = ROOT / relative
        try:
            manifests[relative] = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise ValueError(f"invalid JSON in {path}: {exc}") from exc

    hooks = manifests[".cline/hooks/hooks.json"]
    if not isinstance(hooks, dict):
        raise ValueError("hook manifest must be a JSON object")
    missing_hooks = [
        name for name, script in hooks.items()
        if not isinstance(script, str) or not (ROOT / ".cline/hooks" / script).is_file()
    ]
    if missing_hooks:
        raise ValueError("hook manifest references missing scripts: " + ", ".join(missing_hooks))

    plugin = manifests[".cline/plugins/project-fabric/plugin.json"]
    if not isinstance(plugin, dict) or not isinstance(plugin.get("entry"), str):
        raise ValueError("project-fabric plugin manifest must declare a string entry")
    entry = ROOT / ".cline/plugins/project-fabric" / plugin["entry"]
    if not entry.is_file():
        raise ValueError(f"project-fabric plugin entry does not exist: {entry}")

def backup(path: Path) -> Path:
    target = path.with_name(path.name + ".backup-" + datetime.now().strftime("%Y%m%d-%H%M%S"))
    shutil.copy2(path, target)
    return target

def main() -> int:
    parser = argparse.ArgumentParser(description="Install the Cline Development Fabric")
    parser.add_argument("--project", default=".")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--install-official-skills", action="store_true")
    parser.add_argument("--install-ts-lsp", action="store_true")
    args = parser.parse_args()
    validate_package()
    project = Path(args.project).expanduser().resolve()
    if not project.is_dir():
        raise ValueError(f"project directory does not exist: {project}")
    sources = sorted(p for p in PACKAGE.rglob("*") if p.is_file() and "__pycache__" not in p.parts and p.suffix != ".pyc")
    written = skipped = 0
    for source in sources:
        destination = project / source.relative_to(ROOT)
        if destination.exists() and not args.force:
            print(f"SKIP  {destination}"); skipped += 1; continue
        if args.dry_run:
            print(f"WRITE {destination}"); written += 1; continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists(): print(f"BACKUP {backup(destination)}")
        shutil.copy2(source, destination)
        if source.suffix == ".py" and ("hooks" in source.parts or "plugins" in source.parts):
            destination.chmod(destination.stat().st_mode | 0o111)
        print(f"WRITE {destination}"); written += 1
    if args.install_official_skills:
        print("INFO  install official skills through Cline Customize/marketplace")
    if args.install_ts_lsp:
        print("INFO  install the TypeScript LSP plugin through Cline Customize/marketplace")
    print(f"Complete: {written} written, {skipped} skipped ({len(sources)} package files)")
    return 0

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)