# AMX-1 / ECM Round 2 DifferenceRecord matrix

Date: 2026-08-20  
Status: independent source comparison; no implementation and no source revision  
Decision authority: none; all dispositions are reconciliation proposals

## 1. Immutable inputs and method

| Artifact | SHA-256 | Result |
|---|---|---|
| AMX-1 normative source | `4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9` | Byte match |
| AMX normalization | `c81aedb9528df2162e5c327f6479a89848e70bf85a3835b3d76b67e5b06dae52` | Byte match |
| ECM normative source | `e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980` | Byte match |
| ECM normalization | `9ddf7754d017384f4d26ef801eac333a8e2a4148ef3d276fd178a032c49c7810` | Byte match |

The two sources were compared first. Normalization rows were used only to locate and cover requirements; when source and envelope differ, the source governs. Three independent read-only audits covered semantic differences, AMX catalog integrity, and ECM catalog integrity.

Classification vocabulary is restricted to `identical`, `equivalent_rename`, `complementary`, `conflict`, `missing`, and `unsupported_or_unevidenced`.

## 2. Complete DifferenceRecord matrix

| ID | Domain | AMX requirements | ECM requirements | Classification | Finding | Proposed disposition |
|---|---|---|---|---|---|---|
| D-0001 | mission.scope | AMX-R-0001–0008 | ECM-R-0001–0010 | `complementary` | AMX governs portable development memory; ECM governs bounded collaboration and coordination. | `merge` |
| D-0002 | privacy.minimization | AMX-R-0009–0012 | ECM-R-0011–0012, ECM-R-0085, ECM-R-0093 | `identical` | Both exclude hidden reasoning/raw transcripts and restrict sensitive material. | `keep` |
| D-0003 | authority.boundary | AMX-R-0006, AMX-R-0012, AMX-R-0019, AMX-R-0023, AMX-R-0035, AMX-R-0117–0123 | ECM-R-0008, ECM-R-0013–0015, ECM-R-0020–0025, ECM-R-0107–0112 | `identical` | Evidence, memory, messages, votes, signatures, and models never grant capability; trusted policy/execution owns authority. | `keep` |
| D-0004 | minimal.infrastructure | AMX-R-0008, AMX-R-0013, AMX-R-0017 | ECM-R-0009, ECM-R-0016–0018 | `equivalent_rename` | Both require locally operable, bounded designs without mandatory graph/vector/distributed infrastructure. | `keep` |
| D-0005 | scope.before.rank | AMX-R-0002–0003, AMX-R-0021, AMX-R-0034 | ECM-R-0026–0027, ECM-R-0095 | `identical` | Hard authority/scope/retention/deletion filters precede relevance; ECM adds context-view digest/expiry/budget. | `merge` |
| D-0006 | contradiction.history | AMX-R-0004, AMX-R-0024, AMX-R-0054, AMX-R-0104–0107 | ECM-R-0003, ECM-R-0028, ECM-R-0089, ECM-R-0110–0112 | `identical` | Both preserve conflict and forbid semantic last-write-wins or majority-as-truth. | `keep` |
| D-0007 | evidence.freshness | AMX-R-0020, AMX-R-0037–0038, AMX-R-0048–0049 | ECM-R-0103–0106, ECM-R-0205 | `complementary` | AMX supplies memory validity/revalidation; ECM supplies exact-subject evidence binding and staleness. | `merge` |
| D-0008 | source.of.truth | AMX-R-0061–0070 | ECM-R-0175–0184 | `complementary` | Maps agree where they overlap; ECM additionally owns collaboration/effect/evidence domains. | `merge` |
| D-0009 | identity.canonicalization | AMX-R-0044–0060 | ECM-R-0031–0044, ECM-R-0138 | `complementary` | AMX is precise for memory IDs/JCS/digests; ECM adds workflow, trace, effect and attestation identifiers but lacks cross-language vectors. | `merge` |
| D-0010 | memory.contract | AMX-R-0071–0094 | ECM-R-0071–0102, ECM-R-0185, ECM-R-0195–0196 | `conflict` | Both claim a canonical durable-memory contract, but AMX record/event/bundle and ECM `evidence-memory-v1` are not isomorphic. | `replace` |
| D-0011 | memory.event.vs.workflow.log | AMX-R-0068, AMX-R-0077–0084, AMX-R-0095, AMX-R-0104–0107, AMX-R-0139 | ECM-R-0133, ECM-R-0166, ECM-R-0177, ECM-R-0200–0201 | `complementary` | AMX can own memory mutation while ECM owns collaboration delivery/reduction; dual write would turn this into a conflict. | `merge` |
| D-0012 | plan.task.effect | AMX-R-0017, AMX-R-0064 | ECM-R-0031, ECM-R-0052–0058, ECM-R-0137–0142, ECM-R-0176–0179 | `complementary` | ExecPlan, ECM collaboration tasks, and ForgeCore effects are different state domains. | `keep` |
| D-0013 | admission.vs.promotion | AMX-R-0028, AMX-R-0077–0083, AMX-R-0098–0103, AMX-R-0115 | ECM-R-0067–0070, ECM-R-0071–0100, ECM-R-0117–0128 | `complementary` | Memory trust/visibility and configuration activation/canary/rollback must remain independent lifecycles linked by typed references. | `merge` |
| D-0014 | cross.project.promotion | AMX-R-0003, AMX-R-0016, AMX-R-0115, AMX-R-0239 | ECM-R-0030, ECM-R-0073, ECM-R-0108, ECM-R-0121 | `missing` | ECM permits organization/exportable visibility but lacks an unconditional current scoped user-authorization rule for cross-project memory promotion. | `merge` |
| D-0015 | origin.trust.quarantine | AMX-R-0022, AMX-R-0032–0033, AMX-R-0036, AMX-R-0050–0051, AMX-R-0098–0099, AMX-R-0215–0216 | ECM-R-0022, ECM-R-0067, ECM-R-0069, ECM-R-0074, ECM-R-0082, ECM-R-0086–0087, ECM-R-0162 | `missing` | ECM exposes producer/principal and trust state without receiver-binding fields, setter authority, or quarantine-release guards. | `merge` |
| D-0016 | deletion.purge.resurrection | AMX-R-0096–0097, AMX-R-0107–0110, AMX-R-0183, AMX-R-0193 | ECM-R-0133, ECM-R-0153, ECM-R-0208 | `unsupported_or_unevidenced` | Neither source defines total purge/partial-failure/receipt/anti-resurrection semantics. | `defer` |
| D-0017 | extensions.downgrade | AMX-R-0073, AMX-R-0160, AMX-R-0193, AMX-R-0206 | ECM-R-0050, ECM-R-0174, ECM-R-0209 | `conflict` | AMX requires unknown-extension preservation; ECM requires unknown security-critical fields to fail closed; criticality and downgrade are undefined. | `merge` |
| D-0018 | context.replay | AMX-R-0021, AMX-R-0034, AMX-R-0046, AMX-R-0164, AMX-R-0175, AMX-R-0222–0231 | ECM-R-0027, ECM-R-0046, ECM-R-0051, ECM-R-0055, ECM-R-0062, ECM-R-0068, ECM-R-0095, ECM-R-0142, ECM-R-0160, ECM-R-0204 | `complementary` | AMX determines record eligibility; ECM owns immutable context-view replay/resume. Binding semantics are missing. | `merge` |
| D-0019 | promotion.statistics.budgets | AMX-R-0167–0185 | ECM-R-0119–0128, ECM-R-0154, ECM-R-0156–0157, ECM-R-0210, ECM-R-0223–0224, ECM-R-0236 | `conflict` | AMX’s three-point efficacy rule and ECM’s equal-quality/lower-cost rule can disagree; both omit calibrated independence and aggregate hierarchical budgets. | `merge` |
| D-0020 | memory.taxonomy | AMX-R-0044–0055 | ECM-R-0071–0081 | `complementary` | AMX uses matrix coordinates; ECM separates function, visibility, and trust and adds class-specific promotion rules. | `merge` |
| D-0021 | memory.write.path | AMX-R-0100–0103, AMX-R-0210–0221 | ECM-R-0086–0093 | `equivalent_rename` | Both validate, minimize, dedupe without event collapse, authorize, audit, and schedule lifecycle work; AMX adds receiver binding/quarantine. | `merge` |
| D-0022 | memory.retrieval | AMX-R-0164, AMX-R-0181, AMX-R-0222–0231 | ECM-R-0094–0100 | `equivalent_rename` | Retrieval stages and authority isolation substantially align; AMX adds branch/path/quarantine and fixed result budget. | `merge` |
| D-0023 | protocol.mcp | AMX-R-0141–0147 | ECM-R-0037–0038, ECM-R-0163 | `equivalent_rename` | MCP is a disposable/stateless adapter and never the authority or system of record. | `keep` |
| D-0024 | protocol.a2a | AMX-R-0148–0154 | ECM-R-0036, ECM-R-0040, ECM-R-0168 | `complementary` | AMX specifies memory artifacts; ECM specifies collaboration tasks, gateway identity, and lease preservation. | `merge` |
| D-0025 | repository.instructions | AMX-R-0124–0126, AMX-R-0158, AMX-R-0163, AMX-R-0232–0241 | ECM-R-0167, ECM-R-0171–0173, ECM-R-0211–0244 | `complementary` | AMX provides memory onboarding; ECM supplies controller/collaboration behavior. | `merge` |
| D-0026 | artifact.evidence.contract | AMX-R-0049, AMX-R-0089–0090 | ECM-R-0103–0106, ECM-R-0135, ECM-R-0180–0181 | `missing` | AMX references artifacts/evidence but lacks ECM’s exact ArtifactRef/EvidenceRef lifecycle. | `keep` |
| D-0027 | collaboration.envelope.lifecycle | — | ECM-R-0045–0070 | `missing` | AMX deliberately lacks peer-message, task, attempt, role-lease, cross-prompt, and task-context contracts. | `keep` |
| D-0028 | consensus.reflection.improvement | AMX-R-0024, AMX-R-0033 | ECM-R-0107–0128 | `complementary` | AMX constrains corroboration/conflict; ECM adds DecisionPolicy, retrospective, canary, and rollback. | `keep` |
| D-0029 | persistence.effects | AMX-R-0127–0140 | ECM-R-0133–0142 | `complementary` | AMX defines memory storage/concurrency; ECM defines collaboration durability and references ForgeCore effects. | `merge` |
| D-0030 | observability.audit | AMX-R-0043, AMX-R-0055, AMX-R-0171–0172 | ECM-R-0143–0148 | `complementary` | ECM has broader run/agent/tool telemetry; AMX has memory influence and lifecycle metrics. | `merge` |
| D-0031 | security.recovery | AMX-R-0029–0043, AMX-R-0110 | ECM-R-0149–0158 | `complementary` | Threat suites overlap; AMX focuses poisoning/deletion while ECM adds delegation/effect/circuit-breaker failures. | `merge` |
| D-0032 | adapters.degradation | AMX-R-0155–0161, AMX-R-0163 | ECM-R-0159–0174 | `complementary` | ECM defines harness execution surfaces; AMX defines memory-semantic preservation and provider deletion limits. | `merge` |
| D-0033 | migration.exit | AMX-R-0186–0193 | ECM-R-0101–0102, ECM-R-0208–0209 | `missing` | ECM has compatibility checks but no complete provider-removal, bundle migration, or failure exit path. | `merge` |
| D-0034 | sequencing.acceptance | AMX-R-0194–0209, AMX-R-0242–0244 | ECM-R-0185–0210 | `complementary` | Both start with neutral contracts and conformance; ECM’s first boundary is broader while AMX prioritizes memory contracts. | `merge` |
| D-0035 | controller.onboarding | AMX-R-0158, AMX-R-0232–0241 | ECM-R-0211–0244 | `complementary` | The controller seed refines AMX’s provider-neutral memory onboarding but duplicates many earlier ECM obligations. | `merge` |
| D-0036 | normalization.amx.traceability | AMX-R-0001–0244 except AMX-R-0162 | — | `unsupported_or_unevidenced` | 243 IDs exist, but modality provenance, exact source spans, quote digests, and relationship metadata are absent; AMX-R-0162 is unexplained. | `defer` |
| D-0037 | normalization.ecm.traceability | — | ECM-R-0001–0244 | `unsupported_or_unevidenced` | 244 contiguous IDs exist, but 168 derived obligations are range-labeled rather than per-row and exact spans/quote digests are absent. | `defer` |
| D-0038 | declared.contracts | AMX-R-0071–0094, AMX-R-0203–0206 | ECM-R-0045–0106, ECM-R-0185–0198 | `unsupported_or_unevidenced` | Both sources contain conceptual contracts and partial diagrams, not machine-readable schemas, conformance vectors, or total transition tables. | `defer` |
| D-0039 | catalog.duplicates | AMX-R-0100, AMX-R-0158, AMX-R-0164, AMX-R-0210–0244 | ECM-R-0013, ECM-R-0020, ECM-R-0057, ECM-R-0123, ECM-R-0154–0157, ECM-R-0180–0185, ECM-R-0200–0244 | `unsupported_or_unevidenced` | Aggregate and controller requirements lack explicit `alias_of`, `refines`, or `aggregate_of` metadata. | `defer` |
| D-0040 | empirical.calibration | AMX-R-0180, AMX-R-0182, AMX-R-0184 | ECM-R-0122, ECM-R-0129–0132, ECM-R-0154, ECM-R-0156–0157 | `unsupported_or_unevidenced` | Thresholds, routing prediction, reviewer correlation, and local storage assumptions lack AutoDev calibration. | `defer` |

