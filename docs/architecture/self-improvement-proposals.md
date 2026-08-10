# Controlled Self-Improvement Proposals

**Date:** 2026-08-09
**Author:** Principal systems architect
**Scope:** Analysis of the completed AutoDev (ForgeCore) system, with improvement
proposals. **No proposals are applied by this document.**

## Purpose

This document analyzes the completed AutoDev system for recurring weaknesses and
turns them into **improvement proposals**. Each proposal follows a fixed structure:

> **Problem → Evidence → Proposed change → Alternatives → Risk → Expected benefit →
> Verification strategy**

Self-improvement is **controlled**, not autonomous. It follows the pipeline:

```text
Proposal → Evidence → Approval → Implementation → Verification → Checkpoint
```

## Hard safety rule (non-negotiable)

AutoDev must **never silently modify**:

- **security policies**
- **execution permissions**
- **trust boundaries**
- **model-provider credentials**
- **approval requirements**

Every proposal below that touches any of these is **explicit, human-approved, and
audited** — never self-applied. Any change to the above requires a human decision
recorded in evidence, and the change is gated behind the `Approval` stage of the
pipeline. No proposal in this document authorizes silent mutation of these.

## Self-improvement pipeline (definition)

1. **Proposal** — a structured proposal (this document, or a new one).
2. **Evidence** — the backing observations (tests, logs, code inspection).
3. **Approval** — a human (or policy-authorized) decision; required for anything
   touching the safety rule above.
4. **Implementation** — a code change, behind the approved scope.
5. **Verification** — CI-equivalent checks (fmt/build/test/clippy) + a targeted test.
6. **Checkpoint** — record the change and its evidence (ADR + test) so it is
   recoverable and auditable.

---

## Observed weaknesses (summary)

| # | Category | Weakness | Location |
| --- | --- | --- | --- |
| P1 | Policy friction | `High` and `Critical` risk are treated identically; critical is only special-cased by capability | `policy.rs` |
| P2 | Repair loops | Replan is declared but not implemented; a permanently failing task can be re-queued | `orchestrator.rs` |
| P3 | Verification failure | The verification fabric was structurally broken (did not compile) | `verification.rs` |
| P4 | Model weakness | Structured-output validation ignores the agent's risk ceiling / capability fit | `runtime.rs` |
| P5 | Tool failure | Git "forbidden" guardrail is a literal string allowlist (false negatives) | `git.rs` |
| P6 | Policy friction | Human approval is attestation-by-flag with no audit/signature | `git.rs`, `runtime.rs` |
---

## P1. Distinguish High vs Critical risk in policy

- **Problem:** The policy layer treats `High` and `Critical` risk identically
  (both `RequireApproval`), and `Critical` is only special-cased via the
  `approval:critical` capability in `validate_action`. This means a critical
  action is not actually gated more strictly than a high one by the policy
  decision itself.
- **Evidence:** `policy.rs::evaluate_policy` maps `Medium | High | Critical` to
  `RequireApproval` with no distinction. `git.rs` and `runtime.rs` re-implement
  the same mapping, so the semantics live in three places.
- **Proposed change:** Introduce an explicit `ApprovalTier` (or a `Critical` flag)
  so `Critical` requires a *stricter* approval path than `High` (e.g. a distinct
  `critical_approval` requirement), and centralize the risk→decision mapping in
  one function used by `policy`, `runtime`, and `orchestrator`.
- **Alternatives:** (a) leave as-is (accept that critical ≈ high); (b) deny
  `Critical` outright unless `approval:critical` is held.
- **Risk:** changing the mapping could require more approvals (product friction);
  mitigated by keeping the default conservative.
- **Expected benefit:** a single, auditable risk→approval policy; critical
  actions get the stricter gate the protocol intends.
- **Verification strategy:** unit tests asserting `Critical` yields a stricter
  decision than `High`; a table-driven test over all `RiskLevel`×`capabilities`.

## P2. Implement the Replan phase with a repair cap

