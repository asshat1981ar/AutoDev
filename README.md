# AutoDev

AutoDev is a local-first, model-agnostic multi-agent software-development runtime. Agents propose typed intent; ForgeCore authorizes and executes that intent inside explicit capability, workspace, approval, evidence, and verification boundaries.

## Core development loop

```text
Goal
  ↓
TaskGraph
  ↓
Context / dispatch
  ↓
ExecutionEnvelope
  ↓
Policy + trusted authorization
  ↓
ForgeCore execution
  ↓
EvidenceStore
  ↓
VerificationFabric
  ↓
Verified ─────────→ complete
Rejected ─────────→ bounded replan
Approval required → blocked until approved
```

The runtime deliberately separates **generation** from **verification**. An action reporting that it succeeded is not enough to complete a task. Required verification evidence must actually run and pass.

## Execution envelope

`ExecutionEnvelope` is the durable hand-off contract between planning, execution, and verification. It binds:

- task, run, operation, and action identity;
- bounded context references;
- risk and capability declarations;
- approval requirements and trusted approval references;
- required and produced evidence references;
- lifecycle state and bounded attempt count.

Lifecycle transitions are explicit and validated:

```text
planned → authorized → executing → verifying → verified
                           ↓            ↓
                        rejected ←──────┘
                           ↓
                       replanning
                           ↓
                         planned
```

## Trusted authorization

Agent/model payloads and declared capabilities are untrusted intent. Human approval is represented separately by a kernel-owned `AuthorizationGrant` and is supplied only at the trusted execution boundary.

This means a payload field such as:

```json
{"approved": true}
```

is **not authority**. ForgeCore strips caller-supplied Git approval state and only recreates the internal authorization marker from a trusted grant. The same grant model is used for high-risk file reads, writes, and patches.

Public effect adapters remain fail-closed when no grant is supplied.

## Required verification evidence

`ExecutionEnvelope.evidence.required` names the checks that must be present and passing before a task can enter `verified`.

Canonical verification names are:

- `unit_tests`
- `build`
- `lint`
- `static_analysis`
- `security`

A report where every executed check passed still fails the task if a declared required check never ran. Unknown required evidence names also fail closed.

## Durable verified orchestration

`VerifiedOrchestrator` composes the existing `TaskGraph` scheduler with the evidence-driven `DevelopmentLoop` without replacing the legacy orchestrator.

It persists one execution envelope per task across attempts:

- `Verified` → `TaskStatus::Completed`
- `Replanned` → `TaskStatus::Ready`
- `Exhausted` → `TaskStatus::Failed`
- missing approval → `TaskStatus::Blocked`

Approval resume reuses the same envelope and does not consume an execution attempt.

`VerifiedOrchestratorState` is serializable so envelope lifecycle and retry state can be recovered after process restart.

## Repository context fabric

ForgeCore includes deterministic bounded repository retrieval. Context selection is local-first, reproducible, and budgeted by maximum files and bytes. Context is treated as evidence for planning rather than permission to mutate the repository.

## Cline / Termux

AutoDev includes a Cline development fabric and a Termux-compatible Kanban launcher. CI validates the Python entry points, Termux launcher, and Cline development-fabric tests alongside the Rust kernel.

For Android/Termux environments, prefer portable subprocess and Streamable HTTP MCP paths instead of assuming desktop-only PTY, Bun, or Docker support.

## Verification gates

The Rust workflow currently requires:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

It also validates the Cline/Termux fabric. A development slice is not considered green until these gates pass.

## Design principle

> Agents propose intent. Policy authorizes capabilities. Trusted components execute. Independent verifiers produce evidence. Orchestrators advance or replan from that evidence.

The intent is not maximum autonomy. AutoDev favors bounded, observable, recoverable development loops whose claims can be independently verified.
