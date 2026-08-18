# ExecPlan Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish AutoDev's first durable ExecPlan control plane so multi-hour development objectives have typed, recoverable, authority-free plan state plus a repository living-plan contract.

**Architecture:** Add a focused `forge-core::exec_plan` domain model that references existing TaskGraph/run/envelope identities but contains no execution or approval capability. `PLANS.md` is the human-readable living-plan contract; typed Rust state is canonical for lifecycle correctness. Harness drift checks enforce the presence and critical invariants of the planning contract.

**Tech Stack:** Rust stable, serde/serde_json, chrono, Python 3 stdlib harness validation, Markdown.

**Spec:** `docs/superpowers/specs/2026-08-18-federated-harness-kernel-design.md`

## Global Constraints

- ForgeCore remains the sole trusted execution authority.
- Plans, plugins, skills, models, MCP servers, and evaluation results cannot mint `AuthorizationGrant` or mark execution verified.
- Existing `TaskGraph`, `ExecutionEnvelope`, verification, and approval semantics are composed, not replaced.
- Rust commands run from `crates/`; Python harness commands run from repository root.
- No new root package manifest or Python dependency is introduced.
- All retries/replans are bounded and persisted.
- Human-readable plan prose is not parsed as trusted machine state.

---

## File map

- Create `PLANS.md` — repository contract and required living ExecPlan sections.
- Create `crates/forge-core/src/exec_plan.rs` — typed durable plan state, milestones, references, budgets, checkpoint/recovery transitions, validation.
- Create `crates/forge-core/tests/exec_plan.rs` — public-contract and adversarial serialization/authority tests.
- Modify `crates/forge-core/src/lib.rs` — expose the ExecPlan domain API.
- Modify `scripts/check_harness_drift.py` — enforce PLANS.md existence and invariant phrases/sections.
- Modify `AGENTS.md` — instruct workers when and how to use/update ExecPlans.
- Modify `README.md` — document durable ExecPlan control-plane behavior.

### Task 1: Repository living-plan contract

**Files:**
- Create: `PLANS.md`
- Modify: `AGENTS.md`
- Modify: `README.md`

**Interfaces:**
- Consumes: existing `TaskGraph`, `ExecutionEnvelope`, `AuthorizationGrant`, and verification semantics documented by the repository.
- Produces: the normative human-readable ExecPlan contract consumed by future agentic workers and checked by Task 4.

- [ ] **Step 1: Create `PLANS.md` with the required contract**

Write a self-contained repository guide requiring every multi-hour/architectural plan to contain these exact headings:

```markdown
# AutoDev ExecPlans

## Purpose
## Non-negotiable authority boundary
## How to author an ExecPlan
## Required living sections
### Progress
### Surprises & Discoveries
### Decision Log
### Outcomes & Retrospective
## Milestones and observable proof
## Checkpoints, interruption, and resume
## Bounded replanning
## Evidence and completion
## Plan maintenance rules
```

The prose must explicitly state: `An ExecPlan is durable coordination state, not execution authority.` It must require workers to update Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective as work proceeds; define milestone completion through observable verification; require reconciliation of interrupted effectful operations before retry; and forbid a plan from minting approvals or self-verifying.

- [ ] **Step 2: Add the ExecPlan rule to `AGENTS.md`**

Under the durable harness guidance, add a concise rule: architectural/multi-hour work must use `PLANS.md`; the plan remains current after every milestone/discovery/decision; typed runtime state is authoritative for lifecycle; plans never confer execution authority.

- [ ] **Step 3: Document the control plane in `README.md`**

Add a `## Durable ExecPlans` section explaining the prose/typed-state split and how it composes with TaskGraph + ExecutionEnvelope + VerificationFabric.

- [ ] **Step 4: Run current harness drift before enforcement changes**

Run from repository root:

```bash
python scripts/check_harness_drift.py
```

Expected: PASS; the new documentation must not invalidate existing gates.

- [ ] **Step 5: Commit**

```bash
git add PLANS.md AGENTS.md README.md
git commit -m "docs: establish durable execplan contract"
```

### Task 2: Typed ExecPlan state and validation

