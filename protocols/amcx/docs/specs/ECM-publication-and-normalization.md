# ECM publication and normalization envelope

Publication date: 2026-08-20  
Reconciliation target: AMX-1  
Change policy: **The ECM source is preserved; this envelope is a non-amending index and normalization.**

## Publication metadata

| Item | Value |
|---|---|
| Normative source | `docs/superpowers/specs/2026-08-20-evidentiary-collaboration-mesh-design.md` |
| Source SHA-256 | `e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980` |
| Source status | Approved design; proposed implementation |
| Source version | `1.0.0-design` |
| Normalization policy | If this envelope conflicts with the source, the source wins. |

ECM is an approved design, not an implemented integrated system. The source includes conceptual contracts, lifecycle models, adapter mappings, acceptance criteria, and a normative controller seed, but it does not include complete machine-readable schemas, total transition tables, conformance fixtures, or an implementation.

## Extraction and modality register

| Requirement range | Extraction kind | Interpretation |
|---|---|---|
| ECM-R-0001–0030 | `source_normative` | The source explicitly uses “must,” “will not,” or named architectural invariants. |
| ECM-R-0031–0198 | `derived_design_obligation` | Declarative architecture and contract choices are normalized as obligations for implementing the approved design; wording may be stronger than the source's prose modality. |
| ECM-R-0199–0210 | `acceptance_obligation` | Direct normalization of the source's production acceptance criteria. |
| ECM-R-0211–0244 | `controller_normative` | Direct normalization of the source-designated normative controller seed. |

This register prevents a derived design obligation from being misrepresented as a verbatim RFC-style `MUST`. A future conformance catalog should add exact source spans, quotation digests, and per-row `alias_of`/`refines` links before these IDs are used for automated compliance scoring.

## Goals and non-goals

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0001 | ECM MUST coordinate heterogeneous agents and LLM harnesses through typed, versioned contracts. | §2 Goals |
| ECM-R-0002 | ECM MUST support private agent work and selectively shared, admission-verified task context. | §2 Goals |
| ECM-R-0003 | ECM MUST preserve facts, failures, artifacts, decisions, contradictions, and procedures with provenance. | §2 Goals |
| ECM-R-0004 | ECM MUST support bounded cross-prompting, adversarial review, debate, hypothesis tournaments, and independent verification. | §2 Goals |
| ECM-R-0005 | ECM MUST recover deterministic coordination state after process, worker, session, or network failure. | §2 Goals |
| ECM-R-0006 | ECM MUST evaluate multi-agent benefit under normalized cost and compute. | §2 Goals |
| ECM-R-0007 | ECM MUST promote memories, prompts, skills, and orchestration strategies only after evidence gates pass. | §2 Goals |
| ECM-R-0008 | Peer messages, memories, prompts, skills, and provider state MUST remain outside the authorization boundary. | §2 Goals |
| ECM-R-0009 | ECM MUST operate locally and on Android/Termux without requiring Docker or a permanent cloud service. | §2 Goals |
| ECM-R-0010 | ECM MUST permit future distributed workers without changing agent-level semantics. | §2 Goals |
| ECM-R-0011 | ECM v1 MUST NOT store private chain-of-thought or hidden model reasoning. | §3 Non-goals |
| ECM-R-0012 | ECM v1 MUST NOT treat raw shared transcripts as durable memory. | §3 Non-goals |
| ECM-R-0013 | Model consensus MUST NOT grant authority or suppress failing evidence. | §3 Non-goals |
| ECM-R-0014 | Agents MUST NOT install or activate generated skills automatically. | §3 Non-goals |
| ECM-R-0015 | ECM MUST NOT replace AutoDev's trusted ForgeCore execution boundary. | §3 Non-goals |
| ECM-R-0016 | ECM v1 MUST NOT require a vector database, graph database, Redis, or distributed consensus. | §3 Non-goals |
| ECM-R-0017 | Durable state MUST NOT be hidden inside MCP, A2A, or provider transport sessions. | §3 Non-goals |
| ECM-R-0018 | Recursive delegation, retries, replanning, and self-modification MUST be bounded. | §3 Non-goals |
| ECM-R-0019 | Agent self-critique and self-reported success MUST NOT count as promotion evidence. | §3 Non-goals |
| ECM-R-0020 | Provider adapters MUST NOT receive policy, promotion, approval, or execution authority. | §3 Non-goals |

