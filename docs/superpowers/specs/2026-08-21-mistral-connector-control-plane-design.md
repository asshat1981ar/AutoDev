# Mistral Connector Control Plane Design

## Goal

Add a Git-authoritative, least-privilege control plane for Mistral Studio MCP Connectors that can validate desired connector manifests, discover remote state, compute safe reconciliation plans, inspect exposed MCP tools, and apply approved non-destructive changes without granting AutoDev agents direct execution authority.

## Context and constraints

AutoDev is local-first and model-agnostic. ForgeCore remains the sole trusted execution authority for workspace/process effects. This integration manages Mistral-side Connector configuration only; it does not create `AuthorizationGrant`, execute repository changes, mark work verified, or turn Connector access into ForgeCore authority.

The repository has no root Python package manifest and its Python fabric is stdlib-only. The first slice therefore uses Python 3.10/3.11 standard library only and talks to the documented Mistral REST API through an injectable HTTP transport. No `mistralai` dependency is introduced.

## Documented Mistral lifecycle represented by the design

The Connector lifecycle modeled here is:

1. validate/debug an MCP endpoint externally;
2. create a Connector from name, server, description, visibility, and optional non-secret fields;
3. re-read Connector metadata and prove desired-state reconciliation returns `NOOP`;
4. establish a credential record even for no-auth connectors when required by Mistral;
5. list/refresh tools after the Connector is authenticated;
6. use direct tool calls for deterministic integration tests;
7. attach restricted tools to agents/workflows outside this first slice;
8. update changed connectors and re-verify remote state;
9. delete only by a separate explicit, approval-gated operation.

Current Mistral management constraints captured by validation:

- connector names are unique in a Workspace, max 64 characters, and use alphanumeric characters, `_`, or `-`;
- visibility is one of `private`, `shared_workspace`, or `shared_org`;
- `shared_org` is treated as elevated risk;
- updates require remote UUID resolution even when desired state uses a logical name;
- OAuth token injection is not automated;
- tool discovery can be refreshed and should be snapshotted before policy changes.

## Source of truth

Desired state is stored under `connectors/` as strict JSON documents with `.yaml` filenames. JSON is a valid YAML subset, which preserves YAML interoperability while allowing dependency-free `json` parsing in AutoDev.

`connectors/registry.yaml` contains the Connector registry. Individual connector files contain concrete desired state. Secret values are never stored; manifests may contain secret reference names only.

Example:

```yaml
{
  "schema_version": 1,
  "key": "deepwiki",
  "name": "autodev_deepwiki",
  "kind": "mcp",
  "managed": true,
  "server": "https://mcp.deepwiki.com/mcp",
  "visibility": "private",
  "description": "Read-only repository intelligence for AutoDev research agents",
  "tool_policy": {
    "include": [],
    "exclude": []
  },
  "confirmation": {
    "required": []
  },
  "risk": "read_only"
}
```

Featured Mistral Connectors may be represented as `kind: featured, managed: false`; the reconciler must never attempt to create/update/delete them.

## Components

### `scripts/mistral_connector_sync.py`

Dependency-free CLI and library with these boundaries:

- manifest and registry loading/validation;
- normalization of desired and remote Connector state;
- deterministic diff planning;
- Mistral REST adapter over an injectable transport;
- read-only remote discovery;
- safe create/update application when explicitly requested;
- mandatory post-mutation remote re-read and `NOOP` verification;
- tool discovery and direct-call primitives for later authenticated integration tests;
- tool-schema drift comparison;
- secret redaction;
- no implicit delete/prune.

### `connectors/*.yaml`

Declarative desired state. The initial portfolio contains managed, private, read-only custom MCP Connectors for Context7 and DeepWiki plus featured/unmanaged GitHub and Linear resources. The policy model therefore distinguishes lifecycle resources AutoDev owns from integrations Mistral manages.

### `policies/mistral-connectors/*.yaml`

Deny-by-default capability intent. Unknown discovered tools remain denied; GitHub source/release mutation is forbidden from the initial Connector layer; permitted Linear mutations require confirmation.

### `tests/test_mistral_connector_sync.py`

Behavioral tests cover manifest/registry validation, create/update/no-op planning, refusal to manage featured connectors, refusal to delete implicitly, elevated visibility checks, HTTP serialization, direct tool calls, post-apply verification, tool drift detection, policy defaults, and redaction.

## Reconciliation algorithm

For each desired connector:

1. validate local schema and policy;
2. resolve remote Connector by name from paginated list results;
3. normalize comparable fields;
4. emit exactly one action: `CREATE`, `UPDATE`, `NOOP`, `EXTERNAL`, or `BLOCKED`;
5. never infer `DELETE` from absence;
6. for `--apply`, execute only `CREATE` and `UPDATE` actions;
7. block `shared_org` writes unless `--allow-org-shared` is present;
8. after mutation, re-read remote Connector state;
9. calculate reconciliation again and fail if the second result is not `NOOP`;
10. after the separate credential/connect step, refresh tools and produce sanitized tool evidence before any Agent policy activation.

The reconciler is idempotent: applying desired state against an unchanged remote result yields `NOOP` on the next plan.

## Tool drift

Tool authority must not expand silently. A tool snapshot stores only sanitized tool metadata and input schemas. Comparing a new discovery to a prior snapshot reports added, removed, and changed tool names. Newly added or changed tools are evidence, not automatically included in agent allowlists.

## Authentication

`MISTRAL_API_KEY` is read from the environment only when a live API call is requested. It is never accepted as a manifest field or CLI argument and is never logged.

No-auth Connector credential initialization and OAuth flows are deliberately separated from ordinary reconciliation:

- no-auth credential initialization may be added after its exact API payload is contract-tested against the current documentation/SDK surface;
- OAuth requires an interactive/user authorization flow and is never silently automated;
- raw OAuth tokens are never persisted by this control plane.

The separation is intentional because registration metadata can be verified before external account identity is attached.

## Safety and authority

- Default CLI operation is validation/planning; mutation requires the explicit `apply` command and `--apply` flag.
- No Connector deletion exists in this slice.
- No organization-shared mutation without an explicit elevated flag.
- No secret fields in manifests.
- No automatic agent attachment in this slice.
- No repository/process execution is exposed to Mistral through this control plane.
- Remote/tool payloads are untrusted data and are never interpreted as executable instructions.
- Direct tool calling exists as a library primitive for deterministic contract testing, not as an unrestricted generic CLI mutation surface.

## Verification

TDD sequence:

1. commit tests that fail because the module or required behavior does not exist;
2. observe GitHub Actions fail for the intended missing behavior;
3. add the minimum implementation;
4. require Python tests plus harness drift to pass at exact PR head;
5. inspect the PR diff and independent review evidence before any completion claim.

Regression RED/GREEN cycles must be recorded when verification exposes a missing safety invariant.

Live Mistral mutation is out of scope until a later explicit approval provides a securely configured `MISTRAL_API_KEY`, identifies the target Workspace, and approves the exact Connector diff.

## Rollback

The first slice is isolated to new `scripts/`, `tests/`, `connectors/`, `policies/`, `snapshots/`, and documentation files. Code rollback removes/reverts those additions; it changes neither ForgeCore nor existing Vibe MCP transport behavior.

A live Connector rollback is a separate operational action: reverse supported metadata updates from captured prior state, or request explicit approval before deleting a newly created Connector after dependency inspection.