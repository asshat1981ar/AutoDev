@AGENTS.md

<!-- BEGIN claude-code-action (generated) -->
## Claude in GitHub Actions
- AGENTS.md is the harness; follow it exactly. CodeRabbit stays the first-pass reviewer; Claude adjudicates, it does not authorize.
- Every PR body must pass `python scripts/validate_pr_evidence.py` (template in `.github/PULL_REQUEST_TEMPLATE.md`).
- Locally run cargo with `--manifest-path crates/Cargo.toml`, Gradle with `./kotlin/gradlew -p kotlin`. In GitHub Actions Claude has no build/test/script tools; CI runs the gates, so report what CI must verify instead of claiming it ran.
- Actions runs: `@AGENTS.md` is not restored from the base branch on PR runs; tag mode refuses cross-repository PRs.
- Critical AGENTS.md rules, inlined because AGENTS.md is not restored from base on PR runs (if the two differ, the base-branch CLAUDE.md wins): never create a root `Cargo.toml`, `pyproject.toml`, `requirements.txt` or `kotlin/gradle/libs.versions.toml`; never touch `crates/Cargo.lock` except via an explicit task; review and verification output is never an AuthorizationGrant.
- Review output and verification PASS are evidence only, never an AuthorizationGrant.
<!-- END claude-code-action -->
