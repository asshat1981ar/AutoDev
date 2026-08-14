# Development Driver — Task Graph & Analysis

**Date:** 2026-08-09  | **Driver:** continuous development

## Verified current state (not assumed)
- 129 tests pass, clippy clean, tree clean, in sync with origin/main.
- Implemented: typed protocol, policy, workspace confinement, read_file,
  write_file, git workspace (read/mutate/destructive tiers), model fabric
  (Ollama + mock + routing), agent registry, agent runtime, execution evidence,
  SDLC orchestrator, patch engine.
- **Dispatch gap:** `execute()` handled only `ReadFile`, `WriteFile`, `Git`.
  `PatchFile`, `Mcp`, `RunTest`, `Execute` were declared but returned
  `UnsupportedAction`.

## Gap analysis
1. **Bottleneck** — the execute-dispatch gap: declared action types with no executor.
2. **Missing dependency** — an MCP client for external tool discovery/call.
3. **Security risk** — process spawns (git/cargo) and any future MCP stdio; the
   tier-2 process sandbox is not yet wired (ADR-001).
4. **Architectural weakness** — risk→decision mapping duplicated in policy/runtime/agent;
   orchestrator `Replan` is a stub.
5. **Highest-value feature** — an MCP tool client (tool discovery + call).
6. **Simpler alternative** — implement `RunTest` executor (reuse verification runner).
7. **More ambitious alternative** — full MCP stdio client + tool registry.

## Primary objective (chosen) — COMPLETE
**Implement `patch_file` executor** — parse a unified-diff patch from the payload,
apply it to the target file via the existing patch engine, and atomic-write the
result. Reuses tested components, spawns no process (no new security boundary),
and closes the `PatchFile` gap.

## Subtasks
- [x] `patch_exec.rs`: `patch_file(action, workspace, PatchMode)` — resolve path,
  parse patch, apply via `Patch::apply`, return evidence (before/after hashes + diff).
- [x] `lib.rs`: wire `ActionType::PatchFile` into `execute()`.
- [x] `error.rs`: add `InvalidPatch` / `PatchConflict` + `Patch` error kind.
- [x] Tests (7 unit + 1 end-to-end): apply, dry-run, stale-context, missing file,
  denied capability, traversal, malformed patch, `execute` path.
- [x] Docs: `patch-file-execution.md`.
- [x] Reviewed as independent security/reliability engineer (pass).

## Reassessment (post-landing)
`patch_file` is verified green (137 tests total, clippy clean). The dispatch gap is
reduced: `ReadFile`, `WriteFile`, `PatchFile`, `Git` are now executable.

**Next candidates (in order):**
1. **`RunTest` executor** — closes the `RunTest` gap by reusing the verification
   fabric's cargo runner (small, safe, no new trust boundary).
2. **Policy de-duplication (P7)** — single risk→decision mapping.
3. **Crosses a security boundary → design/ADR first:** the **tier-2 process sandbox**
   (securing `Execute`/`RunTest` process spawns per ADR-001) and the **MCP client**
   (external tool execution / stdio). These must be a design task before implementation.
