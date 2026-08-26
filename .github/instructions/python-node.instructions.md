---
applyTo: "scripts/**,tests/**,install.py,bootstrap_cline_mcp.py"
---

# Python + Node (fabric and tooling) instructions

This guidance applies to `scripts/**`, `tests/**`, `install.py`, and `bootstrap_cline_mcp.py`.

## Stack

- Python 3.10/3.11 (CI matrix), stdlib `unittest` + `py_compile`, no `pyproject.toml`/`requirements.txt`
- `scripts/autodev-cli.py`: dependency-free (urllib only), read-only observer — no ForgeCore/Git/MCP write authority
- `scripts/termux-kanban.mjs`: Node 24, stdlib builtins only (`node:crypto`, `node:fs`, `node:path`, `node:child_process`), pinned PTY `1.1.2` SHA-256 `660a30…ae8ec8a`
- Cline fabric: `.cline/**` + `.cline/hooks/*.py` + `.cline/plugins/project-fabric/tools.py`

## Commands (run from repo root)

```bash
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
python -m unittest discover -s tests -v
node --check scripts/termux-kanban.mjs
node scripts/termux-kanban.mjs --check
```

## Rules

- Do not add dependencies to `scripts/autodev-cli.py`. If you need a new stdlib module, note it in `AGENTS.md` and `scripts/check_harness_drift.py`.
- Fabric validates strictly: `install.py` checks `.cline/config/capabilities.json`, `permissions.json`, `policies/*.yaml`, `hooks.json`, and `plugins/project-fabric/plugin.json`. Keep those manifests and their referenced files in sync.
- Do not create `package.json`/`pyproject.toml`/`requirements.txt` at root. There is no `web/command-center/package.json` — that directory is static.
- `termux-kanban.mjs` must keep `--check` (diagnostic dry-run) and `--repair-only` modes. PTY replacement is pinned + SHA-256 verified; do not unpin.
- Tests are `tests/test_*.py` with `unittest`. Keep `tests/__pycache__` gitignored.

## Verification

Every change to Python/fabric/launcher must pass `py_compile` + `unittest` + `node --check` + `node --check` launcher probe. Run `python scripts/check_harness_drift.py` for stale-command detection.
