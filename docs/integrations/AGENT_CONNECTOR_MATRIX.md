# Agent × Connector Authority Matrix

Date: 2026-08-21

## Rule

Connector access is deny-by-default. Exact MCP tool names are not approved until the Connector is live, authenticated, and its tool schema has been refreshed and snapshotted. This matrix defines capability intent; a later reviewed tool-discovery step converts capability intent into exact Mistral `include` allowlists and `requires_confirmation` rules.

`R` = read capability. `C` = external mutation allowed only with human confirmation. `—` = not attached.

| Agent | GitHub | Linear | Context7 | DeepWiki | Notes |
|---|---:|---:|---:|---:|---|
| Flow Orchestrator | R | R/C | — | — | May inspect coordination state; backlog mutation requires confirmation |
| Product & Requirements Engineer | R | R/C | R | R | May propose/create/update backlog artifacts only through confirmation |
| Research & Technology Scout | R | R | R | R | Read-only research surface |
| Systems Architect | R | R | R | R | Read-only architecture/repository context |
| Feature Engineer | R | R | R | R | Source mutation remains ForgeCore-only |
| Platform & Integration Engineer | R | R | R | R | Source/build mutation remains ForgeCore-only |
| Test & Verification Engineer | R | R | R | R | Must not self-authorize implementation effects |
| Security & Reliability Engineer | R | R | R | R | Read-only evidence; may block, not directly merge/release |
| Code Review & Maintainability Engineer | R | R | R | R | Independent review; no self-merge authority |
| CI/CD & Release Engineer | R | R | R | R | Release/deployment effects remain approval + ForgeCore gated |

## Capability intent by Connector

### GitHub App

Allowed initially:

- repository discovery
- code/file reading
- issue/PR reading
- review/status evidence
- workflow/check reading

Forbidden initially even if the featured Connector exposes them:

- merge pull request
- push/commit source
- delete branches
- change repository settings
- modify protections
- create/publish releases
- dispatch production deployment workflows

These effects belong behind AutoDev's ForgeCore authority boundary.

### Linear

Allowed reads:

- teams/projects
- issues
- issue status
- project state
- comments needed as task context

Potential writes, always confirmation-gated:

- create issue
- update issue fields
- move issue state
- add project/task comments

Forbidden without a new policy review:

- bulk deletion
- workspace/project administration
- membership/permission changes

### Context7

Read-only intent:

- resolve a library/package identifier
- retrieve current version-specific documentation/examples

No external mutation capability is needed.

### DeepWiki

Read-only intent:

- inspect repository/wiki structure
- ask repository-context questions
- retrieve semantic codebase explanations

No external mutation capability is needed.

## Separation of duties

The model may be able to invoke a Connector tool, but that does not grant an AutoDev `AuthorizationGrant`.

For source/process effects:

```text
Agent reasoning
  -> typed intent
  -> policy evaluation
  -> ForgeCore AuthorizationGrant
  -> execution
  -> evidence
  -> independent verification
```

A Mistral GitHub tool that could bypass this sequence is excluded rather than treated as a shortcut.

## Tool-discovery conversion procedure

For each Connector:

1. Refresh tools through Mistral.
2. Save sanitized schema snapshot.
3. Map every discovered tool to one of: `allow-read`, `confirm-write`, `deny`, `unknown`.
4. Fail the policy build if any tool remains `unknown`.
5. Produce exact per-agent `include` lists.
6. Produce exact `requires_confirmation` lists for permitted mutations.
7. Run negative tests proving denied tools are unavailable.
8. Require review before activating the new tool policy.

Newly discovered tools are denied until this procedure is repeated.