# ForgeOS Authority Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first independently testable ForgeOS authority slice: time-sensitive architecture evidence leases, immutable WorkContracts, scoped capability ownership, and resource leases, without changing existing execution adapters to trust these new objects yet.

**Architecture:** Extend the existing connector-neutral `forge-core` trust boundary with four focused domains. Preserve the existing wire-level `action::Capability` and `policy::AuthorizationGrant`; introduce `CapabilityGrant` as kernel-owned ownership metadata around existing capabilities, and keep execution integration out of this first plan. The final slice proves that intent, evidence eligibility, capability authority, and resource ownership can be evaluated deterministically before later action-transaction integration.

**Tech Stack:** Rust, serde, chrono, sha2 through existing `evidence::sha256_hex`, thiserror, existing ForgeCore tests and GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-18-forgeos-agent-computer-design.md`

## Global Constraints

- `EvidenceRecord` remains an immutable observation; do not add mutable `valid`, `stale`, `lease_state`, or `current_status` fields.
- Historical architecture-decision maturity remains independent of current lease eligibility.
- `LeaseAttestation` is never an `AuthorizationGrant` and cannot authorize execution.
- Existing `action::Capability` remains the protocol/wire capability type; do not create a conflicting second enum named `Capability`.
- Existing `policy::AuthorizationGrant` behavior remains compatible during this plan; execution integration is deferred.
- ForgeCore remains connector-neutral and performs no network calls, connector SDK calls, plugin execution, arbitrary scripting, credential access, or ambient clock reads inside the new evaluators.
- All time-sensitive evaluators receive `DateTime<Utc>` explicitly.
- Unknown/malformed policies and unsupported relaxation comparisons fail closed.
- Repository policy may tighten built-in floors freely; relaxation requires explicit repository-backed approval evidence.
- Canonical fingerprints must be deterministic for identical semantic input.
- Every behavioral task follows RED -> minimum GREEN -> focused tests -> fmt/clippy -> commit.
- Do not modify browser, process, filesystem, Git, MCP, or plugin execution behavior in this plan.

## Agentic Tool and Skill Routing

The controller should use the smallest relevant tool/skill set instead of a generic all-purpose worker.

| Stage | Primary tool/skill | Purpose | Required output |
|---|---|---|---|
| Workspace setup | `superpowers:using-git-worktrees` | isolated implementation tree | worktree path + branch |
| Task execution | `superpowers:subagent-driven-development` | fresh bounded implementer per reviewable task | commit + test evidence |
| Behavior implementation | `superpowers:test-driven-development` | enforce RED/GREEN | failing and passing test commands |
| Unexpected failure | `superpowers:systematic-debugging` | evidence-first diagnosis | root cause + minimal repair |
| Current Rust/library docs | Context7 | verify APIs before dependency/API changes | cited API notes in task brief |
| Research uncertainty | alphaXiv + web/official sources | verify current academic/standards assumptions | evidence note, not production authority |
| Repository state/writes | GitHub connector | files, commits, PRs, CI | exact SHA/run IDs |
| Bespoke static analysis | YepCode | auditable one-off transforms/checks when repo-native tooling is insufficient | script + deterministic output |
| Task review | Superpowers task reviewer | spec compliance + code quality | review report |
| Whole-branch review | CodeRabbit when available + `requesting-code-review` | independent defect/security review | findings with file/line evidence |
| Completion | `verification-before-completion` | prevent unsupported success claims | final commands + results |

Tool results are observations. They do not bypass ForgeCore rules or acceptance criteria.

## File Map

### Existing files modified

- `crates/forge-core/src/architecture_evidence.rs` — add public structural validation and current-verification entry point only.
- `crates/forge-core/src/lib.rs` — module declarations and narrowly scoped public exports.
- `docs/architecture/connectorforge-workshops.md` — document historical-vs-current evidence and ForgeOS foundation boundary.

### New production files

- `crates/forge-core/src/architecture_lease.rs` — lease policy algebra, repository overrides, refresh evaluation, attestations.
- `crates/forge-core/src/work_contract.rs` — immutable user-intent contract and deterministic fingerprint.
- `crates/forge-core/src/capability_ownership.rs` — kernel-owned scoped capability grants around `action::Capability`.
- `crates/forge-core/src/resource_lease.rs` — typed resources and deterministic lease validation.

### New focused tests

- `crates/forge-core/tests/architecture_lease_policy.rs`
- `crates/forge-core/tests/architecture_lease_evaluator.rs`
- `crates/forge-core/tests/architecture_lease_gate.rs`
- `crates/forge-core/tests/work_contract.rs`
- `crates/forge-core/tests/capability_ownership.rs`
- `crates/forge-core/tests/resource_lease.rs`

---

### Task 0: Establish implementation baseline and isolated workspace

**Files:**
- Read: `Cargo.toml`
- Read: `crates/forge-core/Cargo.toml`
- Read: `crates/forge-core/src/lib.rs`
- Read: `crates/forge-core/src/action.rs`
- Read: `crates/forge-core/src/policy.rs`
- Read: `crates/forge-core/src/architecture_evidence.rs`
- Read: `crates/forge-core/tests/architecture_evidence.rs`
- Read: `crates/forge-core/tests/architecture_evidence_integrity.rs`

**Interfaces:**
- Consumes: current `main` plus approved ForgeOS spec and this plan.
- Produces: isolated worktree, baseline SHA, baseline test/lint evidence in the SDD ledger.

- [ ] **Step 1: Create or verify isolated worktree**

Use `superpowers:using-git-worktrees`. Base the implementation branch on the current `main`, not on a stale historical PR head. Name the branch `feat/forgeos-authority-foundation` unless an existing compatible implementation branch is already present.

- [ ] **Step 2: Record repository ground truth**

Run:

```bash
git status --short
git rev-parse HEAD
git log -5 --oneline
```

Expected: clean isolated tree and an exact baseline SHA recorded in `.superpowers/sdd/2026-08-18-forgeos-authority-foundation/progress.md`.

- [ ] **Step 3: Run existing focused evidence tests**

```bash
cargo test -p forge-core --test architecture_evidence
cargo test -p forge-core --test architecture_evidence_integrity
```

Expected: PASS before new code is written.

- [ ] **Step 4: Run existing Rust quality baseline**

```bash
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

