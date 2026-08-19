# ExecPlan harness adoption report

## Adopted

- Living repository `PLANS.md` contract for architectural and multi-hour work.
- Typed ForgeCore coordination state with finite budgets, milestones, references, decisions, discoveries, checkpoints, interruption, reconciliation, and cancellation.
- Explicit separation between coordination state and trusted execution authority.
- Focused CI gate for the new control-plane contract.

## Deferred

- Direct coupling between ExecPlan and TaskGraph persistence.
- Durable external storage adapter.
- Android plan timeline and recovery UI.
- Harness Asset Protocol/import adapters.

These remain separate milestones to keep the first slice reviewable and avoid creating a second execution path.
