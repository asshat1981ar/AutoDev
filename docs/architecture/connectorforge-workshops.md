# ConnectorForge Workshop Architecture

Status: Proposed implementation architecture, approved for specification
Date: 2026-08-17

## Purpose

ConnectorForge turns AutoDev's growing set of skills, connectors, research systems, execution environments, and verification tools into a staged development operating system.

The goal is not to maximize tool usage. The goal is to maximize verified development leverage while keeping one authoritative owner for each semantic object.

The five workshops are reusable operating workflows:

1. Evidence-to-Architecture Forge
2. Intent-to-Executable Backlog Factory
3. Safe Autonomous Build Cell
4. Development Intelligence Reactor
5. Design-to-Vertical-Slice Studio

The rollout order is W1 -> W2 -> W4 -> W5 -> W3. This order first strengthens evidence and planning, then execution, then learning, and only then expands visual/prototyping workflows.

## Repository baseline

AutoDev already contains the trusted execution kernel, task graph, development loop, evidence model, git execution surfaces, model/skill routing architecture, Kotlin control-plane modules, and CI gates. ConnectorForge must extend these existing boundaries rather than create a competing orchestrator.

Two current development slices are intentionally treated as adjacent work, not prerequisites for this design:

- PR #7 adds an Android command center and APK CI path.
- PR #8 adds a Rust HTTP/SSE control-plane adapter and least-privilege container packaging.

Both are currently green in GitHub Actions. ConnectorForge must preserve the same trust boundary: model or connector output may propose intent, but trusted AutoDev components authorize execution and verification evidence determines completion.

## Core architecture principle

Each semantic object has one authoritative owner. Other systems carry references, projections, or derived views.

| Object | Authority | Supporting systems |
| --- | --- | --- |
| Repository, source, branch, PR, CI | GitHub | CodeRabbit, Slack |
| Work item, dependency, milestone | Linear | GitHub, Notion |
| Specification, ADR, durable design knowledge | Notion or repository docs | GitHub |
| Current library/framework API behavior | Context7 | official documentation |
| Academic evidence | alphaXiv | Hugging Face |
| Models, datasets, Spaces | Hugging Face | alphaXiv |
| Visual contract | Figma | Mobbin |
| Experiment metric and structured evaluation | Airtable | GitHub, Engram |
| Database experiment | Neon or Supabase branch | GitHub |
| Learned development heuristic | Engram | Airtable, Notion |
| Human coordination | Slack | Linear, GitHub |
| Interactive agent-facing utility UI | Buildy | GitHub |

ConnectorForge must never silently copy authoritative state into another system and later treat the copy as authoritative.

## Shared contracts

All workshops exchange typed logical records. Implementations may serialize these as Rust types, Kotlin data classes, JSON, or connector-specific payloads, but the semantics must remain stable.

### DevelopmentObjective

Represents the user-visible outcome being pursued.

Required fields:

- id
- title
- desired_outcome
- repository
- constraints
- non_goals
- acceptance_criteria
- risk_level
- status
- created_at
- revision

The revision is monotonic and must change whenever acceptance criteria, constraints, or scope change.

### EvidenceRecord

Represents support for a claim or verification result.

Required fields:

- id
- objective_id
- claim
- evidence_class
- source_system
- source_reference
- observed_at
- confidence
- content_fingerprint
- invalidation_condition

Allowed evidence classes:

- repo_observed
- documented
- research_supported
- experimentally_verified
- inferred
- hypothesis

Only the first four classes may satisfy an evidence gate without explicit human acceptance.

### ArchitectureDecision

Required fields:

- id
- objective_id
- decision
- alternatives
- contradiction
- selected_option
- rationale
- evidence_refs
- reversibility
- risks
- invalidation_conditions

### WorkContract

Represents an executable task that may be delegated to an agent.

Required fields:

- id
- objective_id
- title
- why
- allowed_changes
- forbidden_changes
- dependencies
- acceptance_tests
- evidence_required
- rollback_condition
- status
- revision

### DevelopmentLearningRecord

Required fields:

- id
- context
- trigger
- observation_class
- action
- evidence_refs
- outcome
- confidence
- applicability_scope
- invalidation_conditions

Allowed observation classes:

- fact
- heuristic
- hypothesis
- anti_pattern
- expired_knowledge

## Workshop 1: Evidence-to-Architecture Forge

### Goal

Turn an incomplete engineering objective into an evidence-backed architecture decision without allowing retrieved information or generated prose to become trusted execution authority.

### Inputs

- DevelopmentObjective
- current repository state
- explicit user constraints
- unresolved architecture questions

### Authoritative systems

- GitHub for repository truth
- Context7 for current library/framework documentation
- alphaXiv for academic evidence
- Hugging Face for model, dataset, Space, and implementation ecosystem evidence
- repository architecture docs or Notion for durable architecture decisions

### Supporting systems

