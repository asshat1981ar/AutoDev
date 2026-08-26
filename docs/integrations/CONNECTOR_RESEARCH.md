# Mistral Connector Research

Date: 2026-08-21

## Scope

This research records the current Mistral Studio Connector contract used by AutoDev's Connector-as-Code control plane. The source of truth is current Mistral documentation and the generated Mistral SDK documentation; MCP server metadata is cross-checked against upstream projects and the MCP Registry.

## Primary sources

- Mistral Connector management: https://docs.mistral.ai/studio/connectors/management
- Mistral Connector overview: https://docs.mistral.ai/studio/connectors
- Connector conversations/tool filtering: https://docs.mistral.ai/studio/connectors/conversations
- Direct tool calling: https://docs.mistral.ai/studio/connectors/tool_calling
- Human confirmation: https://docs.mistral.ai/studio/connectors/confirmation
- Connector workflows/slots: https://docs.mistral.ai/studio/workflows/building-workflows/connectors
- Connector API reference: https://docs.mistral.ai/api/endpoint/beta/connectors
- Context7 upstream MCP: https://github.com/upstash/context7
- DeepWiki MCP endpoint used by Mistral examples: https://mcp.deepwiki.com/mcp

## Current lifecycle

The documented lifecycle is:

1. Optionally validate a server in Mistral's Connector Debugger.
2. Create/register the Connector with MCP server URL and visibility.
3. Establish authentication/credential state. Mistral documents a connection call even when no credentials are needed.
4. Discover/refresh tools.
5. Verify deterministic behavior with direct tool calls.
6. Attach the Connector to conversations, Agents, or Workflows with restricted tool configuration.
7. Update mutable Connector fields.
8. Delete only through an explicit lifecycle action.

AutoDev implements 1-7 as staged capabilities but intentionally omits implicit deletion.

## Management contract

### Create

`POST /v1/connectors`

Required conceptual fields:

- `name`
- `server`
- `visibility`

Current visibility values documented by Studio are:

- `private`
- `shared_workspace`
- `shared_org`

`shared_org` is organization-admin scope and therefore elevated risk.

Connector names are unique per Workspace, limited to 64 characters, and restricted to alphanumeric characters plus `_` and `-`.

Optional create fields include `description`, `icon_url`, `headers`, `auth_data`, and `system_prompt`. AutoDev does not permit raw `headers` or `auth_data` in Git-managed manifests because they can contain credentials.

### Retrieve/list

- `GET /v1/connectors/{connector_id_or_name}`
- `GET /v1/connectors`

The list endpoint uses keyset/cursor pagination. The reconciler follows `pagination.next_cursor` until exhausted.

### Tools

`GET /v1/connectors/{connector_id_or_name}/tools`

Relevant parameters:

- `page`
- `page_size`
- `refresh`
- `pretty`

`refresh=true` re-fetches tools from the MCP server. AutoDev treats refreshed tool schemas as untrusted evidence and compares them against a prior snapshot before any authority change.

### Direct tool call

`POST /v1/connectors/{connector_id_or_name}/tools/{tool_name}/call`

Direct calls bypass model tool selection and are therefore the preferred deterministic Connector contract test once a Connector is authenticated.

### Update

`PATCH /v1/connectors/{connector_id}`

The update API requires the Connector UUID, not its logical name. Current updateable fields are documented as:

- `name`
- `description`
- `server`
- `icon_url`
- `system_prompt`
- `headers`
- `auth_data`

`visibility` is not in the documented update field set. The reconciler therefore blocks visibility drift instead of attempting an unsupported in-place update.

### Delete

`DELETE /v1/connectors/{connector_id}` exists, but this control plane intentionally has no implicit prune/delete path. Removal requires a future explicit command, separate approval, impact analysis, and rollback plan.

## Authentication findings

Mistral's management guide states that newly created Connectors need a credential/connection call even when the server requires no credentials. Its SDK example uses `create_or_update_user_credentials` with empty headers and a default credential.

The generated endpoint documentation currently presents a narrower request example for `POST /v1/connectors/{connector_id_or_name}/user/credentials` that emphasizes the credential name. Because these documentation surfaces are not perfectly aligned, AutoDev does not guess a raw credential payload in the initial reconciler. Credential initialization is isolated as the next contract-tested integration step.

OAuth token injection is explicitly not treated as a background automation primitive. Mistral documents user authorization through Studio/auth URLs; AutoDev must preserve that user-controlled flow.

## Permission and confirmation model

Mistral can restrict Connector tools using `include`/`exclude` tool configuration and can mark selected tools as `requires_confirmation`. Organization and Workspace administration can additionally allow, restrict, or block Connector access.

AutoDev uses four layers:

1. **Connector registration** — what external MCP server exists.
2. **Connector permission policy** — which discovered tools may be exposed.
3. **Agent role policy** — which agents may receive which Connector capabilities.
4. **ForgeCore authority** — whether an AutoDev source/process effect is executable.

The first three never substitute for the fourth.

## Workflow Connector slots

Mistral Workflows support Connector slots with identity-aware credential resolution. Important current properties:

- credentials can resolve as the triggering user or deployment identity;
- OAuth can pause and resume workflows;
- a slot can select a named credential;
- Durable Agents require a single `run_as` identity across all Connectors attached to that Agent;
- mixed per-Connector identities should use explicit tool-call workflows rather than one Durable Agent.

This makes Connector slots a strong later integration target, but they remain Public Preview and should stay behind an adapter.

## MCP candidates validated during research

### Context7

Upstream currently publishes a remote streamable-HTTP endpoint:

`https://mcp.context7.com/mcp`

An API key is optional for basic public usage and can be supplied through an Authorization header for higher limits/private capabilities. The AutoDev manifest starts without a credential so no secret enters Git.

### DeepWiki

Mistral's own Connector examples use:

`https://mcp.deepwiki.com/mcp`

It is useful for repository-level semantic exploration and complements Context7's library/version documentation.

## Public Preview risk

Mistral labels Connectors and workflow Connector integration as Public Preview. Therefore AutoDev:

- keeps REST interaction inside one adapter;
- pins desired-state schema version independently from Mistral's SDK;
- captures tool schemas before permission changes;
- fails closed on undocumented state transitions;
- does not let remote tool discovery automatically expand allowlists;
- records documentation/API drift as a migration event.

## Research decisions

1. Use Linear, not both Linear and Jira, as the initial Agile backlog hypothesis to avoid duplicate issue authority.
2. Represent Mistral-featured GitHub and Linear Connectors as external/unmanaged resources rather than trying to recreate them as custom MCP servers.
3. Manage Context7 and DeepWiki as custom MCP Connector desired state.
4. Keep GitHub write/release effects outside Mistral Connector authority for now; repository/process execution continues through ForgeCore.
5. Treat credential provisioning, exact tool allowlists, and live Agent attachment as separate approval-gated phases after live tool discovery.