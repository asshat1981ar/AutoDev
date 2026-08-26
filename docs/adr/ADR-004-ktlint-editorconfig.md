# ADR-004: ktlint configuration via root `.editorconfig` (retire decorative YAML)

**Status:** Accepted
**Date:** 2026-08-22
**Deciders:** ox-alpha (primary), Config Manager role; supersedes part of the centralized-config architecture (`82a53e8`)

## Context

`config/kotlin/ktlint/.ktlint.yaml` was created as the "centralized ktlint rules"
file and consumed via `kotlin/.ktlint.yaml` symlink. Research during cycle
2026-08-21-kotlin-mpp showed:

1. ktlint 12.x (pinterest/ktlint 1.x line) reads **`.editorconfig` only** — it
   never parses YAML rule files. The YAML file had zero effect on linting.
2. The YAML content was internally contradictory (wildcard-import rule both
   enabled and disabled), proving no tool ever consumed it.
3. `org.jlleitschuh.gradle.ktlint` 12.x delegates configuration to ktlint's
   standard editorconfig mechanism (plus an `additionalEditorconfig` extension
   for non-file properties).
4. A complete canonical `config/defaults/common/.editorconfig` already existed
   but was never activated at the repository root.

## Decision

- Activate the canonical file as repo-root symlink:
  `.editorconfig → config/defaults/common/.editorconfig` (contains `root = true`,
  so hierarchical discovery stops there — one source of truth).
- Carry ktlint properties in its Kotlin section: `ktlint_code_style = official`;
  wildcard imports allowed only in `**/*Test.kt`.
- **Delete** `config/kotlin/ktlint/.ktlint.yaml` and the `kotlin/.ktlint.yaml`
  symlink; update `config/validate.py` and affected docs.

## Consequences

- Lint behavior is now actually configurable and matches documented intent.
- Editors (IntelliJ/VS Code) read the same file — single style source for
  humans and CI.
- Symlink-at-root is POSIX-portable and CI-safe (unlike per-module symlinks
  relative to worktree layout).
- Follow-ups retired: the "convert YAML to editorconfig" item from
  KOTLIN_CONFIG_INTEGRATION.md is closed by this ADR.

## Verification

Empirical gate: `./gradlew ktlintCheck --no-daemon` must pass with the new
discovery chain (executed on merged tree; see evidence registry,
`primary--followups-2026-08-22`).

## References

- [docs/architecture/KOTLIN_CONFIG_INTEGRATION.md](../architecture/KOTLIN_CONFIG_INTEGRATION.md)
- [config/defaults/common/.editorconfig](../../config/defaults/common/.editorconfig)
- Cycle retrospective memory `cycle-2026-08-21-kotlin-mpp-completed` (Engram)
