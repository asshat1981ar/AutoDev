# PRO-66 ForgeCore Control-Plane Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a queued control-plane objective advance through model-produced typed intent, ForgeCore policy/execution, independent verification, durable lifecycle state, and SSE events without creating a second authority path.

**Architecture:** Import the already-green `autodev-server` adapter from PR #8 onto the Working APK integration branch. Add a narrow provider-neutral action-proposal seam in ForgeCore so model output can become an `AgentAction` without executing it. `ObjectiveRunner` then plans queued graphs, proposes actions for ready tasks, and passes those actions into the existing `VerifiedOrchestrator`; ForgeCore remains the only component allowed to authorize and execute effects. Objective snapshots are persisted locally with atomic JSON replacement for v0.

**Tech Stack:** Rust, ForgeCore, Axum 0.7, Tokio 1, serde/serde_json, existing `ModelProvider`/`OllamaProvider`, GitHub Actions.

## Global Constraints

- Android/client input is untrusted intent and cannot mint `AuthorizationGrant`s.
- ForgeCore policy, capability, execution, evidence, and verification remain authoritative.
- `POST /api/v1/objectives` remains `202 Accepted`.
- No client approval mutation endpoint in v0.
- Persistence is local/file-backed for v0; do not add Supabase, Neon, Qdrant, or another remote database.
- State-changing execution for one objective is serialized.
- PR #10 async delegation is not a dependency.
- TDD is mandatory: every production slice begins with a failing focused test.
- GitHub Actions is the authoritative CI system. CircleCI is not configured/exposed in the current runtime and must not be claimed as verification evidence.

---

## File Structure

- Import: `crates/autodev-server/Cargo.toml`
- Import: `crates/autodev-server/src/main.rs`
- Replace/split: `crates/autodev-server/src/lib.rs`
- Create: `crates/autodev-server/src/objective.rs` — public lifecycle/API records.
- Create: `crates/autodev-server/src/store.rs` — atomic local persistence.
- Create: `crates/autodev-server/src/runner.rs` — objective lifecycle coordinator.
- Create: `crates/autodev-server/src/events.rs` — typed SSE events.
- Create: `crates/autodev-server/tests/objective_api.rs`
- Create: `crates/autodev-server/tests/objective_runner.rs`
- Modify: `crates/forge-core/src/runtime.rs` — separate action proposal from execution.
- Modify: `crates/forge-core/src/lib.rs` — export proposal contract.
- Modify: `crates/Cargo.toml` — workspace member.

### Task 1: Import the green PR #8 server baseline

**Interfaces:**
- Consumes: PR #8 files at head `40457ca6cf9761c25b25a5109b8d4f58e73a774a`.
- Produces: buildable `autodev-server` with `/health`, objective create/list, SSE, signed GitHub webhook.

- [ ] **Step 1: copy only the server baseline files from PR #8**

Bring these paths onto the integration branch without importing unrelated branch history:

```text
crates/autodev-server/Cargo.toml
crates/autodev-server/src/lib.rs
crates/autodev-server/src/main.rs
crates/Cargo.toml  (add autodev-server workspace member only)
```

Do not copy PR #8 Docker/CI changes in this task; packaging is handled after the API contract is stable.

- [ ] **Step 2: run focused baseline verification**

```bash
cd crates
cargo test -p autodev-server
cargo clippy -p autodev-server --all-targets -- -D warnings
```

Expected: PASS, matching the previously green PR #8 behavior.

- [ ] **Step 3: commit**

```bash
git add crates/Cargo.toml crates/autodev-server
git commit -m "feat(server): import verified control-plane baseline"
```

### Task 2: Add a non-executing typed action proposal seam

**Files:**
- Modify: `crates/forge-core/src/runtime.rs`
- Modify: `crates/forge-core/src/lib.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActionProposal {
    pub action: AgentAction,
    pub decision: PolicyDecision,
    pub model: String,
}

pub fn propose_action(
    agent_id: &str,
    profile: &AgentProfile,
    provider: &dyn ModelProvider,
    task: &Task,
) -> Result<ActionProposal, RuntimeError>;
```

