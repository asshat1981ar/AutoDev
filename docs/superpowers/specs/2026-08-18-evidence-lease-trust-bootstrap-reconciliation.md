# Evidence-Lease Trust and Bootstrap Reconciliation

**Date:** 2026-08-18  
**Status:** Security decision gate — design selected, production implementation requires approval  
**Applies to:** ForgeCore `architecture_lease` A4/A5 boundary before A6 GREEN

## Problem statement

The current lease subsystem has two distinct integrity properties that were accidentally conflated:

1. `content_fingerprint` / `attestation_fingerprint` provide deterministic content identity and tamper detection.
2. ForgeOS also needs proof that a trust-bearing attestation was actually issued by the trusted kernel under the intended evaluation context.

A plain SHA-256 fingerprint over public fields cannot establish issuer provenance. Any caller that knows the canonicalization rule can reproduce the checksum for caller-chosen fields.

Therefore this invariant is insufficient:

```text
attestation.validate() == Ok
```

The required invariant is:

```text
canonical structure is valid
AND deterministic content hash matches
AND trusted issuer proof verifies
AND evidence/policy/source bindings match current context
AND lease has not expired or been invalidated
```

## Observed defects

### 1. Prior attestation is consumed without self-validation

`evaluate_lease(...)` accepts `Option<&LeaseAttestation>` and reads its identifiers, fingerprints, risk, and expiry without first invoking `LeaseAttestation::validate()`.

This means a malformed prior with a bogus `attestation_fingerprint` can currently influence current eligibility.

### 2. A valid checksum would still be forgeable

Adding only `prior.validate()?` is necessary but not sufficient. The current attestation fingerprint is an unkeyed deterministic SHA-256 over public fields. A caller can construct a different attestation and reproduce its correct checksum.

Thus the existing fingerprint proves **integrity**, not **provenance**.

### 3. `LeaseEvaluation` is context-replayable

`LeaseEvaluation` currently binds only:

- status;
- reason;
- evaluated time;
- risk tier.

`attest(...)` accepts a caller-supplied `&LeaseEvaluation` and separately supplied evidence, policy, and proposal. A `Valid` evaluation does not itself prove it was produced for those exact inputs.

### 4. First-attestation bootstrap is underspecified

Existing tests fabricate prior attestations to make automatic low-risk renewal reachable. Once prior attestations are authenticated, those fixtures no longer model a legitimate first issuance path.

ForgeOS therefore needs an explicit bootstrap/revalidation authority rule rather than silently treating a caller-created prior as historical kernel evidence.

## Security invariants

A5-S must enforce all of the following before A6 can become GREEN:

1. **Untrusted data cannot become historical lease authority by serialization alone.**
2. **Every consumed prior attestation is structurally validated and issuer-authenticated.**
3. **A caller cannot replay a `Valid` evaluation against different evidence, policy, source version, proposal, or risk context.**
4. **First issuance is explicit and auditable.**
5. **Automatic renewal is permitted only from an authenticated prior and only under the policy's automatic-renewal rules.**
6. **Material changes fail closed into explicit revalidation.**
7. **The model/agent cannot manufacture bootstrap or revalidation approval fields.**
8. **Lease evidence remains separate from execution authorization.**

## Considered approaches

### Option A — Call `LeaseAttestation::validate()` at consumption only

**Benefit:** smallest patch.  
**Failure:** an unkeyed deterministic hash is forgeable by any caller.  
**Decision:** reject as incomplete.

### Option B — Make fields private and remove `Deserialize`

**Benefit:** blocks ordinary external Rust construction and model JSON deserialization.  
**Failure:** does not provide a durable provenance story for persistence, FFI, process boundaries, restore, or future distributed verification. Trust would depend on every loader remaining perfect.  
**Decision:** useful defense-in-depth, but insufficient as the primary trust mechanism.

### Option C — Shared-secret MAC

**Benefit:** simple and efficient for one trusted process.  
**Failure:** every verifier holding the shared key can also forge attestations, weakening the future independent-verification boundary.  
**Decision:** acceptable fallback for a strictly single-process prototype, but not the preferred durable architecture.

### Option D — Asymmetric issuer signature + self-contained issuance

**Benefit:** only the trusted issuer needs the private signing key; independent components can verify with a public key without acquiring forge authority. Supports persistence and later process separation.  
**Cost:** requires explicit key lifecycle and a signature dependency.  
**Decision:** **selected**.

## Selected design

### 1. Separate content identity from issuer authenticity

Retain deterministic SHA-256 fingerprints for content identity.

Add a distinct issuer-proof envelope:

```text
LeaseIssuerProof
  schema_version
  issuer_id
  key_id
  algorithm
  signature
```

The signed canonical payload binds at minimum:

```text
attestation schema version
+ evidence id
+ objective id
+ evidence fingerprint
+ policy id
+ policy version
+ policy fingerprint
+ source version
+ evaluated_at
+ valid_until
+ risk tier
```

The signature must not replace the deterministic content fingerprint; the two have different purposes.

### 2. Do not trust caller-supplied evaluation during issuance

Replace the semantic shape:

```text
caller evaluates -> caller passes Valid evaluation -> attest(...)
```

with a kernel-owned transaction:

```text
trusted issuance request
  -> validate all inputs
  -> authenticate prior/bootstrap proof
  -> evaluate deterministically inside ForgeCore
  -> if and only if result is Valid, derive expiry
  -> canonicalize
  -> sign
  -> return LeaseAttestation
```

A public diagnostic evaluator may remain, but its result is not an authority token and cannot by itself cause issuance.

