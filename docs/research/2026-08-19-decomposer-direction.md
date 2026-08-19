# Decomposer direction

Decomposition should produce tasks small enough for independent verification and safe scheduling while preserving vertical value. Dependency edges should represent real ordering/data/effect constraints rather than arbitrary sequencing.

Evaluation can measure task churn, dependency mistakes, conflicting parallel edits, and replan frequency to improve decomposition strategies over time.
