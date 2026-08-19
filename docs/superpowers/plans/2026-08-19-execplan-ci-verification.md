# ExecPlan CI Verification

## Progress

- [x] Repository `PLANS.md` contract created.
- [x] Typed authority-free `ExecPlan` state added.
- [x] Recovery/reconciliation and bounded-replan invariants added.
- [x] Focused Rust contract tests added.
- [x] Focused Python planning-contract test and checker added.
- [x] Dedicated GitHub Actions verification workflow added.
- [ ] GitHub Actions evidence collected and reviewed.
- [ ] Any CI findings corrected and re-verified.
- [ ] Full branch review completed.

## Surprises & Discoveries

- The execution environment cannot clone GitHub over the local container network, so executable verification is delegated to GitHub Actions on the isolated feature branch.
- Harness Protocol v1 overlaps strongly with the planned Harness Asset Protocol and should be treated as an interoperability target before freezing AutoDev's external schema.
- Current Deep Agents exposes provider/model harness profiles plus explicit subagent skills, tools, middleware, permissions, and response formats; these are useful adapter/profile semantics but not a substitute for ForgeCore authority.

## Decision Log

- Keep `ExecPlan` authority-free and reference existing task/run/envelope identities rather than embedding execution grants.
- Require explicit reconciliation before resuming interrupted effectful work.
- Add a focused CI workflow so this slice can obtain executable evidence despite the local network restriction.
- Treat Harness Protocol compatibility as a required Milestone 2 research gate.

## Outcomes & Retrospective

Pending CI evidence and final review.
