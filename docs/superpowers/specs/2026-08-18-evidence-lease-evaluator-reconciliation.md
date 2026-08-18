# Evidence Lease Evaluator / Attestation Dependency Reconciliation

Date: 2026-08-18
Status: Binding implementation ruling for A4-A5

## Conflict

The previously approved A4 evaluator signature consumes:

```rust
prior_attestation: Option<&LeaseAttestation>
```

but the same approved implementation plan schedules full `LeaseAttestation` implementation in A5. A4 therefore cannot preserve its approved public signature unless the attestation data contract exists before A5.

## Binding ruling

A4 introduces **only the immutable `LeaseAttestation` data shape** required to read prior lease state:

```rust
pub struct LeaseAttestation {
    pub evidence_id: String,
    pub objective_id: String,
    pub evidence_fingerprint: String,
    pub policy_id: String,
    pub policy_version: String,
    pub policy_fingerprint: String,
    pub source_version: String,
    pub evaluated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub risk_tier: RiskTier,
    pub attestation_fingerprint: String,
}
```

A4 may inspect these fields to determine current eligibility, source/fingerprint changes, policy changes, and expiry boundaries.

A4 must **not** add:

- `attest(...)`;
- attestation fingerprint generation;
- `LeaseAttestation::validate()`;
- caller-controlled attestation issuance;
- any conversion to `AuthorizationGrant`.

Those remain exclusively A5 responsibilities.

## A5 ownership

A5 will:

1. add deterministic `attest(...)` issuance from a `LeaseEvaluationStatus::Valid` result only;
2. derive `valid_until` from the effective policy and injected `evaluated_at` rather than accepting caller-provided expiry;
3. bind evidence/objective/evidence fingerprint/policy id+version+fingerprint/source version/evaluation time/valid-until/risk in a deterministic attestation fingerprint;
4. add `LeaseAttestation::validate()` for public/deserialized attestation integrity;
5. reject malformed, stale, mismatched, or non-Valid issuance inputs.

## Safety effect

The A4 data shape alone is **not trusted proof**. Until A5 adds self-validation and controlled issuance, any externally constructed attestation is merely normalized input to the evaluator and cannot satisfy the later A6 current-verification gate.

This preserves the architectural separation:

```text
A4: read prior lease state and evaluate now
A5: issue and validate immutable proof
A6: require validated matching proof for current verification
```

No execution adapter, capability, policy authorization, or Git mutation semantics change under this ruling.