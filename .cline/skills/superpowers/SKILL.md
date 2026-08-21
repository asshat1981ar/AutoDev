---
name: superpowers
description: |
  Load this skill when doing architectural or multi-agent development work.
  Provides access to subagent-driven-development, cross-prompting,
  adversarial-prompting, and reflexion-loop meta-skills.

  These superpower skills enable:
  - Parallel subagent swarms via team_spawn/team_run_task
  - Multi-agent cross-prompting protocols with critique/review cycles
  - Adversarial/red-team self-critique patterns
  - Iterative self-improvement reflexion loops via Engram memory
model: mistral-large-latest
preferred_tools: ["read_file", "write_file", "edit", "grep", "bash", "team_spawn_teammate", "team_run_task"]
temperature: 0.0
---

# Superpowers — Multi-Agent Meta-Skills

**Purpose:** Enable advanced multi-agent development patterns including parallelism, cross-prompting, adversarial testing, and self-improvement loops.

## Skills Index

| Skill | Path | Purpose |
|-------|------|---------|
| Subagent-Driven Dev | `superpowers/subagent-driven-development/SKILL.md` | Parallel agent teams via task dispatch |
| Cross-Prompting | `superpowers/cross-prompting/SKILL.md` | Multi-agent prompt exchange protocols |
| Adversarial Prompting | `superpowers/adversarial-prompting/SKILL.md` | Red-team / self-critique patterns |
| Reflexion Loop | `superpowers/reflexion-loop/SKILL.md` | Iterative self-improvement via Engram memory |

## Integration with AutoDev

- **ForgeCore**: Superpowers propose typed `AgentAction` intent only — ForgeCore remains execution authority
- **WEX**: Evidence from cross-prompting and reflexion loops is pushed to `.worktrees/.registry/evidence-exchange/`
- **Engram Memory**: Reflexion loops store episodic memories via `skills/engram_memory/engram_mcp.py`
- **AGENTS.md**: All superpower patterns respect AGENTS.md section 5 authority boundaries