## Invariants and threat assumptions

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0021 | Evidence, messages, memories, plans, skills, signatures, votes, evaluations, and provider roles MUST NOT authorize actions. | §4 Invariant 1 |
| ECM-R-0022 | Provenance MUST establish origin and integrity, not correctness. | §4 Invariant 2 |
| ECM-R-0023 | Delegation MUST only attenuate capability, scope, duration, and budget. | §4 Invariant 3 |
| ECM-R-0024 | Medium- and high-risk candidates MUST have proposer/verifier separation. | §4 Invariant 4 |
| ECM-R-0025 | Every consequential operation MUST be an idempotent ledgered effect bound to trusted authorization. | §4 Invariant 5 |
| ECM-R-0026 | Authorization, tenant, project, validity, sensitivity, and deletion filters MUST run before memory relevance ranking. | §4 Invariant 6 |
| ECM-R-0027 | Every context view MUST carry provenance, authority labels, digest, expiry, and token budget. | §4 Invariant 7 |
| ECM-R-0028 | Conflicting claims MUST be linked and surfaced rather than silently overwritten. | §4 Invariant 8 |
| ECM-R-0029 | Retries MUST be bounded and unknown effects MUST be reconciled before replay. | §4 Invariant 9 |
| ECM-R-0030 | Every promoted prompt, skill, routing policy, or memory schema MUST have a version, evidence set, monitoring envelope, and rollback path. | §4 Invariant 10 |

## Terminology, identifiers, and topology

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0031 | A durable orchestrator MUST own task graphs, attempts, budgets, leases, scheduling, and state reduction. | §5 Topology |
| ECM-R-0032 | Agents MUST execute isolated, bounded assignments. | §5 Topology |
| ECM-R-0033 | Shared context MUST admit compact, typed, provenance-rich candidate findings. | §5 Topology |
| ECM-R-0034 | A trusted policy/execution kernel MUST validate capabilities and execute authorized effects. | §5 Topology |
| ECM-R-0035 | Independent verification and promotion services MUST decide acceptance and reuse. | §5 Topology |
| ECM-R-0036 | External agent boundaries SHOULD map ECM tasks to A2A Tasks while retaining both internal and A2A identifiers. | §6.1 A2A |
| ECM-R-0037 | MCP servers MUST remain disposable capability adapters and MUST NOT become systems of record or authorization authorities. | §6.2 MCP |
| ECM-R-0038 | Durable MCP business state MUST use explicit handles. | §6.2 MCP |
| ECM-R-0039 | Internal events SHOULD use a CloudEvents-compatible envelope with ECM extensions where useful. | §6.3 Event layer |
| ECM-R-0040 | ECM MUST NOT duplicate A2A task semantics in a second external protocol. | §6.3 Event layer |
| ECM-R-0041 | W3C Trace Context MUST propagate causality across agent, protocol, model, memory, verifier, and tool boundaries. | §6.4 Observability |
| ECM-R-0042 | OpenTelemetry GenAI conventions MUST be isolated behind a pinned ECM telemetry schema. | §6.4 Observability |
| ECM-R-0043 | Promotion provenance SHOULD use in-toto/SLSA-style bindings to exact artifacts and evaluator inputs. | §6.5 Attestation |
| ECM-R-0044 | Attestation MUST NOT be interpreted as semantic correctness. | §6.5 Attestation |

## Records, messages, tasks, and cross-prompts

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0045 | All peer communication MUST use `collaboration-envelope-v1`. | §7 Collaboration envelope |
| ECM-R-0046 | The collaboration envelope MUST carry the identity, causality, scope, sequence, expiry, classification, context, payload, dedupe, and integrity fields shown in the source. | §7 Collaboration envelope |
| ECM-R-0047 | Capability lease references MUST be lookup references only; effective authority MUST come from authoritative state. | §7 Collaboration envelope |
| ECM-R-0048 | An envelope MUST NOT carry a self-asserted grant. | §7 Collaboration envelope |
| ECM-R-0049 | The initial payload union MUST cover delegation, coordination, collaboration, development, verification, and learning payloads listed by the source. | §7.1 Payload union |
| ECM-R-0050 | Unknown security-critical payload types or fields MUST fail closed. | §7.1 Payload union |
| ECM-R-0051 | Duplicate, reordered, expired, or replayed messages MUST remain auditable and MUST NOT create duplicate logical transitions. | §7.1 Payload union |
| ECM-R-0052 | Task lifecycle MUST support the main and side states declared in §8. | §8 Task lifecycle |
| ECM-R-0053 | A retry MUST create a new immutable `attempt_id` and MUST NOT rewrite a failed attempt. | §8 Task lifecycle |
| ECM-R-0054 | `COMPLETED` MUST mean all declared acceptance and evidence requirements passed, not merely that an agent responded. | §8 Task lifecycle |
| ECM-R-0055 | Roles MUST be represented by expiring, non-transferable `RoleLease` records with the declared fields. | §8 Role lifecycle |
| ECM-R-0056 | The initial role set MUST include planner, scout, researcher, implementer, reviewer, verifier, security adversary, integrator, and observer. | §8 Role lifecycle |
| ECM-R-0057 | The same principal MUST NOT be proposer and verifier for medium- or high-risk promotion. | §8 Role lifecycle |
| ECM-R-0058 | Reviewer correlation by model family or upstream context MUST be recorded. | §8 Role lifecycle |
| ECM-R-0059 | Cross-prompting MUST be typed and budgeted rather than unrestricted prompt forwarding. | §9 Cross-prompting |
| ECM-R-0060 | A cross-prompt request MUST declare objective, requested role, context reference, response schema, token/time budgets, prohibited data, and acceptance conditions. | §9 Cross-prompting |
| ECM-R-0061 | Peer content MUST be placed in a delimited untrusted-evidence frame. | §9 Cross-prompting |
| ECM-R-0062 | Receivers MUST receive only minimum scope-filtered context needed for the decision. | §9 Cross-prompting |
| ECM-R-0063 | A receiver MUST NOT inherit sender authority, private scratch state, or hidden reasoning. | §9 Cross-prompting |
| ECM-R-0064 | ECM SHOULD support the collaboration strategies enumerated in §9, including independent exploration, adversarial review, debate, interface negotiation, and cross-model review. | §9 Cross-prompting |
| ECM-R-0065 | Agents MUST submit compact typed entries to task context instead of broadcasting into every prompt. | §10 Context admission |
| ECM-R-0066 | Task context entries MUST use the initial typed set `FACT`, `FAILURE`, `CONSTRAINT`, `OPEN_QUESTION`, `PATCH_SUMMARY`, `TEST_RESULT`, and `CONTRADICTION`. | §10 Context admission |
| ECM-R-0067 | Admission MUST validate schema, provenance, scope, sensitivity, duplication, validity, and evidence binding. | §10 Context admission |
| ECM-R-0068 | Context subscriptions MUST be restricted to entry types and scopes relevant to the receiving decision. | §10 Context admission |
| ECM-R-0069 | Admission MAY reject, quarantine, deduplicate, or link an entry. | §10 Context admission |
| ECM-R-0070 | Admission MUST NOT grant capabilities, promote procedural memory, or mark tasks verified. | §10 Context admission |

