# Checkpoint direction

A checkpoint marks a recoverable coordination point and references the repository/external state evidence needed to establish it. It should be created after independently useful verified milestones and before risky transitions where practical.

Checkpoint identity belongs in durable run provenance. A checkpoint is not necessarily a Git commit, though Git commits/checkpoints can be referenced as part of repository state evidence.
