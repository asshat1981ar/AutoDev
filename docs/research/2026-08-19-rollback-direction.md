# Rollback direction

Rollback is a new effectful operation with its own capability, risk, authorization, and verification—not an implicit undo button. The system should prefer reversible checkpoints/commits and show exactly what rollback will change.

Automatic rollback may be appropriate only when policy explicitly permits it and the rollback effect is independently verifiable. Otherwise failure preserves evidence and requests review rather than attempting destructive recovery silently.
