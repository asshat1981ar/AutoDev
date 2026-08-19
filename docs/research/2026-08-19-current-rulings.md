# Current program rulings

1. **Federated Harness Kernel over framework assimilation.** Keep ForgeCore as authority; external harnesses integrate through adapters. Cost if wrong: more adapter engineering than directly adopting one framework.
2. **Typed ExecPlan + living prose.** Do not use transcript/PLANS.md prose as canonical lifecycle state. Cost if wrong: additional typed coordination model/persistence work.
3. **Focused ExecPlan drift checker for first slice.** Avoid editing the large monolithic checker before executable verification. Cost if wrong: temporary duplicate harness-check entry point.
4. **Harness Protocol compatibility before custom external schema.** Cost if wrong: effort spent mapping to a young standard that may evolve; adapters/versioning contain the risk.
5. **Android as primary control surface, not universal execution host.** Cost if wrong: remote-companion complexity, but avoids impossible mobile runtime assumptions.
