# Agent Registry

This document describes ForgeCore's **logical agent registry** — the set of
declarative agent profiles that the platform can dispatch work to.

## Design principle

Agents are **logical runtimes, not independent processes or microservices.** An agent
is a declarative [`AgentProfile`] stored in an [`AgentRegistry`] plus (once work is
assigned) an in-process [`AgentRuntime`] handle. No unnecessary processes are created:
an agent is a set of declared characteristics that the orchestrator and policy layer
reason about in-process.

This is consistent with the platform's staged evolution: orchestration is in-process
first, and distributed workers (Go fabric) are a later roadmap item.

## Registry model

```text
AgentRegistry
 ├── profiles[]   (AgentProfile — declarative definition per role)
 └── runtimes[]   (AgentRuntime — live, in-process instances)
```

### Types

| Type | Purpose |
| --- | --- |
| `AgentRole` | The role enum: `Architect, Researcher, Planner, Developer, Tester, SecurityReviewer, Release`. |
| `AgentCapability` | Reuses the platform's `Capability` type (declared capability set). |
| `AgentProfile` | The full declaration of an agent. |
| `ModelRequirement` | Model family, preferred model, minimum context tokens. |
| `AgentPolicy` | Risk ceiling, allowed tools, timeout, retry policy. |
| `RetryPolicy` | `max_attempts` + backoff (ms). |
| `AgentRuntime` | A live instance: profile + state, health, current task, assigned model. |
| `AgentHealth` | `Idle, Running, Healthy, Degraded, Unavailable`. |
| `AgentState` | Lifecycle: `Registered, Instantiated, Running, Healthy, Degraded, Terminated`. |
| `AgentRegistry` | Holds profiles + runtimes; register/instantiate/start/health/terminate. |

## What each agent declares

Every [`AgentProfile`] declares:
- **role** — one of the seven `AgentRole`s.
- **capabilities** — the `Capability` set the agent is granted.
- **model requirements** — family, preferred model, min context.
- **risk ceiling** — the highest `RiskLevel` the agent may undertake.
- **tools** — the tool names the agent may invoke.
- **timeout** — maximum execution timeout (seconds).
- **retry policy** — max attempts + backoff.
- **expected output** — the expected output shape of the agent's work.

### Initial roles (from `default_profiles()`)

| Role | Capabilities | Risk ceiling | Model | Expected output |
| --- | --- | --- | --- | --- |
| **Architect** | read/write/patch, git | High | qwen2.5-coder | architecture_adr |
| **Researcher** | read, git | Low | qwen2.5 | research_summary |
| **Planner** | read, git | Low | qwen2.5 | task_plan |
| **Developer** | read/write/patch, git, git:write | Medium | qwen2.5-coder | code_change |
| **Tester** | read, run_test, execute | Medium | qwen2.5 | test_report |
| **SecurityReviewer** | read, git, execute | High | qwen2.5 | security_review |
| **Release** | git, git:write, git:destructive, approval:critical | High | qwen2.5 | release_manifest |

## Lifecycle

```text
Registered ──(instantiate)──▶ Instantiated ──(start)──▶ Running ──▶ Healthy ──▶ Terminated
                                                                 │
                                                                 └──▶ Degraded
```

- **Registered** — the profile is in the registry.
- **Instantiated** — a runtime handle is created from a profile (`AgentRegistry::instantiate`).
- **Running** — assigned to a task + model (`AgentRegistry::start`).
- **Healthy** — completed work, idle (`AgentRegistry::mark_healthy`).
- **Degraded** — operating under constraints (e.g. a fallback model).
- **Terminated** — no longer active (`AgentRegistry::terminate`; health → `Unavailable`).

## Capability & risk enforcement

`AgentProfile::may(capability, risk)` returns whether the agent holds the capability
**and** the risk is within its ceiling. This lets the policy layer and orchestrator gate
work before dispatch:

```rust
if profile.may(&Capability::WriteFile, action.risk) { /* dispatch */ }
```

A role's ceiling is structural: e.g. a `Researcher` (ceiling `Low`) cannot be assigned
a `High`-risk change even if it holds the capability.

## Tests

Coverage in `crates/forge-core/src/agent.rs`:

- registers all seven roles
- every profile declares role/capabilities/model/timeout/retry/output
- developer can write but researcher cannot
- risk ceiling enforced
- architect has no execute capability
- release holds destructive git
- instantiate + lifecycle (start → running → healthy)
- unknown agent reported
- duplicate register is idempotent