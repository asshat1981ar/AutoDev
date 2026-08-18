# Capability Routing Dataset v1 Design

Date: 2026-08-18
Status: Review required before implementation

## 1. Goal

Extend AutoDev's existing advisory toolset-learning memory into a deterministic capability-routing evidence layer that can answer:

```text
Given TaskProfile X and EnvironmentProfile E,
which ordered CapabilityBundle B has the strongest observed evidence
for producing a verified result with the least coordination/context cost?
```

This is the first bounded slice of the broader CapabilityForge program. It does **not** automatically install tools, activate skills, grant capabilities, change policy, choose execution authority, or promote candidates.

## 2. Existing foundation

AutoDev already contains the key primitives this slice should reuse rather than duplicate:

- `memory/toolsets/patterns.jsonl`: validated repository-local records of successful and failed tool/skill/workflow combinations.
- `scripts/validate_toolset_memory.py`: standard-library dataset validator wired into CI.
- `crates/autodev-eval`: historical exact-SHA evaluation infrastructure for empirical comparisons.
- `crates/forge-core/src/capability_gap.rs`: side-effect-free gap classification, candidate staging, and evidence-gated promotion recommendations.
- `crates/forge-core/src/skill.rs`: logical skill registry/routing.
- `.cline/skills/`: architecture-design, debugging, release-readiness, repo-recon, research-validation, security-review, test-strategy, vertical-slice.
- `.cline/agents/`: architect, builder, reviewer, scout, security, verifier.
- `.cline/hooks/`: deterministic task/tool/context hooks.
- `.cline/plugins/project-fabric`: repository-local development plugin.
- `.cline/mcp/profiles.json`: scoped external MCP profiles.
- provider-neutral model routing and deterministic mock providers in ForgeCore.

The architecture already enforces the core safety boundary: agent output is intent, ForgeCore is the privileged execution boundary, evidence is separate from authority, and capability-gap candidates are proposal-only.

## 3. Program decomposition

The user's broader objective is intentionally decomposed into independently testable slices:

1. **Capability Routing Dataset v1** — this spec.
2. Capability bundle ranker and query API.
3. Token/context/tool-call instrumentation.
4. Agent-combination experiment harness.
5. Skill lifecycle + retirement evidence.
6. MCP/connector discovery and evaluation registry.
7. Framework/dependency candidate evaluator.
8. Capability-gap auto-observation adapters.
9. Optional local analytics over a larger experiment corpus.
10. Only after evidence: controlled installation/promotion workflows.

Each later slice must preserve `evidence != authority`.

## 4. Current capability inventory

### 4.1 Strong

- Rust trusted execution and policy boundary.
- Typed agent actions and evidence/provenance.
- Repository-constrained read/write/patch/Git operations.
- Durable orchestration and verification.
- Deterministic historical evaluation.
- Repository-local toolset learning records.
- Evidence-gated capability-gap staging.
- GitHub/CI-backed verification when local tooling is unavailable.
- Cline specialist skills and role prompts.

### 4.2 Adequate

- Provider-neutral model routing.
- Context selection primitives.
- Cline hooks for guardrails and context compaction.
- External MCP profiles as configuration.
- Android command-center workflow surface.

### 4.3 Weak or missing

1. No common machine-readable `TaskProfile` across experiments.
2. No normalized `CapabilityProfile` inventory with cost/latency/authority metadata.
3. Toolset patterns store qualitative outcomes but not comparable metric vectors.
4. No deterministic bundle-scoring/query layer.
5. Token/context/tool-call usage is not captured as first-class experiment evidence.
6. No explicit negative-interaction/synergy records between capabilities.
7. No measured agent-topology comparison harness (single agent vs specialists vs parallel workstreams).
8. No lifecycle dataset for skill promotion/deprecation/retirement.
9. MCP profiles exist, but there is no evidence-backed discovery/evaluation registry for candidate MCP servers/connectors.
10. No framework/dependency candidate evaluator that compares adoption value against maintenance/trust-surface cost.

## 5. Scope of v1

