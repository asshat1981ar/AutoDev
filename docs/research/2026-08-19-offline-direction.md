# Offline objective/interaction direction

Offline mode should queue user intent and safe local review actions, not fabricate remote execution state. Queue entries need stable identity, creation time, target run/project, expected base state, and replay policy.

When connectivity returns, AutoDev compares expected versus current state. Compatible entries replay deterministically; stale entries enter reconciliation/review. Approval decisions must remain bound to the exact effect/capability scope they reviewed and must not be generalized to changed work.
