# AMX-1 ↔ ECM Cross-Prompt Reconciliation Plan

**Date:** 2026-08-20  
**Status:** Ready for publication handshake; final reconciliation blocked on the full AMX-1 artifact  
**Target:** One versioned specification, provisionally named **AMCX-1 — Agent Memory & Collaboration Exchange**

## 1. Evidence available now

The shared ChatGPT handoff at `https://chatgpt.com/s/t_6a873e70ec408191b2b2b24614828641` says:

- AMX-1 has been specified, self-reviewed, and verified.
- A short commit identifier `a5e4731` is claimed.
- AMX-1 is intended to own canonical records, events, bundles, quarantine, and cross-provider semantics.
- The ChatGPT memory specification delegates those concerns to AMX-1.
- Harness-drift validation reportedly passes.

The handoff does **not** expose the full AMX-1 specification, schemas, decision record, tests, repository path, or full commit SHA. The short commit is not currently resolvable in the published AutoDev repository, and no AMX-1 artifact was found in the shared file store. This most likely means the other worker's artifact remains in an isolated workspace, but that is an inference rather than a verified fact.

The ECM design is available at:

- `docs/superpowers/specs/2026-08-20-evidentiary-collaboration-mesh-design.md`
- SHA-256: `e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980`

## 2. Provisional relationship between the plans

Based on the handoff, AMX-1 and ECM appear complementary but overlap at several dangerous boundaries.

| Domain | AMX-1 claimed role | ECM role | Proposed AMCX-1 ownership |
|---|---|---|---|
| Canonical memory records | Primary owner | Evidentiary Memory Matrix | `amcx.memory.*`; AMX semantics win if they preserve ECM trust and provenance invariants |
| Events | Canonical events | CloudEvents-compatible collaboration/event log | One base event envelope; typed memory and collaboration payloads |
| Bundles | Canonical bundles | Context views, evidence bundles, artifact references | One content-addressed bundle format with typed manifests |
| Quarantine | Canonical quarantine | Admission verification and trust states | One quarantine state machine with explicit release authority |
| Cross-provider semantics | Primary owner | Adapter layer for ChatGPT/Codex, AutoDev/Cline, A2A, Mistral Agents, Mistral Vibe | One provider-neutral contract plus loss-reporting adapters |
| Orchestration | Not established by available evidence | Durable central coordinator with decentralized agents | `amcx.collaboration.*`, owned by ECM-derived orchestration contracts |
| Effects and authority | Not established | ForgeCore boundary, effect ledger, approval gates | `amcx.effect.*`; memory and content never grant authority |
| Evaluation and promotion | Harness-drift claim only | Evidence-gated retrospective, sandbox eval, canary, rollback | `amcx.evidence.*` and promotion state machine |

The core architectural risk is not terminology. It is creating two canonical envelopes, two quarantine models, or two sources of truth for the same datum. The reconciliation must therefore assign exactly one owner for every state-bearing concept.

## 3. Target structure

Produce a single standard, **AMCX-1**, with separated namespaces rather than a monolithic schema:

- `amcx.core.*` — identifiers, provenance, hashes, clocks, schema versioning.
- `amcx.memory.*` — records, events, bundles, retention, deletion, consolidation.
- `amcx.collaboration.*` — tasks, cross-prompts, responses, handoffs, roles.
- `amcx.evidence.*` — claims, tests, evaluations, attestations, promotion decisions.
- `amcx.effect.*` — requested effects, approval, execution receipts, idempotency.
- `amcx.adapter.*` — provider capability maps, transformations, and explicit loss reports.

AMCX-1 may incorporate AMX-1 as its memory subprotocol, but it must publish one root version, one source-of-truth map, and one compatibility policy.

## 4. Collaboration transport

Separate ChatGPT worker chats cannot directly message each other or share hidden workspace state. Cross-prompting will therefore use a user-relayed, artifact-backed protocol:

1. Every round produces a durable artifact or a complete response suitable for saving as one.
2. Every artifact includes a SHA-256 digest, schema version, author role, evidence references, unresolved questions, and superseded artifact identifiers.
3. Workers exchange only artifact references plus a structured prompt; they do not rely on conversational memory.
4. The final integrator treats worker claims as proposals until independently verified.

## 5. Reconciliation rounds

### Round 0 — Publication handshake

The AMX worker publishes the complete AMX-1 document or returns it as an attachment, together with its full commit SHA if it exists in a repository. The ECM worker republishes the ECM digest and source path. No implementation begins during this round.

**Gate R0:** Both source artifacts are readable, immutable by digest, and sufficiently complete to normalize.

