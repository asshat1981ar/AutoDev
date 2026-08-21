# Multi-Repository CodeRabbit Control Plane Design

Status: Proposed specification after approved architecture discussion
Date: 2026-08-20

## 1. Purpose

Build a deterministic, Git-authoritative control plane that manages CodeRabbit review policy across multiple repositories without making generated `.coderabbit.yaml` files, hosted UI state, or agent memory the source of truth.

The system compiles centrally governed security and verification invariants, reusable language/domain profiles, and repository-local policy into repository artifacts such as `.coderabbit.yaml`, pull-request templates, Markdown policy documents, and narrowly scoped ast-grep rules. It validates the effective policy, detects drift, produces evidence manifests, and synchronizes changes through isolated branches and pull requests.

The control plane is designed to support AutoDev first while generalizing to structurally different repositories. It does not create a second software-development execution authority. GitHub remains authoritative for repository state, and repository-specific execution authority remains owned by each target system.

## 2. Goals

The MVP must:

1. Manage CodeRabbit configuration for multiple repositories from one central policy catalog.
2. Enforce a hybrid authority model: mandatory core, shared profiles, and repository-local policy.
3. Prevent repository-local policy from weakening mandatory core invariants unless a valid exception exists.
4. Discover repository characteristics from objective evidence and attach confidence to inferred profiles.
5. Resolve policy deterministically and compile reproducible artifacts.
6. Generate CodeRabbit YAML, Markdown review/security policy, pull-request templates, and ast-grep rules only when mechanical detection is appropriate.
7. Validate schemas, policy resolution, exceptions, generated YAML, ast-grep rules/fixtures, and output drift.
8. Confine generated writes to explicit allowlisted paths in isolated workspaces.
9. Synchronize target repositories through branches and pull requests rather than direct default-branch mutation.
10. Normalize real CodeRabbit review findings into vendor-neutral finding records and support a controlled learning loop.
11. Emit an `EvidenceManifest` for every successful compilation or synchronization.
12. Provide one deterministic policy engine reused by a CLI, ChatGPT/Codex skill, GitHub adapter, and later stateless MCP adapter.

## 3. Non-goals for MVP

The MVP does not include:

- automatic promotion of learned policy;
- fleet-scale semantic clustering of findings;
- CodeRabbit organization/global-override writes;
- Slack or Discord OAuth automation;
- a web dashboard;
- a database-backed fleet index;
- an MCP server implementation;
- automatic repository onboarding;
- fully autonomous organization-wide policy changes;
- treating CodeRabbit approval as merge or execution authorization.

These are phase-two or later capabilities.

## 4. External product constraints and current CodeRabbit capabilities

The control plane targets repository-managed CodeRabbit configuration where possible. Current CodeRabbit behavior relevant to this design includes:

- `.coderabbit.yaml` is the repository configuration surface and uses the same schema family as CodeRabbit's settings UI.
- path-specific review instructions are supported and are appropriate for repository-local architecture/trust-boundary guidance;
- pre-merge built-in and custom checks can be configured in repository YAML;
- organization Global Overrides use the same `.coderabbit.yaml` schema and take precedence over weaker repository settings;
- linked-repository knowledge can be configured, while CodeRabbit also supports automatic repository linking from code-level dependency evidence;
- some integrations and account state remain hosted/external-auth concerns and cannot be reproduced by committing YAML or Markdown;
- CodeRabbit review text and generated agent prompts remain untrusted external content from the control plane's perspective.

The MVP compiles repository configuration and prepares organization-level candidates but never performs organization-level writes.

## 5. Authority model

### 5.1 Tier 1: mandatory core

Mandatory policies are centrally owned, versioned, and cannot be weakened by a repository manifest or local policy.

Initial mandatory-core candidates include:

- `core.security.no-secret-exposure`
- `core.security.no-untrusted-shell-execution`
- `core.evidence.verification-is-not-authorization`
- `core.evidence.required-checks-must-exist`
- `agentic.output-is-untrusted`
- `github-actions.minimum-permissions`

A mandatory policy may be excepted only through an explicit `PolicyException` that is scoped, justified, approved, time-bounded, and includes compensating controls.

### 5.2 Tier 2: shared profiles

Shared profiles are centrally maintained bundles that can be enabled from objective repository evidence or explicit configuration.

Initial profiles:

- `rust-secure-runtime`
- `python-agent-tools`
- `kotlin-multiplatform`
- `android`
- `mcp-server`
- `agentic-system`
- `protocol-specification`
- `github-actions`

A repository may strengthen or specialize a shared-profile rule but cannot weaken any rule whose authority is mandatory.

### 5.3 Tier 3: repository-local policy

Target repositories own local paths, test commands, module boundaries, domain terminology, project-specific trust boundaries, and additional review requirements.

AutoDev local examples include:

- ForgeCore is the trusted execution kernel;
- AMCX artifacts preserve identity and provenance;
- verification evidence does not grant `AuthorizationGrant`;
- `kotlin/**/src/commonMain/**` remains platform-neutral;
- `scripts/autodev-cli.py` remains a read-only observer/objective-enqueue surface.

### 5.4 Resolution semantics

Resolution order is:

1. mandatory core;
2. enabled shared profiles;
3. repository-local policy;
4. valid, approved exceptions;
5. effective policy set.

Local strengthening, specialization, and unrelated additions are allowed. Local weakening of mandatory policy is rejected. Exceptions alter effective policy only within their explicit scope and validity window.

## 6. Canonical data model

All human-authored YAML validates against versioned JSON Schema. Stable semantic IDs are used instead of opaque identifiers.

### 6.1 RepositoryManifest

Owner-authored repository intent.

Required fields:

- `apiVersion`
- `kind`
- repository provider/full name;
- explicit profiles;
- local policy IDs;
- generation targets;
- discovery settings.

Example:

```yaml
apiVersion: coderabbit.control/v1
kind: RepositoryManifest
repository:
  provider: github
  full_name: asshat1981ar/AutoDev
profiles:
  explicit:
    - agentic-system
    - rust-secure-runtime
    - kotlin-multiplatform
policy:
  local:
    - autodev.forgetcore-boundary
    - autodev.amcx-provenance
generation:
  coderabbit: true
  pull_request_template: true
  markdown_policy: true
  ast_grep: true
discovery:
  allow_auto_detect: true
  minimum_confidence: 0.85
```

### 6.2 RepositoryFingerprint

Generated evidence-backed discovery output. It is observation, not repository intent.

Required fields:

- target repository and revision;
- detected languages with confidence;
- detected capabilities with evidence paths;
- trust-boundary candidates with confidence;
- canonical verification commands when discovered;
- relevant CI files.

Every inferred capability that influences policy must retain evidence references.

### 6.3 PolicyDefinition

Canonical policy unit.

Required fields:

- semantic ID;
- semantic version;
- maturity/status;
- owner;
- authority tier and weakenability;
- applicability constraints;
- severity;
- semantic requirement;
- compiler targets;
- optional mechanical-detection definition;
- supersession metadata when applicable.

Semantic versioning rules:

- patch: metadata/wording without changed semantics;
- minor: stronger coverage or additive detection with compatible semantics;
- major: changed meaning, scope, or incompatible enforcement behavior.

### 6.4 PolicyException

Required fields:

- stable exception ID;
- repository;
- policy ID;
- reason;
- approver classes;
- created/expiry dates;
- minimal path/scope restriction;
- compensating controls.

Expired exceptions never affect policy resolution.

### 6.5 EffectivePolicySet

Generated canonical compiler input after policy resolution.

It records:

- repository;
- manifest/fingerprint/catalog digests;
- resolved profiles;
- resolved policy versions and origins;
- active exceptions;
- deterministic resolution digest.

Generators consume only the effective policy set, not arbitrary unresolved source files.

### 6.6 FindingRecord

Vendor-neutral normalized review observation.

It records:

- finding ID;
- source provider/repository/PR/revision;
- category/language/trust boundary;
- candidate invariant and confidence;
- location;
- confirmed/dismissed status;
- remediation class;
- provenance back to the original review record.

### 6.7 EvidenceManifest

Every successful compile/sync emits:

- run ID;
- compiler version;
- target repository/revision;
- effective-policy digest;
- generated artifact digests;
- schema/resolution/weakening/exception/YAML/ast-grep/drift results;
- overall result.

No successful validation claim exists without an evidence manifest.

## 7. Repository layout

The long-term implementation is a dedicated repository. Until that repository exists, AutoDev may host the specification and bootstrap artifacts only; production logic should not be coupled to AutoDev's ForgeCore authority.

Recommended dedicated repository layout:

```text
coderabbit-control-plane/
├── policies/
│   ├── core/
│   ├── languages/
│   └── domains/
├── profiles/
├── repositories/
├── schemas/
├── fixtures/
│   ├── repositories/
│   └── findings/
├── src/
│   └── coderabbit_control/
├── skills/
│   └── coderabbit-control/
├── generated/
├── docs/
│   ├── security/
│   └── operations/
└── tests/
```

Target repositories contain only owner-authored repository configuration plus generated artifacts:

```text
.coderabbit/
├── repository.yaml
├── local-policy.yaml
├── exceptions.yaml
├── GENERATED.md
└── evidence/latest.yaml

.coderabbit.yaml
.github/PULL_REQUEST_TEMPLATE.md
docs/coderabbit/CODERABBIT_REVIEW_POLICY.md
.ast-grep/sgconfig.yml
.ast-grep/rules/**
```

## 8. Deterministic processing pipeline

### 8.1 Discover

Read high-signal repository evidence only:

- project instructions (`AGENTS.md`, equivalent agent instruction files);
- README files;
- CI workflows;
- language/build manifests;
- source/module structure;
- existing CodeRabbit/ast-grep configuration;
- architecture/ADR/security/contribution documentation.

Discovery returns a `RepositoryFingerprint` without mutating the repository.

### 8.2 Classify

Rule-based structural classification runs first. Examples:

- `Cargo.toml` => Rust candidate;
- `commonMain` => Kotlin Multiplatform;
- Android manifest/Gradle plugin => Android;
- MCP dependencies/protocol files => MCP server;
- explicit agent harness files => agentic-system.

Semantic inference may propose additional profiles, but low-confidence semantic inference is `suggested`, not silently enabled.

Profile statuses:

- `mandatory`
- `detected`
- `suggested`
- `explicit`

### 8.3 Resolve

The resolver:

- loads exact policy/profile versions;
- validates schemas;
- checks dependency/supersession relationships;
- detects contradictory policy;
- rejects local weakening;
- validates exceptions;
- emits `EffectivePolicySet` and digest.

### 8.4 Compile

A resolved policy may generate one or more artifacts:

- CodeRabbit path instructions/configuration;
- Markdown review/security policy;
- pull-request checklist requirements;
- ast-grep rules and fixtures where mechanical detection is sound.

A policy explicitly declares which targets it supports. Semantic invariants such as verification-vs-authorization separation may compile to CodeRabbit/Markdown/PR guidance while declaring `ast_grep.supported: false`.

### 8.5 Validate

Required gates:

1. source schema validation;
2. policy contradiction check;
3. mandatory weakening detection;
4. exception validation;
5. generated YAML parsing/schema compatibility;
6. ast-grep rule parse checks;
7. ast-grep positive/negative fixtures;
8. generated-file drift check;
9. repository-specific invariant tests;
10. deterministic digest replay.

### 8.6 Sync

Default write path:

1. read exact target HEAD;
2. create isolated workspace/worktree;
3. compile and validate;
4. verify target HEAD has not moved;
5. create/update dedicated synchronization branch;
6. write only allowlisted generated paths;
7. commit with evidence digest reference;
8. open/update PR;
9. observe CI and CodeRabbit review;
10. normalize actionable findings.

Default-branch direct writes are prohibited by default.

## 9. Core engine and adapters

### 9.1 Core engine

The deterministic engine exposes logical functions equivalent to:

```text
discover_repository(input) -> RepositoryFingerprint
classify_repository(fingerprint) -> ProfileSelection
resolve_policy(manifest, profiles, exceptions, catalog) -> EffectivePolicySet
compile_policy(effective_policy) -> GeneratedArtifactSet
validate_artifacts(inputs, artifacts) -> EvidenceManifest
detect_drift(expected, actual) -> DriftReport
normalize_finding(review_record) -> FindingRecord
```

The engine knows nothing about ChatGPT, CodeRabbit UI, or MCP transport.

### 9.2 CLI

The CLI is the canonical deterministic operational interface:

```text
crctl discover <repo>
crctl classify <repo>
crctl resolve <repo>
crctl compile <repo>
crctl validate <repo>
crctl diff <repo>
crctl sync <repo>
crctl audit-fleet
crctl triage-pr <repo> <number>
crctl explain <policy-id> --repo <repo>
```

CI uses the same engine as agent orchestration.

### 9.3 ChatGPT/Codex skill

