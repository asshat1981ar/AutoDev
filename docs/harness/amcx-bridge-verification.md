# AMCX Bridge Verification Status

The AMCX ↔ ForgeCore bridge is intentionally projection-only and must not authorize or execute effects.

## Local Rust evidence

The ChatGPT sandbox used the user-provided Rust 1.97.1 toolchain. A focused RED/GREEN proxy harness was executed because direct GitHub DNS was unavailable in the sandbox:

- RED: compilation failed with `E0583` while `amcx_bridge` was absent.
- GREEN: five focused tests passed after implementing validation/projection behavior.

This proxy evidence is useful but is not a substitute for repository-level Cargo verification.

## Required remote gate

Before merge, GitHub Actions or another repository-mounted Rust environment must execute:

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p forge-core --test amcx_bridge --locked
cargo test --workspace --locked
cd ..
python scripts/check_harness_drift.py
```

Any failure blocks merge and must be reconciled against `docs/architecture/amcx/reconciliation-v1.1.md`.
