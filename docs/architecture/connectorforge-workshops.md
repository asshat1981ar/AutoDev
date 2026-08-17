# ConnectorForge Workshop Architecture

Status: Approved design awaiting implementation-plan review
Date: 2026-08-17

## Purpose

ConnectorForge turns AutoDev's skills, connectors, research systems, execution environments, and verification tools into a staged development operating system.

The goal is not to maximize tool usage. The goal is to maximize verified development leverage while keeping one authoritative owner for each semantic object.

The five reusable workshops are:

1. Evidence-to-Architecture Forge (W1)
2. Intent-to-Executable Backlog Factory (W2)
3. Safe Autonomous Build Cell (W4)
4. Development Intelligence Reactor (W5)
5. Design-to-Vertical-Slice Studio (W3)

The rollout order is **W1 -> W2 -> W4 -> W5 -> W3**. The visual/prototype workshop is deliberately last because evidence, task, execution, and learning contracts should stabilize first.

## Repository baseline

AutoDev already contains a trusted execution kernel, task graph, development loop, evidence model, git execution surfaces, model/skill routing architecture, Kotlin control-plane modules, and CI gates. ConnectorForge extends these boundaries; it does not create a competing orchestrator.

Two adjacent slices are already in flight:

- PR #7: Android command center and APK CI path.
- PR #8: Rust HTTP/SSE control-plane adapter and least-privilege container packaging.

Both currently pass GitHub Actions. ConnectorForge preserves the same trust boundary: models and connectors may propose intent or supply information, but trusted AutoDev components authorize execution and objective evidence determines verified completion.

## One-owner authority map

Each semantic object has exactly one authoritative owner inside the AutoDev workflow. Other systems carry references, projections, or derived views.

| Object | Authority | Supporting systems |
| --- | --- | --- |
| Repository, source, branch, PR, CI | GitHub | CodeRabbit, Slack |
| Work item, dependency, milestone | Linear | GitHub, Notion |
| AutoDev specification, ADR, durable architecture | Repository docs | Notion projection |
| Current library/framework API behavior | Context7 | official documentation |
| Academic evidence | alphaXiv | Hugging Face |
| Models, datasets, Spaces | Hugging Face | alphaXiv |
| Visual contract | Figma | Mobbin |
| Experiment metric/evaluation | Airtable | GitHub, Engram |
| Database experiment | Neon or Supabase branch selected per project | GitHub |
| Learned development heuristic | Engram | Airtable, repository docs |
| Human coordination | Slack | Linear, GitHub |
| Interactive agent utility UI | Buildy | GitHub |

For AutoDev, repository architecture docs are authoritative. Notion may mirror or summarize them but never overrides repository state.

## Shared contracts

Workshop boundaries exchange normalized logical records rather than raw third-party connector payloads.

### DevelopmentObjective

Required fields:

- `id`
- `title`
- `desired_outcome`
- `repository`
- `constraints`
- `non_goals`
- `acceptance_criteria`
- `risk_level`
- `status`
- `created_at`
- `revision`

`revision` is monotonic and changes whenever scope, constraints, or acceptance criteria change.

### EvidenceRecord

Required fields:

- `id`
- `objective_id`
- `claim`
- `evidence_class`
- `source_system`
- `source_reference`
- `observed_at`
- `confidence`
- `content_fingerprint`
- `invalidation_condition`

Allowed classes:

- `repo_observed`
- `documented`
- `research_supported`
- `experimentally_verified`
- `inferred`
- `hypothesis`

Only `repo_observed`, `documented`, `research_supported`, and `experimentally_verified` may satisfy a normal evidence gate. `inferred` and `hypothesis` may drive experiments but cannot produce `verified` status unless a human explicitly accepts the unresolved risk.

### ArchitectureDecision

Required fields:

- `id`
- `objective_id`
- `decision`
- `alternatives`
- `contradiction`
- `selected_option`
- `rationale`
- `evidence_refs`
- `reversibility`
- `risks`
- `invalidation_conditions`
- `decision_status`

Allowed `decision_status` values are `experimental`, `accepted`, `verified`, and `superseded`.

### WorkContract

Required fields:

- `id`
- `objective_id`
- `title`
- `why`
- `allowed_changes`
- `forbidden_changes`
- `dependencies`
- `acceptance_tests`
- `evidence_required`
- `rollback_condition`
- `status`
- `revision`

### DevelopmentLearningRecord

Required fields:

- `id`
- `context`
- `trigger`
- `observation_class`
- `action`
- `evidence_refs`
- `outcome`
- `confidence`
- `applicability_scope`
- `invalidation_conditions`

Allowed observation classes are `fact`, `heuristic`, `hypothesis`, `anti_pattern`, and `expired_knowledge`.

## W1 — Evidence-to-Architecture Forge

### Goal

