# ADR-007: ForgeCore-Signed Gate Attestations for Client-Side Policy Gates

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** AutoDev maintainers
- **Related:** `ADR-001-forgecore-execution.md`, `ADR-002-evidence-persistence.md`, Linear PRO-78

## Context

GitHubOpsGem (the `github-ops-orchestrator` skill and its `gemcore` library) gates GitHub writes made by Claude Code sessions: protected-branch pushes, merges without tests, leaked secrets, unverified FFI symbols, prompt-injection taint. Those checks run in untrusted places — a Termux shell, a CI runner, the model's own context. Until now each gate emitted a `<PreFlightGate>` block that the model wrote or that a local HMAC key signed. Either way the verdict was self-attested: nothing outside the client could tell a real gate from a forged one.

AutoDev already has the right home for this: ForgeCore is the sole trusted boundary, and `AuthorizationGrant` is created only by the kernel after a recorded approval. A second, independent signing authority inside GitHubOpsGem would compete with that boundary.

## Decision

Add `forge_core::gate_attestation`, a pure module (no I/O, caller-supplied clock and key), and two control-plane routes on `autodev-server`:

- `POST /api/v1/gate/attest` — accepts an untrusted `GateAttestationRequest` and returns a signed `GateAttestation` (`201`).
- `POST /api/v1/gate/verify` — returns `{valid, recorded, attestation_id}` for a presented attestation.

The kernel, not the client, owns every security-relevant field:

1. **Decision.** The request schema rejects unknown fields, so a client cannot send `decision`. The kernel derives `PASS`, `WARN` or `BLOCK` from client violations and the requested mode.
2. **Kernel violations.** The kernel normalizes the ref (`refs/heads/main`, `origin/main` → `main`) and adds its own violation for direct writes to protected refs (`main`, `master`, `production`, `prod`, `trunk`, `release*`, `hotfix*`). Kernel violations always `BLOCK`, even when the client asks for `warn` mode.
3. **Signature.** HMAC-SHA256 over a canonical body with a versioned schema (`gem-gate/1`). The key comes from `AUTODEV_GATE_SIGNING_KEY` (minimum 32 bytes, enforced at startup) and lives only in the server process. `key_id` is a public, non-reversible identifier.
4. **Bounds.** At most 64 violations of 512 characters, 64 KiB of opaque `facts` (hashed, not stored), and a strictly hex `payload_sha256`. Violations of these bounds fail with `400`.

**An attestation is evidence, not authority.** It never constructs an `AuthorizationGrant` and cannot discharge `RequireApproval`. Human approval stays the only way to authorize a gated effect.

Both routes require `AUTODEV_API_BEARER_TOKEN` even on loopback, and both return `503` when either the bearer or the signing key is unconfigured. Objective intake keeps its existing optional-on-loopback behavior.

## Consequences

- A `PASS` from GitHubOpsGem is now checkable: hooks and CI call `/verify` instead of trusting a model-written block.
- A compromised client can still under-report its own heuristic violations. The kernel-owned protected-ref check, GitHub branch rulesets and the CI `gem-review` check are the backstops; the attestation proves what was claimed and when, not that the claim was complete.
- Verification needs the key, so it happens on the server. Asymmetric signatures (so CI can verify offline) are deferred until a second verifier actually exists; adopting them means a new crate and a `gem-gate/2` schema.
- Attestations are cached in memory (bounded at 10,000) to answer `recorded`. Durable storage and a `policy_gate` evidence name in `verification.rs` are follow-ups under ADR-002. No new evidence names are introduced here, so `ExecutionEnvelope.evidence.required` semantics are unchanged.
- `forge-core` gains the `hmac` crate, already present in `crates/Cargo.lock` through `autodev-server`.

## Verification

- `cargo test -p forge-core --lib gate_attestation` — decision derivation, kernel protected-ref block in warn mode, client-supplied decision rejected, tamper/foreign-key/version failures, bounds, key redaction, JSON round trip.
- `cargo test -p autodev-server` — fail-closed without key or bearer, wrong bearer `401`, protected push attested as `BLOCK` and verified, forged decision rejected.
