# AMCX-1 / AutoDev Neutral Contract Registry & Reference Implementation

## Overview
This repository establishes the **AMCX-1 (Agent Memory and Collaboration Exchange)** neutral contract registry, architecture decision records (ADRs), 18-domain trust boundary map (§5), orthogonal state dimensions, and verification test suite.

## Architecture Invariants
1. **AMX is the sole canonical portable-memory contract.**
2. **ECM never owns or mutates AMX causal heads.**
3. **`ECMMemoryBinding` is noncanonical, digest-linked, and rebuildable.**
4. **ForgeCore alone owns capability authorization and effect execution.**
5. **Git-backed Neutral Contract Registry alone publishes and activates schemas.**
6. **Zero raw secrets in memory records or ContextViews.**

## Directory Structure
- `docs/ADRs/`: Architecture Decision Records (`ADR-001` to `ADR-004`).
- `docs/specs/`: Normative AMCX-1 and reconciliation specifications.
- `registry/v1/`: Machine-readable JSON Schemas (AMX, ECM, AMCX).
- `registry/manifest.json`: Cryptographic SHA-256 registry manifest.
- `src/`: Python reference implementation (ContractRegistry, TrustBoundaries, OrthogonalStateTracker, ECMMemoryBinding, BaseAdapter).
- `tests/`: Comprehensive verification test suite enforcing ADR-004 §5 ownership, fail-closed security, and schema validation.
- `scripts/`: Verification runner and export utilities.

## Verification
Run the verification suite:
```bash
python3 -m unittest discover -s tests -p "test_*.py" -v
```


## Durable development and release verification (v1.1)

For long-running agentic development, follow `docs/specs/2026-08-20-autodev-durable-development-harness-v1.1.md` and the v1.1 Mistral Vibe instructions. Architectural implementation belongs in `.worktrees/<task-id>/` (or stronger native isolation), with durable recovery state under `.autodev/checkpoints/`.

Run the portable suite with:

```bash
bash scripts/run_verification.sh
```

Production packages must pass that command after clean extraction; use `scripts/build_release.py` to build and verify a release.