## Memory, lifecycle, and deletion

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0071 | Memory MUST be classified independently by function, visibility, and trust. | §11 Memory Matrix |
| ECM-R-0072 | Function MUST support working, episodic, semantic, temporal, procedural, artifact, and audit classes. | §11 Memory Matrix |
| ECM-R-0073 | Visibility MUST support private attempt, task team, project, organization, and exportable scopes. | §11 Memory Matrix |
| ECM-R-0074 | Trust MUST support candidate, quarantined, verified, superseded, contradicted, expired, and deleted states. | §11 Memory Matrix |
| ECM-R-0075 | Working memory MUST NOT be promoted directly. | §11 Memory Matrix |
| ECM-R-0076 | Episodic promotion MUST require a verified outcome and reuse case. | §11 Memory Matrix |
| ECM-R-0077 | Semantic promotion MUST require corroboration and freshness. | §11 Memory Matrix |
| ECM-R-0078 | Temporal promotion MUST require valid-time and supersession checks. | §11 Memory Matrix |
| ECM-R-0079 | Procedural promotion MUST require held-out evaluation and rollback. | §11 Memory Matrix |
| ECM-R-0080 | Artifact memory MUST be stored by digest reference with integrity and retention checks. | §11 Memory Matrix |
| ECM-R-0081 | Audit memory MUST be immutable and writable only by trusted writers. | §11 Memory Matrix |
| ECM-R-0082 | Evidence memory MUST use a versioned contract carrying the fields shown in `evidence-memory-v1`. | §11.1 Memory contract |
| ECM-R-0083 | Original observations, extracted claims, and derived summaries MUST be separate records. | §11.1 Memory contract |
| ECM-R-0084 | A summary MUST retain links to every source and contradiction. | §11.1 Memory contract |
| ECM-R-0085 | Raw transcripts MUST NOT become long-term memory by default. | §11.1 Memory contract |
| ECM-R-0086 | Memory admission MUST require predicted reuse, explicit scope, acceptable sensitivity, and traceable source. | §11.2 Write path |
| ECM-R-0087 | Memory candidates MUST validate against a versioned schema. | §11.2 Write path |
| ECM-R-0088 | Deduplication MUST preserve distinct events while resolving stable identities and near-duplicates. | §11.2 Write path |
| ECM-R-0089 | Memory writes MUST preserve conflicting claims and valid-time ranges. | §11.2 Write path |
| ECM-R-0090 | Memory writes MUST enforce quotas, payload bounds, retention, and writer authorization. | §11.2 Write path |
| ECM-R-0091 | Large artifacts MUST be stored by immutable digest. | §11.2 Write path |
| ECM-R-0092 | Memory writes MUST emit an audit event and schedule expiry, revalidation, or compaction. | §11.2 Write path |
| ECM-R-0093 | Credentials, tokens, private reasoning, and sensitive personal data MUST be rejected absent a separately governed product requirement. | §11.2 Write path |
| ECM-R-0094 | Retrieval MUST classify queries as exact, recent, semantic, relational, temporal, procedural, or mixed. | §11.3 Retrieval path |
| ECM-R-0095 | Scope, identity, authorization, validity, sensitivity, retention, and deletion filters MUST run before ranking. | §11.3 Retrieval path |
| ECM-R-0096 | Exact lookup, SQLite FTS5, recency, and declared relevance MUST precede optional vector or graph retrieval. | §11.3 Retrieval path |
| ECM-R-0097 | Vector or graph retrieval MUST be added only after measured decision improvement justifies its failure surface. | §11.3 Retrieval path |
| ECM-R-0098 | Retrieval results MUST expose compact IDs, provenance, validity, trust, and contradiction flags. | §11.3 Retrieval path |
| ECM-R-0099 | The system MUST record which retrieved memories influenced the final decision. | §11.3 Retrieval path |
| ECM-R-0100 | Retrieved memory MUST remain labeled evidence and MUST NOT occupy system/developer authority or expand tools. | §11.3 Retrieval path |
| ECM-R-0101 | `memory/toolsets/patterns.jsonl` MUST initially remain AutoDev's reviewable source for `toolset-pattern-v1`. | §11.4 AutoDev compatibility |
| ECM-R-0102 | Any later conversion of toolset records MUST preserve stable IDs, evidence, sample sizes, environment constraints, and validation history. | §11.4 AutoDev compatibility |

