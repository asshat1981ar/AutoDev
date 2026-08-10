# Agent Runtime

This document describes the **agent runtime** — the component that connects an agent
to tasks, models, tools, and typed actions, driving one step through the lifecycle.

## Lifecycle

```text
CREATED → READY → PLANNING → ACTING → WAITING
  → VERIFYING → COMPLETED

Failure states: FAILED, BLOCKED, CANCELLED
```

## The runtime loop

Per step, the runtime (`crates/forge-core/src/runtime.rs`):

1. **receive task context** — `assign_task(Task)` stores the task and moves to `PLANNING`.
2. **assemble agent context** — `assemble_context()` builds a prompt from the task +
   the agent's role + capabilities.
3. **select model** — `select_model()` prefers the agent's model, falling back to a
   provider model that supports chat.
4. **invoke model** — `invoke_model()` calls the provider-neutral `ModelProvider::chat`.
5. **validate structured output** — `validate_output()` parses the model's JSON into an
   `AgentAction`, rejecting malformed or unknown action/risk values.
6. **produce AgentAction** — the typed action (id, task id, agent id, type, reason, risk,
   capabilities, payload).
7. **submit to policy** — `submit_to_policy()` runs `evaluate_policy` + capability check;
   `RequireApproval` moves the runtime to `BLOCKED`.
8. **consume ExecutionResult** — executes the action via an injected `Executor`, records
   the result as evidence in the `EvidenceStore`, and feeds the result back.
9. **maintain state** — the runtime is a validated state machine; `transition()`
   enforces the allowed lifecycle transitions and `state()` exposes the current state.

## State machine

The runtime maintains a well-formed state machine via `transition()`:

- Allowed progress: `READY → PLANNING → ACTING → WAITING → VERIFYING → COMPLETED`
- Failure: any of `PLANNING/ACTING/WAITING/VERIFYING → FAILED`; `ACTING/WAITING → BLOCKED`;
  most states → `CANCELLED`.
- Recovery: `FAILED/BLOCKED → READY` (and `BLOCKED → ACTING`) for retry.
- Identity transitions are no-ops; invalid transitions are rejected (return `false`).

## Privileged-execution boundary

**Agents never directly access privileged execution.** The model produces only
*intent* (a structured output). That intent is validated into an `AgentAction`, submitted
to policy, and only then executed by the runtime through the injected `Executor`. The
agent/model has no reference to the executor and cannot invoke it. A blocked or denied
action never reaches the executor — proven by the `agents_never_directly_access_privileged_execution`
test, which asserts the executor is invoked zero times when policy blocks.

## Testability

The runtime depends only on the injected `ModelProvider` and `Executor`. Tests use
`MockProvider` (a mocked model) with **deterministic actions** (fixed structured output)
and either a mock executor or a workspace-bound executor. Evidence is recorded with a
verifiable fingerprint, so a produced action can be reconstructed from its evidence.

## Key types

| Type | Purpose |
| --- | --- |
| `AgentRuntimeState` | Lifecycle enum: `Created, Ready, Planning, Acting, Waiting, Verifying, Completed, Failed, Blocked, Cancelled`. |
| `Task` | Task context: id, title, context. |
| `AgentRuntime` | The runtime: profile, state, provider, evidence store, executor, current task, last action/result, selected model. |
| `StructuredOutput` | The validated model output `{action, reason, risk, payload}`. |
| `StepOutcome` | The result of a step: state, action, result, message. |
| `Executor` | `Box<dyn Fn(&AgentAction) -> Result<ExecutionResult, ExecutionError>>` — injectable so orchestration can bind a workspace or use a mock. |
| `RuntimeError` | `NoModel`, `InvalidOutput`, `PolicyDenied`, `NoAction`, `ExecutionFailed`. |

## Testability

The runtime depends only on the injected `ModelProvider` and `Executor`. Tests use
`MockProvider` (no real model) and either a mock executor or a workspace-bound executor
that runs real `read_file`/`git` actions. Evidence is recorded with a verifiable
fingerprint, so a produced action can be reconstructed from its evidence.

## Provider & executor neutrality

- **Model**: orchestration talks to `&dyn ModelProvider`; the runtime never sees
  provider-specific details.
- **Execution**: orchestration injects an `Executor`; the runtime never calls `execute()`
  directly, so it can be tested with a mock executor.

## AgentInstance vs AgentRuntime

- `AgentInstance` (registry, `agent.rs`) — a registered runtime *handle* tracked by the
  `AgentRegistry`.
- `AgentRuntime` (runtime, `runtime.rs`) — the live, lifecycle-driven runtime that
  actually drives a step.

## Tests

Coverage in `crates/forge-core/src/runtime.rs` (unit) and
`crates/forge-core/tests/runtime.rs` (integration):

- lifecycle starts `READY` and `assign_task` → `PLANNING`
- assemble context includes role and task
- selects model from provider
- validates structured output into an action
- rejects malformed output
- `run_step` executes and records evidence (fingerprint verifies)
- high-risk action blocks for approval (no execution, no evidence)
- state transitions are validated (invalid transitions rejected)
- `run_step` honors the full lifecycle (ends `COMPLETED`)
- agents never directly access privileged execution (executor invoked 0 times when blocked)
- full step reads a real file via a workspace-bound executor and records evidence
- registry instance matches runtime role