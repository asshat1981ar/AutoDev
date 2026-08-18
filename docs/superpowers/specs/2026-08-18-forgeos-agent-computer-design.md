# ForgeOS Agent Computer Architecture

Date: 2026-08-18
Status: Approved in chat; written-spec review pending
Scope: AutoDev / ConnectorForge evolution from architecture evidence to a capability-secure agent computer

## 1. Problem

AutoDev already separates architecture/research evidence from execution authority. The next step is to extend that trust boundary into a complete agent-computer control plane without allowing models, connectors, or plugins to gain ambient authority.

The target system must support autonomous research, planning, parallel software-development agents, browser/computer control, filesystem/process/git operations, sandboxed components, verification, recovery, and long-horizon execution while keeping the final authorization and state-transition authority in deterministic Rust code.

## 2. Existing invariants to preserve

The following constraints are architectural invariants unless later repository evidence and an explicit architecture revision supersede them:

- `EvidenceRecord` is an immutable observation, not mutable current state.
- Historical architecture decisions remain historical even when their supporting evidence becomes stale.
- Current eligibility must be evaluated separately from historical decision maturity.
- External adapters may provide observations and proposals; ForgeCore decides normalized trust.
- Evidence eligibility is not execution authorization.
- `LeaseAttestation` must never become an `AuthorizationGrant`.
- Policy evaluation must remain deterministic, use injected time, and contain no network calls, connector SDKs, arbitrary scripting, tool execution, credentials, or plugin execution.
- Unknown or malformed policy state fails closed.
- Repository policy may tighten built-in safety floors freely; relaxation requires explicit repository-backed approval evidence.
- ForgeCore remains connector-neutral.

These invariants preserve the existing central rule:

> External systems provide observations and proposals; trusted Rust components determine whether normalized evidence satisfies trust and authorization gates; execution remains a separate operation.

## 3. Architecture decision

Adopt **ForgeOS** as an agent-computer control plane built around five ownership domains:

1. Evidence ownership — who may assert an observation and under what provenance.
2. Capability ownership — which principal possesses authority to request an operation.
3. Resource ownership — which principal currently controls a browser, file, process, worktree, terminal, network origin, or secret handle.
4. Intent ownership — which immutable WorkContract permits the requested operation.
5. Commit ownership — which trusted component may make an irreversible state transition authoritative.

The core authorization relation is:

```text
AuthorizedAction =
    ValidIntent
  ∩ CurrentEvidence
  ∩ EffectivePolicy
  ∩ CapabilityLease
  ∩ ResourceLease
  ∩ TrajectoryInvariant
```

No model output or connector response may substitute for a missing term.

## 4. High-level architecture

```text
                         USER INTENT
                              |
                              v
                    +------------------+
                    | Objective Kernel |
                    +--------+---------+
                             | WorkContract
                             v
 +-----------------------------------------------------------+
 |                    FORGE RUST KERNEL                      |
 |                                                           |
 | Evidence -> Lease -> Policy -> Capability -> Authorization|
 |                            |             |                 |
 |                            v             v                 |
 |                    Trajectory Guard   Action Grant         |
 |                            |             |                 |
 |                       State Ledger <-----+                 |
 +----------------------------+------------------------------+
                              | typed, scoped capabilities
             +----------------+----------------+
             v                v                v
        Browser Cell      Shell Cell       Code/Git Cell
             |                |                |
             +----------------+----------------+
                              v
                       Evidence Events
                              |
                              v
                         Rust Kernel
                              |
                        verify/replan
```

ForgeCore decides; execution adapters act; verification converts effects back into evidence.

## 5. Crate/module boundaries

### 5.1 `forge-core`

Trusted, deterministic decision kernel.

Existing domains remain intact. Add focused modules rather than a monolith:

```text
architecture_evidence.rs
architecture_lease.rs
objective.rs
work_contract.rs
capability.rs
authorization.rs
resource_lease.rs
trajectory.rs
state_transition.rs
audit.rs
```

ForgeCore must answer questions such as:

- Is the evidence structurally valid and currently admissible?
- Does the WorkContract permit this outcome and capability class?
- Does the principal own or validly borrow the required capability?
- Does the principal currently hold the resource lease?
- Does the proposed action remain inside trajectory invariants?
- Is human approval required?
- May this grant be delegated?

ForgeCore must not execute browser, shell, filesystem, git, network, or connector actions.

### 5.2 `forge-runtime`

Owns orchestration mechanics, not trust policy:

- async task scheduler;
- DAG execution;
- cancellation and timeouts;
- retries and budgets;
- agent lifecycle;
- event routing;
- resource-lease coordination;
- failure propagation.

### 5.3 `forge-computer`

Typed host adapters for:

- browser/computer;
- filesystem;
- process and terminal;
- git/worktree;
- network;
- clipboard/window where needed.

Prefer typed operations over unrestricted shell access whenever a narrower primitive exists.

### 5.4 `forge-sandbox`

Wasmtime/WASI component host for sandboxed tools and worker components.

Responsibilities:

- WIT interface definitions;
- generated bindings;
- component instantiation;
- capability-to-import linking;
- resource quotas;
- cancellation;
- sandbox evidence emission.

A component that lacks a capability must not receive the corresponding host interface.

### 5.5 `forge-eval`

Evaluation and regression harness:

- invariant/property tests;
- workflow simulations;
- adversarial scenarios;
- long-horizon agent-computer benchmarks;
- safety evaluation;
- regression corpus;
- reliability metrics.

## 6. WorkContract

`WorkContract` is the immutable runtime representation of user intent.

Conceptual fields:

```text
id
revision
objective
allowed_outcomes
forbidden_outcomes
required_evidence
permitted_capability_classes
resource_scope
approval_thresholds
budget
termination_conditions
verification_criteria
contract_fingerprint
```

Requirements:

- canonical deterministic serialization and fingerprinting;
- grants bind to an exact contract revision/fingerprint;
- agents may propose a `ContractProposal` but cannot mutate a contract;
- only trusted policy/root authorization may issue a `WorkContractRevision`;
- stale contract revisions cannot authorize new actions.

## 7. Capability ownership

Capabilities are closed typed enums/structures rather than arbitrary permission strings.

Initial capability families:

```text
Browser.Read
Browser.Click
Browser.Type
Browser.Navigate(origin)
Filesystem.Read(path)
Filesystem.Write(path)
Process.Spawn(binary, args)
Process.Signal(pid)
Git.Read
Git.Commit(worktree)
Git.Push(branch)
Network.Request(origin, method)
Secret.Use(name, purpose)
Agent.Spawn(role, budget)
Agent.Delegate(capability_subset)
```

Conceptual immutable grant:

```text
CapabilityGrant {
    grant_id,
    objective_id,
    work_contract_id,
    work_contract_fingerprint,
    principal,
    capability,
    resource_scope,
    argument_constraints,
    issued_at,
    valid_until,
    max_uses,
    parent_grant,
    policy_fingerprint,
    evidence_fingerprints,
}
```

### 7.1 Borrow-checker semantics

Runtime capability ownership should mirror useful Rust ownership properties:

- exclusive capability — one active exclusive holder;
- shared read capability — multiple readers where safe;
- bounded borrow — temporary delegated use without authority amplification;
- move — original holder loses use after transfer;
- delegation — strict subset of parent authority;
- expiry — unusable at or after expiry;
- revocation — immediately blocks subsequent authorization;
- usage budget — grant becomes unusable when exhausted.

Invariant:

```text
ChildCapabilities ⊆ ParentCapabilities
```

A child agent cannot amplify authority.

## 8. Resource ownership

Represent resources as typed identities, including:

```text
BrowserSession
BrowserTab
FilesystemPath
GitWorktree
Process
TerminalSession
NetworkOrigin
SecretHandle
```

`ResourceLease` controls mutation/read concurrency.

Default policy:

- exclusive writer where concurrent mutation would conflict;
- shared readers where safe;
- explicit acquisition before mutation;
- lease expiry/revocation stops later operations;
- cancellation or agent failure releases or quarantines resources deterministically.