**Files:**
- Create: `crates/forge-core/src/exec_plan.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/exec_plan.rs`

**Interfaces:**
- Consumes: string identities for existing task/run/envelope objects; `chrono::DateTime<Utc>`; serde.
- Produces: `ExecPlan`, `ExecPlanStatus`, `PlanMilestone`, `PlanReferences`, `PlanBudget`, `PlanCheckpoint`, `PlanDecision`, `PlanDiscovery`, `ExecPlanError`; methods `ExecPlan::new`, `validate`, `checkpoint`, `resume`, `record_decision`, `record_discovery`, `consume_replan`.

- [ ] **Step 1: Write failing construction/serialization tests**

Create `crates/forge-core/tests/exec_plan.rs` with tests equivalent to:

```rust
use forge_core::{ExecPlan, ExecPlanStatus, PlanBudget};

#[test]
fn exec_plan_round_trips_without_authority_fields() {
    let plan = ExecPlan::new("plan-1", "Ship durable planning", PlanBudget::new(3, 5));
    let json = serde_json::to_string(&plan).unwrap();
    assert!(!json.contains("authorization_grant"));
    assert!(!json.contains("approved"));
    let restored: ExecPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.id, "plan-1");
    assert_eq!(restored.status, ExecPlanStatus::Planned);
}
```

Add tests rejecting an empty id/goal and zero replan/attempt budgets.

- [ ] **Step 2: Run the focused test and confirm failure**

```bash
cd crates
cargo test -p forge-core --test exec_plan
```

Expected: FAIL because the ExecPlan API does not exist.

- [ ] **Step 3: Implement the minimal typed domain**

In `exec_plan.rs`, define serde-serializable structs/enums with private mutation through methods. `PlanReferences` contains `task_ids: Vec<String>`, `run_ids: Vec<String>`, and `envelope_ids: Vec<String>`. `PlanBudget` contains `max_replans: u32`, `max_attempts_per_milestone: u32`, and `replans_used: u32`. `ExecPlanStatus` contains `Planned`, `Running`, `Interrupted`, `Blocked`, `Completed`, `Cancelled`, and `Failed` using `snake_case` serde names.

`ExecPlan::validate()` must reject blank identity/goal, zero maximum budgets, used replans greater than maximum, duplicate milestone ids, and a completed plan containing incomplete milestones.

- [ ] **Step 4: Export the domain from `lib.rs`**

Add:

```rust
pub mod exec_plan;
pub use exec_plan::{
    ExecPlan, ExecPlanError, ExecPlanStatus, PlanBudget, PlanCheckpoint, PlanDecision,
    PlanDiscovery, PlanMilestone, PlanReferences,
};
```

- [ ] **Step 5: Run focused tests**

```bash
cargo test -p forge-core --test exec_plan
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add forge-core/src/exec_plan.rs forge-core/src/lib.rs forge-core/tests/exec_plan.rs
git commit -m "feat(forge-core): add durable execplan state"
```

### Task 3: Checkpoint, interruption, recovery, and bounded replan invariants

**Files:**
- Modify: `crates/forge-core/src/exec_plan.rs`
- Modify: `crates/forge-core/tests/exec_plan.rs`

**Interfaces:**
- Consumes: Task 2 domain types.
- Produces: deterministic state transitions that do not execute effects.

- [ ] **Step 1: Add failing recovery tests**

Add tests asserting:

```rust
#[test]
fn interrupted_plan_requires_reconciliation_before_running() {
    let mut plan = test_plan();
    plan.interrupt("process died during effect").unwrap();
    assert!(plan.resume(false).is_err());
    plan.resume(true).unwrap();
    assert_eq!(plan.status, ExecPlanStatus::Running);
}

#[test]
fn replanning_is_bounded() {
    let mut plan = ExecPlan::new("p", "g", PlanBudget::new(1, 2));
    plan.consume_replan("first").unwrap();
    assert!(plan.consume_replan("second").is_err());
}
```

Also test that checkpoints round-trip and preserve references, discoveries, decisions, milestone progress, and consumed budget.

- [ ] **Step 2: Run focused tests and confirm failure**

```bash
cd crates
cargo test -p forge-core --test exec_plan
```

