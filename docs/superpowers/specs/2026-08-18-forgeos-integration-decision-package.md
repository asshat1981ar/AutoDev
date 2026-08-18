# ForgeOS Integration Decision Package

**Date:** 2026-08-18
**Status:** Decision gate for new integration architecture; existing approved authority-foundation DAG may continue.
**Branch:** `feat/forgeos-authority-foundation`
**Primary tracker:** #26

## 1. Repository ground truth

ForgeCore is already the trusted execution boundary. The active authority foundation intentionally separates evidence eligibility, work intent, capability ownership, resource ownership, and existing execution authorization rather than allowing model output to become authority.

Current verified DAG state at creation of this package:

```text
A1 EvidenceRecord validation                 GREEN
A2 Closed lease-policy algebra               GREEN
A3 Safety floors + controlled relaxation     GREEN
A4 Refresh evaluator + trust hardening       GREEN
A5 Immutable LeaseAttestation                GREEN
A6 Lease-aware current verification          NEXT
A7 WorkContract                              BLOCKED BY A6
A8-C Capability ownership                    BLOCKED BY A7
A8-R Resource ownership                      BLOCKED BY A7
A9 Cross-domain authority composition        BLOCKED BY A8-C/A8-R
```

A5 implementation is verified by CI run `32146875626` after a formatting-only repair at commit `8f254cbebcbc73a4403192b4569ff75127a626f6`.

The existing foundation plan explicitly forbids execution integration until authority composition is independently verified. This remains the correct boundary.

## 2. Current architecture

```text
Untrusted model / agent
        |
        v
   proposed intent
        |
        v
EvidenceRecord -> LeaseEvaluation -> LeaseAttestation
        |
        |        (A6-A9 still being completed)
        v
WorkContract -> CapabilityGrant -> ResourceLease
        |
        v
Cross-domain authority composition
        |
        |  future separate action-transaction plan
        v
Authorization -> constrained execution
        |
        v
ExecutionReceipt -> independent verification -> commit / rollback / replan
```

The key rule is unchanged:

> Evidence eligibility is not capability ownership; capability ownership is not resource ownership; none of them is execution authorization.

## 3. Research findings

### Signal A — Schema-driven generated interfaces: validated, later-layer integration

Google Sheets Canvas demonstrates a useful product pattern: structured state can be projected into generated, interactive, read-write interfaces while the underlying data remains authoritative. This supports a future ForgeOS `WorkspaceSpec -> trusted component registry -> renderer` design.

Decision: **do not place generated UI logic in ForgeCore.** A constrained declarative `WorkspaceSpec` is preferable to model-generated arbitrary Kotlin/JS code, but it depends on stable public lifecycle and evidence schemas that do not exist yet.

Primary source: Google Docs Editors Help, “Create a Sheets canvas,” 2026.

### Signal B — Ambient agent state: concept validated; prior Pixel-specific claim rejected

The previously cited Pixel-specific “HiLight” implementation could not be verified from a current primary Google source and is therefore rejected as architectural evidence.

The product pattern itself is independently supported by Android Live Updates and progress-centric notifications: Android explicitly provides prominent system surfaces and status chips for ongoing, user-initiated, finite activities.

Decision: define typed agent lifecycle state first; project it into Android Live Updates later. UI must never infer security-relevant state from natural-language logs.

Primary sources: Android Developers, “Create live update notifications” and “Live update notifications,” 2026.

### Signal C — Zero-trust autonomous-agent execution: strongly validated, already aligned

Google Cloud’s 2026 Agent Identity, Agent Gateway, Agent Sandbox, and Agent Executor architecture reinforces the existing ForgeOS direction: agents need verifiable identities, policy-controlled connections, isolated execution, durable state, and constrained side effects. Agent Executor’s durable event log and single-writer session consistency are especially relevant to later scheduler/state-ledger work.

Decision: **do not create a second zero-trust subsystem.** Complete the existing ForgeCore authority foundation and map later runtime adapters onto it.

Primary sources: Google Cloud, “What’s new in IAM: Security, governance, and runtime defense” (2026); “Introducing Agent Executor, Google’s distributed Agent Runtime” (2026).

### Signal D — Compute-aware model routing: validated, outside trusted kernel

NVIDIA’s Nemotron NVFP4/QAD work shows that lower-precision models can recover near-BF16 quality while materially reducing inference cost, and NeMo Switchyard makes routing an explicit agent-system concern. This supports optimizing verified useful work per unit of compute rather than routing by provider name.

Decision: model routing belongs above ForgeCore. ForgeCore may expose risk/evidence requirements as routing inputs, but it must not choose providers/models or weaken authority based on model tier.

Primary sources: NVIDIA Nemotron QAD technical material (2026); NVIDIA Nemotron 3.5 Lightning / NeMo Switchyard announcement (2026).

### Signal E — Proof-carrying software development: strongly validated, but must follow A9

Vero evaluates repository-scale implementation plus machine-checked proofs and shows that current agents remain weak on multi-module consistency: its strongest evaluated configuration solved 27/43 repositories. Its grading design is particularly important: artifacts are evaluated under controlled grading rather than trusting the agent’s own completion claim.

Lean4Agent separately reports that workflows passing formal semantic verification outperform failing workflows by an average 11.94%, and that workflow evolution grounded in those verifier outputs yields further gains. The useful lesson is not “formalize all model reasoning”; it is to formalize stable workflow/authority invariants and keep acceptance outside model inference.

Decision: the first post-foundation plan should be `forgeos-action-transaction`, binding frozen intent and authority to `ExecutionReceipt`, independent `VerificationBundle`, and a commit/rollback/replan decision. Formal trajectory proofs should follow after the transaction interface stabilizes.