Parallel writer agents must use separate worktrees unless a frozen interface and explicit merge protocol says otherwise.

## 9. Separation of evidence, authority, execution, and verification

The following types must remain distinct:

```text
EvidenceRecord
LeaseAttestation
CapabilityGrant
AuthorizationDecision
ExecutionReceipt
VerificationResult
StateTransitionCommit
```

Semantics:

- `EvidenceRecord`: what was observed.
- `LeaseAttestation`: evidence satisfied a current policy at a specific time.
- `CapabilityGrant`: a principal may request a bounded class of operations.
- `AuthorizationDecision`: this exact proposed action is allowed/denied/requires review.
- `ExecutionReceipt`: an adapter reports what it attempted and observed.
- `VerificationResult`: an independent postcondition check.
- `StateTransitionCommit`: trusted acceptance of the verified state change.

An execution receipt alone never proves task success.

## 10. Action transaction protocol

Every consequential computer action follows:

```text
OBSERVE
  -> PROPOSE
  -> AUTHORIZE
  -> RESERVE RESOURCE
  -> EXECUTE
  -> OBSERVE RESULT
  -> VERIFY POSTCONDITION
  -> COMMIT / ROLLBACK / REPLAN / ESCALATE
```

A `ProposedAction` must contain at least:

```text
objective_ref
work_contract_ref
requested_capability
target_resource
arguments
evidence_refs
policy_fingerprint
expected_state_delta
verification_method
reversibility
estimated_budget
```

For destructive or high-impact operations, support a prepare/commit pattern so the system can inspect expected deltas before accepting an irreversible change.

## 11. Trajectory Guard

Per-action authorization is insufficient for long workflows. ForgeOS therefore maintains a trajectory state spanning the entire objective.

Conceptual state:

```text
TrajectoryState {
    objective,
    work_contract,
    invariants,
    resource_state,
    consumed_capabilities,
    irreversible_transitions,
    evidence_dependencies,
    expected_postconditions,
    budget_usage,
}
```

Evaluate both the local action and the composed trajectory.

Example invariants:

- never delete the source tree;
- never expose a secret outside its approved purpose;
- never push to a protected branch without explicit approval;
- never modify files outside the assigned worktree;
- never exceed configured monetary/tool budgets;
- preserve the current WorkContract objective;
- do not silently broaden network origins;
- do not accept an unverified state transition as complete.

When actual state diverges from expected state:

1. stop dependent actions;
2. reobserve;
3. classify the mismatch;
4. perform deterministic authorized repair where possible;
5. otherwise replan;
6. escalate where policy requires approval;
7. abort if invariants cannot be restored.

## 12. Sandboxed components

Use WASI/Wasmtime where practical to create enforceable capability boundaries for plugins, tools, and worker components.

Define WIT worlds/interfaces such as:

```text
forge:filesystem
forge:network
forge:git
forge:process
forge:agent
forge:evidence
```

The host links only interfaces represented by current capabilities.

```text
No capability
    -> no imported interface
    -> operation unavailable
```

This boundary complements Rust-side authorization; it does not replace it.

## 13. Parallel agent topology

The root orchestrator retains architecture and integration ownership.

Preferred topology after shared interfaces are frozen:

```text
Root Orchestrator
  |
  +-- Researcher / Explorer / Planner
  |
  +-- contract freeze
  |
  +-- Kernel Agent      -> forge-core
  +-- Runtime Agent     -> forge-runtime
  +-- Sandbox Agent     -> forge-sandbox
  +-- Computer Agent    -> forge-computer
  +-- Security Agent    -> independent threat/invariant review
  +-- Verification Agent-> tests/evaluation harness
  |
  +-- Integrator
```

Rules:

- specialists receive bounded task contracts;
- multiple writers do not share a writable tree;
- shared interfaces are frozen before parallel implementation;
- independent reviewers should not see each other's conclusions before initial review;
- final integration decisions remain with the root;
- confident summaries without evidence are rejected.

