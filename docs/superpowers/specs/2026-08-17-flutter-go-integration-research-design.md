# AutoDev Flutter + Go Integration Research Design

**Date:** 2026-08-17  
**Status:** Design approved for research/prototyping; no production implementation authorized by this document  
**Repository:** `asshat1981ar/AutoDev`

## 1. Purpose

Define evidence-backed integration points for Flutter/Dart and Go that improve AutoDev capability and UI/UX without creating duplicate authority, orchestration, execution, or persistence systems.

The system invariant remains:

```text
untrusted intent
  -> trusted authorization
  -> confined execution
  -> independently recorded evidence
  -> verification
  -> bounded replanning
```

Flutter and Go are deliberately outside ForgeCore's trusted execution authority.

## 2. Current repository baseline

Current `main` already contains the product surfaces that earlier integration plans treated as pending:

- PR #7 is merged: `kotlin/android-command-center` provides an observer-only Compose Android client and APK pipeline.
- PR #8 is merged: `crates/autodev-server` provides the Rust objective intake API, bounded SSE broadcast, and signed GitHub webhook intake.
- PR #12 is closed without merge and retained only as a source/recovery branch; future slices must start from current `main`.
- The Rust workspace contains `forge-core` and `autodev-server`.
- The Kotlin workspace contains `mpp-core`, `mpp-codegraph`, `mpp-server`, `mpp-ui`, and `android-command-center`.
- `mpp-core` contains the current code-graph extractor, an MCP JSON-RPC dispatcher, platform filesystem abstractions, semantic patch subagent code, and AST patch review.
- `mpp-codegraph` exposes a small symbol-query surface.
- `mpp-server` exposes a Ktor SSE sample/router.
- `mpp-ui` currently renders unified diffs to escaped HTML through a dependency-free Nano DSL.
- `protocols/` contains JSON Schema contracts for agent actions, execution envelopes, execution results, and tasks.
- No Flutter/Dart source and no Go module currently exists on `main`.

Important semantic finding: `KotlinTreeSitterParser` is currently a deterministic structural tokenizer, not a full Tree-sitter grammar binding. The implementation itself describes a future Tree-sitter-backed extractor as replaceable behind the existing graph surface.

## 3. Architectural decision

Use a **multi-surface architecture with one authority model**.

```text
Android Compose Command Center          Flutter AutoDev Studio
          |                                      |
          +-------------- HTTP/SSE --------------+
                             |
                     Rust autodev-server
                             |
                       ForgeCore APIs
                             |
              trusted policy / execution / evidence

External MCP / provider services
             |
      Go AutoDev Edge
             |
   normalized external observations
             |
      Rust control-plane adapter
             |
          ForgeCore
```

Flutter owns experience-intensive cross-platform presentation. Go owns external connection-intensive protocol adaptation. Rust retains authorization, orchestration, execution, evidence, and verification.

## 4. Flutter integration: AutoDev Studio

### 4.1 Responsibility

Create a new cross-platform operator/developer client under a future `flutter/autodev-studio` workspace.

Primary targets:

- desktop first for dense developer workflows;
- web as a secondary read/review surface;
- mobile support evaluated after the desktop prototype;
- do not replace the merged Android Compose command center during the prototype phase.

### 4.2 Initial capabilities

AutoDev Studio should focus on capabilities that the current Android observer does not provide:

1. **Objective workspace** — list, select, inspect, and submit untrusted objectives.
2. **Execution timeline** — visualize planning, execution, verification, blocked, retry, replan, completed, and failed events as a causal timeline.
3. **Evidence browser** — compiler, test, lint, security, benchmark, and device evidence grouped by objective/task/run.
4. **Semantic diff review** — file/AST-oriented diff presentation instead of only raw event strings.
5. **Code-graph explorer** — pan/zoom/search/filter visualization over the existing graph/query domain.
6. **Connectivity health** — show MCP/provider reachability, reconnect state, latency, and degraded external dependencies without treating connectivity as authority.

### 4.3 Why Flutter is a credible fit

Flutter currently supports Android, iOS, Windows, macOS, Linux, and web deployment. Its rendering APIs provide custom painting, direct repaint control, hit testing, semantics, and pan/zoom primitives suitable for graph and timeline prototypes. Impeller is the default renderer on current iOS and supported Android configurations and is designed around predictable precompiled rendering pipelines.

These are research reasons, not proof of superiority over Compose. The Flutter prototype must be benchmarked against an equivalent Compose implementation for the workloads AutoDev actually needs.

### 4.4 Flutter authority constraints

The Flutter process must not:

- mint `AuthorizationGrant` values;
- carry kernel capability grants;
- execute repository actions directly;
- call ForgeCore effect adapters directly;
- infer success from UI state;
- expose trusted approval references as reusable credentials.

Objective creation from Flutter is untrusted intent. Any future approval UI can request or present a trusted server-side approval workflow, but cannot manufacture approval authority.

## 5. Go integration: AutoDev Edge

### 5.1 Responsibility

