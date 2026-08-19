# Durable memory direction

AutoDev should distinguish three memory classes:

- **Run memory:** typed plan/task/envelope/evidence state required to resume correctly.
- **Project memory:** durable decisions, failure prevention, repository architecture, and learned configuration outcomes.
- **Agent working memory:** bounded task/subagent context that may be summarized or discarded after producing durable artifacts.

Only the first two should influence future runs after explicit validation/provenance rules. Free-form agent working memory must not become an implicit authority or silently override repository truth.
