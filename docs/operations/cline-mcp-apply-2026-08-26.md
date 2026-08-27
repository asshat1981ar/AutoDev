# Cline MCP — Apply & Restore Record

**Date:** 2026-08-26
**Action:** Applied a minimal, reduced-risk MCP capability stack to Cline target config.

## What was created
`/root/.cline/data/settings/cline_mcp_settings.json` (new file; did not exist before).
All entries are remote `streamableHttp`, `disabled: true`, `autoApprove: []`:
- `github/github-mcp-server` -> https://api.githubcopilot.com/mcp/
- `context7/context7` -> https://mcp.context7.com/mcp
- `deepwiki/deepwiki-mcp` -> https://mcp.deepwiki.com/mcp
- `huggingface/mcp-hub-server` -> https://huggingface.co/mcp/

Existing Cline data (`global-settings.json`, `providers.json`) was preserved intact.

## Backups (timestamped, verified)
- **Whole settings dir snapshot (pre-change):**
  `/root/.backup/cline-settings-20260826-184851`
  - `global-settings.json`, `providers.json` — verified identical to live / intact.
- **Defective first-generation of the new file** (contained a malformed huggingface URL; superseded):
  `/root/.backup/cline_mcp_settings-defective-20260826-185017.json`

## Restore / Rollback
The correct pre-change state for the new file is **absence** (it did not exist). To fully restore prior state:

```
# A) Revert the whole settings dir to the pre-change snapshot
rm -rf /root/.cline/data/settings
cp -a /root/.backup/cline-settings-20260826-184851 /root/.cline/data/settings

# B) Or remove just the generated new file (restores "absent" state)
rm -f /root/.cline/data/settings/cline_mcp_settings.json

# C) Keep the defective first-generation file only as investigation evidence.
# Do not restore it as an operational config because it contained a malformed URL.
```

After option A or B, confirm the file is absent. Parse JSON only when intentionally restoring a known-good config:

```bash
test ! -e /root/.cline/data/settings/cline_mcp_settings.json
python3 -c "import json, pathlib; p=pathlib.Path('/root/.cline/data/settings/cline_mcp_settings.json'); print('absent' if not p.exists() else json.load(open(p)))"
```

## Verification performed
- JSON parses; `mcpServers` is an object with 4 entries.
- All 4 added servers are `disabled: true`, `autoApprove: []`.
- All remote URLs are clean, valid `https://` URLs (no annotation leakage).
- No plaintext secret/API-key material in generated config.
- Sibling Cline settings files unchanged.
- Whole-dir backup integrity verified (global-settings identical; providers intact).

## Team additions — stateless MCP servers (2026-08-26, team-led)
Added 4 stateless, self-hosted stdio servers (all `disabled: true`, `autoApprove: []`), via
`cline_mcp_audit_patch.py --include-grade-b --apply` (backup `...backup-20260826-192728`):
- `modelcontextprotocol/server-time` -> `npx -y @modelcontextprotocol/server-time`
- `modelcontextprotocol/server-sequential-thinking` -> `npx -y @modelcontextprotocol/server-sequential-thinking`
- `modelcontextprotocol/server-fetch` -> `uvx mcp-server-fetch`
- `modelcontextprotocol/server-everything` -> `npx -y @modelcontextprotocol/server-everything`

Full whole-dir backup taken before team apply:
`/root/.backup/cline-settings-pre-team-20260826-192719`

The config now holds 8 server entries (4 remote + 4 stdio), all disabled, all `autoApprove: []`.
See memory pattern `autodev-mcp-team-stateless-008` for the selection rationale.

## Runtime probe verification (non-dry-run continuation, 2026-08-26)
MCP `initialize` + `tools/list` probes executed against each self-hosted stdio entry
(JSON-RPC over stdin; first-run package fetch included):

| Entry | Result | Evidence |
|---|---|---|
| `server-time` (npx) | **BROKEN — npm 404** | `@modelcontextprotocol/server-time` does not exist on npm |
| `mcp-server-time` (uvx) | **PASS** | mcp-time v1.29.1, proto `2024-11-05`, tools: `get_current_time`, `convert_time` |
| `server-sequential-thinking` (npx) | **PASS** | v0.2.0, proto `2024-11-05`, tool: `sequentialthinking` |
| `server-fetch` (uvx) | **PASS** | mcp-fetch v1.29.1, proto `2024-11-05`, tool: `fetch` |
| `server-everything` (npx) | **PASS** | v2.0.0, proto `2024-11-05`, 13 tools |

**Remediation applied:** the broken npx-based `time` entry was replaced via the runner with a
`uvx mcp-server-time` entry (backups: `cline_mcp_settings-pre-timefix-*` / `...backup-20260826-195602`).
Probe output confirmed protocolVersion `2024-11-05` on every working server — consistent with the
honest `LEGACY_COMPATIBLE` classification and the standing rule that nothing is enabled without
runtime verification. All entries therefore remain `disabled: true` pending an explicit enablement decision.

## LiveKit ops MCP experiment — rejected and removed (2026-08-27)

The first-party LiveKit administrative MCP experiment was removed after security review. Its room,
participant, token, and dispatch tools performed network effects directly from untrusted MCP calls
using ambient administrator credentials. An environment flag was not a typed, per-action
`AuthorizationGrant`, so the design bypassed AutoDev's ForgeCore authority boundary.

The supported replacement is `services/livekit-agent`: an untrusted LiveKit Agents JS realtime
intake worker that can enqueue a bounded objective for one operator-configured repository. It has no
LiveKit administrative MCP tools and no ForgeCore execution authority. See
`docs/adr/ADR-006-livekit-realtime-intake.md` and
`docs/operations/livekit-agent-deployment.md`.

Any stale `autodev/livekit-ops` entry in the external Cline settings must remain disabled and should
be removed by the operator. No credentials should be added to that entry.

Cleanup completed on 2026-08-27: the stale entry was removed from
`/root/.cline/data/settings/cline_mcp_settings.json` after creating timestamped backup
`cline_mcp_settings.json.backup-livekit-ops-removed-20260827-013750` in the same directory.