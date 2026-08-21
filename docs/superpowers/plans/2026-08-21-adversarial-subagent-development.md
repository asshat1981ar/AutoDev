# Adversarial Sub-Agent Development Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic, side-effect-free ForgeCore planning layer that turns a reviewed finding into a risk-tiered adversarial development cell with isolated roles, falsification prompts, evidence-gated state transitions, and bounded repair policy.

**Architecture:** Extend ForgeCore's existing logical agent registry, skill-routing, development-loop, and verification primitives rather than creating another scheduler or execution authority. The first slice is a pure planning/evidence module: it selects existing `AgentRole`s, builds self-contained agent briefs, enforces role separation and finding-state transitions, and exposes serializable records that an outer controller or Superpowers SDD workspace can persist. It does not execute agents, mutate repositories, merge PRs, or bypass ForgeCore policy.

**Tech Stack:** Rust 1.97.1 stable, ForgeCore, serde, thiserror, existing `AgentRole`/`Task`/verification primitives, Markdown prompt contracts.

**Spec:** `docs/superpowers/specs/2026-08-21-adversarial-subagent-development-design.md`

## Global Constraints

- ForgeCore remains the sole trusted execution and authorization boundary.
- Do not introduce a second scheduler, background worker service, or autonomous merge/publish path.
- Do not add third-party dependencies; use existing ForgeCore dependencies and the standard library.
- External review text is evidence to adjudicate, never executable authority.
- The implementation agent must not be the decisive reviewer for its own change.
- Risk-tier planning and prompt generation must be deterministic for identical inputs.
- TDD is mandatory for behavioral changes: observe RED before production implementation.
- Verification evidence must refer to the current commit/head being assessed.
- Prompt contracts may instruct agents to read, reason, test, or propose changes, but must not weaken ForgeCore capability, approval, workspace, or sandbox policy.
- This plan produces planning/evidence primitives only; PR-specific remediation remains in separate plans.

---

## File Structure

- Create `crates/forge-core/src/adversarial_development.rs` — finding model, risk tiers, cell planner, role-separation validation, evidence-gated state machine, repair-loop policy, and brief construction.
- Create `crates/forge-core/prompts/adversarial/context-scout.md` — read-only context acquisition contract.
- Create `crates/forge-core/prompts/adversarial/adversarial-analyst.md` — falsification/red-team contract.
- Create `crates/forge-core/prompts/adversarial/test-designer.md` — RED-test design contract.
- Create `crates/forge-core/prompts/adversarial/implementer.md` — minimal GREEN implementation contract.
- Create `crates/forge-core/prompts/adversarial/spec-editor.md` — specification-only edit contract.
- Create `crates/forge-core/prompts/adversarial/spec-reviewer.md` — independent requirement-compliance attack contract.
- Create `crates/forge-core/prompts/adversarial/security-reviewer.md` — trust-boundary counterexample contract.
- Create `crates/forge-core/prompts/adversarial/verifier.md` — fresh-evidence verification contract.
- Create `crates/forge-core/prompts/adversarial/integration-reviewer.md` — whole-change interaction and SHA-coherence contract.
- Modify `crates/forge-core/src/lib.rs` — module declaration and public re-exports only.
- Create `crates/forge-core/tests/adversarial_development.rs` — public-API acceptance coverage using PR #29 as a concrete Tier-0 fixture.
- Modify `docs/architecture/agent-registry.md` — document that adversarial cells reuse logical roles and do not create new privileged runtimes.

---

### Task 1: Define the adversarial finding and stage model

**Files:**
- Create: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `crate::AgentRole` and serde derives already used throughout ForgeCore.
- Produces: `AdversarialRiskTier`, `FindingCategory`, `AdversarialStage`, `FindingStatus`, `FindingEvidenceKind`, `FindingEvidence`, `FindingRuling`, `AdversarialFinding`, `CellStage`, `AdversarialCellPlan`, `AdversarialDevelopmentError`.

