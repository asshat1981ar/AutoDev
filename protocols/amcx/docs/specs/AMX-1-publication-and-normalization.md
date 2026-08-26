# AMX-1 publication and normalization envelope

Publication date: 2026-08-20  
Reconciliation target: Evidentiary Collaboration Mesh (ECM)  
Change policy for this round: **AMX-1 source is preserved verbatim; this envelope is a non-amending index and normalization.**

## Publication metadata

| Item | Value |
|---|---|
| Complete AMX-1 source artifact | `2026-08-20-amx1-cross-llm-development-memory-matrix-design.md` |
| Source SHA-256 | `4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9` |
| Repository | `https://github.com/asshat1981ar/AutoDev.git` |
| Repository path | `docs/superpowers/specs/2026-08-20-amx1-cross-llm-development-memory-matrix-design.md` |
| Source commit | `a5e473156b2a83fa77e2b1df40056553b4ea29c5` |
| Publication status of commit | **Not published to the configured remote.** Local `main` is three commits ahead of `origin/main`; no remote-tracking branch contains the source commit. |
| Remote head observed | `origin/main` at `75051b7` (`feat(forge-core): durable ExecPlan control plane`) |
| Envelope digest | Detached SHA-256 manifest accompanies this file; the digest cannot be embedded in the hashed file without self-reference. |

The complete, normative AMX-1 source is the source artifact above. This envelope does not replace it. If this normalization conflicts with the source, the source wins. “Source section” citations below refer to headings in that source.

## Completeness and non-amendment register

AMX-1 is an approved design, not an implemented protocol. The source itself says that schemas, adapters, and the conformance suite are not yet implemented. Consequently, publication must distinguish declared contracts from artifacts that do not yet exist.

| Requested item | Source-faithful publication status |
|---|---|
| Normative schemas | AMX-1 declares three JSON Schema 2020-12 documents and gives one conceptual record example plus event and manifest field inventories. It does **not** contain the three complete machine-readable schemas, immutable absolute `$id` strings for all three, enumerations, constraints, or validation vectors. No missing schema is invented here. |
| Record lifecycle state machine | Event operations (`proposed`, `accepted`, `verified`, `superseded`, `retracted`, `merged`) and concurrency rules are declared. A complete transition relation, preconditions, terminal-state semantics, and invalid-transition table are **not defined**. |
| Quarantine state machine | Admission, import, retrieval, rollout, and recovery rules refer to quarantine, acceptance, retraction, and purge. A complete quarantine status enum and transition table are **not defined**. |
| Deletion state machine | Retraction, purge, deletion boundary, Git limitations, receipts, and projection cleanup are specified in prose. Exact purge authorization, receipt schema, retry/reconciliation states, and completion proof format are **not defined**. |
| Promotion state machine | Separate gates for project-private capture, repository promotion, and cross-project promotion are specified. Exact promotion record/event schemas and rollback transitions are **not defined**. |
| Rollout state machine | Six ordered rollout phases and the rule that failed gates disable later phases are defined. Gate evaluation persistence and rollback mechanics are **not defined**. |

The state summaries later in this envelope are source-derived inventories, not new transitions.

## Goals/non-goals

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0001 | AMX-1 MUST exchange lossless, typed, provenance-rich development-memory records across heterogeneous LLMs. | Goals |
| AMX-R-0002 | All adapters MUST preserve repository, project, user, task, role, time, sensitivity, and retention boundaries. | Goals |
| AMX-R-0003 | The system MUST separate provider-private memory, shared project memory, and user-approved cross-project knowledge. | Goals |
| AMX-R-0004 | Contradictions, revisions, retractions, and causal history MUST remain explicit; silent last-write-wins is prohibited. | Goals; Core Invariants; Contradiction and Concurrency Model |
| AMX-R-0005 | Imports MUST be deterministic, idempotent, inspectable, quarantinable, and reversible within the declared deletion boundary. | Goals |
| AMX-R-0006 | Memory MUST remain advisory and MUST NOT grant tools, approvals, credentials, exceptions, scopes, pushes, merges, releases, deployments, or any capability. | Goals; Core Invariants |
| AMX-R-0007 | Evaluation MUST measure verified development outcomes, not retrieval similarity alone. | Goals; Evaluation Research Project |
| AMX-R-0008 | AMX-1 MUST remain usable without a hosted database, vector store, graph database, or vendor SDK. | Goals; Non-Goals |
| AMX-R-0009 | AMX-1 MUST NOT crawl conversations or repositories outside the active authorization boundary. | Non-Goals |
| AMX-R-0010 | AMX-1 MUST NOT automatically synchronize proprietary chat histories. | Non-Goals |
| AMX-R-0011 | AMX-1 MUST NOT store raw transcripts, hidden reasoning, credentials, private keys, tokens, large logs, source trees, binaries, or unrelated personal data. | Non-Goals; Write and Admission Pipeline |
| AMX-R-0012 | A model output MUST NOT become authoritative because of signature, asserted confidence, repetition, or model strength. | Non-Goals; Core Invariants |
| AMX-R-0013 | AMX-1 v1 MUST NOT provide general-purpose CRDT semantics or silently merge incompatible claims. | Non-Goals; Contradiction and Concurrency Model |
| AMX-R-0014 | AMX-1 MUST NOT claim hard deletion for content committed to ordinary Git history. | Non-Goals; Contradiction and Concurrency Model |
| AMX-R-0015 | Runtime memories MUST NOT be automatically published or committed to a repository. | Non-Goals; Reviewed repository memory |
| AMX-R-0016 | Project memories MUST NOT be promoted into cross-project scope without explicit user approval. | Non-Goals; Provider-Neutral Onboarding Contract; Rollout Phases |
| AMX-R-0017 | AMX-1 MUST NOT replace AGENTS.md, ADRs, issues, tests, CI, telemetry, ForgeCore policy, or the ExecPlan control plane. | Non-Goals; Source-of-Truth Map |
| AMX-R-0018 | `remember-development` MUST consume AMX-1 rather than define a competing contract. | Status; Acceptance Criteria |

