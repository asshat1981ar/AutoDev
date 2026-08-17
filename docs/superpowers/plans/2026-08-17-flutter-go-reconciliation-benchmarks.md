# Flutter + Go Reconciliation and Adoption Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reconcile the Flutter and Go prototypes against Compose/KMP and Rust/Tokio using reproducible UX, performance, security, and maintenance evidence, then make explicit keep/remove decisions.

**Architecture:** This plan changes no trusted execution design. It consumes completed protocol/Flutter/Go prototype evidence, builds only minimal comparison fixtures, runs the full repository verification matrix, and records a reversible adoption decision for each new language boundary.

**Tech Stack:** Existing Rust/Kotlin/Android toolchains, Flutter prototype toolchain, Go prototype toolchain, repository-native benchmark documents and CI artifacts.

## Global Constraints

- No technology is adopted because its prototype merely works.
- A benchmark claim must include toolchain, machine/runner, workload, command, raw result, and interpretation.
- Compare equivalent workloads; do not compare Flutter release mode to Compose debug mode or Go optimized binaries to unoptimized Rust builds.
- Security/authority regression is a veto regardless of UX/performance score.
- Flutter and Go decisions are independent: one may be adopted while the other is removed.
- Existing Compose Android and ForgeCore remain the baseline, not migration targets.

---

## File Structure

- Create `docs/benchmarks/polyglot/flutter-compose-comparison.md`.
- Create `docs/benchmarks/polyglot/go-rust-comparison.md`.
- Create `docs/architecture/flutter-go-adoption-decision.md`.
- Add only minimal benchmark/comparison fixtures needed by the two prototype plans.
- Modify `README.md` only after adoption decisions are made.
- Modify CI ownership/build matrix only for adopted components.

### Task 1: Freeze comparable workloads

- [ ] Record the Flutter workload as exactly 10,000 timeline events, 5,000 graph nodes, fixed seed `20260817`, identical event/graph fixtures and equivalent interaction sequence.
- [ ] Record the Go workload as 1/8/16 concurrent mock MCP sessions, forced disconnect/reconnect, and 100/1,000/10,000 observation events per minute.
- [ ] Record build modes, hardware/runner, OS, and commit SHA before executing comparisons.
- [ ] Reject results where fixture inputs differ materially between language implementations.

### Task 2: Build the minimum Compose comparison slice

**Files:** create a benchmark-only or test-only Compose surface under `kotlin/android-command-center` rather than a second application.

Implement only the same high-value Flutter comparisons:

- virtualized 10,000-event timeline;
- 5,000-node graph canvas with pan/zoom/select;
- objective/evidence navigation with the same canonical public fixtures.

Do not redesign the Android product or port all Flutter Studio UI.

- [ ] Run Android/Compose benchmark or macrobenchmark tooling available in the repository execution environment.
- [ ] If device/emulator performance evidence is unavailable, mark performance comparison incomplete rather than inferring a winner from desktop Flutter data.
- [ ] Record code size/files/dependencies needed for the comparison slice.

### Task 3: Reconcile Flutter evidence

Score qualitatively with evidence references:

| Criterion | Flutter | Compose/KMP | Decision weight |
| --- | --- | --- | --- |
| Dense desktop UX | evidence | evidence | high |
| Graph/timeline rendering | evidence | evidence | high |
| Cross-platform reach | evidence | evidence | medium |
| Startup/memory | evidence | evidence | medium |
| Accessibility/keyboard | evidence | evidence | high |
| Duplicate business logic | evidence | evidence | high negative |
| CI/toolchain cost | evidence | evidence | medium negative |
| Authority safety | pass/fail | pass/fail | veto |

- [ ] Produce one of `ADOPT_STUDIO`, `CONTINUE_PROTOTYPE`, or `REMOVE_FLUTTER`.
- [ ] `ADOPT_STUDIO` requires a demonstrated experience/cross-platform advantage with public protocol isolation preserved.
- [ ] A decision to replace the Android Compose app is explicitly forbidden by this gate; that would require a new Major Change Gate.

