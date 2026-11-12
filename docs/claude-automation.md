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
   - `CLAUDE_GATE_MODE=warn` downgrades adjudicator failures to warnings (default `enforce`).
5. In branch protection for `main`, add `agent-approval-check` as a required check. It is the only check here that is suitable to require. Never require a Claude job (`claude`, `claude-review`, `adjudicate`): the subscription limit makes Claude jobs fail and the adjudicator only runs when a label is applied.

## What it does

| Workflow | Trigger | Behavior |
|---|---|---|
| `claude.yml` | `@claude` from the repo owner on issues, PRs, reviews | Tag mode, Opus, pushes `claude/*` branches. Same-repo PRs only: a step refuses cross-repository PRs (fail closed). Claude has read-only git/gh tools plus the action's file and commit tools; no build, test or repo-script commands, so CI on its pushes (without secrets) supplies test results. `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1`. |
| `claude-review.yml` | PR touching ForgeCore, autodev-server, AMCX, commonMain, schemas, policies; or label `claude-review` | Relevance is checked inside the job (kernel paths or the label, on any PR). Read-only review on `GITHUB_TOKEN`; summary posted by a script; CodeRabbit comments are pre-fetched by a script, filtered to `coderabbitai[bot]`, and read from a file. Not a required check. |
| `claude-coderabbit-adjudicate.yml` | label `claude:adjudicate` | Classifies each CodeRabbit finding with evidence (explanation only). The enforced verdict is deterministic: unresolved CodeRabbit review threads (script count, fails closed if unavailable or over 100 threads), plus any blocking `must_fix` finding Claude derives. Empty Claude output with a clean script count is a WARN, never PASS. Not a required check. |
| `agent-approval-check.yml` | PR events and comments | Requires 1 human approval on PRs with agent-authored commits. Comment `/approve <head-sha>` to re-evaluate immediately; a native Approve review is only counted on the next push or `/approve` comment (the workflow has no `pull_request_review` trigger by design). |

Dependabot PRs and runs are skipped. Fork PRs are skipped or refused before Claude starts.

## Residual risks

- `CLAUDE.md` imports `@AGENTS.md`. The action restores `CLAUDE.md` and `.claude/` from the base branch on PR runs but not `AGENTS.md`, so a same-repo branch can change the instructions Claude reads. The fork guard limits this to branches only the owner can push; no build or script tools are allowlisted, so the effect is limited to what Claude writes with its file tools.
- In tag mode the action embeds PR/issue text and comments filtered by author (`include_comments_by_actor`). `gh pr view`, `gh issue view`, `gh search` and `gh api` return unfiltered comments, so they are not allowlisted and are in `--disallowedTools` in every job; the owner-only trigger remains the main control.
- `--permission-mode auto` sends unlisted tools to a safety classifier instead of denying them, so `claude.yml` also denies `cargo`, `python`, `node`, `npm`/`pnpm`/`yarn`, `./kotlin/gradlew`, `make`, `bash` and `sh`.
- `claude-review.yml` and the adjudicator use `GITHUB_TOKEN` (they never push; comments come from github-actions[bot]). Claude has no comment-writing tool: the review summary returns as `structured_output` and a script upserts one marker comment.
- Tokens still reach the Claude process (the App token sits in `.git/config`); `.claude/settings.json` denies reading it and `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` strips credentials from subprocesses.
- The review and adjudicator read a comment file written from CodeRabbit output, which is model output and treated as data.

## Commands

- `@claude <task>` on an issue or PR.
- `@claude /evidence` drafts a PR evidence block for `scripts/validate_pr_evidence.py` (Claude cannot run it; the harness CI job does); paste it into the PR body yourself (editing the body re-runs all CI jobs).
- `/approve <head-sha>` on a PR with agent commits.

Review output and verification PASS are evidence only, never an AuthorizationGrant.
