# Claude automation

Claude Code Action is wired in as a pilot. AGENTS.md remains the authoritative harness; see `CLAUDE.md`.

## Turn it on

1. Add the repository secret `CLAUDE_CODE_OAUTH_TOKEN` (Settings, Secrets and variables, Actions).
2. Install the Claude GitHub App on this repository: https://github.com/apps/claude
3. Create the labels `claude-review` and `claude:adjudicate`.
4. Optional repository variables:
   - `CLAUDE_PAUSE=true` pauses every Claude job (kill switch). Unset or any other value resumes.
   - `CLAUDE_MODEL` overrides the model for all jobs (defaults: `claude-opus-5-5` for `claude.yml`, `claude-sonnet-5-5` elsewhere).
   - `CLAUDE_RUNNER=ubuntu-24.04-firewall` once enrolled in the egress-firewall preview (`.github/egress-firewall.yaml`; the stack hosts are unverified until the first run).
5. In branch protection for `main`, add `agent-approval-check` as a required check. `claude-adjudication` may be added later; no Claude job should be the only required check, because the subscription limit makes Claude jobs fail.

## What it does

| Workflow | Trigger | Behavior |
|---|---|---|
| `claude.yml` | `@claude` from the repo owner on issues, PRs, reviews | Tag mode, Opus, pushes `claude/*` branches. |
| `claude-review.yml` | PR touching ForgeCore, autodev-server, AMCX, commonMain, schemas, policies; or label `claude-review` | Read-only review that skips what CodeRabbit already said. Not a required check. |
| `claude-coderabbit-adjudicate.yml` | label `claude:adjudicate` | Classifies each CodeRabbit finding with evidence; fails when blocking findings are unresolved. If Claude gives no verdict it warns and reports a script count of unresolved CodeRabbit threads. |
| `agent-approval-check.yml` | PR events and comments | Requires 1 human approval on PRs with agent-authored commits. Approve with a review or `/approve <head-sha>`. |

Dependabot PRs and runs are skipped. Fork PRs are skipped.

## Commands

- `@claude <task>` on an issue or PR.
- `@claude /evidence` drafts a PR evidence block that passes `scripts/validate_pr_evidence.py`; paste it into the PR body yourself (editing the body re-runs all CI jobs).
- `/approve <head-sha>` on a PR with agent commits.

Review output and verification PASS are evidence only, never an AuthorizationGrant.
