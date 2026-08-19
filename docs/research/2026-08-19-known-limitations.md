# Known limitations of Milestone 1 implementation

- `ExecPlan` is currently an in-memory serializable domain object; no production persistence adapter is added.
- Reconciliation is represented by caller acknowledgement, not typed effect-specific evidence yet.
- Per-milestone attempt budget is modeled but not yet enforced by orchestration methods.
- ExecPlan is not yet integrated into TaskGraph/VerifiedOrchestrator runtime flow.
- Android/KMP has no ExecPlan UI/API model yet.
- Focused contract checking is separate from the monolithic harness drift script.

These are intentional boundaries for the first slice, not claims of production completeness.