## 14. Agent task contract

Each dispatched implementation agent receives only the context needed for its workstream:

```text
objective
owned modules/files
frozen interfaces
allowed dependencies
required tests
prohibited changes
capability budget
resource/worktree assignment
required evidence
return schema
```

This prevents context poisoning and accidental cross-workstream architectural mutation.

## 15. Memory and learning boundary

Separate memory categories:

```text
Observation
Evidence
Hypothesis
Decision
Episode
LearnedHeuristic
```

Memory may inform proposals and planning but must not silently mutate policy, WorkContracts, capabilities, or authorization rules.

Historical lease evaluations and execution outcomes may later feed W5-style learning/evaluation loops, but learned heuristics remain proposals until explicitly promoted through policy/architecture review.

## 16. Testing strategy

### 16.1 Unit/property tests

Cover:

- deterministic canonical fingerprints;
- malformed structures fail closed;
- exact expiry boundaries;
- revoked/expired capabilities;
- strict-subset delegation;
- moved capabilities cannot be reused by prior owner;
- exclusive/shared borrow rules;
- stale WorkContract revisions;
- wrong policy/evidence fingerprints;
- wrong resource ownership;
- duplicate/replayed actions;
- deterministic authorization for identical input.

### 16.2 Resource/concurrency tests

Cover:

- double writer;
- shared-reader behavior;
- lease expiry;
- cancellation;
- dead agent;
- process crash;
- orphan cleanup;
- concurrent acquisition ordering;
- worktree isolation.

### 16.3 Action transaction tests

Cover:

- expected success;
- adapter failure before effect;
- partial write;
- effect succeeds but verification fails;
- rollback succeeds/fails;
- postcondition mismatch;
- retry deduplication;
- irreversible operation requiring approval.

### 16.4 Trajectory tests

Cover:

- sequence of individually valid actions that violates a global invariant;
- stale hidden state;
- changed browser/file/git state;
- budget exhaustion;
- secret propagation attempt;
- protected branch mutation attempt;
- unauthorized scope expansion;
- successful repair/replan.

### 16.5 Repository gates

For each coherent implementation card:

```text
focused cargo tests
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
workspace tests
repository CI
```

No slice is complete until objective verification evidence is recorded.

## 17. Evaluation metrics

Measure at least:

```text
task completion
partial completion
unsafe state-transition rate
policy violation rate
verification omission rate
recovery rate
tool/action count
latency
token/model cost
human interventions
rollback success
resource conflict rate
```

Do not optimize task completion at the expense of safety or verification.

## 18. Implementation phases

The implementation plan must decompose this architecture into reviewable phases:

### Phase 0 — Ground truth

- inspect current branch, architecture, tests, CI, and prior merged work;
- reconcile the previously approved Evidence Lease design with current `main`;
- establish RED/GREEN baseline.

### Phase 1 — Evidence Lease foundation

Complete the previously approved lease-gate slice before allowing it to become an input to execution authorization.

Expected sequence:

1. public `EvidenceRecord` structural validation;
2. closed lease-policy algebra and registry;
3. safety-floor comparison and controlled relaxation;
4. `RefreshProposal` + deterministic evaluator;
5. immutable `LeaseAttestation`;
6. lease-aware current verification;
7. docs + repository verification.

### Phase 2 — WorkContract kernel

Introduce immutable intent contracts and contract fingerprints.

### Phase 3 — Capability kernel

Introduce typed capabilities, scoped grants, borrowing, moves, delegation, expiry, revocation, and usage budgets.

### Phase 4 — Resource ownership

Introduce typed resources and resource leases.

### Phase 5 — Action transaction protocol

Introduce proposal, authorization, execution receipt, postcondition verification, and trusted commit/rollback semantics.

### Phase 6 — Typed computer adapters

Build browser, filesystem/process, and git adapters behind frozen interfaces.

### Phase 7 — WASI sandbox

Add WIT worlds and capability-linked Wasmtime components.

### Phase 8 — Trajectory kernel