- Kiuwo for capability maps
- Figma/FigJam for architecture and sequence diagrams where a visual contract adds value
- Parallel Search only when specialist sources do not cover the required external information

### Flow

1. Inspect repository ground truth before researching alternatives.
2. Extract architectural uncertainties and contradictions.
3. Route each uncertainty to the smallest authoritative research source.
4. Classify every important finding as fact, evidence, inference, or hypothesis.
5. Generate at least two credible architectures for meaningful design choices.
6. Apply contradiction analysis rather than compromising both sides of a tension by default.
7. Score alternatives on impact, evidence strength, reversibility, implementation cost, operational complexity, security risk, context burden, reuse, knowledge gain, and failure isolation.
8. Produce ArchitectureDecision records.
9. Persist the accepted decision with evidence references and invalidation conditions.
10. Hand the accepted objective and architecture decisions to Workshop 2.

### Contradiction examples

- autonomy vs safety
- context breadth vs context pollution
- parallelism vs merge conflicts
- memory vs stale assumptions
- abstraction vs debuggability
- speed vs verification
- flexibility vs reproducibility

### Failure behavior

- Missing repository context blocks architectural completion but not evidence collection.
- Missing or rate-limited specialist connector falls back only to another explicitly identified source; the fallback is recorded.
- Conflicting authoritative sources produce a contested decision record rather than silent reconciliation.
- A hypothesis may be selected for experimentation but must not be labeled verified.

### Evidence gate

W1 completes only when every material architecture decision has:

- at least one evidence reference or is explicitly labeled a hypothesis;
- at least one rejected alternative with rationale for non-trivial choices;
- a reversibility classification;
- an invalidation condition;
- no unresolved contradiction that could materially alter the chosen interface boundary.

### Initial implementation slice

The first implementation should stay local to AutoDev and avoid external write dependencies.

Build a repository-native evidence/decision domain model plus a deterministic W1 report generator that can ingest normalized findings produced by connectors. Connectors remain orchestration-time adapters at first rather than hard runtime dependencies inside ForgeCore.

This preserves testability and prevents the trusted kernel from depending directly on SaaS APIs.

The first slice should produce:

- typed EvidenceRecord and ArchitectureDecision structures;
- validation rules for evidence classes and required fields;
- contradiction and option scoring structures;
- a deterministic Markdown architecture report renderer;
- fixtures representing GitHub, Context7, alphaXiv, and Hugging Face findings after normalization;
- unit tests proving that unsupported hypotheses cannot masquerade as verified evidence;
- an integration test producing a stable architecture report from fixed fixtures.

## Workshop 2: Intent-to-Executable Backlog Factory

### Goal

Transform an accepted architecture into small, dependency-aware WorkContracts suitable for delegated execution.

### Authoritative systems

- Linear for task/dependency state when connected
- repository plan documents as the portable fallback
- GitHub for implementation links and completion evidence

### Flow

1. Normalize explicit requirements.
2. Preserve unknowns rather than inventing requirements.
3. Convert acceptance criteria into executable or objectively inspectable checks.
4. Split work along independently reviewable boundaries.
5. Construct dependency DAG.
6. Score work using impact and effort plus Knowledge Gain / Implementation Cost.
7. Move cheap assumption-killing experiments ahead of expensive implementation where appropriate.
8. Emit WorkContracts and persist task references.

### Evidence gate

No task enters Workshop 4 unless it has bounded allowed changes, forbidden changes, acceptance tests, required evidence, dependencies, and rollback condition.

## Workshop 3: Design-to-Vertical-Slice Studio

### Goal

Use visual design and disposable prototypes to validate interaction contracts without confusing prototype state with production source.

### Authoritative systems

- Figma for visual contract
- GitHub for production implementation

### Supporting systems

- Mobbin for interaction references
- Context7 for framework constraints
- Replit for disposable full prototypes
- Buildy for lightweight persistent agent-facing utilities

### Core invariant

Prototype tools answer questions. They do not become the authoritative production codebase.

### Vertical-slice contract

Every meaningful slice should prove interaction, application state, domain event, API contract, error path, persistence behavior where applicable, and testability.

## Workshop 4: Safe Autonomous Build Cell

### Goal

Execute WorkContracts with substantial agent autonomy while preserving isolation, authorization boundaries, review, and objective verification.

### Skill spine

- writing plans
- isolated worktree or branch
- test-driven development
- parallel/subagent execution only for independent work
- systematic debugging on failure
- code review
- verification-before-completion
- controlled branch completion

### Authoritative systems

- GitHub for source and CI
- ForgeCore for trusted authorization and execution policy
- Neon or Supabase branches for isolated database experiments when database changes are involved

### Dual-branch transaction

For a code-plus-database change, the logical release unit binds:

- Git commit SHA
- database branch or migration identifier
- migration verification evidence
- code verification evidence

A release is incomplete if any member of that logical unit is missing.

### Evidence gate

