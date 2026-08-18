---
applyTo: "kotlin/**"
---

# Kotlin Multiplatform instructions

This guidance applies only to files under `kotlin/**`.

## Stack

- KMP 2.x (Kotlin 2.0.21), Gradle 8.10.2, `org.jlleitschuh.gradle.ktlint:12.1.1`, Foojay resolver `0.8.0`
- Modules: `mpp-core` (code-graph + expect/actual fs + MCP dispatcher), `mpp-codegraph` (query engine), `mpp-server` (Ktor SSE), `mpp-ui` (Nano DSL), `android-command-center` (Compose, compileSdk 35, minSdk 26, targetSdk 35)
- Repositories: `mavenCentral()` + `google()`; `gradlePluginPortal()` for pluginManagement

## Commands (run from `kotlin/`)

```bash
./gradlew clean test :mpp-core:assemble :mpp-server:assemble :mpp-ui:assemble :mpp-codegraph:assemble :android-command-center:assembleDebug --no-daemon
./gradlew ktlintCheck --no-daemon
# Module-targeted:
./gradlew :mpp-codegraph:test --no-daemon
./gradlew :mpp-core:assemble --no-daemon
```

## Rules

- Use **only** `kotlin/gradlew`. Never invoke system `gradle`. The wrapper auto-provisions JDK 17 via `jvmToolchain(17)`.
- `commonMain` is pure: no `java.*`, `android.*`, `androidx.*`, `darwin.*` imports. Platform code lives in `jvmMain`/`iosMain`/`androidMain` behind `expect`/`actual`.
- Do not add `kotlin/gradle/libs.versions.toml` (version catalog) without an ADR — current build uses inline versions.
- Do not edit `build/`, `.gradle/`, `.idea/`, `local.properties`, or any `*.apk`/`*.aab`. CI uploads `android-command-center-debug.apk` via `actions/upload-artifact@v4`.
- APK gate requires `ANDROID_HOME` + SDK 35. Use `scripts/build_apk.sh` locally; CI uses `sdkmanager "platforms;android-35" "build-tools;35.0.0"` + `android-actions/setup-android@v3`.
- Keep Gradle cache keys (`kotlin/**/*.gradle.kts`, `kotlin/gradle/wrapper/gradle-wrapper.properties`) in sync with any new `*.gradle.kts` file.

## Verification

Every `kotlin/**` change must pass `clean test` + all `assemble` + `ktlintCheck`. Run `python scripts/check_harness_drift.py` to catch stale wrapper/docs drift.
