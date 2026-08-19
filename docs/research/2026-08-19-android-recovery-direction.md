# Android recovery direction

Android process death should be treated as expected rather than exceptional. UI state must be reconstructable from durable server/KMP/run state. The app should never infer that an in-flight effect failed merely because its local process disappeared.

On reconnect/resume:

1. load durable plan/task/envelope state;
2. identify effects with uncertain acknowledgement;
3. reconcile repository/external state;
4. surface unresolved ambiguity to the user when it cannot be proven automatically;
5. resume only from a consistent checkpoint.

This is the mobile expression of the same reconciliation invariant introduced by the ExecPlan control plane.
