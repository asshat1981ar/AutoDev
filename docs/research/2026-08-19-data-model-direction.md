# Durable data-model direction

Near-term typed state should remain small and serializable in ForgeCore. Persistence technology is deliberately deferred.

Identity relationships should be references rather than nested copies:

```text
ExecPlan
  -> milestone ids
  -> task ids
  -> run ids
  -> envelope ids
  -> checkpoint ids
  -> evidence ids
```

This avoids making the plan a shadow execution database. Later persistence can add event/history storage while keeping execution envelopes and evidence independently auditable.