This removes `LeaseEvaluation` replay as an issuance mechanism rather than adding another fragile cross-check.

### 3. Explicit bootstrap path

First issuance must not fabricate a historical prior.

For this foundation slice, bootstrap is conservative:

```text
BootstrapLeaseApproval
  exact evidence binding
  exact objective binding
  exact policy binding
  exact source version
  risk tier
  approval reference
  approved_at
```

Bootstrap approval is supplied through trusted orchestration/repository approval handling, never an agent payload boolean.

The first signed attestation can be produced only when:

- the evidence and policy validate;
- the bootstrap approval binds exactly to them;
- the policy permits the requested risk tier;
- there is no explicit invalidation;
- all other policy constraints hold.

This is intentionally stricter than future adapter-backed automatic bootstrap. Automation can later replace human/repository approval only after a separate authenticated observation/adapter-attestation design exists.

### 4. Renewal rules

#### Authenticated unchanged low-risk prior

If policy mode is `AutomaticLowRisk`, an authenticated prior may renew automatically only when the normalized source version and content fingerprint remain unchanged and the proposal passes structural/objective/history checks.

#### Material change

Fingerprint change or source-version change requires explicit revalidation.

#### Medium/high risk

Medium/high risk remains explicit regardless of freshness policy unless a later approved policy design says otherwise.

### 5. Consumption rule

A6 current-verification code must never accept raw attestation fields as sufficient evidence.

The gate must perform:

```text
validate structure
-> verify issuer signature against trusted key registry
-> match exact evidence
-> match exact objective
-> match current policy id/version/fingerprint
-> check risk/policy constraints
-> check expiry at injected time
-> check explicit invalidation/current state
-> only then mark evidence currently eligible
```

## Cryptographic primitive selection

Preferred initial implementation: Ed25519-family asymmetric signatures, with the concrete Rust crate/version selected only after current primary documentation and dependency compatibility are verified.

ForgeCore must not fetch keys from a network service. Signer/verifier dependencies are injected explicitly.

Test fixtures use deterministic test keys. Production key material is supplied by trusted runtime configuration and must never be model-visible or serialized into evidence records.

The public verification key/key-id may be persisted with trusted runtime metadata.

## API direction

Exact names remain implementation-detail candidates, but the semantic split should resemble:

```text
LeaseSigner
LeaseVerifier
LeaseIssuerProof
BootstrapLeaseApproval
LeaseIssuanceRequest
issue_lease_attestation(...)
verify_lease_attestation(...)
evaluate_lease(...)              // diagnostic/state evaluation only
```

`LeaseEvaluation` remains useful as a deterministic explanation of state but ceases to be an issuance credential.

## Defense in depth

Even with signatures:

- make trust-bearing attestation fields immutable outside constructors where compatible with serialization requirements;
- avoid generic `Deserialize` directly into an authenticated/trusted wrapper;
- deserialize into an untrusted record shape, then explicitly verify into a trusted wrapper where practical;
- use domain-separated canonical signing bytes, including schema version and object type;
- reject unknown signature algorithms and key ids;
- reject duplicate/ambiguous fields at serialization boundaries;
- compare current policy fingerprint, not only policy id;
- preserve historical attestations instead of mutating them during renewal.

## TDD sequence

### A5-S1 — Consumption authenticity RED

Tests must prove:

- malformed checksum prior is rejected;
- correctly recomputed SHA fingerprint without valid issuer signature is rejected;
- wrong key is rejected;
- correct signature over different evidence is rejected;
- correct signature over different policy is rejected;
- expired signed prior is not current.

### A5-S2 — Issuance replay RED

Tests must prove:

- a diagnostic `Valid` evaluation cannot be replayed to issue an attestation for another proposal;
- issuance computes its own evaluation internally;
- caller cannot choose `valid_until`.

### A5-S3 — Bootstrap RED

Tests must prove:

- no-prior/no-bootstrap cannot issue;
- mismatched bootstrap approval cannot issue;
- exact trusted bootstrap can issue the first attestation;
- model/payload approval booleans have no effect.

### A5-S4 — Renewal GREEN

Tests must prove:

- signed unchanged low-risk prior renews under `AutomaticLowRisk`;
- material change requires explicit revalidation;
- medium/high risk remains explicit;
- historical prior remains unchanged.

### A5-S5 — A6 fixture correction

Replace fabricated `LeaseAttestation { ... "0".repeat(64) ... }` fixtures with attestations issued through the trusted test signer/bootstrap path.

Only then implement `evaluate_current_verification`.

## Migration and compatibility

This is a foundation branch; prefer correcting the contract now over preserving an insecure public constructor shape.

Compatibility priority:

1. security invariant;
2. deterministic behavior;
3. historical evidence preservation;
4. minimal public API churn.

No execution adapter integration depends on this API yet, so this is the least expensive point to fix provenance semantics.

## Out of scope

A5-S does **not** add:

- execution authorization;
- model routing;
- connector/network SDKs;
- generated UI;
- distributed key management service;
- remote signing;
- capability/resource leases;
- post-A9 action transaction receipts.

## Approval gate

The following are major contract changes and require approval before production implementation:

1. adding issuer-authenticated lease attestations;
2. adding a trusted bootstrap approval path;
3. changing attestation issuance so ForgeCore evaluates internally rather than trusting caller-supplied `LeaseEvaluation`;
4. changing A6 fixtures to require legitimately issued authenticated attestations.

Until approved and verified, **A6 may remain RED but must not receive a GREEN implementation that trusts raw/fabricated prior attestations.**
