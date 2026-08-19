# ExecPlan architecture

`ExecPlan` sits above AutoDev's existing durable task/envelope machinery as an authority-free coordination layer.

```text
PLANS.md / ExecPlan
        |
        | references
        v
TaskGraph / ExecutionEnvelope
        |
        v
Policy + AuthorizationGrant
        |
        v
ForgeCore effects
        |
        v
Evidence + independent verification
```

A plan can remember goals, milestones, decisions, discoveries, budgets, checkpoints, and object identities. It cannot turn those references into permission. Interrupted effectful work must reconcile external state before resuming to prevent optimistic replay.