## Artifacts, evidence, decisions, and self-improvement

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0103 | `ArtifactRef` MUST carry the identity, URI, digest, media type, size, producer, revision, sensitivity, retention, and creation fields declared by the source. | §12 Artifacts and evidence |
| ECM-R-0104 | `EvidenceRef` MUST bind evidence to an exact subject digest, artifacts, command/request digest, tool/environment, issuer, validity, result schema, verdict, and integrity proof. | §12 Artifacts and evidence |
| ECM-R-0105 | Evidence MUST become stale when its evaluated subject, revision, toolchain, required environment, prompt, skill, or evaluator changes. | §12 Artifacts and evidence |
| ECM-R-0106 | Artifact writes MUST use stage, hash, finalize, transactional-reference commit, and eventual garbage collection. | §12 Artifacts and evidence |
| ECM-R-0107 | Majority vote MUST NOT be treated as truth or authority. | §13 Consensus and conflict |
| ECM-R-0108 | A versioned `DecisionPolicy` MUST declare eligible roles, review counts, independence, evidence kinds, deterministic verifiers, risk, conflict strategy, and human gates. | §13 Consensus and conflict |
| ECM-R-0109 | Decisions MUST prefer deterministic verification, then authoritative policy, then independent evidence, and use model consensus only for subjective choices. | §13 Consensus and conflict |
| ECM-R-0110 | Disagreements MUST be preserved in a `ConflictSet`. | §13 Consensus and conflict |
| ECM-R-0111 | Automatic conflict resolution MUST require a policy-defined deterministic comparator. | §13 Consensus and conflict |
| ECM-R-0112 | Consensus MUST NOT grant capabilities, ignore missing evidence, or approve destructive effects. | §13 Consensus and conflict |
| ECM-R-0113 | Reflection MUST summarize observable outcomes rather than hidden reasoning. | §14 Autonomous reflection |
| ECM-R-0114 | Retrospectives MUST use a versioned record with subject, expected/observed outcome, evidence, decisions, failures, hypotheses, uncertainty, reuse, and experiment fields. | §14 Autonomous reflection |
| ECM-R-0115 | Reflection MAY propose memory, prompt, skill, or orchestration candidates. | §14 Autonomous reflection |
| ECM-R-0116 | Reflection MUST NOT modify authorization, policy, evaluator ownership, audit retention, or promotion thresholds. | §14 Autonomous reflection |
| ECM-R-0117 | Improvement MUST follow retrospective, hypothesis, candidate, sandbox, gate, canary, promotion, monitoring, and rollback stages. | §15 Self-improvement |
| ECM-R-0118 | Every candidate MUST declare baseline, predicted improvement, applicable tasks, evaluation set, evidence, safety/cost/latency budget, evaluator, expiry, and rollback. | §15 Self-improvement |
| ECM-R-0119 | Evaluations MUST compare a capable single agent, equal-compute single agent, deterministic workflow, production multi-agent configuration, and candidate. | §15.1 Baselines |
| ECM-R-0120 | Evaluations MUST report equal-budget and unconstrained results and normalize tokens, calls, wall time, and cost. | §15.1 Baselines |
| ECM-R-0121 | Promotion lifecycle MUST support draft, candidate, quarantine, sandbox verification, canary, project verification, organization verification, rejection, supersession, rollback, and expiry. | §15.2 Promotion lifecycle |
| ECM-R-0122 | High-impact candidates MUST pass rotating hidden suites across at least three evaluation rounds. | §15.2 Promotion lifecycle |
| ECM-R-0123 | A proposer MUST NOT own verification or the promotion decision. | §15.2 Promotion lifecycle |
| ECM-R-0124 | Promotion MUST bind exact candidate, suite, model/profile, repository, policy, and evidence digests. | §15.2 Promotion lifecycle |
| ECM-R-0125 | Promotion gates MUST enforce zero critical policy, tenant, secret, and unauthorized-capability escapes. | §15.3 Promotion gates |
| ECM-R-0126 | Promotion MUST require no evidence or unsupported-claim regression and material outcome improvement or equal quality at lower cost. | §15.3 Promotion gates |
| ECM-R-0127 | Promotion MUST enforce budgets, deterministic recovery, and rehearsed rollback. | §15.3 Promotion gates |
| ECM-R-0128 | A critical candidate MUST NOT reach production without signed provenance, complete lineage, bounded permissions, and tested rollback. | §15.3 Promotion gates |