Create a future `go/autodev-edge` module as an **external connectivity and MCP/provider transport boundary**, not another control-plane server.

The strongest current repository gap is transport lifecycle rather than JSON-RPC dispatch. Kotlin `McpToolHandler` dispatches already-parsed requests but does not own Streamable HTTP connection lifecycle, upstream health, compatibility, bounded fan-out, retry/backoff, or protocol-version negotiation.

### 5.2 Initial responsibilities

AutoDev Edge may own:

- outbound MCP Streamable HTTP and stdio connectivity;
- MCP protocol-version negotiation and compatibility handling;
- external provider connection health;
- bounded concurrency and rate limiting;
- reconnect/backoff and transient-failure classification;
- upstream capability discovery;
- transport-level backpressure;
- normalized telemetry/health events;
- optional fan-out of external observation streams.

It must not own:

- ForgeCore policy;
- repository capability grants;
- approval authority;
- repository mutation;
- verified orchestration;
- execution evidence acceptance;
- task completion decisions.

### 5.3 MCP-specific design requirement

MCP transport semantics are evolving. The current stable/draft documentation has changed session behavior across revisions, and current standards work adds transport headers for routing/observability. Therefore the Go edge must isolate protocol-version-specific behavior behind a small transport interface instead of leaking MCP revision details into ForgeCore or clients.

For Streamable HTTP it must implement the protocol's security requirements relevant to its role, including origin validation where it accepts inbound HTTP, loopback binding for local-only services, authentication where exposed, and protocol-version validation.

### 5.4 Go process boundary

Do not introduce Go through JNI/FFI.

The default design is a separate optional process with a typed protocol boundary. The prototype should compare two transports:

1. loopback HTTP using an explicit local authentication token;
2. stdio JSON-RPC when the Rust control plane owns the child process lifecycle.

Choose the smaller and more robust mechanism after prototype evidence. Remote deployment is out of scope for the first Go slice.

## 6. Shared public protocol fabric

Do not expose raw internal `ExecutionEnvelope` structures directly to Flutter or Go.

Add, in a later implementation slice, a versioned client/read-model schema family under `protocols/` using the repository's existing JSON Schema convention.

Candidate read models:

- `objective-summary`
- `objective-detail`
- `objective-event`
- `evidence-summary`
- `diff-summary`
- `code-graph-snapshot`
- `connectivity-status`
- `protocol-error`

Rules:

- public models contain observations and untrusted commands, not kernel-owned grants;
- enums have explicit unknown/evolution handling in generated/manual clients;
- every event has a stable ID, objective/run/task correlation where applicable, and a timestamp;
- SSE payloads use typed event envelopes rather than arbitrary strings;
- protocol compatibility tests must validate Rust, Kotlin, Dart, and Go implementations against canonical fixtures;
- JSON Schema remains the starting point because it is already the repository protocol format; protobuf/gRPC is not justified at this stage.

## 7. Code-graph integration

The code graph should remain owned by a replaceable analysis/domain layer rather than the Flutter renderer.

Target flow:

```text
source repository
  -> structural/Tree-sitter extractor
  -> AstSymbolGraph / richer graph domain
  -> read-only graph API
  -> Flutter visualization model
  -> CustomPainter / InteractiveViewer prototype
```

Flutter receives stable graph nodes/edges/spans and presentation metadata. It does not parse source to determine trusted repository structure.

The existing Kotlin structural tokenizer can continue to satisfy current tests while a separate future task evaluates a real Tree-sitter-backed extractor. That parser replacement is independent of whether Flutter is adopted.

## 8. Android and Flutter coexistence

The merged Compose Android command center remains the smallest mobile field-testing client.

Near-term division:

| Surface | Primary role |
| --- | --- |
| Android Compose | lightweight native command center, field testing, mobile status/notifications |
| Flutter Studio | dense cross-platform developer/operator workspace, graph/timeline/diff/evidence UX |

Only consider consolidating the Android UI into Flutter after measurable product evidence shows that maintaining two clients costs more than the native mobile specialization provides. Such consolidation would be a separate Major Change Gate.

## 9. Go and existing Rust/Kotlin server coexistence

Do not create a third generic AutoDev HTTP server.

- `autodev-server` remains the client-facing Rust control-plane adapter.
- Ktor `mpp-server` remains a KMP/server experimentation module until separately reconciled.
- Go Edge handles external connectivity only when it provides capabilities or isolation that are not already implemented in those servers.

If the Go prototype merely duplicates Axum/Tokio networking with no structural or measured benefit, remove it and keep the protocol adapter in Rust.

## 10. Agentic research decomposition

Run the work as independent evidence tracks before implementation decisions:

### ReconAgent
Map current producers, consumers, schemas, authority boundaries, async ownership, and active branches.

### FlutterExperienceAgent
Design Studio information architecture, graph/timeline/diff/evidence surfaces, accessibility, keyboard/pointer interaction, and adaptive layouts.

### ComposeDefender
Build the strongest Compose/KMP alternative for each Flutter proposal and identify duplication cost.