### Task 4: Reconcile Go vs Rust/Tokio evidence

Score:

| Criterion | Go Edge | Rust/Tokio fixture | Decision weight |
| --- | --- | --- | --- |
| MCP compatibility/isolation | evidence | evidence | high |
| Connection lifecycle clarity | evidence | evidence | high |
| Backpressure/cancellation | evidence | evidence | high |
| RSS/CPU/latency | evidence | evidence | medium |
| Race/concurrency confidence | evidence | evidence | high |
| Deployment/process complexity | evidence | evidence | high negative |
| Duplicate networking | evidence | evidence | high negative |
| Authority safety | pass/fail | pass/fail | veto |

- [ ] Produce one of `ADOPT_EDGE`, `CONTINUE_PROTOTYPE`, or `REMOVE_GO`.
- [ ] `ADOPT_EDGE` may be justified by strong protocol isolation/maintainability even without a raw throughput win, but the structural benefit must be evidenced and the deployment cost accepted explicitly.

### Task 5: Security and authority review

Verify all of the following with tests/source inspection:

- Flutter carries no approval grant or trusted capability material.
- Flutter cannot call a repository mutation endpoint outside the Rust control plane.
- Go public/local APIs expose no objective execution or approval endpoints.
- Go logs do not contain bearer tokens or upstream secrets.
- Go local HTTP cannot bind non-loopback in prototype mode.
- malformed SSE/MCP/public-protocol input fails without escalating authority.
- ForgeCore remains the only repository effect boundary.

Any failure blocks adoption and returns the affected prototype to repair/reverify.

### Task 6: Run the complete verification matrix

Run existing repository gates plus adopted prototype gates:

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace
cargo test --workspace

cd ../kotlin
./gradlew clean test :mpp-core:assemble :mpp-server:assemble :mpp-ui:assemble :mpp-codegraph:assemble :android-command-center:assembleDebug --no-daemon
./gradlew ktlintCheck --no-daemon

cd ../flutter/autodev_studio
flutter analyze
flutter test
flutter build linux --release
flutter build web --release

cd ../../go/autodev-edge
gofmt -w .
git diff --exit-code
go vet ./...
go test ./... -race
go build ./cmd/autodev-edge
```

If a prototype is already rejected and removed, omit only that prototype's commands and document the removal commit.

### Task 7: Write the adoption decision record

Create `docs/architecture/flutter-go-adoption-decision.md` containing for each language:

- CHANGE
- Reason
- Repository evidence
- Benchmark evidence
- UX/capability benefit
- Affected areas
- Authority/security result
- Complexity cost
- Alternative
- Rollback
- Final recommendation

Also include a final topology diagram that names actual adopted components only.

### Task 8: Apply removal or adoption cleanup

**If Flutter is rejected:** delete `flutter/autodev_studio` and its CI job, preserving benchmark/design records and public protocol contracts.

**If Flutter is adopted:** keep it explicitly as `AutoDev Studio`; update README/build docs and CI. Do not remove Compose Android.

**If Go is rejected:** delete `go/autodev-edge` and its CI job; preserve protocol/benchmark evidence and implement no substitute in this task.

**If Go is adopted:** keep its scope limited to external connectivity/MCP edge; document process ownership, local authentication, and packaging. Do not move ForgeCore policy/execution into Go.

Run the full verification matrix again after cleanup.

### Task 9: Final completion report

Report:

## Implemented
- adopted components only

## Reconciliation
- what was preserved, adopted, removed, or deferred

## Verification
- commands actually executed and CI run identifiers

## Performance
- comparable baselines/results/deltas

## Security / Authority
- explicit pass/fail for each boundary

## Remaining Risks
- evidence-backed risks only

## Next Highest-Value Task
- one task derived from the adopted architecture

## Status
Use `DONE` only after the post-decision cleanup and complete verification matrix pass.

## Done Gate

This plan is complete when Flutter and Go each have an explicit evidence-backed adoption/removal decision, rejected prototype code is removed, adopted code is documented and CI-gated, and the resulting repository still has one trusted ForgeCore authority path.