## Invariants/threat assumptions

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0019 | Only trusted policy and execution boundaries MAY grant capabilities; authority MUST NOT flow through memory. | Core Invariants |
| AMX-R-0020 | Current repository, runtime, test, CI, policy, approval, and authoritative external evidence MUST outrank memory. | Core Invariants; Retrieval and Context Assembly |
| AMX-R-0021 | Authorization, scope, path, task, time, sensitivity, retention, deletion, validity, and quarantine filters MUST run before relevance ranking. | Core Invariants; Retrieval and Context Assembly |
| AMX-R-0022 | Producer origin labels MUST remain descriptive until the receiver binds them to authenticated or locally observed identity. | Core Invariants; Write and Admission Pipeline |
| AMX-R-0023 | Digests and signatures MUST be treated as integrity/authorship evidence only, not truth, safety, permission, or authority. | Core Invariants; Canonicalization and Identity |
| AMX-R-0024 | Conflicting records MUST coexist until an explicit resolution event relates, supersedes, merges, or retracts them. | Core Invariants; Contradiction and Concurrency Model |
| AMX-R-0025 | Retrieval MUST abstain when scope, freshness, evidence, or decision value is insufficient. | Core Invariants; Retrieval and Context Assembly |
| AMX-R-0026 | Embeddings, provider chunk/message IDs, scores, caches, and generated summaries MUST remain rebuildable, non-canonical projections. | Core Invariants; Bundle manifest; Migration and Exit |
| AMX-R-0027 | Every admitted record MUST have predicted reuse, explicit scope, provenance, sensitivity, retention, and an admission result. | Core Invariants |
| AMX-R-0028 | Capture and promotion MUST be separate operations with separate gates. | Core Invariants; Write and Admission Pipeline |
| AMX-R-0029 | External content MUST have observation-only access and MUST NOT directly write, verify, promote, or execute. | Trust Zones and Actors |
| AMX-R-0030 | Prompt-injection defense MUST combine minimization, schema validation, receiver-bound origin, quarantine, and separation of evidence from instructions. | Threat Model |
| AMX-R-0031 | Memory-poisoning defense MUST combine advisory authority, heightened procedure gating, current-evidence checks, and enforcement at the trusted execution boundary. | Threat Model |
| AMX-R-0032 | Transformations and summaries MUST preserve original source trust and MUST NOT rewrite receiver-bound origin. | Threat Model |
| AMX-R-0033 | Corroboration MUST use independent verifier classes or current executable evidence; duplicated model claims MUST NOT elevate trust. | Threat Model |
| AMX-R-0034 | Cross-project and cross-tenant isolation MUST use canonical repository identity, pre-ranking scope filters, and adversarial distractor tests. | Threat Model |
| AMX-R-0035 | Peer-agent artifacts MUST carry evidence only; the receiving agent MUST retain its original capabilities. | Threat Model; A2A Projection |
| AMX-R-0036 | Self-reported host/model identity MUST be descriptive only; security decisions MUST use authenticated transport or receiver observation. | Threat Model |
| AMX-R-0037 | Stale repository state MUST be controlled with commit-aware validity, revalidation triggers, and current-checkout verification. | Threat Model |
| AMX-R-0038 | Negative-transfer controls MUST include applicability limits, project/domain routing, abstention, and verifier-outcome logging. | Threat Model |
| AMX-R-0039 | Sensitive-data controls MUST include deterministic reject/redact behavior, sensitivity classes, non-Git private storage, and export filters. | Threat Model |
| AMX-R-0040 | Deletion controls MUST declare boundaries, inventory projections, verify purges, and avoid hard-deletion claims for Git. | Threat Model |
| AMX-R-0041 | Event-tampering controls MUST recompute JCS/SHA-256 digests and validate causal parents; signatures remain optional. | Threat Model |
| AMX-R-0042 | Context-growth controls MUST include quotas, compact views, admission thresholds, and the default five-record/1,200-token budget. | Threat Model; Retrieval and Context Assembly |
| AMX-R-0043 | Admission, influence, retraction, and verifier references MUST remain auditable. | Threat Model; Write and Admission Pipeline |

## Terminology/identifiers

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0044 | The memory matrix MUST be a sparse typed relation with one record per row and explicit relation edges. | Memory Matrix |
| AMX-R-0045 | Each record MUST carry stable logical identity, schema identity, revision, and canonical digest coordinates. | Memory Matrix |
| AMX-R-0046 | Each record MUST carry tenant/user, repository, project, branch, path, task, role, and visibility scope coordinates as applicable. | Memory Matrix |
| AMX-R-0047 | Each record MUST carry kind, decision case, normalized content, and applicability semantics. | Memory Matrix |
| AMX-R-0048 | Each record MUST carry observed/ingested time, validity interval, expiry, and revalidation coordinates as applicable. | Memory Matrix |
| AMX-R-0049 | Each record MUST carry immutable evidence references, including relevant artifacts, commits, tests, and verifier outcomes. | Memory Matrix |
| AMX-R-0050 | Each record MUST carry producer/host origin, source trust class, and receiver binding. | Memory Matrix |
| AMX-R-0051 | Each record MUST expose verification, asserted confidence, quarantine, and corroboration state. | Memory Matrix |
| AMX-R-0052 | Each record MUST carry sensitivity, allowed purposes, retention, consent, and deletion-boundary policy as applicable. | Memory Matrix |
| AMX-R-0053 | Each record MUST identify permitted advisory influence and explicitly prohibited authority uses. | Memory Matrix |
| AMX-R-0054 | Relations MUST represent support, contradiction, supersession, derivation, and duplication. | Memory Matrix |
| AMX-R-0055 | Influence telemetry MUST identify retrieval, affected decision, outcome, and later evaluator result. | Memory Matrix |
| AMX-R-0056 | Logical record and event IDs MUST use UUIDv7 URNs and MUST NOT be derived from mutable content. | Canonicalization and Identity |
| AMX-R-0057 | Repository identity MUST normalize to lowercase host plus canonical owner/repository path with credentials, query parameters, and `.git` suffix removed. | Canonicalization and Identity |
| AMX-R-0058 | Timestamps MUST normalize to RFC 3339 UTC `Z` before hashing. | Canonicalization and Identity |
| AMX-R-0059 | Event digests MUST use RFC 8785 JCS plus SHA-256 and be represented as RFC 6920 `ni` URIs. | Memory event; Canonicalization and Identity |
| AMX-R-0060 | Canonical JSON MUST use UTF-8/I-JSON constraints, reject duplicate keys/non-finite or ambiguous numbers, and encode out-of-range integers as schema-governed strings. | Canonicalization and Identity |

