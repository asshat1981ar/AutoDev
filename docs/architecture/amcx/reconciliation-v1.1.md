# AutoDev ↔ AMCX-1 v1.1 Reconciliation

## Purpose

This document reconciles the imported `protocols/amcx/` release with the existing AutoDev runtime before any semantic bridge code is added. The governing rule is **extend existing canonical owners; do not create competing state machines or authority paths**.

## Repository truth

AutoDev already provides the trusted execution and coordination surfaces AMCX expects:

- `forge_core::ExecPlan` owns durable plan lifecycle, bounded attempts/replans, checkpoint creation, decisions, discoveries, and interruption reconciliation.
- `ExecutionEnvelope` composes task intent, bounded context references, policy binding, evidence binding, and PLAN → AUTHORIZE → ACT → VERIFY lifecycle.
- `AuthorizationGrant` is kernel-owned and cannot be supplied by a model.
- `EvidenceStore` owns append-only execution provenance and content fingerprints.
- `VerificationFabric` executes checks independently from generation and produces current verification verdicts.
- `ContextPack` is deterministic, bounded, read-only repository evidence for models.
- `AgentRuntime` converts provider output to typed `AgentAction`, submits it to policy, consumes execution results, and records evidence.

These are stronger and more concrete than the generic host surfaces in the portable AMCX reference implementation, so they remain canonical.

## Reconciliation matrix

| AMCX domain | Existing AutoDev owner | Relationship | Decision |
|---|---|---|---|
| Plan / step lifecycle | `ExecPlan`, `TaskGraph`, `ExecutionEnvelope` | `already-satisfied` | Keep AutoDev lifecycle canonical. ECM stores collaboration references to plan/task/run/envelope IDs only. |
| Collaboration task/attempt/message/lease | none equivalent to full ECM event model | `missing` | Add an ECM bridge/service outside the trusted execution kernel. It may coordinate; it may not authorize effects. |
| Portable memory history | no canonical portable cross-provider memory DAG | `missing` | Add AMX canonical store/projection layer using imported schemas and reducers. |
| Effect authorization | `AuthorizationGrant`, policy, ForgeCore `execute` | `host-is-stronger` | AMCX must never mint or infer a grant. External decision references are revalidated by ForgeCore/host policy. |
| Effect execution / receipts | ForgeCore + `ExecutionResult` / `ExecutionRecord` | `already-satisfied` | Reuse execution records as host effect evidence; AMCX carries immutable references/digests only. |
| Evidence verdict/freshness | `VerificationFabric`, `EvidenceStore` | `already-satisfied` | AMCX GateProfile/evaluation results project to evidence references; they do not replace verifier ownership. |
| Context construction | `ContextPack`, `ContextRefs` | `compatible-extension` | Add an AMCX ContextView projection that filters AMX/ECM material before model context assembly and references repository `ContextPack` rather than copying authority. |
| Retrieval | deterministic `select_context` for repository files | `compatible-extension` | Add pre-authorization/scope filtering for AMX/ECM before any semantic ranking. Existing repository retrieval remains read-only. |
| Agent/provider output | `ModelResponse` → `StructuredOutput` → `AgentAction` | `compatible-extension` | Normalize provider collaboration/memory claims into AMCX bridge objects before any action path. Existing typed action validation remains unchanged. |
| Schema publication | Git repository | `compatible-extension` | `protocols/amcx/registry/` is reviewed schema source. Runtime agents cannot activate schemas. |
| Prompt/skill/router promotion | capability-gap/evaluation infrastructure exists; no AMCX promotion ledger | `compatible-extension` | ECM may record candidates/evidence; activation stays separate from candidate producers and must use host deployment authority. |
| Aggregate budgets | `ExecPlan` has plan budgets; runtime has subsystem limits | `semantic-collision` if merged | Keep ExecPlan attempt/replan budgets canonical for execution planning. ECM owns only collaboration/provider aggregate budgets; adapters must not silently translate one into the other. |
| Durable checkpoints | typed `PlanCheckpoint` plus imported `.autodev` checkpoint contract | `compatible-extension` | Typed ExecPlan checkpoint remains lifecycle authority. `.autodev` checkpoint is recovery coordination metadata referencing typed state, not replacing it. |
| Hard purge | no full deletion coordinator | `missing` | Keep disabled/nonconformant until a separately authorized deletion protocol exists. |

## Non-negotiable boundaries

1. **No new execution entry point.** AMCX bridge code must not call filesystem, Git, process, network, deployment, or secret operations directly.
2. **No grant construction.** AMCX/ECM/AMX types must not construct `AuthorizationGrant` or set approval state.
3. **No self-verification.** AMCX evaluations and GateProfile PASS results are evidence inputs only; `VerificationFabric` remains verdict owner for current host checks.
4. **No duplicate plan lifecycle.** ECM task state may reference an ExecPlan/Task/Envelope, but cannot transition AutoDev plan or execution state implicitly.
5. **No duplicate evidence store.** AMX may retain durable knowledge about evidence and hashes, but current execution evidence remains in `EvidenceStore` or its future persistence backend.
6. **No context authority escalation.** `ContextView`, memory, retrieved repository text, skill descriptions, model output, and peer messages are all untrusted evidence.
7. **Fail closed on unresolved identity/scope.** Cross-repository, worktree, tenant, project, user, role, or provider scope must be explicit before retrieval or replay.

## First bridge boundary

The first implementation slice should be a pure mapping module in ForgeCore, tentatively `amcx_bridge.rs`, containing serializable reference/projection types and conversion functions only.

It should consume existing host objects and produce AMCX-facing references such as:

```text
ExecPlan / PlanCheckpoint ──► PlanRef
ExecutionEnvelope ──────────► TaskExecutionRef
Evidence / VerificationReport ──► EvidenceRef
ContextPack ────────────────► RepositoryContextRef
Model/provider metadata ────► ProviderRef
```

The module MUST NOT contain an executor, command runner, filesystem API, network client, approval constructor, policy mutator, or schema activation operation.

## Required first-slice tests

Before bridge implementation, add tests proving:

1. bridge projections serialize deterministically and retain source IDs/digests;
2. plan projection cannot alter `ExecPlan` lifecycle;
3. evidence projection does not manufacture a PASS verdict;
4. context projection contains references/metadata and does not mutate `ContextPack`;
5. no bridge public API returns `AuthorizationGrant`;
6. unknown/blank critical identity fields fail closed;
7. source objects round-trip unchanged through projection construction.

## Implementation order

1. Add `amcx_bridge` types and validation with TDD.
2. Add adapters from `ExecPlan`, `ExecutionEnvelope`, `Evidence`, `VerificationReport`, and `ContextPack`.
3. Add AMX persistence behind a new non-execution service boundary.
4. Add ECM collaboration state behind a separate orchestrator/service boundary.
5. Build `ContextView` from pre-filtered AMX + ECM + repository context references.
6. Integrate provider adapters after canonical host mapping is stable.
7. Add CI conformance/adversarial gates.

## Deferred deliberately

- hard purge;
- cross-project memory promotion;
- automatic skill/prompt activation;
- direct AMX/ECM persistence inside ForgeCore execution paths;
- networked memory service;
- semantic/vector retrieval before scope filtering is implemented and evaluated.

These remain deferred because implementing them in the first bridge slice would either widen authority or create a second source of truth.