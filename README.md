# AutoDev

AutoDev is a local-first, model-agnostic multi-agent software engineering platform inspired by the ForgeOS architecture evolved from OllamaDev.

The project treats AI-assisted development as an engineered system rather than a single coding prompt. Agents plan and collaborate through typed actions; a policy layer authorizes capabilities; a trusted execution core performs changes; verification produces evidence; and the orchestrator decides whether work advances or is replanned.

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
- Optional distributed workers as the platform matures.

## Planned stack

| Layer | Technology | Responsibility |
| --- | --- | --- |
| Control plane | Kotlin + Jetpack Compose | Android UI, lifecycle, local state |
| Orchestration | Kotlin | Agent lifecycle and SDLC state machine |
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
6. PLAN -> ACT -> VERIFY -> REPAIR orchestration.
7. Approval and policy engine.
8. Verification and artifact provenance.
9. Project memory and knowledge graph.
10. Optional Go worker fabric.
11. Cross-platform clients where justified.

## Design principle

Agents propose intent. Policies authorize intent. ForgeCore executes authorized operations. Verifiers produce evidence. The orchestrator advances or replans.

This separation is the foundation for safe autonomous software development.

## Status

Early architecture and foundation phase. APIs and module boundaries are expected to evolve while the execution protocol is established.

## License

AutoDev is released under the MIT License. See [LICENSE](LICENSE).
