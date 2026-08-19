# Harness inspector direction

A future Harness screen should explain the resolved configuration for a run: model/provider profile, AgentProfile, skills, tools, MCP servers, hooks/workflows, requested capabilities, provenance/integrity, and policy decisions.

This makes configuration differences debuggable and supports evaluation/reproducibility. The inspector is read/diagnostic by default; configuration changes create a new versioned run/profile state rather than mutating historical provenance.
