---
name: cross-prompting
description: |
  Load this skill when you need multi-agent prompt exchange protocols —
  having one agent generate output, another critique it, a third verify.
  Provides structured Architect → Builder → Reviewer → Verifier chains.
model: mistral-large-latest
preferred_tools: ["read_file", "write_file", "edit", "grep", "bash", "team_run_task", "team_await_runs"]
temperature: 0.0
user-invocable: true
---

# Cross-Prompting Protocol

**Purpose:** Structured multi-agent prompt exchange for robust, critiqued output.

## The Four-Stage Chain

### Stage 1: Architect — Design

Dispatch to architecture-design skill:
```
team_run_task(agentId="architect", task="Design the interface for X...")
```

**Output:** Design doc with interface proposal, risks, verification plan.

### Stage 2: Builder — Implement

After design is stable, dispatch implementation:
```
team_run_task(agentId="builder", task="Implement the design for X...")
```

**Output:** Implementation with passing tests.

### Stage 3: Reviewer — Critique

Dispatch review:
```
team_run_task(agentId="reviewer", task="Review the implementation of X for correctness...")
```

**Output:** Findings with severity levels.

### Stage 4: Verifier — Test

Dispatch verification:
```
team_run_task(agentId="verifier", task="Verify the implementation of X...")
```

**Output:** Test evidence, build results, lint results.

## Adversarial Variation (Hard Mode)

Instead of sequential stages, run Builder and Adversary in parallel:

1. **Builder** implements feature
2. **Adversary** independently tries to break it (edge cases, invalid inputs)
3. **Builder** fixes based on adversary findings
4. **Verifier** confirms both pass

## Tools

Use `scripts/cross_prompt.py` for automated chain execution:
```bash
python3 scripts/cross_prompt.py \
  --task "Implement feature X" \
  --chain architect builder reviewer verifier \
  --engram-port 8080
```