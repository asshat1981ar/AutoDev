# Durable scheduling direction

Task scheduling should operate on explicit dependencies, readiness, priority, resource/capability availability, and bounded concurrency. Parallelism is valuable only for tasks whose effects/context do not create unsafe races.

The scheduler should persist dispatch decisions and agent/profile identity for reproducibility. Cancellation and reprioritization are durable state transitions. A resumed scheduler recomputes readiness from trusted current state rather than assuming pre-crash workers are still active.
