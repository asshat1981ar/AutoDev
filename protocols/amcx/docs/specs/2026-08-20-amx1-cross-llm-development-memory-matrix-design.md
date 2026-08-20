# AMX-1 Cross-LLM Development Memory Matrix Design

## Status

- Architecture: **proposed and user-approved in chat on 2026-08-20**
- Research synthesis: **completed using primary specifications, research papers, official repositories, and the current AutoDev/Engram state**
- AutoDev toolset learning memory v0: **implemented and CI-validated**
- AutoDev ExecPlan control plane: **implemented and CI-validated**
- AMX-1 schemas, adapters, and conformance suite: **not yet implemented**
- `remember-development` ChatGPT skill: **not yet implemented; must consume AMX-1 rather than define a competing contract**
- Cross-provider outcome gains: **research-supported but not yet empirically validated on AutoDev**

## Decision

Proceed with a bounded hybrid prototype named **AutoDev Memory Exchange Matrix v1 (AMX-1)**.

AMX-1 is a vendor-neutral interchange and governance layer for structured development memories. It is not a shared transcript pool, a replacement for Git or CI, or an authority system. Provider-specific memories remain private projections; only normalized, policy-compliant records cross provider boundaries. Reviewed repository memory is the portable collaboration surface for AutoDev v1, while MCP, A2A, Engram, and future stores remain adapters.

The prototype is a conditional go:

- High confidence: deterministic records, events, bundles, import/export, MCP and A2A projections, and repository collaboration.
- Medium confidence: measurable improvement in repository development decisions.
- Low confidence without stronger controls: automatic semantic merging, global cross-project sharing, and safe consequential use of unverified memories.

## Research Interpretation

The evidence supports structured, verified, bounded memory rather than indiscriminate accumulation:

- Cross-domain coding memory improved average performance across six coding benchmarks, while high-level insights transferred better than raw trajectories and overly specific memories caused negative transfer.
- Repository memory and working-memory mechanisms improved code localization and issue resolution in multiple research systems.
- Shared memory improved multi-agent performance and cost in some evaluations, but undifferentiated pools degraded agents in other studies.
- Memory poisoning persists across sessions and can influence later tool selection or consequential actions. Content filtering and self-reported lineage alone are insufficient.
- Current MCP and A2A specifications provide suitable transports and extension points, but neither defines durable memory semantics.

Most 2026 results are recent preprints or vendor reports. AMX-1 therefore treats cross-LLM benefit, temporal graphs, learned consolidation, and semantic promotion as hypotheses that must pass AutoDev-specific ablations.

## Goals

AMX-1 must:

1. Let heterogeneous development LLMs exchange lossless, typed, provenance-rich memory records.
2. Preserve repository, project, user, task, role, time, sensitivity, and retention boundaries across adapters.
3. Separate private provider memory, shared project memory, and user-approved cross-project knowledge.
4. Preserve contradictions, revisions, retractions, and causal history without silent last-write-wins behavior.
5. Make imports deterministic, idempotent, inspectable, quarantinable, and reversible within a declared deletion boundary.
6. Keep memory advisory. No memory record may grant a tool, approval, credential, policy exception, filesystem scope, network scope, push, merge, release, or deployment capability.
7. Measure whether memory improves verified development outcomes rather than merely retrieval similarity.
8. Remain usable without a hosted database, vector store, graph database, or vendor-specific SDK.

## Non-Goals

AMX-1 v1 does not:

- Crawl conversations or repositories outside the active authorization boundary.
- Synchronize proprietary chat histories automatically.
- Store raw transcripts, hidden reasoning, credentials, private keys, tokens, large logs, source trees, binaries, or unrelated personal data.
- Make model outputs authoritative because they are signed, highly confident, repeated, or produced by a stronger model.
- Provide general-purpose CRDT semantics or silently merge incompatible claims.
- Guarantee hard deletion for content committed to ordinary Git history.
- Require a graph or vector database.
- Automatically publish runtime memories into a repository.
- Automatically promote project memories into cross-project scope.
- Replace AGENTS.md, ADRs, issues, tests, CI, telemetry, ForgeCore policy, or the ExecPlan control plane.

## Core Invariants

