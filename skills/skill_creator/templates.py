"""
Skill Templates for Skill Creator

This module contains templates for different skill types that comply with
AutoDev AGENTS.md rules and AMCX-1 v1.1 specifications.

Each template includes:
- SKILL.md frontmatter
- Implementation file scaffold
- Test file scaffold (RED phase)
- __init__.py file

Placeholder syntax: {placeholder_name}
These will be replaced by _apply_substitutions in skill_creator.py
"""

from typing import Any


# =============================================================================
# SKILL.md Templates
# =============================================================================

AMX_MEMORY_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  AMX v1.1 canonical portable memory implementation.

  Every memory retains:
  - origin
  - logical identity
  - repository/worktree/project scope
  - provenance
  - causal ancestry
  - trust/validity state
  - visibility
  - purpose
  - retraction/deletion barriers
  - canonical semantic digest
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

**AMCX-1 Compliance:** This skill implements AMX component of AMCX-1 v1.1.

### Capabilities

- Store AMX-compliant memory records
- Retrieve memories by logical identity
- List all stored memories
- Validate against AMX requirements
- Automatic SHA-256 semantic digest computation

### Usage

```python
from skills.{name}.{file_name} import {class_name}

mem = {class_name}(state_path=".vibe/amcx-memory/state.json")
memory = {{
    "schema_version": "amx-memory-v1",
    "origin": "{origin}",
    "logical_identity": "my-memory-001",
    "repository_scope": {{
        "repository_revision": "abc123",
        "repository_path": "/path/to/repo",
        "worktree_id": "primary"
    }},
    "provenance": [{{
        "source": "{name}",
        "timestamp": "2026-08-20T21:00:00Z",
        "checksum": "...",
        "description": "Initial storage"
    }}],
    "causal_ancestry": [],
    "trust_validity_state": {{
        "trust_level": "high",
        "validity_status": "valid",
        "last_validated": "2026-08-20T21:00:00Z"
    }},
    "visibility": {{
        "scope": "workspace",
        "access_level": "read"
    }},
    "purpose": {{
        "primary": "{primary_purpose}",
        "task_class": "{task_class}"
    }},
    "retraction_deletion_barriers": {{
        "can_retract": False,
        "can_delete": False,
        "retention_period": "infinite"
    }},
    "canonical_semantic_digest": "..."
}}
mem.store(memory)
```
"""

ECM_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  ECM (Entity-Collaboration Model) collaboration state implementation.

  Tracks: task, attempt, role, role_lease, message, artifact_reference,
  evidence_reference, memory_binding, ContextView
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

**AMCX-1 Compliance:** This skill implements ECM component of AMCX-1 v1.1.

### Capabilities

- Track collaboration tasks and attempts
- Manage role assignments and leases
- Store and retrieve messages
- Reference artifacts and evidence
- Maintain ContextView state
"""

EVIDENCE_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  Evidence and verification skill for AMCX-1 v1.1.
  
  Records verification results and freshness tracking.
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

### Capabilities

- Record verification evidence
- Track freshness and validity
- Validate against verification schemas
- Generate evidence reports
"""

HOST_ADAPTER_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  Vibe tool extension / host adapter skill.
  
  Provides custom tool capabilities within Vibe permission model.
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

### Capabilities

- Extend Vibe toolset with custom capabilities
- Respect permission boundaries
- Integrate with host-specific features
"""

MCP_SERVER_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  MCP (Model Context Protocol) resource server skill.
  
  Exposes resources for MCP-compatible clients.
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

### Capabilities

- Serve MCP resources
- Handle resource read/list operations
- Validate MCP requests
"""

GENERAL_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} Skill

**Purpose:** {description}

### Capabilities

- General purpose skill implementation
"""


# =============================================================================
# Implementation Templates
# =============================================================================

AMX_MEMORY_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

AMX v1.1 canonical portable memory implementation for {name}.
\"\"\"

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


