# AutoDev Working APK v0 Integration Design

## Status

Approved architecture: **A — Android Compose command center → Rust control plane → ForgeCore trusted execution**.

This design integrates the already-green Android command-center work from PR #7 and the already-green Rust control-plane work from PR #8 with the verified execution/orchestration primitives already present on `main`.

The objective is not to create a second orchestration stack. The objective is to turn the existing pieces into one coherent, installable Android product while preserving AutoDev's authority boundary:

> Agents and clients propose typed intent. Policy authorizes capabilities. Trusted components execute. Independent verifiers produce evidence. Orchestration advances, blocks, replans, or completes from that evidence.

## 1. Product outcome

Working APK v0 is a mobile command center for AutoDev's verified development runtime.

A user can:

1. open the Android app;
2. configure an AutoDev control-plane endpoint;
3. submit a repository objective;
4. see the objective enter the AutoDev task lifecycle;
5. observe lifecycle events and current state;
6. see whether work is queued, running, blocked for approval, verifying, replanned, completed, or failed;
7. reconnect after transient network loss and recover the current server state.

The Android application is not a trusted executor. It does not receive repository-write credentials, mint `AuthorizationGrant`s, or execute repository mutations directly.

## 2. Verified repository ground truth

### `main`

`main` already contains the trusted development primitives that must remain authoritative:

- `TaskGraph` scheduling;
- `ExecutionEnvelope` lifecycle;
- `DevelopmentLoop`;
- `VerifiedOrchestrator` and serializable `VerifiedOrchestratorState`;
- kernel-owned approval handling;
- capability/risk policy checks;
- trusted execution adapters;
- evidence storage and verification gates;
- bounded repository context primitives;
- Cline/Termux development fabric.

`VerifiedOrchestrator::advance()` already maps execution outcomes into task state and `resume_approved()` reuses the existing envelope without consuming an execution attempt. `DevelopmentLoop` already rejects capability mismatch, risk mismatch, weakened approval policy, missing approval, and missing required verification evidence.

### PR #7 — Android command center

PR #7 already provides:

- `kotlin/android-command-center`;
- Android application ID `dev.autodev.commandcenter`;
- Jetpack Compose UI;
- endpoint configuration;
- SSE connection to `/events`;
- bounded in-memory event display;
- debug-only cleartext HTTP for emulator/LAN development;
- release manifest without cleartext opt-in;
- GitHub Actions Android SDK setup;
- deterministic `assembleDebug` build;
- uploaded APK artifact.

GitHub Actions run `32007192996` is green and produced artifact `autodev-command-center-debug`.

Artifact ZIP SHA-256:

`9b26d4ae8e89eb08fa2800afeddf9f131f281fedba5312541c6559feb9c6ad1c`

Extracted APK SHA-256:

`983ec1d7d5360eb1f4c6b82c4e7b951ac10274af952f3a3d430fca04315ce110`

### PR #8 — Rust control plane

PR #8 already provides:

- `autodev-server` Axum service;
- `GET /health`;
- `GET /api/v1/objectives`;
- `POST /api/v1/objectives`;
- `GET /api/v1/events/stream` and `/events` SSE;
- typed objective intake creating a ForgeCore `TaskGraph`;
- fail-closed GitHub webhook verification;
- Docker packaging.

Its current boundary is intentionally queue-only: it creates `TaskGraph`s but does not yet advance them through trusted execution.

## 3. Architecture decision

### Selected: Android-native UI + Rust control plane + ForgeCore kernel

```text
Android APK
  |
  | HTTP/SSE: untrusted product intent and observation
  v
AutoDev control plane (Axum)
  |
  | typed objective/run commands
  v
Trusted orchestration service
  |
  v
ForgeCore VerifiedOrchestrator
  |
  +--> policy / capability / approval checks
  +--> trusted execution
  +--> evidence store
  +--> VerificationFabric
  |
  v
objective/run lifecycle events
  |
  +------------------------------> Android APK
```

### Why this architecture

- Reuses two already-green development slices rather than replacing them.
- Keeps Android product code out of the trusted execution boundary.
- Keeps ForgeCore as the only authority for repository effects.
- Allows the mobile client to evolve independently from kernel internals.
- Supports emulator/LAN/mobile workflows immediately.
- Allows later remote, Termux-local, or embedded runtimes without changing the Android product contract.

### Rejected for v0

**Compose Multiplatform shared UI:** useful later, but migration would not prove the Android product sooner.

**Embed ForgeCore directly in the APK:** increases JNI/native packaging, Android process lifecycle, sandbox, storage, and security complexity before the product workflow is proven.

## 4. Trust boundaries

### Android client

The Android app may:

- submit objective descriptions;
- submit repository identifiers and optional branch hints;
- request objective state;
- subscribe to lifecycle events;
- display verification/evidence summaries;
- display that approval is required.

The Android app may not:

- create or supply a kernel-trusted `AuthorizationGrant` in v0;
- declare itself authorized through JSON fields;
- directly run shell/Git/filesystem operations;
- expand task capabilities;
- bypass verification requirements.

