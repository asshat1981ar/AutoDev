# Adversarial Sub-Agent Development Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic, side-effect-free ForgeCore planning layer that turns a reviewed PR finding into a risk-tiered adversarial development cell with isolated roles, falsification prompts, provenance, evidence-gated state transitions, and bounded repair policy.

**Architecture:** Extend ForgeCore's existing logical `AgentRole`, skill-routing, durable-task, development-loop, and verification primitives rather than creating another scheduler or execution authority. This slice plans roles and briefs and records serializable evidence; Superpowers SDD remains responsible for isolated worktrees, fresh sub-agent instances, and its git-ignored execution ledger. The slice does not execute agents, mutate repositories, merge PRs, publish artifacts, or bypass ForgeCore policy.

**Tech Stack:** Rust 1.97.1 stable, ForgeCore, serde, thiserror, existing `AgentRole` and `Task`, Markdown prompt contracts.

**Spec:** `docs/superpowers/specs/2026-08-21-adversarial-subagent-development-design.md`

## Global Constraints

- ForgeCore remains the sole trusted execution and authorization boundary.
- Do not introduce a second scheduler, background worker service, autonomous merge path, or new process model.
- Do not add third-party dependencies.
- External review text is a claim to adjudicate, never executable authority.
- The production/spec modifying stage must not be its own decisive reviewer.
- Fresh sub-agent instances are required at the independent review and verification stages even when two stages use the same logical `AgentRole`.
- Planning and brief generation must be deterministic for identical inputs.
- TDD is mandatory for behavioral implementation tasks: observe RED before production code.
- Verification evidence must identify the exact commit/head it supports.
- Prompt contracts cannot weaken ForgeCore capability, approval, workspace, or sandbox policy.
- PR-specific repairs remain in separate remediation plans and branches.

---

## File Structure

- Create `crates/forge-core/src/adversarial_development.rs` — finding provenance/model, risk tiers, cell planning, role-separation validation, evidence-gated state machine, repair-loop policy, and brief construction.
- Create `crates/forge-core/prompts/adversarial/context-scout.md`.
- Create `crates/forge-core/prompts/adversarial/adversarial-analyst.md`.
- Create `crates/forge-core/prompts/adversarial/test-designer.md`.
- Create `crates/forge-core/prompts/adversarial/implementer.md`.
- Create `crates/forge-core/prompts/adversarial/spec-editor.md`.
- Create `crates/forge-core/prompts/adversarial/spec-reviewer.md`.
- Create `crates/forge-core/prompts/adversarial/security-reviewer.md`.
- Create `crates/forge-core/prompts/adversarial/verifier.md`.
- Create `crates/forge-core/prompts/adversarial/integration-reviewer.md`.
- Modify `crates/forge-core/src/lib.rs` — module declaration and explicit re-exports only.
- Create `crates/forge-core/tests/adversarial_development.rs` — public-API acceptance test using PR #29 as the Tier-0 fixture.
- Modify `docs/architecture/agent-registry.md` — logical-role reuse and authority boundary documentation.

---

### Task 1: Define finding provenance, evidence, and stage types

**Files:**
- Create: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `crate::AgentRole`, serde.
- Produces: `AdversarialRiskTier`, `FindingSourceType`, `FindingCategory`, `FindingValidity`, `AdversarialStage`, `FindingStatus`, `FindingEvidenceKind`, `FindingEvidence`, `FindingRuling`, `AdversarialFinding`, `CellStage`, `AdversarialCellPlan`, `AdversarialDevelopmentError`.