### Round 1 — Independent normalization

Each worker converts its own specification into the same `PlanVector` without seeing the other worker's normalization first. This reduces anchoring and rhetorical advantage.

Required PlanVector sections:

1. goals and non-goals;
2. invariants and threat assumptions;
3. terminology and identifiers;
4. source-of-truth ownership map;
5. record and event schemas;
6. state machines and lifecycle/deletion rules;
7. authority and trust boundaries;
8. persistence and concurrency model;
9. transport and protocol bindings;
10. provider adapters and degradation behavior;
11. observability, evidence, and evaluation;
12. compatibility and migration;
13. implementation sequence;
14. open questions and unevidenced claims.

**Gate R1:** Every normative requirement has a stable requirement ID and source location.

### Round 2 — Bidirectional difference matrix

Each worker independently classifies every paired requirement:

- `identical`
- `equivalent_rename`
- `complementary`
- `conflict`
- `missing`
- `unsupported_or_unevidenced`

Each difference record contains:

```json
{
  "difference_id": "D-0001",
  "domain": "quarantine.release",
  "amx_requirement_ids": ["AMX-R-..."],
  "ecm_requirement_ids": ["ECM-R-..."],
  "classification": "conflict",
  "semantic_difference": "...",
  "failure_if_amx_selected": "...",
  "failure_if_ecm_selected": "...",
  "candidate_resolution": "...",
  "evidence": ["test:...", "artifact:..."],
  "confidence": 0.0,
  "unresolved": true
}
```

**Gate R2:** The union of all requirements is covered; neither worker may mark its own ambiguity as the other plan's defect.

### Round 3 — Adversarial cross-review

- The ECM worker attacks AMX-1 for ambiguous authority, lossy provider translation, deletion/retention failures, quarantine escape, replay/idempotency failures, and unverified canonicalization.
- The AMX worker attacks ECM for unnecessary abstraction, duplicated canonical state, orchestration/memory coupling, unimplementable evidence requirements, excessive latency, and adapter overreach.

Each attack must include a falsifiable scenario and a proposed test. Style critique and unsupported preference do not count.

**Gate R3:** Every high-severity attack is either reproduced, disproved by evidence, or recorded as an explicit risk accepted by the user.

### Round 4 — Synthesis

The integrator resolves conflicts using this precedence order:

1. existing AutoDev and ForgeCore safety/authority invariants;
2. prevention of authority expansion or untrusted memory execution;
3. one canonical owner per state-bearing concept;
4. interoperability and round-trip preservation;
5. reproducible empirical evidence;
6. simpler implementation and migration burden;
7. novelty or elegance.

Every original requirement receives exactly one disposition: `keep`, `merge`, `replace`, `defer`, or `reject`, with rationale and evidence.

**Gate R4:** No unresolved source-of-truth conflict remains.

### Round 5 — Unified draft and migration

Produce:

- AMCX-1 normative specification;
- AMX-1 → AMCX-1 crosswalk;
- ECM → AMCX-1 crosswalk;
- JSON Schemas or equivalent machine-readable contracts;
- state-machine definitions;
- provider capability/loss matrix;
- migration and rollback plan;
- conformance test plan;
- decision log for rejected alternatives.

**Gate R5:** Every field and state transition round-trips or emits an explicit, machine-readable loss report.

### Round 6 — Independent verification

The verifier must attempt to falsify the unified design. Required checks:

- no memory, prompt, bundle, or provider content can grant authority;
- exactly one canonical owner exists for each datum and transition;
- quarantine cannot be bypassed through adapters or replay;
- deletion and retention propagate across projections and bundles;
- idempotent replay produces no duplicate external effect;
- schema downgrade cannot silently discard safety-relevant fields;
- ChatGPT/Codex, AutoDev/Cline, A2A, Mistral Agents, and Mistral Vibe mappings declare unsupported capabilities;
- harness-drift and migration fixtures pass;
- both source specifications have complete disposition coverage.

**Gate R6:** Zero unresolved critical findings and an explicit user approval checkpoint before implementation.

## 6. Exact cross-prompts

### Prompt A — Send first to the AMX-1 worker

