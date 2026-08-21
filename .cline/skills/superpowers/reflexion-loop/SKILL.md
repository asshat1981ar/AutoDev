---
name: reflexion-loop
description: |
  Load this skill when you need iterative self-improvement via Engram memory.
  Queries past outcomes before starting new work, generates improvement
  suggestions from failure patterns, and stores episodic memories.
  Requires Engram MCP server running on port 8080.
model: mistral-large-latest
preferred_tools: ["read_file", "write_file", "edit", "grep", "bash"]
temperature: 0.0
user-invocable: true
---

# Reflexion Loop

**Purpose:** Iterative self-improvement via persistent memory and pattern recognition.

## The Loop

```
  ┌──────────────────────────────────────────────────────┐
  │                    REFLEXION LOOP                     │
  │                                                        │
  │  1. QUERY past patterns before starting new task       │
  │     → Engram memory: "similar tasks, mistakes to avoid"│
  │                                                        │
  │  2. PLAN with improvement suggestions from history      │
  │     → Apply lessons from past failures                  │
  │                                                        │
  │  3. IMPLEMENT the task                                  │
  │     → Execute with awareness of past issues             │
  │                                                        │
  │  4. CAPTURE outcome as episodic memory                  │
  │     → Engram memory: "what worked, what didn't"         │
  │                                                        │
  │  5. ANALYZE patterns across multiple episodes           │
  │     → Generate improvement suggestions for next loop    │
  │                                                        │
  │  6. PUSH evidence to WEX registry                       │
  │     → Evidence available to all worktrees               │
  │                                                        │
  └──────────────────────────────────────────────────────┘
```

## Usage

```bash
# Start Engram MCP server
python3 -c "from skills.engram_memory import EngramMemory; e = EngramMemory(); e.start_server(); print('Engram on :8080')" &

# Run reflexion loop
python3 scripts/reflexion_loop.py --task "Task description" --store

# Or use the one-shot query before starting work
python3 scripts/reflexion_loop.py --query "What can I learn from past X tasks?"
```

## Memory Types

| Type | Purpose | Example |
|------|---------|---------|
| `episode` | Task outcomes | "Fixed bug by adding input validation" |
| `semantic` | Patterns | "Input validation prevents 80% of test failures" |
| `declarative` | Facts | "function X expects ISO 8601 dates" |
| `procedural` | How-to | "Steps to run cross-prompting chain" |