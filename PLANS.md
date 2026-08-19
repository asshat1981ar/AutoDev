# AutoDev ExecPlans

## Purpose

ExecPlans are the durable coordination format for architectural, multi-hour, or interruption-prone development work in AutoDev. They make the objective, milestones, evidence, decisions, discoveries, and recovery state understandable to a fresh worker after context loss.

An ExecPlan is durable coordination state, not execution authority.

Human-readable plan prose complements typed runtime state. Prose explains intent and progress; typed ForgeCore state is canonical for lifecycle correctness, bounded budgets, checkpoint identity, and references to tasks, runs, envelopes, and evidence.

## Non-negotiable authority boundary

An ExecPlan may reference work that requires trusted authority, but it cannot mint an `AuthorizationGrant`, widen capabilities, bypass `Workspace` confinement, execute effects directly, or mark its own work verified. Models, agents, skills, plugins, MCP servers, plan files, and evaluation results remain untrusted inputs to policy.

Effectful work still flows through existing ForgeCore policy and execution boundaries. Completion still requires independent verification evidence satisfying the declared contract.

## How to author an ExecPlan

Use an ExecPlan when work is architectural, spans multiple independently useful milestones, is expected to run for hours, may survive interruption, or needs durable decision/evidence history. Keep it self-contained enough that a worker can resume from the repository without relying on chat history.

State the goal, architectural constraints, milestone order, interfaces, acceptance evidence, retry/replan budgets, recovery rules, and the concrete commands or observations proving completion. Prefer small vertical milestones whose results can be independently reviewed.

## Required living sections

Every active ExecPlan must maintain the following sections while work proceeds.

### Progress

Record completed and current milestones, including dates or commit references when useful. Update this section after every milestone and whenever work becomes blocked, interrupted, cancelled, or resumed.

### Surprises & Discoveries

Record unexpected repository behavior, failed assumptions, environmental constraints, performance/security findings, and reusable observations. Do not hide discoveries merely because they do not change the immediate implementation.

### Decision Log

Record consequential design and execution decisions with the reason, alternatives considered when material, and what it costs if the decision is wrong. Decisions that alter a trusted boundary or structural architecture also require the repository's ADR process.

### Outcomes & Retrospective

Record what actually shipped, what evidence proved it, what remains incomplete, regressions or limitations discovered, and what should change in the harness or future plans.

## Milestones and observable proof

Each milestone must produce observable behavior or durable evidence. A source file existing is not sufficient proof. Prefer independently reproducible tests, builds, static checks, simulations, rendered behavior, API responses, or evidence records appropriate to the change.

Milestones should be small enough to review independently and ordered so later work consumes explicit interfaces produced by earlier milestones.

## Checkpoints, interruption, and resume

Persist enough state to resume without replaying already-completed work. Verified work is not re-executed. Approval-blocked work stays blocked until trusted approval arrives. Cancellation is explicit and durable.

If an effectful operation was interrupted and its outcome is uncertain, reconciliation with the external/repository state is required before retry. Resume must not optimistically repeat an operation whose effects may already have happened.

## Bounded replanning

Retries and replans must have explicit finite budgets. Replanning preserves prior decisions, discoveries, checkpoints, and verification evidence rather than erasing history. Budget exhaustion produces an explicit failed or blocked outcome instead of unbounded autonomous churn.

## Evidence and completion

An agent's report that work succeeded is not completion evidence. Required verification must actually run and pass. Missing required evidence fails closed, consistent with `ExecutionEnvelope` and `VerificationFabric` semantics.

An ExecPlan can coordinate evidence references but cannot produce trusted verification merely by changing plan state.

## Plan maintenance rules

- Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective current as work proceeds.
- Update the plan when milestone scope, interfaces, recovery rules, or acceptance evidence changes.
- Preserve historical decisions and discoveries when replanning.
- Keep machine-critical lifecycle data in typed state; do not parse prose to decide authorization or execution lifecycle.
- Prefer deterministic references to task, run, envelope, checkpoint, commit, and evidence identities.
- Never embed secrets or trusted approval material in plan prose.