V1 adds three compact JSONL datasets plus deterministic validation and analysis. It deliberately remains outside ForgeCore authority.

```text
memory/capabilities/
  capabilities.jsonl
  experiments.jsonl
  README.md
```

Existing `memory/toolsets/patterns.jsonl` remains supported. V1 does not migrate or invalidate it.

### 5.1 CapabilityProfile

One record per reusable capability:

```json
{
  "schema_version": "capability-profile-v1",
  "capability_id": "github-connector",
  "kind": "connector",
  "domain": ["repository", "ci"],
  "authority": "write_scoped",
  "determinism": "external_state",
  "availability": "installed",
  "strengths": ["repository truth", "CI evidence"],
  "constraints": ["network required"],
  "verification": ["exact-head status", "diff inspection"],
  "estimated_context_cost": "low",
  "estimated_latency": "medium",
  "maintenance": "external",
  "last_validated": "2026-08-18"
}
```

Allowed `kind` values:

```text
skill
agent
connector
mcp
hook
plugin
framework
library
runtime
model_provider
verification
memory
context_strategy
ci
```

Allowed authority classes:

```text
none
read_only
proposal_only
write_scoped
privileged_kernel
```

Profiles describe capability properties; they do not grant the capability.

### 5.2 TaskProfile

Experiments embed a normalized task profile:

```json
{
  "task_class": "rust_feature",
  "languages": ["rust"],
  "risk": "medium",
  "complexity": "medium",
  "research_need": "low",
  "context_size": "medium",
  "parallelizability": "low",
  "verification_burden": "high",
  "security_sensitivity": "high",
  "environment": ["github_actions", "no_local_rust"]
}
```

V1 uses closed enums for dimensions that will be compared; free text is limited to notes/evidence references.

### 5.3 ExperimentRecord

```json
{
  "schema_version": "capability-experiment-v1",
  "experiment_id": "forgeos-a1-remote-tdd",
  "task_profile": { ... },
  "ordered_bundle": ["superpowers-tdd", "github-connector", "github-actions"],
  "outcome": "success",
  "verification_level": "full_ci",
  "sample_size": 1,
  "metrics": {
    "tool_calls": 0,
    "agents": 1,
    "rework_cycles": 1,
    "human_interventions": 0,
    "input_tokens": null,
    "output_tokens": null,
    "elapsed_seconds": null
  },
  "defects_found": [],
  "failure_modes": [],
  "evidence_refs": ["ci:32133875901"],
  "last_validated": "2026-08-18"
}
```

Unknown metrics are `null`, never fabricated.

## 6. Deterministic analysis

Add a standard-library Python analyzer:

```text
scripts/analyze_capability_routing.py
```

Inputs:

- `capabilities.jsonl`
- `experiments.jsonl`
- existing `memory/toolsets/patterns.jsonl`

V1 outputs a JSON report containing:

- coverage by task class;
- capability usage counts;
- bundle success/failure counts;
- evidence-weighted confidence;
- known failure modes;
- missing measurements;
- top capability gaps by deterministic priority score;
- suggested experiments, not automatic routing decisions.

### 6.1 Gap score

Use integer arithmetic to keep results deterministic:

```text
GapPriority = frequency × impact × reuse × strategic_fit × confidence
              ------------------------------------------------------
              max(1, effort × maintenance × operational_risk)
```

All inputs use closed integer scales 1..5. The report must show component scores so recommendations are auditable.

### 6.2 Bundle evidence score

V1 must not pretend the small corpus supports sophisticated ML. Rank only observed bundles:

```text
EvidenceScore = verified_successes × verification_weight × confidence_weight
                - failures × failure_weight
                - safety_regressions × safety_penalty
```

No model training or causal claim is made in v1.

## 7. Seed inventory

The first inventory should cover capabilities already present and evidenced in the repository, including at minimum:

