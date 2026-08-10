# Safe File Mutation (`write_file`) Execution Path

This document traces the execution path of the `write_file` operation through the
ForgeCore kernel. It is the platform's first **mutating** operation, built on the
existing policy and workspace layers plus the patch engine (for diff generation).

## Pipeline

```text
AgentAction
  → validate_action          (structural invariants)
  → capability check         (is the `write_file` capability granted?)
  → evaluate_policy          (risk → Allow / RequireApproval / Deny)
  → workspace.resolve_path   (containment + symlink/traversal defense)
  → proposed change          (validate payload: path + content)
  → diff                     (before/after unified diff via patch::generate_diff)
  → atomic write             (temp file + rename; never partial)
  → evidence                 (before/after hashes + new hash + diff)
```

## Security invariants

1. **Authorization is structural, not advisory.** Writing requires the `write_file`
   capability (checked by `has_required_capability`) and a passing policy decision.
   An action simply *asking* to write is not enough — unauthorized writes are refused
   before any filesystem access. This makes accidental or malicious writes
   structurally difficult rather than relying on agent instructions.
2. **No workspace escape.** `workspace.resolve_path` canonicalizes the path and rejects
   absolute paths outside the allowed roots (`PathOutsideWorkspace`), traversal (`..`)
   (`PathTraversal`), and symlink escapes (`SymlinkEscape`).
3. **Atomic writes.** Content is written to a sibling temporary file and then atomically
   renamed over the target. The target is never observed in a partial state.
4. **Failure recovery / rollback by construction.** If the write or rename fails, the
   temporary file is removed and the original is left untouched. Because the rename is
   atomic, the on-disk state is always either the old or the new file — never a mix.
5. **Bounded writes.** The new content must fit within `Workspace::max_bytes`
   (`OversizedFile`), the same limit enforced by `read_file`.
6. **Dry-run mode.** In `WriteMode::DryRun` the change is fully computed and validated
   (including the diff and hashes) but the filesystem is never modified.
7. **Approval gating.** Medium/High/Critical risk actions require approval; until the
   approval flow is wired up they are refused (`RequiresApproval`) rather than written.

## Evidence

A successful write returns an `ExecutionResult` whose `verification` payload includes:

- `path` — the canonical path written.
- `before_sha256` — SHA-256 of the original contents (or `null` if the file is new).
- `after_sha256` — SHA-256 of the new contents.
- `diff` — the generated unified diff (or `null` when creating a new file).
- `created` — whether the file did not previously exist.

## Explicit authorization

`write_file` enforces the same gate as `read_file`:

1. `evaluate_policy` performs structural validation.
2. `has_required_capability` requires the `WriteFile` capability.
3. `evaluate_policy` maps risk to `Allow` / `RequireApproval` / `Deny`; only `Allow`
   proceeds, and `RequireApproval`/`Deny` are refused.

## Errors

| Error | Trigger |
| --- | --- |
| `CapabilityDenied` | Missing `write_file` capability |
| `RequiresApproval` | Medium/High/Critical risk (approval not wired) |
| `PayloadNotObject` / `MissingPayloadField` / `PayloadFieldNotString` | Malformed payload |
| `PathTraversal` / `SymlinkEscape` / `PathOutsideWorkspace` | Workspace escape attempt |
| `OversizedFile` | Content exceeds `max_bytes` |
| `Io` | Underlying filesystem error (temp file cleaned up on failure) |

## Tests

Coverage in `crates/forge-core/src/write.rs` (unit) and
`crates/forge-core/tests/execute.rs` (integration):

- atomic write creates and replaces a file (content, before/after hashes, diff)
- dry-run does not touch the filesystem
- denied capability
- traversal rejection
- unauthorized (absolute outside) path rejection
- oversized content rejection
- missing `content` payload field
- symlink escape
- approval required for high risk
- end-to-end `execute` write path