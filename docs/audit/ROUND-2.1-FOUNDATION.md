# Round 2.1 Audit Foundation

Status: correction/supersession foundation; implementation is blocked until evidence gates pass.

## Purpose
This document converts the adversarial review thread and architecture audit into repository-enforced working rules. It does not alter prior audit artifacts; corrections must be versioned and mechanically reproducible.

## Core invariant
Untrusted intent -> trusted authorization -> confined execution -> independently verified evidence -> bounded replanning.

## Mandatory architecture gates
1. Kernel-owned `ExecutionAuthority`; model/action payloads may request capabilities but can never grant them.
2. One `SandboxAdapter` boundary for every native subprocess, including Git, compilers, test runners and stdio MCP tools.
3. Immutable `ApprovalRecord` bound to action digest, task, policy version, approver and expiry; mutation or replay invalidates approval.
4. `ExecutionEnvelope` is the durable state aggregate. Planner, dispatcher, executor and verifier are stateless transitions over it.
5. Canonical lifecycle: DISCOVER -> PLAN -> AUTHORIZE -> ACT -> VERIFY -> ACCEPT | REPLAN | BLOCK.
6. Verification is independent of generation and must emit deterministic evidence.
7. Evidence metadata uses SQLite WAL plus content-addressed artifacts; transitions must be reconstructable after crash.
8. Repository intelligence must be benchmarked against lexical baselines before semantic indexing replaces them.
9. Learned/model routing may optimize cost/latency/success but must never expand authority.
10. Feature expansion is subordinate to security, correctness, evidence quality and bounded termination.

## Audit artifact rules
- Never overwrite a rejected or superseded report. Create a new versioned artifact.
- Requirement matrices are generated from explicit references, not maintained manually.
- A primary mapping is valid only when its DifferenceRecord explicitly cites and semantically addresses the requirement ID.
- Requirements may map to multiple DifferenceRecords.
- Every generated audit bundle must include source hashes, coverage counts, missing-ID checks, mapping validation and a detached SHA-256 manifest.
- CI must fail on orphaned requirements, duplicate primary ownership without an explicit exception, stale source digests or manifest mismatch.

## Worktree policy
All autonomous implementation occurs in a disposable isolated worktree rooted under `.worktrees/` or an externally configured sandbox root. The main checkout is coordination-only.

Required flow:
```bash
git fetch --all --prune
git worktree add .worktrees/<task-id> -b task/<task-id> origin/main
cd .worktrees/<task-id>
```

Rules:
- Never execute generated code from the coordination checkout.
- Never share mutable build/output directories between concurrent worktrees.
- Secrets remain outside the worktree and are injected through scoped runtime mechanisms.
- Network, filesystem writes, subprocesses and resource ceilings are deny-by-default and granted per task.
- Destructive Git actions, remote pushes, release publication and policy changes require explicit operator approval evidence.
- On completion, archive evidence before removing the worktree.

## Mandatory skill batches
Skills are repository assets with versioned contracts. Create them in batches grouped by responsibility, with each skill containing: purpose, triggers, inputs, outputs, allowed tools, authority requirements, failure modes, deterministic verification and tests.

Initial mandatory batches:
1. `authority`: capability derivation, approval validation, policy evaluation.
2. `sandbox`: process execution, filesystem confinement, network policy, resource budgets.
3. `evidence`: hashing, manifests, provenance, transition recording, replay verification.
4. `audit`: requirement extraction, DifferenceRecord generation, coverage adjudication, supersession packaging.
5. `repo-intelligence`: lexical retrieval baseline, syntax/index graph, context scoring and benchmark harness.
6. `verification`: compile/test/lint/AST checks, differential review, rollback validation.
7. `planning`: bounded decomposition, termination budgets, replanning criteria.
8. `worktree`: isolated task bootstrap, cleanup, artifact collection and branch hygiene.

No batch is considered complete until contract tests demonstrate fail-closed behavior.

## Repository boundary decision
Keep these concerns in the AutoDev monorepo until their API/security boundaries are stable and independently releasable. A separate repository is justified only when a component has an explicit versioned protocol, isolated CI/release lifecycle, no access to kernel internals, and a measured need for independent distribution.

Likely future extraction candidates: generic audit-ledger tooling and generic skill-contract tooling. Do not extract the authority kernel, evidence kernel or sandbox adapter during the current hardening phase.

## Next implementation order
1. Mechanically encode audit schemas and coverage validation.
2. Add worktree bootstrap/guard scripts and CI checks.
3. Add skill contract schema and generate the eight mandatory batches as stubs with tests.
4. Harden ExecutionAuthority and ApprovalRecord.
5. Route all subprocesses through SandboxAdapter.
6. Collapse orchestration around ExecutionEnvelope.
7. Persist evidence in SQLite WAL/CAS.
8. Introduce independent verification profiles.
9. Benchmark repository intelligence before replacing lexical retrieval.

Each step must ship as a narrow branch/PR with tests and generated evidence. No broad refactor is allowed to bypass these gates.
