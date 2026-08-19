# Multi-agent parallelism direction

Parallel subagents should be permitted when task dependencies and effect scopes are demonstrably independent. Shared-file mutation, overlapping Git operations, or common external resources require serialization or explicit coordination.

The Simulation/Eval Lab should measure whether parallelism improves wall-clock completion without increasing merge conflicts, duplicated work, context overhead, verification failures, or intervention rate. Maximum concurrency should be a policy/resource parameter, not an autonomy objective by itself.
