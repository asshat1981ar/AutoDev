# Adversarial Sub-Agent Development System Design

Status: proposed architecture for review before implementation planning.

## 1. Purpose

AutoDev needs a development workflow that uses multiple isolated agents without allowing one model, one review source, or one context window to become the sole authority for correctness. The system described here turns software changes and external review findings into falsifiable invariants, assigns independent agents to investigate, attack, implement, review, and verify those invariants, and records the resulting evidence in a durable ledger.

The primary objective is not maximum code throughput. It is to minimize surviving defects per accepted change while preserving AutoDev's existing authority boundaries, especially ForgeCore's fail-closed execution and authorization model.

The initial operating target is the current AutoDev pull-request remediation queue, including security-sensitive ForgeCore changes, connector control-plane work, policy/specification work, and advisory development tooling. The architecture is deliberately general enough to become a reusable repository development harness after the current PR queue is stabilized.

## 2. Scope

This design defines:

- an orchestration controller for routing findings and tasks;
- isolated sub-agent roles and prompt contracts;
- adversarial prompting based on falsification rather than confirmation;
- risk-tiered development cells;
- TDD and verification gates;
- concurrency and worktree isolation rules;
- external-review ingestion and adjudication;
- durable finding and execution ledgers;
- bounded repair loops and breaker behavior;
- PR-level merge-readiness evidence requirements;
- an initial routing model for PRs #29, #42, #41, #34, #38, #39, and #40.

This design does not:

- merge pull requests automatically;
- publish packages or releases;
- perform live connector mutation;
- grant execution or approval authority outside ForgeCore;
- replace CodeRabbit, CI, or human review;
- allow review bots to directly execute commands or modify repositories;
- define a distributed scheduler or long-running worker service.

## 3. Architectural principles

### 3.1 Falsification before acceptance

Review agents must attempt to construct counterexamples that violate an invariant. A reviewer may recommend acceptance only after systematically failing to produce a source-supported counterexample within its assigned scope.

### 3.2 Separate roles by responsibility

The agent that implements a change must not be the decisive reviewer that accepts it. Investigation, adversarial analysis, test design, implementation, review, and final verification are separate roles whenever the risk tier requires them.

### 3.3 Evidence before state transitions

A finding or task may move to a stronger completion state only when the evidence required for that transition exists. Agent confidence, prose summaries, stale CI, and previous test runs are not evidence for the current head.

### 3.4 External reviews are hypotheses

CodeRabbit and other external findings are treated as claims to verify against the current source. They are neither ignored nor blindly implemented.

### 3.5 Fail closed on authority boundaries

Security-sensitive adapters and public entry points must validate their own authority assumptions rather than relying exclusively on expected callers or dispatch paths. No remediation may weaken sandboxing, approval gates, capability checks, or policy enforcement to make a test pass.

### 3.6 Parallelize only independent domains

Read-only analysis of independent PRs or subsystems may run concurrently. Stages that depend on one another, or agents that would edit the same branch/worktree, remain sequential.

### 3.7 Minimize change surface

A remediation task should produce the smallest reviewable change that restores the violated invariant. Unrelated refactors and speculative infrastructure are excluded unless required by the binding specification.

## 4. System components

### 4.1 Orchestration Controller

The Orchestration Controller owns coordination state and routing, not product authority.

Responsibilities:

1. Establish the current repository, PR, branch, and head SHA before dispatch.
2. Ingest CodeRabbit, CI, human, and local findings into a normalized record.
3. Assign a risk tier.
4. Build self-contained agent briefs.
5. Enforce worktree and branch isolation.
6. Prevent overlapping write scopes among concurrent agents.
7. Maintain the durable finding ledger and execution ledger.
8. Detect stale findings when the current head no longer matches the reviewed code.
9. Apply repair-loop limits and escalation rules.
10. Require independent verification before marking work complete.
11. Produce merge-readiness evidence without performing the merge.

The controller must not treat an implementer's report that tests passed as proof. It must obtain fresh verifier evidence.

### 4.2 Context Scout

The Context Scout is read-only. It builds the smallest reliable context package needed for the task.

Required output:

- current head SHA;
- finding or requirement;
- affected source and tests;
- direct callers and public exports;
- relevant policy, capability, ownership, or persistence interfaces;
- current invariant;
- claimed failure path;
- scope of allowed modifications;
- likely verification commands.

The Context Scout must not modify code or recommend broad refactors.

### 4.3 Adversarial Analyst

The Adversarial Analyst attempts to violate the target invariant before implementation.

It must consider applicable attack classes such as:

- confused-deputy invocation;
- alternate public entry points;
- malformed-but-type-valid input;
- duplicate records or duplicate receipt kinds;
- mutable ownership or stale state;
- ordering and state-transition attacks;
- restart/persistence loss;
- alias and path-normalization variants;
- capability/policy mismatch;
- partial remote failure;
- retries and idempotency;
- source/reference substitution;
- fail-open defaults.

For each attack it records prerequisites, input, expected behavior, observed or reasoned current behavior, and severity.

### 4.4 Test Designer

The Test Designer receives the invariant plus the Context Scout and Adversarial Analyst reports. It does not implement the production fix.

Each regression test specification must state:

- invariant under test;
- attack represented;
- setup;
- operation;
- expected observable behavior;
- the production change that would make the test pass;
- why the existing implementation should fail.

Tests should use real public behavior and avoid excessive mocking or private implementation assertions.

### 4.5 Implementer

The Implementer receives a narrow brief containing:

- binding invariant;
- approved scope;
- RED regression test or explicit test contract;
- allowed files;
- architectural constraints;
- verification commands.

Rules:

- do not weaken tests to achieve GREEN;
- do not broaden authority;
- fail closed;
- do not add dependencies without demonstrated need;
- do not add public APIs unless required;
- do not perform unrelated refactors;
- prefer the smallest production change that restores the invariant.

The Implementer may self-review its diff but may not certify acceptance.

### 4.6 Spec Compliance Reviewer

The Spec Compliance Reviewer asks whether the implementation satisfies the binding requirement independently of the tests.

It must inspect all relevant public entry points and determine whether a caller can still violate the invariant through another route. It reports concrete counterexamples with path, input, expected behavior, actual behavior, and severity.

### 4.7 Security and Quality Reviewer

The Security and Quality Reviewer searches for defects that may survive normal tests, including:

- authorization ordering;
- confused deputy paths;
- input canonicalization;
- TOCTOU and stale state;
- ownership and aliasing;
- error propagation;
- injection;
- secret exposure;
- unintended mutation;
- partial failure;
- concurrency;
- persistence and restart behavior;
- path traversal;
- unsafe defaults.

It must report source-supported issues only and avoid speculative architecture expansion.

### 4.8 Independent Verifier

The Independent Verifier receives only the current commit SHA, expected invariant, changed paths, and required commands. It does not rely on the Implementer's success summary.

It performs the required combination of:

- targeted regression tests;
- affected package tests;
- formatting checks;
- lint/static analysis;
- workspace tests when required by risk tier;
- repository-specific drift/security checks;
- git diff inspection;
- current head SHA confirmation;
- CI result inspection where local verification is insufficient.

The Verifier produces evidence, not a merge action.

### 4.9 Integration Reviewer

After individual findings are closed, the Integration Reviewer considers the PR as a whole and checks for interactions between separately correct fixes, stale documentation, inconsistent assumptions, unresolved threads, and mismatch between the reviewed/tested SHA and the current head.

## 5. Risk-tiered development cells

### Tier 0 — Critical authority/security

Use the full cell:

Context Scout → Adversarial Analyst → Test Designer → RED evidence → Implementer → targeted GREEN → Spec Compliance Reviewer → Security Reviewer → repair loop if required → Independent Verifier → CodeRabbit incremental review → Integration Reviewer.

Examples:

- ForgeCore authorization bypass;
- capability confusion;
- sandbox escape or process-execution authority;
- credential exposure;
- destructive remote mutation;
- provenance or authority decisions based on mutable/untrusted state.

### Tier 1 — Major correctness/security/reliability

Use:

Context Scout → Adversarial Analyst → Test Designer → Implementer → independent reviewer → Independent Verifier → external review adjudication.

### Tier 2 — Normal feature/correctness work

Use:

Context Scout → Test Designer → Implementer → independent reviewer → verifier.

