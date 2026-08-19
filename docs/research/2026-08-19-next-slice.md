# Next slice entry criteria — Harness Asset Protocol

Do not begin implementation until the ExecPlan control-plane PR has executable CI evidence and review findings are resolved.

Then:

1. Refresh Harness Protocol v1 schema/reference implementation.
2. Complete `2026-08-19-harness-schema-gap-template.md` with evidence-backed classifications.
3. Refresh Deep Agents harness-profile semantics and complete the profile gap matrix.
4. Inspect AutoDev's existing `plugin.rs`, `skill.rs`, capability/policy types, Cline fabric, and MCP server contracts.
5. Write the Harness Asset Protocol ADR/spec with explicit import trust semantics.
6. Implement the smallest typed internal asset slice plus adversarial authority tests before adding loaders/installers.
