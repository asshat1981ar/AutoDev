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
- Fresh sub-agent instances are required at independent review and verification stages even when two stages use the same logical `AgentRole`.
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

    fn pr29_finding() -> AdversarialFinding {
        AdversarialFinding::new(
            "PR29-CR-001",
            29,
            "ab76053135f638943bcdf1d3367af3a16a34c53b",
            FindingSourceType::CodeRabbit,
            "PRRT_kwDOTzuzVM6bG4UZ",
            AdversarialRiskTier::Tier0,
            FindingCategory::Authorization,
            "run_test adapter must reject non-RunTest actions before privileged evaluation",
            vec!["crates/forge-core/src/run_test.rs".into()],
        )
    }

    #[test]
    fn finding_round_trips_with_provenance_and_stable_wire_names() {
        let finding = pr29_finding();
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

- [ ] **Step 3: Implement the exact model and wire helpers**

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
pub enum FindingSourceType {
    #[serde(rename = "coderabbit")]
    CodeRabbit,
    #[serde(rename = "human")]
    Human,
    #[serde(rename = "ci")]
    Ci,
    #[serde(rename = "local")]
    Local,
}

impl FindingSourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CodeRabbit => "coderabbit",
            Self::Human => "human",
            Self::Ci => "ci",
            Self::Local => "local",
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingValidity {
    Unadjudicated,
    Valid,
    PartiallyValid,
    FalsePositive,
    Stale,
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

impl AdversarialStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ContextScout => "context_scout",
            Self::AdversarialAnalysis => "adversarial_analysis",
            Self::TestDesign => "test_design",
            Self::Implement => "implement",
            Self::EditSpecification => "edit_specification",
            Self::SpecReview => "spec_review",
            Self::SecurityReview => "security_review",
            Self::Verify => "verify",
            Self::ExternalReview => "external_review",
            Self::Integrate => "integrate",
        }
    }
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
            finding_id: finding_id.into(),
            pr,
            head_sha: head_sha.into(),
            source_type,
            source_ref: source_ref.into(),
            risk_tier,
            category,
            validity: FindingValidity::Unadjudicated,
            status: FindingStatus::Discovered,
            invariant: invariant.into(),
            allowed_paths,
            adversarial_cases: vec![],
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
- Consumes: Task 1 types and `AgentRole`.
- Produces: `plan_adversarial_cell(&AdversarialFinding) -> Result<AdversarialCellPlan, AdversarialDevelopmentError>` and `AdversarialCellPlan::validate()`.

- [ ] **Step 1: Write exact Tier-0 and Tier-3 tests**

```rust
#[test]
fn tier_zero_uses_full_cell_with_independent_decisive_reviewers() {
    let finding = pr29_finding();
    let plan = plan_adversarial_cell(&finding).unwrap();
    let stages: Vec<_> = plan.stages.iter().map(|stage| stage.stage).collect();
    assert_eq!(
        stages,
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
    );
    let implementer = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::Implement)
        .unwrap();
    assert_eq!(implementer.agent_role, Some(AgentRole::Developer));
    assert!(plan
        .stages
        .iter()
        .filter(|stage| stage.decisive)
        .all(|stage| stage.agent_role != implementer.agent_role));
}

#[test]
fn tier_three_edits_specs_without_self_review() {
    let finding = AdversarialFinding::new(
        "PR41-SPEC-001",
        41,
        "24907feb1c04ead2732c30d385bb325ca3367b8c",
        FindingSourceType::CodeRabbit,
        "spec-review",
        AdversarialRiskTier::Tier3,
        FindingCategory::Specification,
        "managed repositories cannot override mandatory policy",
        vec!["docs/superpowers/specs/2026-08-20-coderabbit-control-plane-design.md".into()],
    );
    let plan = plan_adversarial_cell(&finding).unwrap();
    let editor = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::EditSpecification)
        .unwrap();
    let reviewer = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::SpecReview)
        .unwrap();
    let integrator = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::Integrate)
        .unwrap();
    assert_eq!(editor.agent_role, Some(AgentRole::Architect));
    assert_eq!(reviewer.agent_role, Some(AgentRole::SecurityReviewer));
    assert_eq!(integrator.agent_role, Some(AgentRole::Planner));
    assert_ne!(editor.agent_role, reviewer.agent_role);
}
```

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

