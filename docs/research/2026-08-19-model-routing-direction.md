# Model-routing direction

AutoDev already has model routing and toolset-learning primitives. Future harness profiles should complement rather than replace them.

Routing chooses a candidate model/profile/toolset from task requirements and evidence. A harness profile then declaratively shapes the untrusted prompt/tool/subagent surface for that candidate. ForgeCore policy remains invariant below both layers.

Evaluation should compare `(model, profile, toolset, context strategy)` configurations as explicit versioned tuples so gains are attributable and rollback is possible.
