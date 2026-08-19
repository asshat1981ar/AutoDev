# ADR-002: ExecPlan coordination is authority-free

## Status

Accepted

## Context

AutoDev needs durable multi-hour plans that survive interruption and context loss. Existing ForgeCore objects already own execution authorization, workspace confinement, evidence, verification, and durable task/envelope lifecycle. Treating a plan document or plan-state object as an execution token would create a second authority path and weaken the kernel boundary.

## Decision

`ExecPlan` is durable coordination state only. It may reference task, run, envelope, checkpoint, decision, discovery, and milestone identities, but it cannot contain or mint `AuthorizationGrant`, directly execute effects, widen capabilities, or mark work independently verified.

Human-readable `PLANS.md` is the narrative contract. Typed ForgeCore state is canonical for lifecycle correctness and bounded budgets. Interrupted effectful work requires reconciliation before the plan can resume running.

## Consequences

Long-running orchestration can be recovered without replaying plan prose as trusted state. External harness/plugin formats can be imported later without becoming execution authorities. Additional adapters must translate requested capabilities into ForgeCore policy inputs rather than bypassing policy.

The cost is an extra coordination layer and explicit reconciliation logic between plan state and effectful execution state. This duplication is intentional because coordination and authority have different trust requirements.
