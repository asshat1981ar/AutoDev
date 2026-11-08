@AGENTS.md

<!-- BEGIN claude-code-action (generated) -->
## Claude in GitHub Actions
- AGENTS.md is the harness; follow it exactly. CodeRabbit stays the first-pass reviewer; Claude adjudicates, it does not authorize.
- Every PR body must pass `python scripts/validate_pr_evidence.py` (template in `.github/PULL_REQUEST_TEMPLATE.md`).
- Run cargo with `--manifest-path crates/Cargo.toml`, Gradle with `./kotlin/gradlew -p kotlin`.
- Review output and verification PASS are evidence only, never an AuthorizationGrant.
<!-- END claude-code-action -->