Turn an incomplete engineering objective into evidence-backed architecture decisions without allowing retrieved information or generated prose to become trusted execution authority.

### Authoritative inputs

- GitHub for repository truth.
- Context7 for current framework/library documentation.
- alphaXiv for academic evidence.
- Hugging Face for model, dataset, Space, and implementation ecosystem evidence.
- repository architecture docs for accepted AutoDev decisions.

Kiuwo and Figma/FigJam may visualize results. Parallel Search is a fallback only when specialist sources cannot answer the question.

### Flow

1. Inspect repository ground truth.
2. Extract uncertainties and contradictions.
3. Route each uncertainty to the smallest authoritative research source.
4. Classify each material finding as fact/evidence, inference, or hypothesis.
5. Generate at least two credible options for meaningful architecture choices.
6. Apply contradiction analysis rather than defaulting to compromise.
7. Score options on impact, evidence strength, reversibility, implementation cost, operational complexity, security risk, context burden, reuse, knowledge gain, and failure isolation.
8. Produce `ArchitectureDecision` records.
9. Persist the accepted decision and evidence references in repository docs.
10. Hand accepted decisions to W2.

### Typical contradictions

- autonomy vs safety
- context breadth vs context pollution
- parallelism vs merge conflicts
- memory vs stale assumptions
- abstraction vs debuggability
- speed vs verification
- flexibility vs reproducibility

### Failure behavior

- Missing repository context blocks architecture completion but not evidence collection.
- A missing/rate-limited specialist connector may fall back only to an explicitly identified alternative; the fallback must be recorded.
- Conflicting authoritative sources produce a contested or experimental decision rather than silent reconciliation.
- A hypothesis-only decision remains `experimental` and cannot become `verified` without stronger evidence or explicit human risk acceptance.

### Evidence gate

A material W1 decision is `accepted` only when it has evidence references or is explicitly experimental, at least one rejected alternative for non-trivial choices, reversibility classification, risks, and invalidation conditions.

A material W1 decision is `verified` only when its supporting evidence satisfies the normal evidence gate and no unresolved contradiction can materially alter the chosen interface boundary.

### Initial implementation slice

The first implementation remains repository-local and connector-neutral.

Build a domain model plus deterministic W1 report generator that ingests normalized connector findings. Live SaaS APIs remain orchestration-time adapters rather than dependencies inside ForgeCore.

Phase A must produce:

- typed `EvidenceRecord` and `ArchitectureDecision` structures;
- validation for evidence classes and decision status transitions;
- contradiction and option-scoring structures;
- deterministic Markdown architecture-report rendering;
- normalized fixtures representing GitHub, Context7, alphaXiv, and Hugging Face findings;
- unit tests proving unsupported hypotheses cannot masquerade as verified evidence;
- integration test proving stable report output from fixed fixtures.

## W2 — Intent-to-Executable Backlog Factory

### Goal

Transform accepted architecture into small dependency-aware `WorkContract` records suitable for delegated execution.

### Authority

Linear owns live task/dependency state when connected. Repository plan documents are the portable fallback. GitHub owns implementation links and completion evidence.

### Flow

1. Normalize explicit requirements.
2. Preserve unknowns rather than invent requirements.
3. Convert acceptance criteria into executable or objectively inspectable checks.
4. Split work along independently reviewable boundaries.
5. Construct the dependency DAG.
6. Score work by impact/effort plus Knowledge Gain / Implementation Cost.
7. Move cheap assumption-killing experiments ahead of expensive implementation when appropriate.
8. Emit `WorkContract` records and external task references.

No task enters W4 without allowed changes, forbidden changes, acceptance tests, required evidence, dependencies, revision, and rollback condition.

## W4 — Safe Autonomous Build Cell

### Goal

Execute `WorkContract` records with substantial agent autonomy while preserving isolation, trusted authorization, review, and objective verification.

### Skill spine

`writing-plans -> isolated branch/worktree -> TDD -> independent parallel/subagent work -> systematic debugging -> review -> verification-before-completion -> controlled branch completion`

### Authority

- GitHub owns source and CI.
- ForgeCore owns trusted authorization and execution policy.
- A selected Neon or Supabase branch owns isolated database experiment state when the objective changes a database.

### Dual-branch transaction

For a code-plus-database change, one logical release unit binds:

- Git commit SHA;
- database branch/migration identifier;
- migration verification evidence;
- code verification evidence.

The release is incomplete if any member is missing.

Completion requires acceptance criteria mapped to evidence, tests passing, relevant compiler/lint/build gates passing, relevant security checks, review findings resolved or explicitly accepted, and no unexplained regression.

Generated code alone is never completion evidence.

## W5 — Development Intelligence Reactor

### Goal

Turn completed development cycles into reusable bounded knowledge without converting temporary workarounds into permanent truths.