## Source-of-truth map

| ID | Information | Authoritative owner/source | AMX-1 obligation | Source section |
|---|---|---|---|---|
| AMX-R-0061 | Current code/configuration | Repository checkout at identified commit | Cite and verify; MUST NOT replace. | Source-of-Truth Map |
| AMX-R-0062 | Branch/commit history | Git repository or provider | Use as immutable evidence locator. | Source-of-Truth Map |
| AMX-R-0063 | Test/verifier outcomes | Test runner, CI, or evidence store | Reference as verification evidence. | Source-of-Truth Map |
| AMX-R-0064 | Task lifecycle/effect reconciliation | Typed ExecPlan state | Reference current state; MUST NOT duplicate authority. | Source-of-Truth Map |
| AMX-R-0065 | Repository instructions | Nearest applicable human-authored AGENTS.md and policy files | Treat as higher-priority constraints. | Source-of-Truth Map |
| AMX-R-0066 | Reviewed architecture decision | ADR or reviewed design document | Use as durable rationale source. | Source-of-Truth Map |
| AMX-R-0067 | Runtime development lesson | AMX-1 event history | Treat as advisory evidence with provenance. | Source-of-Truth Map |
| AMX-R-0068 | Memory mutation history | AMX-1 event DAG | Authoritative only for what AMX recorded, not factual truth. | Source-of-Truth Map |
| AMX-R-0069 | Search index | Engram, sidecar, service, or local index | Treat as derived and rebuildable. | Source-of-Truth Map |
| AMX-R-0070 | Authorization/approval | Host policy, ForgeCore, or approval system | MUST NEVER infer from memory. | Source-of-Truth Map |

## Records/events

### Declared schema inventory

| Contract | Declared normative basis | Publication status |
|---|---|---|
| `development-memory-record-v1` | JSON Schema 2020-12; conceptual example includes identity, kind, decision case, coordinates, scope, content, origin, provenance, validity, verification, policy, authority, relations, status, extensions. | Complete JSON Schema absent. |
| `development-memory-event-v1` | CloudEvents 1.0 JSON envelope plus AMX data; six required operation types and state-changing-event field inventory. | Complete JSON Schema absent. |
| `development-memory-bundle-v1` | Manifest field inventory; RFC 7464 normative stream and JSONL ergonomic profile. | Complete JSON Schema absent. |

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0071 | AMX-1 MUST define three JSON Schema 2020-12 documents with immutable absolute `$id` values: record v1, event v1, and bundle v1. | Canonical Contracts |
| AMX-R-0072 | Breaking contract changes MUST use a new major schema ID. | Canonical Contracts |
| AMX-R-0073 | Compatible additions MUST use namespaced extensions, and unknown extensions MUST survive import/export. | Canonical Contracts |
| AMX-R-0074 | A record MUST represent one logical memory and its current normalized state. | Memory record |
| AMX-R-0075 | Record revisions MUST retain the logical record ID and produce new event digests. | Memory record |
| AMX-R-0076 | The record contract MUST define the conceptual fields shown in the source example; the example is illustrative until a complete schema exists. | Memory record; Status |
| AMX-R-0077 | Event operations MUST include proposed, accepted, verified, superseded, retracted, and merged v1 types. | Memory event |
| AMX-R-0078 | Every state-changing event MUST carry event ID, source, type, subject, time, schema, and media type. | Memory event |
| AMX-R-0079 | Every state-changing event MUST carry the complete resulting record state. | Memory event |
| AMX-R-0080 | Every state-changing event MUST carry causal parent event digests. | Memory event |
| AMX-R-0081 | Every state-changing event MUST carry actor and receiver-bound origin context. | Memory event |
| AMX-R-0082 | Every state-changing event MUST carry the applicable admission, verification, promotion, merge, or retraction reason. | Memory event |
| AMX-R-0083 | Every state-changing event MUST carry the policy version used for the decision. | Memory event |
| AMX-R-0084 | File/export order MUST NOT determine event semantics; causal parents define history. | Memory event; Contradiction and Concurrency Model |
| AMX-R-0085 | A bundle MUST be the interchange unit for cross-host/cross-LLM transfer. | Bundle manifest |
| AMX-R-0086 | A bundle manifest MUST identify contract/schema versions and canonicalization/hash profile. | Bundle manifest |
| AMX-R-0087 | A bundle manifest MUST identify producer and receiver-bound source identity when available. | Bundle manifest |
| AMX-R-0088 | A bundle manifest MUST identify export time, scope, and filter. | Bundle manifest |
| AMX-R-0089 | A bundle manifest MUST identify event count, causal heads, ancestor boundary, and unresolved dependencies. | Bundle manifest |
| AMX-R-0090 | A bundle manifest MUST identify redactions, deletion limitations, filenames, and media types. | Bundle manifest |
| AMX-R-0091 | A bundle manifest MAY carry detached-signature information. | Bundle manifest |
| AMX-R-0092 | RFC 7464 JSON Text Sequences MUST be the normative streaming format. | Bundle manifest |
| AMX-R-0093 | LF-delimited JSONL MUST be supported as a developer-ergonomic import/export or compact-snapshot profile. | Bundle manifest; Reviewed repository memory |
| AMX-R-0094 | Non-canonical projections MUST be excluded from bundles unless explicitly requested and identified as non-canonical. | Bundle manifest |
| AMX-R-0210 | Admission MUST observe only evidence visible to the authorized session or supplied by an authorized import. | Write and Admission Pipeline, item 1 |
| AMX-R-0211 | Admission MUST classify decision case, memory kind, matrix coordinates, scope, sensitivity, retention, and origin class. | Write and Admission Pipeline, item 2 |
| AMX-R-0212 | Admission MUST minimize to the smallest independently understandable record and preserve large evidence by immutable reference. | Write and Admission Pipeline, item 3 |
| AMX-R-0213 | Admission MUST redact or reject secrets, unrelated sensitive data, raw transcripts, hidden reasoning, and executable instructions masquerading as evidence. | Write and Admission Pipeline, item 4 |
| AMX-R-0214 | Admission MUST deterministically validate schema, size, identifier, time, scope, authority, and prohibited-content constraints. | Write and Admission Pipeline, item 5 |
| AMX-R-0215 | Admission MUST bind host-observed identity and trust zone and MUST NOT rely on producer prose for security decisions. | Write and Admission Pipeline, item 6 |
| AMX-R-0216 | Admission MUST quarantine imported, external, unverifiable, or conflicting records. | Write and Admission Pipeline, item 7 |
| AMX-R-0217 | Admission MUST deduplicate by stable IDs and content similarity without collapsing distinct events or conflicting observations. | Write and Admission Pipeline, item 8 |
| AMX-R-0218 | Admission MUST preserve support, contradiction, supersession, and derivation relations. | Write and Admission Pipeline, item 9 |
| AMX-R-0219 | Admission MUST authorize automatic writes only for the active project-private store and route other promotions through separate gates. | Write and Admission Pipeline, item 10 |
| AMX-R-0220 | Admission MUST commit idempotently using event ID, source ID, and canonical digest to reject duplicate effects. | Write and Admission Pipeline, item 11 |
| AMX-R-0221 | Admission MUST emit admission outcome, policy version, affected projections, and scheduled expiry or revalidation. | Write and Admission Pipeline, item 12 |

