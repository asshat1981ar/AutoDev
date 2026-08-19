# ExecPlan control plane

AutoDev uses repository-level `PLANS.md` plus typed `forge_core::ExecPlan` state for durable, interruption-tolerant development objectives.

The prose contract is for humans and agentic workers. Typed state is for lifecycle correctness, finite replan budgets, milestone state, checkpoint snapshots, and references to existing task/run/envelope identities.

The control plane grants no authority. Execution remains behind ForgeCore policy, workspace confinement, and `AuthorizationGrant`; completion remains evidence-gated.

For the architectural decision, see `docs/adr/ADR-002-execplan-authority-boundary.md`. For implementation details, see `docs/superpowers/plans/2026-08-18-execplan-control-plane.md`.