## Persistence, concurrency, effects, and observability

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0129 | Multi-agent routing MUST be selected only when predicted value exceeds coordination overhead. | §16 Architecture router |
| ECM-R-0130 | Independent regions, competing hypotheses, broad search, independent security verification, and cross-model review SHOULD favor sub-agents. | §16 Architecture router |
| ECM-R-0131 | Sequential coupled loops, shared mutable debugging state, small deterministic changes, and coordination-heavy tool work SHOULD favor one owner. | §16 Architecture router |
| ECM-R-0132 | Routing decisions and outcomes MUST be evaluable records and SHOULD be ablated against declared baseline topologies. | §16 Architecture router |
| ECM-R-0133 | Coordination state MUST be event-sourced through append-only workflow events, transactional inbox/outbox, rebuildable projections, and durable checkpoints. | §17 Durability |
| ECM-R-0134 | SQLite/Room MUST initially use WAL and one logical writer. | §17 Durability |
| ECM-R-0135 | Large artifacts MUST use content-addressed files. | §17 Durability |
| ECM-R-0136 | Remote workers MUST consume envelopes and leases through an API and MUST NOT receive direct database access. | §17 Durability |
| ECM-R-0137 | Effect lifecycle MUST support planned, authorized, started, committed, verified, unknown, compensating, and compensated states. | §17.1 Effect lifecycle |
| ECM-R-0138 | Stable effect identity MUST hash tenant, run, task, attempt, capability, resolved target, and canonical arguments. | §17.1 Effect lifecycle |
| ECM-R-0139 | Read-only retry MUST use bounded backoff. | §17.1 Effect lifecycle |
| ECM-R-0140 | External writes MUST retry only through tool-native idempotency or authoritative reconciliation. | §17.1 Effect lifecycle |
| ECM-R-0141 | Unknown effects MUST NOT retry blindly. | §17.1 Effect lifecycle |
| ECM-R-0142 | Resume MUST revalidate target identity, repository revision, evidence freshness, role lease, capability lease, and approval. | §17.1 Effect lifecycle |
| ECM-R-0143 | Observability MUST record identities and lineage across runs, tasks, attempts, agents, roles, and parent/child relationships. | §18 Observability |
| ECM-R-0144 | Observability MUST record model/provider/adapter/prompt/skill versions and context/memory influence. | §18 Observability |
| ECM-R-0145 | Observability MUST record tool intents, policy, approvals, receipts, usage, artifacts, evidence, conflicts, promotion, rollback, cancellation, and human intervention. | §18 Observability |
| ECM-R-0146 | Prompt content, tool payloads, and model output MUST be opt-in telemetry. | §18 Observability |
| ECM-R-0147 | Hidden reasoning MUST never be recorded. | §18 Observability |
| ECM-R-0148 | Logs MUST be append-only or tamper-evident and outside agent write authority. | §18 Observability |

## Adversarial evaluation and recovery

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0149 | Evaluation MUST test prompt injection through peers, memories, repositories, and tools. | §19 Adversarial evaluation |
| ECM-R-0150 | Evaluation MUST test forged authority claims, cross-scope retrieval, secret exfiltration, and poisoned skills/tools. | §19 Adversarial evaluation |
| ECM-R-0151 | Evaluation MUST test sybil/collusion/echo consensus, circular citations, stale evidence, evaluator tampering, and reward hacking. | §19 Adversarial evaluation |
| ECM-R-0152 | Evaluation MUST test recursive delegation, retry storms, duplicate/reordered messages, worker death, and cancellation races. | §19 Adversarial evaluation |
| ECM-R-0153 | Evaluation MUST test live model/adapter/protocol upgrades and deletion/expiry/correction/rollback propagation. | §19 Adversarial evaluation |
| ECM-R-0154 | Promotion MUST require zero critical escapes across at least 10,000 adversarial episodes and no rotating hidden-suite regression. | §19 Adversarial evaluation |
| ECM-R-0155 | Automatic execution MUST stop for critical authority, tenant, privacy, secret, lineage, control-mutation, budget, delegation, non-progress, denial, stale-evidence, or envelope violations listed in §20. | §20 Circuit breakers |
| ECM-R-0156 | Delegation depth MUST NOT exceed 4 and fan-out MUST NOT exceed 8. | §20 Circuit breakers |
| ECM-R-0157 | Three identical non-progress states or five consecutive policy denials MUST trip a circuit breaker. | §20 Circuit breakers |
| ECM-R-0158 | Recovery MUST cancel descendants, revoke temporary capability, disable writes/promotions, quarantine derived learning, preserve forensic evidence, restore verified configuration, rotate exposed credentials, and retest before re-enable. | §20 Circuit breakers |