## State machines/lifecycle/deletion

### Source-derived lifecycle inventory (non-amending)

| Lifecycle | Source-declared states/actions | Source-declared transition semantics | Undefined in AMX-1 |
|---|---|---|---|
| Record/event | proposed, accepted, verified, superseded, retracted, merged | Mutations are append-only events; causal parents define history; merge cites all resolved heads; retraction stays auditable. | Complete legal transition table, state enum, terminality, rejected/duplicate event shapes. |
| Admission/quarantine | observed, classified, minimized, validated, receiver-bound, quarantined, accepted/rejected/duplicate (evaluation classification) | Imported/external/unverifiable/conflicting data stays quarantined and cannot affect consequential actions; accepted data may project per policy. | Exact status enum, acceptance authority, release conditions, expiry and retry transitions. |
| Deletion | active/retained, retracted, purge requested/authorized (implied), purged, non-content receipt (optional) | Purge covers only declared boundary; Git history is not erased; projection limitations must be disclosed. | Purge request/authorization schema, receipt schema, partial-failure/retry states, proof format. |
| Promotion | project-private capture, repository candidate/reviewed, cross-project approved | Repository promotion uses PR/CI; cross-project promotion requires user approval. | Promotion event type, rollback/demotion transitions, approver identity schema. |
| Rollout | contract, shadow, advisory, project-capture, shared-repository, cross-project | Ordered gates; gate failure keeps later phases disabled. | Persisted gate-decision record and rollback mechanics. |

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0095 | Canonical event history MUST be append-only during normal lifecycle operations. | Architectural Shape |
| AMX-R-0096 | An authorized hard-delete MAY physically remove content only inside the declared deletion boundary. | Architectural Shape |
| AMX-R-0097 | After authorized purge, a non-content deletion receipt MAY remain when policy permits. | Architectural Shape |
| AMX-R-0098 | Imported, external, unverifiable, or conflicting records MUST remain quarantined and MUST NOT influence consequential actions until independently accepted/verified under policy. | Write and Admission Pipeline; Rollout Phases |
| AMX-R-0099 | Procedural memories MUST receive heightened scrutiny; procedures derived from untrusted content MUST remain advisory and quarantined until independently verified. | Write and Admission Pipeline |
| AMX-R-0100 | Write admission MUST perform, in order, authorized observation, classification, minimization, reject/redact, deterministic validation, origin binding, quarantine, deduplication, relation preservation, write authorization, idempotent commit, and audit emission. | Write and Admission Pipeline |
| AMX-R-0101 | Automatic writes MUST be limited to the active project-private store. | Write and Admission Pipeline |
| AMX-R-0102 | Shared-repository and cross-project promotion MUST use separate authorization gates. | Write and Admission Pipeline |
| AMX-R-0103 | Idempotent commit MUST use event ID, source ID, and canonical digest to reject duplicate effects. | Write and Admission Pipeline |
| AMX-R-0104 | Concurrent updates MUST produce multiple causal heads. | Contradiction and Concurrency Model |
| AMX-R-0105 | Only fields explicitly declared set-like MAY be deterministically unioned. | Contradiction and Concurrency Model |
| AMX-R-0106 | A resolution MUST be a new merge event whose parents include every resolved head and whose reason cites chosen evidence. | Contradiction and Concurrency Model |
| AMX-R-0107 | Retraction MUST be an event and MUST remain visible to audit views. | Contradiction and Concurrency Model |
| AMX-R-0108 | Physical purge MUST be a separate retention operation covering every canonical and derived component inside the declared boundary. | Contradiction and Concurrency Model |
| AMX-R-0109 | Records requiring hard deletion MUST NOT be committed to Git. | Contradiction and Concurrency Model |
| AMX-R-0110 | Suspected poisoning/scope leakage recovery MUST disable affected retrieval without deleting canonical evidence, trace descendants, quarantine heads, invalidate derivatives, rerun adversarial queries, retract confirmed poison, purge within boundary, rotate compromised credentials, rebuild from accepted events, reverify affected outcomes, and add a regression fixture before re-enable. | Incident Recovery |
| AMX-R-0111 | Contract phase MUST keep automatic capture and retrieval disabled. | Rollout Phases |
| AMX-R-0112 | Shadow phase MUST NOT inject candidates or retrievals into decisions and MUST measure admission, scope, latency, and poisoning behavior. | Rollout Phases |
| AMX-R-0113 | Advisory phase MAY enable bounded low-risk project retrieval while recording influence/outcomes; imported/shared records remain quarantined unless accepted. | Rollout Phases |
| AMX-R-0114 | Automatic project-private capture MUST remain disabled until interoperability and security gates pass. | Rollout Phases; Acceptance Criteria |
| AMX-R-0115 | Shared-repository promotion MUST use ordinary PR review, and cross-project promotion MUST be user-approved with no autonomous global sharing. | Rollout Phases |
| AMX-R-0116 | Failure at a rollout gate MUST leave later phases disabled. | Rollout Phases |
| AMX-R-0242 | Rollout MUST proceed in order through contract, shadow, advisory, project-capture, shared-repository, and cross-project phases. | Rollout Phases |
| AMX-R-0244 | Interchange tooling MUST remain usable even if automatic capture or retrieval never earns promotion. | Rollout Phases |

