# Codex Subscription Provider Design

## Status

Approved 2026-08-17 as an addendum to Working APK v0 Architecture A.

## Goal

Allow AutoDev to use an eligible ChatGPT subscription, including ChatGPT Plus, through OpenAI Codex's supported managed ChatGPT authentication path without treating subscription credentials as ordinary OpenAI API keys.

## Architecture

```text
Android APK
  -> AutoDev Rust control plane
  -> ForgeCore provider/action-proposal seam
  -> CodexSubscriptionProvider
  -> `codex app-server` over stdio JSONL
  -> ChatGPT managed OAuth / device-code authentication
```

`codex app-server` is the credential owner. AutoDev never reads, copies, serializes, logs, or sends ChatGPT access/refresh tokens to Android.

## Official protocol boundary

AutoDev must use the documented app-server protocol rather than scraping browser sessions or reusing internal browser cookies.

Required app-server operations:

- connection handshake: `initialize` followed by `initialized`;
- browser subscription login: `account/login/start` with `type: "chatgpt"`;
- headless/Termux login: `account/login/start` with `type: "chatgptDeviceCode"`;
- account state: `account/read`;
- logout: `account/logout`;
- subscription usage state: `account/rateLimits/read`;
- login completion/account changes are consumed from app-server notifications.

AutoDev identifies itself through `initialize.params.clientInfo`.

## Trust boundary

Authentication proves access to a Codex subscription service. It does not authorize repository effects.

Codex output is untrusted model intent. The provider may return text or a typed action proposal, but ForgeCore remains the only authority for:

- capability checks;
- risk checks;
- approval requirements;
- workspace confinement;
- execution;
- evidence;
- independent verification.

The Codex-backed proposal path must run with a read-only/no-escalation Codex configuration. AutoDev must never delegate repository mutation to `codex app-server` merely because the user authenticated successfully.

## API/product surface

The control plane may expose safe account metadata to Android:

- provider availability;
- authentication state;
- auth mode;
- plan type when app-server reports it;
- browser login URL or device-code verification URL/user code;
- normalized rate-limit/reset data;
- recoverable provider errors.

The control plane must not expose:

- access tokens;
- refresh tokens;
- cookies;
- Authorization headers;
- Codex credential-store paths/content.

## Failure behavior

If `codex` is absent, app-server cannot initialize, authentication expires, or protocol messages are malformed, AutoDev reports a provider-unavailable/authentication error. It must not silently fall back to browser cookie extraction, another user's credentials, or API-key billing.

## Testing

CI never requires a live ChatGPT subscription. The app-server client is defined behind a request/notification transport seam and tested with deterministic fake JSON-RPC messages.

Production smoke testing may additionally verify that an installed `codex` binary can initialize app-server, but live user login remains an explicit user action.

## Working APK v0 sequencing

1. PRO-66 adds the provider-neutral typed-action proposal seam and trusted objective runner.
2. PRO-72 adds the Codex app-server subscription adapter and safe account/rate-limit surface.
3. PRO-71 adds Android provider/account UX alongside objective lifecycle UX.
4. PRO-70 verifies the APK install/cold-launch/end-to-end smoke path.

## Non-goals

- cloning OpenAI's OAuth implementation;
- using ChatGPT Plus as an `OPENAI_API_KEY` replacement;
- embedding subscription credentials in the APK;
- browser session scraping;
- bypassing ForgeCore approvals;
- making live ChatGPT subscription login a CI requirement.
