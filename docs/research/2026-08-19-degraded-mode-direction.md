# Degraded-mode direction

AutoDev should expose capability-aware degraded modes: review-only, planning-only, local lightweight execution, remote-companion execution, offline queued intent, etc. The user should know which guarantees/features are available in the current mode.

Degradation may reduce capability or defer work; it must not weaken policy/verification silently to keep a run moving.
