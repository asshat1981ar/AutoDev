# PR overlap evidence candidate

Status: draft, read-only client workflow candidate.

## Gap classification

**Client workflow feature** — deterministic cross-pull-request changed-path collision evidence.

AutoDev has multiple concurrent development branches, including authority-core, evaluation, protocol, client, and external-edge work. The repository contains no existing cross-PR overlap analyzer, and the connected GitHub tool surface provides per-PR changed-file and compare primitives rather than a repeatable repository-local collision report.

## Trust boundary

The candidate:

- accepts already-observed PR-number -> changed-path mappings;
- performs no network access;
- writes no repository state unless a caller explicitly chooses an output file;
- does not merge, rebase, update refs, approve, block, or label pull requests;
- does not call ForgeCore or alter capabilities, policy, credentials, approvals, persistence, or execution authority.

It is evidence-only. Severity is advisory and cannot authorize or prevent execution.

## RED -> GREEN contract

The RED contract requires:

- deterministic ordering independent of input-map ordering;
- duplicate paths within one PR cannot create a false collision;
- a path changed by three PRs is one collision with all owners;
- ForgeCore paths classify as `authority-core` and workflow paths as `ci-governance`;
- authority-core and CI-governance collisions are advisory `high` severity.

The initial implementation exposed a path-normalization defect: `lstrip("./")` damaged `.github/...` paths. The implementation was corrected to remove only explicit leading `./` prefixes.

## Current repository evaluation

Observed open-PR changed paths show a concrete collision that manual inspection must currently discover:

- PR #17 changes `crates/forge-core/src/lib.rs`.
- PR #19 changes `crates/forge-core/src/lib.rs`.
- PR #25 changes `crates/forge-core/src/lib.rs`.

The candidate reports this as one three-way `authority-core` collision with advisory `high` severity.

Additional observed overlap exists between PR #17 and PR #22 on `crates/autodev-server/Cargo.toml` and `crates/autodev-server/src/lib.rs`; those are general integration collisions rather than ForgeCore authority collisions.

This establishes utility without claiming merge conflict certainty: changed-path overlap is a reconciliation-risk signal, not proof that Git will conflict semantically or textually.

## Evaluation gate

Candidate acceptance requires:

1. RED contract observed before implementation.
2. Unit tests GREEN on supported Python CI lanes.
3. Stable output for identical path sets.
4. No network or repository mutation capability added.
5. No false collision from duplicate paths within a single PR.
6. Documentation states that overlap severity is advisory only.

## Rollback

Rollback is complete removal of:

- `scripts/pr_overlap_evidence.py`;
- `tests/test_pr_overlap_evidence.py`;
- this document.

No data migration, credential rotation, policy rollback, or ForgeCore change is required because the candidate has no persistence or authority integration.