## Authority/trust

| Actor/zone | Propose | Verify | Promote | Execute/source-of-authority |
|---|---|---|---|---|
| External content | Observation only; no direct write | No | No | No |
| Development LLM | Project candidate | Evidence-backed proposal only | No unilateral promotion | No authority from memory |
| Host adapter | Bind origin; validate/quarantine | Deterministic checks | Configured policy only | No execution authority |
| Independent verifier/CI | Verifier result | Named check only | No | Existing execution boundary only |
| User | Authorized scope | Yes | Yes | Ordinary host approvals |
| Repository reviewer | Shared-project record | Review evidence | Repository acceptance | Ordinary repository permissions |
| ForgeCore/trusted policy | Audit evidence only | Policy/authorization evidence | Not a memory-promotion role | Sole trusted execution boundary where applicable |

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0117 | A development LLM MAY propose project candidates and evidence-backed verification candidates but MUST NOT unilaterally promote or gain authority from memory. | Trust Zones and Actors |
| AMX-R-0118 | A host adapter MUST bind observed origin, validate, and quarantine; it MAY promote only under configured policy and MUST NOT gain execution authority. | Trust Zones and Actors |
| AMX-R-0119 | An independent verifier/CI MAY verify only within a named check and executes only through its existing boundary. | Trust Zones and Actors |
| AMX-R-0120 | A user MAY propose, verify, and promote only within authorized scope and executes through ordinary approvals. | Trust Zones and Actors |
| AMX-R-0121 | Repository reviewers MAY accept reviewed shared-project memory only through ordinary repository permissions and process. | Trust Zones and Actors; Reviewed repository memory |
| AMX-R-0122 | ForgeCore/trusted policy MUST remain an execution/policy boundary, not a memory-promotion role. | Trust Zones and Actors |
| AMX-R-0123 | Retrieved records MUST be rendered as delimited, untrusted evidence separate from provider instructions and MUST NOT override system, developer, user, repository, or tool-policy instructions. | Retrieval and Context Assembly |
| AMX-R-0124 | Repository instructions MUST remain human-authored and authoritative over dynamic memories. | Repository Instruction Integration |
| AMX-R-0125 | AGENTS.md MUST NOT contain dynamic records or large JSON payloads. | Repository Instruction Integration |
| AMX-R-0126 | Memory-related repository instructions MAY identify locations, query methods, authority classes, prohibited content, and nested-path mapping. | Repository Instruction Integration |

## Persistence/concurrency

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0127 | Canonical records, events, and bundle manifests MUST be separable from policy validation, provider projections, and reviewed repository collaboration. | Architectural Shape |
| AMX-R-0128 | Current-state matrices, indexes, embeddings, graph projections, summaries, and provider conversations MUST be derived views. | Architectural Shape |
| AMX-R-0129 | Provider-private captures MUST be low-risk, authorized-session, project-scoped, and isolated by canonical repository identity. | Provider-private project memory |
| AMX-R-0130 | ChatGPT/Engram MUST initially be treated as a derived private projection with one dedicated conversation per project storing normalized records, not raw transcripts. | Provider-private project memory |
| AMX-R-0131 | Engram search MUST be restricted by conversation ID or canonical project tag. | Provider-private project memory |
| AMX-R-0132 | Each Engram message MUST serialize a complete normalized record and retain canonical IDs in its payload. | Provider-private project memory |
| AMX-R-0133 | The Engram adapter MUST disclose that retrieval is chunk-oriented, writes append, record update/deletion is unavailable, and conversation deletion is the complete supported deletion operation. | Provider-private project memory |
| AMX-R-0134 | Reviewed repository memory MUST contain only non-sensitive, project-scoped, advisory knowledge accepted through normal review. | Reviewed repository memory |
| AMX-R-0135 | Reviewed repository event storage MUST use immutable content-addressed files or per-writer/time shards, not one concurrently appended primary JSONL file. | Reviewed repository memory |
| AMX-R-0136 | Git notes/custom refs MAY be projections only and MUST NOT be required for ordinary interoperability. | Reviewed repository memory |
| AMX-R-0137 | Repository acceptance MUST use the same PR and CI process as other guidance. | Reviewed repository memory |
| AMX-R-0138 | A sidecar/service MAY add transactions, private search, retention, worktree-safe identity, concurrency, tenancy, and policy, but MUST NOT be required for v1. | Local sidecar or service |
| AMX-R-0139 | Event and bundle contracts MUST remain authoritative when replacing storage profiles. | Local sidecar or service |
| AMX-R-0140 | Provider-internal CRDTs MAY be used, but their deltas MUST NOT enter the interchange contract or be treated as resolution of incompatible facts, permissions, or consent. | Contradiction and Concurrency Model |
| AMX-R-0243 | Reviewed repository memory SHOULD use the declared `memory/amx-1` manifest, schema, fixture, record, and event layout. | Reviewed repository memory |

