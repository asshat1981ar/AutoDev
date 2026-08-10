# ADR-003: Distributed Execution — Do We Need It?

- **Status:** Proposed (research)
- **Date:** 2026-08-09
- **Deciders:** Principal systems architect
- **Scope:** Research only. **No distributed workers are implemented.**
- **Related:** `ADR-001-forgecore-execution.md`, `ADR-002-evidence-persistence.md`, `docs/architecture/sdlc-orchestrator.md`

> **Naming note:** this is the third ADR. `ADR-002` is already taken by
> `ADR-002-evidence-persistence.md`, so this document is numbered `ADR-003`.

## Context

AutoDev is a **local-first**, model-agnostic, multi-agent software engineering
platform. The core is a Rust ForgeCore execution kernel plus an in-process SDLC
orchestrator built around durable tasks. The README has a long-range roadmap item for
an optional Go worker fabric / distributed workers.

This ADR asks: **does AutoDev actually need distributed execution at this stage, and
if so, which fabric?** It compares the candidate technologies and the deployment
topologies, and recommends a path — **only** if the evidence supports one.

## Requirements & constraints

| Constraint | Implication |
| --- | --- |
| Offline-first | Distributed execution must not make the core require a network/cluster. |
| Mobile / Android primary target | A phone is a *client*, not a reliable worker host. |
| Security | Remote execution expands the trust boundary (ADR-001's sandbox). |
| Latency | Per-action round trips should not dominate execution. |
| Reliability / recoverability | The orchestrator already checkpoints durable tasks (ADR-002). |
| Complexity budget | The platform is early; a distributed fabric is a large complexity adder. |

## Candidate technologies

| Tech | What it is | Strengths | Weaknesses |
| --- | --- | --- | --- |
| **Go workers** | Separate worker processes/bins | Simple process model, isolated | New language + build; not a protocol |
| **gRPC** | HTTP/2 + protobuf RPC | Typed contracts, streaming, codegen | Needs a server; protobuf toolchain |
| **WebSockets** | Bidirectional TCP transport | Simple, browser-friendly | No queueing/durability/routing |
| **NATS** | Lightweight message bus (pub/sub, JetStream) | Small footprint, durable queues, TLS | A broker; Rust ecosystem thinner |
| **QUIC** | UDP transport (HTTP/3) | Low latency, 0-RTT, migration | Very low-level; NAT traversal hard on mobile |
| **Actor systems** | Message-passing concurrency model | Clean state+comms encapsulation | Overkill; agents already logical + in-process |
| **Task queues** | Durable work queues | Reliable dispatch, retries | A broker dependency |
| **Remote Ollama nodes** | Point model fabric at a remote server | Heavier models off-device | Network + trust; hurts offline story |

## Topologies

| Topology | Offline | Security | Latency | Reliability | Complexity | Scalability | Mobile fit |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Local-only** | ✅ | ✅ | ✅ | ✅ checkpoints | ✅ | ⚠️ single device | ✅ |
| **Controller + workers** | ❌ | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ❌ phone worker |
| **Peer-to-peer** | ❌ | ❌ | ⚠️ | ⚠️ | ❌❌ | ⚠️ | ❌ |
| **Hybrid** | ✅ core | ⚠️ gated | ✓ LAN | ✅ local-first | ⚠️ | ⚠️ optional | ✅ client |

## Analysis against actual needs

1. **Core value is local-first.** A required distributed fabric contradicts primary positioning.
2. **Orchestrator is already durable + recoverable** (checkpoints + transition log). A task queue would duplicate the in-process graph.
3. **Bottleneck is the model, not compute.** Agent logic is cheap; local inference dominates latency. Remote workers don't fix the cost driver.
4. **Mobile constraints.** A phone is a poor worker host (battery, thermals, lifecycle); it is a client.
5. **Early-stage complexity budget.** Hardening the trusted execution boundary (ADR-001 tier 2) matters more than a broker/worker/IDL.

## What the evidence supports

- **A full distributed fabric is not justified now.** No target problem (perf, reliability, scale) is currently blocking.
- **The one genuinely useful increment** is **remote Ollama nodes** (point the existing provider-neutral `ModelProvider` at a remote `OllamaProvider` base URL) — an already-supported configuration change, not a new distributed system.
- **If/when real distributed execution is needed**, the evidence points to a **hybrid topology** with a **task-queue flavor** (NATS JetStream or gRPC request-reply), not P2P/actors/raw QUIC. Controller+workers is acceptable only as explicit opt-in, never the default.

## Recommendation

**Do not implement distributed workers at this stage.** Keep local-first execution as the default and only supported mode. Document and expose **remote Ollama nodes** via the existing `OllamaProvider(base_url)` configuration — no new distributed architecture.

Revisit only when there is *evidence* (usage, performance, or a concrete multi-device requirement) that a single local node is insufficient. Then prefer a **hybrid topology** with a **task-queue fabric** (NATS JetStream or gRPC request-reply) behind a worker abstraction, gated by explicit opt-in.

## Rejected alternatives (for the future)

| Approach | Why not now |
| --- | --- |
| Go workers as default | New language/build system with no need |
| Peer-to-peer | Highest security/complexity; worst offline/mobile fit |
| Actor systems | Agents already logical + in-process |
| QUIC | Too low-level; mobile NAT issues |
| WebSockets | Transport only; no queueing/durability |
| Remote Ollama as default | Network + trust; contradicts offline-first |

## Consequences

- **Positive:** preserves security/offline/complexity posture; no new runtime/broker/IDL; effort stays on hardening the local kernel + orchestrator.
- **Negative:** no multi-device scale-out — accepted, since there is no evidence of need.
- **Migration path:** introduce a `Worker` abstraction behind the orchestrator's `TaskExecutor`/`Verifier` seams later so a task-queue backend could slot in without rewriting orchestration. Remote Ollama is available now via existing config.