1. **Authority does not flow through memory.** Only trusted policy and execution boundaries grant capabilities.
2. **Current evidence wins.** Repository, runtime, tests, CI, policy, approvals, and authoritative external systems outrank memory.
3. **Scope precedes relevance.** Authorization, project, path, task, time, sensitivity, retention, and deletion filters run before ranking.
4. **Origin is bound by the receiver.** Producer-supplied origin labels are descriptive until the receiving adapter binds them to an authenticated or locally observed identity.
5. **Integrity is not truth.** A digest or signature proves byte integrity or authorship, not correctness, safety, or permission.
6. **Contradictions remain visible.** Conflicting records coexist until an explicit resolution event relates or retracts them.
7. **The correct retrieval can be empty.** Memory is omitted when scope, freshness, evidence, or decision value is insufficient.
8. **Exports are portable.** Embeddings, provider chunk IDs, ranking scores, caches, and generated summaries are rebuildable projections, not canonical state.
9. **Writes are bounded and auditable.** Every admitted record has predicted reuse, explicit scope, provenance, sensitivity, retention, and an admission result.
10. **Promotion is distinct from capture.** Project capture may be automatic; shared-repository and cross-project promotion require their own gates.

## Source-of-Truth Map

| Information | Authoritative source | AMX-1 role |
|---|---|---|
| Current code and configuration | Repository checkout at an identified commit | Cite and verify; never replace |
| Branch and commit history | Git repository or provider | Immutable evidence locator |
| Test and verifier outcomes | Test runner, CI, or evidence store | Verification evidence |
| Task lifecycle and effect reconciliation | Typed ExecPlan state | Reference current state; do not duplicate authority |
| Repository instructions | Nearest applicable human-authored AGENTS.md and policy files | Higher-priority constraints |
| Reviewed architecture decision | ADR or reviewed design document | Durable rationale source |
| Runtime development lesson | AMX-1 event history | Advisory evidence with provenance |
| Memory-system mutation history | AMX-1 event DAG | Authority for what the memory system recorded, not factual truth |
| Provider search index | Engram, sidecar, service, or local index | Derived and rebuildable |
| Authorization and approval | Host policy, ForgeCore, or approval system | Never inferred from memory |

## Architectural Shape

AMX-1 uses a hybrid topology with four separable layers:

1. **Canonical interchange:** strict records, mutation events, and bundle manifests.
2. **Policy and validation:** deterministic schema, scope, provenance, sensitivity, origin, and lifecycle checks.
3. **Provider projections:** ChatGPT/Engram, MCP, A2A, JSON sequence, local sidecar, or future service mappings.
4. **Repository collaboration:** reviewed, non-sensitive shared memories and schemas that ordinary coding agents can inspect without a provider integration.

The canonical event history is append-only during normal lifecycle operations. An authorized hard-delete purge may physically remove content inside the declared deletion boundary and retain only a non-content deletion receipt when policy permits. Current-state matrices, search indexes, embeddings, graph projections, summaries, and provider conversations are derived views.

## Trust Zones and Actors

| Actor or zone | May propose | May verify | May promote | May execute |
|---|---:|---:|---:|---:|
| Untrusted external content | No direct write; observation only | No | No | No |
| Development LLM | Project candidate | Evidence-backed candidate verification proposal | No unilateral promotion | No authority from memory |
| Host adapter | Bind observed origin; validate and quarantine | Deterministic checks | Only per configured policy | No execution authority |
| Independent verifier or CI | Emit verifier result | Yes, within named check | No | Executes only through its existing boundary |
| User | Any authorized scope | Yes | Yes | Through ordinary host approvals |
| Repository reviewer | Shared project record | Review evidence | Accept into reviewed repository memory | Through ordinary repository permissions |
| ForgeCore or trusted policy boundary | Audit evidence only | Policy/authorization evidence | Not a memory promotion role | Sole trusted execution boundary where applicable |

## Memory Matrix

The matrix is a sparse typed relation, not a dense spreadsheet. Each record occupies one row. Standard axes make records comparable across hosts, while relation edges preserve graph-like structure without requiring a graph database.