## Transport/protocols

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0141 | MCP MUST be a stateless access adapter, not a system of record. | MCP Projection |
| AMX-R-0142 | Every MCP request MUST carry explicit scope, identity, capability metadata, cursor, and transaction or idempotency identifiers as applicable. | MCP Projection |
| AMX-R-0143 | MCP MUST expose the declared record-at-head, record-at-event, schema, and export resource URI patterns. | MCP Projection |
| AMX-R-0144 | MCP MUST expose the declared search/get/propose/accept/verify/retract/merge/import/export tool surface. | MCP Projection |
| AMX-R-0145 | MCP inputs and structured outputs MUST use canonical schemas. | MCP Projection |
| AMX-R-0146 | MCP write tools MUST require explicit idempotency keys and host authorization. | MCP Projection |
| AMX-R-0147 | An MCP tool result MUST NOT change the caller’s permissions. | MCP Projection |
| AMX-R-0148 | A2A MUST be a peer-agent transfer adapter advertising AMX-1 as a versioned Agent Skill and extension URI. | A2A Projection |
| AMX-R-0149 | A2A queries/import requests MUST use structured data parts. | A2A Projection |
| AMX-R-0150 | A2A records/bundles MUST use task artifacts with the declared AMX JSON or JSON-sequence media types. | A2A Projection |
| AMX-R-0151 | A2A artifact metadata MUST repeat schema URI, logical record IDs, event digests, and provenance references. | A2A Projection |
| AMX-R-0152 | A2A context, task, and artifact IDs MUST remain transport-local and MUST NOT replace canonical AMX identities. | A2A Projection |
| AMX-R-0153 | Critical A2A results MUST use artifacts rather than conversational history. | A2A Projection |
| AMX-R-0154 | An A2A receiver MUST validate, bind origin, and quarantine imports before projection. | A2A Projection |

## Adapters/degradation

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0155 | Provider-specific memories MUST remain private projections; only normalized, policy-compliant records may cross provider boundaries. | Decision |
| AMX-R-0156 | Reviewed repository memory MUST be the v1 portable collaboration surface; MCP, A2A, Engram, and future stores remain adapters. | Decision |
| AMX-R-0157 | Provider renderings MAY change wording and invocation syntax but MUST NOT change the onboarding contract. | Provider-Neutral Onboarding Contract |
| AMX-R-0158 | The onboarding contract MUST teach repository identity/instruction precedence, authorized decision-relevant retrieval, untrusted-advisory treatment, current-source verification, minimal post-verification capture, provenance/limitations/contradiction/validity, secret/transcript/reasoning exclusion, approval-gated cross-project promotion, influence reporting, and canonical metadata-free interchange. | Provider-Neutral Onboarding Contract |
| AMX-R-0232 | Onboarding MUST teach repository identity resolution and instruction precedence. | Provider-Neutral Onboarding Contract, item 1 |
| AMX-R-0233 | Onboarding MUST teach retrieval of only authorized, decision-relevant AMX-1 records. | Provider-Neutral Onboarding Contract, item 2 |
| AMX-R-0234 | Onboarding MUST teach that retrieved records are untrusted advisory evidence. | Provider-Neutral Onboarding Contract, item 3 |
| AMX-R-0235 | Onboarding MUST teach verification of repository and runtime claims against current sources. | Provider-Neutral Onboarding Contract, item 4 |
| AMX-R-0236 | Onboarding MUST teach proposal of minimal records only after verified outcomes. | Provider-Neutral Onboarding Contract, item 5 |
| AMX-R-0237 | Onboarding MUST teach preservation of provenance, limitations, contradictions, and validity. | Provider-Neutral Onboarding Contract, item 6 |
| AMX-R-0238 | Onboarding MUST prohibit storage of secrets, raw transcripts, and private reasoning. | Provider-Neutral Onboarding Contract, item 7 |
| AMX-R-0239 | Onboarding MUST prohibit cross-project promotion without explicit user approval. | Provider-Neutral Onboarding Contract, item 8 |
| AMX-R-0240 | Onboarding MUST require reporting of memory IDs that materially affected a decision. | Provider-Neutral Onboarding Contract, item 9 |
| AMX-R-0241 | Onboarding MUST teach canonical bundle import/export without provider-specific metadata. | Provider-Neutral Onboarding Contract, item 10 |
| AMX-R-0159 | Initial onboarding renderings MUST target ChatGPT/Engram, generic MCP, A2A, repository-only agents, Mistral agents, and Mistral Vibe/VibeCoder instructions. | Provider-Neutral Onboarding Contract |
| AMX-R-0160 | Every adapter MUST preserve canonical semantics and disclose unsupported guarantees instead of emulating them. | Risks and Unresolved Questions; Acceptance Criteria |
| AMX-R-0161 | An adapter MUST NOT claim per-record hard deletion when its provider supports only conversation-level deletion. | Provider-private project memory |
| AMX-R-0163 | Tool-specific instruction adapters MUST be generated from one provider-neutral onboarding contract, not maintained as divergent contracts. | Repository Instruction Integration |

