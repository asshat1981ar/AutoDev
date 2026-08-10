# AutoDev — Repository Assessment

**Date:** 2026-08-09
**Head commit:** `819c97a` (`main`)
**Author role:** Principal systems architect
**Scope:** Full-repository inspection. No code was changed.

---

## 1. Executive summary

AutoDev is a **local-first, model-agnostic, multi-agent software engineering platform**
inspired by ForgeOS (evolved from OllamaDev). The intended design separates agent
**reasoning** from privileged **execution**: agents emit typed `AgentAction` intent, a
**policy** layer authorizes it, a Rust **ForgeCore** executes only authorized
operations, **verifiers** produce evidence, and an **orchestrator** advances or replans.

The repository is at the **very earliest foundation phase**. It contains:

- One Rust crate (`forge-core`) with typed domain types, a conservative policy
  evaluator, a dry-run executor, and unit tests.
- Three JSON Schema protocol contracts (agent action, execution result, task).
- One architecture/protocol design document.
- One GitHub Actions CI workflow for the Rust crate.
- A README documenting an ambitious multi-language roadmap.

Almost the entire documented platform (Kotlin control plane, orchestrator, Ollama model
fabric, MCP layer, Git/sandbox/process execution inside ForgeCore, SQLite/Room
persistence, Go worker fabric) is **planned but not yet implemented**. Only the Rust
execution-kernel seed and the protocol contracts exist in code.

A **concrete compile defect** exists today: `forge-core` references `serde_json::Value`
in its public API but declares `serde_json` only as a dev-dependency, so the CI gate
`cargo build --workspace` will fail. This is detailed in §6.

---

## 2. Current architecture

### 2.1 Intended architecture (from README)

```text
                    AutoDev
                       |
                Kotlin Control Plane   <- Android UI, lifecycle, local state
                       |
                Agent Orchestrator     <- SDLC state machine
                       |
          PLAN -> ACT -> VERIFY -> REPLAN
                       |
                Typed Agent Protocol   <- language-neutral JSON contracts
                       |
                Rust ForgeCore         <- sandbox, filesystem, Git, patches, processes, policy
             /        |        \
        sandbox     Git       process
        filesystem  patches   execution
                       |
           +------------+------------+
           |            |            |
        Ollama         MCP        GitHub
           |
    local / LAN / cloud models
```

### 2.2 What actually exists vs. what is planned

| Layer (README) | Technology | Status |
| --- | --- | --- |
| Control plane | Kotlin + Jetpack Compose | **Planned** — no Kotlin/Gradle code |
| Orchestration | Kotlin | **Planned** — none |
| Execution kernel | Rust `forge-core` | **Seed implemented** (types, policy, dry-run) |
| Model fabric | Ollama | **Planned** — none |
| Capability layer | MCP | **Planned** — only an `ActionType::Mcp` variant |
| Persistence | SQLite/Room | **Planned** — none |
| Distributed fabric | Go | **Planned** — none |
| Protocol | JSON Schema | **Implemented** (3 contracts) |
| CI | GitHub Actions | **Implemented** (Rust only) |

### 2.3 Physical layout

```
.
├── .github/workflows/rust.yml        # Rust build/test/lint/format gate
├── .gitignore                        # multi-language ignore rules
├── LICENSE                           # MIT
├── README.md                         # architecture + roadmap
├── crates/
│   ├── Cargo.toml                    # workspace root (edition 2021, resolver 2)
│   └── forge-core/
│       ├── Cargo.toml                # single crate, publish = false
│       └── src/lib.rs                # 192 lines: types + policy + dry-run + tests
├── docs/architecture/
│   ├── agent-protocol.md             # action lifecycle and design rules
│   └── repository-assessment.md      # this document
├── docs/adr/
│   └── ADR-001-forgecore-execution.md  # execution kernel decision
└── protocols/
    ├── agent-action.schema.json      # agent intent contract
    ├── execution-result.schema.json  # execution evidence contract
    └── task.schema.json              # durable task contract
## 3. What actually compiles / runs

### 3.1 Rust workspace

Manifests are structurally valid (edition 2021, resolver 2, workspace package
inheritance). `forge-core` declares:

- `[dependencies]`: `serde` (derive), `thiserror = "2"`
- `[dev-dependencies]`: `serde_json = "1"`

**However, the workspace does not compile cleanly under the CI gate:**

- `src/lib.rs` declares `pub payload: serde_json::Value` in the public `AgentAction`
  type. This is library code (outside `#[cfg(test)]`).
