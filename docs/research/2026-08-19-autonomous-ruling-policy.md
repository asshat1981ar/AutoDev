# Autonomous ruling policy

When spec/plan leave a reversible implementation detail open, choose the smallest option preserving approved invariants, record the decision and cost-if-wrong, and continue. Do not interrupt the long-running program for ordinary naming/layout/internal API choices.

Escalate only for major architecture/product direction, irreversible/destructive/security-sensitive operations, shared-branch/release side effects, or evidence so contradictory that every path is guesswork.