Add an Adversarial Analyst when the change crosses persistence, remote I/O, parsing, routing, or permissions boundaries.

### Tier 3 — Documentation/specification

Use:

Context Scout → one or more adversarial specification critics → spec editor → consistency verifier.

Production implementation is not introduced into a specification-only task merely to satisfy a review suggestion.

### Tier 4 — Style/nit

Use a single scoped implementer plus verification unless the affected code intersects a trust boundary.

## 6. Adversarial prompt contract

All adversarial reviewers receive the following behavioral contract:

> Do not try to prove the patch is correct. Try to construct the smallest source-supported counterexample proving it is incorrect. Only after systematically failing to produce a counterexample within scope may you recommend acceptance. A passing test suite is evidence, not proof. A reviewer comment is a hypothesis, not truth. A type-safe value can still be unauthorized. An implementation comment is not an authority boundary. A private-looking helper may be publicly reachable through re-export. A failure path that defaults to continuation is suspect. Report counterexamples before recommendations.

Additional domain prompts are appended by the controller based on task class.

## 7. Finding model

Every issue, review comment, discovered defect, or test failure is normalized into a Finding record.

Required fields:

```yaml
finding_id: PR29-CR-001
pr: 29
head_sha: ab760531...
source_type: coderabbit
source_ref: PRRT_...
severity: major
category: authorization
status: discovered
invariant: action_type must be RunTest before privileged evaluation
scope:
  allowed_paths: []
validity: unadjudicated
adversarial_cases: []
red_evidence: null
implementation_commit: null
verification:
  targeted: null
  broad: null
  ci: null
reviews:
  spec: null
  security: null
  external: null
rulings: []
```

## 8. Finding state machine

Allowed primary states:

```text
DISCOVERED
  -> INVESTIGATING
  -> CONFIRMED | REJECTED_FALSE_POSITIVE | STALE

CONFIRMED
  -> RED_PROVEN | FIX_JUSTIFIED_WITHOUT_RED

RED_PROVEN | FIX_JUSTIFIED_WITHOUT_RED
  -> IMPLEMENTED
  -> LOCALLY_VERIFIED
  -> ADVERSARIALLY_REVIEWED
  -> CI_VERIFIED
  -> EXTERNAL_REVIEWED
  -> DONE
```

`FIX_JUSTIFIED_WITHOUT_RED` is allowed only when a RED test is technically impossible, unsafe, or meaningless for the change class. The ledger must record the ruling and replacement evidence.

No state transition may be inferred from prose alone when a command, test, or current-source inspection is required.

## 9. Rulings

When the specification, reviewer advice, or available evidence conflicts, the controller records an explicit ruling:

```text
Ruling: <decision>
Evidence: <source, test, code, or tool result>
Reason: <why this interpretation is selected>
Risk if wrong: <consequence>
Next gate: <verification or review step>
```

Rulings are preferred over silent assumptions. Reversible rulings allow development to continue; irreversible or security-sensitive actions retain an explicit human approval gate.

## 10. Repair loop and breaker

For a finding that fails independent review:

- rounds 1-3: the same implementer may repair confirmed defects;
- round 4: replace the implementer with a fresh agent and context brief;
- round 5: use a fresh adversarial reviewer plus architectural adjudication;
- after round 5: every remaining issue receives a documented disposition: must-fix, accepted residual risk, false positive, stale, or deferred with dependency.

The system must not silently enter an unbounded review-fix loop.

## 11. Parallelism and isolation

### 11.1 Safe parallel work

Parallel execution is allowed when tasks:

- concern different PRs or independent subsystems;
- have no shared write scope;
- do not depend on one another's output;
- use separate worktrees or are read-only.

Examples:

- PR #29 Context Scout;
- PR #42 manifest analyst;
- PR #42 permission analyst;
- PR #41 specification critic;
- PR #34 hook-schema scout.

### 11.2 Sequential barriers

Within one finding, the following remains ordered:

RED test → implementation → adversarial review → verification.

Two modifying agents may not share a worktree. An agent may not mutate `main` or another PR's head branch from a task-specific worktree.

## 12. Durable execution ledger

Each plan owns a git-ignored execution workspace containing:

- `progress.md` — append-only task/finding state transitions;
- `briefs/` — self-contained agent briefs;
- `reports/` — Context Scout, adversarial, review, and verification reports;
- `review-packages/` — scoped diffs and evidence presented to reviewers;
- `commands/` — commands executed for verification and their result summaries.

The durable ledger is the recovery authority after context compaction. Git history and ledger state outrank conversational recollection.

## 13. External review ingestion

### 13.1 CodeRabbit

CodeRabbit findings are ingested as untrusted external review data.

For each finding:

1. fetch the current thread and current source;
2. restate the claimed invariant;
3. reproduce or reason through the failure path;
4. classify VALID, PARTIALLY_VALID, FALSE_POSITIVE, or STALE;
5. implement only confirmed defects;
6. verify independently;
7. reply in the inline review thread with evidence;
8. resolve only after the corrected code is present on the PR head and verification evidence exists.

Duplicate one-off review requests must not be issued for already-reviewed unchanged commits. CodeRabbit incremental review is preferred after a corrective commit.

### 13.2 CI

CI status is evidence only for the exact commit SHA it tested. A green run for an earlier head cannot establish the current head as green.

### 13.3 Human review

Human requirements are binding once understood, but ambiguous architectural requirements still require an explicit ruling or clarification before irreversible changes.

## 14. Initial PR routing

### 14.1 PR #29 — ForgeCore RunTest executor

Risk: Tier 0.

Current binding invariant:

`execute_run_test` must reject every action whose `action_type` is not `ActionType::RunTest` before policy evaluation, capability evaluation, runner parsing, or process delegation.

Mandatory adversarial cases:

- `ReadFile` plus its valid capability and `runner: cargo`;
- another policy-permitted action type;
- direct invocation through the public re-export;
- wrong action type plus malformed runner;
- correct RunTest action without RunTest capability;
- RunTest capability attached to a different action type;
- alternate public entry points that bypass the top-level dispatcher.

### 14.2 PR #42 — Mistral connector control plane

Risk: Tier 0 or Tier 1 per finding.

Independent read-only adversarial domains may run in parallel:

- manifest/schema validation;
- permission and confirmation enforcement;
- REST serialization and remote-failure handling;
- deterministic reconciliation and drift;
- secrets and redaction.

Mandatory attack themes:

- read-looking operation that mutates;
- `shared_org` elevation bypass;
- implicit delete/prune;
- secret leakage into logs/evidence;
- connector identity collision;
- partial remote success represented as full success;
- unknown remote state converted to destructive intent;
- dry-run/live-plan divergence.

### 14.3 PR #41 — multi-repository CodeRabbit control-plane specification

Risk: Tier 3.

Use adversarial specification critics for:

- policy precedence;
- cross-repository authority;
- exception/override abuse;
- learned-policy poisoning;
- provenance/fingerprint forgery;
- path confinement;
- drift and rollback.

Assume one managed repository is malicious or materially misconfigured and test whether it can influence policy or state outside its authorized scope.

### 14.4 PR #34 — advisory change-verification router

Risk: Tier 1 before activation, Tier 2 while disabled.

Use:

Schema Scout → Path Adversary → Test Designer → Implementer if required → Verifier.

The actual Cline hook event schema must be established before support is broadened. Attack cases include write/replace/patch variants, multiple paths, Windows separators, `./`, absolute paths, traversal, malformed events, failed mutations, unknown tools, and mixed Kotlin/workflow edits.

The hook must remain advisory-only: `cancel=false`, no subprocess, no shell execution, no file writes, no network access, and no ForgeCore authority.

### 14.5 Existing review debt

After the higher-priority current queue is stable:

- PR #38: authority/provenance/promotion correctness;
- PR #39: persistence plus origin/authentication ordering;
- PR #40: static-rule completeness, reproducible tooling, and executable evidence validation.

## 15. Merge-readiness evidence

No PR is called merge-ready merely because CodeRabbit has no comments.

Merge-readiness requires all applicable evidence:

- no unresolved Critical or Major findings;
- required CI green for the current head SHA;
- security and authority invariants preserved;
- PR scope remains coherent;
- regression tests correspond to real failure modes;
- documentation matches implementation;
- current PR head equals the reviewed and verified SHA;
- no unresolved merge conflicts;
- required independent review complete;
- unresolved lower-severity findings explicitly adjudicated.