## 3. Adjudication of the ten suspected conflicts

### D-0010 — AMX record/event/bundle versus ECM `evidence-memory-v1`

- Applicable IDs: AMX-R-0071–0094; ECM-R-0071–0102, ECM-R-0185, ECM-R-0195–0196.
- Failure if AMX alone is selected: ECM run/attempt/principal, promotion reference, operational memory-class, and observation/claim/summary separation can be lost.
- Failure if ECM alone is selected: canonical event digests, causal heads, complete-state events, receiver-bound origin, bundle manifests, deletion boundary, and projection portability can be lost.
- Falsifiable test: map fixtures containing every field in both source examples bidirectionally. Export/import/export must preserve AMX logical IDs, event digests, heads, unknown extensions, retractions and deletion semantics plus all ECM-only operational fields. Any ambiguous or lossy field fails.
- Recommendation: `replace` ECM’s overlapping canonical memory schema with an AMX profile and namespaced ECM collaboration fields. This is a proposal, not a source revision.

### D-0011 — AMX memory event DAG versus ECM workflow event log

- Classification test result: `complementary`, not a direct conflict, provided no dual authoritative write exists.
- Applicable IDs: AMX-R-0068, AMX-R-0077–0084, AMX-R-0095, AMX-R-0104–0107, AMX-R-0139; ECM-R-0133, ECM-R-0166, ECM-R-0177, ECM-R-0200–0201.
- Falsifiable test: crash between AMX event commit and ECM acknowledgement, then replay duplicate/reordered workflow events. AMX heads and ECM collaboration projections must reconstruct identically without synthesizing or losing a memory mutation.
- Recommendation: `merge` with digest references and an atomic outbox/inbox composition rule; AMX owns memory causality and ECM owns collaboration delivery/reduction.

