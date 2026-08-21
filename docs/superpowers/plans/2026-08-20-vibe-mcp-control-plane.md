# Vibe MCP Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make AutoDev's existing Rust MCP adapter directly usable from Mistral Vibe for durable development observation and proposal workflows without widening execution authority.

**Architecture:** Extend `crates/autodev-server` only. Streamable HTTP remains the canonical transport; ForgeCore typed state backs durable plan projections, while MCP mutation-shaped operations remain proposals. Add localhost-by-default binding and Origin validation around the existing Host allowlist and bearer authentication.

**Tech Stack:** Rust stable, Axum 0.7, rmcp 3.1.2 Streamable HTTP server, ForgeCore ExecPlan types, Tokio, serde/serde_json, GitHub Actions CI.

**Spec:** `docs/superpowers/specs/2026-08-20-vibe-mcp-control-plane-design.md`

## Global Constraints

- ForgeCore remains the sole trusted execution authority.
- No MCP handler may mint `AuthorizationGrant`, execute a process, directly mutate the workspace, or mark work verified.
- Canonical MCP endpoint is `/mcp` over Streamable HTTP.
- Default local bind is `127.0.0.1`; `AUTODEV_BIND_ADDR` is the only widening mechanism in this slice.
- Bearer authentication remains fail-closed.
- `max_replans = 3`; `max_attempts_per_milestone = 3`; initial milestone ID is `objective`.
- No new root Cargo, Python, or Node manifest.
- Rust verification runs from `crates/`.

---

### Task 1: Red tests for Vibe tool contract and durable plan projection

**Files:**
- Create: `crates/autodev-server/tests/vibe_mcp.rs`

**Interfaces:**
- Consumes: public `autodev_server::{router, AppState, ObjectiveRequest}` and HTTP `/mcp`.
- Produces: executable acceptance tests for plan creation, MCP initialization/tool listing, unknown-plan rejection, proposal-only replan behavior, and bearer compatibility.

- [ ] **Step 1: Write failing tests**

Create integration tests that POST an objective, assert a durable plan projection exists through the new public read accessor, initialize MCP with bearer auth, list tools, and assert the six-tool slice includes `autodev.project.status`, `autodev.execplan.get`, `autodev.verification.status`, `autodev.action.propose`, `autodev.test.propose`, and `autodev.replan.propose`.

Add a test that calls `autodev.replan.propose`, reads the plan before and after, and proves `replans_used()` is unchanged.

- [ ] **Step 2: Push tests and verify RED**

Expected CI failure: missing public plan accessor/new MCP tools or equivalent compile/assertion failure caused by the absent feature, not unrelated infrastructure.

- [ ] **Step 3: Record RED evidence**

Record exact failing workflow/job and head SHA before implementation.

---

### Task 2: Implement durable plan projection and Vibe MCP tools

**Files:**
- Modify: `crates/autodev-server/src/lib.rs`
- Modify: `crates/autodev-server/src/mcp.rs`

**Interfaces:**
- Consumes: `forge_core::{ExecPlan, PlanBudget, PlanMilestone}`.
- Produces: `AppState::exec_plan(plan_id) -> Option<ExecPlan>` plus five new/retained tool behaviors in the spec.

- [ ] **Step 1: Create typed plan state on objective intake**

Add `exec_plans: Arc<RwLock<BTreeMap<String, ExecPlan>>>` to `AppState`. On successful objective intake, create `ExecPlan::new(id.clone(), description, PlanBudget::new(3, 3))`, add `PlanMilestone::new("objective", "Complete objective")`, and store it under the objective ID. Do not start or complete the plan automatically.

- [ ] **Step 2: Add read accessor**

Add a public async accessor returning a cloned `ExecPlan` by ID solely for observation/testing.

- [ ] **Step 3: Implement MCP inputs and handlers**

Add schema-validated inputs for `plan_id`, test proposals, and replan proposals. Implement:

- `autodev.project.status`: JSON projection with counts and authority markers.
- `autodev.execplan.get`: serialized typed plan; invalid-params on unknown ID.
- `autodev.verification.status`: explicit non-authoritative status and required verification boundary.
- `autodev.test.propose`: JSON proposal with `execution_authorized=false`; never spawn.
- `autodev.replan.propose`: JSON proposal with current plan budget projection and `execution_authorized=false`; never mutate the plan.

Keep `autodev.action.propose` unchanged except for refactors required by compilation.

- [ ] **Step 4: Verify GREEN for targeted Rust tests**

Run in CI/local environment:

```bash
cd crates
cargo test -p autodev-server
```

Expected: PASS.

---

### Task 3: Red tests then implement transport hardening

**Files:**
- Modify: `crates/autodev-server/src/lib.rs`
- Modify: `crates/autodev-server/src/main.rs`
- Test: `crates/autodev-server/tests/vibe_mcp.rs`

**Interfaces:**
- Produces: localhost default bind resolution and Origin rejection before MCP dispatch.

- [ ] **Step 1: Add failing Origin/bind tests**

Add tests proving an unapproved Origin receives `403`, requests without Origin remain eligible for bearer authentication, and bind resolution defaults to `127.0.0.1` while accepting an explicit `AUTODEV_BIND_ADDR` value through a pure resolver function.

- [ ] **Step 2: Verify RED**

Expected: test failure because Origin enforcement/bind resolver is absent.

- [ ] **Step 3: Implement minimal hardening**

Add MCP middleware validating present `Origin` hostnames against the same configured host policy used for DNS-rebinding protection, returning `403` on mismatch. Add a pure `resolve_bind_addr(Option<&str>) -> &str` or owned equivalent and use `AUTODEV_BIND_ADDR`, default `127.0.0.1`, in `main.rs`.

- [ ] **Step 4: Verify targeted GREEN**

```bash
cd crates
cargo test -p autodev-server
```

Expected: PASS.

---

### Task 4: Vibe usage documentation and full repository verification

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-08-20-vibe-mcp-control-plane.md`

**Interfaces:**
- Produces: copy/paste `vibe mcp add autodev ...` registration instructions and durable progress/evidence record.

- [ ] **Step 1: Document exact Vibe command**

Add:

```bash
export AUTODEV_MCP_BEARER_TOKEN='replace-me'
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

State that `autodev` is the required positional `NAME` argument.

- [ ] **Step 2: Run full Rust and harness gates**

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
cd ..
python scripts/check_harness_drift.py
```

Expected: all PASS at exact branch head.

- [ ] **Step 3: Review diff for authority regressions**

Reject any code path that executes shell/filesystem effects, creates an `AuthorizationGrant`, mutates replan budget from `autodev.replan.propose`, or lets MCP self-declare verification.

- [ ] **Step 4: Update living plan evidence**

Append Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective with exact commit/CI evidence. Do not claim completion without passing checks.

---

## Progress

- Design approved by user.
- Spec committed on `feat/vibe-mcp-control-plane`.
- Implementation plan committed on the same branch.
- Next observable milestone: failing Vibe MCP contract tests on CI.

## Surprises & Discoveries

- Current AutoDev already uses `rmcp` Streamable HTTP and bearer authentication, so no new MCP runtime is needed.
- Current executable binds `0.0.0.0`; MCP transport guidance recommends localhost for local servers.
- This session cannot clone GitHub directly due DNS isolation, so GitHub Actions is the executable RED/GREEN verifier.

## Decision Log

- Extend `autodev-server`; reject parallel MCP runtimes to preserve one authority boundary.
- Use GitHub Actions for TDD verification because the working container has no GitHub network resolution.
- Keep replan/test operations proposal-only in this slice.

## Outcomes & Retrospective

Not yet populated. Completion requires exact-head CI evidence.