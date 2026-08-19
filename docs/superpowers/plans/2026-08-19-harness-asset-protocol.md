# Harness Asset Protocol — Milestone 2 Plan

## Goal

Create AutoDev's portable harness-asset representation while maximizing compatibility with Harness Protocol v1 and preserving ForgeCore's stronger trust boundary.

## Required research gate

Before implementation, compare Harness Protocol v1 schema fields for plugins, skills, MCP servers, environment, instructions, permissions, integrity, and governance against AutoDev's proposed Skill, AgentProfile, Tool, McpServer, Hook, Prompt, Policy, Workflow, Evaluator, and ContextProvider assets.

Classify each field as `lossless`, `extension-required`, or `unsupported`. Do not freeze a competing wire format until this matrix exists.

## Proposed slices

1. Schema-gap/conformance matrix and ADR.
2. Internal typed `HarnessAsset` domain with provenance, integrity, trust classification, compatibility constraints, and requested capabilities.
3. Harness Protocol v1 importer into authority-free AutoDev asset requests.
4. Exporter for losslessly representable assets.
5. Declarative HarnessProfile layer for model/provider-specific prompt/tool/subagent/skill tuning.
6. Adversarial tests proving imported permissions cannot mint `AuthorizationGrant` or bypass policy.
7. Simulation fixtures comparing equivalent native and imported harness configurations.

## Completion criteria

The milestone is complete when representative Harness Protocol profiles round-trip where semantics align, unsupported security semantics fail closed, provider/model profiles remain declarative, and independent tests demonstrate that imported harness configuration has no direct execution authority.
