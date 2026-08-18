# Toolset Learning Memory

This directory stores evidence-backed observations about combinations of tools, skills, connectors, workflow order, and environment constraints that materially affected AutoDev engineering outcomes.

## Source of truth

`patterns.jsonl` is the repository-local, reviewable dataset. Each line is one `toolset-pattern-v1` JSON object. External memory systems may mirror these conclusions for cross-session retrieval, but repository analysis should use this file.

## Advisory-only boundary

These records are planning evidence, not authority. A pattern cannot mint approvals, execute actions, modify policy, install tools, merge code, activate skills, or promote a self-improvement candidate.

## Update rules

Before adding a pattern:

1. Search for a semantically equivalent task class and combination.
2. Update the existing pattern when the new observation is materially the same.
3. Increase `sample_size` only for an additional observed case.
4. Increase confidence only when evidence quality supports it.
5. Preserve contradictory evidence in `failure_modes`, `notes`, or environment constraints.
6. Never store secrets, credentials, private reasoning, or large raw logs.

The main unit of learning is:

`task class × ordered toolset combination × environment → observed outcome`

## Validation

Run:

```bash
python scripts/validate_toolset_memory.py memory/toolsets/patterns.jsonl
python -m unittest tests.test_toolset_memory -v
```

The validator is standard-library only and checks schema version, required fields, stable IDs, enums, dates, list shape, positive sample sizes, malformed JSON, and duplicate IDs.

## Intended use

Future planning agents should search these records before substantial work, prefer the highest-confidence applicable pattern, adapt it to the current environment, execute, measure, and then update the evidence. Later versions may feed the dataset into `autodev-eval` for empirical comparison, but v0 performs no automatic routing or promotion.