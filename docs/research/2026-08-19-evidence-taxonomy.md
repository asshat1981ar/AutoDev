# Evidence taxonomy

- **Repository evidence:** commits, diffs, hashes, status, source files.
- **Execution evidence:** typed execution records, exit status, artifacts.
- **Verification evidence:** tests/build/lint/static/security results.
- **Policy evidence:** capability/risk decision and trusted approval reference.
- **Recovery evidence:** observations reconciling uncertain effects/state.
- **Evaluation evidence:** frozen task/configuration/verifier outcomes and comparisons.
- **User evidence:** explicit scoped approvals/product decisions.

Evidence classes have different trust/provenance requirements; they should not be collapsed into generic agent messages.
