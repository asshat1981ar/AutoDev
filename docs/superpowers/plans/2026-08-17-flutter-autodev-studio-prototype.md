# Flutter AutoDev Studio Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a removable desktop-first Flutter prototype that proves or rejects Flutter as AutoDev's dense objective/timeline/evidence/code-graph review surface.

**Architecture:** Create `flutter/autodev_studio` as an independent untrusted client of the Rust public HTTP/SSE protocol. Keep state and networking inside the Flutter process, use no FFI, and render graph/timeline workloads directly with Flutter primitives so the benchmark measures Flutter rather than a third-party graph engine.

**Tech Stack:** Flutter 3.44.7 stable baseline, Dart 3.12.x, Material, `package:http`, Flutter test, `CustomPainter`, `InteractiveViewer`, `ChangeNotifier`/`ValueNotifier`; no Riverpod/BLoC/code generation in the prototype.

## Global Constraints

- Desktop Linux is the first executable target; web must compile but is not the benchmark authority for this slice.
- Flutter never mints approvals, receives `AuthorizationGrant`, invokes ForgeCore adapters, or mutates repository files directly.
- The client consumes only `protocols/public/v1` models.
- No Flutter-to-Rust FFI/JNI.
- No third-party graph rendering package; benchmark Flutter's own rendering primitives.
- Maintain bounded event state: maximum 10,000 retained timeline events in the synthetic benchmark and a configurable lower live default.
- The merged Compose Android command center remains unchanged except for protocol compatibility fixes if the public SSE envelope requires them.

---

## File Structure

- Create `flutter/autodev_studio/pubspec.yaml` — Flutter app dependencies/tool constraints.
- Create `flutter/autodev_studio/lib/main.dart` — app bootstrap only.
- Create `flutter/autodev_studio/lib/protocol/models.dart` — typed public v1 models.
- Create `flutter/autodev_studio/lib/protocol/api_client.dart` — objective HTTP client.
- Create `flutter/autodev_studio/lib/protocol/sse_client.dart` — cancellable reconnecting SSE client.
- Create `flutter/autodev_studio/lib/state/studio_controller.dart` — explicit application-state boundary.
- Create `flutter/autodev_studio/lib/ui/studio_screen.dart` — adaptive shell.
- Create `flutter/autodev_studio/lib/ui/objective_list.dart` — objective selection.
- Create `flutter/autodev_studio/lib/ui/execution_timeline.dart` — virtualized event list.
- Create `flutter/autodev_studio/lib/ui/code_graph_view.dart` — custom-painted graph.
- Create `flutter/autodev_studio/lib/ui/evidence_panel.dart` — evidence projection.
- Create `flutter/autodev_studio/test/...` — protocol, controller, widget and painter tests.
- Create `flutter/autodev_studio/benchmark/generate_fixture.dart` — deterministic 10k-event/5k-node data generation.
- Modify `.github/workflows/ci.yml` only after the prototype is independently green.

### Task 1: Scaffold a minimal desktop-first Flutter application

**Interfaces:**
- Produces `AutoDevStudioApp` and `StudioScreen`.
- `pubspec.yaml` SDK floor: `dart: ">=3.12.0 <4.0.0"`.

- [ ] **Step 1: Create the project with Flutter 3.44.7 and record `flutter --version` in the commit/PR evidence.**
- [ ] **Step 2: Set `pubspec.yaml` dependencies to Flutter SDK plus `http: ^1.4.0`; do not add state-management or graph packages.**
- [ ] **Step 3: Add a widget test asserting the app renders `AutoDev Studio` and a disconnected status indicator.**
- [ ] **Step 4: Run `flutter test` and confirm failure before implementing the shell.**
- [ ] **Step 5: Implement `main.dart` and a minimal `StudioScreen`; run `flutter analyze` and `flutter test`.**
- [ ] **Step 6: Commit with `git commit -m "feat(flutter): scaffold AutoDev Studio prototype"`.**

### Task 2: Implement typed v1 public protocol models

**Files:** `lib/protocol/models.dart`, `test/protocol/models_test.dart`, canonical fixtures from `protocols/public/v1/fixtures`.

**Interfaces:**

```dart
final class ObjectiveSummary {
  const ObjectiveSummary({
    required this.id,
    required this.repository,
    required this.description,
    required this.branch,
    required this.status,
  });

  final String id;
  final String repository;
  final String description;
  final String branch;
  final String status;

  factory ObjectiveSummary.fromJson(Map<String, Object?> json) {
    return ObjectiveSummary(
      id: json['id']! as String,
      repository: json['repository']! as String,
      description: json['description']! as String,
      branch: json['branch']! as String,
      status: json['status']! as String,
    );
  }
}
```

Also define immutable `ObjectiveEvent`, `EvidenceSummary`, `GraphNode`, `GraphEdge`, `CodeGraphSnapshot`, and `ConnectivityStatus`. Unknown event `type` values remain parseable as strings so old clients do not crash on additive server event types.

- [ ] **Step 1: Copy canonical fixtures into test assets by script or test-relative repository path; do not duplicate fixture contents manually.**
- [ ] **Step 2: Write failing fixture decode tests.**
- [ ] **Step 3: Implement strict field extraction with `FormatException` for missing/wrong required fields.**
- [ ] **Step 4: Run `flutter test test/protocol/models_test.dart`. Expected: PASS.**
- [ ] **Step 5: Commit `feat(flutter): add typed AutoDev public protocol`.**

