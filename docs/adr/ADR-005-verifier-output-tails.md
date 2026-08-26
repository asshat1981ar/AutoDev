# ADR-005: Verifier evidence output-tail retention

## Status

Accepted (2026-08-25)

## Context

`VerifierEvidence` (forge-core `evaluation.rs`) records verifier step outcomes as booleans, exit codes, timings, and SHA-256 hashes of stdout/stderr. Full streams were deliberately hash-only to keep persisted evidence tamper-evident and bounded (ADR-002).

This made real failures undiagnosable from CI logs during EP-2026-08-25: the self-evaluation corpus smoke failed intermittently (`android-debug-apk`, exit 1, 127s/147s elapsed), and because Gradle's actual error text was discarded, no agent or human could determine root cause without re-running the 3-minute eval — which is itself flaky. Two merge-blocking CI failures on byte-identical code resulted.

## Decision

Retain a bounded **tail** of each captured stream alongside the existing hashes:

- `VerifierEvidence.stdout_tail` / `stderr_tail`: last 2048 bytes rendered as lossy UTF-8.
- Fields are `#[serde(default)]`, so previously persisted evidence deserializes unchanged and old producers remain compatible with new readers.
- Full-stream capture remains capped at 64 KiB (existing `MAX_STREAM_BYTES`); tails are taken after capture, so memory/persistence growth is bounded at ~4 KiB per step execution.
- Hashes remain authoritative for integrity; tails are advisory diagnostics and must never be treated as authorization or verification input.

The first consumer is `autodev-eval`'s `required_steps_detail`, which appends a single-line, 400-char-bounded tail excerpt to failure assertion messages so transient verifier failures become self-diagnosing in CI logs.

## Consequences

- Transient verifier failures can be root-caused from existing CI logs.
- Evidence records grow by up to ~4 KiB per step; acceptable against the diagnostic value.
- Tails may contain build-tool noise; consumers must not parse them as contract — they are human/agent diagnostics only.
- Future alternative (rejected for now): full stream persistence behind an evidence store — higher blast radius, deferred until tails prove insufficient.
