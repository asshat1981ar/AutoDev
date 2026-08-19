# Context evaluation direction

Evaluate context strategies on downstream verified task outcomes, not retrieval similarity alone. Useful secondary metrics include bytes/tokens, relevant-file coverage, irrelevant-context ratio, latency, and whether critical dependency/test/policy files were omitted.

Historical tasks can compare deterministic repository retrieval, code-graph augmentation, memory, and subagent-local retrieval. Protected tests should detect context strategies that appear efficient only because they miss safety/verification constraints.
