# ADR-004: Canonical Source-of-Truth, Ownership, and Trust-Boundary Map

## Status
Accepted

## Context
In multi-agent and cross-LLM autonomous engineering systems (AMCX-1 / AutoDev), conflating execution authority with untrusted model generation, memory content, or agent messages creates privilege escalation, prompt injection vulnerabilities, and non-deterministic state corruption. A rigorous, machine-verifiable source-of-truth and trust-boundary map is required to guarantee the core invariant:
`untrusted intent -> trusted authorization -> confined execution -> independently verified evidence -> bounded replanning`.

## Decision
We establish 18 immutable domain boundaries defining the canonical history/representation, decision authority, and execution/materialization authority. No agent, prompt, memory entry, or adapter may usurp these authorities.

### The 18 Canonical Domain Ownership Rows (§5 Map)

| # | Domain | Canonical history/representation | Decision authority | Execution/materialization |
|---|---|---|---|---|
| 1 | Plan and step lifecycle | ExecPlan | AutoDev plan reducer/policy | AutoDev orchestrator |
| 2 | Collaboration | ECM event log | ECM reducer and role policy | ECM orchestrator/adapters |
| 3 | Portable memory | AMX event DAG and bundles | AMX validates grammar only | AMX store/projections |
| 4 | Origin/receiver identity | Attestation reference | Authenticated host/transport | Identity/attestation store |
| 5 | Evidence verdict/freshness | EvidenceStore/VerificationFabric | Independent verifier | Evidence store |
| 6 | Quarantine restriction | AMX event/state | Deterministic AMX restriction | AMX reducer |
| 7 | Release/trust/visibility widening | AMX records result | External memory-governance policy | AMX reducer/projections |
| 8 | Retraction suppression barriers | Memory Governance Ledger | External memory-governance policy | Ledger plus AMX commit coordinator |
| 9 | Cross-project grant | Approval record | Scoped user/host approval | Memory-governance service |
| 10 | Effective retrieval | Current decision | Host/ForgeCore policy intersected with AMX state | Retrieval/context service |
| 11 | Effects and receipts | ForgeCore ledger | ForgeCore/host policy | Trusted executor |
| 12 | ContextView history | ECM artifact/workflow | ECM admission plus current policy | ECM context service/CAS |
| 13 | Hard purge | External deletion ledger | Authorized retention/privacy policy | Deletion coordinator/adapters |
| 14 | Prompt/skill/router activation | ECM promotion log | Trusted deployment/approval authority structurally separate from content-producing agents | Configuration deployment service |
| 15 | GateProfile publication/status | Reviewed Evaluation Policy Registry in Git | Authorized evaluation-policy maintainers, separate from candidate producers/evaluators | Gate validators consume exact active digest |
| 16 | Contract activation | Neutral Contract Registry | Repository review/ADR and authorized maintainers | Validators/adapters |
| 17 | Artifact bytes | CAS | Owning domain’s retention policy | Artifact service |
| 18 | Aggregate budgets | ECM budget ledger | ECM orchestrator/policy | Scheduler/adapters |

## Consequences
- Every state-changing operation MUST derive authority from the specific domain authority identified above.
- Model outputs, prompts, peer messages, and memory records are classified strictly as untrusted evidence.
- Schema activation is exclusively owned by the Git-backed Neutral Contract Registry (Domain #16).
- Provenance test suites verify full adherence across all 18 domains without omissions.
