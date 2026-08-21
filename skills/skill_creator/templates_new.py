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
"""{description}

Engram MCP Server - Persistent memory with MCP interface.
Integrates with AMX memory patterns and AMCX-1 v1.1.
Includes ECM collaboration state tracking for all memory operations.
"""

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
