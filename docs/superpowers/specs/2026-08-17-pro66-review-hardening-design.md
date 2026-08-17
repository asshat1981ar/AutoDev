# PRO-66 Review Hardening Design

## Goal

Close the authority and durability defects found in the independent review of PR #17 without introducing a second orchestrator, database, or new runtime.

## Invariants

1. Model/client data is intent, never authority.
2. Trusted orchestration state and evidence must not be writable through the execution workspace.
3. Risk used for authorization must have a trusted kernel-derived floor; model risk may only increase that value.
4. Approval must be represented as a scoped trusted grant, not an unbound string.
5. Every published evidence reference must resolve to evidence that survives runner reconstruction and process restart.
6. Existing `VerifiedOrchestrator`, `DevelopmentLoop`, workspace confinement, retry lifecycle, and file-backed objective persistence remain the single execution path.

## Design

### Trusted storage separation

Resolve the execution workspace and state directory at startup. The default state directory is a sibling of the workspace named `.autodev-state-<workspace-name>`, never a descendant. `validate_control_plane_paths(workspace, state_dir)` creates/canonicalizes the state directory and rejects equal/descendant paths. Explicit `AUTODEV_STATE_DIR` values are subject to the same validation.

Evidence is persisted with each `ObjectiveSnapshot`, so it inherits the same protected storage boundary. No database is added.

### Trusted risk floor

Add a ForgeCore helper `minimum_risk_for_action(&AgentAction) -> RiskLevel` and `effective_risk_for_action(&AgentAction) -> RiskLevel`. The effective risk is `max(model_risk, trusted_minimum)`.

Minimums for currently executable operations:
- `read_file`: low
- `write_file`: medium
- `patch_file`: medium
- `git` read operations: low
- `git` checkpoint / prepare_commit: medium
- `git` rollback: high
- `execute`: high
- `run_test`: medium
- `mcp`: high
- `request_approval`: low

`propose_action` rewrites `action.risk` to the effective value before deciding allow/approval/deny. The server recomputes the effective risk again while binding trusted capabilities so custom proposers cannot bypass the rule.

### Durable evidence

Make `EvidenceStore` serializable/deserializable and store it in `ObjectiveSnapshot` with `#[serde(default)]` for backward compatibility. Add `DevelopmentLoop::with_evidence(verification, evidence)` so runner reconstruction restores prior records. After each verified-orchestrator advance, persist the updated evidence store back into the snapshot before emitting a public reference.

### Scoped approval grant

Introduce `ObjectiveApprovalGrant` in the control-plane layer with objective id, task id, and non-empty approval reference. `ObjectiveRunner::resume_approved` accepts this typed grant and verifies that it matches the loaded objective and current blocked task before delegating the reference to the existing ForgeCore resume path. Raw strings are no longer accepted by the runner API.

This slice does not add an HTTP approval endpoint. A future authenticated approval authority will be responsible for constructing these grants.

## Error handling

- State/workspace overlap is a startup error.
- Invalid/mismatched approval grants fail closed without mutating persisted state.
- Evidence deserialization remains fail-closed through the existing snapshot corruption behavior.
- Existing worker fairness remains unchanged.

## Tests

Adversarial regression tests must prove:
1. state directory inside/equal to workspace is rejected and sibling state is accepted;
2. a model/custom proposer cannot label write, patch, mutating Git, or rollback below the trusted floor;
3. evidence records remain resolvable after repeated runner reconstruction and file-store restart;
4. an approval grant scoped to another objective/task cannot resume a blocked action;
5. full Rust, Kotlin/Android, Python, formatting, clippy, build, and container gates remain green.
