# Android-first production principles

These constraints guide later workspace/runtime milestones.

- Android is the reference interactive client, not a privileged execution boundary.
- Core plan/task/evidence contracts belong in portable Rust/KMP/server interfaces rather than Compose-only state.
- Process death and network loss are normal operating conditions; durable runs must resume from persisted typed state.
- Offline queued objectives require deterministic replay and visible reconciliation when remote state changed.
- No mandatory Docker, Bun, native PTY, or desktop daemon assumption for core Android workflows.
- Dangerous capabilities surface as exact, reviewable approval requests; mobile convenience cannot collapse capability scope.
- Diff, evidence, failure, and recovery state must be legible on a small screen before adding high-density dashboards.