- ForgeCore
- GitHub connector
- GitHub Actions
- Superpowers TDD
- Superpowers systematic debugging
- Superpowers verification-before-completion
- Cline repo-recon
- Cline architecture-design
- Cline debugging
- Cline test-strategy
- Cline security-review
- Cline vertical-slice
- Cline release-readiness
- Cline research-validation
- Cline project-fabric plugin
- Cline hooks
- Cline MCP profiles
- autodev-eval historical evaluation
- toolset learning memory
- capability-gap discovery
- Ollama provider
- MockProvider
- Engram external memory mirror

External capabilities are inventory observations only. Their inclusion does not make AutoDev depend on them.

## 8. Seed experiments

Seed only evidence already recorded in repository history/toolset memory. Initial conversions should include:

1. remote Rust TDD with GitHub + exact-head CI;
2. subprocess lifecycle security hardening;
3. historical evaluation fixture design;
4. long-running branch reconciliation;
5. minimal Rust lockfile repair;
6. temporary CI mutation workflow, including its failure mode.

Do not increase sample sizes beyond the source records.

## 9. Top ten next capability gaps

V1 should emit these as initial backlog candidates, but implementation is outside this slice unless separately approved:

1. **Usage telemetry gap** — token/context/tool-call counts are missing.
2. **Bundle query gap** — no deterministic task→bundle recommendation API.
3. **Agent topology gap** — no comparative experiment harness for agent combinations.
4. **MCP evaluation gap** — profiles exist without candidate discovery/benchmark evidence.
5. **Framework adoption gap** — no reusable dependency/framework scoring workflow.
6. **Skill lifecycle gap** — no promotion/deprecation/retirement evidence model.
7. **Negative interaction gap** — capability combinations do not encode conflicts/synergies explicitly.
8. **Context efficiency gap** — no measured context contribution/unused-context analysis.
9. **Cross-platform sandbox gap** — subprocess hardening evidence is stronger on Unix than Windows/mobile.
10. **External review capacity gap** — automated independent reviewers can be unavailable due account/subscription limits; fallback evidence paths need standardization.

## 10. Deliberately deferred installations

Do **not** add Rust/Go runtimes, brokers, graph databases, vector databases, orchestration frameworks, or new third-party Python packages in v1.

The repository's accepted architecture favors local-first minimal dependencies, in-memory/JSON evidence at this maturity, optional distributed workers only when measured need exists, and stateless/narrow capability boundaries.

Framework/plugin/MCP installation begins only after the dataset can record the reason, expected benefit, verification, and rollback evidence for the installation.

## 11. Validation and CI

Add dependency-free tests covering:

- valid capability profiles;
- invalid IDs/enums/authority classes;
- valid experiment records;
- unknown capability references;
- duplicate IDs;
- malformed JSON;
- invalid/null metric handling;
- deterministic analysis order;
- deterministic gap scores;
- no fabricated values for missing measurements;
- analyzer handles the six existing toolset records.

CI should invoke validation and analyzer smoke checks in the existing Python job.

## 12. Safety invariants

1. Learned data is advisory evidence only.
2. No dataset record grants execution authority.
3. No analyzer output automatically installs, activates, promotes, merges, or executes anything.
4. Unknown measurements remain unknown.
5. Safety regressions dominate bundle scoring and candidate promotion.
6. Repository evidence remains the source of truth; Engram may mirror but not override it.
7. Any future automatic installation or policy mutation requires a separate authority design and explicit approval.

## 13. Success criteria

V1 is complete when:

- capability and experiment JSONL schemas are enforced deterministically;
- current AutoDev capabilities are inventoryable in one compact dataset;
- the six existing toolset observations can be represented without information loss relevant to routing;
- a deterministic analyzer produces coverage, evidence scores, missing-measurement warnings, and ranked gaps;
- CI validates all datasets offline;
- no new third-party dependency or execution authority is introduced;
- full repository CI remains green.

## 14. Future larger-dataset path

After v1 accumulates a meaningful corpus, later analysis may use stratification, confidence intervals, pairwise synergy measures, ablation studies, and eventually statistical/ML routing models. Those methods are explicitly premature until the repository contains enough comparable experiment records.