- `serde_json` is listed only under `[dev-dependencies]`.
- The GitHub Action step `cargo build --workspace` builds the library **without**
  dev-dependencies, so it would fail with an unresolved-crate error for `serde_json`.
- `cargo test --workspace` would succeed (dev-dependencies are available to tests),
  which is why the in-repo unit tests would pass while the build gate fails.

**Resolution:** move `serde_json` into `[dependencies]` (it is a runtime type of the
public API). This is a one-line fix and the single most immediate correctness action.

### 3.2 Local toolchain

The Rust toolchain (`cargo`/`rustc`) was **not installed** in this environment, so the
workspace could not be executed locally. Findings above are from static analysis of the
manifest and source. The three JSON schemas were machine-validated as well-formed JSON.

### 3.3 Protocol schemas

All three `protocols/*.schema.json` files parse as valid JSON. They are declarative
contracts only; no schema-validator dependency or contract test exists.

---

## 4. Module boundaries

| Module | Location | Boundary | Implemented? |
| --- | --- | --- | --- |
| **Domain types** | `forge-core::` | `ActionType`, `RiskLevel`, `AgentAction`, `PolicyDecision`, `PolicyError`, `ExecutionStatus`, `ExecutionResult` | Yes |
| **Policy** | `forge-core::validate_action`, `evaluate_policy` | Structural + risk-based authorization | Yes (conservative) |
| **Execution** | `forge-core::dry_run` | Dry-run only; refuses privileged effects | Yes (stub) |
| **Protocol** | `protocols/*.json` | Language-neutral JSON contracts | Yes |
| **Control plane** | *(planned)* | Kotlin/Android | No |
| **Orchestrator** | *(planned)* | Agent lifecycle state machine | No |
| **Model fabric** | *(planned)* | Ollama | No |
| **Capabilities** | *(planned)* | MCP | No |
| **Persistence** | *(planned)* | SQLite/Room | No |
| **Worker fabric** | *(planned)* | Go | No |

The single crate cleanly separates three core concerns (types, policy, execution) into
functions, but they all live in one file (`lib.rs`); there is no module partitioning yet.
The public API exposes `dry_run` as the only "execution" path — a deliberate safety stub.

---

## 5. Dependency map

```
forge-core (0.1.0, publish = false)
├── serde 1           (derive)      # serialization for protocol types
├── thiserror 2                      # error enum boilerplate
└── [dev] serde_json 1               # JSON payload + test fixtures
        └── (depends on serde 1)
```

- **Runtime deps:** 2 (`serde`, `thiserror`). Lean surface — good for a security-sensitive
  execution kernel.
- **Bug:** `serde_json` is a runtime requirement of the public API but is declared as a
  dev-dependency (§3.1).
