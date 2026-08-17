# PRO-72 Codex Subscription Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ChatGPT subscription authentication to AutoDev through the supported `codex app-server` managed OAuth/device-code path while keeping all repository authority inside ForgeCore.

**Architecture:** `CodexSubscriptionClient` speaks the app-server JSONL protocol through a small transport trait. A production stdio transport launches `codex app-server`; tests use a deterministic fake transport. The client exposes safe account/login/rate-limit metadata only. Later, the PRO-66 typed-action proposal seam consumes Codex output as untrusted intent and ForgeCore performs authorization/execution/verification.

**Tech Stack:** Rust, serde/serde_json, std::process/std::io, existing ForgeCore model/runtime contracts, GitHub Actions.

## Global Constraints

- `codex app-server` owns ChatGPT OAuth tokens and refresh.
- Never serialize access tokens, refresh tokens, cookies, Authorization headers, or credential-store contents into AutoDev state/events/API responses.
- Subscription auth is not an OpenAI API key and must not silently switch to API-key billing.
- Use app-server stdio JSONL for production; websocket mode is not a production dependency.
- Browser flow uses `type: "chatgpt"`; headless/Termux uses `type: "chatgptDeviceCode"`.
- AutoDev must initialize app-server with `clientInfo` before any account/thread method.
- Codex may propose intent; ForgeCore remains the sole authority for mutation, approval, evidence, and verification.
- CI uses fake transport and never requires a live ChatGPT account.
- TDD: every production behavior starts with a failing focused test.

---

### Task 1: Define safe subscription client contracts

**Files:**
- Test: `crates/forge-core/tests/codex_subscription.rs`
- Create after RED: `crates/forge-core/src/codex_subscription.rs`
- Modify after RED: `crates/forge-core/src/lib.rs`

**Produces:**

```rust
pub trait CodexRpcTransport {
    fn request(&mut self, method: &str, params: serde_json::Value)
        -> Result<serde_json::Value, CodexSubscriptionError>;
    fn notify(&mut self, method: &str, params: serde_json::Value)
        -> Result<(), CodexSubscriptionError>;
}

pub struct CodexSubscriptionClient<T: CodexRpcTransport> { /* transport */ }

pub enum CodexLoginStart {
    Browser { login_id: String, auth_url: String },
    DeviceCode { login_id: String, verification_url: String, user_code: String },
}

pub struct CodexAccount {
    pub authenticated: bool,
    pub auth_mode: Option<String>,
    pub plan_type: Option<String>,
}
```

- [ ] Write RED tests proving browser login sends `account/login/start` with only `{type:"chatgpt"}`, device-code sends only `{type:"chatgptDeviceCode"}`, and account parsing surfaces `planType` without any token field.
- [ ] Push RED commit and confirm GitHub Actions fails because the subscription module does not exist.
- [ ] Implement the minimal contracts/client methods and export them.
- [ ] Confirm focused tests and workspace CI are green.

### Task 2: Enforce initialization handshake

**Produces:**

```rust
pub fn initialize(&mut self, version: &str) -> Result<CodexServerInfo, CodexSubscriptionError>;
```

Initialization sends `initialize` with:

```json
{
  "clientInfo": {
    "name": "autodev",
    "title": "AutoDev",
    "version": "<version>"
  }
}
```

and then sends the `initialized` notification.

- [ ] RED test records transport calls and fails until ordering is `initialize` request then `initialized` notification.
- [ ] GREEN implementation rejects account/login calls before initialization.
- [ ] Regression test verifies repeated initialize is rejected locally.

### Task 3: Add safe account and rate-limit reads

**Produces:**

```rust
pub fn account(&mut self) -> Result<CodexAccount, CodexSubscriptionError>;
pub fn rate_limits(&mut self) -> Result<CodexRateLimits, CodexSubscriptionError>;
pub fn logout(&mut self) -> Result<(), CodexSubscriptionError>;
```

- [ ] RED tests for `account/read`, `account/rateLimits/read`, `account/logout`.
- [ ] Add a serialization test that converts all public subscription DTOs to JSON and proves the resulting JSON contains no keys named `access_token`, `accessToken`, `refresh_token`, `refreshToken`, `authorization`, or `cookie`.
- [ ] GREEN implementation keeps raw protocol responses private and explicitly maps only allow-listed fields into public DTOs.

### Task 4: Implement stdio JSONL app-server transport

**Files:**
- Create: `crates/forge-core/src/codex_app_server.rs`
- Modify: `crates/forge-core/src/lib.rs`

**Produces:**

```rust
pub struct StdioCodexTransport { /* child/stdin/stdout/request id */ }

impl StdioCodexTransport {
    pub fn spawn(codex_binary: impl AsRef<std::path::Path>)
        -> Result<Self, CodexSubscriptionError>;
}
```

Behavior:
- spawn `codex app-server --listen stdio://`;
- one JSON object per stdin line;
- monotonically increasing request IDs;
- ignore/queue notifications while waiting for matching response ID;
- map process exit, malformed JSON, protocol error and EOF into typed errors;
- never read credential files directly.

- [ ] RED unit tests exercise line encoder/response matching using in-memory reader/writer helpers.
- [ ] GREEN implementation adds production process spawn only after protocol helpers pass.
- [ ] Test missing executable produces a clear provider-unavailable error.

### Task 5: Add login completion notification handling

**Produces:**

```rust
pub enum CodexNotification {
    LoginCompleted { login_id: String, success: bool, error: Option<String> },
    AccountUpdated { auth_mode: Option<String>, plan_type: Option<String> },
    RateLimitsUpdated(CodexRateLimits),
    Other { method: String },
}

pub fn next_notification(&mut self) -> Result<CodexNotification, CodexSubscriptionError>;
```

- [ ] RED tests parse `account/login/completed`, `account/updated`, `account/rateLimits/updated`.
- [ ] GREEN parser drops unknown/private fields and never emits token material.

### Task 6: Connect Codex to the PRO-66 action-proposal seam

**Depends on:** PRO-66 `ActionProposal`/`propose_action` contract.

- [ ] RED test: Codex-backed proposer returns one typed `AgentAction` and cannot execute it.
- [ ] Start an ephemeral/read-only Codex thread configured so writes cannot execute through app-server.
- [ ] Request exactly one structured action proposal for the current bounded task/context.
- [ ] Parse output through the same ForgeCore structured-action validation used by other providers.
- [ ] Pass resulting action into existing ForgeCore policy/orchestration; never use Codex tool execution as the trusted effect path.

### Task 7: Expose safe control-plane subscription endpoints

**Depends on:** PRO-66 server baseline.

Add safe endpoints such as:
- `GET /api/v1/providers/codex/account`
- `POST /api/v1/providers/codex/login` with `{mode:"browser"|"device_code"}` only
- `POST /api/v1/providers/codex/logout`
- `GET /api/v1/providers/codex/rate-limits`

- [ ] RED API tests prove unknown credential/token fields are rejected or ignored and are never echoed.
- [ ] GREEN handlers map only the safe subscription DTOs.
- [ ] SSE/provider events may report auth/plan/limit state but never secrets.

### Task 8: Android provider/account UX

**Owned with PRO-71.**

- [ ] Show Codex provider availability and authenticated plan.
- [ ] Browser mode opens the returned auth URL through Android's browser intent.
- [ ] Device-code mode displays verification URL + user code.
- [ ] Show rate-limit/reset metadata when available.
- [ ] Do not persist OAuth credentials in Android storage.

## Verification

Before marking PRO-72 complete:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

GitHub Actions must be green on the final implementation head. A live Plus login is a manual/product smoke test, not a CI requirement.