Completion requires acceptance criteria mapped to evidence, relevant tests passing, compiler/lint/build gates passing, relevant security checks, review findings resolved or explicitly accepted, and no unexplained regression.

Generated code is never completion evidence by itself.

## Workshop 5: Development Intelligence Reactor

### Goal

Turn completed development cycles into reusable, bounded development knowledge without converting temporary workarounds into permanent truths.

### Authoritative systems

- GitHub for objective development outcomes
- Airtable for structured experiment/evaluation records
- Engram for reusable development lessons
- repository docs or Notion for durable architecture principles

### Supporting systems

- Buildy for an interactive ForgeLoop development-intelligence dashboard
- Slack for actionable human notifications
- alphaXiv and Hugging Face when a learned heuristic should be compared with external evidence

### Learning rules

Every reusable lesson must record context, trigger, action, evidence, outcome, confidence, applicability scope, and invalidation conditions.

Facts, heuristics, hypotheses, anti-patterns, and expired knowledge are stored separately.

A heuristic must not be promoted to fact because it succeeded once.

## Cross-workshop orchestration

The target sequence is:

Evidence Forge -> Backlog Factory -> Build Cell -> Intelligence Reactor -> next Evidence Forge

The Vertical-Slice Studio is invoked when the objective contains a meaningful interaction or visual uncertainty. It is not mandatory for backend-only work.

Workshop activation is capability-based, not keyword-based. A coordinator should select the smallest workshop set needed to reduce uncertainty and deliver verified progress.

## Connector routing rules

1. Inspect live connector capability schemas before relying on an unfamiliar action.
2. Prefer a specialist source over broad search for specialist questions.
3. Prefer repository-local normalized types over embedding third-party payload shapes into trusted kernel APIs.
4. Connector failures must be observable and must not silently downgrade evidence quality.
5. External write actions require an explicit purpose and must preserve the authoritative ownership map.
6. Secrets never enter model-visible artifacts or repository source.
7. Rate limits and availability are runtime constraints, not reasons to weaken evidence classification.

## Security and trust boundaries

ConnectorForge must preserve AutoDev's existing intent/policy/evidence separation.

- Models and external systems may propose intent.
- Connectors may retrieve or write external state according to their scoped role.
- ForgeCore or another trusted component authorizes execution capabilities.
- Verifiers produce objective evidence.
- Orchestration advances state only from authorized actions and sufficient evidence.
- Humans retain approval over high-impact or irreversible decisions.

External content, retrieved documents, generated plans, and model outputs are untrusted inputs until validated.

## Observability

Every workshop run should emit a compact event stream with:

- run_id
- objective_id
- workshop
- phase
- status
- source_system
- evidence_refs
- error_class
- started_at
- completed_at

Events should be append-oriented so a run can be reconstructed without relying on mutable dashboard state.

## Rollout plan

### Phase A: W1 domain foundation

Repository-native evidence and architecture-decision models plus deterministic reporting and tests.

### Phase B: W2 work contracts

Typed task contracts, dependency graph generation, planning report, and connector-neutral Linear/GitHub references.

### Phase C: W4 execution integration

Bind WorkContracts into the existing ForgeCore task graph and trusted authorization/evidence path. Add database branch adapters only when a concrete database-changing objective requires them.

### Phase D: W5 learning

Add DevelopmentLearningRecord, evaluation ledger adapters, and bounded memory promotion rules.

### Phase E: W3 visual/prototype integration

Add visual-contract references and prototype result normalization after core evidence, planning, execution, and learning contracts are stable.

## Non-goals for the first rollout

- embedding every connector SDK inside ForgeCore;
- creating a second orchestration kernel;
- automatically merging pull requests;
- treating generated prose as verification;
- deploying a vector database without a concrete retrieval contract;
- duplicating GitHub, Linear, or Notion state in another system as a new source of truth;
- requiring all five workshops for every task.

## Acceptance criteria for the ConnectorForge architecture

The architecture is ready for implementation planning when:

1. Every workshop has one explicit goal and bounded authority.
2. Shared contracts define the handoffs between workshops.
3. Evidence quality is represented explicitly.
4. Hypotheses cannot satisfy a verified completion gate without explicit human acceptance.
5. External connector payloads are normalized before entering trusted AutoDev boundaries.
6. Existing ForgeCore authorization/evidence boundaries remain authoritative.
7. The rollout order produces independently useful, testable increments.
8. W1 has a repository-local first slice that does not require live SaaS APIs to test.
9. Failure and degraded-connector behavior is explicit.
10. No workshop requires automatic merge or irreversible action.

## Initial implementation boundary

The next implementation plan should cover only Phase A / Workshop 1.

It should not implement Linear synchronization, external connector credentials, database branch automation, Buildy dashboards, Figma integration, or memory promotion. Those remain separate independently reviewable plans.

The first W1 implementation is successful when AutoDev can deterministically represent evidence, reject invalid evidence claims, compare architecture options, and render an evidence-linked architecture report from normalized fixtures while all existing CI gates remain green.
