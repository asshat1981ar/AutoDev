# AMCX ↔ ForgeCore Bridge v1.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use an isolated worktree and test-driven development. Do not implement production bridge code until the focused tests have been observed failing for the expected missing-feature reason.

**Goal:** Add a pure, non-authorizing mapping layer between existing ForgeCore state and AMCX reference/projection types without creating a second execution, plan, evidence, or verification authority.

**Architecture:** `crates/forge-core/src/amcx_bridge.rs` will contain serializable validated references derived from existing ForgeCore objects. Conversion is one-way projection: host objects remain canonical and are never mutated. The module exposes no executor, policy mutator, grant constructor, filesystem/network/process call, or schema activation function.

**Spec:** `docs/architecture/amcx/reconciliation-v1.1.md`

## Global constraints

- ForgeCore remains sole effect executor.
- `AuthorizationGrant` remains kernel-owned and cannot be constructed by AMCX bridge APIs.
- `ExecPlan`/`ExecutionEnvelope` remain canonical lifecycle state.
- `EvidenceStore`/`VerificationFabric` remain evidence/verdict owners.
- `ContextPack` remains read-only repository evidence.
- Blank/unknown critical source identity fails closed.
- No new dependency is added to `crates/Cargo.toml` for this slice.

---

### Task 1: Define failing projection tests

**Files:**
- Create: `crates/forge-core/tests/amcx_bridge.rs`

**Tests to write before implementation:**

1. `plan_projection_retains_identity_without_mutating_plan`
   - construct an `ExecPlan` with a milestone and checkpoint;
   - project it;
   - assert plan/checkpoint IDs and source kind are retained;
   - assert original plan status and milestone state are unchanged.

2. `evidence_projection_requires_verified_fingerprint`
   - construct valid `Evidence` and project it successfully;
   - tamper with a cloned evidence record;
   - assert projection rejects it rather than copying an unverified digest.

3. `verification_projection_preserves_verdict_as_evidence_only`
   - construct a `VerificationReport` PASS;
   - project it;
   - assert output reports the source verdict and check identifiers;
   - assert the bridge type contains no authorization/approval fields.

4. `context_projection_is_reference_only`
   - construct a `ContextPack`;
   - project using an externally supplied immutable artifact reference/digest;
   - assert output contains query/count/bytes/ref/digest but not copied file contents.

5. `blank_source_identity_fails_closed`
   - attempt each projection with a blank repository/revision/worktree/source ID;
   - assert `AmcxBridgeError::MissingIdentity`.

6. `bridge_surface_has_no_grant_conversion`
   - compile-time API design: bridge exports projection types/functions only; no `From<...> for AuthorizationGrant` and no public method returning `AuthorizationGrant`.

**RED gate:** run `cd crates && cargo test -p forge-core --test amcx_bridge --locked`. Confirm failure because `forge_core::amcx_bridge` does not yet exist.

### Task 2: Implement minimal validated reference types

**Files:**
- Create: `crates/forge-core/src/amcx_bridge.rs`
- Modify: `crates/forge-core/src/lib.rs`

**Types:**

```rust
pub struct AmcxSourceIdentity {
    pub repository: String,
    pub revision: String,
    pub worktree: String,
}

pub struct AmcxPlanRef {
    pub source: AmcxSourceIdentity,
    pub plan_id: String,
    pub checkpoint_id: String,
    pub status: String,
}

pub struct AmcxEvidenceRef {
    pub source: AmcxSourceIdentity,
    pub evidence_id: String,
    pub fingerprint_sha256: String,
}

pub struct AmcxVerificationRef {
    pub source: AmcxSourceIdentity,
    pub verdict: String,
    pub checks: Vec<String>,
}

pub struct AmcxRepositoryContextRef {
    pub source: AmcxSourceIdentity,
    pub artifact_ref: String,
    pub artifact_sha256: String,
    pub query: String,
    pub item_count: usize,
    pub total_bytes: usize,
}

pub enum AmcxBridgeError {
    MissingIdentity,
    InvalidEvidenceFingerprint,
    MissingArtifactReference,
}
```

All types derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` and `deny_unknown_fields` where appropriate.

**Functions:**

```rust
pub fn project_plan(
    source: AmcxSourceIdentity,
    plan: &ExecPlan,
    checkpoint: &PlanCheckpoint,
) -> Result<AmcxPlanRef, AmcxBridgeError>;

pub fn project_evidence(
    source: AmcxSourceIdentity,
    evidence: &Evidence,
) -> Result<AmcxEvidenceRef, AmcxBridgeError>;

pub fn project_verification(
    source: AmcxSourceIdentity,
    report: &VerificationReport,
) -> Result<AmcxVerificationRef, AmcxBridgeError>;

pub fn project_context(
    source: AmcxSourceIdentity,
    pack: &ContextPack,
    artifact_ref: &str,
    artifact_sha256: &str,
) -> Result<AmcxRepositoryContextRef, AmcxBridgeError>;
```

Implementation is validation + immutable projection only.

**GREEN gate:** rerun the focused test until all bridge tests pass.

### Task 3: Regression and authority-boundary verification

Run:

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p forge-core --test amcx_bridge --locked
cargo test --workspace --locked
cd ..
python scripts/check_harness_drift.py
```

Review `git diff` and reject the slice if it:

- adds an execution call to `amcx_bridge.rs`;
- imports/returns `AuthorizationGrant`;
- writes to `ExecPlan`, `EvidenceStore`, `VerificationFabric`, or `ContextPack`;
- copies `ContextPack` file contents into AMCX projection types;
- introduces a new dependency or root manifest.

### Task 4: Independent review

Review the diff against `docs/architecture/amcx/reconciliation-v1.1.md` with emphasis on authority leakage and duplicate source-of-truth creation. Critical/important findings must be corrected and the full regression gate rerun before integration.