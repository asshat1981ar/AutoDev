---
name: subagent-driven-development
description: |
  Load this skill when you need to distribute work across multiple specialized
  subagents using team_spawn_teammate and team_run_task.

  Use cases:
  - Parallel implementation across different modules
  - Multi-domain tasks (Rust + Kotlin + Python changes)
  - Concurrent verification and implementation
model: mistral-large-latest
preferred_tools: ["read_file", "team_spawn_teammate", "team_run_task", "team_task", "team_await_runs", "team_status"]
temperature: 0.0
user-invocable: true
---

# Subagent-Driven Development

**Purpose:** Parallelize development work across specialized subagents.

## When to Use

- The task spans multiple ownership boundaries (e.g., ForgeCore + Kotlin + Python)
- You need concurrent verification while implementation proceeds
- The work decomposes into independently verifiable milestones

## Protocol

### 1. Decompose the Task

Split work into independently verifiable slices, each with:
- Clear ownership boundary matching an agent's domain
- Acceptance criteria with observable evidence
- Bounded scope (one file group, one language, one concern)

### 2. Spawn or Dispatch

Use existing agents if they're already spawned:
```python
# Check current team
team_status()

# Dispatch to an existing agent
team_run_task(agentId="forge-core-engineer", task="...")
```

Or spawn new specialists:
```python
team_spawn_teammate(agentId="my-agent", rolePrompt="You are...")
team_run_task(agentId="my-agent", task="...")
```

### 3. Create Shared Tasks

Track all work items in the shared task list:
```python
team_task(action="create", title="...", description="...", assignee="agent-id")
```

### 4. Collect Results

```python
# Wait for all async runs
team_await_runs()

# Check task status
team_task(action="list")

# Read mailbox for results
team_read_mailbox(unreadOnly=True)
```

### 5. Integrate

Verify results across agents:
- Run the full verification suite (cargo test, gradlew, unittest)
- Check harness drift
- Push evidence to WEX registry
- Log mission handoff

## Safety Rules

- **Never duplicate authority**: Subagents cannot mint AuthorizationGrants
- **Verify independently**: Don't trust subagent claims — run checks yourself
- **Concatenation limits**: Keep delegated tasks focused (< 20 tool calls)
- **Reconcile interruptions**: If a subagent is interrupted, check evidence before retry