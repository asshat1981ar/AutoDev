# Multi-Repository CodeRabbit Control Plane Design

Status: Proposed specification, revised after adversarial review
Date: 2026-08-21

## 1. Purpose and authority scope

Build a deterministic, Git-authoritative control plane for **repository-managed** CodeRabbit review policy across multiple repositories. The control plane compiles centrally governed security and verification invariants, reusable language/domain profiles, and repository-local policy into artifacts such as `.coderabbit.yaml`, review/security Markdown, pull-request templates, and narrowly scoped ast-grep rules.

The control plane validates its repository-managed effective policy, detects drift, emits immutable evidence, and synchronizes generated artifacts through isolated branches and pull requests. It does not create a second software-development execution authority. GitHub remains authoritative for repository source/revision/branch/PR state, and each target system retains its own execution authority.

### 1.1 Repository-managed guarantee

The MVP guarantee is intentionally narrower than CodeRabbit's complete hosted effective configuration:

- the control plane is authoritative only for the catalog, repository manifests, exceptions, and generated repository artifacts that it owns;
- CodeRabbit workspace- or organization-level Global Overrides are a higher hosted authority and may supersede repository `.coderabbit.yaml`;
- the MVP does not write Global Overrides and does not claim that `EffectivePolicySet` equals CodeRabbit's final hosted effective configuration when such overrides exist;
- hosted override state is represented only as a `HostedOverrideObservation` in evidence: `unknown`, `not_configured`, or `observed`, plus an immutable evidence reference/digest when an authorized read is available;
- `unknown` or `observed` hosted override state changes the assurance label to `repository_managed_only`; it does not silently alter repository policy resolution;
- drift guarantees therefore apply to repository-managed inputs and generated artifacts unless a later phase adds an authenticated, immutable hosted-override input.

No successful report may state "complete CodeRabbit effective policy" unless every higher-precedence hosted authority has been independently captured and authenticated.

## 2. MVP goals

The MVP must:

1. Manage repository CodeRabbit configuration for multiple repositories from one central policy catalog.
2. Enforce a hybrid authority model: mandatory core, shared profiles, and repository-local policy.
3. Prevent repository-local weakening of mandatory core unless a valid, verifiably approved exception authorizes the exact delta.
4. Discover repository characteristics from objective evidence and attach confidence to inferred profiles.
5. Pin all replay-relevant Git inputs to immutable revisions before resolution.
6. Resolve policy deterministically and compile byte-reproducible artifacts.
7. Generate CodeRabbit YAML, Markdown policy, pull-request templates, and ast-grep rules only when the target is explicitly supported and owned.
8. Validate source schemas, resolution, exception approval, generated CodeRabbit YAML against a pinned schema, ast-grep configuration/rules/tests, generated-path ownership, and output drift.
9. Confine writes to a versioned machine-readable generated-path manifest and isolated workspaces.
10. Synchronize through revision-checked dedicated branches and pull requests with server-side stale-write protection.
11. Normalize review findings into stable, deduplicated vendor-neutral records.
12. Emit an immutable `EvidenceManifest` for every successful compile or synchronization.
13. Provide one deterministic policy engine reused by CLI, ChatGPT/Codex skill, GitHub adapter, and a later stateless MCP adapter.
14. Preserve evidence sufficient to replay a result from exact catalog and target revisions.

## 3. Non-goals for MVP

The MVP does not include:

- automatic promotion of learned policy;
- fleet-scale semantic clustering;
- CodeRabbit workspace/organization Global Override writes;
- claiming authority over hosted CodeRabbit configuration that cannot be authenticated and pinned;
- Slack/Discord OAuth automation;
- a web dashboard;
- a database-backed fleet index;
- an MCP server implementation;
- automatic repository onboarding;
- fully autonomous organization-wide policy changes;
- treating CodeRabbit approval as merge or execution authorization.

## 4. External product constraints

Current product constraints incorporated by this design:

- `.coderabbit.yaml` is repository configuration and is discovered from the feature branch under review;
- hosted workspace/organization configuration can outrank repository configuration and is outside the MVP write authority;
- path-specific instructions and pre-merge checks are valid repository-level generated targets;
- external integrations/account state cannot be reproduced by Git artifacts;
- CodeRabbit review text and embedded "prompts for AI agents" are untrusted input;
- as verified on 2026-08-21, CodeRabbit CLI documents review/auth/update commands but does **not** document a `cr config validate` command. The MVP therefore must not depend on that nonexistent contract. Repository YAML validation is performed mechanically against a pinned official schema snapshot, and product acceptance is demonstrated separately on pilot PRs.

