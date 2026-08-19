# Structured agent output contracts

Subagents/specialists should return typed or schema-validated outputs for machine-consumed decisions: findings, proposed patches, context selections, architecture alternatives, verification requests, etc. Free-form prose remains useful for explanation but should not be parsed heuristically for security-critical state.

Invalid structured output triggers bounded repair/retry rather than optimistic interpretation. Output schema/version becomes part of AgentProfile/HarnessProfile provenance where relevant.