### GoConnectivityAgent
Design MCP/provider lifecycle, backpressure, reconnect, health, and compatibility boundaries.

### RustDefender
Determine whether Axum/Tokio can satisfy each Go candidate more simply.

### ProtocolAgent
Define canonical JSON Schemas, fixtures, correlation IDs, SSE event typing, and compatibility rules.

### SecurityAgent
Review authority leakage, local-service exposure, replay, approval spoofing, event forgery, DNS rebinding, authentication, and malformed inputs.

### PerformanceAgent
Own reproducible benchmark workloads and resource measurements.

### IntegrationReconciler
Merge the evidence into one recommendation and reject duplicated systems.

## 11. Prototype and measurement gates

### Flutter prototype

Build only after implementation planning is approved.

Exercise real typed event fixtures and a synthetic but reproducible workload:

- 10,000 timeline events;
- 5,000 graph nodes with bounded edge density;
- continuous incremental updates;
- diff/evidence navigation.

Measure:

- frame build/raster time;
- dropped/janky frames;
- startup;
- steady-state memory;
- CPU while streaming;
- binary size;
- interaction latency for pan/zoom/filter/select.

Compare the high-value graph/timeline slice with a Compose alternative before recommending broader Flutter migration.

### Go prototype

Use canonical MCP/provider fixtures and controlled mock upstreams.

Measure:

- connection establishment/reconnect latency;
- bounded concurrent connection behavior;
- memory/RSS;
- CPU at idle and under event flow;
- backpressure correctness;
- malformed-message handling;
- shutdown/cancellation behavior;
- implementation/deployment complexity versus a Rust/Tokio equivalent.

A performance win is not mandatory if the Go boundary provides strong protocol isolation and maintainability value, but that structural value must be explicit.

## 12. Research task graph

Execute in this dependency order:

```text
FG-00 current-repository semantic map
FG-01 public protocol gap analysis
FG-02 authority/security boundary map

FL-01 Studio information architecture
FL-02 typed Dart protocol feasibility
FL-03 SSE/reconnect client design
FL-04 timeline/evidence prototype design
FL-05 code-graph rendering prototype design
FL-06 Compose comparison plan

GO-01 MCP/provider transport gap analysis
GO-02 Go edge process-boundary comparison
GO-03 protocol-version compatibility design
GO-04 bounded concurrency/backpressure design
GO-05 Rust/Tokio comparison plan

PX-01 canonical public JSON Schema design
PX-02 cross-language fixture strategy
PX-03 typed SSE envelope design
PX-04 authentication/replay model

VR-01 Flutter benchmark plan
VR-02 Go benchmark plan
VR-03 security review plan
VR-04 integration/reconciliation decision
```

No production Flutter or Go module is added until the relevant prototype/spec slice has its own accepted implementation plan.

## 13. Major Change Gates

### Flutter production adoption

**CHANGE:** Add a second production UI technology.  
**Reason:** Dense cross-platform Studio UX may be substantially easier to deliver and iterate in Flutter.  
**Evidence required:** working prototype, Compose comparison, performance/resource measurements, maintenance analysis.  
**Risk:** duplicated client logic, larger CI/toolchain surface, design drift.  
**Alternative:** extend Compose/Compose Multiplatform.  
**Rollback:** delete Flutter workspace; public protocol remains reusable.  
**Recommendation:** prototype as a separate read/review client before any replacement discussion.

### Go production adoption

**CHANGE:** Add a new service/runtime language and process.  
**Reason:** isolate rapidly evolving external MCP/provider connectivity and connection-heavy lifecycle behavior from the trusted kernel.  
**Evidence required:** transport gap confirmed, prototype passes conformance/security tests, complexity is lower or capability meaningfully higher than Rust equivalent.  
**Risk:** deployment/toolchain/process complexity and duplicate networking.  
**Alternative:** implement the same adapter in Rust/Tokio.  
**Rollback:** retain schemas and port adapter into Rust.  
**Recommendation:** keep Go narrowly scoped to optional external connectivity; never make it a ForgeCore replacement.

## 14. Non-goals

This design does not authorize:

- replacing ForgeCore with Go;
- replacing the current Android app with Flutter;
- distributed workers;
- remote execution authority;
- a new database;
- a second orchestrator;
- Go JNI/FFI into Android;
- Flutter-to-Rust FFI for ordinary control-plane calls;
- client-side approval grants;
- unbounded event retention;
- direct external MCP authority over repository effects.

## 15. Success criteria

The research is complete when AutoDev has evidence sufficient to answer all of the following:

1. Which exact user workflows improve enough to justify Flutter?
2. Which exact external connectivity responsibilities improve enough to justify Go?
3. What stable public protocol keeps all clients independent of ForgeCore internals?
4. How are authority and approval prevented from leaking across new boundaries?
5. What measurable UX/performance/deployment costs are introduced?
6. Can each new component be removed without changing the trusted execution model?

The target is not a polyglot repository for its own sake. The target is one product in which each language owns a narrow workload that pays for its lifecycle cost.