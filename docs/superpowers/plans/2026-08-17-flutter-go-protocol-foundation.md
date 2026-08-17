# Flutter + Go Public Protocol Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a versioned, authority-safe public protocol that Rust, Kotlin, Dart, and Go can consume without exposing ForgeCore-internal authorization state.

**Architecture:** Keep ForgeCore internal execution envelopes private. Add JSON Schema read/command models under `protocols/public/v1`, mirror them with Rust transport structs in `autodev-server`, and emit typed SSE events. Later Flutter and Go plans consume the same canonical fixtures.

**Tech Stack:** JSON Schema Draft 2020-12, Rust 2021, Axum 0.7, Serde/serde_json, Tokio broadcast, Kotlin 2.0.21 only for compatibility regression checks.

## Global Constraints

- ForgeCore remains the only trusted authorization/execution authority.
- Public protocol objects must never contain `AuthorizationGrant` or reusable approval credentials.
- Objective submission is untrusted intent.
- JSON Schema remains the canonical public wire contract for this slice.
- Existing `/events` remains available while `/api/v1/events/stream` becomes the canonical typed stream.
- Event retention stays bounded by the existing broadcast capacity.
- No database, protobuf, gRPC, JNI, FFI, Flutter, or Go implementation is introduced in this plan.

---

## File Structure

- Create `protocols/public/v1/objective-summary.schema.json` — list/read model.
- Create `protocols/public/v1/objective-create.schema.json` — untrusted submission contract.
- Create `protocols/public/v1/objective-event.schema.json` — typed SSE envelope.
- Create `protocols/public/v1/evidence-summary.schema.json` — verifier evidence projection.
- Create `protocols/public/v1/code-graph-snapshot.schema.json` — read-only graph projection.
- Create `protocols/public/v1/connectivity-status.schema.json` — Go edge/provider observation model.
- Create `protocols/public/v1/protocol-error.schema.json` — stable public errors.
- Create `protocols/public/v1/fixtures/*.json` — canonical cross-language fixtures.
- Create `crates/autodev-server/src/public_protocol.rs` — Rust wire types and conversions.
- Modify `crates/autodev-server/src/lib.rs` — typed broadcaster/routes.
- Add tests in `crates/autodev-server/src/public_protocol.rs` and `crates/autodev-server/src/lib.rs`.

### Task 1: Add authority-safe public schemas and fixtures

**Files:** all `protocols/public/v1/*` paths above.

**Interfaces:**
- Produces wire fields `schema_version`, `event_id`, `type`, `timestamp`, `objective_id`, `run_id`, `task_id`, and `data` for event envelopes.
- Produces objective status enum: `queued`, `planning`, `running`, `blocked`, `verifying`, `replanned`, `completed`, `failed`.

- [ ] **Step 1: Create `objective-event.schema.json` first and include this complete top-level shape**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://autodev.dev/protocols/public/v1/objective-event.schema.json",
  "title": "AutoDev Objective Event v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "event_id", "type", "timestamp", "objective_id", "data"],
  "properties": {
    "schema_version": { "const": "1" },
    "event_id": { "type": "string", "minLength": 1 },
    "type": { "type": "string", "minLength": 1 },
    "timestamp": { "type": "string", "format": "date-time" },
    "objective_id": { "type": "string", "minLength": 1 },
    "run_id": { "type": ["string", "null"] },
    "task_id": { "type": ["string", "null"] },
    "data": { "type": "object", "additionalProperties": true }
  }
}
```

- [ ] **Step 2: Add the remaining schemas with `additionalProperties: false` on stable envelopes and explicit nullable correlation IDs where applicable.**

- [ ] **Step 3: Add canonical fixtures**

`objective-event.queued.json` must be:

```json
{
  "schema_version": "1",
  "event_id": "evt-0001",
  "type": "objective_queued",
  "timestamp": "2026-08-17T12:00:00Z",
  "objective_id": "obj-0001",
  "run_id": null,
  "task_id": null,
  "data": {
    "repository": "owner/repo",
    "branch": "autodev/objective-obj0001",
    "status": "queued"
  }
}
```

- [ ] **Step 4: Verify every JSON document parses**

```bash
python - <<'PY'
import json
from pathlib import Path
for path in Path('protocols/public/v1').rglob('*.json'):
    json.loads(path.read_text())
    print(path)
