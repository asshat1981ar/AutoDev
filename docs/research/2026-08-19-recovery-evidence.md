# Recovery evidence direction

The first ExecPlan API uses a boolean reconciliation acknowledgement to establish the lifecycle invariant. Production durable orchestration should strengthen this into typed reconciliation evidence.

Candidate evidence includes repository HEAD/status/diff observations, filesystem artifact hashes, process/job identifiers and terminal state, remote request idempotency keys/status, and tool-specific receipts. The required evidence depends on the interrupted effect type.

A future transition should therefore resemble `resume(ReconciliationEvidence)` rather than treating a caller-provided boolean as proof.