### D-0012 — ExecPlan step/effect state versus ECM collaboration task/effect state

- Classification test result: `complementary`.
- Applicable IDs: AMX-R-0017, AMX-R-0064; ECM-R-0031, ECM-R-0052–0058, ECM-R-0137–0142, ECM-R-0176–0179.
- Falsifiable test: map one ExecPlan step to zero, one, and multiple ECM tasks. Retry/cancellation must not rewrite an ExecPlan attempt or mark an effect committed without a ForgeCore receipt.
- Recommendation: `keep` three distinct state machines with explicit foreign keys and reducers: ExecPlan for plan state, ECM for collaboration state, ForgeCore for effect state.

### D-0013 — Memory admission/verification versus configuration promotion and visibility

- Classification test result: `complementary`.
- Applicable IDs: AMX-R-0028, AMX-R-0077–0083, AMX-R-0098–0103, AMX-R-0115; ECM-R-0067–0070, ECM-R-0071–0100, ECM-R-0117–0128.
- Falsifiable test: verifying an AMX record must not activate a prompt/skill or widen visibility; rolling back a configuration must not retract or rewrite its supporting memory.
- Recommendation: `merge` through typed subject references while retaining independent lifecycles.

### D-0014 — Cross-project promotion without explicit user authorization

- Classification test result: `missing` in ECM, not an affirmative source conflict.
- Failure exposure: an ECM DecisionPolicy can omit a human gate while promoting project memory to organization/exportable visibility.
- Falsifiable test: a fully evidence-passing cross-project promotion without a current scoped user grant must fail; the same request with a valid grant may proceed.
- Recommendation: `merge` AMX’s mandatory approval invariant into the composed policy.
- Unresolved: approver identity, grant scope, validity, revocation, and approval-record schema.

### D-0015 — Producer-asserted trust and quarantine release

- Classification test result: `missing` receiver-binding and release semantics in ECM.
- Failure exposure: a producer can submit `trust_state=verified`, or a summary can launder origin, unless the receiver ignores producer trust state.
- Falsifiable test: imported records claiming trusted origin or `verified` remain quarantined until receiver binding plus policy-eligible independent evidence exists; every transform preserves original trust lineage.
- Recommendation: `merge` AMX receiver binding and release control into the ECM profile.
- Unresolved: total quarantine transition table and release-authority schema.

### D-0016 — Purge, partial failure, deletion receipts, and pre-delete resurrection

- Classification test result: `unsupported_or_unevidenced` in both.
- Falsifiable test: crash after each projection/store deletion, restore every older checkpoint, and re-import a pre-delete bundle. Deleted content must remain nonretrievable/nonprojectable, while a non-content receipt proves every boundary component’s terminal result.
- Recommendation: `defer` deletion implementation until a total state machine, receipt schema, reconciliation semantics, and anti-resurrection barrier exist.

### D-0017 — Unknown security-critical extensions and adapter downgrade

- Applicable IDs: AMX-R-0073, AMX-R-0160, AMX-R-0193, AMX-R-0206; ECM-R-0050, ECM-R-0174, ECM-R-0209.
- Failure if AMX alone is selected: an adapter can preserve and use a safety-critical field without understanding its enforcement meaning.
- Failure if ECM alone is selected: rejection or stripping can violate canonical round-trip integrity and destroy forward-compatible data.
- Falsifiable test: unknown noncritical extensions round-trip and remain usable; unknown critical extensions round-trip byte-for-byte only in quarantine and cannot influence consequential context or projection. Adapter downgrade must disclose the unsupported guarantee.
- Recommendation: `merge` as “preserve opaquely, fail use closed, never silently discard.”

### D-0018 — Context replay across tenant, task, role, lease, or expiry

- Classification test result: `complementary` with a missing binding profile.
- Falsifiable test: replay the same context digest after independently changing tenant, project, task, attempt, role lease, capability lease, expiry, repository revision, or deletion state. Each change must reject or rebuild the view before use.
- Recommendation: `merge` ECM context-view ownership with AMX record-eligibility policy.

### D-0019 — Statistical promotion gates, verifier independence, and aggregate budgets

- Applicable IDs: AMX-R-0167–0185; ECM-R-0119–0128, ECM-R-0154, ECM-R-0156–0157, ECM-R-0210, ECM-R-0223–0224, ECM-R-0236.
- Failure if AMX alone is selected: a candidate can pass without equal-compute/cost normalization, rotating hidden suites, rollback rehearsal, or reviewer-correlation controls.
- Failure if ECM alone is selected: a cheaper equal-quality memory candidate can become default although AMX says it remains interchange-only below the three-point efficacy target; AMX-specific retrieval, deletion, and latency gates disappear.
- Falsifiable test: preregister sample size, repeated trials, confidence method, correlation exclusion, aggregate token/call/time/cost ceilings, and both profiles. Evaluate at least a +2-point cheaper candidate and a +4-point over-budget candidate; independent implementations must return the same decision.
- Recommendation: `merge` as conjunctive typed gate profiles, then empirically calibrate numeric thresholds before promotion.

## 4. Normalization integrity audit

### AMX catalog

- Coverage: 243 unique requirement rows over serial slots 0001–0244; no duplicate row IDs; `AMX-R-0162` is absent.
- Traceability: 243/243 have a section cell; 32 add item-level location; 211 are heading-only; 0 have an exact span, stable anchor, or quotation digest.
- Modality: 224 rows contain `MUST`, 18 contain `MAY`, 2 contain `SHOULD`, and 8 lack an uppercase keyword. The envelope does not record source modality per row.
- Inflation examples: AMX-R-0023 strengthens signing-key control into authorship; AMX-R-0098 invents a general acceptance/independent-verification release rule; AMX-R-0100 makes the numbered admission list strictly ordered; declarative AMX-R-0071/0141/0148/0159 become RFC-style obligations.
- Deflation/precision loss: AMX-R-0113 weakens advisory-phase enablement to `MAY`; AMX-R-0192 weakens required fallback retention to `MAY`; AMX-R-0169/0171/0172 omit named rates, classifications, percentiles, and correction timing.
- AMX-R-0203 is materially imprecise: it omits exactly three schemas, schema/example existence, at least two independent implementations, and the alternative of one implementation plus fixed external vectors. It is necessary but not sufficient for implementation acceptance.
- Missing source obligations: AutoDev-specific ablation of cross-LLM benefit/temporal graphs/learned consolidation/semantic promotion; explicit CloudEvents envelope use; array/parent evaluation before graph storage; corpus adequacy/uncertainty reporting; exact signature key-control semantics.
- Declared but unimplemented: 3/3 complete schemas absent; 0/5 lifecycle inventories are total state machines. The record example omits required matrix data including revision/digest, conditional tenant/user scope, quarantine/corroboration/confidence, consent, and influence.

### ECM catalog

