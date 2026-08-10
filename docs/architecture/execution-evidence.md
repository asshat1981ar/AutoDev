# Execution Evidence / Provenance Subsystem

This document describes ForgeCore's evidence and provenance subsystem, which makes
every executed action traceable.

## Provenance chain

```text
Task → Agent → Action → Policy Decision → Execution → Artifact → Verification
```

Each of these is captured on an [`ExecutionRecord`] (`crates/forge-core/src/evidence.rs`):

- **Task** — `record.task_id`
- **Agent** — `record.agent_id`
- **Action** — `record.action_id` + a serialized snapshot of the action (`record.action`)
- **Policy Decision** — `record.policy` (`PolicyOutcome::Allow | RequireApproval | Deny`)
- **Execution** — `record.status`, `record.started_at`, `record.completed_at`,
  `record.error`
- **Artifact** — `record.artifacts` (`Vec<Artifact>`)
- **Verification** — `record.verification` (hashes, diffs, etc.)

## Types

| Type | Purpose |
| --- | --- |
| `ExecutionRecord` | The complete, traceable unit of provenance for one action. |
| `Artifact` | An artifact produced by execution (id, name, kind, hash, size, path, timestamp). |
| `ArtifactHash` / `ArtifactHashAlgo` | A content hash (currently SHA-256). |
| `PolicyOutcome` | The captured policy decision. |
| `ExecutionErrorInfo` | Serialized error kind + message for failures. |
| `Evidence` | An immutable record + its SHA-256 fingerprint over canonical JSON. |
| `EvidenceStore` | Append-only, keyed store of `Evidence`. |
| `record_from(...)` | The single builder that turns an action + result into a record. |
| `action_id_from_record` / `action_type_from_record` | Reconstruction helpers. |

## Integrity

`Evidence` wraps a record with a fingerprint computed over the record's canonical JSON
(`Evidence::from_record`). `Evidence::verify()` recomputes the fingerprint and detects
tampering. The fingerprint is stored alongside the record, so a record can be verified
and reconstructed independently.

## Persistence

Per **ADR-002**, persistence is an in-memory `EvidenceStore` whose records serialize to
standalone JSON. This is appropriate at this stage (library-first, minimal deps,
deterministic tests). Files/SQLite can be introduced later behind the same interface; a
graph database is deliberately not used.

## Reconstruction guarantee

Because each record stores a full serialized snapshot of the originating action, an
action can be reconstructed from its evidence:

```text
EvidenceStore.by_action_id(action_id)
  → record.action (serialized AgentAction)
  → serde_json::from_value::<AgentAction>
```

This is proven by tests (see below).

## Tests

Coverage in `crates/forge-core/src/evidence.rs` (unit) and
`crates/forge-core/tests/evidence.rs` (end-to-end):

- `artifact_hash_hashes_content`
- `evidence_fingerprint_verifies_and_detects_tampering`
- `store_inserts_and_looks_up`
- `action_can_be_reconstructed_from_evidence` (unit)
- `read_action_is_reconstructed_from_evidence` (real `read_file` execution)
- `git_action_is_reconstructed_from_evidence` (real `git status` execution)
- `two_actions_are_traceable_by_chain`

The end-to-end tests run a real action through `forge_core::execute`, record the
evidence, verify the fingerprint, and reconstruct the exact `AgentAction` from the
evidence.