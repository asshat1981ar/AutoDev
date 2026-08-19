# Product state language

Use consistent user-facing terms derived from typed state:

- Planned — ready but not executing.
- Running — active coordinated work.
- Needs approval — trusted user decision required.
- Interrupted — work stopped; some effect state may need reconciliation.
- Recovering — AutoDev is reconciling/checking state.
- Verifying — implementation exists but completion evidence is pending.
- Verified — required evidence passed.
- Failed — bounded work ended unsuccessfully with diagnostics.
- Cancelled — future work stopped and uncertain effects reconciled where required.

Avoid ambiguous labels like simply "done" when verification is incomplete.