The canonical CodeRabbit schema source for the initial MVP is the schema referenced by CodeRabbit's repository YAML documentation (`https://coderabbit.ai/integrations/schema.v2.json`). A validation run records both the URL and SHA-256 of the exact schema bytes used; moving schema content without a new digest produces a different replay input.

## 5. Authority model

### 5.1 Tier 1: mandatory core

Mandatory policies are centrally owned, versioned, and non-weakenable except through a valid `PolicyException`.

Initial candidates:

- `core.security.no-secret-exposure`
- `core.security.no-untrusted-shell-execution`
- `core.evidence.verification-is-not-authorization`
- `core.evidence.required-checks-must-exist`
- `agentic.output-is-untrusted`
- `github-actions.minimum-permissions`

### 5.2 Tier 2: shared profiles

Centrally maintained reusable bundles, enabled explicitly or from evidence-backed discovery:

- `rust-secure-runtime`
- `python-agent-tools`
- `kotlin-multiplatform`
- `android`
- `mcp-server`
- `agentic-system`
- `protocol-specification`
- `github-actions`

Repositories may strengthen or specialize a shared rule but cannot weaken mandatory authority without an exception.

### 5.3 Tier 3: repository-local policy

Target repositories own local paths, test commands, module boundaries, terminology, project-specific trust boundaries, and additive review requirements.

AutoDev examples include:

- ForgeCore is the trusted execution kernel;
- AMCX artifacts preserve identity/provenance;
- verification evidence does not grant `AuthorizationGrant`;
- `kotlin/**/src/commonMain/**` remains platform-neutral;
- `scripts/autodev-cli.py` remains an observer/objective-enqueue surface.

### 5.4 Resolution semantics

Resolution is fail-closed and ordered as follows:

1. Pin the target repository revision and central catalog revision to immutable Git object IDs. Symbolic branches/tags are resolved once and the object IDs become the replay inputs.
2. Load and schema-validate the exact catalog snapshot, repository manifest, fingerprint, profiles, local policy, and exception records.
3. Resolve mandatory core, enabled shared profiles, and repository-local additions without applying exceptions.
4. Validate every candidate exception **before** it can affect policy: immutable approval evidence, authorized approver identity/class, protected approval revision, repository/policy/path/operation scope, compensating controls, and UTC validity window.
5. Apply only the explicit policy delta authorized by each valid exception. An exception is not a wildcard waiver and cannot affect an unlisted policy/path/operation.
6. Recompute the candidate effective set and reject every remaining mandatory-core weakening.
7. Detect contradiction/supersession errors and emit the final repository-managed `EffectivePolicySet` plus deterministic digest.

Exception time semantics use RFC 3339 UTC instants and the half-open interval `[created_at, expires_at)`: an exception is valid at `created_at` and invalid at the exact `expires_at` instant. Clock/time input used for validation is recorded in evidence.

## 6. Canonical data model

All human-authored YAML validates against versioned JSON Schema. Canonical JSON serialization used for digests sorts object keys, normalizes Unicode to NFC, preserves array order where semantically ordered, and excludes presentation-only whitespace/comments.

### 6.1 RepositoryManifest

Owner-authored repository intent. Required fields:

- `apiVersion`, `kind`;
- repository provider/full name;
- explicit profiles/local policy IDs;
- generation targets;
- discovery settings;
- generated-path manifest reference.

Canonical repository identity is `github:<owner>/<repo>` for GitHub repositories.

### 6.2 RepositoryFingerprint

Generated observation, not authority. Required fields:

- canonical repository ID and immutable target revision;
- detected languages/capabilities with confidence and evidence paths;
- trust-boundary candidates with confidence;
- discovered verification commands and their evidence;
- relevant CI/build files.

Every inferred capability that changes policy selection retains evidence references.

### 6.3 PolicyDefinition

Required fields:

- semantic ID/version;
- maturity/status/owner;
- authority tier/weakenability;
- applicability constraints/severity;
- semantic requirement;
- compiler targets;
- optional mechanical detection definition;
- dependencies/supersession metadata.

