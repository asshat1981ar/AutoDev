# AutoDev Mistral Connector Portfolio

Date: 2026-08-21

## Selection rule

A Connector is included only when it adds a distinct capability to one or more AutoDev agents and its authority can be bounded. Priority uses RICE as a directional framework, then adjusts for authority risk and coordination value.

## P0 — Core engineering

| Connector | Origin | Primary value | Authority risk | Initial mode | Decision |
|---|---|---|---:|---|---|
| GitHub App | Mistral featured | Repository/issues/PR context | 3 | External, read-oriented | Adopt; no direct source/release mutations yet |
| Linear | Mistral featured | Agile backlog/project state | 2 | External, mutations require confirmation | Adopt as canonical backlog system |
| Context7 | Custom MCP | Current version-specific library docs | 1 | Managed, private, read-only | Adopt |
| DeepWiki | Custom MCP | Repository semantic exploration | 1 | Managed, private, read-only | Adopt |

### Why Linear instead of Linear + Jira

Running two canonical issue trackers creates synchronization and ownership ambiguity. Linear is the initial hypothesis because it maps cleanly onto continuous agentic planning and is already represented as a Mistral featured Connector. Atlassian remains a later substitution option when a project requires Jira/Confluence rather than a parallel source of truth.

## P1 — Collaboration and knowledge

| Connector | Origin | Value | Risk | Status |
|---|---|---|---:|---|
| Notion | Mistral featured | Product/architecture knowledge | 2 | Deferred until a concrete knowledge workflow needs writes |
| Slack | Mistral featured | Team communication and operational signals | 3 | Deferred; outbound messages require confirmation |
| Google Drive / supported MCP path | Featured/MCP depending current platform path | Artifact/requirements retrieval | 2 | Deferred pending current migration path and source-of-truth policy |

P1 is deliberately not provisioned in the first slice. Adding broad communication/document access before a task needs it increases tool entropy and credential scope without improving core software delivery.

## P2 — Specialized automation

Candidates:

- CI/build evidence Connector
- CodeRabbit/review evidence integration
- dependency/supply-chain scanner
- security scanner
- artifact registry
- deployment environment
- observability/incident Connector

These are not automatically approved merely because an MCP server exists. Each requires publisher/repository verification, tool inventory, least-privilege mapping, and a sandbox direct-call test.

## RICE-style comparison

Scores are directional, on a 1–5 scale, and used to explain ordering rather than imply measurement precision.

| Connector | Reach | Impact | Confidence | Effort | RICE | Authority risk | Coordination value |
|---|---:|---:|---:|---:|---:|---:|---:|
| GitHub App | 10 | 5 | 5 | 2 | 125.0 | 3 | 5 |
| Linear | 4 | 4 | 5 | 2 | 40.0 | 2 | 4 |
| Context7 | 6 | 4 | 5 | 2 | 60.0 | 1 | 4 |
| DeepWiki | 6 | 4 | 4 | 2 | 48.0 | 1 | 4 |
| Notion | 4 | 3 | 4 | 2 | 24.0 | 2 | 3 |
| Slack | 3 | 3 | 4 | 2 | 18.0 | 3 | 3 |

High RICE does not override authority boundaries. GitHub is the highest-value Connector and also the highest-risk core Connector, so its initial tool surface is intentionally constrained.

## Provisioning classifications

### Managed custom MCP

AutoDev reconciler owns registration metadata:

- `autodev_context7`
- `autodev_deepwiki`

### Featured/unmanaged

Mistral owns Connector implementation/auth UX; AutoDev owns only the intended policy mapping:

- `github`
- `linear`

The reconciler emits `EXTERNAL` for these resources and must not attempt CREATE/UPDATE/DELETE.

## Admission gate for a new Connector

A candidate moves into the portfolio only when all are answered:

1. What unique capability does it provide?
2. Which agent roles need it?
3. Which exact tools are required after live discovery?
4. What data leaves AutoDev/Mistral?
5. What external state can it mutate?
6. Which tool calls require confirmation?
7. What credential identity does it use?
8. Can it be tested on disposable resources?
9. How is tool-schema drift detected?
10. What is the rollback/removal plan?

If a candidate duplicates an existing capability without a measurable gain, reject it.