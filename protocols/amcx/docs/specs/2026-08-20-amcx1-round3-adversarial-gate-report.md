# AMCX-1 Round 3 Adversarial Gate Report

**Date:** 2026-08-20  
**Input:** `AMX-ECM-Round-2-DifferenceRecord-matrix.md`  
**Input SHA-256:** `b3e36bc2fe7e85b6cc485806339e3124c7908734cd4c1c505e514f59c8527837`  
**Gate decision:** **FAIL — correct Round 2 before synthesis**  
**Implementation:** Not authorized

## 1. What passed

- The report bytes match its detached manifest.
- The 40 DifferenceRecords reproduce the claimed totals: 4 identical, 4 equivalent renames, 18 complementary, 3 conflicts, 5 missing, and 6 unsupported/unevidenced.
- Its AMX ledger contains 243 unique published IDs and excludes absent serial `AMX-R-0162`.
- Its ECM ledger contains 244 unique contiguous IDs.
- The proposed high-level partition—AMX memory, ECM collaboration, ExecPlan plans, ForgeCore authority/effects, EvidenceStore evidence, Git/CAS artifacts—is directionally sound.
- The report correctly identifies three obvious direct conflicts and many missing lifecycle contracts.

## 2. Why the synthesis gate failed

### 2.1 Ledger presence is not semantic coverage

The per-ID ledgers are complete as lists, but some ledger assignments point to DifferenceRecords that do not cite or semantically cover those IDs. Examples:

- `AMX-R-0013–0018` are assigned to D-0002 although D-0002 cites only `AMX-R-0009–0012`.
- `ECM-R-0013–0020` are assigned to D-0002 although that row cites only `ECM-R-0011–0012`, `0085`, and `0093`.
- `AMX-R-0164` is assigned to D-0019 in the ledger even though the matrix places it under D-0022.
- Cross-cutting calibration rows reuse requirements without declaring primary versus `refines` relationships.

Coverage must be regenerated from mechanically expanded explicit row references. A requirement may map to multiple DifferenceRecords, but every primary assignment must point to a row that actually contains the ID and addresses its semantic domain.

### 2.2 Several complementary classifications hide live conflicts

The following Round 2 classifications require correction or explicit adjudication:

| Record(s) | Round 3 challenge |
|---|---|
| D-0001 | ECM explicitly defines memory schemas, admission, retrieval, and promotion; mission scope overlaps until D-0010/D-0013 are resolved. |
| D-0011 | A present dual-write conflict exists unless the AMX event is the sole memory mutation and ECM stores only command/correlation/result references. |
| D-0012, D-0029 | ECM's task/effect states conflict with ExecPlan/ForgeCore until cardinality, reducers, and receipt-only projections are normative. |
| D-0013, D-0020 | ECM generic promotion can mutate memory trust/visibility unless memory promotion is removed from ECM configuration promotion. |
| D-0002 | AMX prohibits credential/token memory; ECM permits sensitive exceptions under separate governance. Raw secrets must never enter AMCX memory. |
| D-0006 | ECM task conflict resolution cannot resolve AMX causal heads without an AMX merge/retraction event. |
| D-0007, D-0008 | Embedded verification state can become stale relative to EvidenceStore; current eligibility must resolve authoritative freshness. |
| D-0009 | Repository and effect-target canonicalization are unsupported pending provider-aware, cross-language vectors. |
| D-0018 | Context replay lacks a critical recipient/eligibility/deletion-epoch binding and should not be treated as safely complementary yet. |
| D-0021, D-0022 | ECM and AMX admission/retrieval are not equivalent while receiver binding, quarantine, branch/path filtering, and result budgets differ. |
| D-0032 | Adapter composition is unsupported until operation identity, acknowledgement, resume, ordering, backpressure, cancellation, and degradation semantics exist. |
| D-0034 | The implementation sequences conflict if ECM freezes a second memory schema before AMX schemas and the crosswalk exist. |

### 2.3 Canonical representation is not decision authority

Round 2 assigned too much authority to an “AMX lifecycle service.” AMX is a memory interchange/governance contract, not an authorization, identity, evidence, or deletion authority.

The required composition is:

```text
authoritative facts and evidence
  -> external policy/governance decision
  -> AMX transition validation
  -> canonical AMX event and reducer
  -> storage/projection/deletion execution
  -> receipts and audit references
```

AMX may deterministically make handling more restrictive—reject, quarantine, expire, or narrow use—when schema or safety rules require it. Release from quarantine, trust elevation, visibility widening, cross-project publication, and destructive purge require current external authority.

## 3. Recommended conflict resolutions