- **No `Cargo.lock` is committed.** The last commit ("keep Rust dependency lockfile under
  version control") removed the ignore rule, but no lockfile exists in-tree yet; it will
  be generated on first `cargo` run in CI. Untracked dependency versions are a minor
  reproducibility risk.
- No Kotlin/Gradle, Go, or other language manifests exist yet.
## 6. Technical debt and correctness issues

| # | Severity | Issue |
| --- | --- | --- |
| 1 | **High** | **`serde_json` declared as dev-dep but used in the public API** → `cargo build --workspace` CI gate fails. |
| 2 | **Medium** | **`ExecutionResult` diverges from `execution-result.schema.json`.** Rust exposes only `action_id/status/message`; the schema requires `started_at/completed_at` and documents `exit_code/stdout/stderr/artifacts/verification/error`. The schema has no `message` field. A Rust-valid result would violate the schema and vice-versa. |
| 3 | **Medium** | **Schema requires `payload`; Rust makes it optional** (`#[serde(default)]`). Interop can silently accept schema-invalid actions. |
| 4 | **Low/Medium** | **Schema `expected` field is not represented in Rust `AgentAction`.** Documented intent (expectations/verification input) is dropped. |
| 5 | **Low** | **No `Cargo.lock` committed** despite the commit intent; dependency versions float until first resolve. |
| 6 | **Low** | **Rename mismatch risk:** Rust uses `#[serde(rename_all = "snake_case")]`/`lowercase`; no contract test asserts Rust serialization matches the JSON schema enums. Drift is currently unchecked. |
| 7 | **Info** | `thiserror` on a 5-variant error enum is a small dependency; acceptable, but could be hand-implemented if dependency minimization is desired. |

**Risk-definition note.** `RiskLevel::Critical` is only enforced for the structural
`approval:critical` capability in `validate_action`. The risk → decision mapping in
`evaluate_policy` treats `Critical` the same as `Medium`/`High` (all `RequireApproval`).
This is safe but semantically incomplete — a critical risk should demand a stricter
approval path than a medium one.

---

## 7. Security boundaries

### 7.1 What is secure today

- **No privileged operations implemented.** `dry_run` explicitly refuses all effects and
  never touches the filesystem or spawns processes. This is the strongest security
  property in the repo.
- **Typed intent, not text.** Actions are typed/structured, not arbitrary shell strings.
- **Capabilities are declared**, not inferred.
- **Structural validation** (`validate_action`) rejects empty identities/reasons and
  enforces the `approval:critical` capability for critical risk.
- **Minimal dependency surface** in the future execution kernel.

### 7.2 Security gaps / future exposure

- **No sandbox** (namespace/cgroup/seccomp/Landlock) yet — the entire execution-isolation
  model is future work. This is the highest-stakes area once real execution lands.
- **No approval flow.** `RequireApproval` currently short-circuits to `Denied` in
  `dry_run`; there is no human-approval path, no approval record, no audit log.
- **No secrets handling**, no key store, no credential scoping.
- **No signing/verification of actions or provenance** — evidence/artifacts are schema
  declarations only.
- **No path canonicalization or workspace confinement** — `payload` is unvalidated
  `serde_json::Value`, so a future filesystem adapter must add its own path-escape
  defenses.
- **`payload` is an untyped `Value`** (schema-level `{ "type": "object" }`). Malicious or
  malformed payloads are structurally accepted as long as required fields are present.

---

## 8. Implemented vs. documented

| Documentation | Implemented? |
| --- | --- |
| README architecture diagram | Partial — only ForgeCore seed + protocol |
| Roadmap items 1–11 | Only items 4 (typed protocol) and 5 (ForgeCore boundary) partially. Items 1–3, 6–11 not done. |
| `agent-protocol.md` action lifecycle | Policy validation + dry-run only; no real execution, verification, or orchestrator |
| `agent-protocol.md` 8 action types | Represented as an enum; none executed |
| `task.schema.json` | Schema only; no task store or lifecycle |
| `execution-result.schema.json` | Schema only; Rust `ExecutionResult` is a **subset** that does not fulfill it |

---

## 9. Missing tests

| Area | Current state | Gap |
| --- | --- | --- |
| Rust unit tests | 5 tests in `lib.rs` (policy + dry-run) | Good seed; no coverage of `ExecutionResult` serialization, `ActionType`/`RiskLevel` serde round-trips, `Deny` path, `High`/`Critical` mapping |
| Protocol contract tests | None | **No test asserts Rust structs serialize to schema-conformant JSON.** This is the mechanism that would have caught issues #2/#3/#6. |
| Schema validation | None | No test validates sample documents against the JSON Schema. |
| Integration tests | None | No cross-boundary flow (validate → policy → record). |
| CI | Rust build/test/clippy/fmt | Present, but the build gate is currently broken by issue #1. |
| Kotlin/Go tests | N/A | No implementation exists to test. |
## 10. Architectural inconsistencies

1. **Protocol/implementation drift** — the Rust domain model does not match the JSON
   Schema contracts it is meant to implement (§6 #2–#4). The schemas are described as
   "the initial protocol contract" and implementations "should validate against them,"
   yet no code validates against them and the types already diverge.
2. **Dependency mis-declaration** — `serde_json` used in the public API but classified as
   a dev-dependency (§6 #1), breaking the stated build gate.
3. **Single-file monolith** — types, policy, and execution all live in `lib.rs` with no
   module boundaries, despite a multi-module roadmap.
4. **Risk semantics incomplete** — `Critical` is not treated distinctly in the policy
   decision (see §6 note), so the "explicit approval capability" enforcement is
   structurally separate from actual risk handling.
5. **Lockfile intent vs. reality** — commit message claims the lockfile is versioned, but
   none is present.
6. **Language neutrality unproven** — the "must remain language-neutral" rule in
   `agent-protocol.md` is asserted, but there is no conformance mechanism and no second
   implementation (Kotlin) to validate neutrality against.

---

## 11. Unnecessary complexity

- **None significant.** The footprint is small and intentionally conservative. The only
  marginal item is `thiserror` for a five-variant error enum (§6 #7). The workspace
  multi-crate layout is slight over-engineering now but is justified by the roadmap.

---

## 12. Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| CI is red at head (build fails) | High | Blocks all future Rust work | Fix `serde_json` dependency |
| Protocol/type drift compounds | High | Contracts become fiction; rework later | Add contract tests early |
| Scope creep into a broad platform before the core is proven | Medium | Effort wasted on orchestration without a working execution+policy core | Sequence: fix core, then contract tests, then real sandbox execution |
| Security expectations ahead of implementation | Medium | Users/contributors assume execution exists | Keep the README status explicit (already marked "early") |
| No lockfile → non-reproducible builds | Low | Flaky CI | Commit generated `Cargo.lock` |

---

## 13. Missing capabilities (highest-value first)

1. **Real, sandboxed execution** in ForgeCore (filesystem + process) behind the policy
   layer — the platform's core value proposition.
2. **Protocol conformance tests** tying Rust types to the JSON schemas — cheap, catches
   the current drift immediately.
3. **Approval flow** (human-in-the-loop) with audit records for `RequireApproval`.
4. **Workspace confinement & path safety** for any filesystem adapter.
5. **Task/action persistence** (durable evidence, provenance).
6. **Ollama integration** for model discovery/routing.
7. **Orchestrator state machine** (PLAN → ACT → VERIFY → REPLAN).
8. **Verification/artifact layer**.
9. **MCP capability layer**.
10. Kotlin control plane, Go worker fabric (later).

### Single highest-value next capability

**A real, sandboxed, policy-gated *filesystem* execution adapter in `forge-core`** — with
path-confinement and return of a schema-conformant `ExecutionResult` — is the most
valuable next step. It converts the current dry-run stub into the platform's actual
differentiator (trusted execution of typed intent), forces resolution of the
protocol-drift and dependency issues, and can be delivered and tested entirely within the
existing Rust boundary. This direction is designed in **ADR-001-forgecore-execution.md**.
## 14. Recommended development sequence

Phase 0 — **Stabilize what exists** (~minimal effort):
1. Fix the `serde_json` dependency so `cargo build --workspace` passes.
2. Commit the generated `Cargo.lock`.

Phase 1 — **Make the contract real**:
3. Add protocol conformance tests (Rust ↔ JSON Schema) for agent action, execution
   result, and task.
4. Align `ExecutionResult` (and `AgentAction` `expected`/`payload` handling) with the
   schemas, or explicitly version the schemas.

Phase 2 — **Prove execution** (per ADR-001):
5. Implement a sandboxed, workspace-confined filesystem adapter (read/write/patch)
   behind policy, returning schema-conformant results.
6. Add process execution behind the same policy with a strict allow-list and OS sandbox.
7. Implement the approval path with audit records.

Phase 3 — **Add orchestration and models**:
8. Task/action persistence (SQLite/Room on Kotlin; plain store in the Rust seed).
9. Ollama discovery/routing.
10. PLAN → ACT → VERIFY → REPLAN orchestrator.

Phase 4 — **Scale**:
11. MCP capability layer.
12. Go worker fabric.
## 15. Next architectural directions (scored)

Three candidate directions for the next major push. Scores are 1–5 (5 = best).

### 15.1 Direction A — Conservative: "Make the execution kernel real, Rust-only"

Solidify `forge-core` into a policy-gated, sandboxed filesystem+process executor with
full protocol conformance, approval hooks, and provenance records. No new languages.

| Attribute | Score | Rationale |
| --- | --- | --- |
| Security | 5 | Adds sandbox + path confinement at the point of maximum risk |
| Complexity | 5 | Single language, no new runtime, incremental on existing crate |
| Maintainability | 5 | Small, focused, fully testable |
| Performance | 4 | Native Rust; no FFI overhead yet |
| Portability | 4 | Rust is portable; sandbox primitives vary by OS |
| Offline capability | 5 | Fully local-first, no network deps |
| Extensibility | 4 | Clean base for later MCP/Go integration |
| Testability | 5 | Unit + integration within one crate |
| **Total** | **37** | |

### 15.2 Direction B — Production: "Full vertical slice (Kotlin + Rust)"

Stand up the Kotlin control plane and orchestrator talking to a real ForgeCore executor
over the typed protocol, with SQLite persistence. Adds the Android UI and end-to-end
flow.

| Attribute | Score | Rationale |
| --- | --- | --- |
| Security | 3 | More surface (Kotlin, FFI, persistence); sandbox still immature |
| Complexity | 3 | Two languages + FFI + new build systems |
| Maintainability | 3 | Larger surface, more conventions to enforce |
| Performance | 3 | FFI overhead; premature for current maturity |
| Portability | 2 | Android-first; less portable to desktop/server |
| Offline capability | 5 | Still local-first |
| Extensibility | 4 | Unlocks UI/UX and real orchestration |
| Testability | 3 | Multi-language E2E tests are heavier |
| **Total** | **26** | |

### 15.3 Direction C — Experimental: "Ollama-first agent loop"

Prioritize model discovery and a minimal self-contained agent loop (model → typed action
→ dry-run → model) before hardening execution.

| Attribute | Score | Rationale |
| --- | --- | --- |
| Security | 2 | Model/hosted-code boundary; risky without execution hardening first |
| Complexity | 3 | New runtime (`ollama`), server management |
| Maintainability | 3 | Early proof-of-concept, likely to be rewritten |
| Performance | 3 | Model latency dominates; unoptimized |
| Portability | 3 | Requires Ollama install per platform |
| Offline capability | 4 | Local models possible, but network typically involved |
| Extensibility | 2 | Builds on current void; risky foundation |
| Testability | 2 | Model outputs nondeterministic → flaky tests |
| **Total** | **22** | |

### 15.4 Recommendation

**Adopt Direction A (Conservative).** It scores highest overall (37), directly addresses
the single highest-value missing capability (§13), forces the fix of the current build
defect and protocol drift, and de-risks everything downstream (the Kotlin orchestrator
and Ollama loop both depend on a trustworthy execution kernel). Direction B should follow
only after A has landed a sandboxed executor with schema-conformant results. Direction C
is the least safe as a *next* step because it increases surface area before the
security-critical execution boundary is hardened.

**Immediate next step (concrete):** fix the `serde_json` dependency so `cargo build
--workspace` passes, then add protocol conformance tests before any new capability.

---

## 16. Method note

- No code was changed; this is a read-only assessment.
- The Rust toolchain was unavailable locally; build findings are from static analysis of
  the manifest and source. JSON schemas were machine-validated as well-formed JSON.
- Git history (17 commits) was reviewed; the repository is coherent and deliberately
  incremental, with no evidence of abandoned or dead code.
```