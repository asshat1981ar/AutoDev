# AutoDev Federated Harness Kernel — Production Evolution Design

## Purpose

Evolve AutoDev from its current evidence-first multi-agent runtime into a polished Android-first, multiplatform agentic software-development system capable of durable multi-hour work while preserving ForgeCore as the sole trusted execution authority.

The design adopts the useful semantics of living ExecPlans and modern agent harnesses without delegating authorization, policy, or verification authority to plugins, models, MCP servers, skills, or external frameworks.

## Architectural invariant

Agents and harness assets propose intent. ForgeCore authorizes and executes effects. Independent verification produces evidence. Orchestration advances only from trusted state plus evidence.

No plugin, skill, agent profile, MCP server, model response, plan file, or evaluation result may mint an `AuthorizationGrant`, widen capabilities, bypass workspace confinement, or mark its own work verified.

## Target stack

1. Android-first AutoDev Workspace — Compose UI over KMP contracts, with desktop/web/server companions.
2. Durable Development Runtime — living ExecPlans, checkpoints, recovery, queues, cancellation, and resumable sessions.
3. Agent Harness Fabric — normalized agents, skills, tools, MCP servers, hooks, prompts, policies, workflows, evaluators, context providers, and provider/model profiles.
4. Adaptive Orchestration — TaskGraph scheduling, context selection, subagent isolation, evidence-backed routing, bounded replanning, and learned toolset selection.
5. Evidence and Evaluation Fabric — build/test/lint/static/security evidence, historical replay, fault injection, configuration tournaments, and regression detection.
6. ForgeCore Trusted Kernel — capability policy, grants, workspace confinement, effect execution, provenance, envelopes, and lifecycle validation.

## Durable ExecPlan model

A long-running development objective is a durable executable object rather than transient prompt context. Its persisted representation contains:

- immutable plan identity, goal, creation provenance, and schema version;
- milestones and dependency edges;
- current progress and checkpoint identity;
- decisions, discoveries, assumptions, and unresolved questions;
- task/run/envelope references rather than duplicated execution authority;
- selected harness profile and asset references;
- required verification contracts and produced evidence references;
- bounded retry/replan budgets;
- interruption, cancellation, and recovery state;
- retrospective and final outcomes.

The human-readable `PLANS.md` convention is the durable narrative companion. Machine state must not be parsed from prose when a typed representation is required for correctness.

## Harness Asset Protocol

Introduce a normalized `HarnessAsset` protocol with these asset kinds:

- Skill
- AgentProfile
- Tool
- McpServer
- Hook
- Prompt
- Policy
- Workflow
- Evaluator
- ContextProvider

Every asset has stable identity, version, provenance, declared capabilities, compatibility constraints, integrity metadata, configuration schema, and trust classification. Provider/model-specific behavior is expressed through declarative harness profiles rather than scattered conditionals.

Adapters may import external ecosystems into this protocol. Imported assets remain untrusted configuration until policy evaluates their requested capabilities.

## Plugin trust model

Plugin discovery and plugin execution are separate operations. Discovery may inspect manifests without granting effects. Activation produces a capability request evaluated by ForgeCore policy. High-risk capabilities require trusted approval. Plugin integrity and provenance are recorded in evidence.

Initial trust levels:

- built-in: shipped and verified with AutoDev;
- local: explicitly installed from the local workspace;
- verified: integrity/provenance validated against configured trust metadata;
- untrusted: discoverable but denied effectful activation until policy permits it.

Trust level never implies unrestricted capability.

## Long-running orchestration

The durable runtime composes existing `TaskGraph`, `ExecutionEnvelope`, and verified development-loop semantics rather than replacing them. A checkpoint captures enough orchestration state to resume after process death without replaying an already-authorized effect blindly.

Resume rules:

- verified work is not re-executed;
- blocked approval remains blocked;
- interrupted effectful operations require reconciliation before retry;
- replanning preserves evidence and decision history;
- retries remain bounded;
- cancellation is explicit and persisted;
- stale external state causes revalidation rather than optimistic continuation.

## Android-first workspace

The Android command center becomes the reference interactive client. It remains a thin client over KMP/server contracts and receives no direct ForgeCore authority.

Primary surfaces:

- project and durable-run dashboard;
- milestone/task graph and progress timeline;
- plan/decision/discovery viewer;
- approval inbox with exact capability/effect scope;
- diff and patch review;
- verification/evidence viewer;
- agent/subagent activity and provenance;
- offline objective queue with deterministic replay;
- interruption, cancellation, reprioritization, and resume controls;
- diagnostics that explain blocked/failed states in user terms.

Android must remain usable under Termux/mobile constraints: no mandatory Docker, Bun, PTY, or desktop-only runtime assumption.

## Adaptive learning boundary

AutoDev may learn which toolsets, agents, profiles, context strategies, and workflows correlate with verified outcomes. Learning data is evidence-backed and evaluated against frozen tasks or controlled experiments.

Learning may change recommendations or routing within existing policy. It may not self-expand authority, install effectful assets without policy, weaken verification, or promote a configuration solely because the same agent reports success.

## Evaluation and simulation

Extend the self-evaluation factory into a Simulation/Eval Lab. Candidate harness configurations are evaluated against deterministic historical tasks plus controlled perturbations. Important dimensions include correctness, required-evidence completion, recovery behavior, context efficiency, latency, resource consumption, unsafe-action rejection, user intervention rate, and regression rate.

Simulation results are advisory evidence. Promotion to a default profile requires an explicit promotion policy and independent verification.

## Production program

The program is decomposed into independently reviewable subprojects:

1. ExecPlan control plane.
2. Harness Asset Protocol.
3. Harness adapters and declarative profiles.
4. Durable long-running orchestration.
5. Context intelligence and provenance.
6. Adaptive evidence-backed routing/learning.
7. Android Workspace v1.
8. Multiplatform parity.
9. Simulation/Eval Lab expansion.
10. Production hardening and plugin supply-chain trust.
11. Developer experience and onboarding.
12. Release qualification and production-readiness scorecard.

Each subproject receives its own implementation plan and slice-sized PRs. Structural/kernel changes require ADRs.

## First implementation slice

The first slice is deliberately narrow: establish the ExecPlan control plane and repository planning contract before adding plugin/runtime complexity.

It will:

- add a repository-root `PLANS.md` defining AutoDev's living-plan requirements;
- add typed durable plan state in ForgeCore without granting execution authority to plans;
- test serialization, lifecycle/recovery invariants, bounded budgets, and authority separation;
- connect plan identity to existing task/run/envelope references;
- document how future workers update Progress, Discoveries, Decision Log, and Outcomes;
- enforce key planning-contract drift in the repository harness.

## Success criteria

The architecture is successful when a development objective can run for hours across interruptions, resume deterministically, use heterogeneous harness assets and subagents, remain operable from Android, and finish only when independent evidence satisfies its declared verification contract.

A production release additionally requires reproducible builds, installation-tested Android artifacts, supply-chain/provenance checks, failure recovery tests, accessibility/usability validation, benchmark baselines, and a green production-readiness scorecard.

## Non-goals

- Replacing ForgeCore with an external agent framework.
- Giving the Android client direct execution authority.
- Treating PLANS.md prose as trusted machine state.
- Unbounded autonomous retries or self-modification.
- Installing every available plugin or MCP server by default.
- Declaring success from model or agent self-report.
