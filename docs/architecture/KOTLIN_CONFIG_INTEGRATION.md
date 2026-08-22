# Kotlin MPP ↔ Centralized Configuration Integration

**Status:** Implemented & verified (cycle-2026-08-21-kotlin-mpp, M3/M4)
**Scope:** `kotlin/*` modules consuming `config/kotlin/*`

## The pattern (as implemented)

1. **Single source of truth.** All Gradle/ktlint tuning lives in
   `config/kotlin/gradle.properties` and `config/kotlin/ktlint/.ktlint.yaml`.
   Version pins must match AGENTS.md §3 exactly
   (Kotlin 2.0.21, Gradle 8.10.2, ktlint plugin 12.1.1, JDK 17, Android SDK 35).
2. **Consumption by copy.** `kotlin/gradle.properties` is a byte-copy of the
   centralized file (Gradle only reads properties from the project root /
   gradle user home — an arbitrary include mechanism does not exist for this
   file). When the centralized file changes, re-copy and commit.
3. **Consumption by editorconfig discovery.** The repo-root `.editorconfig`
   (symlink to `config/defaults/common/.editorconfig`, marked `root = true`)
   carries ktlint properties (`ktlint_code_style = official` etc.). ktlint 12.x
   discovers it hierarchically from every linted source file. The former
   `config/kotlin/ktlint/.ktlint.yaml` was retired in ADR-004: ktlint never
   read YAML; the file was documentation-only.
4. **JDK discovery.** Never set `org.gradle.java.home` to an unexpanded env var.
   Let Gradle auto-detect installed JDKs, export `JAVA_HOME`, or pass
   `-Dorg.gradle.java.installations.paths=...`.

## Hard-won constraints (do not regress)

| Rule | Reason |
|---|---|
| No `$ENV_VAR` values in `.properties` | Gradle does not expand them; literal path breaks every build (proven by simulation) |
| No Groovy/Kotlin expressions | `.properties` is plain java.util.Properties; blocks parse as garbage keys |
| No duplicate keys | Silent last-wins; drift between intent and behavior |
| `android.enableBuildCache` forbidden | Removed in AGP 7.0; fails configuration of android modules outright |
| Low-RAM hosts need `-Dorg.gradle.jvmargs=-Xmx900m --max-workers=2` | 3.5 GB hosts OOM-kill the default `-Xmx2g` build silently mid-configuration |

## Verification recipe

```bash
cd kotlin
export JAVA_HOME=/path/to/jdk-17   # or rely on auto-detection
./gradlew :mpp-core:assemble :mpp-core:jvmTest --no-daemon --console=plain \
  '-Dorg.gradle.jvmargs=-Xmx900m -XX:MaxMetaspaceSize=256m' --max-workers=2
./gradlew ktlintCheck --no-daemon --console=plain \
  '-Dorg.gradle.jvmargs=-Xmx900m -XX:MaxMetaspaceSize=256m' --max-workers=2
```

Verified results (2026-08-21): mpp-core assemble ✅ · jvmTest 23/23 ✅ ·
ktlintCheck across all modules ✅ · mpp-server/mpp-ui/mpp-codegraph assemble ✅.
`android-command-center:assembleDebug` requires the Android SDK (CI-only).

## Known limitations / follow-ups

- ~~`config/kotlin/ktlint/.ktlint.yaml` is documentation-only~~ **Resolved in
  ADR-004**: retired in favor of the root `.editorconfig` carrying real ktlint
  properties.
- Copy-based propagation can silently drift from source; enforced since
  2026-08-22 by `check_config_parity` in `scripts/check_harness_drift.py`.

## References

- [AGENTS.md](../../AGENTS.md) — canonical version pins
- [docs/architecture/CONFIG_ARCHITECTURE.md](CONFIG_ARCHITECTURE.md)
- [WEX Protocol](WEX_PROTOCOL.md) — worktree coordination used during integration
