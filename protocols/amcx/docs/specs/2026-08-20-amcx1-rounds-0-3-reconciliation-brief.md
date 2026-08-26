# AMCX-1 Reconciliation Brief — Rounds 0–3

**Date:** 2026-08-20  
**Status:** Sources and normalizations verified; bidirectional external review pending  
**Implementation:** Prohibited during this reconciliation round

## 1. Verified inputs

| Artifact | SHA-256 | Verification |
|---|---|---|
| AMX-1 normative source | `4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9` | Local bytes match publication claim |
| AMX-1 normalization | `c81aedb9528df2162e5c327f6479a89848e70bf85a3835b3d76b67e5b06dae52` | Local bytes and detached manifest match |
| ECM normative source | `e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980` | Local bytes match published digest |
| ECM normalization | `9ddf7754d017384f4d26ef801eac333a8e2a4148ef3d276fd178a032c49c7810` | 244 contiguous unique `ECM-R-*` IDs |

AMX exposes 243 unique IDs from `AMX-R-0001` through `AMX-R-0244`; `AMX-R-0162` is absent and must be recorded as reserved, retired, or accidentally omitted. ECM exposes 244 contiguous IDs.

The AMX commit `a5e473156b2a83fa77e2b1df40056553b4ea29c5` exists in the worker's local repository but is not contained by a configured remote-tracking branch. The artifact digest, not the unpublished commit, is the current reconciliation identity.

## 2. Source-publication corrections

Before the normalized requirement catalogs become machine-scored conformance inputs:

1. Every requirement must carry `extraction_kind`: `source_normative`, `derived_design_obligation`, `acceptance_obligation`, or equivalent.
2. Aggregate requirements must declare `alias_of` or `refines` links to atomic requirements.
3. Every row must acquire an exact source-span locator and quotation digest, not only a heading.
4. AMX must explain the missing `AMX-R-0162` identifier.
5. Declared-but-absent schemas and transitions must be marked `declared_unimplemented`.
6. Volatile Git publication metadata must move into a detached attestation.
7. AMX-R-0203 must preserve the source's exact two-implementation/external-vector reproducibility rule.

These corrections do not change either normative source.

## 3. Preliminary difference matrix

| Domain | Classification | Preliminary disposition |
|---|---|---|
| Authority is external to memory/messages | `identical` | Preserve shared invariant; ForgeCore remains sole capability/effect authority. |
| AMX record/event/bundle vs ECM `evidence-memory-v1` | `conflict` | AMX becomes sole canonical durable-memory contract. ECM memory becomes an AMX reference/view with namespaced collaboration annotations. |
| AMX memory event DAG vs ECM workflow event log | `conflict_if_dual_write` | Store AMX events through the durable infrastructure, but retain AMX causal heads as the only memory-domain mutation history. ECM workflow events own collaboration only. |
| ExecPlan task lifecycle vs ECM tasks | `equivalent_name_different_domain` | Rename/separate `ExecPlanStep` from `CollaborationTask`; ECM references but never replaces plan/effect state. |
| ForgeCore effect ledger vs ECM effect lifecycle | `conflict` | ForgeCore owns authoritative effects and receipts. ECM stores requested-effect correlation and read-only projections. |
| ECM cross-prompts, roles, context views, conflict sets | `missing_in_amx` | Keep ECM collaboration contracts; memory-bearing payloads carry AMX IDs, event digests, or bundles. |
| AMX scope/origin/receiver binding | `missing_or_weaker_in_ecm` | Import AMX repository/branch/path/role, purpose, consent, origin trust, and receiver-binding semantics into all memory views. |
| ECM ArtifactRef/EvidenceRef and VerificationFabric binding | `complementary` | Keep ECM evidence contracts; AMX references them immutably and cannot make copied verdicts authoritative. |
| AMX quarantine/admission vs ECM context admission | `complementary_with_gap` | AMX owns durable memory admission. ECM owns ephemeral task-context admission. Neither transition promotes the other implicitly. |
| AMX and ECM promotion lifecycles | `conflict_if_conflated` | Separate memory epistemic/admission state, configuration deployment state, and visibility scope. Cross-project scope always requires current user authorization. |
| AMX deletion/purge vs ECM propagation criteria | `complementary_but_incomplete` | Use AMX deletion boundary and retraction/purge distinction; add non-resurrection epochs, partial-failure states, receipts, and ECM projection propagation. |
| Harness adapters vs memory/protocol adapters | `equivalent_name_different_domain` | Define separate `HarnessAdapter`, `AMXMemoryAdapter`, and protocol gateway interfaces. |
| Adapter version downgrade | `missing` | Add `minimum_reader_version`, `critical_extensions`, signed capability negotiation, and fail-closed degradation. |
| AMX memory evaluation vs ECM system evaluation | `complementary_with_gate_conflict` | Run topology × memory-mode factorial evaluations. ECM cost wins cannot bypass AMX memory safety/effectiveness gates. |
| ECM observability and attestation | `missing_in_amx` | Keep as derived telemetry/provenance; traces are not authority, evidence verdicts, or business state. |
| ECM platform-neutral claim vs AutoDev-specific types | `conflict_of_abstraction` | Split AMCX Core from an AutoDev integration profile. |