- **Problem:** The orchestrator declares `REPLAN` and `REPAIR`, but `Repairer`
  retries solely on `task.retries < 2`; there is no overall loop backstop and no
  `REPLAN` implementation, so a permanently failing plan can spin or stall.
- **Evidence:** `orchestrator.rs` has `Phase::Replan` but `advance()` never
  returns it; `default_repairer` caps at 2 retries at the *task* level only.
- **Proposed change:** Add a plan-level retry counter and implement `REPLAN` as a
  bounded re-plan of the root (reset ready tasks, increment plan attempts, stop
  after N). Refuse to re-queue a task whose cumulative retries exceed a hard cap.
- **Alternatives:** (a) leave Replan as a stub; (b) fail the whole graph on first
  non-recoverable task.
- **Risk:** a bounded replan could loop if the plan generator is deterministic
  and broken; mitigated by the hard cap and by requiring manual intervention after
  the cap.
- **Expected benefit:** the loop always terminates; recoverability improves
  (replan is checkpointed).
- **Verification strategy:** a loop test that reaches `REPLAN`, then a test that a
  permanently failing plan terminates (not infinite) and is marked `BLOCKED`.

## P3. Verification fabric must compile and be green

- **Problem:** The verification fabric (`verification.rs`) was structurally
  broken — `default_fabric`/`verdict_from_report` were inserted inside the
  `VerificationReport` struct, so the crate did not compile. This is a concrete
  tool/verification failure.
- **Evidence:** `cargo build` failed with "functions are not allowed in struct
  definitions" / "expected item, found `}`". **Fixed in this pass** (the module now
  compiles; 6 fabric tests pass).
- **Proposed change:** Keep the fabric structurally sound and add a
  `verification.rs` integration test wired into the orchestrator's VERIFY phase
  (via `verdict_from_report`) so fabric breakage fails CI.
- **Alternatives:** (a) drop the fabric; (b) leave it unintegrated.
- **Risk:** low; the fix is already applied. The residual risk is future
  structural regressions, covered by CI.
- **Expected benefit:** verification is independently runnable and green; the
  orchestrator can consume `VerificationReport` verdicts.
- **Verification strategy:** `cargo build`/`test`/`clippy` green; an integration
  test proving `verdict_from_report` maps a `Pass`/`Fail` report into the
  orchestrator verdict.

## P4. Enforce the agent's risk ceiling at output validation

- **Problem:** The runtime validates structured model output into an `AgentAction`
  but does not check the agent profile's risk ceiling or capability fit before
  producing the action — a model can emit a high-risk action for a low-ceiling
  agent, reaching policy friction instead of being precluded.
- **Evidence:** `runtime.rs::validate_output` parses action/risk/payload but never
  calls `AgentProfile::may`; the profile's ceiling is only consulted later (and
  `submit_to_policy` duplicates the risk mapping).
- **Proposed change:** In `validate_output`, reject (or downgrade to a
  `Blocked`/validation error) actions whose risk exceeds the profile risk ceiling
  or whose capability the agent lacks — before producing an `AgentAction`.
- **Alternatives:** (a) leave enforcement to policy only; (b) clamp the risk to
  the ceiling.
- **Risk:** over-strict validation could reject a legitimate model suggestion;
  mitigated by returning a clear `InvalidOutput`/`PolicyDenied` so the model can
  re-plan.
- **Expected benefit:** the agent never *produces* an action it cannot execute;
  less policy friction; clearer separation of "what the agent may propose".
- **Verification strategy:** a runtime test where a low-ceiling agent's model
  emits a high-risk action → rejected at `validate_output` (no executor call).
---

## P5. Replace the literal-string git-forbidden allowlist

- **Problem:** `git::forbidden_command` matches exact literal strings (e.g.
  `"push --force"`, `"push -f"`). Equivalent invocations must each be enumerated,
  so a missed spelling is a false negative and the list is brittle to maintain.
- **Evidence:** `git.rs::forbidden_command` is a `matches!` over ~a dozen literal
  strings; `rollback` refuses only those exact commands.
- **Proposed change:** Classify the *operation class* (force-push, delete-remote,
  credential, history-rewrite) and reject any command whose normalized first
  subcommand + key flags fall in a forbidden class, rather than matching full
  strings. Keep the existing explicit list as the baseline.
