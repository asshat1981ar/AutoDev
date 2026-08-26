# Harness Profile Fabric v0

## Purpose

The Harness Profile Fabric is AutoDev's declarative orchestration layer for reusable development workflows. A harness profile describes intent, ordered or parallel stages, versioned asset references, verification contracts, success metrics, memory policy, and evidence-gated improvement policy. It does not execute privileged effects.

The implementation lives in `crates/forge-core/src/harness.rs`, with built-in profiles in `crates/forge-core/src/harness/builtins.rs`, deterministic routing in `crates/forge-core/src/harness/routing.rs`, and promotion evaluation in `crates/forge-core/src/harness/evaluation.rs`.

## Non-negotiable authority boundary

Harness configuration cannot mint authorization.

Harness configuration cannot self-verify.

Harness profiles, routing results, and promotion decisions are advisory configuration/evidence objects. They cannot create an `AuthorizationGrant`, widen capabilities, bypass policy, execute an effect, mutate the profile registry as a side effect of evaluation, or declare their own work verified. ForgeCore remains the trusted execution and authorization boundary.

## Stable built-in profiles

The v0 catalog has exactly five stable public identities. Renaming or removing one is a compatibility change and must be deliberate rather than accidental drift.

### `forgeflow-sdlc`

General software-delivery lifecycle harness. It composes discovery, architecture/planning, test-first implementation, independent review, and verification stages for feature and repository-development work.

### `sprintmesh-agile`

Agile delivery harness. It structures backlog refinement, bounded parallel work, integration, review, and sprint evidence without granting agents authority to merge or approve their own output.

### `idea-tournament`

Innovation and product-discovery harness. It generates competing hypotheses/prototypes, evaluates them against explicit evidence, and advances the strongest candidate rather than accepting a single untested idea.

### `optiforge-optimizer`

Measured optimization harness. It establishes a baseline, identifies a bottleneck, runs bounded experiments, and requires non-regression plus measurable efficiency improvement before an optimization can be recommended.

### `harnessforge-meta`

Meta-harness for proposing improvements to harnesses and reusable development capabilities. Its outputs remain candidates until independently evaluated; the meta-harness cannot promote itself or alter ForgeCore authority.

## Deterministic routing

`route_harness` is intentionally model-free. It searches the normalized `DevelopmentContract` text—goal, acceptance criteria, and constraints—against each profile's declared trigger terms. Each selected profile carries matched-term evidence and a deterministic score. Equal scores are ordered by stable profile ID.

This baseline is deliberately simple and reproducible. Future learned or semantic routing can be evaluated against it, but may not silently replace its auditability or authority boundary.

## Promotion guardrails

`evaluate_harness_candidate` consumes externally measured `HarnessEvaluation` evidence and returns an advisory `HarnessPromotionDecision`. Percentage-like rates use integer basis points, where `10_000` means 100%, avoiding floating-point threshold ambiguity.

A candidate is rejected when any of the following is true:

- the profile is structurally invalid;
- a basis-point metric is outside `0..=10_000`;
- the evaluation has zero samples;
- independent verification references are missing or blank;
- correctness falls below baseline;
- evidence completion falls below baseline;
- unsafe-action rejection falls below baseline;
- neither measured duration nor measured resource use improves.

Eligibility therefore requires non-regression on correctness and safety/evidence gates, independent verification, a nonzero sample, and at least one strict efficiency improvement. `Eligible` is not authorization to deploy, merge, execute, or rewrite policy.

## Verification contract

Every stage in every built-in profile must carry a non-empty verification contract. Built-in profiles must validate structurally and may not repeat stage IDs or duplicate an asset reference within a stage. These invariants are exercised by Rust integration tests.

Repository drift is additionally checked by `scripts/check_harness_drift.py`. The checker protects the five stable IDs in both this document and the built-in source, and protects the two explicit authority statements above. `tests/test_harness_drift.py` tests the checker itself.

## Change policy

Changes to v0 should preserve these invariants unless a versioned successor explicitly supersedes them:

1. declarative profiles remain authority-free;
2. verification remains independent of the harness producing the work;
3. routing remains evidence-bearing and deterministic for the baseline implementation;
4. promotion remains advisory and evidence-gated;
5. stable IDs remain protected from accidental documentation/source drift;
6. ForgeCore remains the sole trusted execution boundary.
