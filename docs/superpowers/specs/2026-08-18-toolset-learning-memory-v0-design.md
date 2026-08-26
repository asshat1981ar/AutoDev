# Toolset Learning Memory v0 Design

## Goal

Create a durable, machine-readable record of which tools, skills, connectors, ordering strategies, and execution environments work well for different AutoDev engineering task classes.

## Boundary

This subsystem records observations and recommendations only. It grants no execution, approval, policy, merge, installation, routing, or promotion authority. Learned patterns are evidence for future planning, never authority.

## Storage

Repository-local records live in `memory/toolsets/patterns.jsonl`. One JSON object represents one reusable toolset pattern. The append-friendly JSONL format is intentionally simple for local-first operation, Git review, offline analysis, and later ingestion into `autodev-eval` or another evaluator.

External persistent memory may mirror the same conclusions for cross-session retrieval, but the repository dataset is the reproducible source used for analysis.

## Pattern fields

Required fields:

- `pattern_id`: stable lowercase slug.
- `task_class`: reusable task category.
- `context`: environment or problem setting where evidence was observed.
- `combination`: ordered list of tools, skills, connectors, or workflow stages.
- `result`: `high`, `medium`, `low`, or `failed`.
- `evidence`: concise list of observable supporting facts.
- `strengths`: reusable advantages.
- `failure_modes`: known limitations or negative effects.
- `when_to_use`: positive applicability rules.
- `when_not_to_use`: negative applicability rules.
- `confidence`: `high`, `medium`, or `low`.
- `sample_size`: positive integer.
- `last_validated`: ISO `YYYY-MM-DD` date.

Optional fields include `better_alternative`, `environment_constraints`, `failed_strategy`, `metrics`, and `notes`.

## Validation

`scripts/validate_toolset_memory.py` validates every JSONL record with Python standard library only. It fails on malformed JSON, missing required fields, duplicate pattern IDs, invalid enums, invalid dates, empty ordered combinations, non-positive sample sizes, or non-lowercase-slug IDs.

`tests/test_toolset_memory.py` covers valid seed data and representative invalid records.

## Seed evidence

v0 seeds six patterns observed during Self-Evaluation Factory v0:

1. TDD + GitHub Actions when local Rust execution is unavailable.
2. Security review + targeted descendant-process RED regressions + process-group repair.
3. Git history + frozen exact-SHA tasks + hidden verifiers + full-history smoke.
4. Compare-commits + overlap analysis + true merge-parent reconciliation.
5. Preserve-main lockfile + minimal Cargo resolution + `--locked` CI.
6. One-shot write-enabled CI workflows as a context-dependent fallback, including their permission and cleanup hazards.

## Learning loop

For substantial work:

1. Search existing patterns by task class/context.
2. Select the highest-confidence applicable combination.
3. Execute and measure.
4. Update an existing semantically equivalent pattern rather than duplicating it.
5. Increase sample size and confidence only when new evidence supports doing so.
6. Preserve contradictory evidence and environment constraints.

The primary optimization target is verified useful work per human intervention, risk, execution cost, and context cost.

## Non-goals

v0 does not implement learned automatic routing, autonomous promotion, database storage, embeddings, probabilistic scoring, or automatic mutation of the pattern dataset.