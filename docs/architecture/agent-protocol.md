# Agent Action Protocol

## Purpose

AutoDev separates agent reasoning from privileged execution. An agent produces a typed `AgentAction`. The policy layer validates the action, checks capabilities and risk, and either authorizes, denies, or requests human approval. Only authorized actions reach the execution layer.

## Lifecycle

```text
Agent
  |
  v
AgentAction
  |
  v
Schema validation
  |
  v
Capability + policy evaluation
  |          \
  |           +--> approval required --> Human
  v
Execution
  |
  v
ExecutionResult
  |
  v
Verification / artifacts
  |
  v
Orchestrator
```

## Design rules

1. Agent output is intent, not authority.
2. Every action has an explicit type, reason, risk level and task identity.
3. Capabilities are declared rather than inferred from arbitrary shell text.
4. High-risk operations must be policy-controlled.
5. Execution produces durable evidence.
6. Verification is separate from generation.
7. Actions must be traceable to a task and agent.
8. The protocol must remain language-neutral so Kotlin, Rust, Go and future clients can interoperate.

## Initial action types

- `read_file`
- `write_file`
- `patch_file`
- `execute`
- `git`
- `mcp`
- `run_test`
- `request_approval`

The schemas in this directory are the initial protocol contract. Implementations should validate against them rather than duplicating the contract in prompts.
