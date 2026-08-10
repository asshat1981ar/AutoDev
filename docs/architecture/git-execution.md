# Git Workspace Execution

This document describes how ForgeCore provides Git awareness, and the three-tier
capability separation that governs read-only, mutating, and destructive Git
operations.

## Approach

Git operations are shelled out to the system `git` binary via
[`std::process::Command`] using an **argv array and no shell** — there is no shell
interpolation, so there is no command-injection surface. Every command is scoped to
the workspace root with `git -C <root>`, so Git cannot operate outside the workspace
(though Git itself resolves the real repository root, which may be an ancestor of the
workspace root).

This is a controlled, necessary exception to the platform's "no process execution"
rule: Git operations inherently require the git executable.

## Three capability tiers

Operations are separated by privilege, and each tier requires its own capability:

| Tier | Capability | Operations | Effect |
| --- | --- | --- | --- |
| **Read-only** | `git` (default-grant for read) | `repository_info`, `status`, `diff`, `branch`, `log` | No state change |
| **Mutating** | `git:write` | `checkpoint`, `prepare_commit` | Changes index/refs (reversible) |
| **Destructive** | `git:destructive` + **approval** | `rollback` (`reset --hard`, `checkout --`) | Irreversible; refused without approval |

**Default policy (per decision):** read-only Git is permitted for any agent that holds
the `git` capability. Mutating operations require `git:write`. Destructive operations
require BOTH the `git:destructive` capability AND an explicit approval attestation
(`payload.approved == true` or the `approval:critical` capability); otherwise they are
refused with `RequiresApproval`.

The gate is enforced by `git::gate(GitTier, capabilities)` before any Git command runs
— it is structural, not advisory. An action that merely *mentions* `git` in its payload
is not enough; the correct capability must be present.

## Never operations (structural guardrail)

The following operations are **never** performed without explicit policy
authorization, enforced by `git::forbidden_command` before any command runs:

- **Force push** — `push --force`, `push --force-with-lease`, `push -f`
- **Delete remote branches** — `push --delete`, `push -d`
- **Modify credentials** — `config credential`, `credential`
- **Rewrite protected history** — `filter-branch`, `rebase`, `clean -fd`, `remote remove`,
  `remote rm`, `remote set-url`

Any attempt to run one of these returns `GitOperationForbidden`. This makes the
"never" property structural rather than dependent on an agent's instructions (which are
not authority).

## Implemented operations

### Read-only
- `repository_info` — whether the workspace is a repo, its root, branch, HEAD, clean.
- `status` — `git status --short --branch` (branch + changed entries + clean flag).
- `diff` — `git diff` (unstaged working-tree changes).
- `branch` — current branch and all local branches.
- `log` — `git log --oneline -n <limit>` (default 10).

### Mutating
- `checkpoint` — `git stash push -m <message>`; returns the stash reference. Reversible.
- `prepare_commit` — `git add -A` and, if `commit=true`, `git commit -m <message>`.

### Destructive
- `rollback` — `git reset --hard <ref>` (discard changes, move HEAD) or
  `git checkout -- .` (discard working-tree changes only).

## Dispatch

`git::execute_git(action, workspace)` reads the payload `operation` field and dispatches
to the appropriate operation, applying the tier gate via `action.capabilities`. It is
wired into the top-level `execute` dispatcher under `ActionType::Git`.

## Errors

| Error | Trigger |
| --- | --- |
| `NotARepository` | The workspace is not inside a git work tree |
| `GitCapabilityDenied` | Missing the capability for the operation's tier |
| `RequiresApproval` | Destructive operation without an approval attestation |
| `GitOperationForbidden` | Attempted a "never" operation (force push, delete remote, credentials, history rewrite) |
| `GitFailed(op, stderr)` | The git command exited non-zero (git's stderr attached) |
| `MissingPayloadField("operation")` | Payload has no `operation` field |
| `PayloadNotObject` | Payload is not a JSON object |

## Tests

Coverage in `crates/forge-core/src/git.rs` (unit) and
`crates/forge-core/tests/git.rs` (end-to-end via `execute`), using throwaway git
repositories:

- repository detection (repo vs. non-repo)
- status clean/dirty
- diff reports unstaged changes
- branch information
- repository info (branch, HEAD, clean)
- checkpoint stashes changes
- commit preparation commits changes
- rollback resets the working tree
- read gate: `git` capability permits read; `git` alone does NOT permit mutate/destructive
- mutate/destructive require `git:write` / `git:destructive`
- not-a-repository reported
- forbidden operations refused (force push, delete remote, credentials, history rewrite)
- rollback refuses forbidden commands
- destructive requires approval (refused without, allowed with `approved: true`)
- end-to-end: read status, read denied without capability, mutate/destructive denied
  with only read capability, checkpoint with write capability, destructive approval gate