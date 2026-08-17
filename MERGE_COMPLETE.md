# MERGE_COMPLETE

The Kotlin Multiplatform modules live under `kotlin/` and are rebuilt on top of
the Rust `forge-core` core:

- `mpp-core` — code-graph extraction, platform filesystem (`expect`/`actual`),
  MCP tool dispatcher, AST patch review.
- `mpp-codegraph` — symbol-graph query engine.
- `mpp-server` — Ktor Netty server with Server-Sent Events streaming.
- `mpp-ui` — dependency-free diff/preview rendering.

Build and test (the Gradle wrapper auto-provisions the distribution and JDK 17
toolchain via the Foojay resolver):

```bash
cd kotlin
./gradlew clean assemble test
./gradlew ktlintCheck
```

`commonMain` is pure; OS primitives live behind `expect`/`actual` contracts.
CI is consolidated in `.github/workflows/ci.yml` (Rust + Kotlin + Python).
