# CodeRabbit Review Policy

CodeRabbit is AutoDev's automated first-pass reviewer. It supplements, but does not replace, CI, repository policy, human review, or ForgeCore authorization.

## Review priorities

1. Trusted-boundary and authority violations.
2. Security defects and fail-open behavior.
3. Protocol semantic or provenance corruption.
4. Missing or misleading verification evidence.
5. Functional correctness.
6. Concurrency and state consistency.
7. Multiplatform boundary violations.
8. Documentation and maintainability.

## ForgeCore

`crates/forge-core/**` is the trusted execution kernel.

Flag:

- execution outside Workspace confinement;
- AuthorizationGrant bypass or fabrication;
- confused-deputy behavior;
- verification converted into execution authority;
- unchecked external input;
- path traversal or symlink escape;
- unsafe process execution;
- provenance or identity loss;
- fail-open trust-boundary behavior.

Verification PASS is evidence only. It MUST NOT grant execution authority.

## Control plane

MCP, HTTP, SSE, RPC, CLI, model output, and agent output are untrusted inputs.

They must not bypass ForgeCore policy or directly gain trusted execution authority.

## Kotlin Multiplatform

`kotlin/**/src/commonMain/**` must remain platform-neutral.

Flag direct use of java.*, javax.*, android.*, or platform-specific APIs.

## CI

`.github/workflows/**` is canonical verification evidence.

Required verification must exist and pass.

CodeRabbit approval is not execution authorization.