`propose_action` may call the model and policy evaluator but MUST NOT call an executor or create trusted approval.

- [ ] **Step 1: write RED tests in `runtime.rs`**

```rust
#[test]
fn proposal_generates_typed_action_without_execution() {
    let provider = MockProvider::new(serde_json::json!({
        "action": "read_file",
        "reason": "inspect repository",
        "risk": "low",
        "payload": {"path": "README.md"}
    }).to_string());
    let profile = default_profiles().into_iter()
        .find(|p| p.role == AgentRole::Developer).unwrap();
    let proposal = propose_action("dev-1", &profile, &provider, &task()).unwrap();
    assert_eq!(proposal.action.action_type, ActionType::ReadFile);
    assert_eq!(proposal.decision, PolicyDecision::Allow);
    assert_eq!(proposal.model, "mock-model");
}

#[test]
fn proposal_preserves_approval_requirement_without_approving() {
    let provider = MockProvider::new(serde_json::json!({
        "action": "write_file",
        "reason": "change code",
        "risk": "high",
        "payload": {"path": "src/lib.rs", "content": "x"}
    }).to_string());
    let profile = default_profiles().into_iter()
        .find(|p| p.role == AgentRole::Developer).unwrap();
    let proposal = propose_action("dev-1", &profile, &provider, &task()).unwrap();
    assert_eq!(proposal.decision, PolicyDecision::RequireApproval);
}
```

- [ ] **Step 2: verify RED**

```bash
cd crates
cargo test -p forge-core proposal_generates_typed_action_without_execution -- --exact
```

Expected: compile/test FAIL because `ActionProposal`/`propose_action` do not exist.

- [ ] **Step 3: implement the minimal seam**

Extract the model selection, invocation, structured-output validation, and policy decision currently embedded in `AgentRuntime::run_step()` into `propose_action`. Refactor `run_step()` to call the new helper and preserve all existing lifecycle semantics.

The helper must reject malformed output and missing capability exactly as existing runtime code does. It must not accept any approval reference parameter.

- [ ] **Step 4: verify GREEN and regressions**

```bash
cargo test -p forge-core runtime
cargo clippy -p forge-core --all-targets --all-features -- -D warnings
```

Expected: PASS.

- [ ] **Step 5: commit**

```bash
git add forge-core/src/runtime.rs forge-core/src/lib.rs
git commit -m "refactor(forge-core): separate typed action proposal from execution"
```

### Task 3: Replace free-form objective status with a closed API contract

**Files:**
- Create: `crates/autodev-server/src/objective.rs`
- Modify: `crates/autodev-server/src/lib.rs`
- Create: `crates/autodev-server/tests/objective_api.rs`

