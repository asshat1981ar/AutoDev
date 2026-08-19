# ExecPlan control-plane review checklist

- [ ] `ExecPlan` contains no trusted authorization material.
- [ ] No ExecPlan method directly executes filesystem/process/Git/network effects.
- [ ] Empty identity/goal and invalid budgets fail validation.
- [ ] Duplicate milestone identities fail validation.
- [ ] Completed plan cannot contain incomplete milestones.
- [ ] Replan budget cannot be exceeded.
- [ ] Interrupted run cannot resume without reconciliation acknowledgement.
- [ ] Checkpoint serialization preserves coordination references/history.
- [ ] `PLANS.md` required living sections are enforced.
- [ ] Focused Rust/Python CI passes.
- [ ] Existing repository CI remains green.
- [ ] Documentation reflects the authority boundary and later Harness Protocol compatibility constraint.
