# LiveKit Agent Deployment

`services/livekit-agent` is AutoDev's optional realtime voice/video intake edge. It joins LiveKit rooms, clarifies requests, and can enqueue a bounded objective. It has no ForgeCore, filesystem, Git, shell, approval, deployment, or LiveKit administrative authority.

## Required configuration

Copy `services/livekit-agent/.env.example` to a secret-managed deployment environment; never commit populated values.

| Variable | Requirement |
|---|---|
| `LIVEKIT_URL` | LiveKit WebSocket URL used by Agents JS |
| `LIVEKIT_API_KEY` / `LIVEKIT_API_SECRET` | Worker credentials injected at runtime |
| `LIVEKIT_AGENT_NAME` | Required explicit-dispatch worker name |
| `AUTODEV_REPOSITORY` | Fixed repository for every objective queued by this worker |
| `AUTODEV_LIVEKIT_STT_MODEL` | LiveKit Inference STT model string |
| `AUTODEV_LIVEKIT_LLM_MODEL` | LiveKit Inference LLM model string |
| `AUTODEV_LIVEKIT_TTS_MODEL` | LiveKit Inference TTS model and voice string |

`AUTODEV_URL` defaults to `http://127.0.0.1:8080`. A non-loopback value is rejected unless it uses HTTPS, `AUTODEV_ALLOW_REMOTE=1`, and `AUTODEV_API_BEARER_TOKEN`.

Set `AUTODEV_API_BEARER_TOKEN` to the same high-entropy value configured on `autodev-server`. The worker sends it only in the Authorization header. Do not put secrets in URLs, room metadata, prompts, or objective text.

## Optional LemonSlice avatar

Set `LEMONSLICE_API_KEY` and exactly one of:

- `LEMONSLICE_AGENT_ID`
- `LEMONSLICE_AGENT_IMAGE_URL`

The image URL must use HTTPS and must not contain embedded credentials. Local image paths are not
accepted because the realtime edge has no general filesystem-read authority.

The worker follows the official 1.7.0 order: connect to the room, create the `AgentSession`, start the LemonSlice `AvatarSession`, then start the voice session. Avatar cleanup is registered with the LiveKit job shutdown lifecycle. If no LemonSlice configuration is present, the worker runs as an audio/text agent.

## Local verification

```bash
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
```

Build the production image from the repository root:

```bash
docker build -f services/livekit-agent/Dockerfile -t autodev-livekit-agent:local .
```

Run the worker in development mode after loading secrets:

```bash
pnpm --filter @autodev/livekit-agent build
pnpm --filter @autodev/livekit-agent dev
```

Use `start` rather than `dev` in production so Agents JS enables production worker behavior and graceful drain.

## Dispatch

`LIVEKIT_AGENT_NAME` is required. The worker therefore registers for explicit dispatch and does not automatically join every new room. Dispatch is performed through the operator's LiveKit control plane; AutoDev does not provide an MCP tool that holds LiveKit administrator credentials or dispatches agents.

## Credentialed smoke test

1. Start `autodev-server` on loopback with `AUTODEV_API_BEARER_TOKEN` configured.
2. Start the worker with a disposable LiveKit project/room and a test repository.
3. Join from LiveKit Agent Console or a supported frontend.
4. Ask the agent to clarify a harmless objective, then explicitly ask it to queue it.
5. Confirm the voice response reports only `queued` status.
6. Verify `GET /api/v1/objectives` contains exactly one record for `AUTODEV_REPOSITORY`.
7. Confirm no secret appears in transcripts, logs, or the objective body.
8. Send SIGTERM and confirm the worker drains without accepting new jobs.

The smoke test is not a substitute for ForgeCore verification. It proves realtime connectivity and bounded intake only.

## Rollback

Stop the LiveKit worker deployment and remove dispatch routing to its `LIVEKIT_AGENT_NAME`. AutoDev's Rust, Kotlin, Python, Cline, and local-first execution paths continue without it. No database migration or ForgeCore state rollback is required.