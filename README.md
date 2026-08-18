# AutoDev

AutoDev is a local-first, model-agnostic multi-agent software-development runtime. Agents propose typed intent; ForgeCore authorizes and executes that intent inside explicit capability, workspace, approval, evidence, and verification boundaries.

## Core development loop

```text
Goal
  ↓
TaskGraph
  ↓
Context / dispatch
  ↓
ExecutionEnvelope
  ↓
Policy + trusted authorization
  ↓
ForgeCore execution
  ↓
EvidenceStore
  ↓
VerificationFabric
  ↓
Verified ─────────→ complete
Rejected ─────────→ bounded replan
Approval required → blocked until approved
```

The runtime deliberately separates **generation** from **verification**. An action reporting that it succeeded is not enough to complete a task. Required verification evidence must actually run and pass.

## Execution envelope

`ExecutionEnvelope` is the durable hand-off contract between planning, execution, and verification. It binds:

- task, run, operation, and action identity;
- bounded context references;
- risk and capability declarations;
- approval requirements and trusted approval references;
- required and produced evidence references;
- lifecycle state and bounded attempt count.

Lifecycle transitions are explicit and validated:

```text
planned → authorized → executing → verifying → verified
                           ↓            ↓
                        rejected ←──────┘
                           ↓
                       replanning
                           ↓
                         planned
```

## Trusted authorization

Agent/model payloads and declared capabilities are untrusted intent. Human approval is represented separately by a kernel-owned `AuthorizationGrant` and is supplied only at the trusted execution boundary.

This means a payload field such as:

```json
{"approved": true}
```

is **not authority**. ForgeCore strips caller-supplied Git approval state and only recreates the internal authorization marker from a trusted grant. The same grant model is used for high-risk file reads, writes, and patches.

Public effect adapters remain fail-closed when no grant is supplied.

## Required verification evidence

`ExecutionEnvelope.evidence.required` names the checks that must be present and passing before a task can enter `verified`.

Canonical verification names are:

- `unit_tests`
- `build`
- `lint`
- `static_analysis`
- `security`

A report where every executed check passed still fails the task if a declared required check never ran. Unknown required evidence names also fail closed. This prevents a partial verifier set from accidentally satisfying a stronger task contract.

## Durable verified orchestration

`VerifiedOrchestrator` composes the existing `TaskGraph` scheduler with the evidence-driven `DevelopmentLoop` without replacing the legacy orchestrator.

It persists one execution envelope per task across attempts:

- `Verified` → `TaskStatus::Completed`
- `Replanned` → `TaskStatus::Ready`
- `Exhausted` → `TaskStatus::Failed`
- missing approval → `TaskStatus::Blocked`

Approval resume reuses the same envelope and does not consume an execution attempt.

`VerifiedOrchestratorState` is serializable so envelope lifecycle and retry state can be recovered after process restart.

## Rust objective control plane

`autodev-server` is the Rust HTTP/SSE adapter over the verified ForgeCore path. Submitted objectives are persisted locally, advanced by one serialized worker, converted from model output into typed intent, rebound to trusted agent-profile capabilities and trusted risk floors, and then executed through the existing `VerifiedOrchestrator`. The HTTP and MCP adapters do not execute repository effects directly.

Start the server from the Rust workspace:

```bash
cd crates
AUTODEV_WORKSPACE=/path/to/repository \
AUTODEV_MODEL_BASE_URL=http://localhost:11434 \
AUTODEV_MCP_BEARER_TOKEN=replace-with-a-local-secret \
cargo run -p autodev-server
```

Configuration:

- `AUTODEV_PORT` — HTTP port, default `8080`.
- `AUTODEV_WORKSPACE` — trusted repository workspace root, default `.`.
- `AUTODEV_STATE_DIR` — optional trusted objective/evidence state directory. When omitted, AutoDev uses a sibling directory named `.autodev-state-<workspace-name>`; a state directory equal to or beneath the execution workspace is rejected after canonicalization.
- `AUTODEV_MODEL_BASE_URL` — model-provider base URL, default `http://localhost:11434`.
- `AUTODEV_MAX_FILE_BYTES` — ForgeCore workspace file-size limit, default 16 MiB.
- `AUTODEV_MCP_BEARER_TOKEN` — bearer credential for `/mcp`; when absent or empty, the MCP route fails closed.
- `AUTODEV_MCP_ALLOWED_HOSTS` — optional comma-separated MCP Host-header allowlist.
- `GITHUB_WEBHOOK_SECRET` — optional HMAC secret for the GitHub webhook endpoint.

Objective API:

- `POST /api/v1/objectives` — submit repository metadata, description, and optional branch as untrusted intent.
- `GET /api/v1/objectives` — list public objective projections.
- `GET /api/v1/objectives/:id` — fetch one public objective projection.
- `GET /api/v1/events/stream` and `GET /events` — consume typed lifecycle events over SSE.
- `POST /webhooks/github` — accept signed GitHub issue-opened events when a webhook secret is configured.
- `/mcp` — stateless MCP proposal/observation adapter backed by the same durable objective store and protected by its bearer/host checks.

Objective snapshots persist the public projection, `TaskGraph`, `VerifiedOrchestratorState`, and fingerprinted execution-evidence records. Produced evidence references therefore remain resolvable after runner reconstruction and process restart. The `repository` field in an objective is metadata; it does not select a local execution path. Only `AUTODEV_WORKSPACE` establishes the ForgeCore workspace boundary.

Model-declared risk is advisory. The control-plane proposal and execution-binding boundaries apply ForgeCore's trusted semantic minimum risk before profile checks or approval decisions; an untrusted model or custom proposer cannot downgrade a mutating write or Git operation to bypass approval.

There is deliberately **no public client approval endpoint** in this slice. Caller/model fields such as `{"approved": true}` cannot unblock an objective. The internal resume path accepts an approval grant scoped to the objective and task and then resumes the same durable execution envelope.

The MCP route has its own bearer and Host-header checks, but the general objective HTTP/SSE API does **not** yet have a client-authentication layer. The service currently binds to `0.0.0.0`; treat it as a local/development control plane and do not expose it to an untrusted network until that authentication boundary is implemented.

## Repository context fabric

ForgeCore includes deterministic bounded repository retrieval. Context selection is local-first, reproducible, and budgeted by maximum files and bytes. Context is treated as evidence for planning rather than permission to mutate the repository.

## Cline / Termux

AutoDev includes a Cline development fabric and a Termux-compatible Kanban launcher. CI validates the Python entry points, Termux launcher, and Cline development-fabric tests alongside the Rust kernel.

For Android/Termux environments, prefer portable subprocess and Streamable HTTP MCP paths instead of assuming desktop-only PTY, Bun, or Docker support.

## Verification gates

The Rust workflow currently requires:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

It also validates the Cline/Termux fabric. A development slice is not considered green until these gates pass.

## Design principle

> Agents propose intent. Policy authorizes capabilities. Trusted components execute. Independent verifiers produce evidence. Orchestrators advance or replan from that evidence.

This separation is the foundation for safe autonomous software development.
The intent is not maximum autonomy. AutoDev favors bounded, observable, recoverable development loops whose claims can be independently verified.

## Kotlin Multiplatform modules

The `kotlin/` workspace contains the KMP 2.x control-plane modules. Build and
test from the `kotlin/` directory with the Gradle wrapper (no system Gradle
required; the wrapper auto-provisions the distribution):

```bash
cd kotlin
./gradlew clean assemble test
./gradlew ktlintCheck
```

| Module | Source set | Responsibility |
| --- | --- | --- |
| `mpp-core` | `commonMain` + `jvmMain`/`iosMain` | Code-graph extraction, platform filesystem (`expect`/`actual`), MCP tool dispatcher, AST patch review |
| `mpp-codegraph` | `commonMain` | Symbol-graph query engine (declarations, scope membership, offset resolution) |
| `mpp-server` | `jvmMain` | Ktor Netty server with Server-Sent Events streaming (`/health`, `/events`) |
| `mpp-ui` | `commonMain` | Dependency-free diff/preview rendering (Nano DSL) |

`commonMain` is pure: no JVM or Darwin types leak across the boundary. OS
primitives live behind `expect`/`actual` contracts resolved per target.

## Status

Early architecture and foundation phase. The trusted execution, agent registry, model fabric, orchestration, verification, provenance, and first deterministic repository-context primitives are now established. APIs and module boundaries are expected to evolve while the execution protocol is integrated.

## Cline Development Fabric

The repository includes a Cline-native development fabric under `.cline/`. Install it into
another repository with `python install.py --project /path/to/repo`, or inspect changes first
with `python install.py --project /path/to/repo --dry-run`. Existing project files are skipped
by default; `--force` creates backups before replacement. The package provides routing rules,
progressive Skills, specialist agents, safety/context hooks, local plugin tools, and scoped
external MCP profiles. See [.cline/README.md](.cline/README.md).

## Termux Cline Kanban compatibility

On Android ARM64, Cline Kanban can fail when upstream `node-pty` has no usable Android native binding. AutoDev includes a self-healing compatibility launcher that probes the installed PTY, repairs it only when needed with a pinned Android ARM64 prebuilt, verifies the native binary checksum, and then launches Kanban.

```bash
node scripts/termux-kanban.mjs --repair-only
node scripts/termux-kanban.mjs
```

See [docs/termux-kanban.md](docs/termux-kanban.md) for diagnostics, force-repair, and shell-alias usage.

## License

AutoDev is released under the MIT License. See [LICENSE](LICENSE).