Use a private constructor `stage(stage, role, read_only, decisive, depends_on)` so dependencies are explicit. For Tier 0, both `SpecReview` and `SecurityReview` depend on `Implement`; `Verify` depends on both reviewers; `ExternalReview` depends on `Verify`; `Integrate` depends on `ExternalReview`. Apply the analogous serial dependencies to other tiers.

`ExternalReview` has `agent_role: None`, `read_only: true`, `decisive: true`, and cannot produce an agent brief. Reviewer/verifier/integration stages are read-only. `Implement` and `EditSpecification` are modifying stages. `TestDesign` may create/modify tests during execution, but it is not an acceptance authority; fresh sub-agent instances preserve separation from `Verify` even though both use the logical Tester role.

`validate()` finds the production/spec modifying role and returns `ReviewerIndependenceViolated` when any decisive local stage has that same non-`None` role.

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
- Create all nine Markdown prompt files listed in File Structure.
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Consumes: `AdversarialFinding`, `CellStage`, existing `crate::Task`.
- Produces: `AgentBrief` and `build_agent_brief(&AdversarialFinding, &CellStage)`.

- [ ] **Step 1: Write exact RED tests**

```rust
#[test]
fn adversarial_brief_is_falsification_biased_and_self_contained() {
    let finding = pr29_finding();
    let plan = plan_adversarial_cell(&finding).unwrap();
    let stage = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::AdversarialAnalysis)
        .unwrap();
    let brief = build_agent_brief(&finding, stage).unwrap();
    assert_eq!(brief.agent_role, AgentRole::SecurityReviewer);
    assert_eq!(brief.task.id, "PR29-CR-001:adversarial_analysis");
    assert!(brief.task.context.contains("PR29-CR-001"));
    assert!(brief
        .task
        .context
        .contains("ab76053135f638943bcdf1d3367af3a16a34c53b"));
    assert!(brief
        .task
        .context
        .contains("smallest source-supported counterexample"));
    assert!(brief
        .task
        .context
        .contains("crates/forge-core/src/run_test.rs"));
}

#[test]
fn external_review_is_a_gate_not_an_agent_prompt() {
    let finding = pr29_finding();
    let plan = plan_adversarial_cell(&finding).unwrap();
    let stage = plan
        .stages
        .iter()
        .find(|stage| stage.stage == AdversarialStage::ExternalReview)
        .unwrap();
    assert!(matches!(
        build_agent_brief(&finding, stage),
        Err(AdversarialDevelopmentError::NoAgentForStage(
            AdversarialStage::ExternalReview
        ))
    ));
}
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
cargo test -p forge-core external_review_is_a_gate_not_an_agent_prompt -- --nocapture
```

Expected: compile failure because `AgentBrief` and `build_agent_brief` do not exist.

- [ ] **Step 3: Create exact prompt contracts**

`context-scout.md`:

```markdown
You are the Context Scout. Do not modify code. Establish the current head SHA, claimed invariant, affected source and tests, public entry points and callers, relevant authority or persistence interfaces, allowed modification scope, and verification commands. Treat review text as a claim to verify against current source. Return a context brief only.
```

`adversarial-analyst.md`:

```markdown
Do not try to prove the patch is correct. Try to construct the smallest source-supported counterexample proving it is incorrect. When applicable test alternate public entry points, confused-deputy calls, malformed-but-type-valid input, duplicate or stale state, ordering, restart or persistence loss, capability-policy mismatch, partial failure, aliases or path normalization, and fail-open defaults. Report confirmed counterexamples before recommendations. Do not modify production code.
```

`test-designer.md`:

```markdown
Design the smallest real behavioral regression test that distinguishes the violated invariant from the current defect. State the invariant, attack represented, setup, operation, expected observable behavior, production change that would make the test pass, and why the current implementation should fail. Write or propose test-only changes; do not implement the production fix and do not weaken existing assertions.
```

`implementer.md`:

```markdown
Implement only the approved invariant and scope. Preserve ForgeCore authority, fail closed, do not weaken RED tests, do not broaden capabilities or approval, do not add unrelated refactors, and prefer the smallest production change that makes the validated regression test pass. Report the diff and commands run but do not certify acceptance.
```

`spec-editor.md`:

```markdown
Edit specification text only. Resolve the confirmed contract ambiguity or defect without introducing production implementation into a specification-only task. Preserve explicit authority, precedence, rollback, provenance, and fail-closed semantics. Do not change runtime code.
```

`spec-reviewer.md`:

```markdown
Review against the binding requirement independently of the tests. Search every relevant public entry point and contract for a counterexample. Report path, input, expected behavior, actual behavior, and severity for each source-supported defect. The modifying agent is not the acceptance authority.
```

`security-reviewer.md`:

```markdown
Assume caller-controlled fields are hostile. Search for authorization ordering, confused-deputy paths, canonicalization gaps, TOCTOU, stale or mutable ownership, injection, secret exposure, unintended mutation, partial failure, concurrency, persistence or restart defects, traversal, and unsafe defaults when applicable. Report source-supported counterexamples only.
```

`verifier.md`:

```markdown
Verify independently using the current commit SHA and required commands. Do not rely on implementer claims or previous runs. Run targeted regression checks, affected package checks, format, lint or static analysis, broader tests required by risk tier, diff inspection, and head-SHA or CI coherence checks. Report evidence and failures; do not merge.
```

`integration-reviewer.md`:

```markdown
Review the whole change after individual findings are addressed. Check interaction between fixes, unresolved review threads, documentation and code mismatch, authority drift, stale evidence, and whether the reviewed and tested SHA equals the current head. Produce merge-readiness evidence only; do not merge.
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

Use `include_str!("../prompts/adversarial/<prompt-file>.md")` in a private match from stage to prompt. Return `NoAgentForStage(ExternalReview)` before prompt lookup when `agent_role` is `None`.

Build `Task.context` in this exact order:

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

The angle-bracket tokens above describe fields rendered from the supplied finding and stage; implementation must substitute the actual values. Sort a cloned `allowed_paths` before rendering. Use task id `format!("{}:{}", finding.finding_id, stage.stage.as_str())` and title `format!("{}: {}", stage.stage.as_str(), finding.finding_id)`.

- [ ] **Step 5: Verify GREEN and commit**

```bash
cargo test -p forge-core adversarial_brief_is_falsification_biased_and_self_contained -- --nocapture
cargo test -p forge-core external_review_is_a_gate_not_an_agent_prompt -- --nocapture
cargo check -p forge-core
git add crates/forge-core/prompts/adversarial crates/forge-core/src/adversarial_development.rs
git commit -m "feat(forge-core): add adversarial agent briefs"
```

---

### Task 4: Enforce validity adjudication and evidence-gated state transitions

**Files:**
- Modify: `crates/forge-core/src/adversarial_development.rs`

**Interfaces:**
- Produces: `AdversarialFinding::adjudicate`, `AdversarialFinding::transition`, `AdversarialFinding::add_ruling`.

- [ ] **Step 1: Write exact RED tests**

```rust
fn evidence(kind: FindingEvidenceKind, reference: &str) -> FindingEvidence {
    FindingEvidence {
        kind,
        reference: reference.into(),
    }
}

#[test]
fn finding_cannot_claim_red_without_red_test_evidence() {
    let mut finding = pr29_finding();
    finding.transition(FindingStatus::Investigating, None).unwrap();
    finding
        .adjudicate(
            FindingValidity::Valid,
            evidence(FindingEvidenceKind::SourceInspection, "run_test.rs@ab760531"),
        )
        .unwrap();
    assert_eq!(finding.status, FindingStatus::Confirmed);
    assert!(matches!(
        finding.transition(FindingStatus::RedProven, None),
        Err(AdversarialDevelopmentError::MissingEvidence {
            status: FindingStatus::RedProven,
            required: FindingEvidenceKind::RedTest,
        })
    ));
}

#[test]
fn finding_cannot_skip_to_done() {
    let mut finding = pr29_finding();
    assert!(matches!(
        finding.transition(FindingStatus::Done, None),
        Err(AdversarialDevelopmentError::InvalidTransition {
            from: FindingStatus::Discovered,
            to: FindingStatus::Done,
        })
    ));
}