## Observability/evaluation

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0164 | Retrieval MUST resolve current coordinates and authorization, filter before ranking, classify the need, search only required projections, prefer exact/verifier evidence, rank by documented signals, enforce budget, surface trust metadata, separate instructions, log influence/outcome, and abstain when insufficient. | Retrieval and Context Assembly |
| AMX-R-0222 | Retrieval MUST resolve current repository, branch, path, task, role, decision, and authorization boundary. | Retrieval and Context Assembly, item 1 |
| AMX-R-0223 | Retrieval MUST apply authority, scope, sensitivity, retention, deletion, validity, and quarantine filters before ranking. | Retrieval and Context Assembly, item 2 |
| AMX-R-0224 | Retrieval MUST classify the need as exact, recent, semantic, procedural, temporal, relational, or mixed. | Retrieval and Context Assembly, item 3 |
| AMX-R-0225 | Retrieval MUST search only required projections and prefer exact repository/verifier evidence over semantic similarity. | Retrieval and Context Assembly, item 4 |
| AMX-R-0226 | Retrieval MUST rank using documented exact-coordinate, lexical, semantic, verification, commit-compatibility, valid-time, recency, and evidence-diversity signals. | Retrieval and Context Assembly, item 5 |
| AMX-R-0227 | Retrieval MUST return no more than five records within the default 1,200-token budget. | Retrieval and Context Assembly, item 6 |
| AMX-R-0228 | Retrieval MUST surface IDs, provenance, valid time, verification, quarantine, contradiction, and staleness flags. | Retrieval and Context Assembly, item 7 |
| AMX-R-0229 | Retrieval MUST render provider-neutral evidence separately from provider-specific instructions. | Retrieval and Context Assembly, item 8 |
| AMX-R-0230 | Retrieval MUST record material decision influence and attach the later verifier outcome. | Retrieval and Context Assembly, item 9 |
| AMX-R-0231 | Retrieval MUST abstain when no record is sufficiently scoped, current, and useful. | Retrieval and Context Assembly, item 10 |
| AMX-R-0165 | The initial evaluation MUST cover the ten declared decision cases, including no-memory, cross-project distractor, malicious procedure, contradiction, retraction, deletion, and clean re-import. | Decision cases |
| AMX-R-0166 | Evaluation cases MUST be split by repository or time to prevent near-duplicate leakage. | Decision cases |
| AMX-R-0167 | Evaluation MUST compare no memory, recent context plus exact lookup, Engram private memory, reviewed repository memory, and AMX-1 hybrid retrieval. | Baselines |
| AMX-R-0168 | Evaluation MUST individually ablate semantic retrieval, verification weighting, branch compatibility, contradiction surfacing, origin quarantine, provider rendering, and repository projection. | Baselines |
| AMX-R-0169 | Retrieval evaluation MUST report Recall@5, nDCG@5, reciprocal rank, forbidden retrieval, temporal precision, contradiction coverage, duplicate rate, and evidence diversity. | Metrics |
| AMX-R-0170 | Decision evaluation MUST report verified success, unsupported claims, unsafe actions, repeated failures, interrupted recovery, negative transfer, and correct abstention. | Metrics |
| AMX-R-0171 | Interoperability/lifecycle evaluation MUST report schema/digest conformance, round-trip identity/head/extension/retraction preservation, import classifications, deletion completeness, provenance completeness, and correction propagation. | Metrics |
| AMX-R-0172 | Operations evaluation MUST report read/write percentiles, decision-token and inference cost, store growth, rebuild time, and unavailable-projection errors/timeouts. | Metrics |
| AMX-R-0173 | All valid fixtures MUST preserve canonical digests and causal heads through round-trip interchange. | Promotion gates |
| AMX-R-0174 | All invalid, tampered, and ID-collision fixtures MUST fail or quarantine with a deterministic reason. | Promotion gates |
| AMX-R-0175 | Adversarial evaluation MUST return zero cross-project, deleted, retracted, expired, or unauthorized records. | Promotion gates |
| AMX-R-0176 | No memory record MAY expand authority or directly authorize a consequential action. | Promotion gates |
| AMX-R-0177 | Poisoned imported procedures MUST produce zero consequential actions in the bounded threat suite. | Promotion gates |
| AMX-R-0178 | Every retrieved record MUST include usable provenance, validity, and verification state. | Promotion gates |
| AMX-R-0179 | The hybrid system MUST NOT regress unsafe-action or unsupported-claim rates against the exact-lookup baseline. | Promotion gates |
| AMX-R-0180 | AMX-1 SHOULD achieve at least three percentage points absolute verified-success improvement over the simple baseline; otherwise it MUST remain interchange-only. | Promotion gates |
| AMX-R-0181 | Default retrieval MUST return no more than five records and 1,200 tokens. | Retrieval and Context Assembly; Promotion gates |
| AMX-R-0182 | Local p95 retrieval MUST remain within 500 ms for the v1 fixture corpus. | Promotion gates |
| AMX-R-0183 | Deletion MUST complete for every named component in the declared deletion boundary. | Promotion gates |
| AMX-R-0184 | Stochastic evaluations MUST use repeated runs and report confidence intervals or observed variation; untested metrics MUST be labeled. | Promotion gates |
| AMX-R-0185 | The AutoDev evaluation MUST publish measured results for all five baselines. | Acceptance Criteria |

## Compatibility/migration

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0186 | The canonical migration unit MUST be a full AMX-1 bundle containing schemas, events, causal heads, and manifest. | Migration and Exit |
| AMX-R-0187 | Migration MUST NOT require provider embeddings, chunks, scores, caches, or internal conversation IDs. | Migration and Exit |
| AMX-R-0188 | Legacy/pre-AMX normalized records MUST import as proposed events only after schema mapping, receiver-bound origin, and quarantine. | Migration and Exit |
| AMX-R-0189 | Engram exports MUST preserve canonical IDs embedded in record payloads, while conversations remain deletable projections. | Migration and Exit |
| AMX-R-0190 | Provider removal MUST export accepted events, verify digests, rebuild another projection, and delete the old projection within its supported boundary. | Migration and Exit |
| AMX-R-0191 | A future AMX major version MUST supply explicit migration fixtures and MUST NOT silently reinterpret an old schema. | Migration and Exit |
| AMX-R-0192 | If outcome evaluation fails, automatic retrieval and capture MUST be disabled while schemas/interchange MAY remain. | Migration and Exit; Rollout Phases |
| AMX-R-0193 | Interchange MUST preserve logical IDs, causal heads, unknown extensions, and retractions through export/import/export. | Metrics; Acceptance Criteria |

## Implementation sequence

This section records the approved order only; **no implementation is authorized or performed by this publication round.**

