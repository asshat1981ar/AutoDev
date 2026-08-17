# Evidence Lease Gate — Approved Design

**Status:** Approved for implementation  
**Scope:** ConnectorForge W1 hardening / current-evidence eligibility  
**Trusted authority:** ForgeCore

## Purpose

W1 architecture evidence is historical, immutable evidence. The Evidence Lease Gate adds a separate, current-time eligibility layer so stale or materially changed evidence cannot satisfy a future verification gate without rewriting historical evidence or architecture decisions.

The model is:

- `EvidenceRecord` = what was observed.
- `LeasePolicy` = what current eligibility requires.
- `RefreshProposal` = what newly fetched source material proposes.
- `LeaseEvaluation` = what ForgeCore determines now.
- `LeaseAttestation` = immutable proof that evidence satisfied an exact policy state at an exact evaluation time.
- `ArchitectureDecision` = what was historically decided.

Historical evidence and decisions remain immutable. Do **not** add mutable fields such as `valid`, `stale`, `lease_state`, or `current_status` to `EvidenceRecord`.

## Trust boundary

The trust flow remains:

```text
model/connector intent
        |
        v
normalized proposal
        |
        v
ForgeCore policy + evaluator
        |
        v
lease evaluation / attestation
        |
        v
current verification gate
```

A lease attestation is **not** an `AuthorizationGrant`. It cannot authorize code execution, GitHub mutations, or any capability. External tools/connectors may fetch source or approval information, but they only provide observations/proposals. ForgeCore decides whether normalized information satisfies policy.

No connector/network calls, tool invocation, plugin execution, credentials, or arbitrary code are permitted inside policy evaluation.

## Architecture

```text
Repository policy configuration
        |
        v
LeasePolicyRegistry
        |
        +-- ForgeCore built-in safety floors
        |
        +-- repository extensions / controlled relaxations
                        |
                        v
                  PolicyCompiler
                        |
                        v
                  EffectivePolicy
                        |
Adapter ------> RefreshProposal
                        |
                        v
              EvidenceLeaseEvaluator
                /        |        \
             reject    review    acceptable
                                  |
                                  v
                           LeaseAttestation
                                  |
                                  v
                        Current Verification Gate
```

## Policy model

Policies are named and versioned. ForgeCore owns built-in safety floors. Repository policy may tighten freely. Relaxation is allowed only when all of the following are present and valid:

1. `allow_relaxation` is explicitly enabled.
2. A non-empty rationale is supplied.
3. A repository-backed approval reference is supplied.
4. Matching normalized repository approval evidence is supplied.

Repository-backed approval kinds are limited to:

- commit
- pull request
- ADR

Unknown or malformed policies fail closed.

### Closed policy algebra

The evaluator supports a small closed algebra:

- `MaxAge`
- `SourceVersionRequired`
- `FingerprintStable`
- `RiskAtMost`
- `ExplicitRevalidation`
- `ExplicitInvalidationAbsent`
- `AllOf`
- `AnyOf`

This is not a scripting language. Unsupported policy comparisons fail closed rather than attempting general theorem proving.

## Risk and revalidation

Risk tiers:

- `Low`
- `Medium`
- `High`

Revalidation rules:

- Low-risk evidence may automatically renew only when the authoritative source remains unchanged and policy explicitly permits the automatic path.
- Medium/high-risk material changes require explicit revalidation.
- Any policy relaxation requires repository-backed approval.

A fingerprint change under the same nominal source version is a material change.

## Time semantics

The evaluator receives `evaluated_at` explicitly and must never read the ambient system clock.

The validity boundary is exact:

```text
evaluated_at < valid_until   => current

evaluated_at >= valid_until  => stale
```

Source-version or fingerprint changes invalidate prior current eligibility. Explicit invalidation overrides freshness.

## LeaseAttestation

`LeaseAttestation` is immutable and binds the exact evidence and policy state:

- `evidence_id`
- `objective_id`
- `evidence_fingerprint`
- `policy_id`
- `policy_version`
- `policy_fingerprint`
- `source_version`
- `evaluated_at`
- `valid_until`
- `risk_tier`
- `attestation_fingerprint`

The attestation fingerprint must be deterministic and bind all attested state.

An attestation only proves current eligibility under a specific policy and evaluation time. It does not convert `Inferred` or `Hypothesis` evidence into intrinsically verified evidence.

## Current verification

Current verified contribution is:

```text
intrinsically supported EvidenceClass
AND
current matching LeaseAttestation
```

The current gate must validate:

1. `EvidenceRecord` structural validity.
2. Evidence class can intrinsically satisfy a verified gate.
3. `LeaseAttestation` self-validity.
4. Evidence/objective/fingerprint match.
5. Policy fingerprint matches the current `EffectivePolicy`.
6. `evaluated_at < valid_until`.

Historical `DecisionMaturity::Verified` is not rewritten when supporting evidence later expires.

## Module boundary

Keep W1 historical architecture evidence in `architecture_evidence.rs` and add lease behavior in a focused `architecture_lease.rs` module. The lease module may depend on normalized W1 evidence types but must remain separate from execution evidence and authorization.

## Safety invariants

- ForgeCore remains the trusted authorization/execution authority.
- Lease data never grants capabilities.
- Historical records are immutable.
- Current eligibility is computed, not stored as mutable state on evidence.
- Evaluation is deterministic for identical inputs.
- No ambient clock reads.
- No network/tool/plugin calls in the evaluator.
- Unknown/malformed policy or proposal input fails closed.
- Policy relaxation requires repository-backed approval.
