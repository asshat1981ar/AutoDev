---
applyTo: "crates/**"
---

# Rust — ForgeCore instructions

This guidance applies only to files under `crates/**`.

## Stack

- Workspace `crates/Cargo.toml` → members `forge-core` (lib + tests) + `autodev-server` (Axum + rmcp)
- Edition 2021, `resolver = "2"`, MIT license
- Dependencies: `serde`, `serde_json`, `thiserror`, `sha2`, `chrono`, `ureq` (forge-core); `axum`, `tokio`, `hmac`, `rmcp 3.1.2` (server)

## Commands (run from `crates/`)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace
```

Container gate: `docker build -f ../Dockerfile -t autodev-server:ci ..`

## Rules

- Do not create a root `Cargo.toml`. The workspace lives at `crates/Cargo.toml` — CI runs with `working-directory: crates`.
- Never add direct fs/net/process execution that bypasses `Workspace` confinement or `AuthorizationGrant`. Public adapters are fail-closed.
- `ExecutionEnvelope.evidence.required` is closed-world: unknown names fail the task. Update `src/verification.rs` if adding a new evidence kind.
- Adversarial tests live in `forge-core/tests/` (`adversarial_*`). Add a new case for every workspace/policy change.
- Keep `crates/Cargo.lock` committed; update via `cargo update` only. Revert collateral diffs.
- Do not edit `target/` or generated artifacts.

## Verification

Changes under `crates/**` must pass `cargo fmt --check` + `clippy -D warnings` + `cargo test --workspace` before PR. Run `python scripts/check_harness_drift.py` to confirm harness alignment.