| Axis | Required meaning |
|---|---|
| Identity | Stable logical record ID, schema ID, revision and canonical digest |
| Scope | Tenant/user, repository, project, branch, path, task, role and visibility |
| Semantics | Memory kind, decision case, normalized content and applicability |
| Time | Observed, ingested, valid-from, valid-until, expiry and revalidation |
| Evidence | Immutable source references, artifacts, commits, tests and verifier outcomes |
| Origin | Producing host, agent/model/tool, source trust class and receiver binding |
| Trust | Verification state, confidence assertion, quarantine state and corroboration |
| Policy | Sensitivity, allowed purposes, retention, consent and deletion boundary |
| Authority | Advisory influence allowed; explicit prohibited authority uses |
| Relations | Supports, contradicts, supersedes, derives from and duplicates |
| Influence | Retrieval event, decision affected, outcome and later evaluator result |

## Canonical Contracts

AMX-1 defines three JSON Schema 2020-12 documents with immutable absolute `$id` values:

- `development-memory-record-v1`
- `development-memory-event-v1`
- `development-memory-bundle-v1`

Breaking changes use a new major schema ID. Compatible additions use namespaced extensions. Unknown extensions must survive import/export even when a host cannot interpret them.

### Memory record

A record represents one logical memory and its current normalized state. Its stable logical ID is not a content hash; revisions retain the logical ID and produce new event digests.

Required conceptual fields:

```json
{
  "$schema": "https://github.com/asshat1981ar/AutoDev/schemas/amx-1/development-memory-record-v1.schema.json",
  "record_id": "urn:uuid:0198d43b-6b45-7c8d-8a79-3df1c6c79e11",
  "kind": "episode",
  "decision_case": "Recover a Rust installation with truncated extracted binaries",
  "coordinates": {
    "repository": "github.com/asshat1981ar/autodev",
    "branch": null,
    "task": "rust-toolchain-recovery",
    "role": "development-agent"
  },
  "scope": {
    "visibility": "project",
    "owner": "user",
    "paths": [],
    "allowed_purposes": ["diagnosis", "planning", "verification-selection"]
  },
  "content": {
    "media_type": "application/json",
    "schema": "urn:autodev:memory-content:repair-lesson:v1",
    "data": {
      "symptom": "rustc could not load libLLVM",
      "root_cause": "large extracted archive members were truncated",
      "solution": "compare declared and extracted sizes, re-extract affected members, reinstall, and verify versions",
      "applies_when": ["an installer succeeds but a large runtime artifact is unreadable"],
      "does_not_apply_when": ["the source archive fails checksum validation"]
    }
  },
  "origin": {
    "producer": "chatgpt",
    "producer_kind": "software-agent",
    "host": "chatgpt-work-mode",
    "source_trust": "authorized-session-observation",
    "receiver_binding": "locally-observed"
  },
  "provenance": {
    "observed_at": "2026-08-20T15:00:00Z",
    "ingested_at": "2026-08-20T16:00:00Z",
    "evidence": [
      {
        "kind": "tool-result",
        "uri": "conversation://authorized-message",
        "digest": null,
        "summary": "declared and installed ELF byte sizes were compared before repair"
      }
    ]
  },
  "validity": {
    "valid_from": "2026-08-20T15:00:00Z",
    "valid_until": null,
    "expires_at": null,
    "revalidate_on": ["toolchain-change", "repository-identity-change"]
  },
  "verification": {
    "state": "verified",
    "method": "executable-check",
    "verifiers": ["rustc-version-check"],
    "verified_at": "2026-08-20T16:05:00Z"
  },
  "policy": {
    "sensitivity": "internal-development",
    "retention": "project-lifetime",
    "delete_after": null,
    "deletion_boundary": ["canonical-store", "indexes", "provider-projection"]
  },
  "authority": {
    "class": "advisory",
    "may_influence": ["diagnosis", "planning", "verification-selection"],
    "must_not": ["grant-capability", "approve-action", "override-policy", "claim-current-truth-without-check"]
  },
  "relations": {
    "supports": [],
    "contradicts": [],
    "supersedes": [],
    "derived_from": [],
    "duplicates": []
  },
  "status": "active",
  "extensions": {}
}
```

### Memory event

An event represents a mutation to one logical record. It uses the CloudEvents 1.0 JSON envelope and an AMX-1 data schema.

Required operations:

- `org.autodev.memory.record.proposed.v1`
- `org.autodev.memory.record.accepted.v1`
- `org.autodev.memory.record.verified.v1`
- `org.autodev.memory.record.superseded.v1`
- `org.autodev.memory.record.retracted.v1`
- `org.autodev.memory.record.merged.v1`

Every state-changing event carries:

- event ID, source, type, subject, time, schema and media type;
- the complete resulting record state for portability;
- causal parent event digests;
- actor and receiver-bound origin context;
- admission, verification, promotion, merge or retraction reason;
- the policy version used for the decision.

RFC 8785 canonical JSON plus SHA-256 produces the event digest. The digest uses an RFC 6920 `ni` URI. Event order in a file or export is not semantic; causal parents define history.

### Bundle manifest

A bundle is the interchange unit for cross-host or cross-LLM transfer. Its manifest contains:

- contract and schema versions;
- canonicalization and hash profile;
- producer and receiver-bound source identity when available;
- export time, scope and filter;
- event count and causal head digests;
- included ancestor boundary and unresolved dependencies;
- redaction summary and declared deletion limitations;
- filenames and media types;
- optional detached signature information.

The normative streaming format is RFC 7464 JSON Text Sequences. A compact LF-delimited JSONL profile is supported for developer ergonomics. Embeddings, provider message IDs, chunk IDs, ranking scores, caches and summaries are excluded unless explicitly requested as non-canonical projections.

## Canonicalization and Identity

- Use UTF-8 and the I-JSON constraints required by RFC 8785.
- Reject duplicate JSON object keys, non-finite numbers and ambiguous numeric values.
- Encode integers outside the interoperable JSON number range as strings under a field-specific schema.
- Normalize timestamps to RFC 3339 UTC with `Z` before hashing.
- Normalize repository identities to a lowercase host plus canonical owner/repository path, removing credentials, query parameters and `.git` suffixes.
- Use UUIDv7 URNs for logical record and event IDs.
- Use JCS/SHA-256 event digests for integrity and causal addressing.
- Do not derive stable logical IDs from mutable record content.
- A digest establishes integrity; an optional signature profile establishes control of a signing key; neither elevates the record's authority.

## Write and Admission Pipeline

1. **Observe:** accept only evidence visible to the authorized session or supplied through an authorized import.
2. **Classify:** resolve decision case, memory kind, matrix coordinates, scope, sensitivity, retention and origin class.
3. **Minimize:** extract the smallest independently understandable record; preserve large evidence by immutable reference.
4. **Redact or reject:** exclude secrets, unrelated sensitive data, raw transcripts, hidden reasoning and executable instructions masquerading as evidence.
5. **Validate:** enforce schema, size, identifier, time, scope, authority and prohibited-content constraints deterministically.
6. **Bind origin:** attach the host-observed identity and trust zone; never rely on producer prose for security decisions.
7. **Quarantine:** imported, external, unverifiable or conflicting records remain quarantined and cannot influence consequential actions.
8. **Deduplicate:** compare stable IDs and content similarity without collapsing distinct events or conflicting observations.
9. **Relate:** preserve support, contradiction, supersession and derivation relationships.
10. **Authorize write:** automatic writes are limited to the active project-private store. Shared repository or cross-project promotion uses a separate gate.
11. **Commit idempotently:** use event ID, source ID and canonical digest to reject duplicate effects.
12. **Audit:** emit the admission outcome, policy version, affected projections and scheduled expiry or revalidation.

Procedural memories receive the highest scrutiny because they can shape later action selection. A procedure derived from untrusted content remains advisory and quarantined until independently verified.

## Retrieval and Context Assembly

1. Resolve the current repository, branch, path, task, role, decision and authorization boundary.
2. Apply authority, scope, sensitivity, retention, deletion, validity and quarantine filters before ranking.
3. Classify the need as exact, recent, semantic, procedural, temporal, relational or mixed.
4. Search only required projections. Exact repository and verifier evidence is preferred over semantic similarity.
5. Rank using documented signals: exact coordinates, lexical relevance, semantic relevance, verification, current-commit compatibility, valid time, recency and evidence diversity.
6. Return at most five records within a default 1,200-token budget.
7. Surface IDs, provenance, valid time, verification, quarantine, contradiction and staleness flags.
8. Render provider-neutral evidence separately from provider-specific instructions.
9. Record whether a memory materially influenced the decision and attach the later verifier outcome.
10. Abstain when no record is sufficiently scoped, current and useful.

