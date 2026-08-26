# Mistral Connector Security Review

Date: 2026-08-21

## Threat model

The Connector layer introduces external MCP servers and external SaaS capabilities into an agentic development system. Primary threat classes are:

1. credential disclosure;
2. malicious or compromised MCP servers;
3. tool-schema drift that silently expands authority;
4. prompt/tool descriptions influencing model behavior;
5. confused-deputy use of a high-privilege Connector by the wrong Agent;
6. bypass of AutoDev's ForgeCore execution boundary;
7. destructive external writes without confirmation;
8. shared-Workspace/organization scope accidentally replacing private scope;
9. stale Connector configuration after Public Preview API changes;
10. log/snapshot leakage of authentication material.

## Controls implemented

### Secret exclusion

Git manifests reject secret-like key names recursively, including `Authorization`, `api_key`, `X-Api-Key`, password, secret, and token variants. Secret-reference suffixes (`*_ref`, `*_reference`) are reserved for future non-secret indirection.

The live Mistral API key is read from `MISTRAL_API_KEY` only and is not accepted as a CLI argument or emitted by the sanitizer.

### Least privilege

- Connector desired state defaults to `private`.
- Context7 and DeepWiki are modeled read-only.
- Featured GitHub and Linear Connectors are external/unmanaged.
- GitHub write/merge/release/deploy capabilities are explicitly forbidden by initial policy.
- Linear mutations are confirmation-gated capability intent.
- Unknown discovered tools default to deny.

### Unsupported transition handling

Visibility drift is blocked instead of coerced because the current documented update endpoint does not list visibility as mutable.

No implicit delete/prune path exists.

### Tool drift

Tool schemas are refreshed and snapshotted separately from permissions. Added/changed tools do not automatically enter an allowlist. This prevents an MCP server update from silently granting new model capabilities.

### Separation of authority

A Mistral Connector is not an AutoDev execution authorization mechanism. Source/process effects must still pass ForgeCore policy and `AuthorizationGrant` requirements. Direct GitHub write operations are therefore excluded from Mistral agent policy in the initial deployment.

## Residual risks

### Public Preview API drift

Mistral Connectors and workflow integration are Public Preview. The REST adapter may need migration after API changes. Mitigation: isolate Mistral behavior in one module, contract-test serialization, and re-check docs before live changes.

### MCP server trust

A remote MCP server can change behavior while keeping a stable tool schema. Tool snapshot comparison does not prove server integrity. Mitigation: prefer reputable/open upstream projects, keep read-only scope where possible, monitor upstream/release changes, and sandbox direct calls.

### Model routing

A correct allowlist does not guarantee the model selects the best allowed tool. Mitigation: direct-call contract tests first; routing evaluation only after deterministic Connector behavior passes.

### Featured Connector implementation

Featured connectors are maintained by Mistral and can evolve independently of AutoDev. Mitigation: treat them as external, inspect current tools/permissions before use, and keep the same deny-by-default mapping.

### Credential identity confusion

Workflow OBO versus deployment identity can change which external account is acting. Mitigation: record `run_as` deliberately, never silently substitute service/user identity, and split mixed-identity work into explicit workflow/tool-call boundaries.

## Security gates before live provisioning

Before CREATE/UPDATE or credential setup:

- review exact desired/remote diff;
- confirm target Workspace and visibility;
- verify server hostname and upstream publisher;
- confirm credential mode;
- verify no secret appears in Git diff;
- run unit/CI gates;
- define rollback.

Before attaching a Connector to an Agent:

- refresh tools;
- classify every tool;
- create exact include list;
- create confirmation list;
- prove denied tools unavailable;
- prove read tools work;
- review identity (`run_as`/credential scope);
- verify ForgeCore boundary remains intact.

## Current verdict

**CONDITIONAL / safe for dry-run and code review.**

The implementation is intentionally not authorized for live production/shared Connector mutation in this development pass. Live CREATE/credential/activation/Agent-attachment operations require a separate approval with exact remote diff, permissions, credentials identity, test evidence, and rollback procedure.