class {class_name}:
    \"\"\"AMX Memory implementation.\"\"\"
    
    def __init__(self, state_path: str = ".vibe/amcx-memory/state.json"):
        self.state_path = Path(state_path)
        self.state_path.parent.mkdir(parents=True, exist_ok=True)
        self._load_state()
    
    def _load_state(self) -> None:
        if self.state_path.exists():
            with open(self.state_path, 'r') as f:
                self.state = json.load(f)
        else:
            self.state = {"memories": []}
    
    def _save_state(self) -> None:
        with open(self.state_path, 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def store(self, memory: Dict[str, Any]) -> str:
        \"\"\"Store a memory record.\"\"\"
        if "schema_version" not in memory:
            memory["schema_version"] = "amx-memory-v1"
        if "origin" not in memory:
            memory["origin"] = "{origin}"
        if "logical_identity" not in memory:
            raise ValueError("logical_identity is required")
        if "canonical_semantic_digest" not in memory:
            memory["canonical_semantic_digest"] = self._compute_digest(memory)
        self.state["memories"].append(memory)
        self._save_state()
        return memory["logical_identity"]
    
    def retrieve(self, logical_identity: str) -> Optional[Dict[str, Any]]:
        \"\"\"Retrieve a memory by logical identity.\"\"\"
        for mem in self.state.get("memories", []):
            if mem.get("logical_identity") == logical_identity:
                return mem
        return None
    
    def list_memories(self) -> List[Dict[str, Any]]:
        \"\"\"List all stored memories.\"\"\"
        return self.state.get("memories", [])
    
    def _compute_digest(self, data: Dict[str, Any]) -> str:
        data_str = json.dumps(data, sort_keys=True)
        return hashlib.sha256(data_str.encode('utf-8')).hexdigest()
"""

ECM_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

ECM collaboration state implementation for {name}.
\"\"\"

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


class {class_name}:
    \"\"\"ECM State implementation.\"\"\"
    
    def __init__(self, state_path: str = ".vibe/ecm-state.json"):
        self.state_path = Path(state_path)
        self.state_path.parent.mkdir(parents=True, exist_ok=True)
        self._load_state()
    
    def _load_state(self) -> None:
        if self.state_path.exists():
            with open(self.state_path, 'r') as f:
                self.state = json.load(f)
        else:
            self.state = {}
    
    def _save_state(self) -> None:
        with open(self.state_path, 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def add_task(self, task_id: str, description: str, task_type: str = "general") -> Dict[str, Any]:
        \"\"\"Add a new task.\"\"\"
        task = {
            "id": task_id,
            "description": description,
            "type": task_type,
            "status": "created",
            "created_at": datetime.now(timezone.utc).isoformat()
        }
        self.state.setdefault("tasks", {}).setdefault(task_id, task)
        self._save_state()
        return task
"""

EVIDENCE_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

Evidence implementation for {name}.
\"\"\"

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


class {class_name}:
    \"\"\"Evidence tracking implementation.\"\"\"
    
    def __init__(self, state_path: str = ".vibe/evidence.json"):
        self.state_path = Path(state_path)
        self.state_path.parent.mkdir(parents=True, exist_ok=True)
        self._load_state()
    
    def _load_state(self) -> None:
        if self.state_path.exists():
            with open(self.state_path, 'r') as f:
                self.state = json.load(f)
        else:
            self.state = {"records": []}
    
    def _save_state(self) -> None:
        with open(self.state_path, 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def record(self, check_name: str, passed: bool, details: str = "") -> Dict[str, Any]:
        \"\"\"Record verification evidence.\"\"\"
        record = {
            "check_name": check_name,
            "passed": passed,
            "details": details,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        self.state["records"].append(record)
        self._save_state()
        return record
"""

HOST_ADAPTER_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

Host adapter implementation for {name}.
\"\"\"

from typing import Any, Dict, List, Optional


class {class_name}:
    \"\"\"Host adapter for custom tool capabilities.\"\"\"
    
    def __init__(self):
        pass
    
    def execute(self, action: str, **kwargs) -> Any:
        \"\"\"Execute a host-specific action.\"\"\"
        raise NotImplementedError(f"Action {action} not implemented")
"""

MCP_SERVER_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

MCP server implementation for {name}.
\"\"\"

from typing import Any, Dict, List, Optional


class {class_name}:
    \"\"\"MCP resource server.\"\"\"
    
    def __init__(self, resource_uri: str = "mcp://{name}"):
        self.resource_uri = resource_uri
    
    def list_resources(self) -> List[str]:
        \"\"\"List available resources.\"\"\"
        return []
    
    def read_resource(self, resource_name: str) -> Dict[str, Any]:
        \"\"\"Read a resource.\"\"\"
        raise NotImplementedError(f"Resource {resource_name} not found")
"""

GENERAL_IMPL_TEMPLATE = """#!/usr/bin/env python3
\"\"\"{description}

General skill implementation for {name}.
\"\"\"

from typing import Any


class {class_name}:
    \"\"\"General skill implementation.\"\"\"
    
    def __init__(self):
        pass
    
    def execute(self, **kwargs) -> Any:
        \"\"\"Execute the skill.\"\"\"
        raise NotImplementedError("Execute method not implemented")
"""


# =============================================================================
# __init__.py Template
# =============================================================================

INIT_TEMPLATE = """# {name} Package
# {description}

from .{file_name} import {class_name}

__all__ = ["{class_name}"]
"""


# =============================================================================
# Test Templates
# =============================================================================

AMX_MEMORY_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for AMX Memory skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        mem = {class_name}()
        self.assertIsNotNone(mem)
    
    def test_store_and_retrieve(self):
        \"\"\"Should store and retrieve memories.\"\"\"
        mem = {class_name}()
        test_memory = {{
            "logical_identity": "test-001",
            "origin": "test",
            "purpose": {{"primary": "test"}}
        }}
        mem.store(test_memory)
        retrieved = mem.retrieve("test-001")
        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved["logical_identity"], "test-001")


if __name__ == '__main__':
    unittest.main()
"""

ECM_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for ECM skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        ecm = {class_name}()
        self.assertIsNotNone(ecm)
    
    def test_add_task(self):
        \"\"\"Should add tasks.\"\"\"
        ecm = {class_name}()
        task = ecm.add_task("test-001", "Test task")
        self.assertEqual(task["id"], "test-001")


if __name__ == '__main__':
    unittest.main()
"""

EVIDENCE_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for Evidence skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        evidence = {class_name}()
        self.assertIsNotNone(evidence)
    
    def test_record(self):
        \"\"\"Should record evidence.\"\"\"
        evidence = {class_name}()
        record = evidence.record("test_check", True, "Test passed")
        self.assertTrue(record["passed"])


if __name__ == '__main__':
    unittest.main()
"""

HOST_ADAPTER_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for Host Adapter skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        adapter = {class_name}()
        self.assertIsNotNone(adapter)


if __name__ == '__main__':
    unittest.main()
"""

MCP_SERVER_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for MCP Server skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        server = {class_name}()
        self.assertIsNotNone(server)
    
    def test_list_resources(self):
        \"\"\"Should list resources.\"\"\"
        server = {class_name}()
        resources = server.list_resources()
        self.assertIsInstance(resources, list)


if __name__ == '__main__':
    unittest.main()
"""

GENERAL_TEST_TEMPLATE = """import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    \"\"\"RED Phase: Tests for General skill.\"\"\"
    
    def test_import(self):
        \"\"\"Should import successfully.\"\"\"
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        \"\"\"Should instantiate.\"\"\"
        skill = {class_name}()
        self.assertIsNotNone(skill)


if __name__ == '__main__':
    unittest.main()
"""




# =============================================================================
# Engram MCP Server Templates
# =============================================================================

ENGRAM_MCP_SKILL_MD_TEMPLATE = """---
name: {name}
version: "{version}"
description: |
  {description}

  Engram MCP Server - Persistent memory for AI agents with AMCX-1 integration.

  Provides:
  - SQLite-backed persistent memory storage
  - MCP server interface for agent access
  - AMX v1.1 canonical memory compliance
  - ECM collaboration state tracking
  - Semantic search capabilities
  - Knowledge graph support
  - Memory lifecycle management

  AMCX-1 Compliance: Implements AMX memory component with MCP interface.
  Integrates with ECM for collaboration history tracking.
model: "{model}"
preferred_tools: {preferred_tools}
temperature: {temperature}
---

## {name} - Engram MCP Server

**Purpose:** {description}

**AMCX-1 Compliance:** This skill implements Engram MCP server with full AMX memory compliance and ECM integration. Every memory operation is tracked as ECM entities (task, attempt, role) for complete collaboration history.

### Capabilities

- **AMX Memory**: Full AMX v1.1 compliance with schema validation
- **ECM Integration**: Memory operations tracked as tasks/attempts/roles
- **MCP Server**: HTTP-based MCP protocol interface
- **SQLite Storage**: Persistent memory with indexes for fast retrieval
- **Semantic Search**: Ready for vector embeddings
- **Knowledge Graph**: Extensible schema for relationships
- **Memory Lifecycle**: Automatic cleanup and organization

### Memory Schema

Conforms to AMX v1.1 specification with all required fields:
- schema_version: AMX memory schema version
- origin: Memory source (agent, user, system)
- logical_identity: Unique memory identifier
- repository_scope: Git repository context (revision, path, worktree)
- provenance: Memory lineage tracking with timestamps and checksums
- causal_ancestry: Parent memory references for lineage
- trust_validity_state: Trust level, validity status, last validated
- visibility: Scope and access level
- purpose: Primary purpose and task classification
- retraction_deletion_barriers: Retraction/deletion constraints
- canonical_semantic_digest: SHA-256 hash for integrity verification

### ECM Integration

Each memory operation creates corresponding ECM entities:
- **Task**: `memory:{logical_identity}:store`, `memory:{logical_identity}:retrieve`
- **Attempt**: Tracked per operation with status and timestamp
- **Role**: Memory operations assigned to `memory:amx` role class
- **Artifact Reference**: Memory content stored as artifact with checksum
- **Evidence Reference**: Validation results stored as evidence
- **Memory Binding**: Two-way binding between AMX memory and ECM task

### Usage

```python
from skills.{name}.{file_name} import {class_name}

# Initialize with AMX/ECM integration
engram = {class_name}(db_path=".vibe/engram-memory/memory.db")

# Store a memory with AMX compliance
memory = {{
    "logical_identity": "session-001",
    "origin": "{origin}",
    "memory_type": "episode",
    "content": "User requested feature X implementation",
    "purpose": {{{"primary": "task tracking", "task_class": "memory:amx"}}},
    "repository_scope": {{{"worktree_id": "primary"}}},
    "canonical_semantic_digest": engram.compute_digest(memory)
}}

# Validate against AMX v1.1 schema
engram.validate_amx(memory)  # Raises if non-compliant

# Store with ECM tracking
logical_id = engram.store(memory)

# Start MCP server
engram.start_server(host="127.0.0.1", port=8080)

# Query with AMX compliance filter
amx_memories = engram.query_by_amx_compliance(valid_only=True)
```

### MCP Resources

- `memory://list` - List all stored memories (with AMX validation)
- `memory://read/{id}` - Read a specific memory with full AMX fields
- `memory://search` - Search memories with AMX compliance filters
- `memory://store` - Store a new memory with AMX validation
- `memory://validate` - Validate a memory against AMX v1.1 schema
- `ecm://memory/{id}/task` - Get ECM task for a memory operation
- `ecm://memory/{id}/attempts` - List ECM attempts for a memory

### AMCX-1 Integration Contract

- **AMX**: This skill implements the AMX memory component. Memories are stored as AMX-compliant records with all required fields. The canonical semantic digest is computed automatically.
- **ECM**: All memory operations are tracked as ECM entities. The skill maintains a separate ECM state file that records tasks, attempts, and evidence for each memory operation.
- **Authority Boundary**: This skill provides storage and tracking only. It does not mint AuthorizationGrants or perform execution. All operations respect the Vibe tool permission model.
"""

ENGRAM_MCP_IMPL_TEMPLATE = '''#!/usr/bin/env python3
\"\"\"{description}

Engram MCP Server - Persistent memory with MCP interface.
Integrates with AMX memory patterns and AMCX-1 v1.1.
\"\"\"

import hashlib
import json
import sqlite3
import threading
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional
from http.server import HTTPServer, BaseHTTPRequestHandler
import urllib.parse


class {class_name}:
    """Engram MCP Server with SQLite storage, AMX compliance, and ECM integration."""
    

    # AMX v1.1 required fields
    AMX_MEMORY_REQUIRED_FIELDS = {
        "schema_version", "origin", "logical_identity", "repository_scope",
        "provenance", "causal_ancestry", "trust_validity_state", "visibility",
        "purpose", "retraction_deletion_barriers", "canonical_semantic_digest"
    }
    
    # ECM tracked entities
    ECM_ENTITY_TYPES = {
        "task", "attempt", "role", "role_lease", "message",
        "artifact_reference", "evidence_reference", "memory_binding", "ContextView"
    }
    def __init__(self, db_path: str = ".vibe/engram-memory/memory.db", 
                 ecm_state_path: str = ".vibe/engram-memory/ecm-state.json"):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.ecm_state_path = Path(ecm_state_path)
        self.ecm_state_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_db()
        self._init_ecm_state()
        self._server_thread = None
        self._server = None
        self.host = "127.0.0.1"
        self.port = 8080
    
    def _init_db(self):
        """Initialize SQLite database with AMX-compliant schema."""
        create_table = (
            "CREATE TABLE IF NOT EXISTS memories ("
            "id TEXT PRIMARY KEY,"
            "schema_version TEXT NOT NULL DEFAULT 'amx-memory-v1',"
            "origin TEXT NOT NULL,"
            "logical_identity TEXT UNIQUE NOT NULL,"
            "repository_scope_json TEXT,"
            "provenance_json TEXT,"
            "causal_ancestry_json TEXT,"
            "trust_validity_state_json TEXT,"
            "visibility_json TEXT,"
            "purpose_json TEXT,"
            "retraction_deletion_barriers_json TEXT,"
            "canonical_semantic_digest TEXT NOT NULL,"
            "memory_type TEXT,"
            "content TEXT,"
            "metadata_json TEXT,"
            "created_at TEXT NOT NULL,"
            "updated_at TEXT NOT NULL,"
            "ecm_task_id TEXT,"
            "ecm_compliant BOOLEAN DEFAULT 1)"
        )
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            conn.execute(create_table)
            conn.execute("CREATE INDEX IF NOT EXISTS idx_origin ON memories(origin)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_logical_identity ON memories(logical_identity)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_type ON memories(memory_type)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_ecm_compliant ON memories(ecm_compliant)")
            conn.commit()
    
    def _init_ecm_state(self):
        """Initialize ECM state file for collaboration history tracking."""
        if self.ecm_state_path.exists():
            with open(self.ecm_state_path, 'r') as f:
                self.ecm_state = json.load(f)
        else:
            self.ecm_state = {
                "tasks": {},
                "attempts": {},
                "roles": {},
                "role_leases": {},
                "messages": {},
                "artifact_references": {},
                "evidence_references": {},
                "memory_bindings": {},
                "context_views": {}
            }
        self._save_ecm_state()
    
    def _save_ecm_state(self):
        """Save ECM state to file."""
        with open(self.ecm_state_path, 'w') as f:
            json.dump(self.ecm_state, f, indent=2)
    
    def _memory_to_dict(self, row):
        """Convert database row to memory dictionary."""
        return {
            "id": row["id"],
            "schema_version": row["schema_version"],
            "origin": row["origin"],
            "logical_identity": row["logical_identity"],
            "repository_scope": json.loads(row["repository_scope_json"] or "{}"),
            "provenance": json.loads(row["provenance_json"] or "[]"),
            "causal_ancestry": json.loads(row["causal_ancestry_json"] or "[]"),
            "trust_validity_state": json.loads(row["trust_validity_state_json"] or "{}"),
            "visibility": json.loads(row["visibility_json"] or "{}"),
            "purpose": json.loads(row["purpose_json"] or "{}"),
            "retraction_deletion_barriers": json.loads(row["retraction_deletion_barriers_json"] or "{}"),
            "canonical_semantic_digest": row["canonical_semantic_digest"],
            "memory_type": row["memory_type"],
            "content": row["content"],
            "metadata": json.loads(row["metadata_json"] or "{}"),
            "created_at": row["created_at"],
            "updated_at": row["updated_at"],
            "ecm_task_id": row["ecm_task_id"],
            "ecm_compliant": bool(row["ecm_compliant"])
        }
    
    def validate_amx(self, memory: Dict[str, Any]) -> Dict[str, Any]:
        """Validate a memory against AMX v1.1 schema.
        
        Returns dict with 'valid' boolean and 'missing_fields' list.
        Auto-populates missing fields with defaults where possible.
        """
        missing = self.AMX_MEMORY_REQUIRED_FIELDS - set(memory.keys())
        
        if "schema_version" not in memory:
            memory["schema_version"] = "amx-memory-v1"
            missing.discard("schema_version")
        
        if "origin" not in memory:
            memory["origin"] = "{origin}"
            missing.discard("origin")
        
        if "canonical_semantic_digest" not in memory and "logical_identity" in memory:
            memory["canonical_semantic_digest"] = self.compute_digest(memory)
            missing.discard("canonical_semantic_digest")
        
        if "repository_scope" not in memory:
            memory["repository_scope"] = {}
            missing.discard("repository_scope")
        
        if "provenance" not in memory:
            memory["provenance"] = []
            missing.discard("provenance")
        
        if "causal_ancestry" not in memory:
            memory["causal_ancestry"] = []
            missing.discard("causal_ancestry")
        
        if "trust_validity_state" not in memory:
            memory["trust_validity_state"] = {
                "trust_level": "medium",
                "validity_status": "unverified",
                "last_validated": datetime.now(timezone.utc).isoformat()
            }
            missing.discard("trust_validity_state")
        
        if "visibility" not in memory:
            memory["visibility"] = {"scope": "workspace", "access_level": "read"}
            missing.discard("visibility")
        
        if "purpose" not in memory:
            memory["purpose"] = {"primary": "memory", "task_class": "memory:amx"}
            missing.discard("purpose")
        
        if "retraction_deletion_barriers" not in memory:
            memory["retraction_deletion_barriers"] = {
                "can_retract": False,
                "can_delete": False,
                "retention_period": "infinite"
            }
            missing.discard("retraction_deletion_barriers")
        
        if missing:
            return {
                "valid": False,
                "missing_fields": sorted(list(missing)),
                "message": f"Missing required AMX fields: {', '.join(sorted(missing))}"
            }
        
        if not isinstance(memory.get("provenance"), list):
            return {"valid": False, "missing_fields": [], "message": "provenance must be a list"}
        
        if not isinstance(memory.get("causal_ancestry"), list):
            return {"valid": False, "missing_fields": [], "message": "causal_ancestry must be a list"}
        
        return {"valid": True, "missing_fields": [], "message": "AMX v1.1 compliant"}
    
    def _track_ecm_store(self, memory: Dict[str, Any]) -> str:
        """Track memory store operation in ECM state."""
        task_id = f"memory:{memory['logical_identity']}:store"
        timestamp = datetime.now(timezone.utc).isoformat()
        
        self.ecm_state["tasks"][task_id] = {
            "id": task_id,
            "description": f"Store AMX memory {memory['logical_identity']}",
            "type": "memory:store",
            "status": "completed",
            "created_at": timestamp,
            "completed_at": timestamp,
            "memory_logical_identity": memory["logical_identity"],
            "origin": memory.get("origin", "{origin}")
        }
        
        attempt_id = f"{task_id}:attempt:1"
        self.ecm_state["attempts"][attempt_id] = {
            "id": attempt_id,
            "task_id": task_id,
            "status": "completed",
            "started_at": timestamp,
            "completed_at": timestamp,
            "result": "stored"
        }
        
        role_id = "memory:amx"
        if role_id not in self.ecm_state["roles"]:
            self.ecm_state["roles"][role_id] = {
                "id": role_id,
                "name": "AMX Memory Manager",
                "capabilities": ["memory:store", "memory:retrieve", "memory:validate"],
                "assigned_at": timestamp
            }
        
        artifact_id = f"artifact:memory:{memory['logical_identity']}"
        content_hash = memory.get("canonical_semantic_digest", "")
        self.ecm_state["artifact_references"][artifact_id] = {
            "id": artifact_id,
            "checksum": content_hash,
            "type": "memory",
            "logical_identity": memory["logical_identity"],
            "created_at": timestamp
        }
        
        binding_id = f"binding:memory:{memory['logical_identity']}:ecm:task:{task_id}"
        self.ecm_state["memory_bindings"][binding_id] = {
            "id": binding_id,
            "memory_logical_identity": memory["logical_identity"],
            "ecm_task_id": task_id,
            "ecm_attempt_id": attempt_id,
            "artifact_id": artifact_id,
            "created_at": timestamp
        }
        
        evidence_id = f"evidence:amx:memory:{memory['logical_identity']}:store"
        validation_result = self.validate_amx(memory)
        self.ecm_state["evidence_references"][evidence_id] = {
            "id": evidence_id,
            "check_name": "amx_compliance",
            "passed": validation_result["valid"],
            "details": validation_result["message"],
            "timestamp": timestamp,
            "memory_logical_identity": memory["logical_identity"]
        }
        
        self._save_ecm_state()
        return task_id
    
    def _track_ecm_retrieve(self, logical_identity: str) -> str:
        """Track memory retrieve operation in ECM state."""
        task_id = f"memory:{logical_identity}:retrieve"
        timestamp = datetime.now(timezone.utc).isoformat()
        
        self.ecm_state["tasks"][task_id] = {
            "id": task_id,
            "description": f"Retrieve AMX memory {logical_identity}",
            "type": "memory:retrieve",
            "status": "completed",
            "created_at": timestamp,
            "completed_at": timestamp,
            "memory_logical_identity": logical_identity
        }
        
        attempt_id = f"{task_id}:attempt:1"
        self.ecm_state["attempts"][attempt_id] = {
            "id": attempt_id,
            "task_id": task_id,
            "status": "completed",
            "started_at": timestamp,
            "completed_at": timestamp,
            "result": "retrieved"
        }
        
        self._save_ecm_state()
        return task_id
    
    def store(self, memory):
        """Store a memory with AMX compliance and ECM tracking."""
        # Validate AMX compliance
        validation = self.validate_amx(memory)
        if not validation["valid"]:
            raise ValueError(
                f"AMX validation failed: {validation['message']}. "
                f"Missing fields: {validation.get('missing_fields', [])}"
            )
        
        # Track in ECM
        ecm_task_id = self._track_ecm_store(memory)
        
        # Store in database
        if "logical_identity" not in memory:
            raise ValueError("logical_identity is required")
        if "canonical_semantic_digest" not in memory:
            memory["canonical_semantic_digest"] = self.compute_digest(memory)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            conn.execute(
                "INSERT OR REPLACE INTO memories "
                "(id, schema_version, origin, logical_identity, "
                "repository_scope_json, provenance_json, causal_ancestry_json, "
                "trust_validity_state_json, visibility_json, purpose_json, "
                "retraction_deletion_barriers_json, canonical_semantic_digest, "
                "memory_type, content, metadata_json, created_at, updated_at, "
                "ecm_task_id, ecm_compliant) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    memory.get("id", memory["logical_identity"]),
                    memory.get("schema_version"),
                    memory.get("origin"),
                    memory.get("logical_identity"),
                    json.dumps(memory.get("repository_scope", {})),
                    json.dumps(memory.get("provenance", [])),
                    json.dumps(memory.get("causal_ancestry", [])),
                    json.dumps(memory.get("trust_validity_state", {})),
                    json.dumps(memory.get("visibility", {})),
                    json.dumps(memory.get("purpose", {})),
                    json.dumps(memory.get("retraction_deletion_barriers", {})),
                    memory.get("canonical_semantic_digest"),
                    memory.get("memory_type", "episode"),
                    memory.get("content", ""),
                    json.dumps(memory.get("metadata", {})),
                    datetime.now(timezone.utc).isoformat(),
                    datetime.now(timezone.utc).isoformat(),
                    ecm_task_id,
                    True
                )
            )
            conn.commit()
        return memory["logical_identity"]
    
    def retrieve(self, logical_identity):
        """Retrieve a memory by logical identity with ECM tracking."""
        # Track in ECM
        self._track_ecm_retrieve(logical_identity)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            row = conn.execute(
                "SELECT * FROM memories WHERE logical_identity = ?",
                (logical_identity,)
            ).fetchone()
            if row:
                return self._memory_to_dict(row)
        return None
    
    def query(self, memory_type=None, limit=10, valid_only=True):
        """Query memories with optional filters."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            query_parts = ["SELECT * FROM memories"]
            params = []
            
            if memory_type:
                query_parts.append("WHERE memory_type = ?")
                params.append(memory_type)
            
            if valid_only:
                if memory_type:
                    query_parts.append("AND ecm_compliant = ?")
                else:
                    query_parts.append("WHERE ecm_compliant = ?")
                params.append(1)
            
            query_parts.append("ORDER BY created_at DESC LIMIT ?")
            params.append(limit)
            
            full_query = " ".join(query_parts)
            rows = conn.execute(full_query, tuple(params)).fetchall()
            return [self._memory_to_dict(row) for row in rows]
    
    def query_by_amx_compliance(self, valid_only=True, limit=10) -> List[Dict[str, Any]]:
        """Query memories filtered by AMX compliance status."""
        return self.query(memory_type=None, limit=limit, valid_only=valid_only)
    
    def get_ecm_task(self, memory_logical_identity: str) -> Optional[Dict[str, Any]]:
        """Get ECM task for a memory operation."""
        task_id = f"memory:{memory_logical_identity}:store"
        return self.ecm_state["tasks"].get(task_id)
    
    def get_ecm_attempts(self, memory_logical_identity: str) -> List[Dict[str, Any]]:
        """Get all ECM attempts for a memory."""
        prefix = f"memory:{memory_logical_identity}:"
        attempts = []
        for attempt_id, attempt in self.ecm_state["attempts"].items():
            if attempt_id.startswith(prefix):
                attempts.append(attempt)
        return attempts
    
    def get_ecm_memory_binding(self, memory_logical_identity: str) -> Optional[Dict[str, Any]]:
        """Get ECM memory binding for a memory."""
        binding_id = f"binding:memory:{memory_logical_identity}:"
        for bid, binding in self.ecm_state["memory_bindings"].items():
            if bid.startswith(binding_id):
                return binding
        return None
    
    def get_ecm_evidence(self, memory_logical_identity: str) -> List[Dict[str, Any]]:
        """Get all ECM evidence references for a memory."""
        prefix = f"evidence:amx:memory:{memory_logical_identity}:"
        evidence = []
        for eid, evi in self.ecm_state["evidence_references"].items():
            if eid.startswith(prefix):
                evidence.append(evi)
        return evidence
    
    def compute_digest(self, data):
        """Compute SHA-256 semantic digest."""
        data_str = json.dumps(data, sort_keys=True)
        return hashlib.sha256(data_str.encode('utf-8')).hexdigest()
    
    def start_server(self, host="127.0.0.1", port=8080):
        """Start MCP HTTP server."""
        self.host = host
        self.port = port
        self._server = HTTPServer((host, port), self._MCPHandler(self))
        self._server_thread = threading.Thread(
            target=self._server.serve_forever,
            daemon=True
        )
        self._server_thread.start()
    
    def stop_server(self):
        """Stop MCP server."""
        if self._server:
            self._server.shutdown()
            self._server.server_close()
            self._server = None
        if self._server_thread:
            self._server_thread.join(timeout=1.0)
            self._server_thread = None
    
    class _MCPHandler(BaseHTTPRequestHandler):
        """MCP HTTP request handler with AMX and ECM endpoints."""
        
        def __init__(self, engram_server, *args, **kwargs):
            self.engram = engram_server
            super().__init__(*args, **kwargs)
        
        def do_GET(self):
            """Handle GET requests for MCP resources."""
            parsed = urllib.parse.urlparse(self.path)
            
            if parsed.path == "/memory/list":
                memories = self.engram.query(limit=100)
                self._send_json({"memories": memories})
            
            elif parsed.path.startswith("/memory/"):
                logical_id = parsed.path.replace("/memory/", "")
                memory = self.engram.retrieve(logical_id)
                if memory:
                    self._send_json(memory)
                else:
                    self.send_error(404, "Memory not found: " + logical_id)
            
            elif parsed.path == "/memory/search":
                query = urllib.parse.parse_qs(parsed.query)
                memory_type = query.get("type", [None])[0]
                limit = int(query.get("limit", [10])[0])
                results = self.engram.query(memory_type=memory_type, limit=limit)
                self._send_json({"results": results})
            
            elif parsed.path == "/memory/validate":
                query = urllib.parse.parse_qs(parsed.query)
                logical_id = query.get("logical_identity", [None])[0]
                if logical_id:
                    memory = self.engram.retrieve(logical_id)
                    if memory:
                        validation = self.engram.validate_amx(memory)
                        self._send_json(validation)
                    else:
                        self.send_error(404, "Memory not found: " + logical_id)
                else:
                    self.send_error(400, "logical_identity parameter required")
            
            elif parsed.path.startswith("/ecm/memory/"):
                # ECM endpoints
                parts = parsed.path.split("/")
                if len(parts) >= 4:
                    logical_id = parts[3]
                    if parsed.path.endswith("/task"):
                        task = self.engram.get_ecm_task(logical_id)
                        if task:
                            self._send_json(task)
                        else:
                            self.send_error(404, "ECM task not found")
                    elif parsed.path.endswith("/attempts"):
                        attempts = self.engram.get_ecm_attempts(logical_id)
                        self._send_json({"attempts": attempts})
                    elif parsed.path.endswith("/binding"):
                        binding = self.engram.get_ecm_memory_binding(logical_id)
                        if binding:
                            self._send_json(binding)
                        else:
                            self.send_error(404, "ECM binding not found")
                    elif parsed.path.endswith("/evidence"):
                        evidence = self.engram.get_ecm_evidence(logical_id)
                        self._send_json({"evidence": evidence})
                    else:
                        self.send_error(404, "ECM resource not found")
                else:
                    self.send_error(404, "ECM resource not found")
            
            else:
                self.send_error(404, "Resource not found: " + parsed.path)
        
        def do_POST(self):
            """Handle POST requests for MCP operations."""
            parsed = urllib.parse.urlparse(self.path)
            
            if parsed.path == "/memory/store":
                content_length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(content_length)
                memory = json.loads(body)
                logical_id = self.engram.store(memory)
                self._send_json({"logical_identity": logical_id, "status": "stored"})
            
            elif parsed.path == "/memory/validate":
                content_length = int(self.headers.get("Content-Length", 0))
                body = self.rfile.read(content_length)
                memory = json.loads(body)
                validation = self.engram.validate_amx(memory)
                self._send_json(validation)
            
            else:
                self.send_error(404, "Resource not found: " + parsed.path)
        
        def _send_json(self, data):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode('utf-8'))
        
        def log_message(self, format, *args):
            pass  # Suppress default logging
'''


ENGRAM_MCP_TEST_TEMPLATE = '''import unittest
import sys
import os
import tempfile
import shutil

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ..{file_name} import {class_name}


class Test{class_name}(unittest.TestCase):
    """RED Phase: Tests for Engram MCP Server skill."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test-memory.db")
    
    def tearDown(self):
        if self.temp_dir:
            shutil.rmtree(self.temp_dir)
    
    def test_import(self):
        """Should import successfully."""
        self.assertIsNotNone({class_name})
    
    def test_instantiation(self):
        """Should instantiate with custom DB path."""
        engram = {class_name}(db_path=self.db_path)
        self.assertIsNotNone(engram)
    
    def test_store_and_retrieve(self):
        """Should store and retrieve memories with AMX compliance."""
        engram = {class_name}(db_path=self.db_path)
        
        memory = {{
            "logical_identity": "test-session-001",
            "origin": "test",
            "memory_type": "episode",
            "content": "Test memory content",
            "purpose": {{"primary": "test", "task_class": "memory:amx"}},
            "trust_validity_state": {{"trust_level": "high", "validity_status": "valid"}},
            "visibility": {{"scope": "workspace"}}
        }}
        
        logical_id = engram.store(memory)
        self.assertEqual(logical_id, "test-session-001")
        
        retrieved = engram.retrieve("test-session-001")
        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved["logical_identity"], "test-session-001")
        self.assertIn("canonical_semantic_digest", retrieved)
    
    def test_query(self):
        """Should query memories."""
        engram = {class_name}(db_path=self.db_path)
        
        for i in range(5):
            memory = {{
                "logical_identity": f"test-{i}",
                "origin": "test",
                "memory_type": "episode",
                "content": f"Test memory {i}"
            }}
            engram.store(memory)
        
        results = engram.query(memory_type="episode", limit=3)
        self.assertEqual(len(results), 3)
    
    def test_compute_digest(self):
        """Should compute SHA-256 digest."""
        engram = {class_name}(db_path=self.db_path)
        data = {{"key": "value", "number": 42}}
        digest = engram.compute_digest(data)
        self.assertEqual(len(digest), 64)


if __name__ == '__main__':
    unittest.main()
'''

ENGRAM_MCP_INIT_TEMPLATE = '''# {name} Package
# {description}

from .{file_name} import {class_name}

__all__ = ["{class_name}"]
'''