## 4. Canonical ownership map

| Datum or lifecycle | Sole canonical owner | Other layers may hold |
|---|---|---|
| Code and reviewed history | Git | Digests and immutable references |
| Development plan and authorized work lifecycle | AutoDev `ExecPlan`/`TaskGraph` | Collaboration mapping references |
| Identity, policy, grants, capabilities | ForgeCore | Decision/receipt references |
| Consequential effects and receipts | ForgeCore effect ledger | Correlation projections |
| Collaboration tasks, attempts, role leases, messages, budgets, scheduling | ECM workflow ledger | Adapter-local caches |
| Durable/reusable development memory, causal heads, bundles, admission state | AMX-1 | ECM views, search indexes, provider projections |
| Verifier outcomes and exact-subject evidence | EvidenceStore/VerificationFabric | Immutable evidence references |
| Prompt/skill/router canary and activation state | ECM promotion service | Candidate artifact references |
| Current toolset learning before explicit migration | `memory/toolsets/patterns.jsonl` | AMX index/export projection |
| Provider execution sessions | Provider adapter | Opaque, non-authoritative handles |
| Telemetry | Observability system | Derived traces only |

## 5. Provisional AMCX-1 structure

- `amcx.core.*`: identifiers, digests, versioning, scopes, critical extensions.
- `amcx.memory.*`: AMX-derived record, event, bundle, causal, quarantine, retention, deletion, migration contracts.
- `amcx.collaboration.*`: ECM-derived tasks, attempts, roles, cross-prompts, context views, conflict sets, cancellation.
- `amcx.evidence.*`: artifacts, exact-subject evidence, evaluation results, attestations.
- `amcx.promotion.*`: separate memory admission, visibility, and configuration deployment decisions.
- `amcx.effect.*`: non-authoritative requests and correlations to ForgeCore-owned decisions/receipts.
- `amcx.adapter.*`: harness, memory, and protocol capability/degradation profiles.
- `amcx.profile.autodev.*`: concrete ExecPlan, ForgeCore, EvidenceStore, SQLite/Room, Android, and Cline mappings.

## 6. Critical blockers before a unified draft

1. A closed legal transition matrix for quarantine, receiver acceptance, verification, supersession, merge, retraction, purge, partial failure, and non-resurrection.
2. One-way ownership mappings among ExecPlan, ECM tasks, ForgeCore effects, and AMX memory.
3. Fail-closed critical-extension and adapter downgrade behavior.
4. Receiver-bound context/memory handles that cannot be replayed across tenant, task, role, lease, or expiry.
5. Machine-evaluable promotion gates with metric, population, comparator, minimum effect, confidence bound, sample floor, budget, and missing-data behavior.
6. A deterministic verifier-independence profile by risk class.
7. Hierarchical non-expandable budgets with reserve-on-spawn and aggregate descendant/concurrency limits.
8. Immutable publication of the exact unified source and future complete machine-readable schemas.

