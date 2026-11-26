# ADR-006: LiveKit as an Untrusted Realtime Intake Edge

- **Status:** Accepted
- **Date:** 2026-08-27
- **Deciders:** AutoDev maintainers
- **Related:** `ADR-001-forgecore-execution.md`, `ADR-003-distributed-workers.md`

## Context

AutoDev needs an optional realtime voice/video interface without moving execution authority out of ForgeCore. LiveKit Agents JS provides room participation, speech pipelines, worker lifecycle, dispatch, graceful drain, and optional avatar plugins. LemonSlice `1.7.0` requires the matching `@livekit/agents@1.7.0` and `@livekit/rtc-node@^0.13.34`.

An earlier experiment added Node and Rust MCP servers that called LiveKit administrative APIs directly with ambient API credentials. That design is rejected: an MCP/model call is untrusted intent and cannot authorize room creation, token minting, agent dispatch, participant removal, or room deletion. An environment flag is not an `AuthorizationGrant`.

## Decision

Adopt a root pnpm workspace solely for bounded Node services and add `services/livekit-agent` as an **untrusted realtime intake adapter**.

The worker may:

1. join a LiveKit room through the official Agents JS worker lifecycle;
2. converse using operator-selected LiveKit Inference STT, LLM, and TTS models;
3. optionally start a LemonSlice avatar using the official `AvatarSession` lifecycle;
4. enqueue a bounded objective for one operator-configured repository through `POST /api/v1/objectives`.

The worker may not:

- import ForgeCore or construct an `AuthorizationGrant`;
- execute shell, filesystem, Git, deployment, or LiveKit administrative effects;
- accept a repository or control-plane URL from the model or room participant;
- expose LiveKit, LemonSlice, or AutoDev bearer secrets to the model;
- claim that enqueueing means approval, execution, verification, merge, or deployment.

AutoDev adds an optional `AUTODEV_API_BEARER_TOKEN` for mutating API routes. The worker defaults to the loopback AutoDev origin. A remote origin requires all of: `AUTODEV_ALLOW_REMOTE=1`, HTTPS, and `AUTODEV_API_BEARER_TOKEN`.

`LIVEKIT_AGENT_NAME` is required so the worker uses explicit dispatch through LiveKit's trusted control plane. AutoDev does not expose an administrative dispatch MCP tool.

## Package-management exception

This ADR deliberately adopts these root files:

- `package.json`
- `pnpm-lock.yaml`
- `pnpm-workspace.yaml`

They exist only to coordinate `services/*`. Node 24 and pnpm 11.24.0 are pinned in documentation and CI. Packages use exact LiveKit versions where peer compatibility is strict. Native dependency build scripts are allowlisted explicitly in `pnpm-workspace.yaml`.

## Consequences

### Positive

- Realtime voice/video becomes an optional edge without weakening ForgeCore.
- LiveKit worker scaling and graceful drain remain provider-managed concerns.
- LemonSlice is isolated behind an optional configuration boundary.
- Objective intake can be protected independently of the read-only API.
- Unit tests require no LiveKit, model-provider, or LemonSlice credentials.

### Negative

- A supported root Node workspace and additional CI job are required.
- A credentialed media smoke test cannot run in public CI without secrets.
- Enqueued objectives remain in the control plane's current in-memory store.

## Rejected alternatives

- **LiveKit administrative MCP server:** rejected because direct network effects bypass typed policy and trusted grants.
- **Giving the voice worker ForgeCore authority:** rejected because room media and model output are untrusted.
- **Model-selected repository or endpoint:** rejected because it enables confused-deputy and SSRF attacks.
- **Replacing AutoDev orchestration with LiveKit jobs:** rejected because LiveKit jobs are realtime session lifecycle, not AutoDev execution authority or evidence state.

## Verification

- `pnpm install --frozen-lockfile`
- `pnpm typecheck`
- `pnpm test`
- `docker build -f services/livekit-agent/Dockerfile -t autodev-livekit-agent:ci .`
- `cd crates && cargo test --workspace`
- `python scripts/check_harness_drift.py`

Credentialed smoke testing is documented separately and must use disposable rooms and scoped secrets.