- Coverage: 244 unique contiguous requirement rows; no missing or duplicate row IDs.
- Extraction register: 30 `source_normative`, 168 `derived_design_obligation`, 12 `acceptance_obligation`, and 34 `controller_normative`.
- Traceability: 244/244 have a section cell; 39 add item-level location; 205 are heading-only; 0 have an exact source span, stable anchor, or quotation digest.
- Modality: within the 168 derived rows, 150 contain `MUST`; recommended topology (ECM-R-0031–0035), mapping matrix (ECM-R-0174), and proposed implementation program (ECM-R-0185–0198) are materially inflated. ECM-R-0036/0043/0132 may deflate categorical source statements to `SHOULD`.
- Semantic overreach: ECM-R-0022 changes a signature claim into generalized provenance integrity; ECM-R-0155 invents a named envelope-violation class; ECM-R-0174 adds authority-semantics wording not present in §23.
- Declared but unimplemented: 8 structured conceptual surfaces, 5 prose-only contracts, 22/23 payload variants without field contracts, 7 context-entry types without schemas, and no total task/role/promotion/effect/recovery transition table.
- Unevidenced numeric thresholds: 3 evaluation rounds, 10,000 episodes, depth 4, fan-out 8, three non-progress states, and five denials. The ECM ambiguity register acknowledges only the first two.
- Omitted source obligations include authoritative AutoDev TaskGraph/ExecutionEnvelope/self-evaluation factory, repeated-failure and human-correction promotion gates, SQLite saturation instrumentation, key rotation/recovery, target canonicalization tests, provider-session cache semantics, expanded reviewer-correlation fields/fallback, pinned Vibe versions/conformance, and benchmark-contamination controls.

## 5. Duplicate and refinement registry

No duplicate requirement is removed. Relationships below are reconciliation metadata only.

### AMX

| Relationship | Requirements |
|---|---|
| `alias_of` | AMX-R-0220 → AMX-R-0103; AMX-R-0227 → AMX-R-0181 |
| `refines` aggregate | AMX-R-0210–0221 refine AMX-R-0100 |
| `refines` aggregate | AMX-R-0222–0231 refine AMX-R-0164 |
| `refines` aggregate | AMX-R-0232–0241 refine AMX-R-0158 |
| `refines` aggregate | AMX-R-0111–0116 refine AMX-R-0242 |
| `refines` | AMX-R-0213→0011/0039; 0215→0022/0036; 0216→0098; 0217/0218→0054; 0219→0028/0101/0102; 0223→0021; 0229→0123; 0230→0055; 0231→0025; 0238→0011; 0239→0016; 0206→0160; 0208→0114 |

### ECM

| Relationship | Requirements |
|---|---|
| `alias_of` | ECM-R-0184→0037; 0201→0051; 0240→0054; 0235→0060; 0230→0083; 0135→0091; 0180→0135; 0231→0100; 0182→0101; 0237→0107; 0225→0116; 0223→0119; 0215→0131; 0242→0158 |
| `refines` | ECM-R-0013/0020→0021; 0057/0123/0207/0220/0238→0024; 0095→0026; 0084/0089/0110/0206/0219→0028; 0141/0142/0202/0227→0029; 0085→0012; 0093/0147/0162/0218/0226/0233→0011 |
| `aggregate_of` | ECM-R-0214→0129–0131; 0217→0021/0047/0063/0100/0162; 0218→0011/0012/0065/0066; 0219→0028/0084/0110; 0220→0024/0109; 0221→0104/0105/0124/0142; 0222→0115/0116/0123; 0224→0125–0128; 0226→0092/0099/0143–0148; 0227→0018/0029/0139–0142/0155–0157; 0232→0076–0079/0117–0128; 0234→0045/0049; 0241→0155–0157 |
| broad aggregate | ECM-R-0239 aggregates routing, contracting, context admission, verification, reflection, promotion, and completion obligations; ECM-R-0244 refines ECM-R-0019/0054/0240. |

## 6. Proposed canonical owners for all state-bearing domains

| State-bearing domain | Proposed sole canonical owner | Other representations |
|---|---|---|
| Source code/configuration/history | Git repository at identified commit | AMX/ECM references only |
| Durable plan/milestone/step/replan lifecycle | AutoDev typed ExecPlan | ECM tasks reference plan IDs |
| Collaboration task/worker attempt/role lease/message/context-view state | ECM workflow event log and reducers | Provider/A2A state is a projection |
| Memory logical record/revision/mutation/causal heads/bundle | AMX canonical contracts/event DAG | ECM uses an AMX profile/reference |
| Memory admission/trust/quarantine/visibility/retraction/retention/purge job | AMX lifecycle service | ECM submits typed requests and observes decisions |
| Task-context entry admission/subscription | ECM context service/workflow log | AMX references are selected under AMX eligibility policy |
| Authorization/capabilities/approvals | ForgeCore/host policy and grant store | Messages/memory carry references only |
| Consequential effects/idempotency/receipts | ForgeCore effect ledger | ExecPlan/ECM reference effect IDs |
| Verification evidence/freshness/verdict | EvidenceStore/VerificationFabric | AMX/ECM records reference evidence IDs |
| Prompt/skill/router/schema candidate promotion/canary/rollback | ECM promotion service and decision log | AMX memory may support but never activate candidates |
| Large immutable artifact bytes | Content-addressed artifact store | ArtifactRef is metadata/reference |
| Schema/version/critical-extension registry | Repository-reviewed Neutral Contract Registry | Runtime caches are derived |
| Aggregate run budgets/reservations/leases | ECM durable orchestrator/budget ledger | Agents receive attenuated views |
| Provider session/execution state | Provider adapter, opaque and noncanonical | Recover from ECM events and explicit handles |
| Search indexes/embeddings/summaries/projections | Derived stores owned by their adapter | Rebuild from canonical domain events |
| Reviewed repository memory | Git-reviewed AMX projection | Canonical memory identity/history remains AMX |
| Current `toolset-pattern-v1` learning | `memory/toolsets/patterns.jsonl` until explicit migration | ECM/AMX may index or reference |
| Telemetry/traces/audit projection | Append-only/tamper-evident observability service | Never grants authority |

## 7. Unresolved critical blockers

1. Complete machine-readable AMX schemas and an ECM-to-AMX memory profile/crosswalk do not exist.
2. No atomic composition rule connects AMX memory events to ECM workflow events.
3. Task, role, memory, quarantine, promotion, effect, recovery, and deletion lifecycles lack total transition/guard/invalid-transition tables.
4. Purge receipts, partial-failure reconciliation, durable deletion barriers, and pre-delete anti-resurrection semantics are undefined.
5. Cross-project approval identity, scope, validity, revocation, and record schema are undefined in ECM.
6. Receiver-origin binding and quarantine-release authority are incomplete.
7. Extension criticality and adapter capability/downgrade profiles are absent.
8. Context-view binding, replay, reducer, lease, expiry, and revalidation semantics are incomplete.
9. Verifier-independence scoring and correlated-review treatment are undefined.
10. Aggregate descendant/concurrency/token/call/time/cost budget reservation and accounting are undefined.
11. Statistical thresholds are uncalibrated by risk class; power, interval, stopping, and multiple-comparison rules are absent.
12. Cross-language canonicalization and deterministic decision vectors do not exist.
13. Both normalization envelopes require non-normative correction metadata: exact source spans, quote digests, per-row extraction kind/modality, and duplicate relationships.
14. AMX-R-0162 must be declared reserved/retired or filled only by a separately attested normalization delta; later IDs must not be renumbered.
15. AMX-R-0203 must be corrected in a non-source normalization delta to preserve the exact two-implementation-or-one-plus-fixed-external-vectors rule and its necessary-but-not-sufficient status.

