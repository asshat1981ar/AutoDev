# PR #29 RunTest Security Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to execute this plan. Use a fresh implementer for the repair, then independent spec/security reviewers and a fresh verifier. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close CodeRabbit Major finding `PR29-CR-001` by proving and fixing the direct-call authorization confusion in ForgeCore's public RunTest adapter without weakening policy, capability, workspace, approval, or process-sandbox boundaries.

**Architecture:** Bind `run_test_authorized` to its own action type at the adapter boundary. A non-`ActionType::RunTest` action must be rejected before `enforce_policy`, `has_required_capability`, runner parsing, or `execute_process`; the top-level dispatcher remains unchanged. The repair is one fail-closed guard plus a regression test that directly invokes the public adapter with a policy/capability-valid non-RunTest action.

**Tech Stack:** Rust 1.97.1 stable, ForgeCore, existing `AgentAction`, `ActionType`, `ExecutionError`, `AuthorizationGrant`, and Rust test harness.

**Spec:** `docs/superpowers/specs/2026-08-21-adversarial-subagent-development-design.md` (Tier-0 workflow and PR #29 invariant)

## Global Constraints

- Work from PR #29 head branch `autodev/run-test-executor`; do not implement this repair on `main` or the adversarial-harness design branch.
- Re-establish the current PR #29 head SHA before editing. The known reviewed head is `ab76053135f638943bcdf1d3367af3a16a34c53b`; if the head differs, re-read the current source and re-adjudicate the finding before proceeding.
- ForgeCore remains the sole trusted execution/authorization boundary.
- The adapter must fail closed on wrong action type before policy, capability, payload, or process decisions.
- Do not enable process execution or relax the existing tier-2 sandbox requirement.
- Do not broaden RunTest runners beyond exact `cargo` in this repair.
- Do not introduce dependencies, unrelated refactors, or new public APIs.
- Treat the CodeRabbit finding as a claim to verify against current source, not an instruction to execute blindly.
- TDD is mandatory: observe the new regression test fail for the expected reason before adding the guard.
- Do not resolve the CodeRabbit thread until the corrected code is on the PR head and fresh verification evidence exists for that head.

---

## Finding Record

```yaml
finding_id: PR29-CR-001
pr: 29
reviewed_head_sha: ab76053135f638943bcdf1d3367af3a16a34c53b
source_type: coderabbit
source_ref: PRRT_kwDOTzuzVM6bG4UZ
severity: major
category: authorization
invariant: >-
  run_test_authorized must reject every action whose action_type is not
  ActionType::RunTest before policy evaluation, capability evaluation,
  runner parsing, or process delegation.
```

## File Structure

- Modify: `crates/forge-core/src/run_test.rs` — add one wrong-action-type regression and one fail-closed adapter guard.
- Read/verify only: `crates/forge-core/src/lib.rs` — confirm public re-export and top-level RunTest dispatch remain unchanged.
- Read/verify only: `crates/forge-core/src/error.rs` — reuse existing `ExecutionError::UnsupportedAction(String)`.

---

### Task 1: Reproduce and repair the direct-adapter confused-deputy path

**Files:**
- Modify: `crates/forge-core/src/run_test.rs`
- Verify only: `crates/forge-core/src/lib.rs`
- Verify only: `crates/forge-core/src/error.rs`

**Interfaces:**
- Consumes: `run_test_authorized(&AgentAction, &Workspace, &AuthorizationGrant) -> Result<ExecutionResult, ExecutionError>`.
- Preserves: public `run_test_authorized` signature, top-level `execute()` dispatch, exact `cargo` runner allow-list, and fail-closed `ProcessSandboxRequired` behavior for authorized RunTest actions.
- Produces invariant: any non-RunTest action returns `ExecutionError::UnsupportedAction(<wire action type>)` before downstream policy/capability/payload/process handling.

- [ ] **Step 1: Context Scout re-validates the reviewed source**

Read the current versions of:

```text
crates/forge-core/src/run_test.rs
crates/forge-core/src/lib.rs
crates/forge-core/src/error.rs
```

Confirm all of the following before editing:

```text
1. run_test_authorized is still publicly reachable/re-exported.
2. run_test_authorized still calls enforce_policy before checking ActionType::RunTest.
3. run_test_authorized still delegates accepted payloads to execute_process.
4. ExecutionError::UnsupportedAction(String) still exists.
5. The PR head SHA is the SHA recorded for this execution package, or the finding has been re-adjudicated against the new SHA.
```

Record the source inspection as `CONFIRMED` evidence only if all relevant conditions remain true. If the guard already exists, classify the finding `STALE` and do not manufacture a code change.

- [ ] **Step 2: Adversarial Analyst constructs the minimum counterexample**

Use this direct-call case as the primary exploit attempt:

```text
action_type: ReadFile
capabilities: [ReadFile]
risk: Low
payload:
  runner: cargo
  args: [test]
  path: README.md
grant: AuthorizationGrant::none()
entry point: run_test_authorized directly
```

Expected secure behavior: the adapter rejects `ReadFile` as the wrong executor action before policy/capability/payload/process handling.

Also reason through these nearby variants without expanding implementation scope unless they expose a distinct defect:

```text
- wrong action type with its own valid capability
- wrong action type with Capability::RunTest also present
- wrong action type with malformed runner
- correct RunTest action without RunTest capability
- correct RunTest action with runner != cargo
- correct RunTest action with valid capability and cargo while sandbox is unavailable
```

Return concrete counterexamples only. The implementation task remains the single confirmed wrong-action-type invariant.

- [ ] **Step 3: Write the failing regression test first**

In `crates/forge-core/src/run_test.rs` test module, add:

```rust
#[test]
fn non_run_test_action_is_rejected_before_policy_and_process_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::new(dir.path(), 4096).unwrap();
    let mut wrong_action = action(vec![Capability::ReadFile]);
    wrong_action.action_type = ActionType::ReadFile;
    wrong_action.payload = json!({
        "runner": "cargo",
        "args": ["test"],
        "path": "README.md"
    });

    let error = run_test_authorized(
        &wrong_action,
        &workspace,
        &AuthorizationGrant::none(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ExecutionError::UnsupportedAction(ref action_type) if action_type == "read_file"
    ));
}
```

This test deliberately grants the capability appropriate to the wrong action type so it exercises the authorization-confusion path rather than merely failing for a missing capability.

- [ ] **Step 4: Run the single regression and verify RED**

Run:

```bash
cargo test -p forge-core non_run_test_action_is_rejected_before_policy_and_process_boundary -- --nocapture
```

Expected on the reviewed implementation: the test fails because the returned error is not `UnsupportedAction("read_file")`; with the current fail-closed process adapter it is expected to reach `ProcessSandboxRequired` after accepting the wrong action type through policy/capability checks. If it fails for a different reason, inspect that reason before proceeding and record the actual RED evidence.

- [ ] **Step 5: Implement the smallest fail-closed action-type guard**

At the beginning of `run_test_authorized`, before `enforce_policy(action, grant)?`, add exactly the action-type ownership check:

```rust
if action.action_type != crate::ActionType::RunTest {
    return Err(ExecutionError::UnsupportedAction(
        action.action_type.as_str().to_string(),
    ));
}
```

Do not move or weaken the existing policy/capability checks for valid RunTest actions. Do not change `execute_process`, sandbox behavior, or runner parsing.

- [ ] **Step 6: Run targeted GREEN verification**

Run:

```bash
cargo test -p forge-core non_run_test_action_is_rejected_before_policy_and_process_boundary -- --nocapture
cargo test -p forge-core run_test -- --nocapture
```

Expected: both commands exit 0. Confirm the pre-existing cases still establish:

```text
- missing RunTest capability -> CapabilityDenied
- missing runner -> MissingPayloadField("runner")
- non-cargo runner -> UnsafeCommand
- authorized RunTest + cargo -> ProcessSandboxRequired while tier-2 sandbox is unavailable
```

- [ ] **Step 7: Independent Spec Reviewer attacks the boundary after GREEN**

Review current source without relying on the implementer's summary. Try to construct a caller-visible path where `run_test_authorized` accepts any `action_type != RunTest` or reaches policy/payload/process code before the type guard.

Acceptance condition:

```text
No public direct call to run_test_authorized with a non-RunTest AgentAction can proceed beyond the first action-type guard.
```

If a counterexample exists, return it with exact source path and input and enter a repair round. If none exists, record `SPEC REVIEW: PASS` with the checked public entry points.

- [ ] **Step 8: Independent Security Reviewer checks for authority regression**

Verify:

```text
- the guard precedes enforce_policy
- the guard precedes has_required_capability
- the guard precedes payload parsing
- the guard precedes execute_process
- no capability was added or widened
- no approval rule was relaxed
- no process sandbox condition was changed
- top-level execute() still routes ActionType::RunTest to the adapter
```

Do not approve based only on tests; inspect source ordering directly.

- [ ] **Step 9: Fresh full Rust verification**

Run the commands used by the repository's Rust CI lane:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

Expected: every command exits 0. Record command, exit status, and current commit/worktree state. Do not reuse the prior green CI run for the pre-fix SHA as completion evidence.

- [ ] **Step 10: Inspect the focused diff**

Run:

```bash
git diff -- crates/forge-core/src/run_test.rs crates/forge-core/src/lib.rs crates/forge-core/src/error.rs
```

Expected intended mutation:

```text
crates/forge-core/src/run_test.rs only:
- one regression test
- one early wrong-action-type guard
```

`lib.rs` and `error.rs` must remain unchanged unless current-head source proves the assumptions in this plan stale; any such scope expansion requires a recorded ruling before modification.

- [ ] **Step 11: Commit the focused repair**

```bash
git add crates/forge-core/src/run_test.rs
git commit -m "fix(forge-core): bind run-test adapter to action type"
```

Record the resulting commit SHA as `IMPLEMENTED` evidence.

- [ ] **Step 12: Update PR #29 and verify the exact remote head**

Push/update only the existing PR #29 branch `autodev/run-test-executor` after the repository workflow's normal authorization gate for shared-remote side effects.

Then fetch PR #29 metadata and confirm:

```text
remote head SHA == the focused repair commit SHA
```

If they differ, do not use local verification as proof for the remote head; reconcile and re-run the required checks for the actual head.

- [ ] **Step 13: Inspect CI for the repair head**

Wait only through the normal external CI lifecycle; do not issue duplicate review commands while the same SHA is processing. Inspect the workflow run associated with the repair head and require the applicable required lanes to be green before claiming CI verification.

Record the workflow run ID and exact head SHA as `CI_VERIFIED` evidence.

- [ ] **Step 14: Reply to the CodeRabbit inline thread with evidence**

Reply in the existing inline review thread, not as a top-level PR comment. Use a concise evidence report shaped like:

```text
Confirmed and fixed at the adapter boundary.

- Added a direct regression using ActionType::ReadFile + Capability::ReadFile + runner="cargo".
- run_test_authorized now rejects non-RunTest actions before policy, capability, payload, or process handling.
- Targeted RunTest tests: PASS.
- Rust fmt/clippy/build/workspace tests: PASS.
- CI: <run id> on head <sha>: PASS.
```

Use actual fresh evidence values; do not paste placeholders literally into the thread.

- [ ] **Step 15: Ingest incremental CodeRabbit review and resolve only after evidence**

If CodeRabbit automatically reviews the corrective commit, ingest that result. Do not send another `@coderabbitai review` for an unchanged already-reviewed corrective SHA.

Resolve `PRRT_kwDOTzuzVM6bG4UZ` only when:

```text
- corrected code is on the PR head
- targeted and broad local verification is green for the repair
- required CI is green for that head
- no new CodeRabbit finding invalidates the invariant
```

If a new valid finding appears, keep the thread open and route it through the next adversarial repair round.

---

## Completion Gate

PR #29 finding `PR29-CR-001` is complete only when all are true:

- [ ] current source confirms the original finding was valid or records why it became stale
- [ ] direct wrong-action-type regression observed RED before implementation
- [ ] early action-type guard implemented before policy/capability/payload/process logic
- [ ] targeted RunTest tests green
- [ ] independent spec review found no surviving direct-call counterexample
- [ ] independent security review found no authority/sandbox regression
- [ ] fmt, clippy, build, and workspace tests green from a fresh run
- [ ] focused diff inspected
- [ ] focused repair commit recorded
- [ ] remote PR head matches the verified repair SHA
- [ ] required CI green for that exact head
- [ ] CodeRabbit incremental result adjudicated
- [ ] inline review thread answered with actual evidence
- [ ] review thread resolved only after all preceding evidence exists