## 7. Minimum falsification suite

1. AMX → ECM/AMCX view → AMX round trip preserves logical IDs, event digests, heads, critical fields, unknown extensions, retractions, and deletion semantics.
2. Kotlin and Rust canonicalization yield identical IDs/digests for valid fixtures and identical failures for invalid fixtures.
3. Duplicate/reordered imports and crash replay yield one logical memory transition and at most one ForgeCore effect.
4. Concurrent memory heads remain visible until an evidence-citing merge includes every resolved parent.
5. Provider-authored `verified` or `active` state cannot escape receiver quarantine.
6. Peer content, signatures, memories, and forged approvals mint zero capabilities and cause zero unauthorized effects.
7. Cross-tenant/project/repository/branch/path/private-attempt retrieval leakage is zero over the declared adversarial suite.
8. Adapter downgrade with unknown critical fields fails closed or remains read-only passthrough.
9. Cross-project visibility fails without an authenticated current user grant, regardless of test or consensus outcome.
10. Purge removes content across the declared boundary and pre-purge bundles cannot resurrect it.
11. Task-context entries cannot become durable AMX memory without a separate authorized promotion decision.
12. Two independent promotion-gate implementations produce byte-identical decisions from the same evaluation record.
13. The maximum legal agent tree must reserve aggregate budget before spawn and remain within run caps.
14. Adapter substitution cannot change ForgeCore authorization decisions.

## 8. Next cross-prompt for the AMX worker

```text
Round 0 is independently verified. The following immutable artifacts are now available:

- AMX-1 source SHA-256: 4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9
- AMX normalization SHA-256: c81aedb9528df2162e5c327f6479a89848e70bf85a3835b3d76b67e5b06dae52
- ECM source SHA-256: e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980
- ECM normalization SHA-256: 9ddf7754d017384f4d26ef801eac333a8e2a4148ef3d276fd178a032c49c7810

Do not implement and do not revise AMX-1 or ECM. Independently compare both source artifacts and both normalization envelopes. Produce a complete DifferenceRecord matrix with classifications: identical, equivalent_rename, complementary, conflict, missing, unsupported_or_unevidenced.

For every conflict, cite AMX-R and ECM-R IDs; explain the failure caused by selecting either side; propose a falsifiable test; recommend keep, merge, replace, defer, or reject. Explicitly test these suspected conflicts without assuming the proposed answer is correct:

1. AMX record/event/bundle versus ECM evidence-memory-v1.
2. AMX memory event DAG versus ECM workflow event log.
3. ExecPlan step/effect state versus ECM collaboration task/effect state.
4. Memory verification/admission versus configuration promotion versus visibility scope.
5. Cross-project promotion without explicit user authorization.
6. Quarantine release and producer-asserted trust.
7. Purge, partial failure, deletion receipts, and pre-delete bundle resurrection.
8. Unknown security-critical extensions and adapter downgrade.
9. Receiver/context replay across tenant, task, role, lease, or expiry.
10. Statistical promotion gates, verifier independence, and aggregate agent budgets.

Audit the AMX normalization for modality inflation, aggregate/atomic duplicates, heading-only traceability, AMX-R-0162, AMX-R-0203 precision, and declared-but-unimplemented contracts. Apply the same standard to the ECM normalization.

Return coverage counts for all 243 AMX IDs and all 244 ECM IDs. Do not collapse multiple IDs merely because they are duplicates; record alias/refinement relationships. End with your proposed sole canonical owner for every state-bearing domain and list every unresolved critical blocker. Return no implementation code.
```