Versioning: patch = non-semantic metadata/wording; minor = compatible stronger/additive coverage; major = changed meaning/scope/incompatible enforcement.

### 6.4 PolicyException

A `PolicyException` is ineffective unless its approval is independently verifiable. Required fields:

- stable exception ID;
- canonical repository ID;
- policy ID/version constraint;
- exact authorized delta;
- minimal path/operation scope;
- reason and compensating controls;
- `created_at` and `expires_at` as RFC 3339 UTC instants with `[created_at, expires_at)` semantics;
- `approval_ref`: immutable provider object ID, signed record, or protected Git object reference;
- `approver_identity` and allowed `approver_class`;
- `approval_revision`: the protected revision containing/binding the approval;
- `scope_digest`: SHA-256 of canonical repository/policy/delta/path/operation/expiry scope;
- `approval_evidence_digest` binding approver identity, protected revision, scope digest, and expiry.

Validation rejects mutable/unprotected approval references, self-approval where policy forbids it, mismatched scope digests, unauthorized approvers, expired/not-yet-valid records, and any attempt to broaden the approved delta.

### 6.5 EffectivePolicySet

Generated canonical compiler input. Required provenance:

- canonical repository ID and immutable target revision;
- repository manifest/fingerprint digests;
- immutable `catalog_revision` (Git commit/tree object ID) and catalog content digest;
- resolved profile/policy versions and origins;
- active exception IDs plus approval evidence digests;
- `HostedOverrideObservation` and assurance label;
- canonical replay-input digest;
- deterministic resolution digest.

`resolve_policy` accepts a pinned catalog snapshot, never a moving branch. Replay must use the same `catalog_revision`, target revision, schemas, exception approval evidence, and clock instant where time validity affects the result.

### 6.6 FindingRecord

A normalized finding has stable occurrence identity.

Canonical occurrence key input is the canonical JSON tuple:

```text
source_provider
source_review_or_comment_id
repository_id
reviewed_revision
normalized_path
normalized_line_or_range
candidate_invariant_id_or_digest
```

`occurrence_key = sha256(canonical_json(tuple))` and `finding_id = "finding:sha256:" + occurrence_key`.

Normalization rules are versioned. Path separators use `/`; repository-relative paths contain no `.`/`..`; line ranges are `start:end` with inclusive start/end; absent line is `null`; candidate invariants use a stable semantic ID when available, otherwise SHA-256 of normalized invariant text.

Reprocessing the same canonical key updates the existing record/last-seen evidence rather than creating a duplicate. A changed reviewed revision or materially changed invariant creates a distinct occurrence while retaining an optional `supersedes`/`related_findings` relation. Learning metrics count canonical occurrences, not ingestion attempts.

### 6.7 EvidenceManifest

Every successful compile/sync emits an immutable manifest containing:

- run ID and manifest schema version;
- compiler version/build digest;
- canonical target repository/revision;
- catalog revision/content digest;
- effective-policy/replay-input digests;
- generated artifact digests;
- schema/resolution/weakening/exception/generated-path/CodeRabbit-schema/ast-grep/drift results;
- tool versions and command exit codes;
- hosted-override observation/assurance label;
- overall result.

Storage contract:

```text
.coderabbit/evidence/runs/<run-id>.yaml   # immutable manifest
.coderabbit/evidence/latest.yaml          # pointer/index only
```

`latest.yaml` contains only run ID, immutable manifest path, manifest SHA-256, target revision, and created timestamp. Writers refuse to overwrite an existing run path or reuse a run ID with different content. Evidence referenced by an open/merged synchronization PR is retained; retention may archive older runs but cannot mutate their bytes/digests.

No successful validation claim exists without an immutable per-run manifest.

### 6.8 GeneratedPathManifest

`.coderabbit/generated-paths.v1.yaml` is owner-authored, versioned write authority. Each entry records:

- anchored repository-relative path or narrowly bounded directory pattern;
- `owner: repository | control_plane`;
- compiler target;
- write mode (`replace` for MVP generated files; owner-authored paths are never replaced);
- optional generated-block marker only if that merge mode is separately implemented/tested;
- expected artifact type.

Rules:

