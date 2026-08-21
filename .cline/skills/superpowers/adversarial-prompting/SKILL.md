---
name: adversarial-prompting
description: |
  Load this skill when you need adversarial/red-team self-critique patterns.
  Use to stress-test solutions, find edge cases, and harden implementations
  before they reach production.
model: mistral-large-latest
preferred_tools: ["read_file", "write_file", "edit", "grep", "bash", "team_run_task", "team_spawn_teammate"]
temperature: 0.0
user-invocable: true
---

# Adversarial Prompting

**Purpose:** Red-team self-critique to harden implementations before verification.

## Protocol

### 1. Define the Target

State clearly what the implementation claims to do:
```
"Function X accepts input Y and produces output Z, handling cases A, B, C."
```

### 2. Generate Attack Surface

List all edge cases and failure modes:
- Empty inputs
- Maximum/malformed inputs  
- Concurrent access
- Resource exhaustion
- Permission bypasses
- State corruption on error paths

### 3. Red-Team Attack

Dispatch an adversarial agent:
```
team_spawn_teammate(agentId="adversary", rolePrompt="You are a red-team...")
team_run_task(agentId="adversary", task="Break the implementation of X by...")
```

### 4. Blue-Team Defense

The original builder receives findings and hardens the implementation.

### 5. Third-Party Verdict

A neutral agent (Verifier) judges:
- Did the adversary find real issues?
- Did the defense properly address them?
- Are there remaining risks?

## Tools

Use `scripts/cross_prompt.py --chain builder adversary verifier` for automated adversarial cycles.