Expected: PASS. If not, invoke `superpowers:systematic-debugging` and record whether the defect predates this plan.

- [ ] **Step 5: Commit only if workspace setup required a tracked change**

Normally no commit is expected for Task 0.

---

### Task 1: Harden `EvidenceRecord` structural validation

**Files:**
- Modify: `crates/forge-core/src/architecture_evidence.rs`
- Test: `crates/forge-core/tests/architecture_evidence_integrity.rs`

**Interfaces:**
- Consumes: existing `EvidenceRecord` fields and `ArchitectureEvidenceError`.
- Produces: `pub fn EvidenceRecord::validate(&self) -> Result<(), ArchitectureEvidenceError>`; report rendering validates deserialized/public records.

- [ ] **Step 1: Add failing tests for deserialized/public invalid records**

Add tests equivalent to:

```rust
#[test]
fn public_record_with_empty_id_is_rejected() {
    let mut record = valid_record();
    record.id.clear();
    assert_eq!(
        record.validate(),
        Err(ArchitectureEvidenceError::EmptyField("id"))
    );
}

#[test]
fn public_record_with_malformed_fingerprint_is_rejected() {
    let mut record = valid_record();
    record.content_fingerprint = "not-a-sha256".into();
    assert!(matches!(
        record.validate(),
        Err(ArchitectureEvidenceError::InvalidFingerprint(_))
    ));
}

#[test]
fn renderer_rejects_malformed_public_record() {
    let mut input = valid_report_input();
    input.evidence[0].source_system.clear();
    assert_eq!(
        render_architecture_report(&input),
        Err(ArchitectureEvidenceError::EmptyField("source_system"))
    );
}
```

