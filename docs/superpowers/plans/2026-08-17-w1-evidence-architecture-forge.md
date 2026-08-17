# W1 Evidence-to-Architecture Forge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a repository-native, deterministic architecture-evidence domain to ForgeCore that represents normalized research findings, rejects unsupported verified decisions, scores architecture options, and renders a stable evidence-linked Markdown report without live SaaS dependencies.

**Architecture:** Add a new `architecture_evidence` module beside the existing execution-evidence module. The new module owns design/research evidence semantics while reusing ForgeCore's existing `sha256_hex` provenance primitive. Connector payloads are normalized outside this module; W1 accepts only repository-native types. Public types are re-exported from `forge_core::lib` and integration-tested from the crate's public API.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `thiserror`, `chrono`, existing ForgeCore SHA-256 helper, Cargo tests/clippy/rustfmt.

## Global Constraints

- ForgeCore remains the trusted authorization/execution authority; W1 does not execute connector actions.
- No live GitHub, Context7, alphaXiv, Hugging Face, Notion, Linear, database, or other SaaS dependency is added.
- Repository architecture docs remain authoritative for AutoDev design knowledge; external systems may only project/reference it.
- Connector payload shapes must be normalized before entering the W1 domain.
- `repo_observed`, `documented`, `research_supported`, and `experimentally_verified` may satisfy a verified evidence gate.
- `inferred` and `hypothesis` may guide experiments but cannot by themselves satisfy a verified evidence gate.
- Hypothesis-only architecture decisions must remain experimental.
- Report output must be deterministic for identical inputs.
- Existing execution evidence types and behavior remain unchanged.
- No automatic merge or irreversible action is introduced.

---

## File Structure

- Create `crates/forge-core/src/architecture_evidence.rs` — W1 domain types, validation, scoring, evidence gate, and deterministic Markdown renderer.
- Modify `crates/forge-core/src/lib.rs` — expose the new module and re-export its public contracts.
- Create `crates/forge-core/tests/architecture_evidence.rs` — public-API integration tests using normalized fixed fixtures.

No new crate dependency is required.

### Task 1: Add normalized architecture-evidence types and validation

**Files:**
- Create: `crates/forge-core/src/architecture_evidence.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/src/architecture_evidence.rs` unit tests

**Interfaces:**
- Consumes: `crate::evidence::sha256_hex(bytes: &[u8]) -> String`
- Produces:
  - `EvidenceClass`
  - `EvidenceRecord`
  - `ArchitectureAlternative`
  - `Reversibility`
  - `DecisionMaturity`
  - `ArchitectureDecision`
  - `ArchitectureEvidenceError`
  - `EvidenceRecord::new(...) -> Result<EvidenceRecord, ArchitectureEvidenceError>`
  - `EvidenceRecord::can_satisfy_verified_gate() -> bool`
  - `ArchitectureDecision::validate(&BTreeMap<String, EvidenceRecord>) -> Result<(), ArchitectureEvidenceError>`

- [ ] **Step 1: Write failing unit tests for evidence validation and gate classes**

Add these tests at the bottom of the new module before implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn observed_at() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap()
    }

    #[test]
    fn verified_gate_accepts_only_supported_evidence_classes() {
        assert!(EvidenceClass::RepoObserved.can_satisfy_verified_gate());
        assert!(EvidenceClass::Documented.can_satisfy_verified_gate());
        assert!(EvidenceClass::ResearchSupported.can_satisfy_verified_gate());
        assert!(EvidenceClass::ExperimentallyVerified.can_satisfy_verified_gate());
        assert!(!EvidenceClass::Inferred.can_satisfy_verified_gate());
        assert!(!EvidenceClass::Hypothesis.can_satisfy_verified_gate());
    }

    #[test]
    fn evidence_record_rejects_empty_claim() {
        let err = EvidenceRecord::new(
            "ev-1",
            "obj-1",
            "",
            EvidenceClass::Documented,
            "context7",
            "docs://axum/sse",
            observed_at(),
            90,
            "normalized finding",
            "library API changes",
        )
        .unwrap_err();
        assert_eq!(err, ArchitectureEvidenceError::EmptyField("claim"));
    }

    #[test]
    fn evidence_record_rejects_confidence_above_100() {
        let err = EvidenceRecord::new(
            "ev-1",
            "obj-1",
            "Axum supports SSE responses",
            EvidenceClass::Documented,
            "context7",
            "docs://axum/sse",
            observed_at(),
            101,
            "normalized finding",
            "library API changes",
        )
        .unwrap_err();
        assert_eq!(err, ArchitectureEvidenceError::InvalidConfidence(101));
    }
}
```

- [ ] **Step 2: Run the focused test and confirm it fails because the module/types do not exist**

Run:

```bash
cd crates
cargo test -p forge-core architecture_evidence -- --nocapture
```

Expected: compile failure for missing `architecture_evidence` module/types.

- [ ] **Step 3: Implement the domain enums, record, fingerprinting, and validation**

Create `crates/forge-core/src/architecture_evidence.rs` with these public shapes:

```rust
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::sha256_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    RepoObserved,
    Documented,
    ResearchSupported,
    ExperimentallyVerified,
    Inferred,
    Hypothesis,
}