## Adapters and degradation

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0159 | Every harness adapter MUST implement capability discovery, session opening, turn execution, cancellation, and result export. | §21 Adapter contract |
| ECM-R-0160 | Turn requests MUST contain a trusted instruction frame, filtered context reference, immutable prompt/skill references, and budgets. | §21 Adapter contract |
| ECM-R-0161 | Harness results MUST normalize text, claims, tool intents, artifacts, usage, and opaque provider metadata. | §21 Adapter contract |
| ECM-R-0162 | Adapters MUST NOT execute privileged tools, mint grants, promote memory, use provider session IDs as identity, or expose hidden reasoning. | §21 Adapter contract |
| ECM-R-0163 | Tool requests MUST become canonical capability requests evaluated by trusted policy/execution. | §21 Adapter contract |
| ECM-R-0164 | The ChatGPT/Codex adapter MUST support role prompts, isolated tasks, bounded follow-ups, MCP, artifact/evidence export, context/memory references, status, cancellation, and normalized results. | §22.1 ChatGPT/Codex |
| ECM-R-0165 | ChatGPT/Codex integration MUST tolerate controller-only durable persistence. | §22.1 ChatGPT/Codex |
| ECM-R-0166 | Agent-tree communication MUST remain an optimization; the ECM event ledger MUST be durable. | §22.1 ChatGPT/Codex |
| ECM-R-0167 | AutoDev/Cline mapping MUST preserve existing authority and verification boundaries. | §22.2 AutoDev/Cline |
| ECM-R-0168 | A2A mapping MUST validate A2A identity/authorization at the gateway and MUST NOT replace ECM leases. | §22.3 A2A |
| ECM-R-0169 | Mistral Agents mapping MUST normalize handoffs, durable workflows, tools, claims, and artifacts. | §22.4 Mistral Agents |
| ECM-R-0170 | Managed Connector credentials MUST remain provider-managed and MUST NOT enter ECM memory. | §22.4 Mistral Agents |
| ECM-R-0171 | Mistral Vibe mapping MUST support the instruction, skill, custom-agent, trust, CLI, MCP, approval, diff, command, usage, and session surfaces listed in §22.5. | §22.5 Mistral Vibe |
| ECM-R-0172 | Vibe approvals MUST be adapter observations, not ForgeCore authorization grants. | §22.5 Mistral Vibe |
| ECM-R-0173 | The Vibe adapter MUST default to explicit review and account for UNIX/Termux constraints and version/session drift. | §22.5 Mistral Vibe |
| ECM-R-0174 | Provider concepts MUST map through the neutral concept matrix in §23 without changing authority semantics. | §23 Mapping matrix |

## Source-of-truth ownership map

| ID | Datum | Canonical owner | Source section |
|---|---|---|---|
| ECM-R-0175 | Source code and history | Git repository | §24 Source-of-truth map |
| ECM-R-0176 | Durable plan lifecycle | AutoDev typed `ExecPlan` state | §24 Source-of-truth map |
| ECM-R-0177 | Tasks, attempts, roles, messages | ECM workflow event log | §24 Source-of-truth map |
| ECM-R-0178 | Authority and capabilities | ForgeCore policy, identity, and grants | §24 Source-of-truth map |
| ECM-R-0179 | Effects and receipts | ForgeCore effect ledger | §24 Source-of-truth map |
| ECM-R-0180 | Large immutable artifacts | Content-addressed artifact store | §24 Source-of-truth map |
| ECM-R-0181 | Verification evidence | EvidenceStore/VerificationFabric | §24 Source-of-truth map |
| ECM-R-0182 | Current toolset learning | `memory/toolsets/patterns.jsonl` | §24 Source-of-truth map |
| ECM-R-0183 | Provider execution state | Opaque adapter session state | §24 Source-of-truth map |
| ECM-R-0184 | MCP servers | No authority unless backed by an explicitly governed store | §24 Source-of-truth map |

## Compatibility, migration, and implementation sequence

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0185 | Implementation MUST begin with neutral schemas for envelopes, memories, artifacts, evidence, promotions, and adapter capabilities. | §25 Implementation program |
| ECM-R-0186 | ForgeCore work MUST cover canonicalization, scope validation, capability lookup, integrity verification, and effect receipts. | §25 Implementation program |
| ECM-R-0187 | Ledger work MUST cover SQLite events, inbox/outbox, reducers, leases, replay, and migrations. | §25 Implementation program |
| ECM-R-0188 | Memory work MUST cover private role memory, context admission, contradiction preservation, and retrieval filters. | §25 Implementation program |
| ECM-R-0189 | Adapter work MUST define a stable interface and conformance kit before provider adapters. | §25 Implementation program |
| ECM-R-0190 | Initial provider sequence MUST cover ChatGPT/Codex, AutoDev/Cline, A2A, Mistral Agents, and Mistral Vibe. | §25 Implementation program |
| ECM-R-0191 | Evaluation work MUST include hidden suites, canary, promotion, and rollback. | §25 Implementation program |
| ECM-R-0192 | Android command surfaces MUST expose topology, provenance, conflicts, evidence, cost, approvals, and rollback. | §25 Implementation program |
| ECM-R-0193 | Every program item MUST be independently reviewable with its own specification, plan, tests, and slice-sized change set. | §25 Implementation program |
| ECM-R-0194 | Structural or trusted-kernel changes MUST require an AutoDev ADR. | §25 Implementation program |
| ECM-R-0195 | The first implementation plan MUST be limited to Neutral Contract Kernel v1. | §25.1 First boundary |
| ECM-R-0196 | The first plan MUST cover schema placement/versioning, neutral records, deterministic canonicalization vectors, and conformance tests. | §25.1 First boundary |
| ECM-R-0197 | The first plan MUST NOT implement execution, database, network, MCP, A2A, provider adapters, or ForgeCore effects. | §25.1 First boundary |
| ECM-R-0198 | Later program items MUST receive separate designs and implementation plans after kernel verification. | §25.1 First boundary |

