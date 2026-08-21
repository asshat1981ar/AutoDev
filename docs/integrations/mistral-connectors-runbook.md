# Mistral Connector Operations Runbook

## Preconditions

Before a live operation:

- exact PR head is green;
- desired Connector manifest is reviewed;
- `MISTRAL_API_KEY` exists only in the operator environment/secret store;
- the target Mistral Workspace is identified;
- the proposed remote diff is recorded;
- any credential or external write capability has explicit approval;
- a rollback path is known.

## Validate

```bash
python scripts/mistral_connector_sync.py validate connectors/registry.yaml
python scripts/mistral_connector_sync.py validate connectors/context7.yaml
python scripts/mistral_connector_sync.py validate connectors/deepwiki.yaml
```

Validation is offline and does not require credentials.

## Offline plan

```bash
python scripts/mistral_connector_sync.py plan connectors/context7.yaml
python scripts/mistral_connector_sync.py plan connectors/deepwiki.yaml
```

With captured remote state:

```bash
python scripts/mistral_connector_sync.py plan connectors/context7.yaml --remote-file remote-context7.json
```

## Live plan

```bash
export MISTRAL_API_KEY='...'
python scripts/mistral_connector_sync.py live-plan connectors/context7.yaml
python scripts/mistral_connector_sync.py live-plan connectors/deepwiki.yaml
```

`live-plan` reads remote state only. Do not paste the key into shell history on shared systems; use the environment/secret mechanism appropriate to the execution environment.

## Apply registration/update

Only after reviewing the exact live plan:

```bash
python scripts/mistral_connector_sync.py apply connectors/context7.yaml --apply
```

The first implementation permits only supported CREATE/UPDATE actions. It refuses EXTERNAL/BLOCKED states and has no delete command.

`shared_org` state additionally requires the explicit elevated switch, but that switch is only an operator acknowledgement; it is not sufficient approval by itself.

## Credential/connect step

Registration and authentication are separate.

For no-auth MCP servers, use Mistral's documented credential/connect initialization after the exact API contract is verified against the current SDK/docs. For OAuth, complete Mistral's user authorization flow. Do not inject OAuth tokens into Git-managed files.

The initial reconciler deliberately does not guess across the current documentation differences in the user-credential request model.

## Discover and snapshot tools

After the Connector is usable:

```bash
python scripts/mistral_connector_sync.py tools autodev_context7 --refresh > /tmp/context7-tools.json
python scripts/mistral_connector_sync.py tools autodev_deepwiki --refresh > /tmp/deepwiki-tools.json
```

Review output before copying sanitized schema evidence into `snapshots/mistral-connectors/`.

If a previous snapshot exists:

```bash
python scripts/mistral_connector_sync.py diff-tools \
  snapshots/mistral-connectors/context7.tools.json \
  /tmp/context7-tools.json
```

Any added or changed tool is denied until mapped and reviewed.

## Permission activation

Use `policies/mistral-connectors/permissions.yaml` and `confirmation.yaml` as capability intent. Convert them to exact Mistral `include` and `requires_confirmation` entries only after tool discovery.

Run negative tests before Agent attachment:

- excluded write tool unavailable;
- unknown tool unavailable;
- invalid arguments fail;
- unauthenticated call fails safely;
- confirmation-gated mutation cannot execute without approval.

## Agent attachment

Attach only the restricted Connector tool configuration approved for that Agent. Do not expose the full GitHub Connector to a coding/release Agent merely because the Connector contains useful read operations.

GitHub repository/process mutations remain behind ForgeCore.

## Incident diagnosis

Classify the failing boundary before changing anything:

```text
manifest
-> reconciler
-> Mistral API
-> credential/auth
-> MCP transport
-> tool discovery
-> tool invocation
-> model routing
-> external service
-> AutoDev ForgeCore authority/execution
```

For a failure:

1. capture exact error/status;
2. reproduce;
3. identify the failing boundary;
4. compare against a working example/current docs;
5. state one root-cause hypothesis;
6. test one variable;
7. apply the minimal fix;
8. add regression coverage.

## Rollback

### Code/config rollback

Revert the feature commit(s) and rerun offline validation/CI.

### Connector metadata rollback

If an UPDATE changed a supported field, use the previously captured sanitized remote state to construct a reviewed reverse UPDATE.

### Newly created Connector

Deletion is intentionally not automated by the current reconciler. If a newly created Connector must be removed:

1. identify its exact UUID;
2. verify no Agent/Workflow references it;
3. capture current tools/metadata;
4. present deletion impact and recovery plan;
5. obtain explicit approval;
6. delete through Mistral Studio/API manually or through a future dedicated approval-gated command;
7. verify absence.

### Credential compromise

Revoke/rotate the credential in the upstream service and Mistral credential store. Do not attempt to repair compromise by editing Git manifests.

## Completion evidence

A live Connector operation is complete only when:

- remote re-read matches desired supported fields;
- a second reconciliation plan is `NOOP`;
- authentication works;
- tool refresh succeeds;
- direct safe read call succeeds;
- tool snapshot is sanitized;
- negative permission tests pass;
- attached Agent routing tests pass where applicable.