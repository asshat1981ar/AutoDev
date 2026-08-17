# Flutter + Go Integration Execution Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute the approved Flutter/Go research design in dependency order without allowing either prototype to define authority or public protocol ad hoc.

**Architecture:** Protocol foundation is the serial prerequisite. After it is green, Flutter Studio and Go Edge are independent prototype branches and should be developed in parallel worktrees. Reconciliation/benchmarking runs only after both prototypes have reached their own Done Gates or one has been explicitly abandoned with evidence.

**Tech Stack:** Rust/Axum public protocol, Flutter/Dart Studio prototype, Go/MCP Edge prototype, existing Kotlin/Compose comparison baseline, GitHub Actions.

## Global Constraints

- Start every implementation slice from current `main`, not the archived PR #12 integration branch.
- Use isolated worktrees for implementation.
- Every task follows RED -> GREEN -> REFACTOR -> focused verification -> commit -> review.
- No production adoption decision is made inside a prototype task.
- ForgeCore remains the sole trusted authorization/execution/evidence authority.

---

## Execution Order

- [ ] **Phase 1 — Public protocol foundation**

Execute `docs/superpowers/plans/2026-08-17-flutter-go-protocol-foundation.md` completely. Do not start language-specific protocol clients until its Rust/public fixture Done Gate is green.

- [ ] **Phase 2A — Flutter Studio prototype**

Execute `docs/superpowers/plans/2026-08-17-flutter-autodev-studio-prototype.md` in an isolated worktree based on the protocol-foundation integration commit.

- [ ] **Phase 2B — Go Edge prototype**

Execute `docs/superpowers/plans/2026-08-17-go-autodev-edge-prototype.md` in a separate isolated worktree based on the same protocol-foundation integration commit.

Phases 2A and 2B may run concurrently because they share only the frozen public v1 schemas/fixtures.

- [ ] **Phase 3 — Reconciliation/adoption gate**

Execute `docs/superpowers/plans/2026-08-17-flutter-go-reconciliation-benchmarks.md` only after prototype evidence is available. This phase may delete either prototype if its value case fails.

## Review Gates

After Phase 1, independently review protocol authority leakage and compatibility before merging.

After each Phase 2 prototype, independently review correctness/security first, then benchmark methodology. Do not treat a green build as adoption evidence.

After Phase 3, require one explicit result per language: Flutter = `ADOPT_STUDIO`, `CONTINUE_PROTOTYPE`, or `REMOVE_FLUTTER`; Go = `ADOPT_EDGE`, `CONTINUE_PROTOTYPE`, or `REMOVE_GO`.

## Final Verification

Run the repository verification matrix defined in the reconciliation plan after all adoption/removal cleanup. `DONE` is allowed only when the final topology has one ForgeCore authority path and CI covers every retained production/prototype component.