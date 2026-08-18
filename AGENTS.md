# AGENTS.md — AutoDev Agent Harness

This file is the durable agent harness for AutoDev. It is project-specific and enforceable via `scripts/check_harness_drift.py` and `.github/workflows/ci.yml`. Follow it exactly; do not substitute generic defaults.

## 1. Project purpose and ownership boundaries

AutoDev is a local-first, model-agnostic multi-agent runtime. The trusted boundary is **ForgeCore** (`crates/forge-core`) — agents propose typed `AgentAction` intent, policy authorizes, ForgeCore executes inside workspace/approval/evidence boundaries.

Key ownership boundaries:

| Area | Path | Owner contract |
|------|------|---------------|
| Trusted execution kernel | `crates/forge-core/src/**` | Rust, no untrusted I/O without `AuthorizationGrant`; workspace-confined |
| Control-plane server | `crates/autodev-server/**` | Axum + rmcp stateless MCP adapter, no direct ForgeCore execution authority from CLI |
| Kotlin control plane | `kotlin/mpp-core`, `mpp-codegraph`, `mpp-server`, `mpp-ui` | KMP 2.x pure `commonMain`; platform code only in `jvmMain`/`iosMain`/`android-command-center` |
| Android command center | `kotlin/android-command-center/**` | Thin Compose app over KMP contracts |
| Python/Cline fabric | `install.py`, `bootstrap_cline_mcp.py`, `.cline/**`, `tests/**` | Fabric installer + hooks + skill routing; stdlib-only where noted |
| Termux launcher | `scripts/termux-kanban.mjs` | Node, pinned PTY with SHA-256 verification |
| Reference observability | `scripts/autodev-cli.py`, `web/command-center/**` | Read-only HTTP/SSE observer; **no** ForgeCore/Git/MCP authority |

## 2. Setup

### Prerequisites

- Rust stable toolchain with `rustfmt` + `clippy` (managed via `dtolnay/rust-toolchain@stable`)
- JDK 17 provisioned via Gradle Foojay resolver (no system Gradle required)
- Android SDK 35 + build-tools 35.0.0 only if building the APK
- Python 3.10 or 3.11
- Node 24 (for `scripts/termux-kanban.mjs` validation)

### One-time setup

```bash
# Rust — nothing to install; toolchain is pinned in CI via rustup
rustup show

# Kotlin — wrapper auto-provisions Gradle 8.10.2 + JDK 17 via Foojay
cd kotlin && ./gradlew --version && cd ..

# Python — stdlib only; optional virtual env
python3 -m venv .venv && source .venv/bin/activate

# No root package.json / pyproject.toml — do not create one
```

## 3. Build, lint, test, and verification commands

Run these **from the documented working directory**. Every command below is checked by `scripts/check_harness_drift.py` against `.github/workflows/ci.yml` and `README.md`.

### Rust (working directory `crates/`)

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
# Optional container gate (requires Docker):
docker build -f ../Dockerfile -t autodev-server:ci ..
```

### Kotlin (working directory `kotlin/`)

```bash
cd kotlin
./gradlew clean test :mpp-core:assemble :mpp-server:assemble :mpp-ui:assemble :mpp-codegraph:assemble :android-command-center:assembleDebug --no-daemon
./gradlew ktlintCheck --no-daemon
# Single-module shortcuts (also valid):
./gradlew :mpp-core:test --no-daemon
```

Package manager rule: **use only `kotlin/gradlew`**. Never invoke a system `gradle`. Do not add `kotlin/gradle/libs.versions.toml` unless the version catalog is adopted deliberately. Respect the Gradle cache keys in CI (`kotlin/**/*.gradle.kts`, `kotlin/gradle/wrapper/gradle-wrapper.properties`). Android SDK used in CI: `sdkmanager "platforms;android-35" "build-tools;35.0.0"` via `android-actions/setup-android@v3`.

### Python + Cline fabric (repo root)

```bash
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
python -m unittest discover -s tests -v
```

Package manager rule: **stdlib only** for `scripts/autodev-cli.py`. Do not add `requirements.txt`/`pyproject.toml` dependencies without an ADR.

### Node / Termux launcher (repo root)

```bash
node --check scripts/termux-kanban.mjs
node scripts/termux-kanban.mjs --check
```

### Full local verification (mirrors CI)

```bash
# Rust
cd crates && cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo build --workspace && cargo test --workspace; cd ..

