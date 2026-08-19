# PR review order

1. Review `crates/forge-core/src/exec_plan.rs` and `tests/exec_plan.rs` for correctness/authority boundary.
2. Review `lib.rs` export-only change.
3. Review focused CI/checker/test.
4. Review PLANS.md + ADR/architecture docs for semantic agreement.
5. Treat remaining `docs/research/` as future design evidence; flag contradictions with the approved spec but do not confuse them with implemented runtime behavior.