## 8. Coverage summary

- AMX: **243/243 published IDs covered**, spanning AMX-R-0001–0244 with AMX-R-0162 absent. The absent serial is recorded as a catalog blocker, not counted as a requirement.
- ECM: **244/244 published IDs covered**, contiguous ECM-R-0001–0244.
- Duplicate/aggregate requirements remain present and are related through `alias_of`, `refines`, or `aggregate_of`; none is omitted from coverage.
- The per-ID coverage ledger below assigns every published ID to a primary DifferenceRecord and records duplicate/refinement status.

## 9. Per-ID coverage ledger

+
### 9.1 AMX per-ID ledger

| Requirement | Primary DifferenceRecord | Catalog relation |
|---|---|---|
| AMX-R-0001 | D-0001 | primary |
| AMX-R-0002 | D-0001 | primary |
| AMX-R-0003 | D-0001 | primary |
| AMX-R-0004 | D-0001 | primary |
| AMX-R-0005 | D-0001 | primary |
| AMX-R-0006 | D-0001 | primary |
| AMX-R-0007 | D-0001 | primary |
| AMX-R-0008 | D-0001 | primary |
| AMX-R-0009 | D-0002 | primary |
| AMX-R-0010 | D-0002 | primary |
| AMX-R-0011 | D-0002 | primary |
| AMX-R-0012 | D-0002 | primary |
| AMX-R-0013 | D-0002 | primary |
| AMX-R-0014 | D-0002 | primary |
| AMX-R-0015 | D-0002 | primary |
| AMX-R-0016 | D-0002 | primary |
| AMX-R-0017 | D-0002 | primary |
| AMX-R-0018 | D-0002 | primary |
| AMX-R-0019 | D-0003 | primary |
| AMX-R-0020 | D-0003 | primary |
| AMX-R-0021 | D-0003 | primary |
| AMX-R-0022 | D-0003 | primary |
| AMX-R-0023 | D-0003 | primary |
| AMX-R-0024 | D-0031 | primary |
| AMX-R-0025 | D-0031 | primary |
| AMX-R-0026 | D-0031 | primary |
| AMX-R-0027 | D-0031 | primary |
| AMX-R-0028 | D-0031 | primary |
| AMX-R-0029 | D-0031 | primary |
| AMX-R-0030 | D-0031 | primary |
| AMX-R-0031 | D-0031 | primary |
| AMX-R-0032 | D-0031 | primary |
| AMX-R-0033 | D-0031 | primary |
| AMX-R-0034 | D-0031 | primary |
| AMX-R-0035 | D-0031 | primary |
| AMX-R-0036 | D-0031 | primary |
| AMX-R-0037 | D-0031 | primary |
| AMX-R-0038 | D-0031 | primary |
| AMX-R-0039 | D-0031 | primary |
| AMX-R-0040 | D-0031 | primary |
| AMX-R-0041 | D-0031 | primary |
| AMX-R-0042 | D-0031 | primary |
| AMX-R-0043 | D-0031 | primary |
| AMX-R-0044 | D-0009 | primary |
| AMX-R-0045 | D-0009 | primary |
| AMX-R-0046 | D-0009 | primary |
| AMX-R-0047 | D-0009 | primary |
| AMX-R-0048 | D-0009 | primary |
| AMX-R-0049 | D-0009 | primary |
| AMX-R-0050 | D-0009 | primary |
| AMX-R-0051 | D-0009 | primary |
| AMX-R-0052 | D-0009 | primary |
| AMX-R-0053 | D-0009 | primary |
| AMX-R-0054 | D-0009 | primary |
| AMX-R-0055 | D-0009 | primary |
| AMX-R-0056 | D-0009 | primary |
| AMX-R-0057 | D-0009 | primary |
| AMX-R-0058 | D-0009 | primary |
| AMX-R-0059 | D-0009 | primary |
| AMX-R-0060 | D-0009 | primary |
| AMX-R-0061 | D-0008 | primary |
| AMX-R-0062 | D-0008 | primary |
| AMX-R-0063 | D-0008 | primary |
| AMX-R-0064 | D-0008 | primary |
| AMX-R-0065 | D-0008 | primary |
| AMX-R-0066 | D-0008 | primary |
| AMX-R-0067 | D-0008 | primary |
| AMX-R-0068 | D-0008 | primary |
| AMX-R-0069 | D-0008 | primary |
| AMX-R-0070 | D-0008 | primary |
| AMX-R-0071 | D-0010 | primary |
| AMX-R-0072 | D-0010 | primary |
| AMX-R-0073 | D-0010 | primary |
| AMX-R-0074 | D-0010 | primary |
| AMX-R-0075 | D-0010 | primary |
| AMX-R-0076 | D-0010 | primary |
| AMX-R-0077 | D-0010 | primary |
| AMX-R-0078 | D-0010 | primary |
| AMX-R-0079 | D-0010 | primary |
| AMX-R-0080 | D-0010 | primary |
| AMX-R-0081 | D-0010 | primary |
| AMX-R-0082 | D-0010 | primary |
| AMX-R-0083 | D-0010 | primary |
| AMX-R-0084 | D-0010 | primary |
| AMX-R-0085 | D-0010 | primary |
| AMX-R-0086 | D-0010 | primary |
| AMX-R-0087 | D-0010 | primary |
| AMX-R-0088 | D-0010 | primary |
| AMX-R-0089 | D-0010 | primary |
| AMX-R-0090 | D-0010 | primary |
| AMX-R-0091 | D-0010 | primary |
| AMX-R-0092 | D-0010 | primary |
| AMX-R-0093 | D-0010 | primary |
| AMX-R-0094 | D-0010 | refines AMX-R-0026 |
| AMX-R-0095 | D-0011 | primary |
| AMX-R-0096 | D-0011 | primary |
| AMX-R-0097 | D-0011 | primary |
| AMX-R-0098 | D-0011 | primary |
| AMX-R-0099 | D-0011 | primary |
| AMX-R-0100 | D-0011 | primary |
| AMX-R-0101 | D-0011 | primary |
| AMX-R-0102 | D-0011 | primary |
| AMX-R-0103 | D-0011 | primary |
| AMX-R-0104 | D-0011 | primary |
| AMX-R-0105 | D-0011 | primary |
| AMX-R-0106 | D-0011 | primary |
| AMX-R-0107 | D-0011 | primary |
| AMX-R-0108 | D-0011 | primary |
| AMX-R-0109 | D-0011 | refines AMX-R-0014 |
| AMX-R-0110 | D-0011 | primary |
| AMX-R-0111 | D-0011 | refines AMX-R-0242 |
| AMX-R-0112 | D-0011 | refines AMX-R-0242 |
| AMX-R-0113 | D-0011 | refines AMX-R-0242 |
| AMX-R-0114 | D-0011 | refines AMX-R-0242 |
| AMX-R-0115 | D-0011 | refines AMX-R-0242 |
| AMX-R-0116 | D-0011 | refines AMX-R-0242 |
| AMX-R-0117 | D-0003 | primary |
| AMX-R-0118 | D-0003 | primary |
| AMX-R-0119 | D-0003 | primary |
| AMX-R-0120 | D-0003 | primary |
| AMX-R-0121 | D-0003 | primary |
| AMX-R-0122 | D-0003 | primary |
| AMX-R-0123 | D-0003 | primary |
| AMX-R-0124 | D-0003 | primary |
| AMX-R-0125 | D-0003 | primary |
| AMX-R-0126 | D-0003 | primary |
| AMX-R-0127 | D-0029 | primary |
| AMX-R-0128 | D-0029 | refines AMX-R-0026 |
| AMX-R-0129 | D-0029 | primary |
| AMX-R-0130 | D-0029 | primary |
| AMX-R-0131 | D-0029 | primary |
| AMX-R-0132 | D-0029 | primary |
| AMX-R-0133 | D-0029 | primary |
| AMX-R-0134 | D-0029 | primary |
| AMX-R-0135 | D-0029 | primary |
| AMX-R-0136 | D-0029 | primary |
| AMX-R-0137 | D-0029 | primary |
| AMX-R-0138 | D-0029 | primary |
| AMX-R-0139 | D-0029 | primary |
| AMX-R-0140 | D-0029 | refines AMX-R-0013 |
| AMX-R-0141 | D-0023 | primary |
| AMX-R-0142 | D-0023 | primary |
| AMX-R-0143 | D-0023 | primary |
| AMX-R-0144 | D-0023 | primary |
| AMX-R-0145 | D-0023 | primary |
| AMX-R-0146 | D-0023 | primary |
| AMX-R-0147 | D-0023 | primary |
| AMX-R-0148 | D-0024 | primary |
| AMX-R-0149 | D-0024 | primary |
| AMX-R-0150 | D-0024 | primary |
| AMX-R-0151 | D-0024 | primary |
| AMX-R-0152 | D-0024 | primary |
| AMX-R-0153 | D-0024 | primary |
| AMX-R-0154 | D-0024 | primary |
| AMX-R-0155 | D-0032 | primary |
| AMX-R-0156 | D-0032 | primary |
| AMX-R-0157 | D-0032 | primary |
| AMX-R-0158 | D-0032 | primary |
| AMX-R-0159 | D-0032 | primary |
| AMX-R-0160 | D-0032 | primary |
| AMX-R-0161 | D-0032 | primary |
| AMX-R-0163 | D-0025 | primary |
| AMX-R-0164 | D-0019 | primary |
| AMX-R-0165 | D-0019 | primary |
| AMX-R-0166 | D-0019 | primary |
| AMX-R-0167 | D-0019 | primary |
| AMX-R-0168 | D-0019 | primary |
| AMX-R-0169 | D-0019 | primary |
| AMX-R-0170 | D-0019 | primary |
| AMX-R-0171 | D-0019 | primary |
| AMX-R-0172 | D-0019 | primary |
| AMX-R-0173 | D-0019 | primary |
| AMX-R-0174 | D-0019 | primary |
| AMX-R-0175 | D-0019 | primary |
| AMX-R-0176 | D-0019 | primary |
| AMX-R-0177 | D-0019 | primary |
| AMX-R-0178 | D-0019 | primary |
| AMX-R-0179 | D-0019 | primary |
| AMX-R-0180 | D-0019 | primary |
| AMX-R-0181 | D-0019 | primary |
| AMX-R-0182 | D-0019 | primary |
| AMX-R-0183 | D-0019 | refines AMX-R-0040/0108 |
| AMX-R-0184 | D-0019 | primary |
| AMX-R-0185 | D-0019 | primary |
| AMX-R-0186 | D-0033 | primary |
| AMX-R-0187 | D-0033 | refines AMX-R-0026 |
| AMX-R-0188 | D-0033 | primary |
| AMX-R-0189 | D-0033 | primary |
| AMX-R-0190 | D-0033 | primary |
| AMX-R-0191 | D-0033 | primary |
| AMX-R-0192 | D-0033 | primary |
| AMX-R-0193 | D-0033 | primary |
| AMX-R-0194 | D-0034 | primary |
| AMX-R-0195 | D-0034 | primary |
| AMX-R-0196 | D-0034 | primary |
| AMX-R-0197 | D-0034 | primary |
| AMX-R-0198 | D-0034 | primary |
| AMX-R-0199 | D-0034 | primary |
| AMX-R-0200 | D-0034 | primary |
| AMX-R-0201 | D-0034 | primary |
| AMX-R-0202 | D-0034 | primary |
| AMX-R-0203 | D-0034 | primary |
| AMX-R-0204 | D-0034 | primary |
| AMX-R-0205 | D-0034 | primary |
| AMX-R-0206 | D-0034 | refines AMX-R-0160 |
| AMX-R-0207 | D-0034 | primary |
| AMX-R-0208 | D-0034 | refines AMX-R-0114 |
| AMX-R-0209 | D-0034 | primary |
| AMX-R-0210 | D-0021 | refines AMX-R-0100 |
| AMX-R-0211 | D-0021 | refines AMX-R-0100 |
| AMX-R-0212 | D-0021 | refines AMX-R-0100 |
| AMX-R-0213 | D-0021 | refines AMX-R-0100/0011/0039 |
| AMX-R-0214 | D-0021 | refines AMX-R-0100 |
| AMX-R-0215 | D-0021 | refines AMX-R-0100/0022/0036 |
| AMX-R-0216 | D-0021 | refines AMX-R-0100/0098 |
| AMX-R-0217 | D-0021 | refines AMX-R-0100/0054 |
| AMX-R-0218 | D-0021 | refines AMX-R-0100/0054 |
| AMX-R-0219 | D-0021 | refines AMX-R-0100/0028/0101/0102 |
| AMX-R-0220 | D-0021 | alias_of AMX-R-0103 |
| AMX-R-0221 | D-0021 | refines AMX-R-0100 |
| AMX-R-0222 | D-0022 | refines AMX-R-0164 |
| AMX-R-0223 | D-0022 | refines AMX-R-0164/0021 |
| AMX-R-0224 | D-0022 | refines AMX-R-0164 |
| AMX-R-0225 | D-0022 | refines AMX-R-0164 |
| AMX-R-0226 | D-0022 | refines AMX-R-0164 |
| AMX-R-0227 | D-0022 | alias_of AMX-R-0181 |
| AMX-R-0228 | D-0022 | refines AMX-R-0164 |
| AMX-R-0229 | D-0022 | refines AMX-R-0164/0123 |
| AMX-R-0230 | D-0022 | refines AMX-R-0164/0055 |
| AMX-R-0231 | D-0022 | refines AMX-R-0164/0025 |
| AMX-R-0232 | D-0035 | refines AMX-R-0158 |
| AMX-R-0233 | D-0035 | refines AMX-R-0158 |
| AMX-R-0234 | D-0035 | refines AMX-R-0158 |
| AMX-R-0235 | D-0035 | refines AMX-R-0158 |
| AMX-R-0236 | D-0035 | refines AMX-R-0158 |
| AMX-R-0237 | D-0035 | refines AMX-R-0158 |
| AMX-R-0238 | D-0035 | refines AMX-R-0158/0011 |
| AMX-R-0239 | D-0035 | refines AMX-R-0158/0016 |
| AMX-R-0240 | D-0035 | refines AMX-R-0158 |
| AMX-R-0241 | D-0035 | refines AMX-R-0158 |
| AMX-R-0242 | D-0034 | primary |
| AMX-R-0243 | D-0034 | primary |
| AMX-R-0244 | D-0034 | primary |

