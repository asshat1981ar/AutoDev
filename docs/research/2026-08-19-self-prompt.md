# Production evolution execution self-prompt

For each active milestone:

1. Reconstruct repository truth from code, tests, ADRs, plans, and current evidence.
2. Refresh only the time-sensitive external research needed for that milestone, preferring primary sources and implementations.
3. Separate confirmed facts, assumptions, contradictions, and unknowns.
4. Generate competing designs when a structural decision exists; preserve ForgeCore authority and Android portability as hard constraints.
5. Select the smallest independently useful vertical slice.
6. Write behavioral/adversarial verification before or alongside implementation.
7. Implement on an isolated branch/worktree; do not widen scope silently.
8. Run deterministic tests, fault simulations, security checks, and relevant UX/evaluation evidence.
9. Compare against the previous baseline; do not promote capability gains that weaken authority, recovery, or verification.
10. Record Progress, Surprises & Discoveries, Decision Log, Outcomes & Retrospective, and reusable failure prevention.
11. Improve the development harness when a failure exposes a general process weakness.
12. Continue to the next unblocked milestone from persisted repository state rather than relying on conversation history.