If existing test helpers have different names, keep fixture construction local and explicit rather than refactoring unrelated tests.

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test architecture_evidence_integrity
```

Expected: compile/test failure because `EvidenceRecord::validate` and `InvalidFingerprint` do not yet exist.

- [ ] **Step 3: Implement public structural validation**

Add:

```rust
impl EvidenceRecord {
    pub fn validate(&self) -> Result<(), ArchitectureEvidenceError> {
        required_ref(&self.id, "id")?;
        required_ref(&self.objective_id, "objective_id")?;
        required_ref(&self.claim, "claim")?;
        required_ref(&self.source_system, "source_system")?;
        required_ref(&self.source_reference, "source_reference")?;
        required_ref(&self.invalidation_condition, "invalidation_condition")?;
        if self.confidence > 100 {
            return Err(ArchitectureEvidenceError::InvalidConfidence(self.confidence));
        }
        if self.content_fingerprint.len() != 64
            || !self.content_fingerprint.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(ArchitectureEvidenceError::InvalidFingerprint(
                self.content_fingerprint.clone(),
            ));
        }
        Ok(())
    }
}
```

Add the error variant:

```rust
#[error("invalid SHA-256 fingerprint `{0}`")]
InvalidFingerprint(String),
```

Call `record.validate()?` for every report input record before objective/duplicate checks.

- [ ] **Step 4: Run GREEN and regressions**

```bash
cargo test -p forge-core --test architecture_evidence_integrity
cargo test -p forge-core --test architecture_evidence
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/architecture_evidence.rs crates/forge-core/tests/architecture_evidence_integrity.rs
git commit -m "fix: validate deserialized architecture evidence"
```

---

### Task 2: Add the closed Evidence Lease policy algebra

**Files:**
- Create: `crates/forge-core/src/architecture_lease.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/architecture_lease_policy.rs`

**Interfaces:**
- Consumes: `architecture_evidence::EvidenceRecord`, `evidence::sha256_hex`, `chrono::Duration`.
- Produces: `RiskTier`, `RevalidationMode`, `LeaseRule`, `LeasePolicyDefinition`, `LeasePolicyRegistry`, `EffectivePolicy`, `LeasePolicyError`, deterministic policy fingerprinting.

- [ ] **Step 1: Write failing policy tests**

Cover all of these cases with explicit fixtures:

```rust
#[test]
fn registry_resolves_builtin_policy() { /* assert id/version/rules */ }
#[test]
fn unknown_policy_fails_closed() { /* assert UnknownPolicy */ }
#[test]
fn effective_policy_fingerprint_is_deterministic() { /* same semantic input => same hash */ }
#[test]
fn malformed_max_age_is_rejected() { /* Duration::zero or negative => InvalidMaxAge */ }
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test architecture_lease_policy
```

Expected: compile failure because the module/types do not exist.

- [ ] **Step 3: Implement closed policy types**

Use these public shapes:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskTier { Low, Medium, High }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevalidationMode { AutomaticIfUnchanged, Explicit }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseRule {
    MaxAgeSeconds(i64),
    SourceVersionRequired,
    FingerprintStable,
    RiskAtMost(RiskTier),
    ExplicitRevalidation,
    ExplicitInvalidationAbsent,
    AllOf(Vec<LeaseRule>),
    AnyOf(Vec<LeaseRule>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeasePolicyDefinition {
    pub id: String,
    pub version: u32,
    pub risk_tier: RiskTier,
    pub revalidation: RevalidationMode,
    pub rules: Vec<LeaseRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectivePolicy {
    pub id: String,
    pub version: u32,
    pub risk_tier: RiskTier,
    pub revalidation: RevalidationMode,
    pub rules: Vec<LeaseRule>,
    pub policy_fingerprint: String,
}
```

Use canonical JSON serialization of a private normalized fingerprint input whose rule ordering is deterministic. Hash through existing `sha256_hex`.

`LeasePolicyRegistry` owns a `BTreeMap<String, LeasePolicyDefinition>` and exposes:

```rust
pub fn with_builtins() -> Self;
pub fn insert(&mut self, policy: LeasePolicyDefinition) -> Result<(), LeasePolicyError>;
pub fn resolve(&self, id: &str) -> Result<EffectivePolicy, LeasePolicyError>;
```

No evaluator in this task reads system time or invokes external state.

- [ ] **Step 4: Export only required policy types**

Add `pub mod architecture_lease;` and public re-exports in `lib.rs`.

- [ ] **Step 5: Run GREEN**

```bash
cargo test -p forge-core --test architecture_lease_policy
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 6: Commit**

```bash
git add crates/forge-core/src/architecture_lease.rs crates/forge-core/src/lib.rs crates/forge-core/tests/architecture_lease_policy.rs
git commit -m "feat: add deterministic evidence lease policies"
```

---

### Task 3: Add repository policy tightening and controlled relaxation

**Files:**
- Modify: `crates/forge-core/src/architecture_lease.rs`
- Test: `crates/forge-core/tests/architecture_lease_policy.rs`

**Interfaces:**
- Consumes: `LeasePolicyDefinition`, `EffectivePolicy`.
- Produces: `ApprovalReferenceKind`, `ApprovalReference`, `RepositoryApprovalEvidence`, `PolicyRelaxation`, `RepositoryPolicyOverride`, `compile_effective_policy`.

- [ ] **Step 1: Add failing override tests**

Required cases:

```rust
#[test]
fn repository_tightening_is_allowed_without_relaxation_approval() { /* stricter max age */ }
#[test]
fn silent_relaxation_is_rejected() { /* weaker max age => UnauthorizedRelaxation */ }
#[test]
fn relaxation_requires_nonempty_rationale() { /* MissingRelaxationRationale */ }
#[test]
fn relaxation_requires_matching_repository_approval() { /* MissingApprovalEvidence */ }
#[test]
fn unsupported_comparison_fails_closed() { /* UnsupportedPolicyComparison */ }
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test architecture_lease_policy
```

- [ ] **Step 3: Implement repository-backed approval types**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalReferenceKind { Commit, PullRequest, Adr }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalReference {
    pub kind: ApprovalReferenceKind,
    pub repository: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryApprovalEvidence {
    pub approval: ApprovalReference,
    pub normalized_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRelaxation {
    pub rationale: String,
    pub approval: ApprovalReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPolicyOverride {
    pub rules: Vec<LeaseRule>,
    pub allow_relaxation: bool,
    pub relaxation: Option<PolicyRelaxation>,
}
```