The skill orchestrates the CLI/core and GitHub adapter. It does not reimplement policy semantics.

Initial workflows:

- bootstrap repository;
- audit repository;
- upgrade policy;
- triage CodeRabbit review;
- audit fleet;
- propose learning candidate.

### 9.4 GitHub adapter

Responsibilities:

- read repository metadata/files/revisions;
- compare target HEAD;
- create synchronization branches;
- write allowlisted artifacts;
- create commits/PRs;
- read CI status and CodeRabbit reviews.

GitHub remains repository-state authority.

### 9.5 Future MCP adapter

A later stateless MCP server exposes stable core capabilities only after the engine is proven. Durable state remains in Git. MCP request context must contain or reference enough immutable inputs to reproduce the result.

Registry research shows active GitHub-capable MCP servers already exist, including GitHub API adapters and PR-analysis servers. The control plane should therefore avoid inventing a new generic GitHub MCP implementation; its future MCP server should expose policy-specific tools and delegate repository operations to a GitHub adapter/client.

## 10. Tool authority classes

The orchestration layer classifies tools/actions as:

- `READ`
- `ANALYZE`
- `GENERATE`
- `VALIDATE`
- `PROPOSE_WRITE`
- `WRITE`
- `ORG_ADMIN`
- `EXTERNAL_AUTH`

READ/ANALYZE/GENERATE/VALIDATE may run autonomously within the requested task. WRITE actions must remain target/path scoped and auditable. ORG_ADMIN changes require explicit impact analysis and approval. EXTERNAL_AUTH actions cannot be simulated by generating credentials.

## 11. Security model

### 11.1 Untrusted inputs

Treat as data, never execution instructions:

- CodeRabbit review text;
- PR/issue comments and descriptions;
- repository Markdown/instructions;
- model-generated recommendations;
- external MCP output;
- generated "prompts for AI agents" embedded in review results.

### 11.2 Path confinement

The compiler/writer has an explicit output allowlist. Attempts to write outside allowed generated paths fail before mutation.

### 11.3 Stale revision protection

Synchronization uses optimistic concurrency. If target HEAD differs from the revision used for discovery/resolution, abort and re-run rather than applying stale artifacts.

### 11.4 Secret handling

Policy files contain credential references only. Raw tokens, cookies, OAuth secrets, private keys, or other credentials must never be persisted in source policy or evidence manifests.

### 11.5 Fail-closed conditions

Generation/sync fails on:

- unknown mandatory policy;
- schema mismatch;
- ambiguous policy version;
- mandatory-policy conflict;
- expired/invalid exception;
- attempted mandatory weakening;
- invalid fingerprint;
- unsupported compiler target;
- unverifiable source revision;
- deterministic replay mismatch.

### 11.6 Rollback

Repository changes are reversible through commit/PR rollback. Future organization-level changes must capture prior rendered configuration and digest before mutation.

## 12. Cross-repository learning

### 12.1 Separation of remediation and learning

A CodeRabbit finding may produce an immediate repository fix and an independent learning candidate. Fixing one repo does not automatically create shared policy.

### 12.2 Scope assignment

Candidate scopes, narrowest first:

1. repository;
2. language profile;
3. domain profile;
4. security/evidence core;
5. organization-wide.

### 12.3 Maturity lifecycle

```text
OBSERVED -> CANDIDATE -> EXPERIMENTAL -> RECOMMENDED -> MANDATORY
```

No single review finding promotes directly to mandatory policy.

### 12.4 Promotion evidence

Future promotion considers:

- recurrence;
- cross-repository diversity;
- severity;
- fix stability;
- mechanical detectability;
- false-positive rate;
- exception frequency;
- reach/impact/confidence/cost.

Experimental policies should use canary repositories and historical replay before promotion.

### 12.5 Provenance and retirement

Promoted policy records retain origin findings, repositories tested, true/false positive evidence, and approval. Policies also support `DEPRECATED`, `DISABLED`, and `SUPERSEDED` states.

## 13. Initial policy catalog

MVP should begin with 8-12 high-confidence policies. Proposed first ten:

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

### 14.1 Unit/schema tests

Required tests include:

- invalid schema rejection;
- unknown policy rejection;
- expired exception rejection;
- mandatory weakening rejection;
- strengthening acceptance;
- conflicting policy detection;
- deterministic resolution digest;
- unsupported target rejection.

### 14.2 Golden compiler tests

