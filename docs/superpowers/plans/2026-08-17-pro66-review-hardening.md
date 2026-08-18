# PRO-66 Review Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the four merge-blocking authority/durability defects found in PR #17 while preserving the existing ForgeCore execution architecture.

**Architecture:** Keep `VerifiedOrchestrator` and `DevelopmentLoop` as the sole execution path. Harden values before they enter that path: isolate trusted persistence from the workspace, normalize model risk to a trusted minimum, persist the `EvidenceStore` with objective snapshots, and scope approval grants to objective/task identity.

**Tech Stack:** Rust 1.97-compatible workspace, serde/serde_json, Axum server, ForgeCore, GitHub Actions CI.

## Global Constraints

- No new database, runtime, orchestrator, or policy engine.
- Model/client fields remain untrusted intent.
- Trusted state must not be writable from the configured execution workspace.
- Model-declared risk may increase authorization requirements but may never lower the kernel minimum.
- Public HTTP API continues to expose objective projections only and no approval endpoint.
- Backward-compatible deserialization for existing objective snapshots is required.

---

### Task 1: Trusted state/workspace separation

**Files:**
- Modify: `crates/autodev-server/src/main.rs`
- Modify: `crates/autodev-server/src/lib.rs`
- Test: `crates/autodev-server/tests/objective_api.rs`

**Interfaces:**
- Produces: `pub fn validate_control_plane_paths(workspace: &Workspace, state_dir: &Path) -> Result<PathBuf, ControlPlanePathError>`
- Produces: `pub fn default_state_dir(workspace: &Workspace) -> PathBuf`

- [ ] **Step 1: Write failing tests** proving an equal/descendant state path is rejected and a sibling path is accepted.
- [ ] **Step 2: Push tests and verify GitHub Actions Rust job fails for the missing API.**
- [ ] **Step 3: Implement canonical path validation and an outside-workspace default.**
- [ ] **Step 4: Wire `main.rs` to validate before opening `FileObjectiveStore`.**
- [ ] **Step 5: Verify targeted tests and full Rust gates pass.**

### Task 2: Trusted risk floor

**Files:**
- Modify: `crates/forge-core/src/policy.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Modify: `crates/forge-core/src/action_proposal.rs`
- Modify: `crates/autodev-server/src/runner.rs`
- Test: `crates/forge-core/tests/action_proposal.rs`
- Test: `crates/autodev-server/tests/objective_runner.rs`

**Interfaces:**
- Produces: `pub fn minimum_risk_for_action(action: &AgentAction) -> RiskLevel`
- Produces: `pub fn effective_risk_for_action(action: &AgentAction) -> RiskLevel`

- [ ] **Step 1: Write failing tests** where model/custom proposer labels `write_file`, `git checkpoint`, and `git rollback` as low risk.
- [ ] **Step 2: Verify RED in GitHub Actions.**
- [ ] **Step 3: Implement trusted minimum/effective-risk helpers.**
- [ ] **Step 4: Rewrite proposal risk before policy decision and recompute in runner binding.**
- [ ] **Step 5: Verify low-risk reads remain low while mutating operations require approval or fail profile policy.**

### Task 3: Durable evidence records

**Files:**
- Modify: `crates/forge-core/src/evidence.rs`
- Modify: `crates/forge-core/src/development_loop.rs`
- Modify: `crates/autodev-server/src/store.rs`
- Modify: `crates/autodev-server/src/runner.rs`
- Test: `crates/autodev-server/tests/objective_runner.rs`

**Interfaces:**
- Produces: serializable `EvidenceStore`
- Produces: `DevelopmentLoop::with_evidence(verification: VerificationFabric, evidence: EvidenceStore) -> Self`
- `ObjectiveSnapshot` gains `#[serde(default)] pub evidence: EvidenceStore`

- [ ] **Step 1: Write failing tests** that resolve `latest_evidence_ref` from persisted evidence after a second runner call and after `FileObjectiveStore` reopen.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Make `EvidenceStore` serializable and restore it into each reconstructed `DevelopmentLoop`.**
- [ ] **Step 4: Copy updated evidence back into the snapshot after orchestration.**
- [ ] **Step 5: Verify backward-compatible snapshot loading and durable evidence lookup.**

### Task 4: Scoped approval grants

**Files:**
- Modify: `crates/autodev-server/src/runner.rs`
- Modify: `crates/autodev-server/src/lib.rs`
- Test: `crates/autodev-server/tests/objective_runner.rs`

**Interfaces:**
- Produces: `ObjectiveApprovalGrant::new(objective_id, task_id, approval_ref) -> Result<Self, RunnerError>`
- Changes: `ObjectiveRunner::resume_approved(&self, grant: &ObjectiveApprovalGrant) -> Result<ObjectiveView, RunnerError>`

- [ ] **Step 1: Write failing tests** for mismatched objective and task grants and an empty reference.
- [ ] **Step 2: Verify RED.**
- [ ] **Step 3: Implement scoped grant validation and update the existing positive approval-resume test.**
- [ ] **Step 4: Verify no public HTTP approval route exists.**

### Task 5: Final verification and review

**Files:**
- Modify: `README.md` only if configuration/approval/evidence wording needs correction.
- Modify: PR #17 review/body metadata as evidence, not architecture.

- [ ] **Step 1: Run/verify `cargo fmt --all -- --check`.**
- [ ] **Step 2: Run/verify `cargo clippy --workspace --all-targets --all-features -- -D warnings`.**
- [ ] **Step 3: Run/verify `cargo build --workspace` and `cargo test --workspace`.**
- [ ] **Step 4: Verify server container, Kotlin/Android, ktlint, and Python jobs.**
- [ ] **Step 5: Independently re-read the resulting PR diff for trust-boundary regressions.**
- [ ] **Step 6: Update PR #17 with exact head SHA, CI evidence, remaining risks, and merge recommendation. Do not auto-merge.**