Implement:

```rust
pub fn compile_effective_policy(
    base: &LeasePolicyDefinition,
    override_policy: Option<&RepositoryPolicyOverride>,
    approval_evidence: &[RepositoryApprovalEvidence],
) -> Result<EffectivePolicy, LeasePolicyError>
```

Comparison scope in this slice is deliberately closed: compare supported scalar constraints (`MaxAgeSeconds`, `RiskAtMost`, explicit-revalidation presence). If rule shapes cannot be proven equal-or-stricter, return `UnsupportedPolicyComparison`; do not attempt theorem proving.

- [ ] **Step 4: Run GREEN**

```bash
cargo test -p forge-core --test architecture_lease_policy
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/architecture_lease.rs crates/forge-core/tests/architecture_lease_policy.rs
git commit -m "feat: gate lease policy relaxation by repository approval"
```

---

### Task 4: Add deterministic refresh evaluation and immutable attestations

**Files:**
- Modify: `crates/forge-core/src/architecture_lease.rs`
- Test: `crates/forge-core/tests/architecture_lease_evaluator.rs`

**Interfaces:**
- Consumes: `EvidenceRecord`, `EffectivePolicy`, explicit `evaluated_at`.
- Produces: `RefreshProposal`, `LeaseEvaluationStatus`, `LeaseEvaluationReason`, `LeaseEvaluation`, `LeaseAttestation`, `evaluate_lease`, `LeaseAttestation::validate`.

- [ ] **Step 1: Add failing evaluator tests**

Use fixed UTC timestamps and cover:

```rust
fresh evidence
exact expiry (`evaluated_at == valid_until`) is stale
one instant before expiry is valid
explicit invalidation wins over freshness
source version change requires revalidation
fingerprint change under same source version requires revalidation
low-risk unchanged automatic renewal may pass when policy permits
medium/high-risk material change requires explicit review
policy fingerprint change invalidates prior attestation
malformed proposal is rejected
deterministic identical input => identical evaluation
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test architecture_lease_evaluator
```

- [ ] **Step 3: Implement evaluation types**

