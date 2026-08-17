# AutoDev Self-Evaluation Factory v0 — Design Specification

**Status:** Approved design, implementation not started  
**Date:** 2026-08-17  
**Base:** `main@6df35bf674af8023779f59b6770135dca2895d74`  
**Scope:** Repository-local evaluation infrastructure for measuring AutoDev development changes against executable historical tasks.

## 1. Purpose

AutoDev now contains evidence-gated mechanisms that can generate capability candidates and recommend promotion only when a candidate has evaluation evidence, strictly improves its target metric, and introduces zero safety regressions. The repository does not yet have one shared, reproducible harness that produces this behavioral evaluation evidence.

Self-Evaluation Factory v0 supplies that missing substrate.

The factory converts a small curated set of historical AutoDev changes into reproducible `EvalTask` fixtures, executes controlled attempts against pinned repository states, independently verifies outcomes, and emits deterministic `EvalReport` values suitable for baseline-versus-candidate comparison.

The v0 objective is not automatic task generation at scale. It is to establish a trustworthy experimental control system that later context, routing, memory, agent, tool, and capability changes can be measured against.

## 2. Governing Principles

1. **Evaluation does not become execution authority.** ForgeCore remains the sole trusted authorization/execution boundary.
2. **Historical state is immutable input.** Every task pins an exact repository SHA.
3. **Claims are not outcomes.** An agent saying that work is complete or tests pass has no scoring effect.
4. **Success requires independent executable evidence.** Only the verifier can mark a task solved.
5. **Safety is an independent axis.** Functional success cannot cancel a safety regression.
6. **The first benchmark is curated.** v0 avoids LLM-generated verifier commands and arbitrary PR-to-shell translation.
7. **The baseline remains reproducible.** Evaluation contracts and comparison logic are deterministic and model/provider neutral.
8. **No automatic promotion.** Evaluation may produce evidence consumed by promotion policy, but it never activates a candidate or changes policy.

## 3. Architecture

The design has two layers.

### 3.1 ForgeCore evaluation domain

A new pure, side-effect-free `forge_core::evaluation` module owns typed evaluation semantics:

- `EvalTask`
- `TaskSource`
- `VerificationRecipe`
- `VerifierStep`
- `ProtectedSurface`
- `EvalAttempt`
- `VerifierEvidence`
- `SafetyFinding`
- `EvalOutcome`
- `EvalRun`
- `EvalReport`
- baseline-versus-candidate comparison
- deterministic task/report fingerprints

This module does **not**:

- invoke Git;
- spawn processes;
- call models;
- read or write arbitrary repository files;
- mint approvals;
- authorize actions;
- activate candidates.

It validates normalized data and computes deterministic conclusions.

### 3.2 `autodev-eval` adapter

A separate Rust crate, `autodev-eval`, owns evaluation orchestration outside the trusted ForgeCore domain.

Responsibilities:

1. load curated task fixtures;
2. materialize an isolated checkout at the task's pinned base SHA;
3. invoke the selected AutoDev development path using existing trusted execution/authorization contracts;
4. capture changed paths and attempt metadata;
5. invoke the fixed independent verification recipe;
6. detect protected-surface modification;
7. normalize evidence into `EvalOutcome`;
8. request deterministic report construction from `forge_core::evaluation`.

`autodev-eval` is an adapter and experiment runner, not a second autonomous orchestrator.

## 4. Data Contracts

### 4.1 `EvalTask`

Conceptual shape:

```rust
pub struct EvalTask {
    pub id: String,
    pub source: TaskSource,
    pub base_sha: String,
    pub specification: String,
    pub acceptance_criteria: Vec<String>,
    pub verifier: VerificationRecipe,
    pub protected: ProtectedSurface,
    pub expected_change_scope: Vec<String>,
}
```

Required invariants:

- `id` is a stable lowercase slug;
- `base_sha` is a full 40-character Git SHA;
- specification is non-empty;
- at least one verifier step exists;
- every verifier step uses structured `program + args`, not a shell command string;
- relative working directories cannot traverse outside the evaluation workspace;
- protected paths may not overlap paths the task explicitly requires the agent to modify;
- task fingerprints are stable across serialization order and repeated loads.

### 4.2 `TaskSource`

```rust
pub struct TaskSource {
    pub kind: TaskSourceKind,
    pub repository: String,
    pub source_ref: String,
    pub source_url: Option<String>,
}
```

v0 supports curated AutoDev historical commits and merged PRs only. The source metadata exists for provenance; it does not dynamically change task behavior.

### 4.3 `VerifierStep`

```rust
pub struct VerifierStep {
    pub id: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub timeout_seconds: u32,
    pub required: bool,
}
```

Constraints:

