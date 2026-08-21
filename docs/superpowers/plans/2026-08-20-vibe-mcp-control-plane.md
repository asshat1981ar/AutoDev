# Vibe MCP Control Plane Implementation Plan

**Goal:** Make AutoDev's existing Rust MCP adapter directly usable from Mistral Vibe for durable development observation and proposal workflows without widening execution authority.

**Architecture:** Extend `crates/autodev-server` only. Streamable HTTP remains canonical; ForgeCore typed state backs durable plan projections; mutation-shaped MCP operations remain proposals. Local transport defaults to `127.0.0.1`, present browser Origins are validated, and bearer authentication remains fail-closed.

**Spec:** `docs/superpowers/specs/2026-08-20-vibe-mcp-control-plane-design.md`

## Constraints

- ForgeCore remains the sole trusted execution authority.
- MCP never mints `AuthorizationGrant`, executes a process, directly mutates the workspace, widens capabilities, or marks work verified.
- Endpoint: `/mcp` over Streamable HTTP.
- Default bind: `127.0.0.1`; explicit `AUTODEV_BIND_ADDR` may widen it.
- ExecPlan defaults: `max_replans=3`, `max_attempts_per_milestone=3`, milestone ID `objective`.

## TDD evidence

- [x] Run 519 RED — missing `AppState::exec_plan`.
- [x] Run 520 GREEN — durable ExecPlan projection passed Rust format, Clippy, build, and tests.
- [x] Run 521 RED — missing Vibe observation/proposal handlers and schemas.
- [x] Run 522 GREEN — Vibe tool tests passed Rust format, Clippy, build, and tests.
- [x] Run 523 RED — untrusted Origin reached MCP negotiation and returned 406 rather than required 403.
- [x] Run 524 GREEN — Origin hardening passed Rust format, Clippy, build, tests, container, harness, Kotlin, and Python jobs.
- [x] Run 525 RED — missing `resolve_bind_addr` after formatting passed.
- [ ] Final exact-head CI — this is the sole remaining completion gate.

## Implemented slice

- [x] Durable ExecPlan projection for accepted objectives.
- [x] `autodev.project.status`.
- [x] `autodev.execplan.get` with unknown-ID rejection.
- [x] `autodev.verification.status` with explicit non-authority.
- [x] `autodev.test.propose` as bounded non-executing intent.
- [x] `autodev.replan.propose` without plan mutation or budget consumption.
- [x] Existing action/objective/gap tools preserved.
- [x] Untrusted present browser Origins rejected before MCP dispatch.
- [x] Server defaults to localhost with explicit bind override.
- [x] `docs/integrations/vibe-mcp.md` contains the exact Vibe registration and troubleshooting workflow.

## Progress

- Branch: `feat/vibe-mcp-control-plane`.
- Draft PR: #39.
- Code, tests, security hardening, design, and integration documentation are committed.
- No completion claim until fresh exact-head CI passes.

## Surprises & discoveries

- The existing Rust `rmcp` server was the correct integration anchor; a parallel MCP runtime was unnecessary.
- The preexisting `0.0.0.0` default was wider than needed for local Vibe usage.
- Origin rejection needed a dedicated pre-dispatch policy because invalid Origins previously reached content negotiation.

## Decision log

- One MCP runtime and one ForgeCore authority boundary.
- All new mutation-shaped tools remain proposal-only.
- Host and browser-Origin hostname policy share `AUTODEV_MCP_ALLOWED_HOSTS` in this slice.
- Vibe setup is documented in a dedicated integration guide to avoid duplicating canonical build-command blocks.

## Outcomes & retrospective

The implementation matches the approved architecture and preserves ForgeCore authority. Completion remains gated on the fresh exact-head CI run for this commit.