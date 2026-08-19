# Agentic-system testing pyramid

1. Pure domain/unit tests for lifecycle, policy, schema, routing, and parsing.
2. Adversarial boundary tests for authority/workspace/evidence/import semantics.
3. Integration tests across plan -> task -> envelope -> verification contracts.
4. Fault-injection recovery tests.
5. Historical task/evaluation fixtures.
6. Android/KMP UI/API integration tests and install/startup smoke tests.
7. Production release end-to-end qualification.

Higher layers complement rather than replace cheap deterministic lower-layer tests.
