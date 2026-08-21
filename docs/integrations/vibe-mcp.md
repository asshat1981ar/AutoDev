# Mistral Vibe MCP Integration

AutoDev exposes its development control plane over authenticated MCP Streamable HTTP at `/mcp`. The adapter exposes observation and proposal tools only; ForgeCore remains the sole trusted execution authority.

## 1. Start AutoDev MCP locally

From the repository root:

```console
cd crates
export AUTODEV_MCP_BEARER_TOKEN='replace-me-with-a-long-random-token'
cargo run --locked -p autodev-server
```

The server binds to `127.0.0.1:8080` by default. `AUTODEV_PORT` changes the port. `AUTODEV_BIND_ADDR` may explicitly widen the bind address when a deployment requires it; do not set it to `0.0.0.0` for a local-only Vibe workflow.

The `/mcp` route fails closed when `AUTODEV_MCP_BEARER_TOKEN` is absent and rejects incorrect bearer tokens. Browser-origin requests are additionally checked against the configured MCP host policy.

## 2. Register AutoDev with Vibe

The first positional argument after `vibe mcp add` is required. Use `autodev` as the server `NAME`:

```console
vibe mcp add autodev \
  --transport streamable-http \
  --url http://127.0.0.1:8080/mcp \
  --api-key-env AUTODEV_MCP_BEARER_TOKEN \
  --api-key-header Authorization \
  --api-key-format "Bearer {token}" \
  --no-login \
  --startup-timeout-sec 10 \
  --tool-timeout-sec 120
```

The error below means the required positional alias was omitted:

```text
vibe mcp add: error: the following arguments are required: NAME
```

Correct shape:

```text
vibe mcp add NAME [options]
```

For AutoDev, `NAME` is `autodev`.

## 3. Exposed development tools

The Vibe-oriented slice provides:

- `autodev.project.status` — read the objective/ExecPlan control-plane projection and authority boundary.
- `autodev.execplan.get` — read one typed durable ForgeCore ExecPlan by ID.
- `autodev.verification.status` — read a deliberately non-authoritative verification projection; it never self-verifies work.
- `autodev.action.propose` — create a typed candidate write action without executing it.
- `autodev.test.propose` — propose a bounded verification command without starting a process.
- `autodev.replan.propose` — propose a goal change without mutating the plan or consuming replan budget.

Existing `autodev.objectives.list` and `autodev.gaps.scan` tools remain available.

## 4. Authority model

The intended path is:

```text
Vibe
  -> MCP Streamable HTTP adapter
  -> typed observation or proposal
  -> ForgeCore policy / authorization boundary
  -> workspace-confined execution when separately authorized
  -> evidence collection and VerificationFabric
```

MCP does not mint `AuthorizationGrant`, run arbitrary proposals directly, widen capabilities, or mark its own work verified.

## 5. Security configuration

Default local MCP hosts are `localhost`, `127.0.0.1`, `::1`, and `autodev-server`. `AUTODEV_MCP_ALLOWED_HOSTS` accepts a comma-separated replacement allowlist for deployments that deliberately use other hostnames. The same hostname policy is applied to present browser `Origin` headers to mitigate DNS-rebinding attacks.

Keep bearer material in environment variables rather than command history or repository files. Do not commit the token.

## 6. Quick troubleshooting

- `NAME` required: add `autodev` immediately after `vibe mcp add`.
- `503` from `/mcp`: set `AUTODEV_MCP_BEARER_TOKEN` on the AutoDev server process.
- `401` from `/mcp`: Vibe is sending a missing or incorrect bearer token; verify the same environment variable is exported in the Vibe environment.
- `403` with `untrusted MCP Origin`: the browser Origin hostname is outside the MCP allowlist.
- Connection refused: confirm AutoDev is running on `127.0.0.1:8080`, or update `--url` to match the deliberately configured bind address and port.
