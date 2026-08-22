# Harness Profile Fabric v0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic, authority-free Harness Asset/Profile protocol to ForgeCore, ship five built-in development harness profiles, route development contracts to profiles, and verify profile integrity without allowing harness configuration to authorize effects.

**Architecture:** Extend the existing federated harness-kernel design with a narrow `forge_core::harness` module. Harness profiles are declarative intent/configuration only: they describe stages, asset references, verification contracts, and optimization metadata, while ForgeCore policy/execution remains the only authority boundary. Five built-in profiles cover SDLC, adaptive Agile, structured idea generation, software-development optimization, and meta-harness generation; deterministic routing provides a reproducible baseline for later learned routing.

**Tech Stack:** Rust 2021, serde/serde_json, existing `forge-core` `DevelopmentContract`, existing GitHub Actions Rust/harness gates.

**Spec:** `docs/superpowers/specs/2026-08-18-federated-harness-kernel-design.md`

## Global Constraints

- Harness assets and profiles must never construct, contain, deserialize, or return `AuthorizationGrant`.
- Harness configuration is advisory intent; it may not execute effects or mark its own work verified.
- Routing must be deterministic and model-free in v0.
- Every executable stage description must declare at least one independent verification contract.
- Built-in profiles must be versioned and have stable IDs.
- No new runtime dependency is required beyond existing `serde`, `serde_json`, and `thiserror`.
- Android/Termux compatibility must not depend on Docker, Bun, PTY, or desktop-only tools.

---

### Task 1: Harness Asset and Profile Protocol

**Files:**
- Create: `crates/forge-core/src/harness.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/harness_profiles.rs`

**Interfaces:**
- Consumes: existing `DevelopmentContract` from `forge_core::skill`.
- Produces: `HarnessKind`, `HarnessAssetKind`, `HarnessAssetRef`, `HarnessStage`, `HarnessProfile`, `HarnessRegistry`, `HarnessError`.

- [x] **Step 1: Write failing protocol tests**

Add tests that require the new public types, duplicate-profile rejection, duplicate-stage rejection, empty-verification rejection, and JSON round-trip stability.

- [x] **Step 2: Run the focused test and verify RED**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: compile failure because `forge_core::harness` public API does not exist yet.

- [x] **Step 3: Implement minimal protocol and validation**

Create typed serde models and deterministic registry validation. `HarnessProfile::validate()` must reject empty IDs/versions/objectives/triggers/stages, duplicate stage IDs, stages with no verification contracts, and empty asset IDs/versions. Do not add effect execution methods.

- [x] **Step 4: Export the protocol from `lib.rs`**

Add `pub mod harness;` and public re-exports for the protocol types.

- [x] **Step 5: Run focused tests and verify GREEN**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: PASS.

- [x] **Step 6: Commit**

Commit message: `feat(forge-core): add harness asset protocol`

### Task 2: Five Built-in Harness Profiles

**Files:**
- Modify: `crates/forge-core/src/harness.rs`
- Test: `crates/forge-core/tests/harness_profiles.rs`

**Interfaces:**
- Consumes: Task 1 profile and registry types.
- Produces: `default_harness_profiles() -> HarnessRegistry` with exactly five stable profiles.

- [x] **Step 1: Write failing built-in-profile tests**

Require these IDs and kinds:
- `forgeflow-sdlc` → `HarnessKind::Sdlc`
- `sprintmesh-agile` → `HarnessKind::Agile`
- `idea-tournament` → `HarnessKind::Innovation`
- `optiforge-optimizer` → `HarnessKind::Optimizer`
- `harnessforge-meta` → `HarnessKind::Meta`

Assert every built-in validates, has at least three stages, contains verification at every stage, and has unique asset/stage references.

- [x] **Step 2: Run focused tests and verify RED**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: test failure because built-in profile catalog is absent.

- [x] **Step 3: Implement the five profiles**

Encode the approved workflows:
- ForgeFlow: discover → requirements → architecture → plan → isolated implementation → review → CI/verification → retrospective.
- SprintMesh: product goal → backlog refinement → prioritization → flow selection → parallel execution → integration → demo/retro.
- IdeaTournament: problem model → AutoTRIZ contradiction pass → Six Hats perspectives → research validation → RICE-plus scoring → game-theoretic stress test → prototype selection.
- OptiForge: observe → baseline → bottleneck detection → hypothesis → controlled experiment → benchmark → accept/reject → learning.
- HarnessForge: inventory → repetition mining → harness generation → adversarial simulation → benchmark → promotion proposal → versioned learning.

