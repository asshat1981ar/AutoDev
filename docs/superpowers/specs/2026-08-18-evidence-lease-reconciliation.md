# ForgeOS Evidence Lease Contract Reconciliation

Date: 2026-08-18
Status: Binding implementation ruling for A2-A6

## Reason

The approved ForgeOS authority-foundation plan simplified several Evidence Lease types relative to the previously approved Evidence Lease Gate design and implementation plan. The earlier Evidence Lease contract is more specific and remains authoritative for lease semantics. This ruling reconciles the documents before A2 production code is written.

## Binding A2 contract

Use these types and meanings:

```rust
pub enum RiskTier {
    Low,
    Medium,
    High,
}

pub enum RevalidationMode {
    AutomaticLowRisk,
    ExplicitOnMaterialChange,
    Explicit,
}

pub enum LeaseRule {
    MaxAgeSeconds(u64),
    SourceVersionRequired,
    FingerprintStable,
    RiskAtMost(RiskTier),
    ExplicitRevalidation,
    ExplicitInvalidationAbsent,
    AllOf(Vec<LeaseRule>),
    AnyOf(Vec<LeaseRule>),
}

pub struct LeasePolicyDefinition {
    pub id: String,
    pub version: String,
    pub rule: LeaseRule,
    pub revalidation_mode: RevalidationMode,
}
```

A2 `EffectivePolicy` contains the resolved definition plus a deterministic `policy_fingerprint`. Relaxation metadata is added in A3, where the approval types exist.

## Registry boundary

A2 exposes deterministic built-in-only resolution:

```rust
LeasePolicyRegistry::built_ins()
LeasePolicyRegistry::get(id)
LeasePolicyRegistry::resolve(id)
```

A3 introduces the final repository-aware `compile(...)` API because it consumes `RepositoryPolicyOverride`, `RepositoryApprovalEvidence`, and `PolicyRelaxation`, which do not exist until A3. This fixes the older implementation plan's type-order defect without changing final semantics.

Unknown policy IDs fail closed.

## Built-in policy ruling

`repo_state` is the required built-in policy identifier for A2 tests.

Do not invent production TTL values. Policy duration values are versioned product-policy data. A built-in may therefore use only constraints that do not require an unapproved duration. Test-only policy definitions may use explicit local `MaxAgeSeconds` values to exercise validation and fingerprinting.

The initial `repo_state` floor is conservative and source-stability based:

```text
AllOf(
  SourceVersionRequired,
  FingerprintStable,
  RiskAtMost(Low),
  ExplicitInvalidationAbsent
)
RevalidationMode = AutomaticLowRisk
```

This does not make evidence current by itself. Later A4 evaluation still determines whether unchanged low-risk evidence is eligible for automatic revalidation.

## Structural validation

- policy `id` and `version` are non-empty;
- `MaxAgeSeconds(0)` is invalid;
- empty `AllOf` and `AnyOf` are invalid;
- nested rules validate recursively;
- enums remain closed and non-Turing-complete;
- policy evaluation performs no I/O or ambient-time lookup.

## Canonical fingerprinting

Do not hash `Debug` output. Canonicalize explicitly using stable tokens:

```text
max_age(<seconds>)
source_version_required
fingerprint_stable
risk_at_most(low|medium|high)
explicit_revalidation
explicit_invalidation_absent
all(<child>;...)
any(<child>;...)
```

The policy fingerprint binds policy ID, version, revalidation mode, and canonical rule. Identical semantic input must produce an identical SHA-256 fingerprint.

## Scope

This ruling changes no execution adapter and grants no authority. `LeasePolicy`, `EffectivePolicy`, and later `LeaseAttestation` remain evidence-eligibility concepts only and must never become `AuthorizationGrant`.