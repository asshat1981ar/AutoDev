# Network-Isolated Build Gates — Cargo/Gradle Offline Failure in Sandbox

## Summary
Cargo `build`/`test`/`clippy` and Gradle `check` fail in the local WSL sandbox, not in CI. Cargo hits `CONNECT tunnel failed, response 502` for `crates.io` (even `chrono` download) and offline mode has no cached registry (`no matching package named chrono`). Gradle daemon fails `Could not connect to the Gradle daemon` (`InetAddresses` debug) due to sandbox `--unshare-net` proxy-only network and read-only `gradle-8.10.2-bin.zip.lck`. Pytest `test_cli_*` fails `HTTP 502 upstream connection failed: Connection refused` because `autodev-server` not running. No code defect; environment isolation.

Who saw it: verification gate run 2026-08-18T07:3x from fresh process (`cargo fmt --check` PASS, others env-fail). Why it matters: `ExecutionEnvelope.evidence.required` fails closed if build/test never runs; stale local green hides required evidence.

## Root Cause
- Sandbox is bwrap with `--unshare-net` + `--ro-bind /home/dev/AutoDev/.git` + proxy-only `http://127.0.0.1:36355`. Crates.io and Gradle distributions require egress.
- Minimal Rust install (profile `minimal`) has no registry cache; offline build cannot resolve `forge-core` deps.
- Gradle wrapper `ExclusiveFileAccessManager` needs writable `~/.gradle/wrapper/dists/...lck` — read-only mount fails without `require_escalated`.

## Prevention
- **Instruction:** `AGENTS.md` §3 already lists canonical gates with working dirs `crates/` and `kotlin/`; add note “network required for Rust/Kotlin; use Docker or `--offline` with cached deps” — captured here.
- **Detection:** `scripts/check_harness_drift.py` PASS (checks docs↔CI fragment alignment). For this failure, run gates via Docker as CI does:

```bash
docker build -f Dockerfile .  # as in .github/workflows/ci.yml rust job
docker run --rm autodev cargo test --workspace --offline  # with cached layers
cd kotlin && ./gradlew --no-daemon check  # outside sandbox or with --offline if deps cached
python -m pytest tests/ -k "not test_cli"  # unit fabric only, no server
```

Manual review point: if changing deps, update `crates/Cargo.lock` and `kotlin/gradle/wrapper/gradle-wrapper.properties` together; verify `cargo build --offline` inside Docker before merge.

## Evidence
- `cargo fmt --all -- --check` → `FMT_EXIT:0`
- `cargo build --workspace` → `error: failed to get chrono … download of config.json failed [7] Could not connect 502 BUILD_EXIT:101` (online + offline both 101)
- `cargo test/clippy --offline` → same 101
- `./gradlew --version` → `Gradle 8.10.2` ok after `mkdir -p ~/.gradle/wrapper/dists` with escalation
- `./gradlew check` / `--no-daemon check` → `FAILURE: Could not connect to the Gradle daemon … InetAddresses … GRADLE_CHECK_EXIT:1` (sandbox net)
- `python -m pytest tests/ -v` → `8 passed, 2 failed (test_cli HTTP 502) PYTEST_EXIT:1`, `py_compile` 0, `node --check` 0, `check_harness_drift` PASS
- CI workflow: `.github/workflows/ci.yml` (rust/kotlin/python jobs), `Dockerfile` (rust:bookworm base)