- ambiguous/overlapping ownership entries are invalid;
- paths containing traversal, absolute prefixes, or symlink escapes fail closed;
- every write is checked against the manifest immediately before mutation;
- unlisted paths are denied;
- `owner: repository` files/custom templates may be read and proposed as diffs but are not overwritten;
- a generated path cannot become writable merely because a compiler emitted it.

## 7. Repository layout

Production implementation belongs in a dedicated repository; AutoDev hosts specification/bootstrap artifacts only.

Recommended control-plane repository:

```text
coderabbit-control-plane/
├── policies/{core,languages,domains}/
├── profiles/
├── repositories/
├── schemas/
├── fixtures/{repositories,findings}/
├── src/coderabbit_control/
├── skills/coderabbit-control/
├── generated/
├── docs/{security,operations}/
└── tests/
```

Target repository example:

```text
.coderabbit/
├── repository.yaml                 # owner-authored
├── local-policy.yaml               # owner-authored
├── exceptions.yaml                 # owner-authored, approval refs only
├── generated-paths.v1.yaml         # owner-authored write authority
├── GENERATED.md                     # generated only if manifest-owned
└── evidence/
    ├── latest.yaml                  # generated pointer/index
    └── runs/<run-id>.yaml           # immutable generated evidence

.coderabbit.yaml                     # generated only if manifest-owned
.github/PULL_REQUEST_TEMPLATE.md      # generated only if explicitly manifest-owned
docs/coderabbit/CODERABBIT_REVIEW_POLICY.md
.ast-grep/sgconfig.yml
.ast-grep/rules/**
.ast-grep/tests/**
```

For ast-grep output, `.ast-grep/sgconfig.yml` is generated/validated with paths relative to that config, for example:

```yaml
ruleDirs:
  - rules
testConfigs:
  - testDir: tests
```

## 8. Deterministic processing pipeline

### 8.1 Discover

Read only high-signal repository evidence: project instructions, README, CI, language/build manifests, module structure, existing CodeRabbit/ast-grep config, and architecture/security/contribution docs. Output is a non-mutating `RepositoryFingerprint` bound to an immutable target revision.

### 8.2 Classify

Structural rules run first; semantic inference can only propose low-confidence profiles. Profile states are `mandatory`, `detected`, `suggested`, `explicit`. Suggested profiles never silently become effective.

### 8.3 Resolve

Before resolution, the caller supplies/resolves both immutable `target_revision` and `catalog_revision`. The engine loads exactly those objects and follows section 5.4. A moving branch name is metadata only and cannot participate in the resolution digest.

### 8.4 Compile

Compilers consume only `EffectivePolicySet`. Supported targets include CodeRabbit repository YAML/path instructions, Markdown policy, PR checklist, and ast-grep rules/tests where mechanical detection is sound. Each policy declares supported targets. Semantic-only invariants explicitly declare `ast_grep.supported: false`.

Compilation produces a `GeneratedArtifactSet`; it does not write the repository.

### 8.5 Validate

Required gates are executable and evidence-producing:

1. source schema validation;
2. immutable target/catalog revision validation;
3. contradiction/dependency/supersession checks;
4. exception approval/scope/time validation;
5. remaining mandatory-weakening detection after authorized exception application;
6. `GeneratedPathManifest` ownership/confinement validation;
7. generated YAML parse validation;
8. generated `.coderabbit.yaml` validation against the exact pinned official schema snapshot; record schema URL, SHA-256, validator version, and exit code `0`;
9. ast-grep project validation using the pinned implementation version:
   - `ast-grep scan --config .ast-grep/sgconfig.yml .`
   - `ast-grep test --config .ast-grep/sgconfig.yml`
   Both must exit `0`; positive/negative fixtures must be present for generated rules;
10. generated-file drift check;
11. repository-specific invariant tests;
12. deterministic replay from the recorded target/catalog/schema/tool inputs.

As of 2026-08-21 the documented CodeRabbit CLI does not provide `cr config validate`; the control plane must not fabricate that command. Product-level acceptance is a separate pilot gate: CodeRabbit must detect/use the feature-branch `.coderabbit.yaml` without reporting configuration rejection, and the associated PR/review evidence is retained in the `EvidenceManifest`.

### 8.6 Synchronize

Synchronization is a two-revision transaction over target base and dedicated sync branch:

