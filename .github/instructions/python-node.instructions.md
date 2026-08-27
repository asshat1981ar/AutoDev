---
applyTo: "scripts/**,tests/**,install.py,bootstrap_cline_mcp.py,services/**,package.json,pnpm-lock.yaml,pnpm-workspace.yaml"
---

# Python + Node (fabric and tooling) instructions

This guidance applies to `scripts/**`, `tests/**`, `install.py`, `bootstrap_cline_mcp.py`, the ADR-006 pnpm workspace, and `services/livekit-agent/**`.

## Stack

- Python 3.10/3.11 (CI matrix), stdlib `unittest` + `py_compile`, no `pyproject.toml`/`requirements.txt`
- `scripts/autodev-cli.py`: dependency-free (urllib only), read-only observer — no ForgeCore/Git/MCP write authority
- `scripts/termux-kanban.mjs`: Node 24, stdlib builtins only (`node:crypto`, `node:fs`, `node:path`, `node:child_process`), pinned PTY `1.1.2` SHA-256 `660a30…ae8ec8a`
- Cline fabric: `.cline/**` + `.cline/hooks/*.py` + `.cline/plugins/project-fabric/tools.py`
- LiveKit service: Node 24, pnpm 11.24.0, TypeScript, exact `@livekit/agents`/LemonSlice peers; untrusted objective-intake edge only

## Commands (run from repo root)

```bash
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
python -m unittest discover -s tests -v
node --check scripts/termux-kanban.mjs
node scripts/termux-kanban.mjs --check
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
```

## Rules

- Do not add dependencies to `scripts/autodev-cli.py`. If you need a new stdlib module, note it in `AGENTS.md` and `scripts/check_harness_drift.py`.
- Fabric validates strictly: `install.py` checks `.cline/config/capabilities.json`, `permissions.json`, `policies/*.yaml`, `hooks.json`, and `plugins/project-fabric/plugin.json`. Keep those manifests and their referenced files in sync.
- Root `package.json`, `pnpm-lock.yaml`, and `pnpm-workspace.yaml` are limited to ADR-006. Do not add unrelated packages or use npm/yarn at root. Do not create root `pyproject.toml`/`requirements.txt`.
- `services/livekit-agent` may enqueue bounded objectives only. It must not import ForgeCore, construct grants, execute shell/filesystem/Git/deployment actions, or hold LiveKit administrative tools.
- `termux-kanban.mjs` must keep `--check` (diagnostic dry-run) and `--repair-only` modes. PTY replacement is pinned + SHA-256 verified; do not unpin.
- Tests are `tests/test_*.py` with `unittest`. Keep `tests/__pycache__` gitignored.

## Verification

Every change to Python/fabric/launcher must pass `py_compile` + `unittest` + `node --check` + launcher probe. LiveKit service changes must also pass `pnpm install --frozen-lockfile`, `pnpm typecheck`, and `pnpm test`. Run `python scripts/check_harness_drift.py` for stale-command detection.