## Acceptance and evaluation requirements

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0199 | Production ECM MUST prove two heterogeneous adapters can complete a shared task through neutral contracts. | §26 Acceptance 1 |
| ECM-R-0200 | Crash replay MUST reconstruct identical task, message, memory, and promotion state. | §26 Acceptance 2 |
| ECM-R-0201 | Duplicate and reordered messages MUST cause one logical transition. | §26 Acceptance 3 |
| ECM-R-0202 | Worker death at every effect boundary MUST yield one bounded effect or explicit recovery state. | §26 Acceptance 4 |
| ECM-R-0203 | Peer-injected authority claims MUST fail closed. | §26 Acceptance 5 |
| ECM-R-0204 | Private memory MUST NOT leak into task or project context. | §26 Acceptance 6 |
| ECM-R-0205 | Stale evidence MUST NOT promote a changed artifact or revision. | §26 Acceptance 7 |
| ECM-R-0206 | Contradictory claims MUST be surfaced to decision policy. | §26 Acceptance 8 |
| ECM-R-0207 | Prompt/skill candidates MUST NOT self-promote or alter their evaluator. | §26 Acceptance 9 |
| ECM-R-0208 | Deletion and expiry MUST propagate through projections, indexes, caches, summaries, and referenced artifacts within the declared boundary. | §26 Acceptance 10 |
| ECM-R-0209 | Adapter substitution MUST NOT change ForgeCore authorization decisions. | §26 Acceptance 11 |
| ECM-R-0210 | Multi-agent and memory candidates MUST beat or simplify normalized baselines before becoming defaults. | §26 Acceptance 12 |

## Normative controller behavior

