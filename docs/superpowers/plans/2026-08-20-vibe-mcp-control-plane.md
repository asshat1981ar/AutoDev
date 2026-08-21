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

## Task 1 — Durable ExecPlan projection

- [x] Add RED acceptance test for objective → observable typed ExecPlan.
- [x] Verify RED: CI run 519 failed because `AppState::exec_plan` did not exist.
- [x] Add `exec_plans` state, `exec_plan()` read accessor, `PlanBudget::new(3, 3)`, and initial `objective` milestone.
- [x] Verify GREEN: CI run 520 passed Rust format, Clippy, build, and tests.

## Task 2 — Vibe observation and proposal tools

- [x] Add RED tests for project status, typed plan lookup, non-authoritative verification, inert test proposals, and non-mutating replan proposals.
- [x] Verify RED: CI run 521 failed on the absent Vibe handlers/types after formatting passed.
- [x] Implement `autodev.project.status`.
- [x] Implement `autodev.execplan.get` with unknown-plan fail-closed behavior.
- [x] Implement `autodev.verification.status` with `verified=false`, `authority=none`, and no self-verification.
- [x] Implement `autodev.test.propose` as bounded inert intent; no process spawning.
- [x] Implement `autodev.replan.propose` without calling `ExecPlan::replan` or consuming budget.
- [x] Preserve existing `autodev.action.propose`, `autodev.objectives.list`, and `autodev.gaps.scan` semantics.
- [x] Verify GREEN: CI run 522 passed Rust format, Clippy, build, and tests.

## Task 3 — Transport hardening

- [x] Add RED integration test requiring an untrusted browser Origin to receive HTTP 403.
- [x] Verify RED: CI run 523 observed HTTP 406 from MCP negotiation instead of a pre-dispatch 403.
- [x] Add Origin-host validation against the configured MCP host policy while allowing non-browser requests with no Origin header.
- [x] Verify Origin GREEN: CI run 524 passed Rust format, Clippy, build, tests, container build, harness, Kotlin, and Python jobs.
- [x] Add RED test requiring localhost as the default bind and explicit bind override support.
- [x] Verify bind RED: CI run 525 passed formatting and failed Clippy specifically because `resolve_bind_addr` was absent.
- [x] Implement `resolve_bind_addr`, `AUTODEV_BIND_ADDR`, and default `127.0.0.1` binding.
- [ ] Record bind GREEN from CI run 526 after its Rust test/container steps finish.

## Task 4 — Vibe integration documentation and final verification

- [x] Add `docs/integrations/vibe-mcp.md` with exact server start and `vibe mcp add autodev ...` commands.
- [x] Document that `autodev` is the required positional `NAME` that fixes `vibe mcp add: error: the following arguments are required: NAME`.
- [x] Document bearer, bind, Origin, and authority behavior.
- [x] Review changed code for direct shell/filesystem execution, `AuthorizationGrant` creation, replan mutation, or MCP self-verification; none are part of the implemented handlers.
- [ ] Require the final exact-head CI run after this evidence update to pass before marking the slice complete.

## Progress

- Branch: `feat/vibe-mcp-control-plane`.
- Draft PR: #39, `feat(mcp): add Vibe development control plane`.
- TDD evidence recorded through RED runs 519/521/523/525 and GREEN runs 520/522/524.
- Functional implementation and Vibe usage documentation are committed.
- Remaining gate: exact-head CI after the living-plan reconciliation.

## Surprises & discoveries

- AutoDev already had the correct architectural anchor: `rmcp` Streamable HTTP plus fail-closed bearer auth. A parallel Python/Node MCP server would have duplicated protocol and weakened the single authority boundary.
- The existing server bound `0.0.0.0`, which was broader than needed for local Vibe use; localhost is now the default with an explicit deployment override.
- Invalid browser Origins previously reached MCP content negotiation and returned 406; a dedicated pre-dispatch Origin policy now returns 403.
- GitHub Actions served as the executable RED/GREEN verifier because this session does not have a local clone capable of running Cargo against the repository.

## Decision log

- Extend `autodev-server`; reject parallel MCP runtimes.
- Keep all new mutation-shaped tools proposal-only.
- Store durable ExecPlan coordination state in `AppState`, but do not automatically start, complete, authorize, or verify plans.
- Reuse `AUTODEV_MCP_ALLOWED_HOSTS` as the hostname policy for present browser Origin headers so Host/Origin deployment configuration cannot drift independently in this slice.
- Put Vibe commands in `docs/integrations/vibe-mcp.md` instead of duplicating canonical repository build-command blocks in README.

## Outcomes & retrospective

The implementation now provides a Vibe-facing development control plane with typed durable plan observation, explicit verification non-authority, inert test/replan proposals, bearer authentication, Origin rejection, and localhost-by-default binding. The design maintained ForgeCore as the sole trusted execution boundary. Completion is intentionally not claimed until the final exact-head CI run reports all required checks passing.