Expected: FAIL on missing transition methods.

- [ ] **Step 3: Implement minimal transitions**

Implement `start`, `interrupt`, `block`, `resume(reconciled: bool)`, `cancel`, `complete`, `checkpoint`, `record_decision`, `record_discovery`, and `consume_replan`. `resume(false)` from `Interrupted` returns `ExecPlanError::ReconciliationRequired`. None of these methods accepts or returns `AuthorizationGrant`.

- [ ] **Step 4: Run focused and crate tests**

```bash
cargo test -p forge-core --test exec_plan
cargo test -p forge-core
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add forge-core/src/exec_plan.rs forge-core/tests/exec_plan.rs
git commit -m "feat(forge-core): add execplan recovery invariants"
```

### Task 4: Harness enforcement for durable planning

**Files:**
- Modify: `scripts/check_harness_drift.py`
- Test: existing Python unittest suite plus direct drift invocation.

**Interfaces:**
- Consumes: `PLANS.md` from Task 1.
- Produces: deterministic CI-suitable detection of missing/stale planning invariants.

- [ ] **Step 1: Add a failing planning-contract check**

Add `PLANS = ROOT / "PLANS.md"` and a `check_plans_contract(errors, verbose)` function requiring the file and these fragments:

```python
REQUIRED_PLAN_FRAGMENTS = [
    "## Non-negotiable authority boundary",
    "### Progress",
    "### Surprises & Discoveries",
    "### Decision Log",
    "### Outcomes & Retrospective",
    "An ExecPlan is durable coordination state, not execution authority.",
    "reconciliation",
    "verification",
]
```

Call it from the existing main validation flow.

- [ ] **Step 2: Prove the check detects drift**

Temporarily point the helper at missing content in a unit-level invocation or add a unittest fixture that omits one required fragment. Expected error contains `PLANS.md drift`.

- [ ] **Step 3: Run harness validation**

```bash
python -m py_compile scripts/check_harness_drift.py
python -m unittest discover -s tests -v
python scripts/check_harness_drift.py
```

Expected: all PASS against the real repository contract.

- [ ] **Step 4: Commit**

```bash
git add scripts/check_harness_drift.py tests
git commit -m "chore(harness): enforce execplan contract"
```

### Task 5: Full slice verification and evidence review

**Files:**
- Modify only if verification exposes a defect.

**Interfaces:**
- Consumes: Tasks 1-4.
- Produces: independently reproducible evidence that the first Federated Harness Kernel slice is safe to review.

- [ ] **Step 1: Run Rust formatting and static checks**

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
cd ..
```

Expected: PASS.

- [ ] **Step 2: Run repository harness gates**

```bash
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py scripts/check_harness_drift.py
python -m unittest discover -s tests -v
node --check scripts/termux-kanban.mjs
node scripts/termux-kanban.mjs --check
python scripts/check_harness_drift.py
bash scripts/verify_reproducible.sh
```

Expected: PASS or only the already-documented network-isolation skips from `verify_reproducible.sh`.

- [ ] **Step 3: Inspect authority separation**

Search the new module and tests:

```bash
rg "AuthorizationGrant|approved|execute\(" crates/forge-core/src/exec_plan.rs crates/forge-core/tests/exec_plan.rs
```

Expected: no production ExecPlan API that accepts/grants approval or directly executes effects. A test may mention forbidden serialized authority field names.

- [ ] **Step 4: Review the diff for scope discipline**

```bash
git status --short
git diff --check
git diff --stat
```

Expected: only ExecPlan-contract/domain/harness files changed; no generated files or lockfile drift.

- [ ] **Step 5: Commit verification fixes if any**

If verification required corrections, commit only those corrections:

```bash
git add <corrected-files>
git commit -m "fix(execplan): satisfy verification gates"
```

If no corrections were required, do not create an empty commit.

## Completion evidence

This plan is complete only when the focused ExecPlan tests, ForgeCore tests, full Rust workspace gates, Python harness suite, Termux syntax/check gate, harness drift check, and reproducible verification script have produced passing evidence (allowing only explicitly documented environmental skips). A worker's statement that the implementation is complete is not completion evidence.
