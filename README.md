# AutoDev

AutoDev is a local-first, model-agnostic multi-agent software engineering platform inspired by the ForgeOS architecture evolved from OllamaDev.

The project treats AI-assisted development as an engineered system rather than a single coding prompt. Agents plan and collaborate through typed actions; a policy layer authorizes capabilities; a trusted execution core performs changes; repository exploration supplies bounded evidence; verification produces evidence; and the orchestrator decides whether work advances or is replanned.

## Architecture

```text
                    AutoDev
                       |
                Kotlin Control Plane
                       |
                Agent Orchestrator
                       |
          PLAN -> ACT -> VERIFY -> REPLAN
                       |
              Repository Context Fabric
                       |
                Typed Agent Protocol
                       |
                Rust ForgeCore
             /        |        \
        sandbox     Git       process
        filesystem  patches   execution
                       |
          +------------+------------+
          |            |            |
       Ollama         MCP        GitHub
          |
   local / LAN / cloud models
```

## Initial goals

- Local-first development with Ollama-compatible models.
- Multi-agent SDLC orchestration.
- Typed, validated agent actions instead of uncontrolled command text.
- Human approval for risky operations.
- Rust-based trusted execution for filesystem, process, patch and Git operations.
- Reproducible artifacts and execution provenance.
- Independent verification through tests, builds, static analysis and security checks.
- Repository-scale context selection with deterministic local retrieval.
- Optional distributed workers as the platform matures.

## Planned stack

| Layer | Technology | Responsibility |
| --- | --- | --- |
| Control plane | Kotlin + Jetpack Compose | Android UI, lifecycle, local state |
| Orchestration | Kotlin | Agent lifecycle and SDLC state machine |
| Context fabric | Rust | Repository retrieval, ranking and context budgets |
| Execution kernel | Rust | Sandbox, filesystem, Git, patches, processes, policy |
| Model fabric | Ollama | Local and network model execution |
| Capability layer | MCP | External tools and services |
| Persistence | SQLite/Room initially | Tasks, agents, checkpoints and metadata |
| Distributed fabric | Go, planned | Remote workers and agent federation |

## Development roadmap

1. Workspace foundation.
2. Agent registry and capability model.
3. Ollama model discovery and routing.
4. Typed agent-action protocol.
5. Rust ForgeCore execution boundary.
6. Repository context fabric and bounded retrieval.
7. PLAN -> ACT -> VERIFY -> REPAIR orchestration.
8. Approval and policy engine.
9. Verification and artifact provenance.
10. Project memory and knowledge graph.
11. Optional Go worker fabric.
12. Cross-platform clients where justified.

## Design principle

Agents propose intent. Context retrieval supplies relevant repository evidence. Policies authorize intent. ForgeCore executes authorized operations. Verifiers produce evidence. The orchestrator advances or replans.

This separation is the foundation for safe autonomous software development.

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
