# AutoDev Web Command Center

The Web Command Center is a dependency-free browser client for AutoDev's existing objective HTTP and SSE control-plane APIs.

## Authority model

The browser can:

- list `/api/v1/objectives`;
- enqueue a bounded objective with `POST /api/v1/objectives`;
- observe `/events` through Server-Sent Events.

It does **not** call `/mcp`, ForgeCore execution, Git/process execution, approval, credential, or policy endpoints. Enqueueing an objective creates control-plane intent only; ForgeCore remains the trusted authorization and execution boundary.

## Deployment

Serve these static files from the **same origin** as the AutoDev server, or place both behind a reverse proxy that presents one origin. AutoDev does not enable permissive cross-origin resource sharing by default, so opening `index.html` from `file://` or an unrelated web origin is not a supported production deployment.

A typical reverse-proxy layout is:

```text
https://autodev.example/                 -> web/command-center/index.html
https://autodev.example/app.js           -> web/command-center/app.js
https://autodev.example/styles.css       -> web/command-center/styles.css
https://autodev.example/api/v1/*         -> autodev-server:8080/api/v1/*
https://autodev.example/events           -> autodev-server:8080/events
```

The server input defaults to `http://127.0.0.1:8080` for local development and can be changed in the UI. Embedded URL credentials are rejected.

## CLI companion

The repository also includes a stdlib-only Python CLI over the same control-plane surface:

```bash
python scripts/autodev-cli.py objectives list
python scripts/autodev-cli.py objectives create \
  --repository owner/repo \
  --description "Implement a bounded vertical slice"
python scripts/autodev-cli.py events
```

Use `--server https://autodev.example` to target another deployment. The CLI intentionally has no MCP, approval, or direct execution commands.