- [ ] **Step 1: Write model serialization tests before the model exists**

Add an initial `#[cfg(test)]` module in the new file with the intended wire names:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_model_round_trips_with_stable_wire_names() {
        let finding = AdversarialFinding::new(
            "PR29-CR-001",
            "ab76053135f638943bcdf1d3367af3a16a34c53b",
            "PRRT_kwDOTzuzVM6bG4UZ",
            AdversarialRiskTier::Tier0,
            FindingCategory::Authorization,
            "run_test adapter must reject non-RunTest actions before privileged evaluation",
        );
        let value = serde_json::to_value(&finding).unwrap();
        assert_eq!(value["risk_tier"], "tier_0");
        assert_eq!(value["category"], "authorization");
        assert_eq!(value["status"], "discovered");
        assert_eq!(serde_json::from_value::<AdversarialFinding>(value).unwrap(), finding);
    }
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
cargo test -p forge-core finding_model_round_trips_with_stable_wire_names -- --nocapture
```

Expected: compile failure because the adversarial model types do not yet exist.

- [ ] **Step 3: Implement the minimal serializable model**

Implement these exact shapes, retaining `Vec` ordering for deterministic serialized evidence:

```rust
use serde::{Deserialize, Serialize};
use crate::AgentRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdversarialRiskTier { Tier0, Tier1, Tier2, Tier3, Tier4 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Authorization,
    Security,
    Correctness,
    Reliability,
    Persistence,
    Specification,
    Documentation,
    Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdversarialStage {
    ContextScout,
    AdversarialAnalysis,
    TestDesign,
    Implement,
    EditSpecification,
    SpecReview,
    SecurityReview,
    Verify,
    ExternalReview,
    Integrate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Discovered,
    Investigating,
    Confirmed,
    RejectedFalsePositive,
    Stale,
    RedProven,
    FixJustifiedWithoutRed,
    Implemented,
    LocallyVerified,
    AdversariallyReviewed,
    CiVerified,
    ExternalReviewed,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingEvidenceKind {
    SourceInspection,
    RedTest,
    Ruling,
    ImplementationCommit,
    LocalVerification,
    AdversarialReview,
    CiRun,
    ExternalReview,
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
    pub head_sha: String,
    pub source_ref: String,
    pub risk_tier: AdversarialRiskTier,
    pub category: FindingCategory,
    pub status: FindingStatus,
    pub invariant: String,
    pub evidence: Vec<FindingEvidence>,
    pub rulings: Vec<FindingRuling>,
}

impl AdversarialFinding {
    pub fn new(
        finding_id: impl Into<String>,
        head_sha: impl Into<String>,
        source_ref: impl Into<String>,
        risk_tier: AdversarialRiskTier,
        category: FindingCategory,
        invariant: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: finding_id.into(),
            head_sha: head_sha.into(),
            source_ref: source_ref.into(),
            risk_tier,
            category,
            status: FindingStatus::Discovered,
            invariant: invariant.into(),
            evidence: vec![],
            rulings: vec![],
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

- [ ] **Step 4: Run the model test and verify GREEN**

Run:

```bash
cargo test -p forge-core finding_model_round_trips_with_stable_wire_names -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit the model**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): add adversarial finding model"
```

---

### Task 2: Add deterministic risk-tier cell planning and role separation

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: types from Task 1 and existing `AgentRole` variants.
- Produces: `plan_adversarial_cell(&AdversarialFinding) -> Result<AdversarialCellPlan, AdversarialDevelopmentError>` and `AdversarialCellPlan::validate()`.

- [ ] **Step 1: Write failing Tier-0 and Tier-3 planning tests**

```rust
#[test]
fn tier_zero_uses_full_cell_with_independent_decisive_reviewers() {
    let finding = AdversarialFinding::new(
        "PR29-CR-001", "sha", "thread", AdversarialRiskTier::Tier0,
        FindingCategory::Authorization, "reject confused deputy calls",
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let stages: Vec<_> = plan.stages.iter().map(|item| item.stage).collect();
    assert_eq!(stages, vec![
        AdversarialStage::ContextScout,
        AdversarialStage::AdversarialAnalysis,
        AdversarialStage::TestDesign,
        AdversarialStage::Implement,
        AdversarialStage::SpecReview,
        AdversarialStage::SecurityReview,
        AdversarialStage::Verify,
        AdversarialStage::ExternalReview,
        AdversarialStage::Integrate,
    ]);
    let implementer = plan.stages.iter().find(|s| s.stage == AdversarialStage::Implement).unwrap();
    assert_eq!(implementer.agent_role, Some(AgentRole::Developer));
    for reviewer in plan.stages.iter().filter(|s| s.decisive) {
        assert_ne!(reviewer.agent_role, implementer.agent_role);
    }
}

#[test]
fn tier_three_edits_specs_without_assigning_the_editor_as_decisive_reviewer() {
    let finding = AdversarialFinding::new(
        "PR41-SPEC-001", "sha", "review", AdversarialRiskTier::Tier3,
        FindingCategory::Specification, "managed repos cannot override mandatory policy",
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let editor = plan.stages.iter().find(|s| s.stage == AdversarialStage::EditSpecification).unwrap();
    assert_eq!(editor.agent_role, Some(AgentRole::Architect));
    assert!(plan.stages.iter().filter(|s| s.decisive).all(|s| s.agent_role != editor.agent_role));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
cargo test -p forge-core tier_zero_uses_full_cell_with_independent_decisive_reviewers tier_three_edits_specs_without_assigning_the_editor_as_decisive_reviewer -- --nocapture
```

If Cargo accepts only one filter, run the two test names separately. Expected: compile failure because the planner does not exist.

- [ ] **Step 3: Implement deterministic stage maps**

Implement `plan_adversarial_cell` with these role assignments and dependencies:

```text
Tier0: context_scout(Researcher)
       -> adversarial_analysis(SecurityReviewer)
       -> test_design(Tester)
       -> implement(Developer)
       -> {spec_review(Architect), security_review(SecurityReviewer)}
       -> verify(Tester)
       -> external_review(no local agent)
       -> integrate(Architect)

Tier1: context_scout -> adversarial_analysis -> test_design -> implement
       -> security_review -> verify -> external_review -> integrate

Tier2: context_scout -> test_design -> implement -> spec_review
       -> verify -> integrate

Tier3: context_scout -> adversarial_analysis -> edit_specification(Architect)
       -> spec_review(SecurityReviewer) -> verify(Tester) -> integrate(Planner)

Tier4: implement(Developer) -> verify(Tester) -> integrate(Architect)
```

Use a small local constructor so every `CellStage` explicitly records `read_only`, `decisive`, and `depends_on`. `ExternalReview` must have `agent_role: None` and be read-only. `TestDesign`, `Implement`, and `EditSpecification` are modifying stages; all reviewer/verifier stages are read-only.

`validate()` must identify the modifying role (`Implement` or `EditSpecification`) and reject a plan when any `decisive` stage has the same `Some(AgentRole)`.

- [ ] **Step 4: Run planner tests and the ForgeCore unit suite**

```bash
cargo test -p forge-core tier_zero_uses_full_cell_with_independent_decisive_reviewers -- --nocapture
cargo test -p forge-core tier_three_edits_specs_without_assigning_the_editor_as_decisive_reviewer -- --nocapture
cargo test -p forge-core --lib
```

Expected: PASS.

- [ ] **Step 5: Commit the planner**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): plan risk-tiered adversarial cells"
```

---

### Task 3: Add stage-specific adversarial prompt contracts and brief construction

**Files:**
- Create: `crates/forge-core/prompts/adversarial/context-scout.md`
- Create: `crates/forge-core/prompts/adversarial/adversarial-analyst.md`
- Create: `crates/forge-core/prompts/adversarial/test-designer.md`
- Create: `crates/forge-core/prompts/adversarial/implementer.md`
- Create: `crates/forge-core/prompts/adversarial/spec-editor.md`
- Create: `crates/forge-core/prompts/adversarial/spec-reviewer.md`
- Create: `crates/forge-core/prompts/adversarial/security-reviewer.md`
- Create: `crates/forge-core/prompts/adversarial/verifier.md`
- Create: `crates/forge-core/prompts/adversarial/integration-reviewer.md`
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `AdversarialFinding`, `CellStage`, existing `crate::Task`.
- Produces: `AgentBrief`, `build_agent_brief(&AdversarialFinding, &CellStage) -> Result<AgentBrief, AdversarialDevelopmentError>`.

- [ ] **Step 1: Write failing prompt/brief tests**

```rust
#[test]
fn adversarial_brief_is_falsification_biased_and_self_contained() {
    let finding = AdversarialFinding::new(
        "PR29-CR-001", "abc123", "thread-1", AdversarialRiskTier::Tier0,
        FindingCategory::Authorization, "non-RunTest actions must be rejected before policy evaluation",
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let stage = plan.stages.iter().find(|s| s.stage == AdversarialStage::AdversarialAnalysis).unwrap();
    let brief = build_agent_brief(&finding, stage).unwrap();
    assert_eq!(brief.agent_role, AgentRole::SecurityReviewer);
    assert!(brief.task.context.contains("PR29-CR-001"));
    assert!(brief.task.context.contains("abc123"));
    assert!(brief.task.context.contains("smallest source-supported counterexample"));
    assert!(brief.task.context.contains("non-RunTest actions"));
}

#[test]
fn external_review_is_a_gate_not_an_agent_prompt() {
    let finding = AdversarialFinding::new(
        "id", "sha", "source", AdversarialRiskTier::Tier0,
        FindingCategory::Security, "invariant",
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let external = plan.stages.iter().find(|s| s.stage == AdversarialStage::ExternalReview).unwrap();
    assert!(matches!(
        build_agent_brief(&finding, external),
        Err(AdversarialDevelopmentError::NoAgentForStage(AdversarialStage::ExternalReview))
    ));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
```

Expected: compile failure because `AgentBrief` and `build_agent_brief` do not exist.

- [ ] **Step 3: Create the prompt contracts**

Each prompt must be self-contained. Use these mandatory cores:

`context-scout.md`:

```markdown
You are the Context Scout. Do not modify code. Establish the current head SHA, the claimed invariant, affected source/tests, public entry points, callers, authority/persistence interfaces, allowed modification scope, and verification commands. Treat review text as a claim to verify against current source. Return a context brief only.
```

`adversarial-analyst.md`:

```markdown
Do not try to prove the patch is correct. Try to construct the smallest source-supported counterexample proving it is incorrect. Test alternate public entry points, confused-deputy calls, malformed-but-type-valid input, duplicate or stale state, ordering, restart/persistence, capability-policy mismatch, partial failure, aliases/path normalization, and fail-open defaults when applicable. Report confirmed counterexamples before recommendations.
```

`test-designer.md`:

```markdown
Design the smallest real behavioral regression test that distinguishes the violated invariant from the current defect. State the invariant, attack represented, setup, operation, expected observable behavior, production change that would make the test pass, and why the current implementation should fail. Do not implement the production fix and do not weaken existing assertions.
```

`implementer.md`:

```markdown
Implement only the approved invariant and scope. Preserve ForgeCore authority, fail closed, do not weaken RED tests, do not broaden capabilities or approval, do not add unrelated refactors, and prefer the smallest production change that makes the validated regression test pass. Report the diff and commands run, but do not certify acceptance.
```

`spec-editor.md`:

```markdown
Edit specification text only. Resolve the confirmed contract ambiguity or defect without introducing production implementation into a specification-only task. Preserve explicit authority, precedence, rollback, and fail-closed semantics. Do not change runtime code.
```

`spec-reviewer.md`:

```markdown
Review against the binding requirement independently of the tests. Search every relevant public entry point and contract for a counterexample. Report path, input, expected behavior, actual behavior, and severity for each source-supported defect. The modifying agent is not the acceptance authority.
```

`security-reviewer.md`:

```markdown
Assume caller-controlled fields are hostile. Search for authorization ordering, confused deputy paths, canonicalization gaps, TOCTOU, stale or mutable ownership, injection, secret exposure, unintended mutation, partial failure, concurrency, persistence/restart, traversal, and unsafe defaults. Report source-supported counterexamples only.
```

`verifier.md`:

```markdown
Verify independently using the current commit SHA and required commands. Do not rely on implementer claims or previous runs. Run targeted regression checks, affected package checks, format/lint/static analysis, broader tests required by risk tier, diff inspection, and head-SHA/CI coherence checks. Report evidence and failures; do not merge.
```

`integration-reviewer.md`:

```markdown
Review the whole change after individual findings are addressed. Check interaction between fixes, unresolved review threads, documentation/code mismatch, authority drift, stale evidence, and whether the reviewed and tested SHA equals the current head. Produce merge-readiness evidence only; do not merge.
```

- [ ] **Step 4: Implement `AgentBrief` and deterministic prompt selection**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBrief {
    pub stage: AdversarialStage,
    pub agent_role: AgentRole,
    pub task: crate::Task,
}
```

Use `include_str!("../prompts/adversarial/<name>.md")` for each prompt. Build `Task.context` in this deterministic order:

```text
Finding: <finding_id>
Head SHA: <head_sha>
Source: <source_ref>
Invariant: <invariant>

<stage prompt text>
```

Use task id `<finding_id>:<stage wire name>` and title `<stage wire name>: <finding_id>`. Return `NoAgentForStage(ExternalReview)` when `agent_role` is `None`.

- [ ] **Step 5: Run prompt tests and compile all prompt includes**

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
cargo test -p forge-core external_review_is_a_gate_not_an_agent_prompt -- --nocapture
cargo check -p forge-core
```

Expected: PASS.

- [ ] **Step 6: Commit prompt contracts**

```bash
git add crates/forge-core/prompts/adversarial crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): add adversarial agent briefs"
```

---

### Task 4: Enforce the evidence-gated finding state machine

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `FindingStatus`, `FindingEvidenceKind`, `FindingEvidence`.
- Produces: `AdversarialFinding::transition(next, evidence) -> Result<(), AdversarialDevelopmentError>` and `AdversarialFinding::add_ruling(...)`.

- [ ] **Step 1: Write failing transition tests**

```rust
#[test]
fn finding_cannot_claim_red_without_red_test_evidence() {
    let mut finding = AdversarialFinding::new(
        "id", "sha", "source", AdversarialRiskTier::Tier0,
        FindingCategory::Security, "invariant",
    );
    finding.transition(FindingStatus::Investigating, None).unwrap();
    finding.transition(
        FindingStatus::Confirmed,
        Some(FindingEvidence { kind: FindingEvidenceKind::SourceInspection, reference: "source@sha".into() }),
    ).unwrap();
    assert!(matches!(
        finding.transition(FindingStatus::RedProven, None),
        Err(AdversarialDevelopmentError::MissingEvidence {
            status: FindingStatus::RedProven,
            required: FindingEvidenceKind::RedTest,
        })
    ));
}

#[test]
fn finding_reaches_done_only_through_verified_reviewed_states() {
    let mut finding = AdversarialFinding::new(
        "id", "sha", "source", AdversarialRiskTier::Tier0,
        FindingCategory::Security, "invariant",
    );
    assert!(matches!(
        finding.transition(FindingStatus::Done, None),
        Err(AdversarialDevelopmentError::InvalidTransition { .. })
    ));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
```

Expected: compile failure because transition enforcement does not exist.

- [ ] **Step 3: Implement exact transition and evidence rules**

Allow only:

```text
discovered -> investigating
investigating -> confirmed | rejected_false_positive | stale
confirmed -> red_proven | fix_justified_without_red
red_proven | fix_justified_without_red -> implemented
implemented -> locally_verified
locally_verified -> adversarially_reviewed
adversarially_reviewed -> ci_verified
ci_verified -> external_reviewed
external_reviewed -> done
```

Require evidence on the destination state:

```text
confirmed/rejected_false_positive/stale -> source_inspection
red_proven -> red_test
fix_justified_without_red -> ruling
implemented -> implementation_commit
locally_verified -> local_verification
adversarially_reviewed -> adversarial_review
ci_verified -> ci_run
external_reviewed -> external_review
```

`Done` requires no new evidence because `ExternalReviewed` is already evidence-gated. Reject empty `reference` strings as missing evidence. Push valid evidence into `finding.evidence` before changing status.

`add_ruling` must append a `FindingRuling` and also append `FindingEvidence { kind: Ruling, reference: ruling.evidence.clone() }` so `FixJustifiedWithoutRed` can be audited.

- [ ] **Step 4: Run transition tests and serde round-trip tests**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
cargo test -p forge-core finding_reaches_done_only_through_verified_reviewed_states -- --nocapture
cargo test -p forge-core adversarial_development::tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit state-machine enforcement**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): gate finding state on evidence"
```

---

### Task 5: Add bounded repair-loop strategy

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Produces: `RepairStrategy` and `repair_strategy(round: u32) -> RepairStrategy`.

- [ ] **Step 1: Write the failing breaker test**

```rust
#[test]
fn repair_strategy_changes_agent_after_three_rounds_and_breaks_after_five() {
    assert_eq!(repair_strategy(1), RepairStrategy::ResumeImplementer);
    assert_eq!(repair_strategy(3), RepairStrategy::ResumeImplementer);
    assert_eq!(repair_strategy(4), RepairStrategy::ReplaceImplementer);
    assert_eq!(repair_strategy(5), RepairStrategy::ArchitecturalAdjudication);
    assert_eq!(repair_strategy(6), RepairStrategy::BreakerDisposition);
}
```

- [ ] **Step 2: Run and verify RED**

```bash
cargo test -p forge-core repair_strategy_changes_agent_after_three_rounds_and_breaks_after_five -- --nocapture
```

Expected: compile failure because `RepairStrategy` is missing.

- [ ] **Step 3: Implement the bounded policy**

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

Treat round `0` as pre-first-repair and therefore equivalent to resuming the selected implementer; callers increment after a failed review.

- [ ] **Step 4: Run and verify GREEN**

```bash
cargo test -p forge-core repair_strategy_changes_agent_after_three_rounds_and_breaks_after_five -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit repair-loop policy**

```bash
git add crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): bound adversarial repair loops"
```

---

### Task 6: Export the harness and prove it against the PR #29 acceptance fixture

**Files:**
- Modify: `crates/forge-core/src/lib.rs`
- Create: `crates/forge-core/tests/adversarial_development.rs`
- Modify: `docs/architecture/agent-registry.md`

**Interfaces:**
- Consumes: all public types/functions from Tasks 1-5.
- Produces: stable ForgeCore public API for adversarial development planning.

- [ ] **Step 1: Write the public-API acceptance test first**

Create `crates/forge-core/tests/adversarial_development.rs`:

```rust
use forge_core::{
    build_agent_brief, plan_adversarial_cell, AdversarialFinding, AdversarialRiskTier,
    AdversarialStage, AgentRole, FindingCategory,
};

#[test]
fn pr29_major_authorization_finding_builds_a_full_adversarial_cell() {
    let finding = AdversarialFinding::new(
        "PR29-CR-001",
        "ab76053135f638943bcdf1d3367af3a16a34c53b",
        "PRRT_kwDOTzuzVM6bG4UZ",
        AdversarialRiskTier::Tier0,
        FindingCategory::Authorization,
        "execute_run_test must reject non-RunTest actions before policy, capability, runner parsing, or process delegation",
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let adversary = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::AdversarialAnalysis)
        .unwrap();
    let verifier = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::Verify)
        .unwrap();

    assert_eq!(adversary.agent_role, Some(AgentRole::SecurityReviewer));
    assert_eq!(verifier.agent_role, Some(AgentRole::Tester));
    assert!(adversary.read_only);
    assert!(verifier.decisive);

    let brief = build_agent_brief(&finding, adversary).unwrap();
    assert!(brief.task.context.contains("confused-deputy"));
    assert!(brief.task.context.contains("ab76053135f638943bcdf1d3367af3a16a34c53b"));
}
```

- [ ] **Step 2: Run and verify RED because exports are absent**

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
```

Expected: compile failure from unresolved public imports.

- [ ] **Step 3: Add module declaration and explicit re-exports in `lib.rs`**

Add:

```rust
pub mod adversarial_development;
```

and re-export:

```rust
pub use adversarial_development::{
    build_agent_brief, plan_adversarial_cell, repair_strategy, AdversarialCellPlan,
    AdversarialDevelopmentError, AdversarialFinding, AdversarialRiskTier, AdversarialStage,
    AgentBrief, CellStage, FindingCategory, FindingEvidence, FindingEvidenceKind, FindingRuling,
    FindingStatus, RepairStrategy,
};
```

Do not re-export prompt text as independent authority; prompts remain implementation resources used by `build_agent_brief`.

- [ ] **Step 4: Document logical-role reuse**

Append a short `Adversarial development cells` section to `docs/architecture/agent-registry.md` stating:

```markdown
## Adversarial development cells

Adversarial development does not create privileged agent processes or a second execution authority. A cell is a deterministic plan that assigns existing logical roles to separate investigation, test-design, implementation, review, and verification stages. The modifying role is prohibited from serving as its own decisive reviewer. All executable actions still pass through the existing ForgeCore policy, workspace, approval, and sandbox boundaries.
```

- [ ] **Step 5: Run the acceptance test and full ForgeCore verification**

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

Expected: all commands exit 0. These commands mirror the repository's current Rust CI lane.

- [ ] **Step 6: Inspect the complete diff for authority drift**

Run:

```bash
git diff main...HEAD -- crates/forge-core docs/architecture/agent-registry.md
```

Confirm manually:

```text
- no execution path was added
- no capability or policy rule was weakened
- no background worker/scheduler was added
- no merge/publish operation was added
- prompt text cannot bypass ForgeCore execution policy
- only existing AgentRole values are used
```

- [ ] **Step 7: Commit the public slice**

```bash
git add crates/forge-core/src/lib.rs crates/forge-core/tests/adversarial_development.rs docs/architecture/agent-registry.md
git commit -m "feat(forge-core): expose adversarial development planning"
```

---

## Plan Self-Review Checklist

Before requesting implementation review, verify this plan against the spec:

- [ ] Context Scout is read-only and receives current-head/source context.
- [ ] Adversarial analysis explicitly optimizes for counterexamples rather than confirmation.
- [ ] Test design is distinct from production implementation.
- [ ] Implementer cannot be its own decisive reviewer.
- [ ] Tier-0 and Tier-1 cells include adversarial/security treatment.
- [ ] Tier-3 specification work does not introduce runtime implementation.
- [ ] Finding states require fresh evidence rather than prose claims.
- [ ] The repair loop changes agent context after three failed repair rounds and breaks after five.
- [ ] External review is represented as a gate, not an executable agent.
- [ ] No new scheduler, process model, dependency, merge path, or execution authority was added.
- [ ] PR #29 is represented as an acceptance fixture only; its actual security fix is executed from its own remediation plan/branch.