### Control plane

The control plane is a protocol adapter and lifecycle coordinator. It validates API input, persists/retrieves objective state, emits events, and submits typed work to the trusted kernel integration.

It must not duplicate ForgeCore policy decisions or treat client-declared approval as trusted authority.

### ForgeCore

ForgeCore remains authoritative for:

- capability checks;
- risk checks;
- approval requirements;
- execution;
- evidence;
- verification;
- verified/replanned/failed result semantics.

## 5. Objective lifecycle contract

Replace free-form status strings at the API boundary with a closed lifecycle enum serialized as snake case:

- `queued`
- `planning`
- `running`
- `blocked`
- `verifying`
- `replanned`
- `completed`
- `failed`

The API view must expose enough information for the app without exposing kernel-owned secrets or mutable internal authorization state.

Minimum objective view:

```json
{
  "id": "uuid",
  "repository": "owner/repo",
  "description": "Implement feature X",
  "branch": "autodev/objective-ab12cd34",
  "status": "queued",
  "current_task_id": null,
  "current_phase": null,
  "latest_evidence_ref": null,
  "blocked_reason": null
}
```

Internal `TaskGraph`, envelopes, execution details, and trusted approval material remain server/kernel state rather than writable API fields.

## 6. Control-plane execution integration

The existing `AppState` mixes API state and in-memory objective storage. v0 should introduce explicit responsibilities:

### `ObjectiveStore`

Owns durable objective metadata and serialized execution state.

Responsibilities:

- create objective record;
- fetch/list objective records;
- persist lifecycle transitions;
- persist `TaskGraph` plus `VerifiedOrchestratorState` or an equivalent recovery snapshot;
- reject unknown objective/task IDs.

A simple local durable store is preferred for v0. Do not add Supabase, Neon, Qdrant, or another network database unless executable evidence proves the local store cannot meet the v0 recovery contract.

### `ObjectiveRunner`

Bridges objective records into existing ForgeCore orchestration.

Responsibilities:

- create or restore the ForgeCore execution state for an objective;
- advance work through the existing `VerifiedOrchestrator` path;
- translate ForgeCore task/phase outcomes to the API lifecycle enum;
- emit lifecycle events;
- stop on completion, terminal failure, approval block, or explicit attempt budget exhaustion;
- never invent approval references.

The runner must serialize state-changing ForgeCore execution for a given objective. PR #10 async delegation may later parallelize eligible read-only/compute work, but v0 must not require concurrent repository mutation.

### Production composition gap

`main` proves `VerifiedOrchestrator` through focused tests, but PR #8 does not yet compose it into a long-running production service. PRO-66 owns this integration. The implementation must reuse existing `Decomposer`, `Assigner`, `DevelopmentLoop`, `VerificationFabric`, workspace, envelope, and authorization contracts rather than implementing parallel substitutes.

## 7. API contract

Preserve:

- `GET /health`
- `POST /api/v1/objectives`
- `GET /api/v1/objectives`
- `GET /events` and `/api/v1/events/stream`

Add:

- `GET /api/v1/objectives/{id}`

The create response remains `202 Accepted` because objective completion is asynchronous from the caller's perspective.

For v0, there is no client approval mutation endpoint. When ForgeCore reports approval required, the objective becomes `blocked`; the Android app displays the block. A later design may add approval/resume only if the server can bind an authenticated human action to a kernel-owned grant.

## 8. Event contract

SSE events are JSON objects with stable event type and objective identity.

Minimum types:

- `objective_queued`
- `objective_planning`
- `objective_running`
- `objective_blocked`
- `objective_verifying`
- `objective_replanned`
- `objective_completed`
- `objective_failed`

Minimum payload:

```json
{
  "type": "objective_running",
  "objective_id": "uuid",
  "task_id": "task-1",
  "phase": "act",
  "status": "running",
  "evidence_ref": null,
  "message": "executing verified task attempt"
}
```

Events are observational evidence for the product UI; they do not authorize actions.

## 9. Android product design

Preserve the existing Android-native Compose shell.

Split the current single `MainActivity.kt` responsibilities as the product grows:

- `MainActivity.kt`: Android entry point only.
- `CommandCenterScreen.kt`: screen rendering and user interaction.
- `CommandCenterViewModel.kt`: lifecycle state and user intents.
- `AutoDevApi.kt`: HTTP objective create/list/get operations.
- `AutoDevEventStream.kt`: SSE lifecycle transport.
- `Models.kt`: API DTOs and UI state mapping.

The first coherent user journey is:

```text
Launch
  -> configure endpoint
  -> server health check
  -> list current objectives
  -> enter repository + objective
  -> submit
  -> observe queued/running/verifying/etc.
  -> recover from disconnect by re-fetching objective state
```

Errors must be recoverable and explicit:

- invalid endpoint/input;
- server unavailable;
- HTTP rejection;
- SSE disconnect;
- malformed event payload;
- objective blocked;
- objective failed.