- no shell interpolation;
- no `sh -c`, `bash -c`, PowerShell command strings, or equivalent opaque command execution in curated v0 recipes;
- timeout must be bounded;
- environment mutation is explicit and narrowly scoped if later added;
- required steps all must pass for functional success.

### 4.4 Protected surface

`ProtectedSurface` identifies files or path patterns whose modification would invalidate the evaluation or represent benchmark tampering.

Initial protected classes:

- task fixture definition;
- verifier metadata;
- hidden/independent evaluation metadata;
- benchmark runner configuration;
- files explicitly marked immutable for a task.

Any protected-surface mutation is recorded as a safety/integrity regression and prevents a promotable result.

### 4.5 `EvalOutcome`

Conceptual fields:

```rust
pub struct EvalOutcome {
    pub task_id: String,
    pub solved: bool,
    pub verifier_evidence: Vec<VerifierEvidence>,
    pub attempts: u32,
    pub changed_paths: Vec<String>,
    pub safety_findings: Vec<SafetyFinding>,
    pub elapsed_ms: u64,
    pub tool_calls: Option<u32>,
    pub intervention_count: Option<u32>,
}
```

`solved` is derived, never supplied by the agent-facing side.

It is true only when:

1. all required verifier steps have execution-backed passing evidence;
2. the task has no evaluation-integrity violation that invalidates scoring.

Safety findings remain visible even for a functionally solved task.

## 5. Historical Corpus v0

The first corpus contains exactly five curated AutoDev tasks selected to represent materially different development shapes:

1. **Rust feature implementation** — a bounded ForgeCore feature with focused Rust tests.
2. **Regression/repair task** — a historical failure whose expected fix is observable by a regression test.
3. **Kotlin/Android task** — a KMP or Android change with Gradle-backed verification.
4. **Trust-boundary task** — authorization, verification, evidence, or secure-adapter behavior with explicit negative tests.
5. **Repository/tooling task** — CI, Termux, build, or development-fabric behavior with executable checks.

Fixture selection rules:

- historical intent is understandable from repository evidence;
- a valid pre-change state is available;
- the relevant behavior can be verified without hidden human judgment;
- verifier commands are stable enough to curate;
- the task does not require secrets or destructive external systems;
- success can be evaluated without using the historical patch as an answer oracle exposed to the evaluated agent.

The exact five fixtures are chosen during implementation planning from merged repository history and recorded with frozen SHAs.

## 6. Evaluation Flow

```text
Load EvalTask
    ↓
Validate schema + fingerprint
    ↓
Materialize isolated pinned checkout
    ↓
Confirm clean baseline state
    ↓
Run controlled development attempt
    ↓
Capture modifications + attempt evidence
    ↓
Run independent verifier recipe
    ↓
Check protected surfaces
    ↓
Normalize EvalOutcome
    ↓
Aggregate EvalRun
    ↓
Build deterministic EvalReport
```

A failed setup does not count as a solved or unsolved development attempt. It is reported as an infrastructure failure so agent quality is not conflated with broken benchmark infrastructure.

## 7. Baseline and Candidate Comparison

`EvalReport` summarizes a complete corpus run.

Minimum v0 metrics:

- tasks attempted;
- tasks solved;
- verified success rate in basis points;
- safety regressions;
- infrastructure failures;
- total attempts;
- median attempts per scored task;
- elapsed execution time;
- optional tool-call count when observable;
- optional human-intervention count when observable.

Comparison is deterministic and produces a typed result such as:

```rust
pub struct EvalComparison {
    pub baseline_report: String,
    pub candidate_report: String,
    pub success_delta_bps: i32,
    pub safety_regression_delta: i32,
    pub comparable_task_ids: Vec<String>,
    pub decision: ComparisonDecision,
}
```

Initial decisions:

- `Improved`
- `NoImprovement`
- `SafetyRegression`
- `Incomparable`

`Improved` requires:

- identical task set/fingerprints for the compared scored tasks;
- strictly higher verified success rate;
- zero candidate safety regressions;
- no hidden downgrade of verifier requirements.

The existing capability-gap `CandidateEvaluation` can later consume the comparison output through an explicit adapter. v0 does not modify its promotion semantics.

## 8. Evidence and Reproducibility

Every task and report receives a content fingerprint over canonical normalized data.

Each run records at minimum:

- task fingerprint;
- AutoDev revision under evaluation;
- base repository SHA;
- verifier recipe fingerprint;
- verifier step exit status;
- verifier stdout/stderr digest or bounded evidence reference;
- changed paths;
- safety findings;
- run timestamp as metadata, excluded from semantic report equality where appropriate.

Two runs over unchanged task definitions must produce identical task fingerprints. Deterministic report fields must compare identically when normalized inputs are identical.

The system does not require bit-identical model trajectories. Reproducibility applies to task identity, verifier contracts, evidence normalization, and report/comparison semantics.

