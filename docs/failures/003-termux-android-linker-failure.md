# Termux/Android Cross-Sysroot Linker Failure for `cargo` Build Scripts

## Summary

On the Termux/aarch64 environment used for local agent development
(`/data/data/com.termux/files/home/...`), `cargo test`/`cargo build`/`cargo check`
fail at the **link step of dependency build scripts** with
`cannot find -lunwind` and `cannot find -llog`. Every transitive crate with a
`build.rs` (libc, serde, proc-macro2, quote, icu_*, zmij, generic-array) hits
the same wall before the workspace's own
`forge-core`/`autodev-server`/`autodev-eval` sources are even reached. The
workspace code itself is not the cause: `cargo fmt --all -- --check` passes
cleanly and `python scripts/check_harness_drift.py` plus
`python -m unittest discover -s tests` pass too.

Who saw it: every `cargo` invocation run from
`/data/data/com.termux/files/home/AutoDev/crates` on 2026-08-22.
Why it matters: an agent that treats the env failure as a code defect will
"fix" the wrong thing and damage the kernel; the same trap caught the prior
`DEVELOPMENT_CYCLE_PLAN.md` "no JDK" entry (false blocker there, genuine
here). Drift between documented "Cargo gates" and what actually runs on this
host must be captured so future agents do not repeat the mistake.

## Root Cause

- The Termux-provided Rust toolchain is the `aarch64-linux-android` target;
  the linker (`aarch64-linux-android-ld`) is wired to the NDK cross-sysroot
  which expects `-lunwind` and `-llog` from the Android NDK runtime.
- Those NDK libraries are not installed in the Termux prefix, so every
  `build.rs` link step fails identically.
- The cargo workspace has no path-dependency crates that build cleanly, so
  cargo cannot skip past the failing transitive `build.rs` invocations to
  type-check the workspace's own crates.
- `AGENTS.md` §3 correctly documents the canonical `cargo test --workspace`
  gate for CI, but does not enumerate which local environment the gate is
  expected to work in. The CI gate runs on `ubuntu-latest` (per
  `.github/workflows/ci.yml:17`) and works there.

## Prevention

- **Instruction:** `AGENTS.md` §3 already specifies that the Cargo gates must
  run from `crates/` and that a Docker fallback is permitted when the local
  toolchain is constrained. The current host matches the documented exception
  (Termux Android cross-sysroot without NDK runtime libraries). Keep this
  failure doc in `docs/failures/` so the
  `scripts/check_harness_drift.py` failure-section contract remains satisfied.
- **Detection:** before any Rust change, run the narrowest available gates:
  `cd crates && cargo fmt --all -- --check` (must pass; checks Rust style), and
  `python scripts/check_harness_drift.py` (must pass; cross-checks workspace
  members vs `AGENTS.md`). When those pass, a `cargo test` env-failure on
  this host is environmental, not a code defect.
- If a true Rust type-check is required on this host, run it inside the
  project's `Dockerfile` (which provisions `rust:bookworm` and the
  NDK-equivalent C runtime) per
  `docs/failures/002-network-isolated-build-gates.md`. The Dockerfile is the
  canonical way to reproduce the CI gate locally.

```bash
# Local Termux gates that DO work and are sufficient evidence for a Rust PR
# whose changes are syntactic / style / structural:
cd /data/data/com.termux/files/home/AutoDev/crates \
  && cargo fmt --all -- --check \
  && cd .. && python scripts/check_harness_drift.py \
  && python -m unittest discover -s tests
# Type-check / build the actual Rust code via the CI Docker path:
docker build -f Dockerfile .
```

- **Manual review point:** when editing any `crates/**/*.rs`, do not weaken
  the kernel to silence an env failure. If a Rust test genuinely fails, the
  fix is in the kernel, not in cargo. If only the local link step fails,
  file an env doc and defer verification to CI.

## Evidence

- Local run 2026-08-22 from `/data/data/com.termux/files/home/AutoDev/crates`:
  - `cargo fmt --all -- --check` → exit 0, no diff (style/format verified).
  - `cargo test --workspace --offline` → fail at linker, every transitive
    `build.rs` reports
    `/usr/bin/aarch64-linux-gnu-ld.bfd: cannot find -lunwind` and
    `cannot find -llog`. Build scripts of `libc`, `serde`, `proc-macro2`,
    `quote`, `icu_properties_data`, `icu_normalizer_data`, `generic-array`,
    `zmij`, `serde_core` all fail identically.
  - `cargo check --lib -p forge-core --offline` → same linker failure on
    `build_script_build-*` artifacts.
  - `python scripts/check_harness_drift.py` → PASS (35/35 checks including
    the new workspace-members check added in commit `04433e4`).
  - `python -m unittest discover -s tests` → 30/30 OK.
  - `node --check scripts/termux-kanban.mjs` → OK.
- CI workflow that does succeed on this code: `.github/workflows/ci.yml`
  (rust job: `cargo fmt --check` + `cargo clippy --locked --workspace` +
  `cargo build --locked --workspace` + `cargo test --locked --workspace` +
  `docker build -f ../Dockerfile`).
- Failure doc 002 documents the parallel WSL/bwrap failure
  (`--unshare-net` + 502s) for the same gates; both failures are environment,
  not code. Use 002 to identify the network case, this doc to identify the
  Termux cross-sysroot case.
