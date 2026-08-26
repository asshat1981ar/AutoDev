# Stale Verification Gates — Docs/CI Mismatch

## Summary

After the Kotlin MPP + Cline fabric merge, the repository had three independent verification surfaces (Rust `cargo`, Kotlin `gradlew` + `ktlint`, Python `py_compile` + `unittest` + `node --check`) but contributor guidance in `README.md` and ad-hoc agent instructions only described the Rust gates (`cargo fmt` / `cargo test`). Contributors and coding agents following the stale docs could produce a "green" Rust run while silently missing required Kotlin/Python gates. CI would later fail on `ktlint` or `py_compile`, causing wasted review cycles. No production incident occurred; risk was repeated CI churn and false confidence in local green status.

Who saw it: maintainers reviewing PRs after the `merge/kotlin-mpp` branch landed.
Why it matters: ForgeCore's evidence contract (`ExecutionEnvelope.evidence.required` must be present AND passing; unknown names fail closed) means a partial verifier set must never satisfy a stronger task contract. Stale docs undermine that guarantee.

## Root Cause

- The repo evolved from Rust-only to polyglot (Rust workspace at `crates/`, Kotlin at `kotlin/`, Python fabric at `tests/` + `.cline/`) without a single durable agent instruction file. Knowledge was split across `README.md`, `MERGE_COMPLETE.md`, and per-module docs.
- No drift check compared documented commands against `.github/workflows/ci.yml`. CI added the Kotlin and Python jobs; docs did not track them.
- Historically there was no `AGENTS.md` and no file-scoped `.github/instructions/*.instructions.md`, so agents had no deterministic instruction hierarchy to consult before finishing.

## Prevention

- **Instruction:** `AGENTS.md` section 3 now enumerates all canonical verification commands with exact working directories and package-manager rules. `.github/instructions/rust.instructions.md`, `kotlin.instructions.md`, and `python-node.instructions.md` provide file-scoped reinforcement via `applyTo` frontmatter.
- **Detection:** `scripts/check_harness_drift.py` checks:
  1. Every canonical CI fragment in `.github/workflows/ci.yml` appears in `AGENTS.md` (and Rust gates in `README.md`).
  2. Referenced wrapper/manifest files exist.
  3. Forbidden root manifests are absent.
  4. Failure docs have required sections + a `Detection` mention in Prevention.
  Failure mode is a non-zero exit with actionable messages, suitable as a CI job and local pre-PR gate.

  ```bash
  python scripts/check_harness_drift.py         # local gate
  python scripts/check_harness_drift.py --verbose  # explain passing checks
  ```

  **Manual review point:** If adding a new verification gate (e.g., a new lint tool), update `.github/workflows/ci.yml`, `AGENTS.md`, and `scripts/check_harness_drift.py`'s `CANONICAL_CI_FRAGMENTS` in the same PR. The script intentionally fails closed on unknown required evidence names.

## Evidence

- CI workflow with three jobs: `.github/workflows/ci.yml` (rust: `cargo fmt`/`clippy`/`build`/`test`/`docker build`; kotlin: `gradlew clean test` + `assemble*` + `ktlintCheck` + APK upload; python: `py_compile` + `unittest` + `node --check`)
- Durable instructions: `AGENTS.md`, `.github/instructions/rust.instructions.md`, `.github/instructions/kotlin.instructions.md`, `.github/instructions/python-node.instructions.md`
- Drift script: `scripts/check_harness_drift.py`
- Adoption report: `docs/harness/adoption-report.md`
- ADRs: `docs/adr/ADR-001-forgecore-execution.md`
- Verified locally:

  ```text
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo build --workspace
  cargo test --workspace
  python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
  python -m unittest discover -s tests -v
  node --check scripts/termux-kanban.mjs && node scripts/termux-kanban.mjs --check
  python scripts/check_harness_drift.py
  ```
