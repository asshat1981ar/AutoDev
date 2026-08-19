# Deep Agents source summary

Primary sources reviewed: current `langchain-ai/deepagents` graph construction, subagent middleware, architecture notes, and changelog.

Relevant current semantics include declarative HarnessProfile overrides keyed by provider/model; profile tuning of prompts, tools, middleware, subagents, and skills; explicit subagent tool/model/middleware/skill/permission/output configuration; filesystem permission enforcement in middleware; and load-bearing middleware that cannot be silently excluded.

Implication for AutoDev: adopt declarative assembly/profile ideas and explicit subagent contracts where useful, but keep kernel security/verification outside any removable profile or middleware layer.