The system prepares this evidence but does not merge automatically.

## 16. Controller prompt contract

The runtime controller prompt must encode the following behavior:

1. Establish current head SHA.
2. Identify the binding invariant.
3. Dispatch an isolated Context Scout.
4. Dispatch an Adversarial Analyst when required by risk tier.
5. Synthesize falsifiable requirements.
6. Dispatch a Test Designer.
7. Require observed RED where practical.
8. Dispatch an Implementer with minimal necessary context.
9. Require targeted GREEN.
10. Dispatch independent Spec and/or Security reviewers according to risk tier.
11. Remediate confirmed findings only.
12. Run independent verification.
13. Compare tested SHA with current PR head.
14. Ingest CodeRabbit incrementally.
15. Adjudicate external comments rather than blindly accepting them.
16. Resolve threads only after evidence exists.
17. Update the durable ledger after every stage.
18. Stop before destructive, irreversible, merge, publish, or otherwise security-sensitive actions that require explicit authorization.

## 17. Initial execution waves

### Wave 1 — parallel read-only analysis

- PR #29 Context Scout;
- PR #42 manifest/schema adversary;
- PR #42 permission/confirmation adversary;
- PR #42 remote-failure/reconciliation adversary;
- PR #42 secrets/redaction adversary;
- PR #41 adversarial specification cell;
- PR #34 hook-schema scout.

### Barrier A

Collect and normalize reports. Confirm there are no overlapping write scopes before implementation begins.

### Wave 2 — PR #29 security repair

Sequential:

Test Designer → RED evidence → Implementer → targeted GREEN.

### Wave 3 — PR #29 independent challenge

The Spec Compliance Reviewer and Security Reviewer may run independently against the same immutable candidate commit. Any confirmed defect returns to the repair loop.

### Barrier B

Independent Verifier runs targeted and broad gates on the final candidate commit and confirms the PR head SHA.

### Wave 4 — external review

Allow CodeRabbit incremental review of the corrective commit, adjudicate any new findings, then run the Integration Reviewer.

### Subsequent waves

Repeat the same risk-tiered cell model for the highest-severity confirmed findings from #42, #41, #34, then #38/#39/#40.

## 18. Acceptance criteria for this orchestration system

The design is considered successfully implemented only when the resulting harness can demonstrate all of the following in repository-controlled tests or deterministic simulations:

1. A finding cannot reach DONE without required evidence states.
2. The implementer identity cannot satisfy the independent-review gate for its own task.
3. Two concurrent modifying tasks cannot claim the same worktree/write scope.
4. External review findings require validity adjudication before implementation state.
5. A stale review tied to an older head SHA is detected.
6. Tier 0 findings require adversarial analysis, independent review, and independent verification.
7. Tier 3 specification work cannot silently introduce production implementation.
8. Repair loops trip the breaker after the configured maximum rounds.
9. Merge-readiness is denied when current head differs from the verified SHA.
10. The ledger can recover task/finding position after controller restart or context loss.
11. No orchestration component obtains ForgeCore execution or authorization authority.
12. No review comment can directly cause command execution without passing through the normal implementation/verification workflow.

## 19. Non-goals for the first implementation

The first implementation must not add:

- distributed queues;
- remote agent workers;
- a new database service;
- a general-purpose workflow language;
- autonomous merge;
- autonomous release;
- live connector mutation;
- policy authority outside existing AutoDev boundaries.

The first implementation should remain repository-local, inspectable, deterministic where practical, and easy to remove if the evaluation does not show a quality improvement.

## 20. Evaluation

The implementation plan must include a baseline-versus-candidate evaluation using representative findings from the current PR queue.

At minimum compare:

- defects caught before external review;
- CodeRabbit findings that survive internal adversarial review;
- false-positive adjudication rate;
- repair-loop count;
- duplicate work/review rate;
- time or task count to verified closure;
- number of completion claims rejected due to missing evidence;
- authority-boundary regressions introduced by remediation.

The orchestration system is accepted for broader use only if it increases defect-detection quality without creating a second execution authority or materially increasing unsafe/unbounded automation.