Reserved serial: `AMX-R-0162` — no published requirement; status unresolved.

### 9.2 ECM per-ID ledger

| Requirement | Primary DifferenceRecord | Catalog relation |
|---|---|---|
| ECM-R-0001 | D-0001 | primary |
| ECM-R-0002 | D-0001 | primary |
| ECM-R-0003 | D-0001 | primary |
| ECM-R-0004 | D-0001 | primary |
| ECM-R-0005 | D-0001 | primary |
| ECM-R-0006 | D-0001 | primary |
| ECM-R-0007 | D-0001 | primary |
| ECM-R-0008 | D-0001 | primary |
| ECM-R-0009 | D-0001 | primary |
| ECM-R-0010 | D-0001 | primary |
| ECM-R-0011 | D-0002 | primary |
| ECM-R-0012 | D-0002 | primary |
| ECM-R-0013 | D-0002 | refines ECM-R-0021 |
| ECM-R-0014 | D-0002 | primary |
| ECM-R-0015 | D-0002 | primary |
| ECM-R-0016 | D-0002 | primary |
| ECM-R-0017 | D-0002 | primary |
| ECM-R-0018 | D-0002 | primary |
| ECM-R-0019 | D-0002 | primary |
| ECM-R-0020 | D-0002 | refines ECM-R-0021 |
| ECM-R-0021 | D-0003 | primary |
| ECM-R-0022 | D-0003 | primary |
| ECM-R-0023 | D-0003 | primary |
| ECM-R-0024 | D-0003 | primary |
| ECM-R-0025 | D-0003 | primary |
| ECM-R-0026 | D-0031 | primary |
| ECM-R-0027 | D-0031 | primary |
| ECM-R-0028 | D-0031 | primary |
| ECM-R-0029 | D-0031 | primary |
| ECM-R-0030 | D-0031 | primary |
| ECM-R-0031 | D-0009 | primary |
| ECM-R-0032 | D-0009 | primary |
| ECM-R-0033 | D-0009 | primary |
| ECM-R-0034 | D-0009 | primary |
| ECM-R-0035 | D-0009 | primary |
| ECM-R-0036 | D-0009 | primary |
| ECM-R-0037 | D-0009 | primary |
| ECM-R-0038 | D-0009 | primary |
| ECM-R-0039 | D-0009 | primary |
| ECM-R-0040 | D-0009 | primary |
| ECM-R-0041 | D-0009 | primary |
| ECM-R-0042 | D-0009 | primary |
| ECM-R-0043 | D-0009 | primary |
| ECM-R-0044 | D-0009 | primary |
| ECM-R-0045 | D-0027 | primary |
| ECM-R-0046 | D-0027 | primary |
| ECM-R-0047 | D-0027 | primary |
| ECM-R-0048 | D-0027 | primary |
| ECM-R-0049 | D-0027 | primary |
| ECM-R-0050 | D-0027 | primary |
| ECM-R-0051 | D-0027 | primary |
| ECM-R-0052 | D-0027 | primary |
| ECM-R-0053 | D-0027 | primary |
| ECM-R-0054 | D-0027 | primary |
| ECM-R-0055 | D-0027 | primary |
| ECM-R-0056 | D-0027 | primary |
| ECM-R-0057 | D-0027 | refines ECM-R-0024 |
| ECM-R-0058 | D-0027 | primary |
| ECM-R-0059 | D-0027 | primary |
| ECM-R-0060 | D-0027 | primary |
| ECM-R-0061 | D-0027 | primary |
| ECM-R-0062 | D-0027 | primary |
| ECM-R-0063 | D-0027 | primary |
| ECM-R-0064 | D-0027 | primary |
| ECM-R-0065 | D-0027 | primary |
| ECM-R-0066 | D-0027 | primary |
| ECM-R-0067 | D-0027 | primary |
| ECM-R-0068 | D-0027 | primary |
| ECM-R-0069 | D-0027 | primary |
| ECM-R-0070 | D-0027 | primary |
| ECM-R-0071 | D-0010 | primary |
| ECM-R-0072 | D-0010 | primary |
| ECM-R-0073 | D-0010 | primary |
| ECM-R-0074 | D-0010 | primary |
| ECM-R-0075 | D-0010 | primary |
| ECM-R-0076 | D-0010 | primary |
| ECM-R-0077 | D-0010 | primary |
| ECM-R-0078 | D-0010 | primary |
| ECM-R-0079 | D-0010 | primary |
| ECM-R-0080 | D-0010 | primary |
| ECM-R-0081 | D-0010 | primary |
| ECM-R-0082 | D-0010 | primary |
| ECM-R-0083 | D-0010 | primary |
| ECM-R-0084 | D-0010 | refines ECM-R-0028 |
| ECM-R-0085 | D-0010 | refines ECM-R-0012 |
| ECM-R-0086 | D-0010 | primary |
| ECM-R-0087 | D-0010 | primary |
| ECM-R-0088 | D-0010 | primary |
| ECM-R-0089 | D-0010 | refines ECM-R-0028 |
| ECM-R-0090 | D-0010 | primary |
| ECM-R-0091 | D-0010 | primary |
| ECM-R-0092 | D-0010 | primary |
| ECM-R-0093 | D-0010 | refines ECM-R-0011 |
| ECM-R-0094 | D-0010 | primary |
| ECM-R-0095 | D-0010 | refines ECM-R-0026 |
| ECM-R-0096 | D-0010 | primary |
| ECM-R-0097 | D-0010 | primary |
| ECM-R-0098 | D-0010 | primary |
| ECM-R-0099 | D-0010 | primary |
| ECM-R-0100 | D-0010 | primary |
| ECM-R-0101 | D-0010 | primary |
| ECM-R-0102 | D-0010 | primary |
| ECM-R-0103 | D-0026 | primary |
| ECM-R-0104 | D-0026 | primary |
| ECM-R-0105 | D-0026 | primary |
| ECM-R-0106 | D-0026 | primary |
| ECM-R-0107 | D-0028 | primary |
| ECM-R-0108 | D-0028 | primary |
| ECM-R-0109 | D-0028 | primary |
| ECM-R-0110 | D-0028 | refines ECM-R-0028 |
| ECM-R-0111 | D-0028 | primary |
| ECM-R-0112 | D-0028 | primary |
| ECM-R-0113 | D-0028 | primary |
| ECM-R-0114 | D-0028 | primary |
| ECM-R-0115 | D-0028 | primary |
| ECM-R-0116 | D-0028 | primary |
| ECM-R-0117 | D-0028 | primary |
| ECM-R-0118 | D-0028 | primary |
| ECM-R-0119 | D-0019 | primary |
| ECM-R-0120 | D-0019 | primary |
| ECM-R-0121 | D-0019 | primary |
| ECM-R-0122 | D-0019 | primary |
| ECM-R-0123 | D-0019 | refines ECM-R-0024 |
| ECM-R-0124 | D-0019 | primary |
| ECM-R-0125 | D-0019 | primary |
| ECM-R-0126 | D-0019 | primary |
| ECM-R-0127 | D-0019 | primary |
| ECM-R-0128 | D-0019 | primary |
| ECM-R-0129 | D-0029 | primary |
| ECM-R-0130 | D-0029 | primary |
| ECM-R-0131 | D-0029 | primary |
| ECM-R-0132 | D-0029 | primary |
| ECM-R-0133 | D-0029 | primary |
| ECM-R-0134 | D-0029 | primary |
| ECM-R-0135 | D-0029 | alias_of ECM-R-0091 |
| ECM-R-0136 | D-0029 | primary |
| ECM-R-0137 | D-0029 | primary |
| ECM-R-0138 | D-0029 | primary |
| ECM-R-0139 | D-0029 | primary |
| ECM-R-0140 | D-0029 | primary |
| ECM-R-0141 | D-0029 | refines ECM-R-0029 |
| ECM-R-0142 | D-0029 | refines ECM-R-0029 |
| ECM-R-0143 | D-0030 | primary |
| ECM-R-0144 | D-0030 | primary |
| ECM-R-0145 | D-0030 | primary |
| ECM-R-0146 | D-0030 | primary |
| ECM-R-0147 | D-0030 | refines ECM-R-0011 |
| ECM-R-0148 | D-0030 | primary |
| ECM-R-0149 | D-0031 | primary |
| ECM-R-0150 | D-0031 | primary |
| ECM-R-0151 | D-0031 | primary |
| ECM-R-0152 | D-0031 | primary |
| ECM-R-0153 | D-0031 | primary |
| ECM-R-0154 | D-0040 | refines ECM-R-0122/0125 |
| ECM-R-0155 | D-0040 | primary |
| ECM-R-0156 | D-0040 | primary |
| ECM-R-0157 | D-0040 | primary |
| ECM-R-0158 | D-0031 | primary |
| ECM-R-0159 | D-0032 | primary |
| ECM-R-0160 | D-0032 | primary |
| ECM-R-0161 | D-0032 | primary |
| ECM-R-0162 | D-0032 | refines ECM-R-0011 |
| ECM-R-0163 | D-0032 | primary |
| ECM-R-0164 | D-0032 | primary |
| ECM-R-0165 | D-0032 | primary |
| ECM-R-0166 | D-0032 | primary |
| ECM-R-0167 | D-0032 | primary |
| ECM-R-0168 | D-0032 | primary |
| ECM-R-0169 | D-0032 | primary |
| ECM-R-0170 | D-0032 | primary |
| ECM-R-0171 | D-0032 | primary |
| ECM-R-0172 | D-0032 | primary |
| ECM-R-0173 | D-0032 | primary |
| ECM-R-0174 | D-0032 | primary |
| ECM-R-0175 | D-0008 | primary |
| ECM-R-0176 | D-0008 | primary |
| ECM-R-0177 | D-0008 | primary |
| ECM-R-0178 | D-0008 | primary |
| ECM-R-0179 | D-0008 | primary |
| ECM-R-0180 | D-0008 | alias_of ECM-R-0135 |
| ECM-R-0181 | D-0008 | primary |
| ECM-R-0182 | D-0008 | alias_of ECM-R-0101 |
| ECM-R-0183 | D-0008 | primary |
| ECM-R-0184 | D-0008 | alias_of ECM-R-0037 |
| ECM-R-0185 | D-0034 | aggregate_of/refined_by ECM-R-0195/0196 |
| ECM-R-0186 | D-0034 | primary |
| ECM-R-0187 | D-0034 | primary |
| ECM-R-0188 | D-0034 | primary |
| ECM-R-0189 | D-0034 | primary |
| ECM-R-0190 | D-0034 | aggregate_of ECM-R-0164-0173 |
| ECM-R-0191 | D-0034 | primary |
| ECM-R-0192 | D-0034 | primary |
| ECM-R-0193 | D-0034 | primary |
| ECM-R-0194 | D-0034 | primary |
| ECM-R-0195 | D-0034 | primary |
| ECM-R-0196 | D-0034 | primary |
| ECM-R-0197 | D-0034 | primary |
| ECM-R-0198 | D-0034 | primary |
| ECM-R-0199 | D-0034 | primary |
| ECM-R-0200 | D-0034 | refines ECM-R-0133 |
| ECM-R-0201 | D-0034 | alias_of ECM-R-0051 |
| ECM-R-0202 | D-0034 | refines ECM-R-0029 |
| ECM-R-0203 | D-0034 | refines ECM-R-0021/0047/0048 |
| ECM-R-0204 | D-0034 | refines ECM-R-0002/0026/0073/0095 |
| ECM-R-0205 | D-0034 | refines ECM-R-0105/0142 |
| ECM-R-0206 | D-0034 | refines ECM-R-0028 |
| ECM-R-0207 | D-0034 | refines ECM-R-0024 |
| ECM-R-0208 | D-0034 | refines ECM-R-0026/0092/0095 |
| ECM-R-0209 | D-0034 | refines ECM-R-0020/0034/0162 |
| ECM-R-0210 | D-0034 | refines ECM-R-0006/0119/0120/0129 |
| ECM-R-0211 | D-0035 | primary |
| ECM-R-0212 | D-0035 | primary |
| ECM-R-0213 | D-0035 | primary |
| ECM-R-0214 | D-0035 | aggregate_of ECM-R-0129-0131 |
| ECM-R-0215 | D-0035 | alias_of ECM-R-0131 |
| ECM-R-0216 | D-0035 | primary |
| ECM-R-0217 | D-0035 | aggregate_of ECM-R-0021/0047/0063/0100/0162 |
| ECM-R-0218 | D-0035 | aggregate_of ECM-R-0011/0012/0065/0066 |
| ECM-R-0219 | D-0035 | refines ECM-R-0028 |
| ECM-R-0220 | D-0035 | refines ECM-R-0024 |
| ECM-R-0221 | D-0035 | aggregate_of ECM-R-0104/0105/0124/0142 |
| ECM-R-0222 | D-0035 | aggregate_of ECM-R-0115/0116/0123 |
| ECM-R-0223 | D-0035 | alias_of ECM-R-0119 |
| ECM-R-0224 | D-0035 | aggregate_of ECM-R-0125-0128 |
| ECM-R-0225 | D-0035 | alias_of ECM-R-0116 |
| ECM-R-0226 | D-0035 | aggregate_of ECM-R-0092/0099/0143-0148 |
| ECM-R-0227 | D-0035 | refines ECM-R-0029 |
| ECM-R-0228 | D-0035 | primary |
| ECM-R-0229 | D-0035 | refines ECM-R-0082/0086 |
| ECM-R-0230 | D-0035 | alias_of ECM-R-0083 |
| ECM-R-0231 | D-0035 | alias_of ECM-R-0100 |
| ECM-R-0232 | D-0035 | aggregate_of ECM-R-0076-0079/0117-0128 |
| ECM-R-0233 | D-0035 | refines ECM-R-0093/0147 |
| ECM-R-0234 | D-0035 | aggregate_of ECM-R-0045/0049 |
| ECM-R-0235 | D-0035 | alias_of ECM-R-0060 |
| ECM-R-0236 | D-0035 | refines ECM-R-0058 |
| ECM-R-0237 | D-0035 | alias_of ECM-R-0107 |
| ECM-R-0238 | D-0035 | refines ECM-R-0024 |
| ECM-R-0239 | D-0035 | broad aggregate_of prior lifecycle obligations |
| ECM-R-0240 | D-0035 | alias_of ECM-R-0054 |
| ECM-R-0241 | D-0035 | aggregate_of ECM-R-0155-0157 |
| ECM-R-0242 | D-0035 | alias_of ECM-R-0158 |
| ECM-R-0243 | D-0035 | primary |
| ECM-R-0244 | D-0035 | refines ECM-R-0019/0054/0240 |

## 10. Round 2 conclusion

The sources are compatible only if canonical ownership is partitioned by domain. The three direct conflicts require explicit adjudication; the missing safeguards and unsupported lifecycle semantics remain critical blockers. No implementation should begin until the blockers in §7 are resolved by a separately versioned reconciliation decision.