| ID | Normative requirement | Source section |
|---|---|---|
| AMX-R-0194 | Work MUST begin with contract schemas and a deterministic canonical validator. | Implementation Decomposition |
| AMX-R-0195 | Event hashing, causal validation, quarantine, and transactional bundle interchange MUST follow the contracts. | Implementation Decomposition |
| AMX-R-0196 | Conformance, poisoning, scope, contradiction, deletion, and round-trip fixtures MUST precede provider integration. | Implementation Decomposition |
| AMX-R-0197 | The provider-neutral skill/onboarding contract MUST precede provider-specific renderings. | Implementation Decomposition |
| AMX-R-0198 | ChatGPT/Engram private projection MUST follow portable contracts and quarantine boundaries. | Implementation Decomposition |
| AMX-R-0199 | Reviewed repository projection MUST follow the private projection in the declared sequence. | Implementation Decomposition |
| AMX-R-0200 | MCP/A2A documentation and fixtures MUST precede Mistral-family renderings. | Implementation Decomposition |
| AMX-R-0201 | The AutoDev ablation harness and feasibility report MUST precede automatic capture and live bounded retrieval. | Implementation Decomposition |
| AMX-R-0202 | The former direct Engram-first plan MUST be replaced or substantially revised before execution. | Implementation Decomposition |
| AMX-R-0203 | AMX-1 is implemented only when complete schemas/examples validate deterministically and hashing/causal validation are independently reproducible or fixed-vector reproducible. | Acceptance Criteria |
| AMX-R-0204 | Bundle import/export MUST be transactional, idempotent, and round-trip tested before acceptance. | Acceptance Criteria |
| AMX-R-0205 | Scope, origin, authority, quarantine, contradiction, retraction, and deletion MUST have adversarial tests. | Acceptance Criteria |
| AMX-R-0206 | ChatGPT/Engram, repository, MCP, A2A, and Mistral mappings MUST preserve canonical semantics and disclose unsupported guarantees. | Acceptance Criteria |
| AMX-R-0207 | Repository memory MUST remain reviewed, and runtime memory MUST NOT commit or promote itself. | Acceptance Criteria |
| AMX-R-0208 | Automatic capture MUST remain disabled until security, interoperability, and outcome gates pass. | Acceptance Criteria |
| AMX-R-0209 | Documentation MUST include recovery, migration, exit, and a provider-neutral onboarding prompt. | Acceptance Criteria |

## Open questions

### Explicit unresolved questions from AMX-1

| ID | Open question / risk | Evidence status | Source section |
|---|---|---|---|
| AMX-UQ-0001 | Will cross-vendor memory improve AutoDev task success rather than mainly portability? | Unevidenced on AutoDev; longitudinal cross-vendor evidence is limited. | Status; Risks and Unresolved Questions |
| AMX-UQ-0002 | Can the three-point improvement target be estimated with the initially available corpus? | Unknown; may be underpowered. | Risks and Unresolved Questions |
| AMX-UQ-0003 | Can semantic retrieval avoid harmful superficial matches at acceptable recall? | Hypothesis; exact scope and verifier evidence remain primary. | Risks and Unresolved Questions |
| AMX-UQ-0004 | Do relationship arrays/event parents suffice before temporal graph storage is considered? | Must be evaluated. | Research Interpretation; Risks and Unresolved Questions |
| AMX-UQ-0005 | How should adapters expose heterogeneous provider capabilities without false guarantees? | Contract direction exists; capability vocabulary is absent. | Risks and Unresolved Questions |
| AMX-UQ-0006 | What is the operational value/cost of signatures and receiver origin binding? | Integrity/authorship value known; truth and operational tradeoff unvalidated. | Risks and Unresolved Questions |
| AMX-UQ-0007 | What review/history-growth threshold admits a repository memory? | “High-value, non-sensitive” is stated but not quantified. | Risks and Unresolved Questions |
| AMX-UQ-0008 | How should Engram field-query, mutation, and record-deletion gaps affect conformance? | Limitations are known; conformance/degradation contract is incomplete. | Risks and Unresolved Questions |

### Unevidenced or under-specified assumptions exposed by normalization

| ID | Assumption/gap | Why it matters |
|---|---|---|
| AMX-UA-0001 | The three declared schemas can be made mutually consistent without changing conceptual fields. | Not testable until schemas and vectors exist. |
| AMX-UA-0002 | UUIDv7 generation, canonical-number handling, URI normalization, and unknown-extension preservation will interoperate across languages. | Requires external vectors or two implementations. |
| AMX-UA-0003 | “Predicted reuse,” “consequential action,” “independent verifier class,” “accepted,” and “current-commit compatible” can be made deterministic enough for conformance. | These terms currently lack normative algorithms/enums. |
| AMX-UA-0004 | Receiver-bound identity can be represented portably across ChatGPT, MCP, A2A, repository, and Mistral hosts. | Authentication and binding profiles are undeclared. |
| AMX-UA-0005 | A complete deletion boundary can be enumerated and verified across caches, summaries, indexes, provider projections, backups, and Git. | Receipt/proof and partial-failure semantics are absent. |
| AMX-UA-0006 | Transactional bundle import can be provided across adapters whose stores are append-only or lack transactions. | Compensation/degradation semantics are absent. |
| AMX-UA-0007 | Five records/1,200 tokens and 500 ms p95 are suitable defaults for real AutoDev corpora. | Targets are declared but not yet empirically justified. |
| AMX-UA-0008 | Ordinary PR review provides adequate memory-governance review quality. | Reviewer rubric and required evidence are undeclared. |
| AMX-UA-0009 | Causal parents alone are enough to reconstruct all relevant mutation history when ancestors are intentionally omitted at bundle boundaries. | Missing-ancestor resolution semantics are undeclared. |
| AMX-UA-0010 | The source’s reference list supports all 2026 research claims with stable accessible artifacts. | This publication did not re-run the literature review; citations are source-preserved, not revalidated here. |

## Harness-drift commands and results

Executed from the repository root `/workspace/scratch/7aa63b16ad22/AutoDev` on 2026-08-20:

```text
$ python scripts/check_harness_drift.py
Harness drift check: PASS
```

Observed exit status: `0`.

Publication-integrity commands and observed results:

```text
$ sha256sum docs/superpowers/specs/2026-08-20-amx1-cross-llm-development-memory-matrix-design.md
4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9  docs/superpowers/specs/2026-08-20-amx1-cross-llm-development-memory-matrix-design.md

$ git rev-parse a5e4731
a5e473156b2a83fa77e2b1df40056553b4ea29c5

$ git branch -r --contains a5e4731
# no output

$ git status --short --branch
## main...origin/main [ahead 3]
```

The harness command validates the repository’s existing harness-drift policy. It is **not** an AMX schema/conformance result; those artifacts do not yet exist.

## Publication interpretation

- The complete AMX-1 source is published unchanged as the separately linked source artifact with the source digest above.
- This envelope assigns stable IDs to the source’s normative obligations and exposes gaps; it does not resolve them.
- No schema, transition, adapter, service, skill, fixture, or implementation code was created in this round.
- No commit was pushed or merged in this round.
- ECM reconciliation can cite `AMX-R-*` IDs, but any proposed change must be recorded as a separate delta rather than silently folded into AMX-1.