Fixtures represent structurally different repositories such as:

- Rust CLI/runtime;
- Python agent tool;
- Kotlin Multiplatform project;
- MCP server;
- mixed agent runtime.

Fixed inputs must produce byte-for-byte expected generated artifacts.

### 14.3 Adversarial tests

Required cases:

- local policy attempts mandatory weakening;
- malicious review comment embeds shell commands;
- README contains prompt injection;
- generated path attempts `../` escape;
- symlink escapes workspace;
- target HEAD changes during compilation;
- credential-like value appears in output;
- exception scope is unnecessarily broad;
- unknown policy version appears.

All security-boundary failures are fail-closed.

### 14.4 Fleet simulation

`crctl audit-fleet --dry-run` evaluates all configured repositories without mutation and returns per-repository compile/drift/conflict/suggested-profile status plus an overall partial-failure state where necessary.

### 14.5 Property invariants

At minimum:

- adding stricter policy cannot weaken the effective set;
- removing local policy cannot remove mandatory policy;
- expired exceptions cannot alter effective policy;
- policy ordering cannot change output;
- identical inputs produce identical resolution/output digests.

## 15. MVP phases

### Phase 0: specification and threat model

Finalize schemas, authority rules, trust boundaries, and acceptance criteria.

### Phase 1: schema/model/resolver

Implement typed entities, schema validation, exception validation, and deterministic resolution with weakening detection.

### Phase 2: compilers and validator

Implement CodeRabbit YAML, Markdown, PR-template, and basic ast-grep compilation plus golden/adversarial validation.

### Phase 3: CLI and fixtures

Expose deterministic CLI commands and repository fixtures.

### Phase 4: repository discovery

Implement dry-run evidence-backed fingerprinting and conservative profile classification.

### Phase 5: GitHub synchronization

Add revision-checked branch/PR synchronization and evidence manifests.

### Phase 6: ChatGPT/Codex skill

Add orchestration workflows that call the deterministic engine and GitHub adapter.

### Phase 7: three-repository pilot

Validate one catalog against AutoDev, a Python/agent-tool repository, and a simpler structurally distinct repository.

### Phase 8: hardening

Close a real CodeRabbit finding -> normalized record -> source-policy correction -> recompile -> sync loop and verify deterministic clean-checkout reproduction.

The MVP ends after phases 7-8. MCP and automated cross-repo learning are later phases.

## 16. Acceptance criteria

The MVP is accepted only when all are demonstrated:

1. Identical inputs produce identical artifacts and digests.
2. A repository cannot weaken mandatory core policy without a valid approved exception.
3. Generation cannot write outside the output allowlist.
4. A clean checkout reproduces checked-in generated artifacts.
5. At least three structurally different repositories compile successfully from one central catalog.
6. Manual edits to generated policy are detected as drift.
7. GitHub synchronization uses dedicated branches and pull requests.
8. Every successful compilation emits a complete `EvidenceManifest`.
9. Generated `.coderabbit.yaml` is accepted by CodeRabbit on the pilot repositories.
10. At least one real CodeRabbit finding is normalized into a `FindingRecord`, corrected in source policy, recompiled, and synchronized.
11. External review/comment text is never executed as instruction by the core engine.
12. A stale target revision aborts synchronization rather than applying stale output.

## 17. Research decisions and reuse

- Reuse the existing GitHub connector/client boundary rather than building a generic GitHub MCP server. The MCP Registry already contains active GitHub API and PR-analysis servers, so the future MCP adapter should remain policy-specific.
- Preserve AutoDev's existing ConnectorForge one-owner principle: repository/source/branch/PR/CI truth belongs to GitHub; repository architecture docs remain authoritative; learned heuristics may be persisted to Engram but never override Git policy state.
- Use CodeRabbit's repository YAML and path instructions as generated targets, while recognizing that organization Global Overrides and external integrations are distinct authority classes.
- Treat automatic repository linking as useful discovery evidence for cross-repo relationships but not as the control plane's authoritative repository policy map.

## 18. Open implementation constraint

The approved architecture calls for a dedicated `coderabbit-control-plane` repository. The currently available GitHub connector can create branches/files/commits/PRs in existing repositories but does not expose repository creation. Therefore this specification is stored in AutoDev as the bootstrap design record. Implementation should begin in a dedicated repository once that repository exists; until then, AutoDev should not absorb production control-plane code merely to work around connector limitations.
