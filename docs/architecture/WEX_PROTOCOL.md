# WEX Protocol — Worktree Exchange Protocol

**Version:** 1.0.0 · **CLI:** `scripts/wex-cli.py` · **State:** `.worktrees/.registry/` (runtime state, gitignored)

## Purpose

WEX coordinates multiple git worktrees (and their agent crews) working in parallel on
the same repository without path collisions, with durable heartbeats and an
evidence exchange registry. It complements AGENTS.md ownership boundaries: WEX
coordinates *who works where*; it never grants execution or approval authority.

## Concepts

| Concept | Meaning |
|---|---|
| Worktree identity | `worktree_id`, branch, purpose, revision — persisted in `manifest.json` at `init` |
| Heartbeat | Liveness record per worktree (`heartbeat.json`); stale after 300s (5 min) |
| Path reservation | Claim on a file/dir so only one worktree edits it at a time |
| Collision | Two worktrees reserving overlapping paths — must be resolved before commit |
| Evidence exchange | JSON evidence records pushed to `evidence-exchange/`, pullable by any worktree |

## Registry layout

```
.worktrees/.registry/
├── manifest.json          # worktree identities
├── reservations.json      # path → worktree claims
├── handoff-queue/         # cross-worktree handoffs
└── evidence-exchange/     # shared evidence records (JSON)
```

> Note: this directory is runtime state under `.worktrees/` (gitignored).
> Durable copies of important evidence should also be committed to `docs/`
> when they document landed work.

## Commands

All commands are run from inside a worktree:

```bash
python scripts/wex-cli.py init                       # register this worktree
python scripts/wex-cli.py heartbeat --task T --status active --milestone M
python scripts/wex-cli.py list                       # all known worktrees
python scripts/wex-cli.py dashboard                  # list + collisions + evidence count
python scripts/wex-cli.py reserve <path>             # claim a path for this worktree
python scripts/wex-cli.py check-collisions           # fail if overlaps exist
python scripts/wex-cli.py push-evidence <file.json>  # publish evidence record
python scripts/wex-cli.py pull-evidence              # fetch registry evidence locally
```

Heartbeat statuses: `active` · `blocked` · `completed`.

## Rules

1. **Reserve before editing.** Check `check-collisions` before every commit.
2. **One reservation owner.** Never edit a path reserved by another worktree;
   coordinate release instead.
3. **Heartbeat honesty.** Set `blocked` immediately when blocked; include the
   blocker in `--milestone`. False blockers waste cycles (see cycle-2026-08-21:
   "no JDK 17" recorded although Corretto was installed off-PATH).
4. **Evidence records are structured.** Required fields: `evidence_id`,
   `type`, `result` (`passed|failed|wired-blocked|…`), plus check/file lists.
   Use `passed` only when checks actually ran green.
5. **Fail closed.** A collision or missing registry is a stop condition, not a warning.

## Evidence record example

```json
{
  "evidence_id": "team-1--mpp-core-integration-2026-08-21",
  "type": "config-integration",
  "result": "passed",
  "module": "mpp-core",
  "checks": ["./gradlew :mpp-core:assemble --no-daemon", "./gradlew ktlintCheck --no-daemon"],
  "files_modified": ["kotlin/gradle.properties"],
  "_wex_source": "team-1"
}
```

## References

- [AGENTS.md](../../AGENTS.md) — harness rules and verification contract
- [DEVELOPMENT_CYCLE_PLAN.md](../../DEVELOPMENT_CYCLE_PLAN.md) — current cycle usage
- [TEAM_COLLABORATION.md](../../TEAM_COLLABORATION.md) — crew roles and workflow
