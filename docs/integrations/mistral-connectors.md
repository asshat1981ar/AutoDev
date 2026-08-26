# Mistral Studio Connector Control Plane

AutoDev manages custom Mistral Studio MCP Connector desired state from Git without giving Mistral agents ForgeCore execution authority.

## Scope

This first slice supports:

- dependency-free connector manifest validation;
- deterministic CREATE / UPDATE / NOOP / EXTERNAL / BLOCKED planning;
- live connector discovery through the Mistral REST API;
- safe CREATE and UPDATE application after explicit CLI opt-in;
- tool discovery and refresh;
- tool-schema drift comparison;
- secret redaction;
- fail-closed handling for `shared_org` changes;
- explicit modeling of featured Mistral connectors as external/unmanaged.

It intentionally does **not** support implicit deletion, OAuth token injection, production agent attachment, or automatic expansion of tool allowlists.

## Manifest format

Files under `connectors/` use strict JSON syntax with a `.yaml` extension. JSON is a valid YAML subset, so the files remain interoperable with YAML tooling while AutoDev can parse them with Python's standard-library `json` module.

`connectors/deepwiki.yaml` is the initial managed MCP example. `connectors/github.yaml` and `connectors/linear.yaml` demonstrate featured Mistral connectors that are represented for policy/routing purposes but are not lifecycle-managed by this reconciler.

Raw secrets are forbidden. A future credential integration may use secret-reference fields, but credential values must remain outside Git.

## Validate desired state

```bash
python scripts/mistral_connector_sync.py validate connectors/deepwiki.yaml
```

Validation checks the documented Mistral connector name and visibility constraints, requires HTTPS for managed MCP endpoints, rejects featured connectors marked as managed, and rejects secret-like manifest fields.

## Deterministic offline plan

Plan against no existing remote connector:

```bash
python scripts/mistral_connector_sync.py plan connectors/deepwiki.yaml
```

Plan against a captured remote-state JSON object:

```bash
python scripts/mistral_connector_sync.py plan connectors/deepwiki.yaml --remote-file remote.json
```

This mode makes no network request.

## Live plan

Set the Mistral API key only in the process environment:

```bash
export MISTRAL_API_KEY='...'
python scripts/mistral_connector_sync.py live-plan connectors/deepwiki.yaml
```

The key is not accepted as a CLI argument and is never written into manifests or output.

## Apply

Mutation requires the dedicated command and explicit `--apply` switch:

```bash
python scripts/mistral_connector_sync.py apply connectors/deepwiki.yaml --apply
```

Only CREATE and UPDATE plans can mutate. NOOP returns without a request. EXTERNAL and BLOCKED plans refuse mutation. No DELETE path exists.

Organization-shared connector changes fail closed unless the operator also supplies:

```bash
--allow-org-shared
```

That flag is an operator acknowledgement, not an AutoDev `AuthorizationGrant` and not ForgeCore authority.

## Tool discovery

List cached tools:

```bash
python scripts/mistral_connector_sync.py tools autodev_deepwiki
```

Refresh directly from the MCP server through Mistral:

```bash
python scripts/mistral_connector_sync.py tools autodev_deepwiki --refresh
```

Tool output is untrusted data. Newly discovered tools are evidence only; discovery never adds them to an agent allowlist.

## Tool drift

Given two sanitized tool snapshots:

```bash
python scripts/mistral_connector_sync.py diff-tools old-tools.json new-tools.json
```

The result reports sorted `added`, `removed`, and `changed` tool names. Authority expansion requires a separate reviewed policy change.

## Mistral API lifecycle represented

The implementation follows the current Studio Connector management lifecycle:

1. create an MCP Connector from name, server, visibility and description;
2. authenticate/connect separately when required;
3. list or refresh tools;
4. direct-call or attach tools to agents/workflows separately;
5. update changed connector fields;
6. delete only through a future explicit approval-gated operation.

The current code deliberately excludes OAuth and credential mutation because Mistral's documented OAuth flow requires interactive user authorization and does not support arbitrary token injection.

## Authority boundary

Mistral Connector access does not confer AutoDev execution authority. Connector configuration and MCP tool access remain outside ForgeCore's trusted `AuthorizationGrant` boundary. Any future Connector that can write repositories, trigger CI, deploy software, or mutate shared systems must be mapped through separate role/tool policy and human-confirmation controls before agent attachment.