# Repo conventions

Shared across asshat1981ar repos. Source: the Nov 25, 2026 dev efficiency review (rules R1 to R12). Status per repo is in the PR that added this file.

| ID | Convention |
|---|---|
| R1 | Only `*.example` env files are tracked. A token that was ever committed is rotated, not just deleted. GitHub secret scanning and push protection are on. |
| R2 | Claude Code Action workflows come from one reusable workflow (`workflow_call`) in GitHubOpsGem, pinned to one `claude-code-action` SHA. Repos call it and add their own gates. |
| R3 | No `--permission-mode auto` or bypass in CI. Review runs use `default`; `acceptEdits` only for @claude with an allowlisted tool set. A test asserts this. |
| R4 | One agent doc per repo, `CLAUDE.md` (AGENTS.md may point to it), under ~150 lines: purpose, setup, exact build/test/lint commands, write limits, current version. |
| R5 | Every action is pinned by full commit SHA with a version comment; Dependabot keeps the `github-actions` group current. |
| R6 | Dependabot runs weekly and groups updates per ecosystem, so a week is one PR per ecosystem, not one per package. |
| R7 | Stacked PRs go at most two deep. Drafts older than 14 days get triaged: merge, close, or a note saying why they stay open. Duplicates are closed on sight. |
| R8 | A red required check on the default branch blocks new feature PRs until it is fixed. |
| R9 | Runtime state is gitignored (`.worktrees/`, `*_pid`, agent state dirs, `logs/`, coverage JSON, planning scratch). Status reports live in `docs/reports/`, not the root. |
| R10 | The language's linter and formatter are configured in-repo and enforced in CI. Adopt by fixing the backlog first, then turning the check on. |
| R11 | A date-based switch (for example an enforce-after date) needs install evidence and a test that fails once the date passes without it. |
| R12 | Agent workflows unused for 30 days are retired. |
