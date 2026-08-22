# AMCX ↔ ForgeCore Bridge v1.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use an isolated worktree and test-driven development. Do not implement production bridge code until the focused tests have been observed failing for the expected missing-feature reason.

**Goal:** Add a pure, non-authorizing mapping layer between existing ForgeCore state and AMCX reference/projection types without creating a second execution, plan, evidence, or verification authority.

**Architecture:** `crates/forge-core/src/amcx_bridge.rs` contains serializable validated references derived from existing ForgeCore objects. Conversion is one-way projection: host objects remain canonical and are never mutated. The module exposes no executor, policy mutator, grant constructor, filesystem/network/process call, or schema activation function.

**Spec:** `docs/architecture/amcx/reconciliation-v1.1.md`

## Progress

- [x] Repository ownership reconciliation completed.
- [x] Focused RED gate observed under Rust 1.97.1: `E0583` because `forge_core::amcx_bridge` did not exist.
- [x] Minimal projection-only bridge implemented.
- [x] Local focused GREEN proxy passed: 5/5 bridge projection tests.
- [x] Manual authority-surface review found no executor or `AuthorizationGrant` construction/return path; context projection does not copy source bodies.
- [ ] Canonical repository Rust gate: fmt, clippy, focused bridge test, workspace test.
- [ ] Harness drift gate.
- [ ] Independent PR review.
- [ ] Mark PR ready only after required gates are evidenced.

## Surprises & Discoveries

- The ChatGPT sandbox can run the user-provided offline Rust 1.97.1 toolchain but cannot resolve `github.com`, so a complete checkout/workspace test cannot be reconstructed there without fabricating repository state.
- PR #38 initially exposed no GitHub status checks despite `.github/workflows/ci.yml` declaring `pull_request` on `main`. This plan checkpoint intentionally updates the PR branch to emit a fresh `synchronize` event; absence of a run after that event must be treated as a CI/integration finding rather than a pass.
- The offline Rust bundle exposes `rustc`, `cargo`, and `rustfmt`; `cargo clippy` is not registered in that sandbox installation. Canonical CI remains responsible for Clippy.

## Decision Log

- Preserve AutoDev's existing `ExecPlan`, `EvidenceStore`, `VerificationFabric`, `ContextPack`, `ExecutionEnvelope`, and kernel-owned `AuthorizationGrant` as canonical host authorities. AMCX receives projections/references only.
- Keep PR #38 draft until canonical repository verification and independent review are observable.
- Do not substitute the local proxy harness for workspace verification; it proves the bridge behavior only.

## Outcomes & Retrospective

Current outcome is **implemented but not integration-verified**. The bridge surface exists and focused behavior is exercised locally, while merge readiness remains blocked on repository-level CI/harness evidence.

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

**Tests:**

1. `plan_projection_retains_identity_without_mutating_plan`
2. `evidence_projection_requires_verified_fingerprint`
3. `verification_projection_preserves_verdict_as_evidence_only`
4. `context_projection_is_reference_only`
5. `blank_source_identity_fails_closed`
6. API review: bridge exposes projection types/functions only; no `From<...> for AuthorizationGrant` and no public method returning `AuthorizationGrant`.

**RED evidence:** observed with Rust 1.97.1: module-not-found `E0583` before production bridge creation.

### Task 2: Implement minimal validated reference types

**Files:**
- Create: `crates/forge-core/src/amcx_bridge.rs`
- Modify: `crates/forge-core/src/lib.rs`

Implementation is validation + immutable projection only.

**GREEN evidence:** local focused proxy: 5 passed, 0 failed.

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
