# Multi-agent incentive/adversarial direction

Treat planner, implementer, verifier, and evaluator roles as having potentially misaligned incentives: an implementer benefits from easy verification; a planner may under-specify difficult evidence; a candidate configuration may overfit its evaluator.

Architectural response: separate roles/evidence, hide/protect some verification where useful, make promotion policy independent, and evaluate collusion-like failure modes (e.g., agent writes tests that merely confirm its implementation). This is a practical incentive-design lens, not a requirement for complex game-theoretic machinery.