PY
```

Expected: every file path prints and the process exits 0.

- [ ] **Step 5: Commit**

```bash
git add protocols/public/v1
git commit -m "feat(protocol): add public v1 read models"
```

### Task 2: Add Rust transport types without exposing ForgeCore authority

**Files:**
- Create: `crates/autodev-server/src/public_protocol.rs`
- Modify: `crates/autodev-server/src/lib.rs`

**Interfaces:**
- Produces `pub const PUBLIC_SCHEMA_VERSION: &str = "1"`.
- Produces `PublicObjectiveSummary`, `PublicObjectiveCreate`, `PublicObjectiveEvent`, `PublicProtocolError`.
- Produces `PublicObjectiveEvent::queued(&ObjectiveRecord) -> PublicObjectiveEvent`.

- [ ] **Step 1: Write failing serde round-trip tests against the canonical queued fixture.**

Test code must deserialize with `serde_json::from_str::<PublicObjectiveEvent>` and assert `schema_version == "1"`, `event_type == "objective_queued"`, and `objective_id == "obj-0001"`.

- [ ] **Step 2: Run**

```bash
cd crates
cargo test -p autodev-server public_protocol -- --nocapture
```

Expected: FAIL because `public_protocol` is missing.

- [ ] **Step 3: Implement the types with explicit serde names**

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PUBLIC_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicObjectiveEvent {
    pub schema_version: String,
    pub event_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub objective_id: String,
    pub run_id: Option<String>,
    pub task_id: Option<String>,
    pub data: Value,
}
```

Use RFC3339 UTC timestamps and UUID event IDs at production event creation. Do not serialize `TaskGraph`, capabilities, policy grants, or approval references into public event payloads.

- [ ] **Step 4: Re-run focused tests**

```bash
cd crates
cargo test -p autodev-server public_protocol -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/autodev-server/src/public_protocol.rs crates/autodev-server/src/lib.rs
git commit -m "feat(server): add typed public protocol models"
```

### Task 3: Replace string SSE broadcasting with typed public events

**Files:** `crates/autodev-server/src/lib.rs` and its tests.

**Interfaces:**
- Change `events: broadcast::Sender<String>` to `events: broadcast::Sender<PublicObjectiveEvent>`.
- `event_stream` serializes each typed event once at the Axum boundary.

- [ ] **Step 1: Add a failing test that subscribes to `state.events`, calls `enqueue`, and asserts the received event is `objective_queued` with the returned objective ID.**
- [ ] **Step 2: Run `cargo test -p autodev-server objective_event -- --nocapture` and confirm failure.**
- [ ] **Step 3: Change the broadcaster type and `enqueue` construction; keep capacity `256`.**
- [ ] **Step 4: In `event_stream`, call `serde_json::to_string(&event)` and drop serialization failures rather than emitting malformed SSE.**
- [ ] **Step 5: Run `cargo test -p autodev-server`. Expected: PASS.**
- [ ] **Step 6: Commit with `git commit -m "refactor(server): emit typed objective events"`.**

### Task 4: Add public objective projection and preserve internal task graph isolation

**Files:** `crates/autodev-server/src/public_protocol.rs`, `crates/autodev-server/src/lib.rs`.

**Interfaces:**
- `impl From<&ObjectiveRecord> for PublicObjectiveSummary`.
- `GET /api/v1/objectives` returns `Vec<PublicObjectiveSummary>` rather than internal `ObjectiveRecord`.
- `POST /api/v1/objectives` accepts `PublicObjectiveCreate` and returns `PublicObjectiveSummary`.

- [ ] **Step 1: Add a failing API test asserting the response omits `graph`.**
- [ ] **Step 2: Run the focused API test and confirm current behavior fails because `graph` is present.**
- [ ] **Step 3: Implement conversions and route projections.**
- [ ] **Step 4: Add a regression assertion that `repository`, `description`, `branch`, `id`, and `status` remain present.**
- [ ] **Step 5: Run `cargo test -p autodev-server`. Expected: PASS.**
- [ ] **Step 6: Commit with `git commit -m "feat(server): isolate public objective read model"`.**

### Task 5: Run repository verification and record the protocol baseline

- [ ] Run:

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
cd ../kotlin
./gradlew clean test :mpp-core:assemble :mpp-server:assemble :mpp-ui:assemble :mpp-codegraph:assemble :android-command-center:assembleDebug --no-daemon
./gradlew ktlintCheck --no-daemon
```

- [ ] Confirm the Android observer still receives the SSE payload as a string even though the payload is now typed JSON.
- [ ] Record exact commands/results in the PR body.
- [ ] Commit any documentation-only fixture corrections separately.

## Done Gate

This plan is complete only when public Rust API responses no longer expose `TaskGraph` or authority-bearing internals, canonical fixtures parse, typed SSE tests pass, and the existing Rust/Kotlin repository gates remain green.