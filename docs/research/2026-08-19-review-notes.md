# Pre-CI review notes

Potential issues to verify in CI/review:

- Rustfmt may reflow compact constructor/transition statements in `exec_plan.rs`; `cargo fmt --check` will enforce canonical formatting.
- `PlanMilestone` fields are public in the first slice to keep fixture construction simple; later orchestration may need mutation methods to enforce per-milestone attempt budgets.
- `resume(true)` is intentionally only a first-slice lifecycle guard; production orchestration should replace boolean acknowledgement with typed reconciliation evidence.
- The focused drift checker is separate from the existing monolithic drift script; follow-up consolidation is optional after evidence.
- Research artifacts are non-executable and must not be mistaken for completed later milestones.
