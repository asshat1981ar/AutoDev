# Read File Execution Path

This document traces the execution path of the `read_file` operation through the
ForgeCore kernel. It is the first *real* (filesystem-touching) operation in the
platform, and it establishes the security invariants that later operations
(`write_file`, `patch_file`, `execute`, etc.) must follow.

## Pipeline

```text
AgentAction
  → validate_action          (structural invariants)
  → capability check         (is the `read_file` capability granted?)
  → evaluate_policy          (risk → Allow / RequireApproval / Deny)
  → workspace.resolve_path   (containment + symlink/traversal defense)
  → metadata + size gate     (reject missing / directory / oversized)
  → bounded read             (never read more than max_bytes)
  → evidence                 (schema-conformant ExecutionResult + content hash)
```

## Line-by-line responsibilities

| Stage | Where | Purpose |
| --- | --- | --- |
| 1. Structural validation | `policy::validate_action` | Rejects empty ids/reason; enforces `approval:critical` for critical risk. |
| 2. Capability check | `policy::has_required_capability` | `read_file` requires the `ReadFile` capability to be granted; otherwise `CapabilityDenied`. |
| 3. Policy decision | `policy::evaluate_policy` | `Low → Allow`; `Medium/High/Critical → RequireApproval`. Approval is not yet implemented, so `RequireApproval` surfaces `RequiresApproval`. |
| 4. Payload extraction | `read::read_file` | Reads `payload.path` (must be a JSON object with a string `path`). |
| 5. Workspace resolution | `workspace::resolve_path` | Anchors relative paths to the workspace root, normalizes `..`, canonicalizes (resolves symlinks), and verifies containment. |
| 6. Metadata + size gate | `read::read_resolved` | `fs::metadata` rejects missing files and directories; enforces `max_bytes`. |
| 7. Bounded read | `read::read_resolved` | `fs::read` on a size-gated file — never reads more than the limit. |
| 8. Evidence | `evidence::*` | Produces a schema-conformant `ExecutionResult` with SHA-256 hash + metadata. |

## Security invariants

1. **No privileged mutation.** `read_file` performs read-only access. It never
   creates, modifies, or deletes files, and never executes a process.
2. **Workspace containment.** Every path is resolved against allow-listed roots.
   Absolute paths outside the workspace are denied (`PathOutsideWorkspace`);
   traversal (`..`) is rejected (`PathTraversal`).
3. **Symlink escape defense.** `resolve_path` canonicalizes the path, so a symlink
   inside the workspace pointing outward resolves outside and is denied
   (`SymlinkEscape`).
4. **Size-bounded reads.** A file larger than `Workspace::max_bytes` is rejected
   (`OversizedFile`) before any read, so the kernel never reads unbounded data.
5. **Capability-gated.** An action without the `ReadFile` capability is denied even
   if the path is valid.
6. **Deterministic evidence.** Identical input yields an identical content hash and
   verification payload.

## Timeout / cancellation strategy

`read_file` is a bounded, synchronous operation. The primary control is the **size
gate**: the file is rejected if it exceeds `max_bytes`, so the read is always bounded
and cannot block indefinitely on a large file. A full asynchronous
cancellation/timeout mechanism (e.g., an async runtime or subprocess timeouts) is
**deferred**; it is not needed until process/network execution is introduced, where it
will be handled by the `SandboxAdaptor` (see `ADR-001-forgecore-execution.md`).

## Result shape

A successful read returns an `ExecutionResult` (schema-conformant):

- `stdout` — the UTF-8 decoded file contents.
- `verification` — a `ReadMetadata` object: `{ path, sha256, size, modified_at }`.
- `artifacts` — `[canonical_path]`.
- `status` — `succeeded`.
- `exit_code` — `null` (not a process).

Non-UTF-8 content produces a structured `InvalidUtf8` error (the hash is still
meaningful, but `stdout` is not populated).

## Errors

| Error | Trigger |
| --- | --- |
| `MissingActionId` / `MissingTaskId` / `MissingAgentId` / `MissingReason` | Structural validation failure |
| `CriticalApprovalRequired` | Critical risk without `approval:critical` |
| `CapabilityDenied` | Action lacks the `read_file` capability |
| `RequiresApproval` | Medium/High/Critical risk (approval not yet implemented) |
| `PayloadNotObject` / `MissingPayloadField("path")` / `PayloadFieldNotString("path")` | Malformed payload |
| `PathTraversal` | `..` escape attempt |
| `SymlinkEscape` | Symlink resolves outside the workspace |
| `PathOutsideWorkspace` | Absolute path outside allowed roots |
| `FileNotFound` | Target does not exist |
| `IsDirectory` | Target is a directory, not a file |
| `OversizedFile` | File exceeds `max_bytes` |
| `InvalidUtf8` | Content is not valid UTF-8 |
| `Io` | Underlying filesystem error |

## Tests

Coverage in `crates/forge-core/src/read.rs` (unit) and
`crates/forge-core/tests/execute.rs` (integration):

- successful read (content + hash + metadata)
- missing file
- directory
- oversized file
- unauthorized (absolute outside) path
- traversal (`..`)
- symlink escape
- denied capability
- missing `path` payload field
- end-to-end `execute` path
- determinism (same input → same evidence)
- unsupported action type rejected