### Task 3: Implement cancellable HTTP and SSE transport

**Interfaces:**
- `AutoDevApi.listObjectives() -> Future<List<ObjectiveSummary>>`.
- `AutoDevApi.createObjective({required String repository, required String description, String? branch}) -> Future<ObjectiveSummary>`.
- `AutoDevEventStream.connect(Uri endpoint) -> Stream<ObjectiveEvent>`.
- `AutoDevEventStream.close() -> Future<void>`.

- [ ] **Step 1: Add HTTP tests using a local `HttpServer` fixture that returns the canonical public JSON.**
- [ ] **Step 2: Add SSE tests with `data:` frames, blank-line event delimiters, malformed frames, cancellation, and server disconnect.**
- [ ] **Step 3: Implement the clients with an injected `http.Client` and explicit close ownership. Avoid globals.**
- [ ] **Step 4: Reconnect with bounded exponential delays of 250 ms, 500 ms, 1 s, 2 s, then 5 s maximum; reset after a successful event. Cancellation must stop reconnect immediately.**
- [ ] **Step 5: Run protocol/transport tests and `flutter analyze`.**
- [ ] **Step 6: Commit `feat(flutter): add objective HTTP and SSE transport`.**

### Task 4: Add the Studio controller and bounded observable state

**Interfaces:**
- `StudioController extends ChangeNotifier`.
- State owns `objectives`, `selectedObjectiveId`, `events`, `connectionState`, and `lastError`.
- Live event retention defaults to 2,000; synthetic benchmark mode allows 10,000.

- [ ] **Step 1: Write tests for connect, refresh, select objective, event append, 2,000-event eviction, disconnect, and recoverable network error.**
- [ ] **Step 2: Implement `StudioController` with one owned stream subscription and no work on the UI isolate beyond JSON model construction/state mutation.**
- [ ] **Step 3: Ensure `dispose` cancels the subscription and closes owned transport resources.**
- [ ] **Step 4: Run `flutter test test/state/studio_controller_test.dart` and `flutter analyze`.**
- [ ] **Step 5: Commit `feat(flutter): add bounded Studio application state`.**

### Task 5: Build objective/timeline/evidence review UX

**Files:** `studio_screen.dart`, `objective_list.dart`, `execution_timeline.dart`, `evidence_panel.dart` and widget tests.

**Layout:**
- >= 1100 logical px: three panes: objectives | timeline | detail/evidence.
- < 1100 logical px: two panes with detail opened through selection.
- < 700 logical px: single-pane navigation for prototype usability only; Android replacement is not in scope.

- [ ] **Step 1: Write widget tests for wide/narrow layout, empty state, selected objective, blocked event, failed event, and evidence detail.**
- [ ] **Step 2: Implement with `LayoutBuilder`, `ListView.builder`, keyboard-focusable rows, semantic labels, and no unbounded nested scrollables.**
- [ ] **Step 3: Represent blocked/failed/verifying states with text/icon semantics in addition to visual styling.**
- [ ] **Step 4: Run widget tests and `flutter analyze`.**
- [ ] **Step 5: Commit `feat(flutter): add objective and evidence workspace`.**

### Task 6: Build direct Flutter code-graph rendering prototype

**Interfaces:**
- `CodeGraphView(snapshot: CodeGraphSnapshot)`.
- `CodeGraphPainter extends CustomPainter`.
- Deterministic layout input is supplied by fixture data; this task evaluates rendering/interaction, not graph-layout algorithms.

- [ ] **Step 1: Add painter tests for empty graph, 5-node graph, node selection hit testing, and unchanged-snapshot `shouldRepaint == false`.**
- [ ] **Step 2: Implement `InteractiveViewer` + `CustomPaint`; draw only viewport-relevant nodes/edges after transforming bounds.**
- [ ] **Step 3: Add search/filter controls outside the painter and keep selected node in controller-local presentation state.**
- [ ] **Step 4: Run tests/analyze.**
- [ ] **Step 5: Commit `feat(flutter): prototype interactive code graph`.**

### Task 7: Add deterministic benchmark workload and collect Flutter evidence

- [ ] Generate exactly 10,000 events and 5,000 nodes with a fixed seed `20260817`.
- [ ] Add a benchmark mode reachable with `--dart-define=AUTODEV_BENCHMARK=true`; it must not change production transport behavior.
- [ ] Run desktop profile build and collect startup, frame build/raster timing, steady-state RSS, CPU during event ingestion, graph pan/zoom interaction latency, and binary size.
- [ ] Store machine/toolchain metadata and raw measurements under `docs/benchmarks/flutter-studio/`.
- [ ] Do not claim superiority over Compose until the reconciliation plan runs the comparison slice.

### Task 8: CI gate after local prototype is green

Modify `.github/workflows/ci.yml` to add a `flutter` job that installs the pinned stable toolchain, runs `flutter pub get`, `flutter analyze`, `flutter test`, and `flutter build linux --debug` on Ubuntu with required desktop packages.

Run all existing Rust/Kotlin/Python jobs unchanged. Commit with `ci: verify Flutter Studio prototype`.

## Done Gate

The Flutter prototype is complete only when typed fixtures decode, objective/SSE state is bounded and cancellable, dense workspace and graph tests pass, profile benchmark evidence is recorded, Linux desktop builds, web compilation is separately checked, and no trusted authority crossed into Flutter.