Primary sources: Vero, arXiv:2608.13522 (2026); Lean4Agent, arXiv:2606.06523 (2026).

## 4. Contradiction analysis

| Contradiction | Resolution |
|---|---|
| More autonomy vs less trust | Increase capability breadth while narrowing authority through contracts, scoped grants, resource leases, and independent verification. |
| Parallelism vs shared-state consistency | Isolated speculative work + typed resource ownership + single commit barrier. |
| Generated UI vs arbitrary-code risk | Declarative `WorkspaceSpec` rendered only through trusted registered components. |
| Strong models vs compute cost | Risk/evidence-aware hierarchical routing outside ForgeCore. |
| Fewer human approvals vs trustworthy outcomes | Machine-checkable evidence thresholds and external acceptance checks. |
| Historical evidence vs current truth | Preserve immutable history while separately evaluating current lease eligibility. |

## 5. Weighted opportunity matrix

Weights: architectural leverage 20%, security/reliability 20%, future autonomy 15%, verification quality 15%, implementation-cost efficiency 10%, Android/product value 10%, research support 5%, reversibility 5%.

| Candidate | Weighted score | Decision |
|---|---:|---|
| Complete authority foundation A6-A9 | 9.15 | Immediate; already approved |
| Proof-carrying action transaction | 9.30 | First new architecture after A9 |
| Fresh/independent acceptance verifier | 8.90 | Part of action-transaction program |
| Formal trajectory invariants | 8.15 | Near-term after transaction freeze |
| Typed `AgentPhase` + Android Live Updates | 8.05 | Near-term product projection |
| Compute-aware model routing | 7.00 | Prototype outside ForgeCore |
| Declarative generated `WorkspaceSpec` | 6.45 | Defer until public lifecycle schema stabilizes |
| Pixel-specific ambient effect | 3.15 | Reject; unverified and non-portable |

## 6. Target architecture

```text
                    Product / Android
          WorkspaceSpec + typed AgentPhase projection
                            |
                            v
                Coordinator / Model Router
                            |
                       ProposedAction
                            |
                            v
                        ForgeCore
       EvidenceLease + WorkContract + CapabilityGrant
               + ResourceLease + policy composition
                            |
                       authorization
                            |
                            v
                 Constrained capability adapter
                            |
                     ExecutionReceipt
                            |
                            v
                Independent verifier context
                            |
                    VerificationBundle
                            |
                            v
          Commit | Rollback | Repair | Replan | Escalate
```

Generated UI and routing remain outside the trusted Rust kernel. Security-relevant lifecycle truth originates from typed control-plane state, not presentation logs.

## 7. Threat model additions

The post-foundation action-transaction plan must explicitly defend against:

- stale or replayed evidence;
- contract revision drift between authorization and execution;
- capability or resource scope widening;
- TOCTOU between authorization and mutation;
- self-authored/self-accepted success claims;
- verification performed only in the mutated agent workspace;
- model-supplied approval bits or completion status;
- trajectory-level policy violations composed from individually valid actions;
- concurrent writers to the same resource;
- verifier input contamination from agent-produced evaluation signals.

## 8. Critical path and task DAG

```text
A6 current evidence gate
  -> A7 immutable WorkContract
      -> interface freeze
          +-> A8-C capability ownership --+
          +-> A8-R resource ownership -----+
                                             -> A9 composition + independent review
                                                 -> NEW: action transaction / proof-carrying receipts
                                                     -> typed AgentPhase
                                                     -> trajectory guard
                                                     -> model routing evaluation
                                                     -> WorkspaceSpec generated UI
```

A8-C and A8-R are the first safe parallel implementation workstreams, provided shared exports are frozen before dispatch and integration is performed by one owner.

## 9. Verification strategy

Every security-relevant slice continues to use:

```text
RED test
 -> verify RED failure is the intended missing behavior
 -> minimal GREEN implementation
 -> focused tests
 -> rustfmt
 -> clippy -D warnings
 -> regressions
 -> full CI where required
 -> independent review gate
```

For post-A9 execution, success must additionally bind:

```text
Task/contract identity
+ start revision/state
+ exact authorized action
+ capability/resource grants
+ execution result
+ resulting revision/state
+ verifier evidence
```

The verifier should operate on a fresh/frozen reconstruction where practical, following the same anti-self-grading principle demonstrated by Vero.

## 10. APK/product impact

The authority work remains connected to the Android product through CI, which currently builds/tests the Kotlin/Android lane and produces the debug APK. New agent-status UX should not begin until typed lifecycle state exists; after that, Android Live Updates are a better first system-level projection than custom ambient effects.

## 11. Major architectural changes requiring a separate approval gate

Do not implement these merely because they scored well:

1. `forgeos-action-transaction` introducing `ProposedAction`, `ExecutionReceipt`, `VerificationBundle`, and state-transition commit/rollback semantics.
2. Formal trajectory-guard / Lean-backed kernel-invariant work.
3. Public typed `AgentPhase` lifecycle protocol and Android Live Update integration.
4. Compute-aware model-routing policy and evaluation harness.
5. Declarative generated `WorkspaceSpec` and trusted component registry.

The existing A6-A9 authority-foundation plan is already approved and should continue without absorbing these later programs.

## 12. Decision

**Proceed now:** A6 -> A7 -> A8-C/A8-R -> A9, exactly within the existing authority-foundation boundaries.

**After A9:** propose the proof-carrying action-transaction architecture as the first new integration slice.

**Reject/defer now:** arbitrary generated UI code, model routing inside ForgeCore, execution integration before A9, and any Pixel-specific ambient design based on the unverified prior claim.

Optimization objective remains:

> Maximize verified autonomous progress per unit of authority, compute, and human attention.
