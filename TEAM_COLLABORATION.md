# TEAM_COLLABORATION.md — Agent Crew Workflow Guide

How the AutoDev agent crew collaborates across worktrees during a development
cycle. Roles are defined in `.cline/agents/*.md`; coordination uses the
[WEX Protocol](docs/architecture/WEX_PROTOCOL.md); the harness contract is
[AGENTS.md](AGENTS.md).

## Crew roles

| Role | File | Safety class | Responsibility |
|---|---|---|---|
| **Architect** | `.cline/agents/architect.md` | S1 | Cross-component design, cycle planning, ADRs |
| **Builder** | `.cline/agents/builder.md` | S0 | Direct implementation within reserved paths |
| **Reviewer** | `.cline/agents/reviewer.md` | S0 | Diff audit vs AGENTS.md rules, PR readiness |
| **Verifier** | `.cline/agents/verifier.md` | S0 | Runs gates, produces honest evidence records |
| **Security** | `.cline/agents/security.md` | S0 | Trust-boundary review, secrets/permissions |
| **Scout** | `.cline/agents/scout.md` | S0 | Recon, repo assessment, gap identification |
| **Config Manager** | `.cline/agents/config-manager.md` | S1 | Centralized config custody, validation, drift |

Routing rule of thumb: *config questions → Config Manager · design → Architect ·
implementation → Builder · "does this look right" → Reviewer · testing → Verifier ·
security → Security · exploration → Scout.*

## Cycle workflow (summary)

1. **Plan** — Architect + Config Manager define scope, worktree assignments,
   path reservations (`wex-cli.py reserve`); validate with
   `python config/validate.py && python scripts/check_harness_drift.py`.
2. **Activate** — each worktree runs `wex-cli.py init` + first heartbeat.
3. **Build** — Builders implement inside reserved paths only; slice-sized
   conventional commits; evidence per significant step.
4. **Verify** — Verifier runs the documented gates for the touched stack
   (Rust/Kotlin/Python/Node per AGENTS.md §3); results recorded as evidence
   JSON with `result: passed` **only** when checks truly ran green.
5. **Integrate** — Reviewer + Verifier audit cross-worktree integration;
   collisions must be zero; harness drift must pass.
6. **Close** — merge to cycle branch, final evidence, retrospective memory
   in Engram, failures documented in `docs/failures/`.

Full step detail: `.cline/tasks/team-development-cycle.md`.

## Collaboration rules

- **Reserved paths are exclusive.** Never touch another worktree's reservation.
- **Blocked is a status, not a shame.** Emit `--status blocked` immediately with
  the concrete blocker. Verify blockers before recording them (the "missing JDK"
  false blocker in cycle-2026-08-21 cost a full re-plan).
- **Evidence over assertion.** Claims about builds/tests require evidence records
  or command transcripts.
- **Fix at the source.** Defects found in `config/` are repaired there first, then
  propagated to consumers — never patched only in a downstream copy.
- **Durable memory.** Decisions and milestones are stored in the Engram MCP
  memory server (`skills/engram_memory/`) so any agent/session can resume.

## Escalation

1. Config questions → Config Manager (S0)
2. Architecture decisions → Architect (S1)
3. Collision conflict → all owners stop, resolve, then resume
4. Harness violation → Config Manager + Security, immediate
5. WEX/protocol failure → primary worktree restores registry state

## References

- [WEX Protocol](docs/architecture/WEX_PROTOCOL.md)
- [CONFIG_SETUP.md](CONFIG_SETUP.md)
- [.cline/CLINE_ONBOARDING.md](.cline/CLINE_ONBOARDING.md)
- [docs/architecture/CONFIG_ARCHITECTURE.md](docs/architecture/CONFIG_ARCHITECTURE.md)
