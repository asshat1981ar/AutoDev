# ExecPlan branch change summary

Production behavior added:

- serializable authority-free `ExecPlan` coordination state;
- validation for identity/budget/milestone/completion invariants;
- explicit start/block/interrupt/resume/cancel/complete transitions;
- finite replan consumption;
- checkpoint snapshots and decision/discovery history;
- focused Rust/Python verification workflow.

Repository/process added:

- living `PLANS.md` contract;
- ADR/architecture/harness docs;
- durable research/program artifacts constraining Harness Protocol compatibility, Android-first design, recovery, evaluation, and later production milestones.
