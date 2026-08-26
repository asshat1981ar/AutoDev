# AutoDev Durable Development Harness v1.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reconcile the AMCX-1 release with portable verification, isolated worktree execution, mandatory process-skill orchestration, durable recovery checkpoints, and clean-extraction release gating.

**Architecture:** Keep AMX/ECM protocol semantics unchanged. Add development-harness policy and recovery artifacts around the existing implementation, while making verification location-independent and release validation self-checking.

**Tech Stack:** Python 3 standard library, Bash, Git worktrees, Markdown/JSON.

**Spec:** `docs/specs/2026-08-20-autodev-durable-development-harness-v1.1.md`

## Global Constraints
- Preserve Round 2 and Round 2.1 evidence unchanged.
- Skills/tools never authorize effects.
- Worktree isolation is mandatory unless a stronger native isolation mechanism exists.
- Verification must pass from a clean extraction.
- Do not persist secrets or hidden reasoning in checkpoints.

---

### Task 1: Portable verification
**Files:** modify `tests/test_*.py`; modify `scripts/run_verification.sh`.
- [x] Replace build-specific absolute paths with repository-relative resolution.
- [x] Export repository `src` through `PYTHONPATH` in the verification runner.
- [x] Derive verification totals from the discovered suite.
- [ ] Run the suite and record evidence.

### Task 2: Durable development harness
**Files:** create `scripts/autodev_checkpoint.py`; create `.gitignore`; test `tests/test_autodev_checkpoint.py`.
- [ ] Write failing checkpoint contract tests.
- [ ] Implement atomic checkpoint initialization/update/validation.
- [ ] Verify secret-like fields and absolute external paths are rejected.
- [ ] Run tests and record evidence.

### Task 3: Vibe and operator instructions
**Files:** create `docs/specs/AutoDev-Mistral-Vibe-Project-Instructions-v1.1.md`; modify `README.md`.
- [ ] Incorporate mandatory process-skill batching.
- [ ] Require `.worktrees/<task-id>/` isolation and durable checkpoint recovery.
- [ ] Preserve least-authority and AMCX ownership rules.
- [ ] Document clean-extraction release verification.

### Task 4: Release gate
**Files:** create `scripts/build_release.py`; test `tests/test_release_builder.py`.
- [ ] Write failing tests for deterministic exclusion and clean extraction.
- [ ] Implement packaging, cache exclusion, extraction, verification, and SHA-256 generation.
- [ ] Run the builder from the reconciled tree.
- [ ] Verify the resulting ZIP again from a second independent extraction.

### Task 5: Reconciliation evidence
**Files:** create `docs/specs/2026-08-20-work-chat-drive-reconciliation-v1.1.md`.
- [ ] Record retained, superseded, added, and corrected requirements.
- [ ] Record verification commands and results.
- [ ] Confirm Round 2/2.1 evidence hashes remain unchanged.
- [ ] Produce final release and detached SHA-256 manifest.
