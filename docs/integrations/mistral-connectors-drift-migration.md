# Mistral Connector Drift and Migration Procedure

## Drift classes

### Configuration drift

Remote Connector metadata differs from reviewed Git desired state.

Action:

1. run `live-plan`;
2. classify `CREATE`, `UPDATE`, `NOOP`, `EXTERNAL`, or `BLOCKED`;
3. never mutate while investigating a `BLOCKED` result;
4. require review for any remote update.

### Tool-schema drift

The MCP server exposes added, removed, or changed tools.

Action:

1. refresh tools;
2. sanitize snapshot;
3. compare with prior snapshot;
4. deny added/changed tools by default;
5. re-run capability mapping and negative permission tests;
6. update policy and snapshot together.

### Authentication drift

Credentials, OAuth scopes, or identity behavior changes.

Action:

1. determine whether failure is user, workspace, organization, or deployment identity;
2. inspect current Mistral authentication method documentation;
3. rotate/re-authorize outside Git;
4. rerun safe direct-call tests.

### API/SDK drift

Mistral Public Preview endpoints or payloads change.

Action:

1. compare current Studio management docs and API reference;
2. inspect current generated SDK signatures;
3. add a failing contract test for the new behavior;
4. modify only the Mistral adapter boundary;
5. run full CI and dry-run reconciliation;
6. update this research record and any affected manifests.

## Scheduled review checklist

For every planned Mistral upgrade and periodically while Public Preview remains active:

- Connector create/update/delete endpoint shapes;
- visibility values and sharing behavior;
- credential request models;
- OAuth flows;
- list-tools pagination and refresh semantics;
- direct tool call contract;
- tool filtering/confirmation schema;
- Agent Connector configuration;
- Workflow Connector slots and `run_as` rules;
- API key Connector scopes;
- featured Connector names/capabilities;
- deprecations and migration notices.

## Migration rule

Never migrate remote state merely because documentation changed. First prove the old assumption is no longer valid with current docs/API or a reproducible integration failure, then encode the new contract in tests.

## Connector replacement

When replacing one Connector with another:

1. inventory all Agents/Workflows referencing the old Connector;
2. create/test the replacement in private scope;
3. capture and classify replacement tools;
4. build restricted policy;
5. run representative Agent routing tests;
6. switch consumers in a bounded batch;
7. verify old Connector has no consumers;
8. request separate deletion approval.

Do not run two canonical backlog or repository-management Connectors indefinitely unless there is an explicit synchronization/ownership model.

## Known 2026 migration consideration

Mistral's Connector surface is evolving toward MCP-backed Connectors and workflow Connector slots. Legacy knowledge-specific integration paths should not be hard-coded into AutoDev. The control plane therefore models MCP server endpoints and featured Connector identities rather than embedding provider-specific transport logic.

## Drift evidence

Store only sanitized evidence in Git:

- Connector metadata required for comparison;
- tool name/description/input schema;
- documentation/API version notes;
- test/run identifiers;
- policy diff.

Never store bearer tokens, OAuth tokens, API keys, raw authorization headers, or upstream secrets in drift artifacts.