# Connector Architecture

## Purpose

AutoDev's Mistral Connector layer is a control plane, not an alternate execution plane. Its job is to register external MCP capability, constrain tool exposure, resolve credential identity through Mistral, and provide deterministic evidence for tool discovery and integration tests.

## Architecture

```text
Git desired state
  connectors/*.yaml
  policies/mistral-connectors/*.yaml
          |
          v
Manifest validation
          |
          v
Desired/remote normalization
          |
          v
Deterministic reconciler
  CREATE | UPDATE | NOOP | EXTERNAL | BLOCKED
          |
          +-------- dry-run evidence --------+
          |                                  |
          v                                  v
Mistral Connector API                  tool snapshots
          |                                  |
          v                                  v
Registered MCP Connector <---- drift comparator
          |
          v
Mistral tool configuration
 include / exclude / confirmation
          |
          v
Agent / Workflow
          |
          v
Typed AutoDev intent for source/process effects
          |
          v
ForgeCore policy + AuthorizationGrant
          |
          v
Execution -> evidence -> verification
```

## Boundaries

### Desired state

Git contains only non-secret Connector metadata and policy intent. Credentials are referenced by identity/purpose in policy documentation but raw values never enter the repository.

### Reconciler

`scripts/mistral_connector_sync.py` owns:

- manifest validation;
- secret-like field rejection;
- remote Connector discovery;
- deterministic planning;
- safe create/update serialization;
- tool discovery;
- tool drift comparison;
- dry-run/live-plan CLI behavior.

It deliberately lacks an implicit delete path.

### Mistral platform

Mistral owns:

- MCP transport to registered Connector servers;
- user/workspace/org credential storage;
- OAuth interaction;
- Connector tool discovery;
- tool selection in conversations/Agents;
- workflow Connector slots and identity resolution.

### ForgeCore

ForgeCore remains the only trusted AutoDev authority for repository/process side effects. A Mistral Connector's ability to expose a write tool does not create or imply an AutoDev authorization grant.

## Desired-state action semantics

| Action | Meaning | Mutation allowed |
|---|---|---:|
| `CREATE` | Managed MCP Connector is absent | Yes, explicit apply only |
| `UPDATE` | Supported mutable fields drift | Yes, explicit apply only |
| `NOOP` | Desired and remote comparable state match | No |
| `EXTERNAL` | Featured/unmanaged Connector | No |
| `BLOCKED` | Unsafe/unsupported transition | No |

Visibility drift is `BLOCKED` because Mistral's current documented update API does not list visibility as mutable.

## Authentication architecture

Credentials are separated from Connector registration.

- `none`: still requires Mistral's documented connection/credential initialization before tool use.
- bearer/header: secret comes from Mistral credential storage or a secure runtime source, never the manifest.
- OAuth2: use Mistral's interactive authorization flow; do not inject arbitrary tokens.
- workflow OBO: triggering-user identity where a hardened deployment is required.
- deployment identity: worker/service credentials where interactive OAuth is unavailable.

Durable Agents require a single Connector `run_as` identity. If one task needs mixed identities, prefer an explicit Workflow/ToolCallClient boundary instead of hiding the mix inside an Agent.

## Tool policy compiler target

Current policy files express capability intent. After live tool discovery, a future compiler should generate exact Mistral tool configuration:

```json
{
  "type": "connector",
  "connector_id": "<logical-or-uuid>",
  "tool_configuration": {
    "include": ["<approved-tool>"],
    "requires_confirmation": ["<approved-write-tool>"]
  }
}
```

Rules:

- no discovered tool is automatically allowed;
- `include` is preferred over broad exposure when stable tool names exist;
- unknown tools fail closed;
- permission changes require review;
- tool schema changes invalidate previous approval until re-evaluated.

## Failure domains

Failures are classified by boundary:

1. manifest/schema
2. reconciliation logic
3. Mistral API/auth
4. MCP transport
5. tool discovery/schema
6. tool invocation
7. model tool routing
8. external service
9. AutoDev authority/execution

Diagnosis must identify the failing layer before a fix is attempted.

## Idempotency

The intended invariant is:

```text
apply(desired, remote)
-> refresh remote
-> plan(desired, refreshed remote) == NOOP
```

If the second plan is not `NOOP`, reconciliation is not considered successful.

## Deletion and rollback

Absence from Git never means delete. A future delete command must resolve exact UUID, list dependent Agents/Workflows, present impact and rollback strategy, and require explicit approval.

For code rollback, revert the feature commits. For a newly created Mistral Connector, rollback is a separately approved remote deletion only after confirming no dependent Agent/Workflow still references it.