**Interfaces:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveStatus {
    Queued, Planning, Running, Blocked, Verifying, Replanned, Completed, Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveView {
    pub id: String,
    pub repository: String,
    pub description: String,
    pub branch: String,
    pub status: ObjectiveStatus,
    pub current_task_id: Option<String>,
    pub current_phase: Option<String>,
    pub latest_evidence_ref: Option<String>,
    pub blocked_reason: Option<String>,
}
```

Internal graph/envelope state must not be writable API fields.

- [ ] **Step 1: RED tests**

Add tests that assert snake-case serialization, `GET /api/v1/objectives/{id}` returns a created objective, and an unknown ID returns `404`.

- [ ] **Step 2: verify RED**

```bash
cargo test -p autodev-server --test objective_api
```

Expected: FAIL because the new route/types do not exist.

- [ ] **Step 3: implement types and route**

Expose only `ObjectiveView` from list/get/create handlers. Keep `TaskGraph` internal.

- [ ] **Step 4: verify GREEN**

```bash
cargo test -p autodev-server --test objective_api
```

- [ ] **Step 5: commit**

```bash
git add autodev-server/src autodev-server/tests/objective_api.rs
git commit -m "feat(server): add typed objective lifecycle API"
```

### Task 4: Add atomic local objective persistence

**Files:**
- Create: `crates/autodev-server/src/store.rs`
- Modify: `crates/autodev-server/src/objective.rs`
- Test: `crates/autodev-server/tests/objective_runner.rs`

**Interfaces:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveSnapshot {
    pub view: ObjectiveView,
    pub graph: TaskGraph,
    pub orchestrator: VerifiedOrchestratorState,
}

pub trait ObjectiveStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<ObjectiveSnapshot>, StoreError>;
    fn get(&self, id: &str) -> Result<Option<ObjectiveSnapshot>, StoreError>;
    fn put(&self, snapshot: &ObjectiveSnapshot) -> Result<(), StoreError>;
}

pub struct FileObjectiveStore { root: PathBuf }
```

Each objective is stored as `<root>/<objective-id>.json`. `put()` writes `<id>.json.tmp`, `sync_all()`s the file, then renames to `<id>.json`; no partial JSON file may be treated as valid state.

- [ ] **Step 1: RED tests**

Create a temp directory, `put()` a snapshot, create a fresh store instance, and verify the snapshot round-trips. Add a test proving a stray `.tmp` file is ignored.

- [ ] **Step 2: verify RED**

```bash
cargo test -p autodev-server store_
```

- [ ] **Step 3: implement atomic file store**

Use `std::fs::{create_dir_all, read_dir, rename, File}` and `serde_json::{from_reader,to_writer_pretty}`. Do not introduce a database dependency.

- [ ] **Step 4: verify GREEN**

```bash
cargo test -p autodev-server store_
```

- [ ] **Step 5: commit**

```bash
git add autodev-server/src/store.rs autodev-server/src/objective.rs autodev-server/tests/objective_runner.rs
git commit -m "feat(server): persist objective execution snapshots locally"
```

### Task 5: Add typed lifecycle events

**Files:**
- Create: `crates/autodev-server/src/events.rs`
- Modify: `crates/autodev-server/src/lib.rs`

**Interfaces:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub objective_id: String,
    pub task_id: Option<String>,
    pub phase: Option<String>,
    pub status: ObjectiveStatus,
    pub evidence_ref: Option<String>,
    pub message: String,
}
```

Allowed `event_type` values are exactly `objective_queued`, `objective_planning`, `objective_running`, `objective_blocked`, `objective_verifying`, `objective_replanned`, `objective_completed`, `objective_failed`.

- [ ] **Step 1: RED test** proving a queued objective emits the new flat event shape and contains no `approval_ref` field.
- [ ] **Step 2: verify RED** with `cargo test -p autodev-server objective_event`.
- [ ] **Step 3: replace ad-hoc `json!` SSE payloads with `ObjectiveEvent`.**
- [ ] **Step 4: verify GREEN.**
- [ ] **Step 5: commit** with `feat(server): emit typed objective lifecycle events`.

### Task 6: Implement `ObjectiveRunner` over `VerifiedOrchestrator`

**Files:**
- Create: `crates/autodev-server/src/runner.rs`
- Modify: `crates/autodev-server/src/lib.rs`
- Test: `crates/autodev-server/tests/objective_runner.rs`

**Interfaces:**

```rust
pub trait ActionProposer: Send + Sync {
    fn propose(&self, task: &TaskNode) -> Result<ActionProposal, RunnerError>;
}

pub struct ObjectiveRunner<S: ObjectiveStore, P: ActionProposer> {
    store: Arc<S>,
    proposer: Arc<P>,
    events: broadcast::Sender<ObjectiveEvent>,
}

impl<S: ObjectiveStore, P: ActionProposer> ObjectiveRunner<S, P> {
    pub fn advance_once(&self, objective_id: &str) -> Result<ObjectiveView, RunnerError>;
}
```

`advance_once` algorithm:

```text
load snapshot
if root == queued:
  Planner::default().plan(graph)
  persist + emit planning
else if a ready task lacks planned_action:
  proposer.propose(task)
  store serialized AgentAction in task.planned_action
  persist
