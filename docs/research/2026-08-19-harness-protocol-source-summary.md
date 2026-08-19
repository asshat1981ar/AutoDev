# Harness Protocol source summary

Primary source reviewed: `harnessprotocol/harness-protocol` GitHub repository, v1 schema-layer candidate.

The protocol describes a vendor-neutral `harness.yaml` for portable AI coding-agent operational context. Its documented surface includes plugins, skills, MCP servers, environment requirements, behavioral instructions, permissions, integrity metadata, and governance policy. A reference implementation (`harness-kit`) is described as providing parsing, validation, plugin loading, MCP lifecycle management, and CLI behavior.

Implication for AutoDev: use it as an interoperability target for the external harness representation, while retaining internal extensions for trusted authorization, evidence, recovery, and provenance where the protocol is insufficient.