1. Read immutable `expected_target_head` and the current dedicated sync-branch head (if it exists).
2. Create an isolated workspace from `expected_target_head`.
3. Resolve/compile/validate with pinned target/catalog revisions.
4. Re-read target HEAD immediately before preparing the write; abort on mismatch.
5. Verify the synchronization branch belongs to the control plane and capture `expected_sync_head`.
6. Apply only paths authorized by `GeneratedPathManifest`; revalidate path/symlink ownership immediately before each mutation.
7. Create a commit whose parent is the captured sync head (or expected target head for a new branch).
8. Update the sync ref with a server-side expected-old-revision lease/CAS. The adapter must use an atomic compare-and-swap/lease primitive (for example Git's expected-old-object ref update semantics); no unconditional force update is permitted. If the provider cannot enforce the expected old revision, synchronization is unsupported/fails closed.
9. Re-read the written sync ref and require it to equal the new commit.
10. Re-read the target/base HEAD **again** immediately before opening/updating the PR. If it differs from `expected_target_head`, abort and restart; do not present stale artifacts as current.
11. Open/update the PR only after actor/repository/approval authorization succeeds.
12. Observe CI/CodeRabbit and normalize findings.

The only permitted ref transition is the one derived from the captured expected old revision. Branch divergence, concurrent movement, or any base revision mismatch aborts. Direct default-branch writes are prohibited.

## 9. Core engine and adapters

### 9.1 Core engine

Logical interfaces:

```text
discover_repository(repository_id, target_revision) -> RepositoryFingerprint
classify_repository(fingerprint) -> ProfileSelection
resolve_policy(manifest, profiles, exceptions, catalog_snapshot, target_revision, validation_instant) -> EffectivePolicySet
compile_policy(effective_policy) -> GeneratedArtifactSet
validate_artifacts(inputs, artifacts, generated_path_manifest) -> EvidenceManifest
detect_drift(expected, actual) -> DriftReport
normalize_finding(review_record, normalization_version) -> FindingRecord
```

The engine is transport-agnostic and performs no GitHub/UI/MCP writes.

### 9.2 Canonical CLI contract

Repository identifiers use exactly `github:<owner>/<repo>`. Commands must not infer semantic inputs from the process working directory. The working directory is used only for locating an explicitly supplied/default control-plane config root; all repository/catalog data is addressed by canonical IDs and immutable revisions.

Common immutable flags:

```text
--repo github:<owner>/<repo>
--target-revision <immutable-git-oid>
--catalog-revision <immutable-git-oid>
--config-root <path>        # defaults to the control-plane repository root
--format json              # canonical automation output
```

Canonical commands:

```text
crctl discover --repo ... --target-revision ... --format json
crctl classify --repo ... --target-revision ... --format json
crctl resolve --repo ... --target-revision ... --catalog-revision ... --format json
crctl compile --repo ... --target-revision ... --catalog-revision ... --format json
crctl validate --repo ... --target-revision ... --catalog-revision ... --format json
crctl diff --repo ... --target-revision ... --catalog-revision ... --format json
crctl sync --repo ... --target-revision ... --catalog-revision ... --approval-ref ... --format json
crctl audit-fleet --catalog-revision ... --dry-run --format json
crctl triage-pr --repo ... --pr <number> --reviewed-revision <oid> --format json
crctl explain --policy-id <id> --repo ... --catalog-revision ... --format json
```

`stdout` contains one versioned result object (`apiVersion: coderabbit.control/result/v1`) and no human prose in JSON mode. Diagnostics go to `stderr`. Every result echoes canonical repository ID and all revisions used.

Exit codes:

| Code | Meaning |
|---:|---|
| 0 | requested operation completed and all required gates passed |
| 2 | schema/policy/validation rejection |
| 3 | stale target/catalog/sync revision or deterministic replay mismatch |
| 4 | fleet partial failure (at least one repository failed; per-repo results still emitted) |
| 5 | authorization/approval/repository/path write denial |
| 64 | usage/invalid CLI arguments |
| 70 | internal/unclassified failure |

`audit-fleet --dry-run` exits `4` on any per-repository failure; it never returns `0` for partial success.

### 9.3 ChatGPT/Codex skill

The skill orchestrates the CLI/core and GitHub adapter without reimplementing policy semantics. Workflows: bootstrap, audit, upgrade, CodeRabbit triage, fleet audit, and learning-candidate proposal.

### 9.4 GitHub adapter and least-privilege WRITE authority

READ operations and WRITE operations use explicit authority. Before `crctl sync` enables any branch/commit/PR mutation it must validate:

- canonical repository is in a centrally protected target allowlist;
- authenticated actor/service identity is authorized for that repository and operation;
- an immutable approval reference authorizes this sync run/repository/revision/path set;
- credential scope is the minimum necessary. GitHub App/token write permissions are limited to repository contents and pull requests for the allowed target; metadata is read-only; checks/actions access is read-only only when required for observation; administration/organization permissions are forbidden for sync;
- branch name is in the dedicated control-plane namespace and ownership metadata matches the service;
- every path passes `GeneratedPathManifest` validation;
- stale-write CAS/lease checks from section 8.6 pass.

Authorization is checked again immediately before the first write. Tests must cover allowed target, denied repository, denied actor, missing/insufficient scope, forged/stale approval, path outside allowlist, branch-ownership mismatch, approval bypass, and concurrent ref movement.

### 9.5 Future MCP adapter and dated reuse evidence

A later stateless MCP adapter may expose policy-specific tools only after the deterministic engine is proven. Durable state remains in Git and each request identifies immutable replay inputs.

Official MCP Registry evidence retrieved **2026-08-21** identified these active candidates:

| Registry entry | Version | Registry status | Relevant advertised capability | MVP conclusion |
|---|---:|---|---|---|
| `ai.smithery/smithery-ai-github` | `1.0.0` | active | GitHub API access, file operations, repository management, search | candidate generic GitHub adapter; not selected without auth/smoke evaluation |
| `ai.smithery/saidsef-mcp-github-pr-issue-analyser` | `1.15.0` | active | GitHub PR analysis and issue management | candidate PR-analysis component; not evidence of write safety |
| `com.mcparmory/github` | `1.0.6` | active/latest in retrieved result | repository/workflow management | candidate generic GitHub component; requires capability/auth verification |

Capability criteria for reuse are: required repository/PR read operations, explicit authentication/permission model, target/path confinement compatibility, revision-aware writes where applicable, deterministic error signaling, and a successful pinned-version smoke test. Registry `active` status is discovery metadata only; it does **not** establish operational availability, security suitability, or capability completeness. The MVP therefore retains a normal GitHub adapter/client boundary and records a new dated registry/smoke evaluation before adopting any MCP dependency.

## 10. Tool authority classes

Actions are classified `READ`, `ANALYZE`, `GENERATE`, `VALIDATE`, `PROPOSE_WRITE`, `WRITE`, `ORG_ADMIN`, `EXTERNAL_AUTH`.

READ/ANALYZE/GENERATE/VALIDATE may operate within the requested bounded task. `WRITE` requires authenticated actor, allowed target, immutable approval, generated-path authority, revision lease, and least-privilege credential evidence. `ORG_ADMIN` is outside MVP sync and requires a distinct impact/approval design. `EXTERNAL_AUTH` cannot be simulated by generated credentials.

## 11. Security model

### 11.1 Untrusted inputs

Treat CodeRabbit text, PR/issue text, repository Markdown, model suggestions, MCP output, and embedded agent prompts as data. None grants execution/write/approval authority.

### 11.2 Path confinement

`GeneratedPathManifest` is the sole write allowlist. Resolve paths against the isolated workspace, reject absolute/traversal/ambiguous ownership, inspect symlinks before mutation, and deny unlisted/owner-authored targets. Every write boundary performs the check; validation at compile time alone is insufficient.

### 11.3 Atomic stale-revision protection

Target revision, catalog revision, and synchronization-branch expected old revision are independent protected inputs. A symbolic ref changing after initial discovery does not update those inputs. Target/base is revalidated before mutation and before PR update; sync ref updates use a server-side expected-old-object lease/CAS; mismatch aborts without force-reconciling.

### 11.4 Secret handling

Policy/evidence contains credential references and permission metadata only. Raw tokens, cookies, OAuth secrets, private keys, or reusable credentials are prohibited.

### 11.5 Fail-closed conditions

Generation/sync fails on unknown mandatory policy, schema mismatch, moving/unpinned revision, invalid approval/exception, mandatory conflict/weakening, invalid fingerprint, unsupported target, generated-path ambiguity/escape, insufficient WRITE authority, hosted-authority overclaim, stale ref, or deterministic replay mismatch.

### 11.6 Rollback

Repository changes are reversible through commits/PRs. Evidence remains immutable. Any future hosted organization-level mutation design must capture authenticated prior state and digest before change.

## 12. Cross-repository learning

### 12.1 Remediation and learning are separate

A finding may produce an immediate local fix and an independent learning candidate. One finding never auto-promotes shared policy.

### 12.2 Candidate scope

Narrowest first: repository, language profile, domain profile, security/evidence core, organization-wide.

### 12.3 Maturity lifecycle

`OBSERVED -> CANDIDATE -> EXPERIMENTAL -> RECOMMENDED -> MANDATORY`

Promotion uses canonical deduplicated findings and considers recurrence, repository diversity, severity, fix stability, mechanical detectability, false-positive rate, exceptions, reach/impact/confidence/cost. Experimental policy uses canaries and historical replay.

### 12.4 Provenance and retirement

Promoted records retain canonical origin finding IDs, tested repositories/revisions, true/false-positive evidence, and approval. Policy supports `DEPRECATED`, `DISABLED`, `SUPERSEDED`.

## 13. Initial policy catalog

Initial high-confidence candidates:

1. `core.security.no-secret-exposure`
2. `core.security.no-untrusted-shell-execution`
3. `core.evidence.verification-is-not-authorization`
4. `core.evidence.required-checks-must-exist`
5. `agentic.output-is-untrusted`
6. `github-actions.minimum-permissions`
7. `python.no-shell-true`
8. `rust.no-unchecked-trusted-boundary-panic`
9. `kotlin.commonmain-platform-purity`
10. `mcp.untrusted-tool-input`

AutoDev-specific policies remain repository-local.

## 14. Testing strategy

### 14.1 Unit/schema/resolver tests

Required tests:

- invalid/unknown policy schema rejection;
- immutable catalog/target revision required; moving symbolic refs do not affect a pinned run;
- exception approval identity/revision/scope digest verification;
- `[created_at, expires_at)` boundary tests including exact expiry instant;
- unauthorized/broad/expired exception rejection;
- authorized scoped exception applied before remaining mandatory-weakening check;
- strengthening acceptance and conflict detection;
- deterministic resolution/replay digest.

### 14.2 Finding/evidence integrity tests

- canonical FindingRecord key stable across ingestion replay;
- duplicate ingestion updates, not duplicates;
- changed revision/invariant creates a distinct occurrence;
- immutable evidence run path cannot be overwritten;
- `latest.yaml` points to and hashes an existing immutable run manifest;
- evidence referenced by synchronization cannot be silently mutated.

### 14.3 Golden compiler/generated-path tests

Fixtures cover Rust, Python agent tool, Kotlin Multiplatform, MCP, and mixed agent runtime. Fixed inputs produce byte-identical artifacts. Tests cover every generated path, denied unlisted/owner-authored writes, overlapping ownership, `../`, absolute path, symlink escape, and customized PR-template preservation.

### 14.4 Mechanical validator tests

- generated `.coderabbit.yaml` parses and validates against pinned schema bytes/digest;
- validation evidence records schema URL/hash/validator version/exit code;
- `.ast-grep/sgconfig.yml` resolves `ruleDirs`/`testConfigs` relative to config;
- `ast-grep scan --config .ast-grep/sgconfig.yml .` exits 0;
- `ast-grep test --config .ast-grep/sgconfig.yml` exits 0 with positive/negative fixtures;
- pilot PR evidence demonstrates CodeRabbit accepted/detected generated branch config; no nonexistent CodeRabbit CLI command is used as evidence.

### 14.5 Synchronization/authorization adversarial tests

Required cases:

- target HEAD changes before write;
- target HEAD changes after branch preparation but before PR update;
- sync branch moves concurrently;
- provider lacks expected-old-ref/CAS primitive;
- unauthorized repository/actor;
- insufficient credential scope;
- forged/stale approval;
- branch namespace ownership mismatch;
- generated path escape/symlink;
- review text embeds shell/prompt injection;
- credential-like output.

Every boundary fails closed.

### 14.6 CLI/fleet contract tests

Every command fixture asserts repository/revision echo, versioned JSON output, stderr separation, and exact exit code. `audit-fleet --dry-run` emits all per-repository results and exits `4` if any repository fails.

### 14.7 Property invariants

- stricter policy cannot weaken the effective set;
- local removal cannot remove mandatory policy;
- invalid/expired exceptions cannot affect policy;
- policy input ordering cannot change canonical output;
- identical immutable inputs produce identical digests;
- a writer can never mutate an owner/unlisted path;
- replay cannot inflate FindingRecord counts.

## 15. MVP phases

0. Finalize schemas/threat model/authority and acceptance contracts.
1. Implement typed model, immutable-input resolver, verifiable exceptions, deduplicated findings.
2. Implement compilers, generated-path manifest, CodeRabbit-schema and ast-grep validators, golden/adversarial tests.
3. Implement canonical CLI/result/exit-code contracts and fixtures.
4. Implement dry-run evidence-backed discovery/classification.
5. Implement least-privilege GitHub adapter, revision lease/CAS synchronization, immutable evidence storage.
6. Add ChatGPT/Codex orchestration skill using the same engine.
7. Pilot against three structurally different repositories.
8. Close one real finding -> canonical record -> source-policy correction -> recompile -> sync loop and reproduce from clean immutable inputs.

MCP and automated cross-repository learning remain post-MVP.

## 16. Acceptance criteria

MVP acceptance requires evidence that:

1. Identical immutable target/catalog/schema/tool inputs produce identical repository-managed artifacts/digests.
2. Mandatory policy cannot be weakened except by a verifiably approved, exactly scoped, currently valid exception.
3. Generated writes cannot escape `GeneratedPathManifest` or overwrite owner-authored/customized files.
4. Clean checkout at recorded target/catalog revisions reproduces generated artifacts.
5. At least three structurally different repositories compile from one pinned catalog revision.
6. Manual generated-policy edits are detected as drift.
7. Sync uses dedicated owned branches, least-privilege credentials, immutable approval, expected-old-ref lease/CAS, and final target-base revalidation.
8. Every successful run stores an immutable per-run `EvidenceManifest`; `latest.yaml` is only a verified pointer.
9. Generated `.coderabbit.yaml` passes pinned official-schema validation and is actually accepted/detected by CodeRabbit on pilot PRs, with product evidence retained.
10. At least one real review finding is normalized using the canonical occurrence key, deduplicated, corrected in source policy, recompiled, and synchronized.
11. External review/comment/MCP/model text never grants instruction, write, or approval authority.
12. Any stale target/catalog/sync revision aborts rather than applying stale output.
13. Assurance reports explicitly state `repository_managed_only` whenever higher hosted override authority is unknown or observed but not authenticated as a replay input.
14. CLI commands satisfy their documented JSON/exit-code contracts; fleet partial failure is non-zero.

## 17. Research decisions and reuse evidence

- Preserve one-owner authority: GitHub owns repository/source/revision/branch/PR/CI truth; protected catalog Git history owns control-plane policy; target architecture docs own local architecture intent; memory/learned heuristics never override Git policy state.
- Use CodeRabbit repository YAML/path instructions as generated targets while explicitly separating higher hosted Global Overrides from repository-managed assurance.
- As of 2026-08-21, current CodeRabbit CLI documentation does not list `cr config validate`; use pinned schema validation plus retained pilot product evidence instead of inventing a CLI gate.
- Official ast-grep documentation confirms `sgconfig.yml` `ruleDirs` and `testConfigs`, and `scan`/`test` accept `--config`; those commands are therefore canonical mechanical gates for generated ast-grep projects.
- Official MCP Registry retrieval on 2026-08-21 identified the three active GitHub-related candidates recorded in section 9.5. Registry status is discovery evidence, not operational/security proof. Any reuse decision requires pinned-version smoke/auth/capability evaluation and records retrieval date, version, status, and criteria.
- Automatic CodeRabbit repository linking may inform discovery but is not the authoritative policy map.

## 18. Open implementation constraint

The approved architecture calls for a dedicated `coderabbit-control-plane` repository. This AutoDev PR is specification-only bootstrap material and must not absorb production control-plane logic merely because repository creation or deployment tooling is inconvenient. Implementation begins in the dedicated repository once provisioned, preserving AutoDev/ForgeCore authority boundaries.