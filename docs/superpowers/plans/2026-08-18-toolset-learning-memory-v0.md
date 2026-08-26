# Toolset Learning Memory v0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a validated repository-local JSONL dataset of reusable evidence-backed tool/skill/connector workflow combinations.

**Architecture:** Keep learning memory outside ForgeCore authority. Store append-friendly JSONL under `memory/toolsets/`, validate with a dependency-free Python script, cover behavior with `unittest`, and seed only observations already supported by executed AutoDev work.

**Tech Stack:** JSONL, Python 3.10+, `unittest`, GitHub Actions existing Python matrix.

**Spec:** `docs/superpowers/specs/2026-08-18-toolset-learning-memory-v0-design.md`

## Global Constraints

- Memory records are advisory evidence only and cannot authorize execution or promotion.
- No secrets, credentials, private reasoning, or raw logs are stored.
- No new third-party dependencies.
- Existing semantically equivalent patterns are updated instead of duplicated.
- Validation must be deterministic and offline.

---

### Task 1: Validation Contract

**Files:**
- Create: `tests/test_toolset_memory.py`
- Create: `scripts/validate_toolset_memory.py`

**Interfaces:**
- Produces: `validate_record(record: dict) -> list[str]`
- Produces: `validate_file(path: pathlib.Path) -> list[str]`
- Produces CLI exit `0` for valid datasets and `1` for validation failures.

- [ ] Write tests for valid records, duplicate IDs, malformed JSON, invalid slug/date/enums, empty combination, and non-positive sample size.
- [ ] Run `python -m unittest tests.test_toolset_memory -v` and confirm RED because validator implementation is absent.
- [ ] Implement standard-library-only validation.
- [ ] Re-run the focused tests and confirm GREEN.
- [ ] Commit the validator slice.

### Task 2: Seed Dataset

**Files:**
- Create: `memory/toolsets/patterns.jsonl`
- Create: `memory/toolsets/README.md`

**Interfaces:**
- Consumes the validator contract from Task 1.
- Produces six valid `toolset-pattern-v1` records.

- [ ] Encode the six evidence-backed patterns from the approved design as one compact JSON object per line.
- [ ] Document search/update/append rules and field semantics.
- [ ] Run `python scripts/validate_toolset_memory.py memory/toolsets/patterns.jsonl` and confirm six valid records.
- [ ] Run `python -m unittest tests.test_toolset_memory -v`.
- [ ] Commit the dataset slice.

### Task 3: Repository Integration

**Files:**
- Modify: `README.md`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Existing Python CI invokes the validator and test module.

- [ ] Add a short README section explaining the toolset-learning dataset and advisory-only boundary.
- [ ] Add deterministic validator/test commands to existing Python CI rather than creating a new workflow.
- [ ] Run/verify the complete existing CI on the exact branch head.
- [ ] Confirm no temporary write-enabled workflows exist.
- [ ] Open a dependent PR targeting `feat/self-evaluation-factory-v0` so PR #19's verified head is unchanged.
