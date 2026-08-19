# CI layering direction

Keep fast focused checks close to each domain while retaining full repository regression workflows. Focused workflows accelerate diagnosis; full CI catches cross-module drift.

Longer simulation/evaluation suites can run as separate gated/nightly/release workflows once they are stable, with promotion decisions consuming their artifacts rather than slowing every trivial documentation change.
