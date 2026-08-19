# ExecPlan implementation ruling

The implementation plan originally called for modifying the existing monolithic `scripts/check_harness_drift.py`. Because executable local verification is unavailable in the current tool environment and that file is large/high-coupling, this branch instead adds a focused stdlib-only `scripts/check_execplan_contract.py` plus CI and unittest coverage.

This preserves the required enforceability while reducing accidental drift in unrelated harness checks. After CI is green, integration into `check_harness_drift.py` can be evaluated as a small follow-up if repository maintainers prefer one entry point.

Cost if wrong: two harness-check entry points temporarily exist. The dedicated GitHub Actions workflow prevents the new contract from being silently skipped on this branch/PR.
