# Quality-gate philosophy

Gates should correspond to concrete failure classes: formatting/static correctness, compilation, unit/integration behavior, authority/security boundaries, reproducibility, Android packaging, recovery, and user-facing usability/accessibility.

Avoid accumulating ceremonial gates with no known defect class. When a new failure occurs, either strengthen an existing gate or add a narrowly enforceable new one and document the failure it prevents.
