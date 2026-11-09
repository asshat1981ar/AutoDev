---
allowed-tools: mcp__github_ci__get_ci_status,mcp__github_ci__get_workflow_run_details,Bash(git rev-parse:*)
description: Draft a PR evidence block for scripts/validate_pr_evidence.py
---

Use as `@claude /evidence` on a pull request. The harness CI job fails unless the PR body passes `scripts/validate_pr_evidence.py`; Claude does not run that script here (no script tools), CI does.

1. Get the head SHA with `git rev-parse HEAD` (the PR branch is checked out). Do not use `gh pr view`: it is denied. Get the CI run for that exact head SHA with the CI tools.
2. Write a `### Commands run` section with the exact commands from the `.github/workflows/ci.yml` jobs that ran, one per line.
3. Write a `### Evidence` section whose first line is `CI run / artifact / output:` followed by the run URL.
4. Keep the template's trust-boundary rule: exactly one top-level box ticked. Tick "No trusted execution or authorization changes" only if the diff touches no ForgeCore, MCP/HTTP/SSE/RPC, protocol identity or KMP boundary.
5. Say plainly that the block is unvalidated and the harness CI job validates it.
6. Reply with the block in your tracking comment for the owner to paste into the PR body. Do not edit the PR body yourself: an `edited` event re-runs every CI job (`ci.yml` pull_request types include `edited`).
