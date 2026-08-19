# ContextProvider asset direction

Context providers produce bounded evidence for planning/agents: repository retrieval, documentation, code graph, issue/PR context, memory, or other sources. Each result should retain provider/source identity and retrieval parameters where practical.

A context provider grants no mutation authority. Context selection should be deterministic or reproducibly parameterized enough for evaluation. Sensitive sources require policy-aware access and should avoid copying secrets into long-lived plan state.