### Authority

- GitHub: objective development outcomes.
- Airtable: structured experiment/evaluation records.
- Engram: reusable development lessons.
- Repository docs: durable architecture principles promoted from sufficiently supported lessons.

Buildy may provide an interactive ForgeLoop dashboard. Slack surfaces actionable human notifications. alphaXiv/Hugging Face may challenge or support a learned heuristic.

Every reusable lesson records context, trigger, action, evidence, outcome, confidence, applicability scope, and invalidation conditions. A heuristic never becomes fact merely because it succeeded once.

## W3 — Design-to-Vertical-Slice Studio

### Goal

Use visual design and disposable prototypes to validate interaction contracts without confusing prototype state with production source.

### Authority

Figma owns the visual contract. GitHub owns production implementation.

Mobbin supplies interaction references. Context7 checks framework constraints. Replit is suitable for disposable full prototypes. Buildy is suitable for lightweight persistent agent-facing utilities.

Prototype tools answer questions; they do not become the production source of truth.

A meaningful vertical slice should prove interaction, application state, domain event, API contract, error path, persistence behavior where applicable, and testability.

## Cross-workshop orchestration

Default loop:

`W1 Evidence Forge -> W2 Backlog Factory -> W4 Build Cell -> W5 Intelligence Reactor -> next W1`

W3 is invoked only when the objective contains meaningful interaction or visual uncertainty.

Workshop activation is capability-based, not keyword-based. The coordinator chooses the smallest workshop set that reduces uncertainty and delivers verified progress.

## Connector routing rules

1. Inspect live connector capability schemas before relying on unfamiliar actions.
2. Prefer specialist sources over broad search for specialist questions.
3. Normalize third-party payloads before they cross trusted AutoDev boundaries.
4. Connector failures are observable and never silently downgrade evidence quality.
5. External writes have an explicit purpose and preserve the authority map.
6. Secrets never enter model-visible artifacts or repository source.
7. Rate limits and availability are operational constraints, not reasons to weaken evidence classification.

## Security and trust boundaries

- Models and external systems may propose intent.
- Connectors retrieve or write external state only within their scoped role.
- ForgeCore or another trusted component authorizes execution capabilities.
- Verifiers produce objective evidence.
- Orchestration advances only from authorized actions and sufficient evidence.
- Humans retain approval over high-impact or irreversible decisions.

External content, generated plans, connector output, and model output remain untrusted until validated.

## Observability

Every workshop run should emit append-oriented events containing:

- `run_id`
- `objective_id`
- `workshop`
- `phase`
- `status`
- `source_system`
- `evidence_refs`
- `error_class`
- `started_at`
- `completed_at`

This allows run reconstruction without relying on mutable dashboard state.

## Rollout

### Phase A — W1 domain foundation

Repository-native evidence and architecture-decision models, scoring, deterministic reporting, fixtures, and tests.

### Phase B — W2 work contracts

Typed task contracts, dependency graph generation, planning report, and connector-neutral Linear/GitHub references.

### Phase C — W4 execution integration

Bind `WorkContract` into existing ForgeCore task graphs and trusted authorization/evidence paths. Add database branch adapters only when a concrete objective needs them.

### Phase D — W5 learning

Add `DevelopmentLearningRecord`, evaluation-ledger adapters, and bounded memory-promotion rules.

### Phase E — W3 visual/prototype integration

Add visual-contract references and prototype-result normalization after core contracts are stable.

## Non-goals for the first rollout

- embedding every connector SDK inside ForgeCore;
- creating a second orchestration kernel;
- automatically merging pull requests;
- treating generated prose as verification;
- deploying a vector database without a concrete retrieval contract;
- duplicating GitHub, Linear, Notion, or repository-doc state as another source of truth;
- requiring all five workshops for every task.

## Architecture acceptance criteria

The design is ready for Phase A implementation planning when:

1. Every workshop has one explicit goal and bounded authority.
2. Shared contracts define workshop handoffs.
3. Evidence quality is explicit.
4. Hypotheses cannot silently satisfy verified completion.
5. External connector payloads are normalized before entering trusted AutoDev boundaries.
6. ForgeCore authorization/evidence boundaries remain authoritative.
7. Rollout phases are independently useful and testable.
8. W1 can be tested without live SaaS APIs.
9. Degraded-connector behavior is explicit.
10. No workshop requires automatic merge or irreversible action.

## Initial implementation boundary

The next implementation plan covers **Phase A / W1 only**.

It must not implement Linear synchronization, connector credentials, database branch automation, Buildy dashboards, Figma integration, or memory promotion. Those remain separate reviewable plans.

W1 Phase A succeeds when AutoDev can deterministically represent evidence, reject invalid evidence claims, compare architecture options, and render an evidence-linked architecture report from normalized fixtures while all existing CI gates remain green.