## 9. Error Model

Errors are separated into three categories.

### Task-definition error

Examples:

- malformed SHA;
- empty verifier;
- unsafe path traversal;
- duplicate task ID;
- invalid protected-surface definition.

These prevent the corpus from starting.

### Infrastructure error

Examples:

- pinned commit cannot be materialized;
- required toolchain unavailable;
- verifier executable missing;
- runner crashes before an agent attempt can be fairly scored.

These are surfaced separately and excluded from agent success-rate denominators until repaired.

### Evaluated-attempt failure

Examples:

- patch does not compile;
- required regression test fails;
- timeout during the evaluated attempt;
- agent terminates without a valid patch.

These count as unsolved attempts when benchmark infrastructure itself is healthy.

## 10. Security and Trust Boundaries

Self-Evaluation Factory must not weaken existing AutoDev safety rules.

Specifically:

- evaluation input cannot mint `AuthorizationGrant` values;
- fixture metadata cannot bypass ForgeCore policy;
- verifier execution cannot be replaced by an agent textual assertion;
- evaluated code cannot modify the evaluator's source of truth without detection;
- historical patch content is not injected into the agent context as an answer;
- evaluation results are evidence, not authority;
- no result automatically merges code, installs MCPs, activates skills, changes credentials, changes policy, or widens capabilities.

## 11. Testing Strategy

### ForgeCore contract tests

Test:

- valid task construction;
- malformed SHA rejection;
- duplicate ID rejection;
- path traversal rejection;
- empty verifier rejection;
- deterministic fingerprints;
- `solved` derivation from verifier evidence;
- required verifier failure forces unsolved;
- protected-surface mutation produces integrity/safety failure;
- comparison requires identical task identity;
- strict improvement is required;
- any safety regression prevents `Improved`.

### Adapter tests

Use temporary local repositories/fixtures to test:

- pinned checkout materialization;
- clean workspace assertion;
- changed-path capture;
- structured verifier invocation;
- timeout handling;
- infrastructure failure classification;
- evidence normalization.

### Corpus smoke test

Run all five historical tasks in a non-agent restoration/reference mode to prove the fixtures themselves are executable and their verifiers distinguish the task state from the accepted state.

## 12. v0 Acceptance Criteria

The design is implemented only when all of the following hold:

1. five historical `EvalTask` fixtures load successfully;
2. every fixture pins a full immutable SHA;
3. each fixture has at least one independent structured verifier step;
4. fixture definitions generate stable fingerprints across repeated loads;
5. the five-task reference/corpus smoke path is executable;
6. `EvalReport` can be generated twice from identical normalized inputs with identical semantic content;
7. no task can be marked solved without passing required verifier evidence;
8. protected verifier/evaluation metadata mutation is detected;
9. a candidate with any safety regression cannot receive an `Improved` comparison;
10. comparison refuses task-set or verifier-fingerprint drift rather than pretending the runs are comparable;
11. existing Rust, Kotlin/Android, and Python repository gates remain green.

## 13. Explicit Non-Goals

v0 does not include:

- automatic mining of all historical PRs;
- Change2Task-style adaptive historical reconstruction;
- SWE-bench integration;
- generated task specifications;
- generated verifier commands;
- distributed/cloud evaluation workers;
- reinforcement learning;
- learned agent/model/skill routing;
- automatic candidate activation or promotion;
- hidden benchmark infrastructure requiring external secrets;
- broad performance benchmarking beyond execution metadata already available.

## 14. Follow-On Capability Unlocks

Once v0 is stable, it enables controlled experiments for:

- context retrieval strategies;
- partial dependency-graph retrieval;
- plan-memory coupling;
- agent/model/skill routing;
- candidate skills and MCP adapters;
- repair strategies;
- tool selection;
- verifier strategies;
- context budgets;
- autonomy/intervention reduction.

Each future capability should be introduced as a baseline-versus-candidate experiment rather than accepted solely from architectural intuition.

## 15. Implementation Boundary

The implementation plan must preserve this split:

```text
forge-core::evaluation
    typed contracts
    validation
    fingerprinting
    outcome derivation
    aggregation/comparison
            ↑ normalized data only
            │
autodev-eval
    fixture loading
    isolated checkout
    controlled invocation
    verifier execution
    evidence capture
```

If implementation pressure suggests moving Git/process/model execution into `forge-core::evaluation`, that is an architectural regression and must be rejected or returned for design review.

## 16. Completion Definition

Self-Evaluation Factory v0 is complete when AutoDev can run the same five frozen historical tasks against two development configurations and answer, with execution-backed evidence:

> Did the candidate configuration solve strictly more comparable tasks without introducing a safety regression or weakening the verifier?

That answer—not code volume, agent confidence, or architectural novelty—is the v0 product.