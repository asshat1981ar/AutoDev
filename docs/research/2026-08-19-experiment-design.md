# Experiment design direction

Start with controlled ablations before broad autonomous optimization:

1. Freeze task corpus, policy, and verifier recipe.
2. Establish baseline configuration tuple.
3. Vary one factor (profile, toolset, context strategy) where possible.
4. Run repeated deterministic or seeded scenarios plus fault injections.
5. Measure verified outcomes, safety/recovery gates, cost/context, and intervention.
6. Reject candidates failing any protected surface regardless of efficiency gain.
7. Promote only after held-out confirmation.

Later Bayesian/search optimization can operate over configuration space only after the measurement harness is trustworthy.
