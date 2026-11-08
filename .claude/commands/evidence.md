---
allowed-tools: mcp__github_ci__get_ci_status,mcp__github_ci__get_workflow_run_details,Bash(gh pr view:*),Bash(python scripts/validate_pr_evidence.py:*)
description: Draft a PR evidence block that passes scripts/validate_pr_evidence.py
---

Use as `@claude /evidence` on a pull request. The harness CI job fails unless the PR body passes `python scripts/validate_pr_evidence.py`.

1. Read the PR and its head SHA (`gh pr view`). Get the CI run for that exact head SHA with the CI tools.
2. Write a `### Commands run` section with the exact commands from the `.github/workflows/ci.yml` jobs that ran, one per line.
3. Write a `### Evidence` section whose first line is `CI run / artifact / output:` followed by the run URL.
4. Keep the template's trust-boundary rule: exactly one top-level box ticked. Tick "No trusted execution or authorization changes" only if the diff touches no ForgeCore, MCP/HTTP/SSE/RPC, protocol identity or KMP boundary.
5. Synthesize `{"pull_request":{"body": <full body>}}` into `$RUNNER_TEMP/ev.json` and prove `python scripts/validate_pr_evidence.py "$RUNNER_TEMP/ev.json"` reports PASS. Do not post a block that does not pass.
6. Post the block as a comment for the owner to paste into the PR body. Do not edit the PR body yourself: an `edited` event re-runs every CI job (`ci.yml` pull_request types include `edited`).
