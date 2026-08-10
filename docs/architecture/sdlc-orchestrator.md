# SDLC Orchestrator

This document describes the first autonomous development loop in ForgeCore, built
around **durable tasks** rather than conversational messages.

## The loop

```text
PLAN → DECOMPOSE → ASSIGN → ACT → VERIFY → REPAIR → CHECKPOINT → REPLAN
```

Each phase is a **pure, testable transition** on a [`TaskGraph`]. The orchestrator
(`crates/forge-core/src/orchestrator.rs`) advances the loop one deterministic step at a
time via `advance()`.

## Design goals

The orchestrator is deliberately **not** optimized for maximum autonomy. It optimizes
for:

- **Observability** — every transition is recorded in a `Transition` log.
- **Recoverability** — the whole graph is serializable and a `CHECKPOINT` snapshots it
  for restoration.
- **Deterministic state** — each phase is a pure function on the graph.
- **Clear task transitions** — tasks move through explicit `TaskStatus`es.
- **Human intervention** — a task can be blocked awaiting approval and resumed; retries
  are capped.

## Components

| Component | Phase | Responsibility |
| --- | --- | --- |
| `Planner` | PLAN | Produce a plan for the root task; mark it `PLANNING`. |
| `Decomposer` | DECOMPOSE | Break a task into sub-tasks (created `READY`). |
| `Assigner` | ASSIGN | Assign a ready task to an agent role; mark `RUNNING`. |
| `TaskExecutor` | ACT | Run a task's action; move to `VERIFYING`. |
| `Verifier` | VERIFY | Check the result → `COMPLETED` or `FAILED`. |
| `Repairer` | REPAIR | Retry a failed task (capped) → back to `READY`. |
| `Checkpointer` | CHECKPOINT | Snapshot the graph state as JSON. |
| `Orchestrator` | loop | Wire the phases; `advance()` runs one step. |

## Durable tasks

A [`TaskGraph`] is a map of `TaskNode`s (id, title, description, status, priority,
dependencies, acceptance criteria, agent, planned action, retries, timestamps) plus an
auditable `Transition` log and a checkpoint snapshot. This is the durable unit of
orchestration — planning/execution state is not threaded as conversational messages.

`TaskStatus` mirrors `task.schema.json` and adds `repairing`:
`queued, planning, ready, running, verifying, repairing, blocked, completed, failed, cancelled`.

## Deterministic scheduling

`TaskGraph::next_ready()` returns the highest-priority `READY` task whose dependencies
are done. `Orchestrator::advance()` performs exactly one phase transition, so the loop
is observable and testable; `Orchestrator::is_done()` reports terminal state.

## Testability

Every phase is injectable (plan/decompose/assign/run/check/retry are closures), so the
loop is tested with deterministic components and no real model. The integration test
drives a loop with a real workspace-bound `write_file` executor and asserts the graph
reaches a terminal state with a populated transition log and a recovery checkpoint.

## Tests

Coverage in `crates/forge-core/src/orchestrator.rs` (unit) and
`crates/forge-core/tests/orchestrator.rs` (integration):

- PLAN marks the root planning
- DECOMPOSE adds ready sub-tasks
- `advance` drives a full loop to completion (deterministic scheduling)
- a failed task is repaired and retried
- CHECKPOINT snapshots state
- `next_ready` respects dependencies
- integration: loop runs to terminal with durable tasks + checkpoint + log
- integration: graph is recoverable from its snapshot