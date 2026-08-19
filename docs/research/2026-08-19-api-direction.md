# Control-plane API direction

Future APIs should expose typed resources/events for projects, durable plans/runs, milestones/tasks, approvals, evidence, harness configuration, and recovery/reconciliation.

Clients should submit intent and review decisions, not raw trusted grants. Server/kernel layers translate validated requests into trusted internal authorization structures. Event streaming should be resumable by stable cursor/identity so mobile reconnect does not require reconstructing state from transient logs.