| Difference | Preferred resolution |
|---|---|
| D-0010 memory contract | AMX record/event/bundle remains canonical. ECM stores a separate noncanonical `ECMMemoryBinding` keyed by AMX IDs/digests for run, attempt, role, collaboration task, context, and promotion-workflow references. Do not embed provider-local execution data into portable AMX records by default. |
| D-0011 event composition | Local phase: separate AMX and ECM logical streams in one SQLite transaction, with ECM holding only a digest-linked result event. Distributed phase: AMX commit plus transactional outbox/inbox and reconciliation. |
| D-0012 plan/task/effect | Keep independent ExecPlan, ECM collaboration, and ForgeCore effect state machines with typed foreign keys, explicit cardinality, completion reducers, and cancellation propagation. ECM effect state is a receipt projection only. |
| D-0013 promotion | Keep memory trust/visibility lifecycle separate from prompt/skill/router activation/canary/rollback. A shared decision-record shape may be reused, not a shared state machine. |
| D-0014 cross-project scope | Require a current scoped user/host grant binding principal, source/destination, exact digest, purposes, sensitivity, expiry, and revocation. Repository PR approval is repository-local only. |
| D-0015 quarantine | Authenticated adapter binds receiver origin; EvidenceStore owns verdict/freshness; AMX validator quarantines/rejects deterministically; external memory-governance policy authorizes release/trust/visibility widening; AMX records the legal transition. |
| D-0016 purge | An external deletion coordinator owns authorization, component job state, non-content anti-resurrection barriers, partial-failure reconciliation, and completion receipts. AMX records retraction and purge-result references. |
| D-0017 critical extensions | Preserve unknown extensions opaquely. Unknown critical semantics cannot influence retrieval, promotion, or effects. Combine object-declared criticality with a reviewed registry snapshot; an object may strengthen but never weaken registry criticality. |
| D-0018 context replay | Store immutable historical context snapshots, but require fresh recipient, lease, authorization, repository, AMX eligibility, deletion-epoch, and expiry validation before reuse. Historical replay is not current-use authorization. |
| D-0019 gates | Use typed, subject-specific GateProfiles with shared mandatory safety/authority/conformance gates. Memory runtime enablement must pass the AMX profile; orchestration-only candidates use ECM profiles. Cross-domain candidates pass both. |

## 4. Revised ownership split

| State or decision | Canonical history/representation | Decision authority | Execution/materialization |
|---|---|---|---|
| Plan steps and replans | ExecPlan | AutoDev plan reducer/policy | AutoDev orchestrator |
| Collaboration tasks/attempts/roles/messages | ECM event log | ECM reducer and role policy | ECM orchestrator/adapters |
| Portable memory records/events/heads/bundles | AMX event DAG | AMX validates grammar only | AMX stores/projections |
| Receiver-bound origin | Origin attestation referenced by AMX | Authenticated host/transport | Identity/attestation store |
| Evidence verdict/freshness | EvidenceStore/VerificationFabric | Independent verifier | Evidence store |
| Quarantine-by-default | AMX event/state | Deterministic AMX restriction | AMX reducer |
| Trust elevation/quarantine release | AMX records the result | External memory-governance policy using authoritative evidence | AMX reducer/projections |
| Cross-project visibility | Approval reference plus AMX event | Scoped user/host grant | Memory-governance service |
| Effective retrieval eligibility | Current decision; not a durable grant | Host/ForgeCore policy intersected with AMX state | Retrieval/context service |
| Consequential effects/receipts | ForgeCore ledger | ForgeCore/host policy | Trusted executor |
| ContextView history | ECM artifact/workflow record | ECM admission plus current policy | ECM context service/CAS |
| Purge job/barrier/receipt | External deletion ledger | Retention/privacy policy or authorized user | Deletion coordinator/adapters |
| Prompt/skill/router activation | ECM promotion log | ECM promotion policy and independent evaluators | Configuration deployment service |
| Schema publication/activation | Reviewed Neutral Contract Registry in Git | Repository review/ADR and authorized maintainers | Validators/adapters consume version |
| Artifact bytes | CAS | Owning domain's retention policy | Artifact service |
| Aggregate budgets | ECM budget ledger | ECM orchestrator/policy | Scheduler/adapters |

## 5. Evaluation contract required before synthesis

A versioned `GateProfile` must declare:

- identity, digest, owner, approval, validity, and provisional/calibrated/retired status;
- candidate kind, promotion target, task/environment/risk applicability;
- parent profiles and conjunctive composition of safety/authority gates;
- exact candidate/baseline/evaluator/environment digests;
- preregistered sampling, missing-data, contamination, and statistical plans;
- metric units, interval method, confidence, power, effect/noninferiority boundary, multiple-comparison and stopping rules;
- stable gate types and reason codes;
- verifier-independence and hierarchical-budget profiles;
- deterministic fixed-scale serialization and cross-language decision vectors.

Gate results are `PASS`, `DEFER`, `REJECT`, or `PROFILE_ERROR`:

1. Any `REJECT` produces final `REJECT`.
2. Otherwise any `DEFER` or `PROFILE_ERROR` blocks promotion and produces final `DEFER`.
3. Only all applicable `PASS` results produce final `PASS`.