- **Alternatives:** (a) keep the string list (simple, but fragile); (b) allow only
  an explicit allowlist of safe git subcommands and reject everything else.
- **Risk:** classification logic could over- or under-match; mitigated by keeping
  the current explicit cases as tests.
- **Expected benefit:** fewer false negatives around dangerous git operations;
  easier to extend.
- **Verification strategy:** table-driven tests over the forbidden classes,
  including alias spellings (`-f` vs `--force`), plus a test that a *safe* command
  (`checkout`) is not blocked.

## P6. Audit human approval (evidence-backed, not attestation-by-flag)

- **Problem:** `require_approval` accepts `payload.approved == true` (or the
  `approval:critical` capability) with no signature, actor, or audit record. An
  approval is a bare boolean flag, so it is not verifiable evidence.
- **Evidence:** `git.rs::require_approval` reads `payload.approved`; the runtime's
  `submit_to_policy` returns `RequireApproval` but no approval record is written.
- **Proposed change:** Make approval an explicit, recorded decision: an
  `Approval` record (approver, timestamp, reason, scope) stored in evidence and
  required (not just a flag) for `RequireApproval`/destructive operations.
- **Alternatives:** (a) keep the flag (simple, unauditable); (b) require a separate
  approval action type.
- **Risk:** adding an approval record is a small protocol change; must not silently
  raise or lower the approval bar (safety rule).
- **Expected benefit:** approvals are auditable and tamper-evident; the "approval
  required" property is enforced structurally.
- **Verification strategy:** a test that a destructive git action without an
  `Approval` record is refused, and one with a recorded `Approval` proceeds; the
  approval is reconstructable from evidence.
- **Safety check:** this **changes approval requirements** ⇒ requires human
  approval; it is proposed here, not self-applied.

## P7. Single risk→decision mapping (de-duplicate policy drift)

- **Problem:** The risk→decision mapping is duplicated across `policy.rs`,
  `runtime.rs::submit_to_policy`, and `orchestrator`/`agent.rs` (`AgentProfile::may`
  uses `risk_le`). Three implementations invite drift.
- **Evidence:** `runtime.rs` and `policy.rs` both map `Medium/High/Critical →
  RequireApproval`; `agent.rs` has its own `rank()` for ceilings.
- **Proposed change:** Centralize the mapping in `policy.rs` (e.g. a
  `decision_for(risk, capabilities)`) and have `runtime.rs` and `agent.rs` call it,
  so there is one source of truth.
- **Alternatives:** (a) leave as-is; (b) introduce a policy DSL.
- **Risk:** low; a refactor without behavior change (guarded by existing tests).
- **Expected benefit:** no silent drift in authorization semantics; simpler to audit.
- **Verification strategy:** existing tests remain green; add one test asserting
  `runtime::submit_to_policy` and `policy::evaluate_policy` agree for all risks.

---

## Prioritization & sequencing

| Priority | Proposal | Notes |
| --- | --- | --- |
| 1 | P3 Verification fabric green | Already fixed; keep CI-gated |
| 2 | P2 Replan + repair cap | Correctness/termination |
| 3 | P4 Risk ceiling at validation | Agent correctness + less friction |
| 4 | P7 Single risk mapping | Refactor, low risk |
| 5 | P5 Git forbidden classes | Security hardening |
| 6 | P1 High vs Critical | Policy precision |
| 7 | P6 Audited approval | **Requires human approval** (safety rule) |

## Recommendation

Adopt P2–P5 and P7 as implementation proposals (all within existing approval
requirements). Treat **P6** as a **policy change requiring explicit human approval**
per the safety rule, and **P1** as a product-policy decision to be made deliberately
(since it changes the effective approval bar). No proposal here is applied silently:
each proceeds only through
`Proposal → Evidence → Approval → Implementation → Verification → Checkpoint`.
| P7 | Agent weakness | Agent profiles duplicate the risk→decision mapping (drift risk) | `agent.rs`, `runtime.rs` |