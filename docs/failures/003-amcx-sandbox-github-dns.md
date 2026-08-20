# AMCX sandbox GitHub DNS limitation

## Summary
The ChatGPT execution sandbox successfully installed the user-provided Rust 1.97.1 toolchain but could not clone `github.com` because DNS resolution is unavailable in that sandbox. This prevented full workspace Cargo verification directly inside the sandbox after the AMCX bridge implementation.

## Root Cause
The limitation is environmental, not a Rust/toolchain failure: `git clone https://github.com/asshat1981ar/AutoDev.git` fails with `Could not resolve host: github.com`. GitHub repository access remains available through the connected GitHub integration, but that connector is not a shell filesystem mount.

## Prevention
For future agentic development requiring local compilation, prefer one of these evidence-producing paths:

1. a pre-mounted repository/worktree in the execution sandbox;
2. a connected command runner exposing a repository shell;
3. GitHub Actions on an isolated feature branch/draft PR.

Do not treat connector reads as equivalent to compiler/test execution.

## Detection
The AMCX bridge workflow records a local Rust RED/GREEN proxy gate and requires GitHub Actions/full repository verification before merge. Repository CI remains the authoritative integration gate when the sandbox cannot resolve GitHub.

## Evidence
- Rust installed: `rustc 1.97.1 (8bab26f4f 2026-07-14)` and `cargo 1.97.1 (c980f4866 2026-06-30)`.
- RED proxy: `E0583` because `amcx_bridge` did not yet exist.
- GREEN proxy: 5 focused projection tests passed.
- Clone failure: `Could not resolve host: github.com`.
