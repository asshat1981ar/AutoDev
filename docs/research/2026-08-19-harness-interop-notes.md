# Harness interoperability research — 2026-08-19

This note records external design inputs for the later Federated Harness Kernel milestones. It does not change ForgeCore authority semantics.

## Harness Protocol

Harness Protocol v1 proposes a vendor-neutral `harness.yaml` schema for plugins, skills, MCP servers, environment requirements, instructions, permissions, integrity metadata, and governance. AutoDev should evaluate compatibility/import-export before finalizing its own Harness Asset Protocol wire format. Internal AutoDev state may remain richer when required for trusted execution and evidence.

Source: https://github.com/harnessprotocol/harness-protocol

## Deep Agents

Deep Agents now uses declarative harness profiles to vary prompt/tool/middleware/subagent/skill behavior by provider or model. Its agent construction also gives subagents explicit tools, model overrides, middleware, skills, filesystem permissions, structured response formats, and human-interrupt configuration.

AutoDev should adopt the useful separation between portable asset/profile description and runtime assembly, but must not inherit external authority assumptions. ForgeCore remains the only component allowed to authorize effects.

Sources:
- https://github.com/langchain-ai/deepagents/blob/main/libs/deepagents/CHANGELOG.md
- https://github.com/langchain-ai/deepagents/blob/main/libs/deepagents/deepagents/graph.py
- https://github.com/langchain-ai/deepagents/blob/main/libs/deepagents/deepagents/middleware/subagents.py

## Design consequence

Before Milestone 2 (Harness Asset Protocol), perform a schema-gap analysis against Harness Protocol v1 and model/profile semantics against Deep Agents. Prefer a compatibility layer over a competing portable schema where semantics align. AutoDev-specific trust, authorization, provenance, evidence, recovery, and policy fields remain kernel-owned extensions and must fail closed when an imported harness cannot express them.