The UI must distinguish `blocked` from `failed`.

## 10. Context engineering boundary

The mobile API does not transmit arbitrary full-repository context. Repository context selection remains server-side/ForgeCore-side and bounded by existing repository context primitives.

For each objective, the durable handoff is limited to stable identifiers and bounded state references:

- objective ID;
- task graph identity/state;
- execution envelope state;
- context references;
- evidence references;
- current lifecycle state.

This prevents the Android app from becoming a second context store and avoids transmitting unnecessary code to the device.

## 11. Verification strategy

### Rust/control-plane

TDD is mandatory.

Required focused tests include:

- objective lifecycle enum serialization;
- valid create/list/get behavior;
- objective runner maps ForgeCore completion to `completed`;
- verification phase is surfaced;
- missing approval maps to `blocked` without executing an unauthorized retry;
- failed verification maps through replan semantics;
- exhausted attempts map to `failed`;
- recovery restores objective graph/orchestrator state;
- client payload cannot inject approval authority;
- SSE emits lifecycle transitions in objective order.

### Android

Required tests include:

- request/response DTO parsing;
- objective submission state transition;
- objective list refresh;
- SSE event mapping;
- disconnect/reconnect recovery;
- blocked vs failed presentation;
- malformed event/API error handling.

### Repository gates

All existing gates remain required:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
./gradlew clean test ...
./gradlew ktlintCheck
Python development-fabric tests
```

The Android build must continue to produce and upload the debug APK.

## 12. APK device evidence gate

A green Gradle build is necessary but not sufficient.

Working APK v0 is not complete until CI or another reproducible Android test environment proves:

1. the produced APK is installed;
2. cold launch succeeds;
3. the main command-center screen renders;
4. endpoint controls are interactable;
5. the app connects to a test control-plane instance;
6. objective submission succeeds;
7. at least one lifecycle event is displayed;
8. APK SHA-256 is recorded;
9. critical/high release defects are zero.

Prefer a Gradle Managed Device or another reproducible emulator path supported by current official Android tooling. Device/emulator tests must not weaken existing release network-security settings merely to make tests pass.

## 13. Market/competitor validation

Current competitor evidence supports AutoDev investing in:

- bounded repository context rather than sending the whole repository;
- isolated/typed multi-agent delegation;
- task/worktree coordination;
- model-agnostic execution;
- sandboxed/trusted effect boundaries;
- verification/evaluation loops.

AutoDev should not copy competitor surface area feature-for-feature. Its v0 differentiation is:

**mobile/Termux-oriented control + model/provider independence + capability-bound execution + explicit evidence/verification.**

The Android product should expose those verified lifecycle semantics rather than becoming a generic chat or IDE clone.

## 14. Scope decisions

### PRESERVE

- PR #7 Android shell and APK pipeline.
- PR #8 Axum API/SSE adapter.
- ForgeCore trusted execution, policy, evidence, verification, and orchestration.
- existing Cline agent/skill/MCP fabric.

### REFACTOR / RESCOPE

- PR #8 queue-only state into an objective lifecycle service backed by ForgeCore execution.
- Android observer-only event screen into objective submit/list/observe workflow.
- generic project dashboard work into run/evidence/approval visibility.

### DEFER

- broader MCP registry work;
- iOS product work;
- remote SaaS persistence;
- embedded ForgeCore inside APK;
- Android approval mutation UI;
- automatic merge/deployment;
- PR #10 async delegation as a release blocker.

## 15. Work breakdown alignment

The `Working APK v0` milestone is ordered:

1. **PRO-66** — bridge control plane to ForgeCore verified orchestration.
2. **PRO-71** — complete Android objective lifecycle command center.
3. **PRO-70** — install/cold-launch/smoke/artifact integrity gates.
4. **PRO-69** — evidence and approval visibility after the core lifecycle is green.

Each slice must remain independently reviewable and green before the next slice is integrated.

## 16. Stop / rescope conditions

Stop and issue a rescope decision if implementation requires any of the following:

- bypassing ForgeCore policy to make the APK workflow work;
- trusting approval data supplied by the Android client;
- duplicating `VerifiedOrchestrator` semantics in the server;
- adding a remote database without a demonstrated v0 requirement;
- embedding unrestricted credentials in the APK;
- allowing concurrent repository mutation to satisfy responsiveness;
- weakening release network security for test convenience;
- marking device verification complete without an actual install/launch test.

## 17. Definition of done

Working APK v0 is done only when:

- Architecture A is implemented without authority duplication;
- the Android APK can create and observe an objective end to end;
- control-plane lifecycle state is driven by ForgeCore execution/verification evidence;
- approval-required work stops visibly in `blocked` state;
- build/test/lint/security gates are green;
- a real APK artifact is produced with recorded SHA-256;
- install and cold launch are reproducibly verified;
- critical smoke journey passes;
- build/install/run instructions are documented;
- no unresolved critical/high defect remains.
