# PRO-71 Android Objective Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend PR #7's Android command center into a typed objective submit/list/observe client for the PRO-66 control plane.

**Architecture:** Keep the native Compose app and debug-only cleartext rule. Split models, HTTP, SSE, ViewModel, and UI. The app submits untrusted intent and observes lifecycle/evidence; it never executes repository effects or creates trusted approval material.

**Tech Stack:** Kotlin 2.0.21, AGP 8.8.2, Compose, OkHttp 4.12.0, coroutines/StateFlow, kotlin.test.

## Global Constraints
- Preserve `dev.autodev.commandcenter`, minSdk 26, target/compile SDK 35 unless current Android tooling requires a documented change.
- Cleartext HTTP remains debug-only.
- `blocked` and `failed` are distinct states.
- Reconnect must re-fetch server truth.
- No approval mutation endpoint or approval token in the Android client.
- PRO-66 API/event contracts are authoritative.

---

## Files
- Import from PR #7: `kotlin/android-command-center/**`, root Kotlin plugin/settings changes.
- Split `MainActivity.kt` into `MainActivity.kt`, `CommandCenterScreen.kt`, `CommandCenterViewModel.kt`, `Models.kt`, `AutoDevApi.kt`, `AutoDevEventStream.kt`.
- Add unit tests under `kotlin/android-command-center/src/test/kotlin/dev/autodev/commandcenter/`.

### Task 1: Import PR #7 baseline
- [ ] Copy Android module and root Gradle/settings changes from PR #7 head `64c9087a8c8cd8b99a75bb4ac02b32c7f9150a5a`.
- [ ] Do not import CI edits yet; PRO-70 owns final CI composition.
- [ ] Run `./gradlew :android-command-center:assembleDebug :android-command-center:testDebugUnitTest ktlintCheck --no-daemon`.
- [ ] Commit `feat(android): import verified command-center baseline`.

### Task 2: Typed objective models
Create `Models.kt` with `ObjectiveStatus` values `QUEUED, PLANNING, RUNNING, BLOCKED, VERIFYING, REPLANNED, COMPLETED, FAILED`, plus `ObjectiveRequest`, `ObjectiveView`, and `ObjectiveEvent` matching the server JSON field names.

- [ ] RED: parse every server status and reject an unknown status.
- [ ] Implement one JSON codec only; use `kotlinx-serialization-json` if selected, otherwise a single explicit codec.
- [ ] GREEN: `./gradlew :android-command-center:testDebugUnitTest --tests '*ModelsTest*'`.
- [ ] Commit `feat(android): add typed objective lifecycle models`.

### Task 3: HTTP objective client
Create:

```kotlin
interface AutoDevApi {
    suspend fun health(endpoint: String): Result<Unit>
    suspend fun listObjectives(endpoint: String): Result<List<ObjectiveView>>
    suspend fun getObjective(endpoint: String, id: String): Result<ObjectiveView>
    suspend fun createObjective(endpoint: String, request: ObjectiveRequest): Result<ObjectiveView>
}
```

Implement `OkHttpAutoDevApi`; normalize endpoints with `trim().trimEnd('/')` and reject blanks before network access.

- [ ] RED with OkHttp MockWebServer: health, list, get, create-202, malformed JSON, non-2xx.
- [ ] Add `mockwebserver:4.12.0` as test dependency.
- [ ] GREEN focused API tests.
- [ ] Commit `feat(android): add typed AutoDev objective API client`.

### Task 4: Typed SSE transport
Create:

```kotlin
interface AutoDevEventStream {
    fun connect(
        endpoint: String,
        onEvent: (ObjectiveEvent) -> Unit,
        onClosed: (Throwable?) -> Unit,
    ): java.io.Closeable
}
```

Implement `OkHttpAutoDevEventStream` with zero read timeout only for SSE and a cancellable `Call`.

- [ ] RED: two `data:` frames arrive in order; malformed event closes with an error rather than crashing.
- [ ] GREEN focused SSE tests.
- [ ] Commit `feat(android): add typed SSE objective event stream`.

### Task 5: Lifecycle ViewModel
Create `CommandCenterState` with endpoint, connection state, repository, description, optional branch, submitting flag, objectives, selected objective, and recoverable error.

Public intents: `connect`, `disconnect`, `refresh`, setters for objective fields, `submitObjective`, `selectObjective`.

- [ ] RED fake-client tests: connect runs health+list, submit validates fields, SSE updates matching objective, disconnect cancels stream, reconnect refreshes state, BLOCKED remains BLOCKED.
- [ ] Implement dependency-injected ViewModel; production constructor wires OkHttp implementations.
- [ ] GREEN ViewModel tests.
- [ ] Commit `feat(android): manage objective lifecycle in command center`.

### Task 6: Compose objective screen
Split `MainActivity.kt` so it only hosts Compose. `CommandCenterScreen.kt` renders:

```text
AutoDev Command Center
server endpoint + connect/disconnect
new objective: repository + optional branch + description + submit
objective list: status, repository, description, current phase/task
blocked reason when blocked
latest evidence ref when present
recoverable connection/error card
```

Add stable semantics tags: `server_endpoint`, `connect_button`, `repository_input`, `objective_input`, `submit_objective`, `objective_list`, `connection_status`.

- [ ] Preserve text stating execution authority remains in ForgeCore.
- [ ] Add Preview with queued/running/blocked/completed examples.
- [ ] Run assemble, unit tests, ktlint.
- [ ] Commit `feat(android): add objective lifecycle command-center UI`.

### Task 7: Recovery hardening
- [ ] Tests: blank endpoint, server error, malformed event, SSE disconnect after connection, duplicate event, unknown objective event followed by refresh, blocked reason state.
- [ ] Verify no Android request contains an approval reference/grant field.
- [ ] Run `./gradlew :android-command-center:testDebugUnitTest :android-command-center:assembleDebug ktlintCheck --no-daemon`.
- [ ] Commit `test(android): harden objective lifecycle recovery`.

### Task 8: Full PRO-71 verification
- [ ] Run full Kotlin workspace build/tests/ktlint.
- [ ] Push and require GitHub Actions Rust/Kotlin/Python jobs green.
- [ ] Verify debug APK exists at `kotlin/android-command-center/build/outputs/apk/debug/android-command-center-debug.apk`.
- [ ] Attach CI evidence to Linear PRO-71 and move it to In Review.

## Self-Review
- Preserves PR #7 shell and network-security boundary.
- Implements every server lifecycle state explicitly.
- Adds submission and observation without approval authority.
- Reconnect uses durable server state rather than stale events.
- Device install/cold-launch/smoke remain PRO-70 responsibilities.
