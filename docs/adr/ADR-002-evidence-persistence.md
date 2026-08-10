# ADR-002: Evidence Provenance Persistence

- **Status:** Accepted (foundation)
- **Date:** 2026-08-09
- **Deciders:** Principal systems architect
- **Related:** `docs/architecture/repository-assessment.md`,
  `ADR-001-forgecore-execution.md`, `crates/forge-core/src/evidence.rs`

## Context

ForgeCore needs a provenance subsystem so that every executed action is traceable:
`Task → Agent → Action → Policy Decision → Execution → Artifact → Verification`.
The question this ADR answers is **where** that evidence is persisted at this stage of
the project.

The project is at an early foundation phase. The crate is a pure, in-memory, library-
first Rust crate with a deliberately minimal dependency surface (serde, thiserror,
sha2, chrono). The execution kernel is still being established, and autonomous agents
are not yet integrated.

## Options considered

### 1. SQLite (via `rusqlite`)

- **Pros:** Durable, queryable, transactional, widely used; a natural fit for
  relational provenance (tasks, actions, artifacts, hashes).
- **Cons:** Adds a C dependency (`libsqlite3`) and a build/compile surface; requires a
  connection lifecycle and schema migrations; more than the current, single-crate,
  in-memory stage needs. The README/assessment already plan SQLite/Room for *later*
  (persistence layer in the Kotlin control plane).

### 2. Files (JSON on disk)

- **Pros:** Simple, portable, human-inspectable, no new dependency; each record is a
  standalone JSON document.
- **Cons:** No querying or transactions without extra logic; concurrent append requires
  care; still more machinery than the current need.

### 3. In-memory store + serializable records (chosen)

- **Pros:** Zero new dependencies; deterministic and trivially testable; matches the
  current library-first stage. Every `Evidence` already serializes to standalone JSON,
  so persisting to files or SQLite later is a localized change behind a stable
  interface.
- **Cons:** Not durable across process restarts on its own.

### 4. Graph database

- **Explicitly rejected** (per the task): provenance is a small, well-understood
  relation set at this stage. A graph database is unwarranted complexity and is
  premature.

## Decision

Use an **in-memory `EvidenceStore`** whose records serialize to standalone JSON.

- `EvidenceStore` is an append-only, keyed store of `Evidence` (each an immutable
  `ExecutionRecord` + content fingerprint).
- Every record is fully serializable via serde, so files/SQLite can be introduced
  behind the same interface without changing callers.
- A graph database is explicitly not introduced.

## Rationale

- The crate is library-first and offline; adding SQLite now would add a C dependency
  and a schema before the execution kernel is even finalized.
- Deterministic, in-memory storage is sufficient to prove the *provenance model* and
  the reconstruction guarantee (see the tests), which is the actual deliverable at this
  stage.
- The persistence boundary is clean: `EvidenceStore` is the interface, and a `FileStore`
  or `SqliteStore` can be substituted later with no API change to consumers.

## Consequences

- **Positive:** minimal dependencies, fast deterministic tests, clear boundary for
  future persistence.
- **Negative:** evidence is not durable across process restarts until a file/SQLite
  backend is added.
- **Migration path:** (1) add a `FileStore` writing one JSON file per record, then
  (2) add an optional SQLite backend behind the same trait when the control-plane
  persistence layer is built. Graph storage only if/when cross-task graph queries
  become a real requirement.