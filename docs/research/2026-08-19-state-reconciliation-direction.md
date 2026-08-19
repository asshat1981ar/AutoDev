# State reconciliation direction

Reconciliation compares the durable expected state at the last safe checkpoint with observed current state. It should classify each uncertain effect as `not_applied`, `applied_as_expected`, `applied_differently`, or `unknown` where possible.

Only `not_applied` is automatically retryable; `applied_as_expected` advances without replay; `applied_differently` triggers replan/review; `unknown` remains blocked unless a safe idempotent strategy exists. This classification should eventually replace boolean reconciliation acknowledgement.
