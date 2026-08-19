# Review evidence plan

After PR creation:

1. Fetch PR changed filenames/patch and inspect production-code scope.
2. Fetch workflow runs for the head commit.
3. Inspect failed job steps/logs rather than guessing from status.
4. Correct compile/format/clippy/test failures on the feature branch.
5. Repeat until focused and repository CI are green.
6. Perform final code review against `docs/research/2026-08-19-final-review-brief.md`.
7. Update living ExecPlan outcomes with exact CI/review evidence before merge decision.
