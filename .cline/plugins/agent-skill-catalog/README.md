# Agent Skill Catalog Plugin

This plugin provides an AutoDev-safe workflow for discovering, reviewing, and packaging external agent skills. It is inspired by the public [`VoltAgent/awesome-agent-skills`](https://github.com/VoltAgent/awesome-agent-skills) index, but it does **not** bulk-vendor third-party skills.

## Safety model

- Store unreviewed external skills as metadata only.
- Keep copied third-party bodies, scripts, and commands out of active skills until license and security review pass.
- Treat skills as behavioral instructions that can affect agent authority and verification behavior.
- Preserve ForgeCore as AutoDev's trusted execution boundary.
- Do not let a skill grant shell, Git, filesystem, MCP, cloud, production, or secret access by instruction alone.

## Included skills

- `skill-curator` — triage external skills and decide whether they are worth review.
- `skill-security-auditor` — threat-model skill instructions, scripts, and authority assumptions.
- `skill-license-reviewer` — record licensing, attribution, and vendoring constraints.
- `skill-packager` — adapt reviewed skills into AutoDev-native plugin/skill layout.

## Quarantine

The `quarantine/` directory is intentionally metadata-only. Put candidate records and review notes there, not unreviewed copied skill implementations.