Use asset references for Superpowers skills, GitHub, CodeRabbit, Requirements Extractor, Context7, alphaXiv/Parallel Search, Linear, Engram, workflow registries, and evaluation components where appropriate. References remain descriptive and carry no authority.

- [x] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: PASS.

- [x] **Step 5: Commit**

Commit message: `feat(forge-core): add built-in development harnesses`

### Task 3: Deterministic Harness Router

**Files:**
- Modify: `crates/forge-core/src/harness.rs`
- Test: `crates/forge-core/tests/harness_profiles.rs`

**Interfaces:**
- Consumes: `HarnessRegistry`, `DevelopmentContract`.
- Produces: `HarnessRoutingEvidence`, `HarnessRoute`, `route_harness(&HarnessRegistry, &DevelopmentContract, usize) -> HarnessRoute`.

- [x] **Step 1: Write failing routing tests**

Cover SDLC/feature work, sprint/backlog work, ideation/TRIZ requests, performance/optimization work, and harness/workflow-generation requests. Add a deterministic tie-break test that orders equal scores by stable profile ID.

- [x] **Step 2: Run focused tests and verify RED**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: compile/test failure because router API is absent.

- [x] **Step 3: Implement deterministic routing**

Score lowercase trigger-term matches across goal, acceptance criteria, and constraints. Use fixed integer weights and stable ID tie-breaking. Return routing evidence with matched terms; do not call models or external services.

- [x] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: PASS.

- [x] **Step 5: Commit**

Commit message: `feat(forge-core): route development contracts to harnesses`

### Task 4: Evaluation and Promotion Guardrails

**Files:**
- Modify: `crates/forge-core/src/harness.rs`
- Test: `crates/forge-core/tests/harness_profiles.rs`

**Interfaces:**
- Consumes: `HarnessProfile` plus externally produced evaluation metrics.
- Produces: `HarnessEvaluation`, `HarnessPromotionDecision`, `evaluate_harness_candidate()`.

- [x] **Step 1: Write failing promotion tests**

Require promotion to fail when correctness/evidence completion regress, unsafe-action rejection is below baseline, sample size is zero, or verification is self-reported only. Require promotion eligibility only when mandatory safety/correctness metrics meet or exceed baseline and at least one efficiency metric improves.

- [x] **Step 2: Run focused tests and verify RED**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: compile/test failure because promotion evaluation API is absent.

- [x] **Step 3: Implement pure promotion evaluation**

Use integer basis points / integer counts rather than floating-point thresholds. The function returns advisory promotion evidence only and never mutates registries, policy, or execution authority.

- [x] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: PASS.

- [x] **Step 5: Commit**

Commit message: `feat(forge-core): gate harness profile promotion`

### Task 5: Repository Harness Documentation and Final Verification

**Files:**
- Create: `docs/harness/profile-fabric-v0.md`
- Modify: `scripts/check_harness_drift.py`
- Modify: `tests/test_harness_drift.py`

**Interfaces:**
- Consumes: public ForgeCore harness API and built-in profile IDs.
- Produces: documented invariants plus drift enforcement that prevents accidental removal/renaming of the five built-ins or authority-boundary language.

- [x] **Step 1: Write failing drift tests**

Add checks that the repository documentation names all five stable profile IDs and states that harness configuration cannot mint authorization or self-verify.

- [x] **Step 2: Run Python drift tests and verify RED**

Run: `python -m unittest tests.test_harness_drift -v`

Expected: FAIL until documentation/checker support is added.

- [x] **Step 3: Add documentation and drift checks**

Document architecture, profile responsibilities, routing evidence, promotion metrics, and the ForgeCore authority boundary. Extend the drift checker with explicit stable-fragment validation.

- [x] **Step 4: Run focused Python and Rust tests**

Run:
- `python -m unittest tests.test_harness_drift -v`
- `python scripts/check_harness_drift.py --verbose`
- `cargo test --locked -p forge-core --test harness_profiles --test harness_promotion`

Expected: PASS.

- [x] **Step 5: Run repository verification**

Run:
- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
- `cargo test --locked --workspace`
- `python -m unittest discover -s tests -v`
- `bash scripts/verify_reproducible.sh`

Expected: all green in canonical GitHub Actions.

- [x] **Step 6: Independent review**

Open/update a pull request and require CodeRabbit plus branch CI evidence. Resolve load-bearing findings before marking ready for merge.

- [x] **Step 7: Commit**

Commit message: `docs(harness): document and enforce profile fabric`
