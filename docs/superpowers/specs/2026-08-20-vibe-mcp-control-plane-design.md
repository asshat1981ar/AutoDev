# Vibe MCP Control Plane Design

## Goal

Extend AutoDev's existing Rust `rmcp` adapter so Mistral Vibe can inspect durable development state and submit typed development proposals over authenticated Streamable HTTP without gaining direct execution authority.

## Architectural decision

Use `crates/autodev-server` as the sole Vibe-facing MCP server. Do not add a parallel Python, Node, or second Rust MCP runtime. Vibe connects to `/mcp`; handlers expose read-only projections and proposal-only operations. ForgeCore remains the only trusted execution authority.

## Transport and security

- Canonical transport: MCP Streamable HTTP at `/mcp`.
- Default local bind: `127.0.0.1`; `AUTODEV_BIND_ADDR` may explicitly widen it.
- Existing bearer authentication remains mandatory and fail-closed through `AUTODEV_MCP_BEARER_TOKEN`.
- Existing Host allowlisting remains enabled.
- Origin validation must reject invalid browser Origins with `403` while allowing requests without `Origin` from non-browser MCP clients.
- Body limit remains 512 KiB.
- Vibe uses static bearer-token authentication; no OAuth dependency is introduced.

## Initial Vibe tool slice

### Read-only tools

1. `autodev.project.status`
   - Returns server/runtime projection: queued objective count, tracked ExecPlan count, MCP authority=`none`, and trusted execution boundary=`forge_core`.
   - Does not inspect or mutate the filesystem.

2. `autodev.execplan.get`
   - Input: `plan_id`.
   - Returns the serialized typed ForgeCore `ExecPlan` tracked by the server.
   - Unknown IDs return MCP invalid-params errors.

3. `autodev.verification.status`
   - Input: optional `plan_id`.
   - Returns an explicit non-authoritative verification projection.
   - Must state that MCP cannot mark work verified and that repository/VerificationFabric evidence remains required.

### Proposal-only tools

4. Existing `autodev.action.propose`
   - Preserved unchanged in authority semantics.

5. `autodev.test.propose`
   - Input: `task_id`, `agent_id`, `reason`, and a bounded verification command string.
   - Returns a typed proposal document only; it never starts a process.

6. `autodev.replan.propose`
   - Input: `plan_id`, `agent_id`, `reason`, and `proposed_goal`.
   - Returns a proposal only; it must not call `ExecPlan::replan`, increment replan budgets, alter lifecycle state, or persist mutations.

## Durable ExecPlan projection

When an objective is accepted, the control plane creates a typed `forge_core::ExecPlan` projection tied to the objective ID. The initial plan has a finite budget and one milestone representing the objective. This state is coordination state only. Creating or reading it does not authorize any effect.

The initial defaults are deliberately small and explicit:

- `max_replans = 3`
- `max_attempts_per_milestone = 3`
- initial milestone ID: `objective`

## Vibe registration

Expected registration shape:

```bash
vibe mcp add autodev \
  --transport streamable-http \
  --url http://127.0.0.1:8080/mcp \
  --api-key-env AUTODEV_MCP_BEARER_TOKEN \
  --api-key-header Authorization \
  --api-key-format "Bearer {token}" \
  --no-login \
  --startup-timeout-sec 10 \
  --tool-timeout-sec 120
```

`NAME` is the required positional alias; `autodev` is the recommended value.

## Error handling

- Blank IDs/reasons/commands/goals: MCP invalid-params.
- Unknown plan: MCP invalid-params.
- Invalid Origin: HTTP 403 before MCP dispatch.
- Missing configured bearer token: HTTP 503.
- Missing/wrong bearer token: HTTP 401.
- Proposal tools never convert validation success into authorization.

## Testing

TDD order:

1. Add integration/unit tests that fail because the new tools, plan projection, localhost bind helper, or Origin protection are absent.
2. Confirm GitHub Actions fails for the intended missing behavior.
3. Add the minimum implementation.
4. Require Rust fmt, clippy, build, tests, and repository harness drift checks to pass at exact head.

Security/adversarial assertions must prove:

- proposal tools do not mutate ExecPlan state;
- test proposal does not execute a process;
- replan proposal does not consume replan budget;
- invalid Origins fail closed;
- unknown plans fail closed;
- bearer behavior remains unchanged.

## Non-goals

- No direct MCP shell/filesystem executor.
- No `AuthorizationGrant` creation from MCP.
- No MCP self-verification.
- No OAuth implementation.
- No stdio transport in this slice.
- No new root manifest or package ecosystem.

## Rollback

The change is isolated to the server adapter, tests, and documentation. Rollback removes the new tool handlers, ExecPlan projection, bind/origin configuration, and Vibe docs while leaving ForgeCore untouched.