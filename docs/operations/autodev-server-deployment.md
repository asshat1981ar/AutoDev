# AutoDev Server Deployment Contract

The `autodev-server` binary (`crates/autodev-server`) is a stateless control-plane adapter. It does not execute code, write files, or merge PRs — those actions require the kernel-owned `AuthorizationGrant` flow inside `forge-core`. What the server *does* do is:

- accept GitHub webhook traffic on `/webhooks/github` and verify the HMAC-SHA256 `X-Hub-Signature-256`;
- expose read-only objectives + a bounded objective enqueue on `/api/v1/*`;
- serve the read-only MCP transport on `/mcp` behind a bearer-token middleware.

This file documents the deployment contract the server expects. A production deployment that ignores any of these is unsupported.

## Network interface

| Variable | Default | Effect |
|---|---|---|
| `AUTODEV_PORT` | `8080` | TCP port |
| `AUTODEV_BIND` | `0.0.0.0` | Network interface. Set to `127.0.0.1` for local-only operation |

The default `0.0.0.0` keeps the public webhook contract intact (GitHub must reach the listener) **and** exposes the bearer-token-protected `/mcp` route to the LAN. The server logs a warning at startup if `AUTODEV_BIND` is the unspecified address and `AUTODEV_MCP_BEARER_TOKEN` is set, so the operator can see the LAN-bearer exposure in their own logs.

## TLS is the operator's responsibility

`autodev-server` listens for plaintext HTTP. The webhook secret and the MCP bearer token are both secrets in headers; the server provides no `rustls` / `axum-server` integration. **Production deployments are expected to terminate TLS at a reverse proxy** (nginx, Caddy, Cloudflare, an ingress controller, etc.) and forward to `autodev-server` on the loopback interface.

If you do not have a TLS terminator in front of the server, you are transmitting the `GITHUB_WEBHOOK_SECRET` and the `AUTODEV_MCP_BEARER_TOKEN` in cleartext on every request. That is not a supported configuration.

## MCP bearer token

`/mcp` is bearer-token-gated by `AUTODEV_MCP_BEARER_TOKEN`. If the variable is unset, the route returns `503 Service Unavailable` and no MCP traffic is admitted. The token must be a high-entropy random value (≥ 32 bytes) generated and stored as a secret; do not commit it.

The MCP transport also enforces a `Host`-header allowlist (`AUTODEV_MCP_ALLOWED_HOSTS`, default `localhost,127.0.0.1,::1,autodev-server`) and an `Origin` allowlist (`AUTODEV_MCP_ALLOWED_ORIGINS`, default `http://localhost,http://127.0.0.1,http://localhost:8080`). A stolen bearer is therefore insufficient on its own: the attacker must also reach the loopback interface **and** present an allowed `Host`/`Origin`.

## Webhook secret

`GITHUB_WEBHOOK_SECRET` is required for the `/webhooks/github` route to admit any traffic. If the variable is unset, the route returns `503 Service UnAVAILABLE` and no webhook is processed. The HMAC comparison uses the constant-time `hmac::Mac::verify_slice`; there is no length-leak, no string compare, and no oracle.

## Body-size limit

Both the public API surface (`/api/v1/*`, `/events`, `/webhooks/*`) and the MCP surface cap request bodies at 512 KiB. The default Axum 2 MiB limit is intentionally lowered to bound memory pressure on the unauthenticated routes. A request that exceeds the cap is rejected with `413 Payload Too Large`; the body is never read into memory.

## What the server does not do

- No filesystem, no Git, no subprocess execution authority. The trust boundary for those operations is `forge-core::execute` + `AuthorizationGrant`, which the server does not import.
- No persistent storage beyond an in-process `BTreeMap` of objectives; the server is stateless from the protocol's perspective.
- No mutation of agent policy, capabilities, or evidence. The `objective_queued` event the server emits is an observation, not an instruction.

See `crates/autodev-server/src/main.rs` and `crates/autodev-server/src/lib.rs` for the source of truth on these contracts.
