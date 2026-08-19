# Android Workspace information architecture sketch

Reference navigation for later design/prototyping:

1. **Projects** — repositories and environment health.
2. **Runs** — durable objectives, progress, blocked/interrupted state.
3. **Run detail** — goal, current milestone, task graph, agents, decisions/discoveries.
4. **Review** — diffs, requested approvals, exact capability/effect scope.
5. **Evidence** — tests/build/static/security results and provenance.
6. **Recovery** — offline queue, stale-state reconciliation, retry/cancel/resume.
7. **Harness** — active profile, skills/plugins/MCPs, trust/provenance, diagnostics.

Default mobile flow should emphasize current outcome and required intervention. Detailed agent traces and harness internals should be progressively disclosed rather than occupy the primary dashboard.