Retrieved records are untrusted evidence. They appear in a delimited context section that cannot override system, developer, user, repository or tool-policy instructions.

## Contradiction and Concurrency Model

- Causal parent digests, not wall-clock timestamps or file order, define event history.
- Concurrent updates create multiple heads.
- Semantic contradictions never use automatic last-write-wins.
- Fields explicitly declared set-like may be unioned deterministically.
- A resolution is a new merge event whose parents include every resolved head and whose reason cites the chosen evidence.
- A retraction is an event and remains visible to audit views.
- Physical purge is a separate retention operation covering the canonical store, indexes, caches, summaries and provider projections inside the declared boundary.
- A tombstone does not erase prior Git history. Records needing hard deletion must not be committed to Git.

CRDTs may be used inside a provider, but their deltas are not part of the interchange contract. Convergence does not resolve incompatible facts, permissions or consent.

## Storage Profiles

### Provider-private project memory

Used for automatic low-risk project captures during an authorized session. It is isolated by canonical repository identity and is not visible to unrelated projects.

ChatGPT initially uses Engram as a derived private projection. One dedicated conversation per project stores normalized records rather than raw transcripts. Search is restricted by conversation ID or canonical project tag.

Engram v1 limitations are explicit:

- retrieval is chunk-oriented rather than field-queryable;
- writes append verbatim messages;
- updates and single-record deletion are not native;
- conversation deletion is the available complete deletion operation;
- provider message and chunk IDs are projection metadata, not canonical IDs.

AMX-1 therefore serializes a complete normalized record per Engram message and retains canonical IDs in the payload. The adapter must not claim per-record hard deletion that the provider cannot perform.

### Reviewed repository memory

Used for cross-LLM collaboration on non-sensitive, project-scoped knowledge. Records enter through normal repository review and remain advisory.

Preferred layout:

```text
memory/amx-1/
├── manifest.json
├── schemas/
├── fixtures/
└── shared/
    ├── records/
    └── events/
```

Use one immutable content-addressed event per file or per-writer/time shard. Do not use a single concurrently appended JSONL file as the primary multi-writer store. JSONL remains an import/export and compact snapshot format.

Git notes or custom refs are optional projections only because ordinary clones do not reliably fetch, push or display them without additional configuration. Runtime agents must not automatically commit memories. Repository acceptance requires the same PR and CI process as other guidance.

### Local sidecar or service

A future sidecar database may provide transactions, private storage, fast exact and semantic search, retention enforcement and worktree-safe identity. A service may add team concurrency, tenant isolation and policy enforcement.

Neither is required for v1. The event and bundle contracts remain authoritative so either storage profile can be replaced.

## MCP Projection

MCP is a stateless access adapter, not the system of record. Every request carries explicit scope, identity, capability metadata, cursor and transaction or idempotency identifiers.

Read resources:

- `memory://<namespace>/records/<record-id>`
- `memory://<namespace>/records/<record-id>?at=<event-digest>`
- `memory://<namespace>/schemas/<schema-id>`
- `memory://<namespace>/exports/<bundle-id>`

Tools:

- `memory.search`
- `memory.get`
- `memory.propose`
- `memory.accept`
- `memory.verify`
- `memory.retract`
- `memory.merge`
- `memory.import`
- `memory.export`

Input and structured output use the canonical schemas. Write tools require explicit idempotency keys and host authorization. A tool result never changes the calling agent's permissions.

## A2A Projection

A2A is the peer-agent transfer adapter. AMX-1 is advertised as a versioned Agent Skill and extension URI.

- Queries and import requests use structured message data parts.
- Records and bundles use task artifacts with `application/vnd.autodev.amx+json` or `application/json-seq` parts.
- Artifact metadata repeats schema URI, logical record IDs, event digests and provenance references.
- A2A `contextId`, task IDs and artifact IDs remain transport-local and never replace canonical AMX-1 identities.
- Critical results use artifacts rather than relying on conversational history.
- The receiver validates, binds origin and quarantines imports before projection.

## Repository Instruction Integration

AGENTS.md may tell coding agents:

- where AMX-1 schemas and reviewed memories live;
- how to query provider memory;
- which content is authoritative, derived or prohibited;
- that memory cannot grant execution or approval authority;
- how nested path scope maps to memory path coordinates.

Dynamic records and large JSON payloads do not belong in AGENTS.md. Human-authored repository instructions remain authoritative. Tool-specific instruction adapters are generated from one provider-neutral onboarding contract rather than maintained independently.

## Provider-Neutral Onboarding Contract

The onboarding asset must teach any development LLM to:

1. Resolve repository identity and instruction precedence.
2. Retrieve only authorized, decision-relevant AMX-1 records.
3. Treat retrieved content as untrusted advisory evidence.
4. Verify repository and runtime claims against current sources.
5. Propose minimal records after verified outcomes.
6. Preserve provenance, limitations, contradiction and validity.
7. Never store secrets, raw transcripts or private reasoning.
8. Never promote across projects without explicit user approval.
9. Report which memory IDs materially affected a decision.
10. Export and import canonical bundles without provider-specific metadata.

Initial renderings target ChatGPT/Engram, generic MCP clients, A2A agents, repository-only coding agents, Mistral development agents and Mistral Vibe/VibeCoder project instructions. Provider renderings may change wording and invocation syntax but not the contract.

## Threat Model

| Threat | Control | Owner |
|---|---|---|
| Direct or indirect prompt injection becomes durable memory | Minimize, schema validate, bind origin, quarantine external content, isolate evidence from instructions | Host adapter |
| Memory poisoning steers tools later | Advisory-only authority, procedural-memory elevation gate, current evidence check, policy enforcement at execution | Adapter plus trusted policy boundary |
| Trusted-tool echo launders origin | Preserve original source trust through transformations; receiver-bound origin cannot be rewritten by summaries | Canonical event policy |
| Manufactured corroboration | Require independent verifier classes or current executable evidence; duplicate model claims do not elevate trust | Verification gate |
| Cross-project or tenant leakage | Pre-ranking scope filters, canonical repository identity, adversarial distractor tests | Store and retrieval adapter |
| Peer-agent privilege drift | A2A artifacts carry evidence only; receiving agent retains its original capabilities | A2A adapter and host policy |
| Model or host identity spoofing | Treat self-reported IDs as descriptive; use authenticated transport or receiver observation for security decisions | Transport adapter |
| Stale branch or changed repository state | Commit-aware validity, revalidation triggers, current-checkout verification | Retrieval adapter |
| Negative transfer | Applicability and non-applicability fields, project/domain routing, abstention, verifier outcome logging | Retrieval policy |
| Secret or sensitive-data persistence | Deterministic rejection/redaction, sensitivity classes, non-Git private storage, export filtering | Admission and export policy |
| Deletion gaps | Declared deletion boundary, projection inventory, purge verification, no hard-delete claims for Git | Lifecycle manager |
| Event tampering | JCS/SHA-256 digests, causal parents, optional signatures, import recomputation | Canonical store |
| Conflicting concurrent writers | Multiple heads and explicit merge events; no semantic last-write-wins | Event model |
| Context growth and denial of wallet | Quotas, five-result/1,200-token budget, compact views, admission thresholds | Retrieval and operations policy |
| Audit evasion | Append-only admission/influence/retraction events and verifier references | Audit projection |

## Incident Recovery

For suspected poisoning or scope leakage:

1. Disable affected retrieval projection without deleting canonical evidence.
2. Identify records and descendants by causal and derivation edges.
3. Quarantine affected heads and invalidate derived indexes and summaries.
4. Re-run adversarial queries with the projection disabled.
5. Retract confirmed poisoned records and document the reason.
6. Purge data from stores inside the declared deletion boundary.
7. Rotate compromised signing or transport credentials when applicable.
8. Rebuild projections from accepted canonical events.
9. Record affected decisions and reverify consequential outcomes.
10. Add a regression fixture before re-enabling retrieval.

## Evaluation Research Project

### Decision cases

The initial AutoDev evaluation covers:

1. Rust installation recovery after apparently successful extraction.
2. ExecPlan lifecycle ownership and interrupted-effect reconciliation.
3. Formatting-only CI failure versus semantic implementation failure.
4. Repository invariant retrieval under a stale-branch distractor.
5. A failed tool or workflow strategy that must not be repeated.
6. A correct no-memory case.
7. A cross-project distractor that must never be retrieved.
8. A malicious imported procedure attempting to expand tool authority.
9. Contradictory records with different valid times and evidence.
10. Retraction, deletion and clean re-import across projections.

Split cases by repository or time so near-duplicate memories do not leak into the test set.

### Baselines

Compare:

1. No persistent memory.
2. Recent context plus exact repository lookup.
3. ChatGPT/Engram project-private memory.
4. Reviewed repository memory only.
5. AMX-1 hybrid retrieval.

Ablate semantic retrieval, verification weighting, branch compatibility, contradiction surfacing, origin quarantine, provider-specific rendering and repository projection individually.

### Metrics

Retrieval:

- Recall@5, nDCG@5 and reciprocal rank for required evidence.
- Forbidden retrieval rate for wrong-project, expired, deleted, retracted or quarantined records.
- Temporal precision and contradiction coverage.
- Duplicate rate and evidence diversity.

Decision quality:

- Test-verified task success.
- Unsupported-claim and unsafe-action rates.
- Repeated-failure rate after a verified lesson.
- Recovery completion after interruption.
- Negative-transfer rate.
- Correct abstention rate.

Interoperability and lifecycle:

- Schema conformance and canonical digest equality across implementations.
- Export/import/export round-trip preservation of logical IDs, event heads, unknown extensions and retractions.
- Accepted, duplicate, quarantined and rejected import classifications.
- Deletion completeness across declared projections.
- Provenance completeness and correction propagation time.

Operations:

- p50, p95 and p99 read/write latency.
- Tokens added per decision and total inference cost.
- Store growth and index rebuild time.
- Error and timeout behavior under unavailable provider projections.

### Promotion gates

Before AMX-1 is treated as an effective AutoDev memory system:

- 100% valid fixtures preserve canonical digests and causal heads through round-trip interchange.
- 100% invalid, tampered and ID-collision fixtures fail or quarantine with a deterministic reason.
- Zero cross-project, deleted, retracted, expired or unauthorized records are returned in the adversarial set.
- Zero memory record expands authority or directly authorizes a consequential action.
- Poisoned imported procedures produce zero consequential actions in the bounded threat suite.
- Every retrieved record includes usable provenance, validity and verification state.
- The hybrid system does not regress unsafe-action or unsupported-claim rates versus the exact-lookup baseline.
- Target: at least a three-percentage-point absolute improvement in verified decision success over the simple baseline; otherwise retain AMX-1 as interchange only.
- Default retrieval remains at most five records and 1,200 tokens.
- Local p95 retrieval remains within 500 milliseconds for the v1 fixture corpus.
- Deletion is complete for every component named in the declared deletion boundary.

Stochastic decision evaluations use repeated runs and report confidence intervals or observed variation. Untested metrics are labeled rather than inferred.

## Implementation Decomposition

AMX-1 changes the existing `remember-development` implementation order:

1. Contract schemas and deterministic canonical validator.
2. Event hashing, causal validation, quarantine and transactional bundle import/export.
3. Conformance, poisoning, scope, contradiction, deletion and round-trip fixtures.
4. Provider-neutral skill and onboarding contract.
5. ChatGPT/Engram private projection.
6. Reviewed AutoDev repository projection.
7. MCP and A2A contract documentation and adapter fixtures.
8. Mistral and Mistral Vibe/VibeCoder onboarding renderings.
9. AutoDev ablation harness and feasibility report.
10. Only then: automatic project capture and bounded retrieval during real development work.

The original plan must be replaced or substantially revised before execution. Its direct Engram-first implementation would otherwise harden a provider-specific record representation before the portable contract and quarantine boundary exist.

## Rollout Phases

1. **Contract phase:** implement schemas, canonical vectors, bundle tooling and adversarial fixtures. Automatic capture and retrieval remain disabled.
2. **Shadow phase:** create candidates and retrieval results without injecting them into decisions; measure admission, scope, latency and poisoning behavior.
3. **Advisory phase:** enable bounded project retrieval for low-risk development decisions while recording influence and verifier outcomes. Imported and shared records remain quarantined unless accepted.
4. **Project-capture phase:** enable automatic project-private capture only after interoperability and security gates pass.
5. **Shared-repository phase:** promote selected non-sensitive project memories through ordinary PR review.
6. **Cross-project phase:** permit only user-approved normalized promotions; no autonomous global sharing.

