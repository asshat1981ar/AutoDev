# ForgeLoop Skill Routing

## Purpose

AutoDev should not activate every development behavior for every task. Skills are declarative, reusable capabilities selected from repository and task evidence.

The first implementation is deterministic and local-first. It provides a baseline that can later be compared with model-based or learned routing.

## Flow

```text
DevelopmentContract
       |
       v
   SkillRegistry
       |
       v
 deterministic scoring
(term + capability + risk + cost)
       |
       v
    SkillRoute
       |
       +--> selected skill ids
       +--> scores
       +--> routing reasons
```

## DevelopmentContract

The contract normalizes the requested outcome before routing. It includes the goal, acceptance criteria, constraints, required capabilities, and a risk ceiling.

This keeps routing attached to an explicit development objective instead of raw conversational history.

## SkillDefinition

A skill declares:

- stable id and description;
- activation terms;
- required capabilities;
- compatible logical agent roles;
- expected risk;
- verification expectations;
- relative cost hint.

A skill is not an agent or process. Logical agents may execute multiple skills, allowing AutoDev to grow capability without process proliferation.

## Routing evidence

Every selected skill includes a deterministic score and reasons such as `term:context` or `capability:run_test`.

Routing is bounded by `max_skills` and rejects skills above the contract risk ceiling. Equal scores prefer lower-cost skills, then stable skill ids.

## Current catalog

The initial catalog contains broad primitives:

- `map-repository`
- `design-context-fabric`
- `build-vertical-slice`
- `debug-systematically`
- `test-risk-first`
- `review-change-gate`

The catalog is intentionally small. Additional ForgeLoop skills should be added when they have distinct activation, policy, or verification behavior.

## Evolution path

1. Add richer contract signals from repository observation and ContextPack evidence.
2. Add agent-role compatibility to the routing decision.
3. Record routes and outcomes as durable evidence.
4. Build an evaluation dataset from historical tasks.
5. Compare deterministic routing with learned/model routing.
6. Permit learned routing only when it improves measured selection quality without weakening policy controls.

The policy layer remains authoritative regardless of how a skill is selected.