Use these stable public shapes:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshProposal {
    pub evidence_id: String,
    pub objective_id: String,
    pub source_version: String,
    pub content_fingerprint: String,
    pub explicitly_invalidated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseEvaluationStatus { Valid, ReviewRequired, Rejected }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseEvaluationReason {
    Fresh,
    Expired,
    ExplicitlyInvalidated,
    SourceVersionChanged,
    FingerprintChanged,
    PolicyChanged,
    ExplicitRevalidationRequired,
    MalformedProposal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEvaluation {
    pub status: LeaseEvaluationStatus,
    pub reasons: Vec<LeaseEvaluationReason>,
    pub evaluated_at: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}
```

Evaluator signature:

```rust
pub fn evaluate_lease(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    proposal: &RefreshProposal,
    prior_attestation: Option<&LeaseAttestation>,
    evaluated_at: DateTime<Utc>,
) -> Result<LeaseEvaluation, LeasePolicyError>
```

Never call `Utc::now()` inside this function.

- [ ] **Step 4: Implement immutable attestation issuance**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseAttestation {
    pub evidence_id: String,
    pub objective_id: String,
    pub evidence_fingerprint: String,
    pub policy_id: String,
    pub policy_version: u32,
    pub policy_fingerprint: String,
    pub source_version: String,
    pub evaluated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub risk_tier: RiskTier,
    pub attestation_fingerprint: String,
}
```

Expose:

```rust
pub fn attest(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    proposal: &RefreshProposal,
    evaluation: &LeaseEvaluation,
) -> Result<LeaseAttestation, LeasePolicyError>;
```

Only `LeaseEvaluationStatus::Valid` can be attested. `LeaseAttestation::validate()` recomputes its deterministic fingerprint and rejects malformed binding fields.

- [ ] **Step 5: Run GREEN**

```bash
cargo test -p forge-core --test architecture_lease_evaluator
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 6: Commit**

```bash
git add crates/forge-core/src/architecture_lease.rs crates/forge-core/tests/architecture_lease_evaluator.rs
git commit -m "feat: evaluate and attest architecture evidence leases"
```

---

### Task 5: Add lease-aware current architecture verification

**Files:**
- Modify: `crates/forge-core/src/architecture_evidence.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/architecture_lease_gate.rs`

**Interfaces:**
- Consumes: `ArchitectureDecision`, evidence map, attestation map, current effective-policy map, explicit time.
- Produces: `CurrentVerificationStatus`, `CurrentVerificationResult`, `evaluate_current_verification`.

- [ ] **Step 1: Write failing current-gate tests**

Required cases:

```rust
supported evidence class + exact current attestation => eligible
supported evidence + expired attestation => ineligible
Hypothesis + current attestation => still cannot independently satisfy verified gate
historically Verified decision remains structurally Verified after lease expiry
renewed attestation restores current eligibility
attestation with wrong evidence fingerprint => ineligible
attestation with stale policy fingerprint => ineligible
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test architecture_lease_gate
```

- [ ] **Step 3: Implement current gate**

Use:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentVerificationStatus { Eligible, Ineligible }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentVerificationResult {
    pub status: CurrentVerificationStatus,
    pub eligible_evidence_ids: Vec<String>,
    pub ineligible_evidence_ids: Vec<String>,
}

pub fn evaluate_current_verification(
    decision: &ArchitectureDecision,
    evidence: &BTreeMap<String, EvidenceRecord>,
    attestations: &BTreeMap<String, LeaseAttestation>,
    policies: &BTreeMap<String, EffectivePolicy>,
    evaluated_at: DateTime<Utc>,
) -> Result<CurrentVerificationResult, ArchitectureEvidenceError>
```

For each referenced record require:

```text
record.validate()
record.evidence_class.can_satisfy_verified_gate()
matching attestation exists and validates
attestation.evidence_id == record.id
attestation.objective_id == record.objective_id
attestation.evidence_fingerprint == record.content_fingerprint
matching current EffectivePolicy exists
attestation.policy_fingerprint == current policy fingerprint
evaluated_at < attestation.valid_until
```

Do not rewrite `decision.maturity` or historical report rendering.

- [ ] **Step 4: Run GREEN + historical regressions**

```bash
cargo test -p forge-core --test architecture_lease_gate
cargo test -p forge-core --test architecture_evidence
cargo test -p forge-core --test architecture_evidence_integrity
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/architecture_evidence.rs crates/forge-core/src/lib.rs crates/forge-core/tests/architecture_lease_gate.rs
git commit -m "feat: gate current architecture verification by leases"
```

---

### Task 6: Add immutable WorkContracts

**Files:**
- Create: `crates/forge-core/src/work_contract.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/work_contract.rs`

**Interfaces:**
- Consumes: existing `action::Capability`, `evidence::sha256_hex`.
- Produces: `WorkContract`, `WorkContractRevision`, `WorkContractError`, deterministic fingerprint and validation.

- [ ] **Step 1: Write failing WorkContract tests**

Cover:

```rust
empty id/objective rejected
revision must be >= 1
empty allowed outcome rejected when no verification criterion exists
forbidden outcome cannot duplicate allowed outcome
fingerprint deterministic despite input collection insertion order
mutating objective changes fingerprint
stale revision comparison is rejected
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test work_contract
```

- [ ] **Step 3: Implement focused contract types**

Use ordered collections to make semantic ordering explicit:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkContract {
    pub id: String,
    pub revision: u32,
    pub objective: String,
    pub allowed_outcomes: BTreeSet<String>,
    pub forbidden_outcomes: BTreeSet<String>,
    pub required_evidence: BTreeSet<String>,
    pub permitted_capabilities: BTreeSet<Capability>,
    pub resource_scope: BTreeSet<String>,
    pub approval_threshold: RiskLevel,
    pub max_actions: u32,
    pub termination_conditions: BTreeSet<String>,
    pub verification_criteria: BTreeSet<String>,
    pub contract_fingerprint: String,
}
```

Because `Capability` currently lacks `Ord`, either derive `PartialOrd, Ord` for the existing enum if its `Unknown(String)` representation supports stable ordering without wire changes, or store capabilities in a sorted `Vec<Capability>` canonicalized by `Capability::as_str()`. Prefer the latter if deriving `Ord` would create unrelated API surface.

Expose:

```rust
impl WorkContract {
    pub fn new(/* explicit fields except fingerprint */) -> Result<Self, WorkContractError>;
    pub fn validate(&self) -> Result<(), WorkContractError>;
    pub fn is_revision_current(&self, revision: u32, fingerprint: &str) -> bool;
}
```

`WorkContractRevision` records `prior_fingerprint`, new contract, rationale, and approval reference when the revision broadens authority.

- [ ] **Step 4: Run GREEN**

```bash
cargo test -p forge-core --test work_contract
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/work_contract.rs crates/forge-core/src/lib.rs crates/forge-core/tests/work_contract.rs
git commit -m "feat: add immutable ForgeOS work contracts"
```

---

### Task 7: Add scoped capability ownership and strict-subset delegation

**Files:**
- Create: `crates/forge-core/src/capability_ownership.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/capability_ownership.rs`

**Interfaces:**
- Consumes: existing `action::Capability`, `WorkContract`, `EffectivePolicy`, explicit time.
- Produces: `PrincipalId`, `CapabilityScope`, `CapabilityGrant`, `CapabilityGrantError`, `issue_capability_grant`, `delegate_capability_grant`, `validate_capability_request`.

- [ ] **Step 1: Write failing grant/delegation tests**

Cover:

```rust
capability absent from WorkContract is denied
grant binds exact WorkContract fingerprint
grant at exact valid_until is expired
revoked grant is unusable
max_uses zero is unusable
child capability not present in parent is rejected
child resource scope broader than parent is rejected
child expiry after parent expiry is rejected
child max_uses greater than parent remaining budget is rejected
valid strict subset delegation succeeds
identical grant inputs create deterministic binding fingerprint when grant_id is fixed
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test capability_ownership
```

- [ ] **Step 3: Implement ownership metadata around existing Capability**

Use:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PrincipalId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityScope {
    pub resources: BTreeSet<String>,
    pub argument_constraints: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub grant_id: String,
    pub objective_id: String,
    pub work_contract_id: String,
    pub work_contract_fingerprint: String,
    pub principal: PrincipalId,
    pub capability: Capability,
    pub scope: CapabilityScope,
    pub issued_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub max_uses: u32,
    pub uses_consumed: u32,
    pub parent_grant_id: Option<String>,
    pub revoked: bool,
    pub policy_fingerprint: String,
    pub evidence_fingerprints: BTreeSet<String>,
    pub grant_fingerprint: String,
}
```

Expose:

```rust
pub fn issue_capability_grant(
    contract: &WorkContract,
    principal: PrincipalId,
    capability: Capability,
    scope: CapabilityScope,
    issued_at: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    max_uses: u32,
    policy: &EffectivePolicy,
    evidence_fingerprints: BTreeSet<String>,
    grant_id: String,
) -> Result<CapabilityGrant, CapabilityGrantError>;

pub fn delegate_capability_grant(
    parent: &CapabilityGrant,
    child_principal: PrincipalId,
    child_scope: CapabilityScope,
    issued_at: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    max_uses: u32,
    child_grant_id: String,
) -> Result<CapabilityGrant, CapabilityGrantError>;
```

Delegation must preserve `capability` exactly in this first slice; narrowing is through resource/argument scope, expiry, and usage budget. Capability-family transformations are deferred until a formal lattice is specified.

- [ ] **Step 4: Implement request validation without changing execution**

```rust
pub fn validate_capability_request(
    grant: &CapabilityGrant,
    contract: &WorkContract,
    requested: &Capability,
    resource: &str,
    evaluated_at: DateTime<Utc>,
) -> Result<(), CapabilityGrantError>
```

Require exact contract ID/fingerprint, matching capability, current time (`evaluated_at < valid_until`), not revoked, remaining use budget, and resource inside grant scope.

- [ ] **Step 5: Run GREEN**

```bash
cargo test -p forge-core --test capability_ownership
cargo test -p forge-core --test work_contract
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 6: Commit**

```bash
git add crates/forge-core/src/capability_ownership.rs crates/forge-core/src/lib.rs crates/forge-core/tests/capability_ownership.rs
git commit -m "feat: add scoped capability ownership"
```

---

### Task 8: Add typed ResourceLease ownership

**Files:**
- Create: `crates/forge-core/src/resource_lease.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/resource_lease.rs`

**Interfaces:**
- Consumes: `PrincipalId`, explicit time.
- Produces: `ResourceId`, `ResourceAccess`, `ResourceLease`, `ResourceLeaseError`, acquisition/conflict validation helpers.

- [ ] **Step 1: Write failing resource-lease tests**

Cover:

```rust
exclusive writer blocks second writer
exclusive writer blocks new reader
shared readers may coexist
writer cannot acquire while readers active
expired lease does not conflict
revoked lease does not authorize access
wrong principal rejected
exact valid_until is expired
resource mismatch rejected
```

- [ ] **Step 2: Run RED**

```bash
cargo test -p forge-core --test resource_lease
```

- [ ] **Step 3: Implement typed resource identity and lease**

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceId {
    BrowserSession(String),
    BrowserTab(String),
    FilesystemPath(String),
    GitWorktree(String),
    Process(u32),
    TerminalSession(String),
    NetworkOrigin(String),
    SecretHandle(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceAccess { SharedRead, ExclusiveWrite }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLease {
    pub lease_id: String,
    pub resource: ResourceId,
    pub principal: PrincipalId,
    pub access: ResourceAccess,
    pub issued_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub revoked: bool,
}
```

Expose pure functions:

```rust
pub fn validate_resource_lease(
    lease: &ResourceLease,
    principal: &PrincipalId,
    resource: &ResourceId,
    required_access: ResourceAccess,
    evaluated_at: DateTime<Utc>,
) -> Result<(), ResourceLeaseError>;

pub fn conflicts_with_active(
    candidate: &ResourceLease,
    active: &[ResourceLease],
    evaluated_at: DateTime<Utc>,
) -> Result<(), ResourceLeaseError>;
```

Conflict semantics:

```text
SharedRead + SharedRead => allowed
ExclusiveWrite + any active same-resource lease => conflict
SharedRead + active ExclusiveWrite => conflict
expired or revoked existing lease => ignored for conflict
```

- [ ] **Step 4: Run GREEN**

```bash
cargo test -p forge-core --test resource_lease
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/forge-core/src/resource_lease.rs crates/forge-core/src/lib.rs crates/forge-core/tests/resource_lease.rs
git commit -m "feat: add ForgeOS resource leases"
```

---

### Task 9: Integrate foundation invariants without enabling new execution authority

**Files:**
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/capability_ownership.rs`
- Test: `crates/forge-core/tests/resource_lease.rs`
- Test: `crates/forge-core/tests/architecture_lease_gate.rs`
- Modify: `docs/architecture/connectorforge-workshops.md`

**Interfaces:**
- Consumes: all Task 1-8 public APIs.
- Produces: stable ForgeCore exports and documented boundary for the next action-transaction plan.

- [ ] **Step 1: Add an integration test proving separation of concerns**

Add one test (to the most appropriate focused test file) demonstrating:

```rust
// A current LeaseAttestation proves evidence eligibility.
// A CapabilityGrant proves bounded capability ownership.
// A ResourceLease proves current resource ownership.
// None of these values can be passed as policy::AuthorizationGrant,
// and this plan does not modify execute()/ExecutableAction to accept them.
```

Make the behavioral assertions concrete: construct all three valid objects, validate them through their own APIs, then assert that existing `ExecutableAction::new(...)` still contains `AuthorizationGrant::none()` and existing approval behavior remains unchanged.

- [ ] **Step 2: Run all focused foundation tests**

```bash
cargo test -p forge-core --test architecture_evidence
cargo test -p forge-core --test architecture_evidence_integrity
cargo test -p forge-core --test architecture_lease_policy
cargo test -p forge-core --test architecture_lease_evaluator
cargo test -p forge-core --test architecture_lease_gate
cargo test -p forge-core --test work_contract
cargo test -p forge-core --test capability_ownership
cargo test -p forge-core --test resource_lease
```

Expected: PASS.

- [ ] **Step 3: Run legacy execution/policy regressions**

```bash
cargo test -p forge-core --test execute
cargo test -p forge-core --test adversarial
cargo test -p forge-core --test orchestrator
cargo test -p forge-core --test runtime
```

Expected: PASS. This is the evidence that the new foundation has not silently changed execution authority.

- [ ] **Step 4: Update architecture documentation**

Document:

```text
Historical architecture evidence != current lease eligibility
LeaseAttestation != CapabilityGrant
CapabilityGrant != AuthorizationGrant
ResourceLease != execution authorization
ForgeOS foundation APIs are pure trust inputs only
Action-transaction integration is the next separate plan
```

- [ ] **Step 5: Run full Rust verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 6: Run independent review**

Use the Superpowers task/whole-branch review flow. If CodeRabbit is connected and executable, run it as a second independent reviewer. Do not claim CodeRabbit findings if the tool did not actually run.

Review especially:

```text
authority amplification
fingerprint ambiguity
expiry boundary errors
ambient time reads
unvalidated deserialized state
capability/resource scope widening
accidental changes to existing AuthorizationGrant semantics
```

- [ ] **Step 7: Commit documentation/integration evidence**

```bash
git add crates/forge-core/src/lib.rs crates/forge-core/tests docs/architecture/connectorforge-workshops.md
git commit -m "docs: define ForgeOS authority foundation boundary"
```

- [ ] **Step 8: Push implementation branch and capture CI**

Pushing is an external side effect; stop for confirmation immediately before the push if execution norms require it.

After push, record the exact head SHA and GitHub Actions run IDs/results. Do not mark the plan complete until the required Rust checks are green.

---

## Execution DAG and Parallelism

The authority foundation is intentionally dependency-heavy at the beginning, then branches safely:

```text
Task 0 baseline
   |
Task 1 EvidenceRecord validation
   |
Task 2 Lease policy algebra
   |
Task 3 Policy override/relaxation
   |
Task 4 Refresh evaluator + attestation
   |
Task 5 Current evidence gate
   |
Task 6 WorkContract
   |
   +-------------------------+
   |                         |
Task 7 Capability ownership  Task 8 ResourceLease
   |                         |
   +------------+------------+
                |
             Task 9
```

### Parallel workstream rule

Tasks 7 and 8 are the first implementation tasks that may execute in parallel because Task 8 consumes only `PrincipalId` from Task 7. To enable true parallelism without shared-file conflict:

1. Task 7 first lands a tiny interface-freeze commit containing `PrincipalId` and public module export, or the controller places `PrincipalId` in a pre-agreed neutral module before dispatch.
2. Capability worker owns `capability_ownership.rs` + `capability_ownership.rs` test.
3. Resource worker owns `resource_lease.rs` + `resource_lease.rs` test.
4. Neither parallel worker edits `lib.rs`; the integrator performs exports after both commits.
5. Workers use separate git worktrees.
6. Root/integrator resolves final exports and runs cross-tests.

Do not parallelize Tasks 2-5: they share the lease algebra and should remain one coherent TDD sequence.

## Recommended Agent Roles

### Authority Kernel implementer

Owns Tasks 1-5. Model tier: standard/high because deterministic security-policy semantics and fingerprints require judgment.

### Intent Contract implementer

Owns Task 6. Model tier: standard.

### Capability worker

Owns Task 7 after interface freeze. Model tier: standard.

### Resource worker

Owns Task 8 after interface freeze. Model tier: standard/fast because the semantics are mechanically specified.

### Security reviewer

Read-only independent review after Tasks 5, 7, 8. Model tier: highest available. Must look for authority amplification and fail-open behavior.

### Integration verifier

Owns Task 9 integration tests and final branch verification; does not redesign interfaces unless a proven defect requires a ruling.

## Subsequent Plans After This Foundation

Do not absorb these into the current plan:

1. `forgeos-action-transaction` — `ProposedAction`, exact-action authorization, execution receipts, independent verification, state-transition commit/rollback.
2. `forgeos-runtime-resource-scheduler` — async DAG scheduler, resource lease coordinator, cancellation/failure propagation.
3. `forgeos-computer-adapters` — typed browser/filesystem/process/Git/network adapters.
4. `forgeos-wasi-sandbox` — WIT interfaces and capability-linked Wasmtime components.
5. `forgeos-trajectory-guard` — composed-workflow invariants and repair/replan.
6. `forgeos-agent-orchestration` — typed AgentContracts, bounded delegation, model/tool routing, isolated parallel worktrees.
7. `forgeos-evaluation-harness` — adversarial and long-horizon evaluation, reliability/safety metrics.

## Self-Review

### Spec coverage in this plan

Covered now:

- immutable architecture evidence validation;
- current evidence leases and attestations;
- policy floors and controlled relaxation;
- deterministic explicit-time evaluation;
- immutable WorkContract foundation;
- compatibility with existing `action::Capability`;
- scoped capability ownership/delegation;
- typed resource leases;
- authority-type separation;
- TDD, lint, regression, and review gates;
- agentic skill/tool routing and safe parallel workstream rules.

Deferred intentionally to later independent specs/plans:

- action transaction execution integration;
- browser/computer adapters;
- runtime scheduler;
- Wasmtime/WASI sandbox;
- trajectory guard;
- memory/learning integration;
- long-horizon evaluation harness.

### Placeholder scan

The plan contains no `TBD`, `TODO`, unspecified testing instruction, or open implementation placeholder. Where an existing test helper name cannot be known without execution-time repository inspection, the task explicitly requires local explicit fixtures rather than an unspecified helper.

### Type consistency

- `EvidenceRecord::validate` originates Task 1 and is consumed Tasks 4-5.
- lease policy types originate Task 2, override compiler Task 3, evaluator/attestation Task 4, current gate Task 5.
- `WorkContract` originates Task 6 and is consumed Task 7.
- existing `action::Capability` is reused throughout; no conflicting `Capability` type is created.
- `PrincipalId` originates Task 7 and is consumed Task 8.
- `CapabilityGrant` and `ResourceLease` remain distinct from existing `AuthorizationGrant` through Task 9.
