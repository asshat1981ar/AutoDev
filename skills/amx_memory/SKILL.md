---
name: amx-memory
version: "1.0.0"
description: |
  AMCX-1 v1.1 Agent Memory (AMX) canonical portable memory history implementation.
  
  Provides durable, machine-readable memory storage conforming to AMCX-1 specifications.
  Every memory retains: origin, logical identity, repository/worktree/project scope,
  provenance, causal ancestry, trust/validity state, visibility, purpose, retraction/deletion
  barriers, and canonical semantic digest.
model: "mistral-large-latest"
preferred_tools: [read_file, write_file, edit]
temperature: 0.0
---

## AMX Memory Skill

**Purpose:** Canonical portable memory history for agentic systems.

**AMCX-1 Compliance:** This skill implements the AMX (Agent Memory) component of AMCX-1 v1.1.

### Capabilities

- Store AMX-compliant memory records with all required fields
- Retrieve memories by logical identity
- List all stored memories
- Find memories by origin, scope, or other criteria
- Automatic SHA-256 semantic digest computation
- Validation against AMX requirements

### State Format

Memory records are stored in JSON format conforming to `schemas/amx-memory-v1.json`.

Each memory contains:
- `schema_version`: Always "amx-memory-v1"
- `origin`: Source system/agent that created the memory
- `logical_identity`: Stable identifier (lowercase hyphen-separated slug)
- `repository_scope`: Repository revision, path, worktree ID, and optional project scope
- `provenance`: Array of provenance entries with source, timestamp, checksum, description
- `causal_ancestry`: Array of ancestor references with logical_identity and relationship
- `trust_validity_state`: Trust level, validity status, last validated timestamp
- `visibility`: Scope and access level
- `purpose`: Primary purpose and task class
- `retraction_deletion_barriers`: Retractable/deletable flags and conditions
- `canonical_semantic_digest`: SHA-256 hash of the memory content

### Usage

```python
from skills.amx_memory.amx_memory import AMXMemory

# Initialize with state path
mem = AMXMemory(state_path=".vibe/amcx-memory/state.json")

# Store a memory
memory = {
    "schema_version": "amx-memory-v1",
    "origin": "vibe:skill:amx-memory",
    "logical_identity": "my-memory-001",
    "repository_scope": {
        "repository_revision": "abc123",
        "repository_path": "/path/to/repo",
        "worktree_id": "primary"
    },
    "provenance": [{
        "source": "skill:amx-memory",
        "timestamp": "2026-08-20T21:00:00Z",
        "checksum": "...",
        "description": "Initial storage"
    }],
    "causal_ancestry": [],
    "trust_validity_state": {
        "trust_level": "high",
        "validity_status": "valid",
        "last_validated": "2026-08-20T21:00:00Z"
    },
    "visibility": {
        "scope": "workspace",
        "access_level": "read"
    },
    "purpose": {
        "primary": "knowledge:pattern",
        "task_class": "memory:storage"
    },
    "retraction_deletion_barriers": {
        "retractable": True,
        "deletable": False
    },
    "timestamp": "2026-08-20T21:00:00Z"
}
# canonical_semantic_digest is auto-computed
mem.store(memory)

# Retrieve a memory
retrieved = mem.retrieve("my-memory-001")

# List all memories
all_memories = mem.list_all()
```

### Integration with Vibe

This skill integrates with Vibe's native tooling:
- Uses `write_file` and `read_file` for persistence
- Respects Vibe's tool permissions and allowlists
- Does not bypass ForgeCore execution boundaries

### Verification

Run tests:
```bash
python3 skills/amx_memory/tests/test_amx_memory_skill.py -v
```

All tests must pass before integration.

---

**Note:** This skill provides AMX memory storage only. ECM (collaboration history) is separate.