Failure at a phase gate leaves later phases disabled. Interchange tooling remains useful even if automatic capture or retrieval never earns promotion.

## Migration and Exit

- The canonical migration unit is a full AMX-1 bundle with schemas, events, causal heads and manifest.
- Provider embeddings, chunk IDs, scores, cache entries and internal conversation IDs are never required for migration.
- Legacy or pre-AMX normalized memory records are imported as proposed events after schema mapping, origin binding and quarantine.
- Engram conversations remain deletable provider projections; exports preserve canonical IDs embedded in record payloads.
- A provider may be removed by exporting accepted events, verifying digests, rebuilding another projection and deleting the old projection within its supported boundary.
- A future AMX major version supplies explicit migration fixtures and never silently reinterprets an old schema.
- If outcome evaluation fails, retain the schemas and interchange tools while disabling automatic retrieval and capture.

## Risks and Unresolved Questions

- Strong cross-vendor longitudinal evidence remains limited; v1 may improve portability more than task success.
- Current Engram operations do not expose field-level queries, mutation or record-level deletion.
- Signatures and origin binding add operational complexity and still do not establish truth.
- Repository memory creates review and history growth; only high-value, non-sensitive memories belong there.
- A three-point task-success target may require a larger corpus than AutoDev initially has; early results must report uncertainty.
- Semantic retrieval can select superficially similar but harmful guidance. Exact scope and verifier evidence remain primary.
- Graph storage may help temporal contradictions later, but v1 relationship arrays and event parents must be evaluated first.
- Mistral, ChatGPT and other hosts expose different memory/tool capabilities; adapters must report unsupported operations rather than emulate guarantees they cannot provide.

## Acceptance Criteria

The AMX-1 design is implemented when:

- the three canonical schemas and examples exist and validate deterministically;
- canonical event hashing and causal validation are reproducible across at least two independent implementations or one implementation plus fixed external vectors;
- bundle import/export is transactional, idempotent and round-trip tested;
- scope, origin, authority, quarantine, contradiction, retraction and deletion rules have adversarial tests;
- ChatGPT/Engram, repository, MCP, A2A and Mistral-family mappings preserve canonical semantics and disclose unsupported guarantees;
- the `remember-development` skill consumes AMX-1 rather than defining separate memory semantics;
- repository memory remains reviewed and runtime memory cannot commit or promote itself;
- the AutoDev evaluation compares all five baselines and publishes measured results;
- automated capture remains disabled until the security, interoperability and outcome gates pass;
- documentation includes recovery, migration, exit and a provider-neutral onboarding prompt.

## Primary References

- Memory Transfer Learning: https://arxiv.org/abs/2604.14004
- Prometheus repository memory: https://arxiv.org/abs/2507.19942
- Improving Code Localization with Repository Memory: https://arxiv.org/abs/2510.01003
- MemCollab cross-agent memory: https://arxiv.org/abs/2603.23234
- LongMemEval-V2: https://arxiv.org/abs/2605.12493
- MPBench memory poisoning: https://arxiv.org/abs/2606.04329
- AgentSys isolation: https://arxiv.org/abs/2602.07398
- Origin-bound memory authority: https://arxiv.org/abs/2606.24322
- MCP 2026-07-28: https://modelcontextprotocol.io/specification/2026-07-28
- A2A specification: https://a2a-protocol.org/latest/specification/
- JSON Schema 2020-12: https://json-schema.org/draft/2020-12
- JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785.html
- Named Information URIs: https://www.rfc-editor.org/rfc/rfc6920.html
- UUIDv7: https://www.rfc-editor.org/rfc/rfc9562.html
- JSON Text Sequences: https://www.rfc-editor.org/rfc/rfc7464.html
- W3C PROV-O: https://www.w3.org/TR/prov-o/
- CloudEvents 1.0.2: https://github.com/cloudevents/spec/tree/v1.0.2
- Git notes: https://git-scm.com/docs/git-notes
- AGENTS.md: https://agents.md/
- OWASP AI Agent Security Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
