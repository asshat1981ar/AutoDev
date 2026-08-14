# Patch File Execution Path

This document traces the `patch_file` operation through ForgeCore.

## Pipeline

```text
AgentAction
  → validate_action         (structural invariants)
  → capability check        (is the `patch_file` capability granted?)
  → evaluate_policy         (risk → Allow / RequireApproval / Deny)
  → workspace.resolve_path  (containment + symlink/traversal defense)
  → read target             (existing content for before-hash)
  → parse patch             (payload.patch = unified diff)
  → apply patch             (deterministic, context-validated)
  → atomic write            (only on clean apply; temp file + rename)
  → evidence                (before/after hashes + applied patch)
```

## Security / reliability properties

- **No process spawn.** `patch_file` reuses the pure patch engine and the atomic
  write path — it introduces no new subprocess or network security boundary.
- **Workspace confinement.** Every path is canonicalized and validated against the
  workspace (traversal / symlink escape / absolute-outside all rejected).
- **Capability-gated.** Requires the `patch_file` capability; risk policy is enforced.
- **Atomic writes.** Result is written to a sibling temp file and renamed over the
  target, so the file is never observed in a partial state; on failure the original
  is untouched (rollback by construction).
- **Deterministic.** `Patch::apply` is pure; the evidence carries the before/after
  hashes and the number of applied hunks.

## Behavior note

The applied result is re-joined with `\n` and a trailing newline, normalizing any
file that lacked a final newline. This is a deliberate, documented normalization
consistent with line-based diff semantics.

## Errors

| Error | Trigger |
| --- | --- |
| `CapabilityDenied` | Missing `patch_file` capability |
| `RequiresApproval` | Medium/High/Critical risk |
| `FileNotFound` | Target file does not exist |
| `InvalidPatch` | The unified diff text failed to parse |
| `PatchConflict` | A hunk failed to apply (stale context / range / overlap) |
| `PathTraversal` / `SymlinkEscape` / `PathOutsideWorkspace` | Workspace escape attempt |
| `PayloadNotObject` / `MissingPayloadField` / `PayloadFieldNotString` | Malformed payload |

## Tests

Coverage in `crates/forge-core/src/patch_exec.rs` (unit) and
`crates/forge-core/tests/execute.rs` (end-to-end via `execute`):

- patch applies to an existing file
- dry-run does not modify the file
- stale-context patch rejected (`PatchConflict`)
- missing file rejected
- denied capability rejected
- traversal rejected
- malformed patch rejected
- end-to-end `execute` patch path