construct/restore VerifiedOrchestrator
advance exactly one trusted orchestration attempt
map task/envelope result -> ObjectiveStatus
persist graph + VerifiedOrchestratorState
emit exactly one resulting lifecycle event
return ObjectiveView
```

The envelope factory MUST deserialize the previously proposed `AgentAction`; it must never build an action from Android JSON.

- [ ] **Step 1: RED tests**

Use a fake `ActionProposer` returning a low-risk `ReadFile` action and a deterministic passing verification fabric. Assert queued -> planning -> running/verifying -> completed over repeated `advance_once()` calls.

Add separate tests for:
- high-risk proposal -> `blocked` and zero trusted execution before approval;
- verifier rejection -> `replanned`;
- attempt exhaustion -> `failed`;
- restored store state continues from the persisted graph/envelope rather than resetting.

- [ ] **Step 2: verify RED**

```bash
cargo test -p autodev-server --test objective_runner
```

- [ ] **Step 3: implement the minimum runner**

Reuse `Planner`, `Decomposer`, `Assigner`, `DevelopmentLoop`, `VerifiedOrchestrator`, `VerificationFabric`, and `ExecutionEnvelope`. Do not duplicate their policy or transition logic in the server.

- [ ] **Step 4: verify GREEN**

```bash
cargo test -p autodev-server --test objective_runner
cargo clippy -p autodev-server --all-targets -- -D warnings
```

- [ ] **Step 5: commit**

```bash
git add autodev-server/src/runner.rs autodev-server/src/lib.rs autodev-server/tests/objective_runner.rs
git commit -m "feat(server): drive objectives through verified ForgeCore execution"
```

### Task 7: Add the production model proposer and serialized background loop

**Files:**
- Modify: `crates/autodev-server/src/runner.rs`
- Modify: `crates/autodev-server/src/main.rs`
- Modify: `crates/autodev-server/Cargo.toml`

**Interfaces:**

```rust
pub struct ModelActionProposer {
    agent_id: String,
    profile: AgentProfile,
    provider: Arc<dyn ModelProvider + Send + Sync>,
}
```

To support this safely, strengthen `ModelProvider` to `Send + Sync` only if the existing `OllamaProvider` and `MockProvider` satisfy those bounds under `cargo test --workspace`. If that change causes a broad unrelated break, stop and rescope rather than bypassing the bound.

Startup configuration:

```text
AUTODEV_STATE_DIR=.autodev/state
AUTODEV_MODEL_BASE_URL=http://localhost:11434
AUTODEV_PORT=8080
```

Use the existing Developer profile and `OllamaProvider::new(AUTODEV_MODEL_BASE_URL)` as the default concrete provider. The runner API remains provider-neutral.

- [ ] **Step 1: RED integration test** using `MockProvider` that proves the background loop advances a newly queued objective without a second HTTP request.
- [ ] **Step 2: verify RED.**
- [ ] **Step 3: implement one serialized runner worker** that polls ready objective IDs and calls `advance_once`; no concurrent repository mutation.
- [ ] **Step 4: verify GREEN** with focused and workspace Rust tests.
- [ ] **Step 5: commit** with `feat(server): run queued objectives with provider-neutral model proposals`.

### Task 8: Full PRO-66 verification and documentation

**Files:**
- Modify: `README.md`
- Modify: `crates/autodev-server/README.md` if the crate gains one; otherwise document server config in root README.

- [ ] **Step 1: document server start, state directory, model URL, objective API, lifecycle states, and the no-client-approval rule.**
- [ ] **Step 2: run all Rust gates**

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

- [ ] **Step 3: push and require GitHub Actions Rust + existing Kotlin/Python jobs to pass.**
- [ ] **Step 4: attach CI run evidence to Linear PRO-66 and move it to In Review.**
- [ ] **Step 5: commit docs** with `docs(server): document verified objective lifecycle`.

## Self-Review

- Spec coverage: model proposal, typed lifecycle, local recovery, trusted execution, block/replan/failure semantics, SSE, and API get/list/create are covered.
- Authority check: no Android/client field can supply an approval grant or direct `AgentAction`.
- Scope check: remote DB, iOS, PR #10, automatic merge, and approval mutation UI remain excluded.
- Product-risk correction: this plan explicitly connects model-produced typed actions to `VerifiedOrchestrator`; it does not rely on synthetic test-only envelope factories for production.
