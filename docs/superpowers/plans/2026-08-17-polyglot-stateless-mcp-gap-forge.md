# Polyglot Stateless MCP + Capability Gap Forge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a production-grade stateless MCP 2026-07-28 endpoint around ForgeCore, plus an evidence-gated capability-gap forge that can autonomously stage candidate skills and MCP configurations without creating a second authority path.

**Architecture:** `autodev-server` owns the Streamable HTTP transport and constructs a fresh MCP handler per request. Durable objective/evidence state remains in cloneable service handles outside the protocol handler. `forge-core` gains a pure capability-gap domain that turns observed failures into typed candidate blueprints and untrusted file-write proposals; existing authorization, execution, and evidence layers remain the only mutation authority. A seeded simulation track compares hybrid topology candidates before Go or Flutter enter the production runtime.

**Tech Stack:** Rust 2021, Axum 0.7, Tokio, rmcp 3.x with Streamable HTTP server support, Serde/JSON Schema-compatible contracts, ForgeCore ArchitectureEvidence, existing GitHub Actions, KMP/Android unchanged in this slice.

## Global Constraints

- ForgeCore remains the sole trusted authorization/execution/evidence authority.
- MCP handlers are stateless for protocol 2026-07-28; no `Mcp-Session-Id`, standalone GET/DELETE stream, or hidden transport session state.
- Shared application state may be held only through explicit cloneable handles captured by the handler factory.
- MCP mutation-facing tools return typed proposals only; they never call raw filesystem, process, Git, or approval APIs.
- Gap discovery may autonomously generate and stage candidate files but may not grant capabilities, weaken policy, install credentials, change approvals, or merge/deploy itself.
- Unevaluated Skill candidates must stay outside the active `.cline/skills/` discovery namespace. Stage them under `.cline/candidates/skills/` until promotion is separately authorized.
- Skills and MCP candidates require baseline evidence and measurable evaluation before promotion.
- Web/CLI are the next new client surfaces; Flutter and Go remain experiment-gated.
- No production claim without executed verification.

---

### Task 1: ForgeCore Capability Gap Domain

**Files:**
- Create: `crates/forge-core/src/capability_gap.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/capability_gap.rs`

**Interfaces:**
- Consumes: `AgentAction`, `ActionType`, `Capability`, `RiskLevel`, and architecture-evidence concepts.
- Produces: `GapObservation`, `GapKind`, `CandidateKind`, `CapabilityCandidate`, `CandidateArtifact`, `CandidateEvaluation`, `PromotionDecision`, `discover_candidates`, `propose_candidate_writes`.

- [ ] **Step 1: Write failing tests** proving procedure gaps map to Skill candidates, external-capability gaps map to MCP candidates, candidate file writes are only `AgentAction` proposals, and promotion requires positive measured delta with zero safety regressions.
- [ ] **Step 2: Run Rust tests and confirm RED** because the `capability_gap` module does not exist.
- [ ] **Step 3: Implement the minimum pure domain** needed to satisfy the tests, with deterministic ordering and stable candidate IDs.
- [ ] **Step 4: Run focused and workspace tests** and require GREEN.
- [ ] **Step 5: Run fmt/clippy** and commit the slice.

### Task 2: Stateless MCP 2026-07-28 Transport

**Files:**
- Create: `crates/autodev-server/src/mcp.rs`
- Modify: `crates/autodev-server/src/lib.rs`
- Modify: `crates/autodev-server/Cargo.toml`
- Test: `crates/autodev-server/tests/mcp_stateless.rs`

**Interfaces:**
- Consumes: `AppState` through an explicit cloneable facade, ForgeCore query/proposal types, rmcp Streamable HTTP transport.
- Produces: `/mcp`, `AutoDevMcp`, tool list, objective query tools, capability-gap discovery tool, proposal-only mutation tool.

- [ ] **Step 1: Write failing integration tests** for `/mcp`, modern protocol discovery/tool listing, lack of `Mcp-Session-Id`, and cross-request operation without a protocol session.
- [ ] **Step 2: Run CI and confirm RED** because the transport and dependency are absent.
- [ ] **Step 3: Add rmcp 3.x server + Streamable HTTP feature and implement `AutoDevMcp`** using a fresh handler factory per request.
- [ ] **Step 4: Expose only read/query plus proposal-oriented tools**. No raw execution tool is permitted.
- [ ] **Step 5: Run focused tests, workspace tests, fmt, clippy and MCP conformance where practical.**

### Task 3: Gap Forge Candidate Staging

**Files:**
- Modify: `crates/forge-core/src/capability_gap.rs`
- Test: `crates/forge-core/tests/capability_gap.rs`

**Interfaces:**
- Consumes: `CapabilityCandidate`.
- Produces: deterministic `CandidateArtifact` files and `AgentAction` write proposals for `.cline/candidates/skills/<id>/SKILL.md` and `.cline/mcp/generated/<id>.json`.

- [ ] **Step 1: Add RED tests** for deterministic, confined artifact paths and rejection of unsafe IDs/path traversal.
- [ ] **Step 2: Implement deterministic artifact generation** with no runtime privilege grant.
- [ ] **Step 3: Add candidate evaluation and promotion rules** so non-improving candidates are rejected.
- [ ] **Step 4: Re-run security/adversarial tests, including proof that generated Skill candidates do not land under active `.cline/skills/`.**

### Task 4: Hybrid Architecture Simulation

**Files:**
- Create: `crates/forge-core/src/hybrid_simulation.rs`
- Modify: `crates/forge-core/src/lib.rs`
- Test: `crates/forge-core/tests/hybrid_simulation.rs`
- Document: `docs/architecture/polyglot-simulation.md`

**Interfaces:**
- Produces deterministic seeded traces for topology options: Rust/KMP, Rust+Go gateway, Kotlin edge+Rust authority, Rust+bounded Go worker, Rust+future Flutter client.

- [ ] **Step 1: Write RED tests** for deterministic paired-seed results and Pareto-frontier calculation.
- [ ] **Step 2: Implement transparent simulation parameters** with success/cost/latency/security/complexity metrics.
- [ ] **Step 3: Run sensitivity analysis** and report assumptions separately from repository facts.
- [ ] **Step 4: Keep Go/Flutter experimental unless they survive the evidence gate.**

### Task 5: Integration + Verification

**Files:**
- Update relevant README/architecture docs only after code is green.

- [ ] Rebase/reconcile persona branches onto the integration branch without duplicating authority code.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test --workspace`.
- [ ] Run Kotlin `./gradlew clean assemble test ktlintCheck` if shared contracts changed.
- [ ] Run existing Python fabric tests.
- [ ] Review the complete diff for authority drift, path safety, unbounded output, MCP version handling, and generated-artifact confinement.
- [ ] Keep the PR draft until CI is green and review findings are resolved.