impl EvidenceClass {
    pub fn can_satisfy_verified_gate(self) -> bool {
        matches!(
            self,
            Self::RepoObserved
                | Self::Documented
                | Self::ResearchSupported
                | Self::ExperimentallyVerified
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub objective_id: String,
    pub claim: String,
    pub evidence_class: EvidenceClass,
    pub source_system: String,
    pub source_reference: String,
    pub observed_at: DateTime<Utc>,
    pub confidence: u8,
    pub content_fingerprint: String,
    pub invalidation_condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureAlternative {
    pub name: String,
    pub rationale: String,
    pub rejected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Easy,
    Moderate,
    Difficult,
    Irreversible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMaturity {
    Experimental,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureDecision {
    pub id: String,
    pub objective_id: String,
    pub decision: String,
    pub alternatives: Vec<ArchitectureAlternative>,
    pub contradiction: String,
    pub selected_option: String,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub reversibility: Reversibility,
    pub risks: Vec<String>,
    pub invalidation_conditions: Vec<String>,
    pub maturity: DecisionMaturity,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArchitectureEvidenceError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("confidence must be between 0 and 100, got {0}")]
    InvalidConfidence(u8),
    #[error("decision `{0}` must include at least one rejected alternative")]
    MissingRejectedAlternative(String),
    #[error("decision `{0}` references unknown evidence `{1}`")]
    UnknownEvidenceReference(String, String),
    #[error("verified decision `{0}` has no gate-satisfying evidence")]
    UnsupportedVerifiedDecision(String),
    #[error("decision `{0}` must include at least one invalidation condition")]
    MissingInvalidationCondition(String),
}
```

Implement `EvidenceRecord::new` so it validates all required string fields, rejects `confidence > 100`, computes `content_fingerprint` with `sha256_hex(normalized_content.as_bytes())`, and stores the result. Implement `ArchitectureDecision::validate` so it rejects empty required strings, requires at least one rejected alternative, requires at least one invalidation condition, rejects unknown evidence refs, and for `DecisionMaturity::Verified` requires at least one referenced record whose class can satisfy the verified gate.

- [ ] **Step 4: Export the module from `lib.rs`**

Add:

```rust
pub mod architecture_evidence;
```

and re-export:

```rust
pub use architecture_evidence::{
    ArchitectureAlternative, ArchitectureDecision, ArchitectureEvidenceError, DecisionMaturity,
    EvidenceClass, EvidenceRecord, Reversibility,
};
```

- [ ] **Step 5: Run focused unit tests**

Run:

```bash
cd crates
cargo test -p forge-core architecture_evidence -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit the independently reviewable domain model**

```bash
git add crates/forge-core/src/architecture_evidence.rs crates/forge-core/src/lib.rs
git commit -m "feat: add architecture evidence domain"
```

### Task 2: Add contradiction and architecture-option scoring

**Files:**
- Modify: `crates/forge-core/src/architecture_evidence.rs`
- Test: module unit tests

**Interfaces:**
- Consumes: none outside Task 1 types
- Produces:
  - `ArchitectureCriterion`
  - `CriterionScore`
  - `ArchitectureOption`
  - `ArchitectureOption::total_score() -> i32`
  - `rank_options(&[ArchitectureOption]) -> Vec<ArchitectureOption>`

- [ ] **Step 1: Write failing scoring tests**

```rust
#[test]
fn option_total_score_is_weighted_and_deterministic() {
    let option = ArchitectureOption {
        name: "local-domain".into(),
        description: "Normalize connector findings before trusted boundaries".into(),
        scores: vec![
            CriterionScore { criterion: ArchitectureCriterion::EvidenceStrength, weight: 3, score: 5 },
            CriterionScore { criterion: ArchitectureCriterion::ImplementationCost, weight: -2, score: 2 },
        ],
    };
    assert_eq!(option.total_score(), 11);
}

#[test]
fn rank_options_breaks_equal_scores_by_name() {
    let a = ArchitectureOption { name: "alpha".into(), description: "a".into(), scores: vec![] };
    let b = ArchitectureOption { name: "beta".into(), description: "b".into(), scores: vec![] };
    let ranked = rank_options(&[b, a]);
    assert_eq!(ranked[0].name, "alpha");
    assert_eq!(ranked[1].name, "beta");
}
```

- [ ] **Step 2: Run focused tests and confirm failure**

```bash
cd crates
cargo test -p forge-core option_ -- --nocapture
cargo test -p forge-core rank_options -- --nocapture
```

Expected: compile failure for missing scoring types/functions.

- [ ] **Step 3: Implement scoring types and deterministic ranking**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureCriterion {
    Impact,
    EvidenceStrength,
    Reversibility,
    ImplementationCost,
    OperationalComplexity,
    SecurityRisk,
    ContextBurden,
    Reuse,
    KnowledgeGain,
    FailureIsolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriterionScore {
    pub criterion: ArchitectureCriterion,
    pub weight: i32,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureOption {
    pub name: String,
    pub description: String,
    pub scores: Vec<CriterionScore>,
}

impl ArchitectureOption {
    pub fn total_score(&self) -> i32 {
        self.scores.iter().map(|entry| entry.weight * entry.score).sum()
    }
}

pub fn rank_options(options: &[ArchitectureOption]) -> Vec<ArchitectureOption> {
    let mut ranked = options.to_vec();
    ranked.sort_by(|left, right| {
        right
            .total_score()
            .cmp(&left.total_score())
            .then_with(|| left.name.cmp(&right.name))
    });
    ranked
}
```

- [ ] **Step 4: Re-export scoring interfaces from `lib.rs`**

Extend the `pub use architecture_evidence::{...}` list with:

```rust
rank_options, ArchitectureCriterion, ArchitectureOption, CriterionScore,
```

- [ ] **Step 5: Run tests and clippy for the crate**

```bash
cd crates
cargo test -p forge-core
cargo clippy -p forge-core --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 6: Commit scoring**

```bash
git add crates/forge-core/src/architecture_evidence.rs crates/forge-core/src/lib.rs
git commit -m "feat: add deterministic architecture scoring"
```

### Task 3: Add deterministic Markdown report rendering

**Files:**
- Modify: `crates/forge-core/src/architecture_evidence.rs`
- Test: module unit tests

**Interfaces:**
- Consumes: `EvidenceRecord`, `ArchitectureDecision`, `ArchitectureOption`, `rank_options`
- Produces:
  - `ArchitectureReportInput`
  - `render_architecture_report(&ArchitectureReportInput) -> Result<String, ArchitectureEvidenceError>`

- [ ] **Step 1: Write a failing stable-render test**

```rust
#[test]
fn report_render_is_stable_for_identical_input() {
    let input = fixture_report_input();
    let first = render_architecture_report(&input).unwrap();
    let second = render_architecture_report(&input).unwrap();
    assert_eq!(first, second);
    assert!(first.contains("# Architecture Evidence Report: ConnectorForge W1"));
    assert!(first.contains("## Evidence"));
    assert!(first.contains("## Decisions"));
    assert!(first.contains("## Ranked Options"));
}
```

Add a local `fixture_report_input()` helper using fixed timestamps and fixed IDs, never `Utc::now()`.

- [ ] **Step 2: Run the report test and confirm failure**

```bash
cd crates
cargo test -p forge-core report_render_is_stable_for_identical_input -- --nocapture
```

Expected: compile failure for missing report types/functions.

- [ ] **Step 3: Implement report input and renderer**

Use this shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureReportInput {
    pub objective_id: String,
    pub title: String,
    pub desired_outcome: String,
    pub evidence: Vec<EvidenceRecord>,
    pub decisions: Vec<ArchitectureDecision>,
    pub options: Vec<ArchitectureOption>,
}
```

`render_architecture_report` must:

1. validate every decision against a `BTreeMap<String, EvidenceRecord>` indexed by evidence id;
2. sort evidence by `(source_system, id)` before rendering;
3. sort decisions by `id` before rendering;
4. call `rank_options` for architecture options;
5. render evidence class, confidence, source reference, fingerprint, and invalidation condition;
6. render decision maturity, contradiction, selected option, rationale, reversibility, evidence refs, risks, invalidation conditions, and alternatives;
7. render ranked options with their total score and per-criterion weighted contribution;
8. avoid timestamps generated during rendering so identical input produces byte-identical output.

- [ ] **Step 4: Re-export report interfaces**

Extend `lib.rs` re-exports with:

```rust
render_architecture_report, ArchitectureReportInput,
```

- [ ] **Step 5: Run focused and full crate tests**

```bash
cd crates
cargo test -p forge-core report_ -- --nocapture
cargo test -p forge-core
```

Expected: PASS.

- [ ] **Step 6: Commit report rendering**

```bash
git add crates/forge-core/src/architecture_evidence.rs crates/forge-core/src/lib.rs
git commit -m "feat: render architecture evidence reports"
```

### Task 4: Add normalized connector fixtures and public-API integration gate

**Files:**
- Create: `crates/forge-core/tests/architecture_evidence.rs`

**Interfaces:**
- Consumes only public `forge_core` exports from Tasks 1-3
- Produces: integration coverage proving normalized GitHub, Context7, alphaXiv, and Hugging Face findings can create a stable W1 report without connector SDK dependencies

- [ ] **Step 1: Write the public-API integration fixture**

Create `crates/forge-core/tests/architecture_evidence.rs` with fixed evidence records representing normalized findings:

```rust
use chrono::{TimeZone, Utc};
use forge_core::{
    render_architecture_report, ArchitectureAlternative, ArchitectureCriterion,
    ArchitectureDecision, ArchitectureOption, ArchitectureReportInput, CriterionScore,
    DecisionMaturity, EvidenceClass, EvidenceRecord, Reversibility,
};

fn ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap()
}

fn evidence(
    id: &str,
    class: EvidenceClass,
    source_system: &str,
    source_reference: &str,
    claim: &str,
) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        "obj-w1",
        claim,
        class,
        source_system,
        source_reference,
        ts(),
        90,
        claim,
        "source or repository state materially changes",
    )
    .unwrap()
}

#[test]
fn normalized_connector_findings_render_without_live_connectors() {
    let evidence = vec![
        evidence("ev-github", EvidenceClass::RepoObserved, "github", "repo://forge-core", "ForgeCore already owns trusted execution evidence"),
        evidence("ev-context7", EvidenceClass::Documented, "context7", "docs://serde", "Serde supports deterministic serialization of these local domain types"),
        evidence("ev-alphaxiv", EvidenceClass::ResearchSupported, "alphaxiv", "paper://agent-evidence", "Evidence-linked agent workflows improve auditability"),
        evidence("ev-hf", EvidenceClass::Documented, "hugging-face", "hub://normalized-fixture", "External model ecosystem findings can be normalized before trusted boundaries"),
    ];

    let decision = ArchitectureDecision {
        id: "dec-1".into(),
        objective_id: "obj-w1".into(),
        decision: "Keep connector payloads outside ForgeCore domain contracts".into(),
        alternatives: vec![
            ArchitectureAlternative { name: "embed connector SDK payloads".into(), rationale: "tight coupling".into(), rejected: true },
            ArchitectureAlternative { name: "normalize at orchestration boundary".into(), rationale: "stable trusted types".into(), rejected: false },
        ],
        contradiction: "research breadth vs trusted-boundary stability".into(),
        selected_option: "normalize at orchestration boundary".into(),
        rationale: "keeps ForgeCore deterministic and SaaS-neutral".into(),
        evidence_refs: evidence.iter().map(|item| item.id.clone()).collect(),
        reversibility: Reversibility::Easy,
        risks: vec!["normalizers may lose source-specific detail".into()],
        invalidation_conditions: vec!["ForgeCore requires a connector-native capability that cannot be normalized safely".into()],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "ConnectorForge W1".into(),
        desired_outcome: "Evidence-backed architecture decisions without SaaS coupling".into(),
        evidence,
        decisions: vec![decision],
        options: vec![ArchitectureOption {
            name: "normalized-local-domain".into(),
            description: "Repository-native W1 types".into(),
            scores: vec![
                CriterionScore { criterion: ArchitectureCriterion::EvidenceStrength, weight: 3, score: 5 },
                CriterionScore { criterion: ArchitectureCriterion::ImplementationCost, weight: -1, score: 2 },
            ],
        }],
    };

    let report = render_architecture_report(&input).unwrap();
    assert!(report.contains("github"));
    assert!(report.contains("context7"));
    assert!(report.contains("alphaxiv"));
    assert!(report.contains("hugging-face"));
    assert!(report.contains("normalized-local-domain"));
}
```

- [ ] **Step 2: Add the hypothesis-only rejection integration test**

```rust
#[test]
fn hypothesis_only_decision_cannot_be_verified() {
    let evidence = vec![evidence(
        "ev-hypothesis",
        EvidenceClass::Hypothesis,
        "agent",
        "hypothesis://1",
        "A graph database might improve W1",
    )];

    let decision = ArchitectureDecision {
        id: "dec-hypothesis".into(),
        objective_id: "obj-w1".into(),
        decision: "Adopt graph database".into(),
        alternatives: vec![ArchitectureAlternative {
            name: "keep local domain".into(),
            rationale: "lower complexity".into(),
            rejected: true,
        }],
        contradiction: "query flexibility vs operational complexity".into(),
        selected_option: "graph database".into(),
        rationale: "hypothesis only".into(),
        evidence_refs: vec!["ev-hypothesis".into()],
        reversibility: Reversibility::Moderate,
        risks: vec!["unproven operational cost".into()],
        invalidation_conditions: vec!["local domain satisfies retrieval needs".into()],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Hypothesis gate".into(),
        desired_outcome: "Reject unsupported verification".into(),
        evidence,
        decisions: vec![decision],
        options: vec![],
    };

    assert!(render_architecture_report(&input).is_err());
}
```

- [ ] **Step 3: Run the integration test**

```bash
cd crates
cargo test -p forge-core --test architecture_evidence -- --nocapture
```

Expected: PASS.

- [ ] **Step 4: Run complete Rust verification**

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

Expected: all commands PASS.

- [ ] **Step 5: Commit the public integration gate**

```bash
git add crates/forge-core/tests/architecture_evidence.rs
git commit -m "test: verify ConnectorForge W1 evidence gate"
```

### Task 5: Repository-level verification and PR evidence

**Files:**
- No production file required unless CI exposes a defect.
- Update the implementation PR body with exact GitHub Actions run evidence after the branch is pushed.

**Interfaces:**
- Consumes: repository CI workflow
- Produces: objective evidence that W1 does not regress Rust, Kotlin, or Python gates

- [ ] **Step 1: Compare implementation branch with its base and confirm scope**

Expected changed runtime files are limited to:

```text
crates/forge-core/src/architecture_evidence.rs
crates/forge-core/src/lib.rs
crates/forge-core/tests/architecture_evidence.rs
```

plus the approved ConnectorForge spec/plan documents if the implementation branch includes them.

- [ ] **Step 2: Open a draft implementation PR**

The PR must state:

```text
W1 is connector-neutral: live connectors normalize findings outside ForgeCore.
No external connector SDK or credential is added.
Hypothesis/inferred evidence cannot satisfy verified decisions.
Existing execution evidence and authorization boundaries are unchanged.
```

- [ ] **Step 3: Let repository CI run the authoritative gates**

Required green jobs:

```text
Rust fmt/clippy/build/test
Kotlin build/test/ktlint
Python 3.10 development-fabric checks
Python 3.11 development-fabric checks
```

- [ ] **Step 4: If CI fails, inspect the exact failed job/log, make the smallest corrective patch, and rerun the same gate**

Do not weaken lint, tests, evidence validation, or existing CI rules to obtain green status.

- [ ] **Step 5: Record final evidence in the PR body**

Include the final head SHA, workflow run ID, and passed gates. Keep the PR draft until human review/merge decision.

## Plan Self-Review

- Spec coverage: W1 typed evidence, decision validation, contradiction/option scoring, deterministic Markdown reporting, normalized connector fixtures, hypothesis gate, and repository verification are each mapped to a task.
- Scope: no W2 Linear synchronization, W4 execution integration, W5 memory/evaluation adapters, or W3 visual/prototype integration is included.
- Placeholder scan: no implementation placeholder remains; every task defines concrete interfaces and verification commands.
- Type consistency: all public names used by integration tests are defined in Tasks 1-3 and re-exported from `forge_core`.
- Trust boundary: W1 cannot authorize execution and does not modify existing `Evidence`, `EvidenceStore`, `AuthorizationGrant`, or `VerifiedOrchestrator` behavior.