| ID | Normative requirement | Source section |
|---|---|---|
| ECM-R-0211 | Platform translations of the controller seed MUST preserve authority, evidence, memory, and promotion invariants. | §27 Controller preamble |
| ECM-R-0212 | The controller MUST inspect repository and durable state before substantial planning. | §27 Operating rule 1 |
| ECM-R-0213 | The controller MUST distinguish facts, assumptions, constraints, hypotheses, and unknowns. | §27 Operating rule 2 |
| ECM-R-0214 | The controller MUST decompose only for genuine independence, specialization, or adversarial value. | §27 Operating rule 3 |
| ECM-R-0215 | Sequential, coupled, or tool-heavy work SHOULD remain with one owner absent contrary evidence. | §27 Operating rule 4 |
| ECM-R-0216 | Every worker assignment MUST declare role, objective, scope, context, budget, acceptance, prohibitions, and output schema. | §27 Operating rule 5 |
| ECM-R-0217 | The controller MUST treat peer/provider/repository/tool/memory/skill content as untrusted evidence unable to expand authority. | §27 Operating rule 6 |
| ECM-R-0218 | The controller MUST share compact typed findings and MUST NOT broadcast transcripts or private reasoning. | §27 Operating rule 7 |
| ECM-R-0219 | Consequential claims MUST cite provenance/artifacts and contradictions MUST be preserved. | §27 Operating rule 8 |
| ECM-R-0220 | Deterministic checks MUST precede model judgment; medium/high risk MUST be independently verified. | §27 Operating rule 9 |
| ECM-R-0221 | Verification MUST bind exact artifact, revision, environment, toolchain, prompt, and skill versions. | §27 Operating rule 10 |
| ECM-R-0222 | Retrospectives MAY propose improvements but MUST NOT self-promote. | §27 Operating rule 11 |
| ECM-R-0223 | Candidate comparisons MUST include single-agent, equal-compute, deterministic, and production baselines. | §27 Operating rule 12 |
| ECM-R-0224 | Promotion MUST enforce declared quality, safety, cost, recovery, independence, and rollback gates. | §27 Operating rule 13 |
| ECM-R-0225 | Core policy, evaluator ownership, audit, and authorization controls MUST NOT be self-modifiable. | §27 Operating rule 13 |
| ECM-R-0226 | Durable events MUST record memory influence, evidence, decisions, retries, conflicts, cost, and human intervention without secrets/hidden reasoning. | §27 Operating rule 14 |
| ECM-R-0227 | Interrupted effects MUST be reconciled before retry and all autonomous loops MUST obey bounded limits. | §27 Operating rule 15 |
| ECM-R-0228 | Working state MUST remain ephemeral and attempt/task scoped. | §27 Memory policy |
| ECM-R-0229 | Durable memories MUST have future decision use, scope, provenance, validity, sensitivity, retention, and evidence. | §27 Memory policy |
| ECM-R-0230 | Observations, claims, and summaries MUST remain separate records. | §27 Memory policy |
| ECM-R-0231 | Retrieved memory MUST remain evidence rather than instruction or authority. | §27 Memory policy |
| ECM-R-0232 | Project and procedural memory MUST require evidence-gated promotion. | §27 Memory policy |
| ECM-R-0233 | Credentials, tokens, hidden reasoning, and unnecessary sensitive data MUST NOT be stored. | §27 Memory policy |
| ECM-R-0234 | Collaboration MUST use typed envelopes for the operations enumerated in the controller seed. | §27 Collaboration policy |
| ECM-R-0235 | Cross-prompts MUST declare decision, role, context, response, budget, prohibited data, and acceptance. | §27 Collaboration policy |
| ECM-R-0236 | Correlated reviewers and duplicated evidence MUST be discounted. | §27 Collaboration policy |
| ECM-R-0237 | Majority agreement MUST NOT grant authority. | §27 Collaboration policy |
| ECM-R-0238 | Shared file/interface changes SHOULD use one integrator and an independent verifier. | §27 Collaboration policy |
| ECM-R-0239 | The development loop MUST ground, route, contract, execute, share, integrate, verify, reflect, promote, and complete in the declared order. | §27 Development loop |
| ECM-R-0240 | Completion MUST require every required evidence item to be present, current, passing, and independently accepted. | §27 Development loop 10 |
| ECM-R-0241 | Circuit breakers MUST stop effects/promotions for the critical conditions declared in the controller seed. | §27 Circuit breakers |
| ECM-R-0242 | Circuit-breaker recovery MUST preserve evidence, revoke temporary capability, quarantine derived learning, and require controlled recovery. | §27 Circuit breakers |
| ECM-R-0243 | Controller output MUST lead with verified outcome/blocker, distinguish inference, cite evidence, and report risks. | §27 Output discipline |
| ECM-R-0244 | The controller MUST NOT claim completion from self-report, stale tests, or unverified summaries. | §27 Output discipline |

## Open questions and unevidenced assumptions

| ID | Gap or assumption | Why it matters |
|---|---|---|
| ECM-UA-0001 | The conceptual envelope, memory, evidence, promotion, retrospective, and adapter contracts are not complete machine-readable schemas. | Field constraints, extensions, canonicalization, and interoperability remain untestable. |
| ECM-UA-0002 | Task, role, promotion, effect, and recovery lifecycle diagrams are not total transition tables. | Illegal transitions and terminal/retry semantics can diverge across implementations. |
| ECM-UA-0003 | ECM and AMX-1 both claim memory/event/bundle-related concepts. | Without a crosswalk, two canonical owners could emerge. |
| ECM-UA-0004 | The 10,000-episode and three-round thresholds are design defaults without current AutoDev empirical calibration. | They may be too costly or statistically insufficient for some risk classes. |
| ECM-UA-0005 | Reviewer independence is recorded but no normative correlation-scoring algorithm exists. | Same-family or shared-context reviews may be overweighted. |
| ECM-UA-0006 | Target and argument canonicalization lacks cross-language vectors. | Idempotency and capability identity may diverge across Rust, Kotlin, Go, and TypeScript. |
| ECM-UA-0007 | Provider capability loss and downgrade behavior lacks a machine-readable profile. | Adapters could silently discard safety-relevant fields. |
| ECM-UA-0008 | Deletion boundary, purge authorization, partial failure, and proof/receipt formats are incomplete. | Deletion conformance cannot yet be proved. |
| ECM-UA-0009 | Local key ownership and remote federation identity binding are open. | Signatures and receiver binding lack a deployment profile. |
| ECM-UA-0010 | Benchmark tasks and contamination controls are proposed but not assembled. | Multi-agent and memory benefit claims remain unevidenced. |
| ECM-UA-0011 | SQLite one-writer throughput and Android storage behavior are not measured. | Durability architecture may require later scaling changes. |
| ECM-UA-0012 | Adapter behavior under provider compaction, cancellation ambiguity, and non-transactional stores is incomplete. | Replay and atomic import/export may degrade silently. |

## Normalization interpretation

- This envelope assigns 244 stable ECM requirement IDs without changing the ECM source.
- Duplicate requirements in the controller seed remain indexed because the seed is independently normative for adapters.
- `SHOULD` and `MAY` statements preserve the source's weaker obligation; normalization does not strengthen them to `MUST`.
- The source, not this envelope, governs any wording dispute.
- No schema, implementation, adapter, service, fixture, or production behavior is created by this normalization.