- [ ] **Step 1: Write the serialization test before the types exist**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_round_trips_with_provenance_and_stable_wire_names() {
        let finding = AdversarialFinding::new(
            "PR29-CR-001",
            29,
            "ab76053135f638943bcdf1d3367af3a16a34c53b",
            FindingSourceType::CodeRabbit,
            "PRRT_kwDOTzuzVM6bG4UZ",
            AdversarialRiskTier::Tier0,
            FindingCategory::Authorization,
            "run_test adapter must reject non-RunTest actions before privileged evaluation",
            vec!["crates/forge-core/src/run_test.rs".into()],
        );
        let value = serde_json::to_value(&finding).unwrap();
        assert_eq!(value["pr"], 29);
        assert_eq!(value["source_type"], "coderabbit");
        assert_eq!(value["risk_tier"], "tier_0");
        assert_eq!(value["status"], "discovered");
        assert_eq!(value["validity"], "unadjudicated");
        assert_eq!(serde_json::from_value::<AdversarialFinding>(value).unwrap(), finding);
    }
}
```

- [ ] **Step 2: Run and verify RED**

```bash
cargo test -p forge-core finding_round_trips_with_provenance_and_stable_wire_names -- --nocapture
```

Expected: compile failure because the types do not exist.

- [ ] **Step 3: Implement the minimal serializable model**

Use explicit risk-tier wire names so the test and serialized ledger cannot disagree:

```rust
use serde::{Deserialize, Serialize};
use crate::AgentRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdversarialRiskTier {
    #[serde(rename = "tier_0")]
    Tier0,
    #[serde(rename = "tier_1")]
    Tier1,
    #[serde(rename = "tier_2")]
    Tier2,
    #[serde(rename = "tier_3")]
    Tier3,
    #[serde(rename = "tier_4")]
    Tier4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSourceType { CodeRabbit, Human, Ci, Local }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Authorization, Security, Correctness, Reliability, Persistence,
    Specification, Documentation, Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingValidity { Unadjudicated, Valid, PartiallyValid, FalsePositive, Stale }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdversarialStage {
    ContextScout, AdversarialAnalysis, TestDesign, Implement, EditSpecification,
    SpecReview, SecurityReview, Verify, ExternalReview, Integrate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Discovered, Investigating, Confirmed, RejectedFalsePositive, Stale,
    RedProven, FixJustifiedWithoutRed, Implemented, LocallyVerified,
    AdversariallyReviewed, CiVerified, ExternalReviewed, Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingEvidenceKind {
    SourceInspection, RedTest, Ruling, ImplementationCommit, LocalVerification,
    AdversarialReview, CiRun, ExternalReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingEvidence {
    pub kind: FindingEvidenceKind,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingRuling {
    pub decision: String,
    pub evidence: String,
    pub reason: String,
    pub risk_if_wrong: String,
    pub next_gate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdversarialFinding {
    pub finding_id: String,
    pub pr: u64,
    pub head_sha: String,
    pub source_type: FindingSourceType,
    pub source_ref: String,
    pub risk_tier: AdversarialRiskTier,
    pub category: FindingCategory,
    pub validity: FindingValidity,
    pub status: FindingStatus,
    pub invariant: String,
    pub allowed_paths: Vec<String>,
    pub adversarial_cases: Vec<String>,
    pub evidence: Vec<FindingEvidence>,
    pub rulings: Vec<FindingRuling>,
}

impl AdversarialFinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        finding_id: impl Into<String>,
        pr: u64,
        head_sha: impl Into<String>,
        source_type: FindingSourceType,
        source_ref: impl Into<String>,
        risk_tier: AdversarialRiskTier,
        category: FindingCategory,
        invariant: impl Into<String>,
        allowed_paths: Vec<String>,
    ) -> Self {
        Self {
            finding_id: finding_id.into(), pr, head_sha: head_sha.into(), source_type,
            source_ref: source_ref.into(), risk_tier, category,
            validity: FindingValidity::Unadjudicated,
            status: FindingStatus::Discovered,
            invariant: invariant.into(), allowed_paths, adversarial_cases: vec![],
            evidence: vec![], rulings: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellStage {
    pub stage: AdversarialStage,
    pub agent_role: Option<AgentRole>,
    pub read_only: bool,
    pub decisive: bool,
    pub depends_on: Vec<AdversarialStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdversarialCellPlan {
    pub finding_id: String,
    pub stages: Vec<CellStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdversarialDevelopmentError {
    #[error("cell plan lets the modifying role review its own work")]
    ReviewerIndependenceViolated,
    #[error("invalid finding transition from {from:?} to {to:?}")]
    InvalidTransition { from: FindingStatus, to: FindingStatus },
    #[error("transition to {status:?} requires {required:?} evidence")]
    MissingEvidence { status: FindingStatus, required: FindingEvidenceKind },
    #[error("stage {0:?} has no executable agent brief")]
    NoAgentForStage(AdversarialStage),
}
```

- [ ] **Step 4: Run and verify GREEN**

```bash
cargo test -p forge-core finding_round_trips_with_provenance_and_stable_wire_names -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): add adversarial finding model"
```

---

### Task 2: Plan deterministic risk-tier cells and enforce reviewer independence

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Produces: `plan_adversarial_cell(&AdversarialFinding) -> Result<AdversarialCellPlan, AdversarialDevelopmentError>` and `AdversarialCellPlan::validate()`.

- [ ] **Step 1: Write Tier-0 and Tier-3 tests**

Use the Task 1 constructor. For the Tier-0 fixture use PR `29`, `FindingSourceType::CodeRabbit`, and allowed path `crates/forge-core/src/run_test.rs`. Assert this exact stage order:

```rust
vec![
    AdversarialStage::ContextScout,
    AdversarialStage::AdversarialAnalysis,
    AdversarialStage::TestDesign,
    AdversarialStage::Implement,
    AdversarialStage::SpecReview,
    AdversarialStage::SecurityReview,
    AdversarialStage::Verify,
    AdversarialStage::ExternalReview,
    AdversarialStage::Integrate,
]
```

Assert `Implement` uses `AgentRole::Developer`, `SpecReview` uses `AgentRole::Architect`, `SecurityReview` uses `AgentRole::SecurityReviewer`, and `Verify` uses `AgentRole::Tester`. Assert every decisive local reviewer has a role different from the modifying `Developer` role.

For Tier 3, use PR `41`, source `CodeRabbit`, category `Specification`, and assert `EditSpecification` uses `AgentRole::Architect` while decisive `SpecReview` uses `AgentRole::SecurityReviewer` and final `Integrate` uses `AgentRole::Planner`.

- [ ] **Step 2: Run each RED test separately**

```bash
cargo test -p forge-core tier_zero_uses_full_cell_with_independent_decisive_reviewers -- --nocapture
cargo test -p forge-core tier_three_edits_specs_without_self_review -- --nocapture
```

Expected: compile failure because the planner does not exist.

- [ ] **Step 3: Implement exact stage maps**

```text
Tier0: ContextScout(Researcher)
       -> AdversarialAnalysis(SecurityReviewer)
       -> TestDesign(Tester)
       -> Implement(Developer)
       -> {SpecReview(Architect), SecurityReview(SecurityReviewer)}
       -> Verify(Tester)
       -> ExternalReview(no local agent)
       -> Integrate(Architect)

Tier1: ContextScout(Researcher)
       -> AdversarialAnalysis(SecurityReviewer)
       -> TestDesign(Tester)
       -> Implement(Developer)
       -> SecurityReview(SecurityReviewer)
       -> Verify(Tester)
       -> ExternalReview(no local agent)
       -> Integrate(Architect)

Tier2: ContextScout(Researcher)
       -> TestDesign(Tester)
       -> Implement(Developer)
       -> SpecReview(Architect)
       -> Verify(Tester)
       -> Integrate(Architect)

Tier3: ContextScout(Researcher)
       -> AdversarialAnalysis(SecurityReviewer)
       -> EditSpecification(Architect)
       -> SpecReview(SecurityReviewer)
       -> Verify(Tester)
       -> Integrate(Planner)

Tier4: Implement(Developer) -> Verify(Tester) -> Integrate(Architect)
```

`ExternalReview` has `agent_role: None`, `read_only: true`, and cannot produce an agent brief. Reviewer/verifier/integration stages are read-only. `Implement` and `EditSpecification` are modifying stages. `TestDesign` may create/modify tests during execution, but it is not an acceptance authority; fresh sub-agent instances preserve separation from `Verify` even though both use the logical Tester role.

`validate()` rejects a cell if the production/spec modifying role is also assigned to any decisive review/verification stage.

- [ ] **Step 4: Run planner tests and ForgeCore unit tests**

```bash
cargo test -p forge-core tier_zero_uses_full_cell_with_independent_decisive_reviewers -- --nocapture
cargo test -p forge-core tier_three_edits_specs_without_self_review -- --nocapture
cargo test -p forge-core --lib
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): plan risk-tiered adversarial cells"
```

---

### Task 3: Add falsification prompt contracts and self-contained agent briefs

**Files:**
- Create all nine Markdown files listed under `crates/forge-core/prompts/adversarial/` in File Structure.
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `AdversarialFinding`, `CellStage`, existing `crate::Task`.
- Produces: `AgentBrief` and `build_agent_brief(&AdversarialFinding, &CellStage)`.

- [ ] **Step 1: Write RED tests**

Test that an `AdversarialAnalysis` brief for PR #29:

```text
- uses AgentRole::SecurityReviewer
- contains finding id PR29-CR-001
- contains the exact head SHA
- contains the invariant
- contains the phrase "smallest source-supported counterexample"
- contains the allowed path crates/forge-core/src/run_test.rs
```

Also test that `ExternalReview` returns `AdversarialDevelopmentError::NoAgentForStage(AdversarialStage::ExternalReview)`.

Run:

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
cargo test -p forge-core external_review_is_a_gate_not_an_agent_prompt -- --nocapture
```

Expected: compile failure before implementation.

- [ ] **Step 2: Create exact prompt cores**

`context-scout.md` must say it is read-only, must establish current head SHA/source/public entry points/allowed scope, and must verify review claims against current source.

`adversarial-analyst.md` must include:

```text
Do not try to prove the patch is correct. Try to construct the smallest source-supported counterexample proving it is incorrect.
```

It must direct applicable attacks across alternate public entry points, confused deputy calls, malformed-but-type-valid inputs, duplicates/stale state, ordering, restart/persistence, capability-policy mismatch, partial failure, aliases/path normalization, and fail-open defaults.

`test-designer.md` must require a behavioral RED test and prohibit production implementation.

`implementer.md` must require minimal GREEN code, preserve ForgeCore authority, prohibit test weakening and unrelated refactors, and prohibit self-certification.

`spec-editor.md` must allow specification edits only and prohibit runtime code changes.

`spec-reviewer.md` must review the binding invariant independently of tests and return concrete counterexamples before acceptance.

`security-reviewer.md` must attack authorization ordering, confused deputy paths, canonicalization, TOCTOU, stale/mutable ownership, injection, secret exposure, unintended mutation, partial failure, concurrency, persistence/restart, traversal, and unsafe defaults when applicable.

`verifier.md` must require fresh current-SHA evidence and prohibit relying on implementer claims or previous runs.

`integration-reviewer.md` must check interaction between fixes, unresolved review threads, docs/code mismatch, authority drift, stale evidence, and reviewed/tested SHA equality; it may produce merge-readiness evidence but may not merge.

- [ ] **Step 3: Implement `AgentBrief` and prompt selection**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBrief {
    pub stage: AdversarialStage,
    pub agent_role: AgentRole,
    pub task: crate::Task,
}
```

Use `include_str!("../prompts/adversarial/<file>.md")` for local stages. Build `Task.context` deterministically in this order:

```text
Finding: <finding id>
PR: <number>
Head SHA: <sha>
Source: <source type>:<source ref>
Invariant: <invariant>
Allowed paths:
- <sorted allowed path entries>

<stage prompt>
```

Sort a cloned `allowed_paths` before rendering so caller input order cannot change the brief. Use task id `<finding-id>:<stage-wire-name>` and title `<stage-wire-name>: <finding-id>`.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
cargo test -p forge-core external_review_is_a_gate_not_an_agent_prompt -- --nocapture
cargo check -p forge-core
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/prompts/adversarial crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): add adversarial agent briefs"
```

---

### Task 4: Enforce finding validity and evidence-gated transitions

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Produces: `AdversarialFinding::adjudicate`, `AdversarialFinding::transition`, `AdversarialFinding::add_ruling`.

- [ ] **Step 1: Write RED tests**

Test these rules:

```text
Discovered -> Investigating requires no evidence.
Investigating -> Confirmed requires SourceInspection and validity Valid or PartiallyValid.
Investigating -> RejectedFalsePositive requires SourceInspection and validity FalsePositive.
Investigating -> Stale requires SourceInspection and validity Stale.
Confirmed -> RedProven requires RedTest.
Confirmed -> FixJustifiedWithoutRed requires Ruling.
RedProven/FixJustifiedWithoutRed -> Implemented requires ImplementationCommit.
Implemented -> LocallyVerified requires LocalVerification.
LocallyVerified -> AdversariallyReviewed requires AdversarialReview.
AdversariallyReviewed -> CiVerified requires CiRun.
CiVerified -> ExternalReviewed requires ExternalReview.
ExternalReviewed -> Done requires no new evidence.
Every other transition is invalid.
```

Include a test that `Confirmed -> RedProven` without `RedTest` returns `MissingEvidence` and a test that `Discovered -> Done` returns `InvalidTransition`.

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
cargo test -p forge-core finding_cannot_skip_to_done -- --nocapture
```

Expected: compile failure before transition methods exist.

- [ ] **Step 3: Implement adjudication and transitions**

`adjudicate(validity, evidence)` is allowed only while `status == Investigating`; it requires non-empty `SourceInspection` evidence and maps validity to the corresponding destination status. `Valid` and `PartiallyValid` map to `Confirmed`.

`transition(next, evidence)` enforces the remaining ordered state machine and exact destination evidence kinds above. Reject an evidence value whose `reference.trim().is_empty()`.

`add_ruling(ruling)` appends the ruling and records `FindingEvidenceKind::Ruling` using the ruling's non-empty evidence text.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
cargo test -p forge-core finding_cannot_skip_to_done -- --nocapture
cargo test -p forge-core adversarial_development::tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): gate finding state on evidence"
```

---

### Task 5: Add bounded repair-loop routing

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Produces: `RepairStrategy` and `repair_strategy(round: u32)`.

- [ ] **Step 1: Write RED test**

```rust
#[test]
fn repair_strategy_replaces_context_and_trips_a_breaker() {
    assert_eq!(repair_strategy(1), RepairStrategy::ResumeImplementer);
    assert_eq!(repair_strategy(3), RepairStrategy::ResumeImplementer);
    assert_eq!(repair_strategy(4), RepairStrategy::ReplaceImplementer);
    assert_eq!(repair_strategy(5), RepairStrategy::ArchitecturalAdjudication);
    assert_eq!(repair_strategy(6), RepairStrategy::BreakerDisposition);
}
```

Run:

```bash
cargo test -p forge-core repair_strategy_replaces_context_and_trips_a_breaker -- --nocapture
```

Expected: compile failure.

- [ ] **Step 2: Implement exact repair policy**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairStrategy {
    ResumeImplementer,
    ReplaceImplementer,
    ArchitecturalAdjudication,
    BreakerDisposition,
}

pub fn repair_strategy(round: u32) -> RepairStrategy {
    match round {
        0..=3 => RepairStrategy::ResumeImplementer,
        4 => RepairStrategy::ReplaceImplementer,
        5 => RepairStrategy::ArchitecturalAdjudication,
        _ => RepairStrategy::BreakerDisposition,
    }
}
```

Round zero means no failed repair round has yet occurred. The controller increments after each failed independent review.

- [ ] **Step 3: Verify GREEN and commit**

```bash
cargo test -p forge-core repair_strategy_replaces_context_and_trips_a_breaker -- --nocapture
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): bound adversarial repair loops"
```

---

### Task 6: Export the planning API and prove the PR #29 Tier-0 fixture

**Files:**
- Modify: `crates/forge-core/src/lib.rs`
- Create: `crates/forge-core/tests/adversarial_development.rs`
- Modify: `docs/architecture/agent-registry.md`

**Interfaces:**
- Produces the stable public planning/evidence surface for the outer SDD controller.

- [ ] **Step 1: Write the public-API RED test**

Create a PR #29 `AdversarialFinding` with:

```text
finding_id: PR29-CR-001
pr: 29
head_sha: ab76053135f638943bcdf1d3367af3a16a34c53b
source_type: coderabbit
source_ref: PRRT_kwDOTzuzVM6bG4UZ
risk_tier: tier_0
category: authorization
allowed_paths: [crates/forge-core/src/run_test.rs]
invariant: execute_run_test must reject non-RunTest actions before policy, capability, runner parsing, or process delegation
```

Assert the planned cell contains `SecurityReviewer` for adversarial/security stages, `Developer` for implementation, `Tester` for verification, and that the adversarial brief contains the source-supported-counterexample instruction and reviewed SHA.

Run:

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
```

Expected: compile failure because public exports are absent.

- [ ] **Step 2: Add explicit module/re-exports**

In `lib.rs` add `pub mod adversarial_development;` and re-export the public model, planner, brief builder, transition/evidence types, and repair strategy. Do not export prompt text as authority.

- [ ] **Step 3: Document existing logical-role reuse**

Add `## Adversarial development cells` to `docs/architecture/agent-registry.md` with these exact requirements:

```text
- a cell is a deterministic plan over existing logical AgentRole values
- it does not create privileged agent processes or a second execution authority
- modifying and decisive-review stages use fresh sub-agent instances
- executable actions still pass through existing ForgeCore policy, workspace, approval, and sandbox boundaries
```

- [ ] **Step 4: Run acceptance and full Rust verification**

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

Expected: every command exits 0. These are the current repository Rust CI checks.

- [ ] **Step 5: Inspect authority drift before commit**

```bash
git diff main...HEAD -- crates/forge-core docs/architecture/agent-registry.md
```

Confirm from the actual diff:

```text
- no execution path was added
- no capability/policy/approval/sandbox rule was weakened
- no scheduler/background worker was added
- no merge/publish path was added
- prompts cannot bypass ForgeCore execution policy
- only existing AgentRole values are used
```

- [ ] **Step 6: Commit**

```bash
git add crates/forge-core/src/lib.rs crates/forge-core/tests/adversarial_development.rs docs/architecture/agent-registry.md
git commit -m "feat(forge-core): expose adversarial development planning"
```

---

## Execution Setup and Durable Ledger

Before Task 1, the executor must use `superpowers:using-git-worktrees`, then `superpowers:subagent-driven-development`. The SDD workspace's `progress.md`, briefs, reports, review packages, and command evidence are the durable execution ledger for this plan. The repository model above is serializable planning/evidence data; it does not replace that workspace.

Each task ends with a fresh independent review before the controller records `Task N: complete`. After all six tasks, run one broad branch review and one final full verification pass before using `superpowers:finishing-a-development-branch`.

## Plan Self-Review Checklist

- [ ] Every required spec role is represented or intentionally mapped to an existing logical `AgentRole`.
- [ ] PR/source/head provenance and allowed-path scope are represented in the finding model.
- [ ] Tier wire names are explicit and match tests.
- [ ] All RED commands are unambiguous single-test commands.
- [ ] Context Scout and reviewer stages are read-only.
- [ ] Test design is separate from production implementation.
- [ ] Modifying stages cannot serve as their own decisive reviewers.
- [ ] Tier-0/Tier-1 include adversarial security treatment.
- [ ] Tier-3 specification work cannot introduce runtime implementation.
- [ ] Finding state transitions require evidence from the exact ordered gates.
- [ ] Repair rounds 1-3 resume, 4 replace context, 5 adjudicate architecture, and later rounds require explicit breaker disposition.
- [ ] External review is a gate, not an executable agent.
- [ ] No new dependency, scheduler, process model, merge path, or execution authority is introduced.
- [ ] PR #29 is only the harness acceptance fixture; its security repair executes from its separate remediation plan and branch.
