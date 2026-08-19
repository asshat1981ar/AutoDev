# Reproducibility direction

A durable run should record enough configuration identity to explain and, where feasible, reproduce its decisions: repository commit/base state, plan/workflow version, model/profile/toolset/skill identities, policy/verifier versions, and relevant deterministic context parameters.

External nondeterminism must be acknowledged rather than hidden. Evaluation should prefer seeded/frozen fixtures and compare distributions or repeated outcomes when exact replay is impossible.