```text
We are reconciling your AMX-1 plan with the Evidentiary Collaboration Mesh (ECM) into one optimal, versioned specification. Do not begin implementation and do not silently revise AMX-1 during this round.

First publish the complete AMX-1 source as an attached Markdown file or another artifact accessible from this chat. Include:
1) artifact name and SHA-256;
2) full commit SHA and repository/path if the commit is actually published;
3) all normative schemas and state machines;
4) source-of-truth ownership map;
5) invariants, authority/trust boundaries, lifecycle/deletion rules;
6) quarantine and cross-provider semantics;
7) harness-drift commands and results;
8) unresolved questions and unevidenced assumptions.

Then normalize AMX-1 into a PlanVector with these headings: goals/non-goals; invariants/threat assumptions; terminology/identifiers; source-of-truth map; records/events; state machines/lifecycle/deletion; authority/trust; persistence/concurrency; transport/protocols; adapters/degradation; observability/evaluation; compatibility/migration; implementation sequence; open questions.

Assign stable IDs to every normative requirement (AMX-R-####) and cite its source section. End with the exact text: “AMX-1 publication handshake complete” only if the full artifact is accessible and its digest has been computed. Otherwise state the precise blocker. Return no implementation code.
```

### Prompt B — Send to the ECM worker after AMX-1 is accessible

```text
Independently normalize the ECM specification into the agreed PlanVector. Assign stable IDs ECM-R-#### to every normative requirement and cite source sections. Do not read or imitate the AMX worker's PlanVector until your normalization is complete. Identify any ECM ambiguity involving canonical records, events, bundles, quarantine, cross-provider semantics, memory authority, deletion, effect execution, and promotion. Return the PlanVector, digest of the ECM source, and an ambiguity register. Return no implementation code.
```

### Prompt C — Bidirectional comparison, sent separately to both workers

```text
Using both immutable source artifacts and both independently produced PlanVectors, create a complete DifferenceRecord matrix. Classify each relationship as identical, equivalent_rename, complementary, conflict, missing, or unsupported_or_unevidenced. For every conflict, state the failure mode of selecting either side, propose a falsifiable test, and recommend a resolution without altering the source artifacts. Explicitly identify duplicate canonical owners and any path by which memory or provider content could expand authority. Report coverage counts for AMX-R and ECM-R IDs. Return no implementation code.
```

### Prompt D — Adversarial review for the AMX worker

```text
Act as a hostile but evidence-bound reviewer of ECM. Try to falsify its necessity and feasibility: duplicated canonical state, memory/orchestration coupling, excessive abstraction, unimplementable evidence gates, latency and storage cost, adapter overreach, and migration burden. Every finding must include severity, affected requirement IDs, a concrete failure scenario, a reproducible test, and the smallest viable correction. Do not defend AMX-1 merely because you authored it.
```

### Prompt E — Adversarial review for the ECM worker

```text
Act as a hostile but evidence-bound reviewer of AMX-1. Try to falsify its safety and interoperability: ambiguous authority, lossy cross-provider translation, quarantine escape, deletion/retention leakage, replay/idempotency defects, schema downgrade hazards, unverified canonicalization, and hidden coupling to one provider. Every finding must include severity, affected requirement IDs, a concrete failure scenario, a reproducible test, and the smallest viable correction. Do not defend ECM merely because you authored it.
```

### Prompt F — Final integrator

```text
Synthesize AMX-1 and ECM into one AMCX-1 specification. Use the immutable source artifacts, both PlanVectors, the DifferenceRecord matrix, and both adversarial reviews. Apply this precedence: existing AutoDev/ForgeCore safety invariants; no authority expansion; one canonical owner per datum; round-trip interoperability; reproducible evidence; simplicity/migration cost; novelty.

For every AMX-R and ECM-R requirement, record exactly one disposition: keep, merge, replace, defer, or reject, with rationale and evidence. Produce: normative AMCX-1 spec; namespace and ownership map; schemas/state machines; AMX and ECM crosswalks; provider capability/loss matrix for ChatGPT/Codex, AutoDev/Cline, A2A, Mistral Agents, and Mistral Vibe; migration/rollback plan; conformance tests; risk register; decision log. Do not implement. Mark all unresolved decisions and stop for user approval before implementation planning.
```

## 7. Convergence criteria

Reconciliation is complete only when:

1. both original artifacts are immutable and addressable by digest;
2. 100% of AMX-R and ECM-R requirements have dispositions;
3. one and only one canonical owner exists for every persistent datum and transition;
4. no content-bearing object can create authority or bypass approval;
5. all adapter loss is explicit and machine-readable;
6. quarantine, retention, deletion, replay, and rollback semantics are testable;
7. migrations are reversible within a declared compatibility window;
8. independent verification reports zero unresolved critical findings;
9. the user approves AMCX-1 before implementation begins.

## 8. Immediate next action

Send **Prompt A** to the other ChatGPT worker. Return the resulting AMX-1 artifact or accessible attachment here. Once it is available, execute Rounds 1–6 without relying on summaries of the missing source.
