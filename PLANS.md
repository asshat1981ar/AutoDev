# AutoDev ExecPlans

## Purpose

ExecPlans are the repository's durable coordination contract for architectural or multi-hour development work. They keep objectives, milestone proof, decisions, discoveries, interruption state, and outcomes recoverable across agent/session boundaries.

**An ExecPlan is durable coordination state, not execution authority.** The human-readable plan explains intent and progress; typed `forge-core::ExecPlan` state is authoritative for lifecycle and budget invariants.

## Non-negotiable authority boundary

ForgeCore remains the sole trusted execution authority. An ExecPlan may reference tasks, runs, and execution envelopes, but it cannot mint an `AuthorizationGrant`, approve an operation, widen a capability set, execute an effect, or mark its own work verified. Human-readable plan prose is never parsed as trusted authorization state.

Execution still flows through the existing typed action, policy, workspace, approval, evidence, and verification boundaries. A plan records what should happen and what was observed; it does not confer permission for that work to happen.

## How to author an ExecPlan

Use an ExecPlan for work expected to span multiple milestones, sessions, architectural boundaries, or substantial verification cycles. State a concrete goal, decompose it into independently observable milestones, give each milestone bounded attempts, and set a bounded replan budget before execution begins.

Keep the plan current while work proceeds. Do not defer updates until completion: record milestone progress, unexpected discoveries, and decisions when they occur so a later worker can resume from repository evidence rather than conversational memory.

## Required living sections

Every living ExecPlan must contain the following sections.

### Progress

Record each milestone's current state, attempt count, completed proof, and next action. Update this section after every milestone transition. Progress claims must name observable repository or verification evidence rather than relying on an agent statement.

### Surprises & Discoveries

Record material facts that differ from the original plan: repository truth contradicting chat history, hidden dependencies, stale branches, failing gates, security boundaries, performance findings, or environmental constraints. Update this section when the discovery is made.

### Decision Log

Record decisions that change implementation direction, including the evidence, alternatives considered, affected areas, risk, and rollback path where relevant. Update this section when the decision is made.

### Outcomes & Retrospective

Record what actually landed, which acceptance criteria were proven, what remained unfinished, and what should change in the next plan. Populate this section from verification evidence, not from intent.

## Milestones and observable proof

A milestone is complete only when its acceptance criteria have observable proof. Prefer tests, build output, deterministic command results, repository diffs, persisted evidence, or another independent verifier. A model or worker saying that a milestone is complete is not proof.

Milestone attempts are bounded. Exhausting the configured attempt budget must stop automatic retry and surface a blocked/failed condition for replanning or human review.

## Checkpoints, interruption, and resume

Checkpoint durable plan state at meaningful boundaries so task/run/envelope references, budgets, milestones, decisions, discoveries, and interruption context survive process or session loss.

If interruption occurs during or around an effectful operation, perform **reconciliation** before retry. Determine whether the effect happened, partially happened, or did not happen using trusted repository/execution evidence. An interrupted plan must not blindly replay an effect merely because its conversational context was lost.

## Bounded replanning

Every ExecPlan has a finite replan budget. Replanning must record why the previous plan became inadequate, increment persisted budget usage, and produce a revised bounded path. When the replan budget is exhausted, automatic replanning stops; the plan becomes blocked/failed rather than looping indefinitely.

## Evidence and completion

Completion requires independent **verification** through the repository's existing VerificationFabric and relevant CI/harness gates. Plans cannot self-verify and cannot treat generated evidence claims as equivalent to checks that actually ran.

Where an `ExecutionEnvelope` declares required evidence, all required checks must be present and passing before the associated work is considered verified. Unknown or missing required evidence fails closed.

## Plan maintenance rules

- Keep Progress current after every milestone transition.
- Update Surprises & Discoveries when repository truth changes the plan.
- Update the Decision Log at the time of the decision.
- Update Outcomes & Retrospective from final evidence.
- Keep typed runtime state authoritative for lifecycle, attempts, and replan budgets.
- Reconcile interrupted effectful work before retry.
- Never use a plan to mint approvals, create execution authority, widen capabilities, or mark itself verified.
- Preserve the existing TaskGraph, ExecutionEnvelope, AuthorizationGrant, EvidenceStore, and VerificationFabric boundaries rather than duplicating them inside the plan model.
