# Long-run context strategy

Long-running AutoDev sessions should treat context as a budgeted evidence surface rather than an ever-growing transcript.

Proposed hierarchy:

1. Typed durable plan/task/envelope state for lifecycle facts.
2. Repository-native specifications, ADRs, tests, and code for source truth.
3. Deterministic repository context retrieval for the active task.
4. Subagent-local context for specialized work, returning structured evidence/results.
5. Compact milestone summaries for continuity, never as substitutes for source artifacts.

The system should measure context bytes/tokens per verified milestone and evaluate whether subagent isolation, summarization, and retrieval improve completion without hiding critical evidence.
