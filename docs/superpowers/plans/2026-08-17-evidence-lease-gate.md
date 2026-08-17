# Evidence Lease Gate — Implementation Plan

**Status:** Approved  
**Branch:** `docs/connectorforge-workshops`  
**PR:** #9  
**Method:** Superpowers + strict RED/GREEN TDD

## Guardrails

- Keep PR #9 draft.
- Do not merge or enable auto-merge.
- Preserve ForgeCore as the trusted authorization/execution authority.
- Keep lease/current-eligibility behavior separate from execution evidence and authorization.
- Do not add mutable lease/current-state fields to `EvidenceRecord`.
- No network/tool/plugin/credential access inside policy evaluation.
- Use explicit `evaluated_at`; never read ambient system time in evaluation.
- Commit coherent, independently reviewable slices.

## Task 0 — Commit approved design and plan

Add:

- `docs/superpowers/specs/2026-08-17-evidence-lease-gate-design.md`
- `docs/superpowers/plans/2026-08-17-evidence-lease-gate.md`

Suggested commit: `docs: specify evidence lease gate`.

## Task 1 — Harden EvidenceRecord structural validation

### RED

Add focused integrity tests proving public/deserialized records cannot bypass constructor invariants. Cover at least:

- empty required fields;
- confidence greater than 100;
- malformed SHA-256 fingerprint shape;
- report rendering rejects a malformed public/deserialized record.

Verify failure is caused by missing product behavior, not test syntax or harness failure.

### GREEN

Implement the minimum behavior:

- public `EvidenceRecord::validate()`;
- validation of required fields and confidence;
- SHA-256 fingerprint structure validation (64 lowercase/uppercase hexadecimal characters accepted only as structurally valid hex; canonical generated fingerprints remain lowercase);
- constructor builds the record then reuses `validate()`;
- report renderer calls `record.validate()` before using public/deserialized records.

Run focused Rust tests, `cargo fmt --all -- --check`, and clippy for the affected workspace before committing.

## Task 2 — Policy core

Create focused `architecture_lease.rs` with:

- `RiskTier`
- `LeaseRule`
- `RevalidationMode`
- `LeasePolicyDefinition`
- `LeasePolicyRegistry`
- `EffectivePolicy`
- deterministic canonical policy fingerprinting

Rules are a closed algebra:

- `MaxAge`
- `SourceVersionRequired`
- `FingerprintStable`
- `RiskAtMost`
- `ExplicitRevalidation`
- `ExplicitInvalidationAbsent`
- `AllOf`
- `AnyOf`

Use test-first deterministic fingerprint fixtures and registry/compiler validation cases. Unknown/malformed policy input fails closed.

## Task 3 — Repository policy override and controlled relaxation

Add:

- `ApprovalReferenceKind`
- `ApprovalReference`
- `RepositoryApprovalEvidence`
- `PolicyRelaxation`
- `RepositoryPolicyOverride`

Approval reference kinds are limited to commit, pull request, and ADR.

Test and implement:

- tightening accepted without relaxation approval;
- relaxation rejected when `allow_relaxation` is false;
- relaxation rejected without rationale;
- relaxation rejected without repository-backed approval reference;
- relaxation rejected when normalized approval evidence does not match;
- approved relaxation accepted when all requirements match;
- unsupported comparisons fail closed rather than invoking general theorem proving.

## Task 4 — RefreshProposal and deterministic lease evaluation

Add:

- `RefreshProposal`
- `LeaseEvaluationStatus`
- `LeaseEvaluationReason`
- `LeaseEvaluation`
- deterministic `evaluate_lease(...)`

Tests must cover:

- before expiry;
- exact expiry;
- after expiry;
- source-version change;
- fingerprint change;
- explicit invalidation;
- low-risk automatic path when authoritative source is unchanged and policy permits;
- medium/high-risk material changes requiring review;
- policy fingerprint changes;
- malformed refresh proposals.

`evaluate_lease` receives explicit `evaluated_at`; it must not read the ambient system clock and must perform no connector/network/tool calls.

## Task 5 — Immutable LeaseAttestation

Add immutable attestation issuance and self-validation.

The attestation must bind:

- evidence ID and objective ID;
- evidence fingerprint;
- policy ID/version/fingerprint;
- source version;
- evaluated time and validity boundary;
- risk tier;
- deterministic attestation fingerprint.

Tests:

- only a valid/acceptable lease evaluation can issue an attestation;
- identical inputs produce identical fingerprints;
- changing any bound field changes or invalidates the fingerprint;
- malformed/tampered attestations fail self-validation.

## Task 6 — Lease-aware current verification

Add an explicit boundary similar to:

```rust
evaluate_current_verification(...)
```

The current gate verifies:

- `EvidenceRecord::validate()` passes;
- evidence class intrinsically satisfies the verified gate;
- attestation self-validation passes;
- attestation evidence/objective/fingerprint matches the evidence record;
- attestation policy fingerprint matches the current effective policy;
- `evaluated_at < valid_until`.

Tests must prove:

- stale evidence remains historical but cannot satisfy the current gate;
- `Inferred`/`Hypothesis` cannot become intrinsically verified through attestation;
- historical `ArchitectureDecision` maturity is unchanged when lease eligibility expires.

## Task 7 — Documentation and full verification

Update ConnectorForge architecture documentation with the implemented lease/current-verification boundary and deferred work.

Before completion, read the current `.github/workflows/ci.yml` and run its current CI-equivalent gates rather than relying on remembered commands. At minimum expect:

- `cargo fmt --all -- --check`
- clippy with warnings denied
- Rust workspace build
- Rust workspace tests
- Kotlin build/tests
- Kotlin ktlint
- Python checks currently defined by CI

Push the final coherent slice and obtain a fresh GitHub Actions run.

Final report must include:

- exact final head SHA;
- fresh CI run ID;
- per-job results;
- tests added;
- known deferred work;
- whether independent review occurred.

Do not claim CodeRabbit review unless CodeRabbit actually ran. Never merge PR #9 without explicit user authorization.