# Kotlin
cd kotlin && ./gradlew clean test :mpp-core:assemble :mpp-server:assemble :mpp-ui:assemble :mpp-codegraph:assemble :android-command-center:assembleDebug --no-daemon && ./gradlew ktlintCheck --no-daemon; cd ..

# Python + fabric
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
python -m unittest discover -s tests -v

# Termux launcher
node --check scripts/termux-kanban.mjs && node scripts/termux-kanban.mjs --check

# Harness drift (must pass before every PR)
python scripts/check_harness_drift.py
```

### Reproducible verification under sandbox isolation (offline-capable)

```bash
bash scripts/verify_reproducible.sh  # runs offline-capable subset: cargo fmt --check (crates/), gradle --version, py_compile, node --check, drift; SKIPs network gates with Docker fallback
```

Network gates that require egress (`cargo build --workspace`, `cargo test --workspace`, `./gradlew check`) fail in the bwrap sandbox with `CONNECT tunnel failed, response 502` and Gradle daemon `Could not connect` — see [docs/failures/002-network-isolated-build-gates.md](/home/dev/AutoDev/docs/failures/002-network-isolated-build-gates.md). Reproduce fully via `docker build -f Dockerfile .` as in `.github/workflows/ci.yml`.
```

## 4. Package manager and dependency rules

- **Rust:** Workspace at `crates/Cargo.toml` with members `forge-core` + `autodev-server`. No root `Cargo.toml`. Do not run `cargo` from repo root. `Cargo.lock` is committed under `crates/Cargo.lock` — keep it.
- **Kotlin:** Gradle wrapper only. Kotlin 2.0.21, Gradle 8.10.2, ktlint 12.1.1 via `org.jlleitschuh.gradle.ktlint`. Android compileSdk 35 / minSdk 26 / targetSdk 35. Do not bump without updating CI `sdkmanager` line.
- **Python:** No `package.json`/`pyproject.toml`/`requirements.txt` at root. `scripts/autodev-cli.py` must remain dependency-free (urllib only). `tests/` uses `unittest` stdlib.
- **Node:** No root `package.json`. Only `scripts/termux-kanban.mjs` uses Node builtins (`node:crypto`, `node:fs`, `node:path`, `node:child_process`). PTY version pinned to `1.1.2` with SHA-256 `660a3025230f6035b7b8c000e8cca6ca3992bedaa05f7b165e7c3a5f1ae8ec8a`.

Do not run `npm install`/`yarn`/`pip install` that creates or rewrites lockfiles.

## 5. Safe editing rules

- **Do not edit generated artifacts:** `**/build/`, `**/target/`, `**/.gradle/`, `**/__pycache__/`, `*.apk`, `*.aab`, `crates/Cargo.lock` (except via `cargo update`), `kotlin/gradle/wrapper/gradle-wrapper.jar` (except via wrapper upgrader).
- **Do not create:** root `Cargo.toml`, root `package.json`, root `pyproject.toml`, or `kotlin/gradle/libs.versions.toml` without an ADR.
- **Do not bypass ForgeCore boundaries:** Never add direct filesystem/network/process execution that skips `Workspace` confinement or `AuthorizationGrant`. Public adapters stay fail-closed.
- **Preserve `commonMain` purity:** No `java.*`/`android.*`/`darwin.*` types in `kotlin/*/src/commonMain`. Use `expect`/`actual` contracts.
- **Preserve CLI authority boundary:** `scripts/autodev-cli.py` must not gain ForgeCore execution, approval, Git, or MCP write authority. It is read-only observer + objective enqueue.
- **Workspace patch hygiene:** After editing, check `git status` for collateral rewrites (`yarn.lock`, `Cargo.lock` drift, `gradle-wrapper.properties`). Revert unintended changes.
- **Forbidden paths for agents without grant:** `secrets/`, `*.pem`, `*.key`, `*.jks`, `*.p12`, `.env` — treat as read-denied; request `AuthorizationGrant` at trusted boundary if needed.

