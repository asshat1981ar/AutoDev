# Vibe MCP Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make AutoDev's existing Rust MCP adapter directly usable from Mistral Vibe for durable development observation and proposal workflows without widening execution authority.

**Architecture:** Extend `crates/autodev-server` only. Streamable HTTP remains canonical; ForgeCore typed state backs durable plan projections; MCP mutation-shaped operations remain proposals. Local transport defaults to `127.0.0.1`, present browser Origins are validated, and bearer authentication remains fail-closed.

**Spec:** `docs/superpowers/specs/2026-08-20-vibe-mcp-control-plane-design.md`

## Global constraints

- ForgeCore remains the sole trusted execution authority.
- No MCP handler mints `AuthorizationGrant`, executes a process, directly mutates the workspace, widens capabilities, or marks work verified.
- Canonical endpoint is `/mcp` over Streamable HTTP.
- Default bind is `127.0.0.1`; only explicit `AUTODEV_BIND_ADDR` widens it.
- `max_replans = 3`; `max_attempts_per_milestone = 3`; initial milestone ID is `objective`.
- No new root Cargo, Python, or Node manifest.

## TDD evidence

- [x] Run 519 RED — missing `AppState::exec_plan`.
- [x] Run 520 GREEN — durable ExecPlan projection passed Rust format, Clippy, build, and tests.
- [x] Run 521 RED — missing Vibe observation/proposal handlers and schemas.
- [x] Run 522 GREEN — Vibe tool tests passed Rust format, Clippy, build, and tests.
- [x] Run 523 RED — untrusted Origin reached MCP negotiation and returned 406 rather than required 403.
- [x] Run 524 GREEN — Origin hardening passed Rust format, Clippy, build, tests, container, harness, Kotlin, and Python jobs.
- [x] Run 525 RED — missing `resolve_bind_addr` after formatting passed.
- [ ] Run 526 — bind implementation verification was still executing when final documentation reconciliation began; final exact-head CI supersedes it as the completion gate.

## Implemented slice

- [x] Durable `ExecPlan` projection per accepted objective with `PlanBudget::new(3, 3)` and initial `objective` milestone.
- [x] Read-only `AppState::exec_plan()` accessor.
- [x] `autodev.project.status`.
- [x] `autodev.execplan.get`, failing closed for unknown IDs.
- [x] `autodev.verification.status`, explicitly non-authoritative and never self-verifying.
- [x] `autodev.test.propose`, bounded and non-executing.
- [x] `autodev.replan.propose`, non-mutating and non-budget-consuming.
- [x] Existing `autodev.action.propose`, `autodev.objectives.list`, and `autodev.gaps.scan` preserved.
- [x] Present browser Origin hostname validation before MCP dispatch; requests without Origin remain eligible for normal bearer/MCP processing.
- [x] Local bind defaults to `127.0.0.1`; explicit `AUTODEV_BIND_ADDR` supports deployment widening.
- [x] Vibe integration guide at `docs/integrations/vibe-mcp.md`, including required positional `NAME` and bearer configuration.

## Progress

- Branch: `feat/vibe-mcp-control-plane`.
- Draft PR: #39, `feat(mcp): add Vibe development control plane`.
- Functional code, adversarial tests, security hardening, design/spec, and usage guide are committed.
- Remaining gate: a fresh exact-head CI run after this final plan reconciliation.

## Surprises & discoveries

- AutoDev already had the correct anchor: `rmcp` Streamable HTTP plus fail-closed bearer auth. A parallel MCP runtime would have duplicated protocol and weakened the single authority boundary.
- The server originally bound `0.0.0.0`; localhost is safer for local Vibe use and remains overrideable for explicit deployments.
- Invalid browser Origins previously reached MCP content negotiation; the dedicated guard now rejects them before dispatch.
- GitHub Actions served as the executable RED/GREEN verifier because this session did not have a local clone capable of running Cargo against the repository.

## Decision log

- Extend `autodev-server`; do not introduce a parallel Python, Node, or second Rust MCP server.
- Keep every new mutation-shaped MCP operation proposal-only.
- Store durable planning coordination state without starting, completing, authorizing, or verifying plans automatically.
- Reuse `AUTODEV_MCP_ALLOWED_HOSTS` for browser Origin host policy so Host and Origin deployment configuration cannot drift independently in this slice.
- Keep Vibe setup commands in a dedicated integration guide rather than duplicating canonical build-command blocks in README.

## Outcomes & retrospective

The code implements the approved Vibe control-plane design while maintaining ForgeCore as the sole trusted execution authority. No completion claim is recorded here until the final exact-head CI run reports the required checks passing.