#[test]
fn valid_finding_moves_through_every_evidence_gate() {
    let mut finding = pr29_finding();
    finding.transition(FindingStatus::Investigating, None).unwrap();
    finding
        .adjudicate(
            FindingValidity::Valid,
            evidence(FindingEvidenceKind::SourceInspection, "source@head"),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::RedProven,
            Some(evidence(FindingEvidenceKind::RedTest, "red:test-name")),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::Implemented,
            Some(evidence(FindingEvidenceKind::ImplementationCommit, "commit:repair")),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::LocallyVerified,
            Some(evidence(FindingEvidenceKind::LocalVerification, "cargo-test:pass")),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::AdversariallyReviewed,
            Some(evidence(FindingEvidenceKind::AdversarialReview, "review:pass")),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::CiVerified,
            Some(evidence(FindingEvidenceKind::CiRun, "ci:green")),
        )
        .unwrap();
    finding
        .transition(
            FindingStatus::ExternalReviewed,
            Some(evidence(FindingEvidenceKind::ExternalReview, "coderabbit:clear")),
        )
        .unwrap();
    finding.transition(FindingStatus::Done, None).unwrap();
    assert_eq!(finding.status, FindingStatus::Done);
}
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
cargo test -p forge-core finding_cannot_skip_to_done -- --nocapture
cargo test -p forge-core valid_finding_moves_through_every_evidence_gate -- --nocapture
```

Expected: compile failure before transition methods exist.

- [ ] **Step 3: Implement exact adjudication/transition rules**

`adjudicate(validity, evidence)` is valid only in `Investigating`; require `evidence.kind == SourceInspection` and non-empty reference. Set `validity`, append evidence, then map `Valid | PartiallyValid -> Confirmed`, `FalsePositive -> RejectedFalsePositive`, and `Stale -> Stale`. Reject `Unadjudicated` by returning `InvalidTransition { from: Investigating, to: Investigating }`.

`transition(next, evidence)` allows only:

```text
Discovered -> Investigating
Confirmed -> RedProven | FixJustifiedWithoutRed
RedProven | FixJustifiedWithoutRed -> Implemented
Implemented -> LocallyVerified
LocallyVerified -> AdversariallyReviewed
AdversariallyReviewed -> CiVerified
CiVerified -> ExternalReviewed
ExternalReviewed -> Done
```

Destination evidence requirements:

```text
RedProven -> RedTest
FixJustifiedWithoutRed -> Ruling
Implemented -> ImplementationCommit
LocallyVerified -> LocalVerification
AdversariallyReviewed -> AdversarialReview
CiVerified -> CiRun
ExternalReviewed -> ExternalReview
```

If evidence is supplied, reject an empty reference and append valid evidence before the status transition. For a required evidence kind, accept either the supplied evidence or an already-recorded item of that kind; otherwise return `MissingEvidence`.

`add_ruling(ruling)` requires non-empty `ruling.evidence`, appends the ruling, and appends a `FindingEvidenceKind::Ruling` record using that evidence text.

- [ ] **Step 4: Verify GREEN and commit**

```bash
cargo test -p forge-core finding_cannot_claim_red_without_red_test_evidence -- --nocapture
cargo test -p forge-core finding_cannot_skip_to_done -- --nocapture
cargo test -p forge-core valid_finding_moves_through_every_evidence_gate -- --nocapture
cargo test -p forge-core adversarial_development::tests -- --nocapture
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

```bash
cargo test -p forge-core repair_strategy_replaces_context_and_trips_a_breaker -- --nocapture
```

Expected: compile failure.

- [ ] **Step 2: Implement exact policy**

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

### Task 6: Export the API and prove the PR #29 Tier-0 acceptance fixture

**Files:**
- Modify: `crates/forge-core/src/lib.rs`
- Create: `crates/forge-core/tests/adversarial_development.rs`
- Modify: `docs/architecture/agent-registry.md`

**Interfaces:**
- Produces: stable public planning/evidence API for the outer SDD controller.

- [ ] **Step 1: Write the public-API RED test**