Zero observed critical events is not zero population risk. It passes only with declared coverage and an upper confidence bound below a policy tolerance; otherwise it defers.

Verifier independence must be a relationship vector and risk-specific predicate, not an uncalibrated scalar score. Aggregate budgets must form a durable attenuation tree from project/run through ExecPlan step, ECM task, attempt, child delegation, and turn, with atomic reserve-on-spawn and idempotent release/charge.

## 6. Additional critical blockers

Add these to the architectural blocker set:

1. ExecPlan-to-ECM task cardinality, completion reduction, and cancellation propagation.
2. Removal of ECM/ForgeCore dual effect ownership.
3. EvidenceStore versus embedded AMX/ECM verification freshness and authority.
4. Secret handling: AMCX memory stores no raw credentials/tokens; only opaque secret-store references where governed.
5. Adapter delivery/recovery: stable operation IDs, acknowledgements, status/resume, ordering, backpressure, terminal cancellation, idempotency, and degradation reasons.
6. Provider-aware repository identity and effect-target canonicalization with cross-language vectors.
7. Schema activation authority restricted to the reviewed contract registry; ECM may evaluate but cannot activate schemas.
8. A machine-readable task `AcceptanceContract`; an empty or agent-selected evidence set can never satisfy completion.
9. Severity/blast-radius recovery policy distinguishing projection, task-tree, tenant, and global stops.

Publication-maintenance defects `AMX-R-0162` and AMX-R-0203 remain required corrections but are not, by themselves, critical architecture blockers because the immutable source governs.

## 7. Required Round 2 correction

The original Round 2 report remains immutable evidence. Its successor must:

1. Regenerate coverage from explicit expanded DifferenceRecord references and allow multi-record mappings.
2. Add exact source span, quotation digest, original modality, extraction kind, and source-governance precedence for every adjudicated requirement.
3. Reclassify or explicitly rebut every Round 3 challenge in §2.2.
4. Split canonical history, decision authority, and physical execution in the ownership table.
5. Add the blockers in §6.
6. Preserve all 40 original DifferenceRecord IDs; corrections use versioned supersession metadata rather than silent rewriting.
7. Emit a machine-readable coverage-validation result and detached digest.
8. Stop before synthesis and implementation.

## 8. Exact correction prompt for the AMX worker

```text
Round 3 adversarial review has rejected the Round 2 matrix as authoritative synthesis input. Do not revise AMX-1 or ECM and do not implement. Preserve the original report unchanged and create a versioned Round 2 correction/supersession artifact.

First regenerate requirement coverage mechanically from explicit, expanded DifferenceRecord references. The current ledger is semantically false in several places: AMX-R-0013–0018 and ECM-R-0013–0020 are assigned to D-0002 without being cited or fully covered, and AMX-R-0164 is inconsistently assigned. Allow requirements to map to multiple DifferenceRecords, but every primary mapping must cite and semantically address the ID.

Then re-adjudicate these challenged records, either correcting them or rebutting each challenge with source evidence and a falsifiable test: D-0001, D-0002, D-0005–D-0009, D-0011–D-0013, D-0018, D-0020–D-0022, D-0026, D-0029, D-0031–D-0034.

Apply these ownership constraints:
- AMX owns canonical memory representation, legal transition grammar, event history, causal heads, and bundles; it is not the authority for identity, evidence truth, cross-project approval, quarantine release, visibility widening, or purge authorization.
- ECM owns collaboration state only and holds references/projections for ExecPlan, ForgeCore effects, EvidenceStore verdicts, and AMX memory.
- ForgeCore owns authorization/effects; ExecPlan owns plan/step lifecycle; EvidenceStore/VerificationFabric owns verdict/freshness.
- An external memory-governance policy authorizes trust/visibility widening; an external deletion coordinator owns purge jobs/barriers/receipts.
- Only the reviewed Neutral Contract Registry may publish/activate schemas. ECM may evaluate schema candidates but not activate them.

For D-0010, compare AMX plus a separate noncanonical ECMMemoryBinding against embedded ECM extensions and a third superset; do not assume embedded extensions are best. For D-0017, preserve unknown bytes while failing semantic use closed for unknown critical extensions. For D-0019, specify typed subject-specific GateProfiles, verifier relationship predicates, and hierarchical reserve-on-spawn budgets; mark numeric constants provisional until calibrated.

Add critical blockers for task cardinality/reduction, effect dual ownership, evidence freshness, raw secret prohibition, adapter delivery/recovery, repository/effect-target canonicalization, schema activation authority, machine-readable AcceptanceContract, and severity-scoped recovery.

For every adjudicated requirement include exact source span, quotation digest, original modality, extraction kind, and alias/refinement relationship. Preserve all original D-#### identifiers and add supersedes/superseded_by metadata. Report corrected coverage for all 243 AMX and 244 ECM requirements plus machine-verification commands/results and a detached SHA-256 manifest.

End by stating whether the corrected matrix is safe for Round 4 synthesis. Return no implementation code.
```
