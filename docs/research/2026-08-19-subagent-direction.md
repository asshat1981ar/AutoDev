# Subagent architecture direction

Subagents should be explicit scoped workers with identity, role/description, model/profile, skill sources, requested tool/capability surface, context budget, output contract, and provenance.

Default inheritance should be conservative: context and capabilities are narrowed to the task. Any requested expansion is evaluated independently by policy. Subagent completion returns structured results/evidence references to the parent rather than its entire working transcript, reducing long-run context growth.

Async/background subagents require durable task identity, cancellation/status semantics, and checkpoint-safe result collection before production use.