```rust
use forge_core::{
    build_agent_brief, plan_adversarial_cell, AdversarialFinding, AdversarialRiskTier,
    AdversarialStage, AgentRole, FindingCategory, FindingSourceType,
};

#[test]
fn pr29_major_authorization_finding_builds_a_full_adversarial_cell() {
    let finding = AdversarialFinding::new(
        "PR29-CR-001",
        29,
        "ab76053135f638943bcdf1d3367af3a16a34c53b",
        FindingSourceType::CodeRabbit,
        "PRRT_kwDOTzuzVM6bG4UZ",
        AdversarialRiskTier::Tier0,
        FindingCategory::Authorization,
        "run_test_authorized must reject non-RunTest actions before policy, capability, runner parsing, or process delegation",
        vec!["crates/forge-core/src/run_test.rs".into()],
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
    assert!(brief
        .task
        .context
        .contains("smallest source-supported counterexample"));
    assert!(brief
        .task
        .context
        .contains("ab76053135f638943bcdf1d3367af3a16a34c53b"));
}
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
```

Expected: compile failure because public exports are absent.

- [ ] **Step 3: Add module declaration and explicit re-exports**

Add `pub mod adversarial_development;` and re-export:

```rust
pub use adversarial_development::{
    build_agent_brief, plan_adversarial_cell, repair_strategy, AdversarialCellPlan,
    AdversarialDevelopmentError, AdversarialFinding, AdversarialRiskTier, AdversarialStage,
    AgentBrief, CellStage, FindingCategory, FindingEvidence, FindingEvidenceKind, FindingRuling,
    FindingSourceType, FindingStatus, FindingValidity, RepairStrategy,
};
```

Do not export prompt strings as authority.

- [ ] **Step 4: Document logical-role reuse**

Append to `docs/architecture/agent-registry.md`:

```markdown
## Adversarial development cells

Adversarial development cells are deterministic plans over the existing logical `AgentRole` values. They do not create privileged agent processes or a second execution authority. Modifying, decisive-review, and verification stages use fresh sub-agent instances even when two stages share the same logical role. All executable actions continue to pass through the existing ForgeCore policy, workspace, approval, and sandbox boundaries.
```

- [ ] **Step 5: Run acceptance and full Rust verification**

```bash
cargo test -p forge-core --test adversarial_development -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

Expected: every command exits 0. These commands match the current Rust CI lane.

- [ ] **Step 6: Inspect authority drift**

```bash
git diff main...HEAD -- crates/forge-core docs/architecture/agent-registry.md
```

Confirm from the actual diff:

```text
- no execution path was added
- no capability, policy, approval, workspace, or sandbox rule was weakened
- no scheduler or background worker was added
- no merge or publish path was added
- prompt text cannot bypass ForgeCore execution policy
- only existing AgentRole values are used
```

- [ ] **Step 7: Commit**

```bash
git add crates/forge-core/src/lib.rs crates/forge-core/tests/adversarial_development.rs docs/architecture/agent-registry.md
git commit -m "feat(forge-core): expose adversarial development planning"
```

---

## Execution Setup and Durable Ledger

Before Task 1, use `superpowers:using-git-worktrees`, then `superpowers:subagent-driven-development`. The SDD workspace's `progress.md`, briefs, reports, review packages, and command evidence are the durable execution ledger for this plan. The repository model above is serializable planning/evidence data; it does not replace that workspace.

Each task ends with a fresh independent review before the controller records `Task N: complete`. After all six tasks, run one broad branch review and one final full verification pass before using `superpowers:finishing-a-development-branch`.

## Plan Self-Review Checklist

- [ ] Required spec roles are represented through existing logical `AgentRole` values.
- [ ] PR/source/head provenance and allowed-path scope are represented.
- [ ] `coderabbit` and `tier_0` through `tier_4` wire names are explicit and tested.
- [ ] Every test-writing step contains executable test code rather than prose-only instructions.
- [ ] RED commands use one unambiguous test filter each.
- [ ] Context Scout and acceptance-review stages are read-only.
- [ ] Test design is separate from production implementation.
- [ ] Production/spec modifying stages cannot serve as their own decisive reviewers.
- [ ] Tier-0 and Tier-1 include adversarial security treatment.
- [ ] Tier-3 specification work cannot introduce runtime implementation.
- [ ] Finding state transitions require ordered evidence gates.
- [ ] Repair rounds 1-3 resume, round 4 replaces context, round 5 adjudicates architecture, and later rounds require explicit breaker disposition.
- [ ] External review is a gate, not an executable agent.
- [ ] No new dependency, scheduler, process model, merge path, or execution authority is introduced.
- [ ] PR #29 is only the harness acceptance fixture; its security repair executes from its separate remediation plan and branch.
