# Durable run timeline direction

The timeline should be derived from typed transitions/evidence and include milestone start/completion, task assignment, approval blocks, effect execution, verifier results, checkpoints, interruptions, reconciliation, replans, and cancellation/failure.

Agent narrative messages may be attached as context but should not be the authoritative timeline. This distinction lets Android reconstruct a trustworthy run history after process death or cross-device resume.