Add global invariants, state tracking, divergence detection, repair/replan/escalation.

### Phase 9 — Parallel agent scheduler

Add typed `AgentContract`, DAG scheduling, isolated worktrees, cancellation/failure propagation, and root integration.

### Phase 10 — Memory/evaluation integration

Add typed episodes/heuristics and objective evaluation without allowing memory to become policy.

## 19. Parallel implementation waves

Parallel work begins only after the dependencies and interfaces for that wave are frozen.

### Wave A — foundation

Mostly sequential because each trust primitive depends on the previous one:

```text
Evidence Lease -> WorkContract -> Capability -> Resource Lease
```

### Wave B — runtime split

After capability/resource interfaces freeze:

- Runtime Agent — scheduler, cancellation, budgets, DAG;
- Sandbox Agent — WIT/Wasmtime host;
- Computer Agent — typed adapters;
- Verification Agent — cross-cutting test harness;
- Security Agent — independent review.

Use isolated worktrees for all writers.

### Wave C — integration

Root integrator combines green workstreams and introduces Action Transaction + Trajectory Guard using frozen interfaces.

## 20. Security model

Assume untrusted inputs include:

- model output;
- tool output;
- web pages;
- connector responses;
- plugin/component output;
- retrieved memory;
- generated code;
- repository content not yet verified.

The system should defend against:

- confused deputy behavior;
- capability amplification;
- prompt/tool injection;
- stale evidence;
- replay;
- cross-objective resource use;
- secret exfiltration;
- path/network scope expansion;
- agent impersonation;
- incomplete or forged execution receipts;
- individually authorized actions composing into an unsafe trajectory.

## 21. Non-goals for the first implementation plan

Do not initially build:

- a replacement operating system kernel;
- a general-purpose policy scripting language;
- unrestricted shell as the primary API;
- autonomous policy self-modification;
- automatic merge of high-impact branches;
- uncontrolled cross-agent shared mutable state;
- connectors inside ForgeCore;
- a distributed multi-host runtime before the single-host authority model is proven.

## 22. Acceptance criteria

ForgeOS architecture is considered implemented only when objective evidence demonstrates:

- the Evidence Lease Gate is complete and current verification remains distinct from historical state;
- WorkContracts deterministically bind user intent;
- capabilities are typed, scoped, expiring, revocable, and non-amplifiable through delegation;
- resource ownership prevents conflicting mutation;
- a model cannot execute an action merely by requesting it;
- execution receipts cannot masquerade as verification;
- postconditions are independently checked;
- global trajectory invariants can stop locally valid but globally unsafe workflows;
- child agents cannot gain authority their parent lacks;
- sandboxed components expose only linked authorized interfaces;
- parallel writer agents are isolated by worktree/resource contracts;
- critical authorization logic remains deterministic and connector-free;
- focused tests, lint, workspace tests, and repository CI are green;
- no completion claim is made without recorded evidence.

## 23. First implementation-plan boundary

The first executable implementation plan should **not** attempt all ForgeOS phases at once.

It should cover the minimum trusted vertical foundation needed before safe parallel agent-computer execution:

```text
Phase 0  current repo ground truth
Phase 1  Evidence Lease Gate
Phase 2  WorkContract
Phase 3  Capability Grant + delegation subset rules
Phase 4  ResourceLease core
```

At the end of that plan, ForgeCore should be able to answer whether a proposed action principal owns valid intent, current trust, capability, and resource authority, without yet executing that action.

A second design/plan cycle should then introduce the execution adapters, Wasmtime component host, action transactions, and trajectory guard on top of those verified boundaries.

## 24. Design rationale

The key design choice is to make **authority explicit and typed before increasing autonomy**.

The system intentionally uses Rust-inspired ownership at runtime rather than trusting agent prompts to respect boundaries. Evidence, intent, capabilities, resources, execution results, and verification are modeled as separate immutable or tightly controlled artifacts. Parallelism is introduced only after ownership contracts are frozen and isolated.

This makes increasing agent autonomy compatible with decreasing ambient authority.