# Model Fabric

This document describes ForgeCore's **provider-neutral model abstraction** and the
initial Ollama provider.

## Goal

The orchestration layer must talk to models through a neutral interface so it never
depends on a specific provider. Ollama-specific behavior lives entirely inside
`OllamaProvider`; orchestration only sees the `ModelProvider` trait and the neutral
request/response types.

## Types (`crates/forge-core/src/model.rs`)

| Type | Purpose |
| --- | --- |
| `ModelProvider` | The trait orchestration depends on: `list_models`, `generate`, `chat`, `health`. |
| `Model` | A model known to a provider (id, name, size, capabilities). |
| `ModelCapabilities` | Flags: chat, completion, embeddings, tools, vision. |
| `ModelRequest` | Model id + `messages` (chat) or `prompt` (completion) + `ModelOptions`. |
| `ModelResponse` | Content, token `Usage`, provider timing, provider name. |
| `ModelHealth` | `Healthy`, `Degraded`, `Unavailable`. |
| `RoutingPolicy` | The routing factors in priority order. |
| `RoutingFactor` | Capability, Context, Latency, Availability, Privacy, Cost, LocalRemote, TaskType. |
| `OllamaProvider` | Concrete provider backed by a local Ollama server. |
| `MockProvider` | Deterministic mock for tests (no network). |

## Providers

### OllamaProvider
Talks to the Ollama HTTP API over a configurable base URL (default
`http://localhost:11434`):

- `GET /api/tags` — list models.
- `POST /api/generate` — completion (`{model, prompt, options, stream:false}` →
  `{response, done, total_duration, eval_count, ...}`).
- `POST /api/chat` — chat (`{model, messages, stream:false}` →
  `{message:{content}, done, ...}`).
- `GET /api/ps` — health probe.

All wire details (JSON shapes, nanosecond durations, capability flag names) are
confined to this type and its private helpers. Requests are sent with an argv/JSON
body via `ureq` (blocking, 30s timeout) — no shell.

### MockProvider
Returns canned responses with fixed content and usage, and always reports healthy.
No network access. This lets orchestration be tested without an actual model.

## Routing

`RoutingPolicy` declares which factors matter and in what priority order. `route()`
scores candidate `Model`s against a requested `ModelCapabilities` set and returns them
best-first, dropping models that lack required capabilities. The intentionally simple
scoring is a deterministic starting point that will be extended as providers begin
reporting latency/cost/availability.

Factors that routing will eventually consider: **capability**, **context**, **latency**,
**availability**, **privacy**, **cost**, **local/remote status**, **task type**.

## Provider neutrality

Orchestration depends only on `&dyn ModelProvider` and the neutral types. Swapping
Ollama for another provider — or for `MockProvider` in tests — requires no change to
orchestration. This is proven by the integration tests, which route through
`&dyn ModelProvider`.

## Tests

Coverage in `crates/forge-core/src/model.rs` (unit) and `crates/forge-core/tests/model.rs`
(integration):

- mock provider generates/chats with no network
- routing selects models that support requested capabilities
- routing prefers smaller models on cost
- routing drops models lacking capabilities
- routing policy supports() capability checks
- orchestration uses the `ModelProvider` trait (via mock)
- mock lists a model and routes
- Ollama provider constructs without a connection (health reports Unavailable offline)