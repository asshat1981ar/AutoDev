# Production evolution ledger

## 2026-08-18

- Architecture C (Federated Harness Kernel) approved.
- Architecture specification and first ExecPlan implementation plan committed to `main`.

## 2026-08-19

- Created isolated `feat/execplan-control-plane` branch.
- Added `PLANS.md` durable planning contract.
- Added typed authority-free ExecPlan domain, public exports, and adversarial tests.
- Added focused plan-contract checker/test and GitHub Actions workflow.
- Recorded ADR-002 for the authority boundary.
- Refreshed external research: Harness Protocol v1 and Deep Agents harness profiles/subagent semantics.
- Added Harness Protocol compatibility addendum and next-milestone plan.
- Persisted research hypotheses, risks, Android-first principles, simulation scenarios, metrics, release gates, and program self-prompt.
- Ruling: use focused `check_execplan_contract.py` for this slice rather than modifying the large monolithic drift checker before executable verification. Cost if wrong: temporary duplicate harness-check entry point.
- Current gate: open PR and collect executable GitHub Actions evidence before declaring the slice complete or beginning Harness Asset implementation.