## 6. Testing expectations

- **Every Rust change:** `cargo test --workspace` must pass. Add adversarial tests for workspace/policy changes (`crates/forge-core/tests/`).
- **Every Kotlin change:** `./gradlew test` + `./gradlew ktlintCheck` must pass. Keep `commonTest`/`jvmTest` deterministic (no network).
- **Every Python/fabric change:** `python -m py_compile` + `python -m unittest discover -s tests -v` must pass.
- **Every launcher change:** `node --check` + `node scripts/termux-kanban.mjs --check` must pass.
- **Verification evidence contract:** `ExecutionEnvelope.evidence.required` names checks that must be present AND passing. A missing required check fails the task even if all executed checks passed. Unknown required names fail closed — do not invent new evidence names without updating `crates/forge-core/src/verification.rs`.
- **Harness drift:** `python scripts/check_harness_drift.py` must pass for any change to `README.md`, `AGENTS.md`, `docs/**`, `scripts/**`, `.github/workflows/ci.yml`, `kotlin/**/*.gradle.kts`, or `crates/**`.

## 7. PR and commit conventions

- Follow existing history: Conventional-ish prefixes (`feat:`, `fix:`, `docs:`, `chore:`) with optional scope `feat(forge-core):`, `feat(kotlin):`, `docs:`. No enforced linter, but CI titles are human-reviewed.
- Keep PRs slice-sized. One concern per PR. Link to an ADR under `docs/adr/` for structural/kernel changes.
- CI must be green on the three jobs (`rust`, `kotlin`, `python`) — PRs are blocked otherwise (`branches: [main]`).
- Do not force-push to `main`. Branch `merge/kotlin-mpp` is the only additional CI push branch.

## 8. How to record new failures and decisions

- **Failures:** Add `docs/failures/<NNN>-<slug>.md` using the template in `docs/failures/001-stale-verification-gates.md`. Required sections: Summary, Root Cause, Prevention, Evidence. Every failure must name an enforceable check (drift script, CI job, lint rule) or explain why automation would be brittle.
- **Decisions:** Add `docs/adr/ADR-<NNN>-<slug>.md` for kernel, execution boundary, sandbox, or multi-target architecture choices. Update `docs/harness/adoption-report.md` for harness changes.
- **Instructions:** Update this file in place. For file-scoped rules, edit `.github/instructions/*.instructions.md` with `applyTo` frontmatter. Do not create parallel `.github/copilot-instructions.md` without retiring one.

## 9. Harness enforcement

| Rule | Enforcement |
|------|-------------|
| Commands in docs match CI | `scripts/check_harness_drift.py` — compares `README.md`/`AGENTS.md` code fences against `ci.yml` |
| No forbidden lockfiles/root manifests | `scripts/check_harness_drift.py` — fails if `package.json`/`pyproject.toml`/root `Cargo.toml` appear without ADR |
| Failure docs have Detection | `scripts/check_harness_drift.py` — validates `docs/failures/*.md` structure |
| Kotlin `commonMain` purity | `cargo`/`gradle` compilation + `check_harness_drift.py` grep for illegal imports |
| CLI authority boundary | `check_harness_drift.py` grep for `forge_core`/`AuthorizationGrant` in `scripts/autodev-cli.py` |
| Full verification | `ci.yml` jobs + local `python scripts/check_harness_drift.py` |

Run `python scripts/check_harness_drift.py --help` for details.

## 10. References

- `README.md` — architecture, verification gates, KMP module table
- `.github/workflows/ci.yml` — canonical verification commands
- `docs/adr/ADR-001-forgecore-execution.md` — trusted execution boundary
- `docs/architecture/**` — protocol, runtime, and evidence specs
- `crates/forge-core/src/lib.rs` — module boundary map
