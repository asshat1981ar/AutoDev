---
name: amx-ecm-integration
description: |
  Load this skill when mapping or integrating AMX memory systems with ECM collaboration tracking and Vibe.
  
  Provides end-to-end architecture mapping for:
  - AMX v1.1 canonical portable memory
  - ECM collaboration state tracking
  - MCP server interfaces
  - Vibe skill integration
  - AutoDev project constraints
  
  Use this when the user needs to understand how AMX, ECM, MCP, and Vibe integrate together.
model: mistral-medium-3.5
user-invocable: true
---

# AMX/ECM/Vibe End-to-End Integration Guide

**Purpose:** Complete architecture mapping and integration patterns for AMX memory, ECM collaboration tracking, MCP servers, and Vibe skills within AutoDev constraints.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Vibe Agent Runtime                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   User       │    │   Skills     │    │    MCP Servers    │  │
│  │   Request    │───▶│   System     │───▶│    (External)     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│           │                  │                     │             │
│           └──────────────────┴─────────────────────┘             │
│                              │                                     │
│                    ┌─────────┴─────────┐                          │
│                    │   Tool Registry    │                          │
│                    └─────────┬─────────┘                          │
│                              │                                     │
│         ┌────────────────────┼────────────────────┐              │
│         │                    │                    │              │
│   ┌─────▼─────┐      ┌─────▼─────┐        ┌─────▼─────┐        │
│   │ AMX Memory│      │ ECM State │        │ Engram MCP │        │
│   │ Component │      │ Component │        │  Server    │        │
│   └─────┬─────┘      └─────┬─────┘        └─────┬─────┘        │
│         │                    │                    │              │
│   ┌─────▼─────┐      ┌─────▼─────┐        ┌─────▼─────┐        │
│   │ SQLite    │      │ JSON      │        │ HTTP      │        │
│   │ Storage   │      │ Storage   │        │ Interface │        │
│   └───────────┘      └───────────┘        └───────────┘        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. AMX (Agent Memory eXchange) v1.1

**Purpose:** Canonical portable memory format for AI agents.

**Required Fields (11 total):**
```python
AMX_MEMORY_REQUIRED_FIELDS = {
    "schema_version",           # Always "amx-memory-v1"
    "origin",                  # Source identifier
    "logical_identity",        # Unique memory ID
    "repository_scope",        # Repository/worktree/project scope
    "provenance",              # List of provenance entries
    "causal_ancestry",         # List of ancestor references
    "trust_validity_state",    # Trust/validity metadata
    "visibility",              # Access control scope
    "purpose",                 # Intended use metadata
    "retraction_deletion_barriers",  # Deletion controls
    "canonical_semantic_digest"    # SHA-256 hash
}
```

**Field Specifications:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| schema_version | string | Yes | "amx-memory-v1" | Schema version identifier |
| origin | string | Yes | None | Source of the memory |
| logical_identity | string | Yes | None | Unique identifier |
| repository_scope | dict | Yes | {} | Repository/worktree/project scope |
| provenance | list | Yes | [] | Causal history entries |
| causal_ancestry | list | Yes | [] | Ancestry chain references |
| trust_validity_state | dict | Yes | See below | Trust and validity metadata |
| visibility | dict | Yes | {"scope": "workspace", "access_level": "read"} | Access control |
| purpose | dict | Yes | {"primary": "memory", "task_class": "memory:amx"} | Intended use |
| retraction_deletion_barriers | dict | Yes | See below | Deletion controls |
| canonical_semantic_digest | string | Yes | Computed | SHA-256 hash |

**trust_validity_state defaults:**
```python
{
    "trust_level": "medium",
    "validity_status": "unverified",
    "last_validated": "<ISO-8601 timestamp>"
}
```

**retraction_deletion_barriers defaults:**
```python
{
    "can_retract": False,
    "can_delete": False,
    "retention_period": "infinite"
}
```

**Provenance Entry Structure:**
```python
{
    "source": "skill-name",
    "timestamp": "2026-08-21T00:00:00Z",
    "checksum": "sha256:...",
    "description": "Operation description"
}
```

### 2. ECM (Entity-Collaboration Model)

**Purpose:** Track collaboration history and coordination state.

**Tracked Entity Types (9 total):**
```python
ECM_ENTITY_TYPES = {
    "task",              # Collaboration task
    "attempt",           # Task attempt
    "role",              # Role assignment
    "role_lease",        # Role lease (time-bound)
    "message",           # Communication message
    "artifact_reference", # Artifact reference
    "evidence_reference", # Evidence reference
    "memory_binding",    # Memory to ECM binding
    "ContextView"        # Context view state
}
```

**ECM State Structure:**
```json
{
  "tasks": {
    "<task-id>": {
      "id": "<task-id>",
      "description": "<description>",
      "type": "<type>",
      "status": "<created/completed/failed>",
      "created_at": "<ISO-8601>",
      "completed_at": "<ISO-8601>",
      "memory_logical_identity": "<memory-id>",
      "origin": "<origin>"
    }
  },
  "attempts": {
    "<attempt-id>": {
      "id": "<attempt-id>",
      "task_id": "<task-id>",
      "status": "<created/completed/failed>",
      "started_at": "<ISO-8601>",
      "completed_at": "<ISO-8601>",
      "result": "<result>"
    }
  },
  "roles": {
    "<role-id>": {
      "id": "<role-id>",
      "name": "<name>",
      "capabilities": ["<capability>"],
      "assigned_at": "<ISO-8601>"
    }
  },
  "role_leases": {
    "<lease-id>": {
      "id": "<lease-id>",
      "role_id": "<role-id>",
      "assigned_to": "<assignee>",
      "expires_at": "<ISO-8601>",
      "granted_at": "<ISO-8601>"
    }
  },
  "messages": {
    "<message-id>": {
      "id": "<message-id>",
      "task_id": "<task-id>",
      "sender": "<sender>",
      "content": "<content>",
      "timestamp": "<ISO-8601>"
    }
  },
  "artifact_references": {
    "<artifact-id>": {
      "id": "<artifact-id>",
      "checksum": "<hash>",
      "type": "<type>",
      "logical_identity": "<memory-id>",
      "created_at": "<ISO-8601>"
    }
  },
  "evidence_references": {
    "<evidence-id>": {
      "id": "<evidence-id>",
      "check_name": "<check>",
      "passed": true/false,
      "details": "<details>",
      "timestamp": "<ISO-8601>",
      "memory_logical_identity": "<memory-id>"
    }
  },
  "memory_bindings": {
    "<binding-id>": {
      "id": "<binding-id>",
      "memory_logical_identity": "<memory-id>",
      "ecm_task_id": "<task-id>",
      "ecm_attempt_id": "<attempt-id>",
      "artifact_id": "<artifact-id>",
      "created_at": "<ISO-8601>"
    }
  },
  "context_views": {
    "<view-id>": {
      "id": "<view-id>",
      "memory_ids": ["<memory-id>"],
      "created_at": "<ISO-8601>",
      "updated_at": "<ISO-8601>"
    }
  }
}
```

### 3. MCP (Model Context Protocol)

**Purpose:** Standard protocol for AI agents to access external resources.

**MCP Server Types in AutoDev:**
```
1. Engram MCP Server
   - SQLite storage
   - AMX-compliant memory
   - ECM tracking
   - HTTP interface

2. GitHub App MCP
   - GitHub API integration
   - Repository access
   - Issue/PR management

3. Linear MCP
   - Linear API integration
   - Issue tracking
   - Project management
```

**MCP HTTP Endpoints (Engram):**
```
GET  /memory/list              # List all memories
GET  /memory/{id}              # Retrieve specific memory
GET  /memory/search            # Search memories (type, limit)
POST /memory/store            # Store new memory
GET  /memory/validate          # Validate AMX compliance
GET  /ecm/memory/{id}/task    # Get ECM task
GET  /ecm/memory/{id}/attempts # Get ECM attempts
GET  /ecm/memory/{id}/binding  # Get ECM binding
GET  /ecm/memory/{id}/evidence # Get ECM evidence
```

### 4. Vibe Skills System

**Skill Discovery Order:**
1. `skill_paths` from config.toml
2. `.vibe/skills/` (project)
3. `.agents/skills/` (project)
4. `~/.vibe/skills/` (user)
5. `~/.agents/skills/` (user)

**Skill Types in AutoDev:**
```python
class SkillType(Enum):
    AMX_MEMORY = "amx-memory"        # AMX memory skills
    ECM = "ecm"                      # ECM tracking skills
    EVIDENCE = "evidence"            # Evidence/verification skills
    HOST_ADAPTER = "host-adapter"    # Vibe tool extensions
    MCP_SERVER = "mcp-server"        # MCP server skills
    ENGRAM_MCP = "engram-mcp"        # Engram MCP skills
    GENERAL = "general"              # General purpose skills
```

## Integration Patterns

### Pattern 1: AMX + ECM in Engram MCP

```python
class EngramMCPServer:
    """Engram MCP Server with AMX compliance and ECM tracking."""
    
    def __init__(self):
        # AMX compliance
        self.AMX_REQUIRED_FIELDS = {...}
        
        # ECM state
        self.ecm_state = {
            "tasks": {},
            "attempts": {},
            "roles": {},
            # ... all ECM entity types
        }
        
        # Database
        self.db_path = ".vibe/engram-memory/memory.db"
        self.ecm_state_path = ".vibe/engram-memory/ecm-state.json"
    
    def store_memory(self, memory: Dict[str, Any]) -> str:
        """Store memory with AMX compliance and ECM tracking."""
        
        # 1. Validate AMX compliance
        validation = self.validate_amx(memory)
        if not validation["valid"]:
            raise ValueError(validation["message"])
        
        # 2. Auto-populate missing AMX fields
        memory = self._auto_populate_amx(memory)
        
        # 3. Compute canonical digest
        memory["canonical_semantic_digest"] = self.compute_digest(memory)
        
        # 4. Track in ECM
        ecm_task_id = self._track_ecm_store(memory)
        
        # 5. Store in database
        return self._store_in_db(memory, ecm_task_id)
    
    def validate_amx(self, memory: Dict[str, Any]) -> Dict[str, Any]:
        """Validate memory against AMX v1.1 schema."""
        missing = self.AMX_REQUIRED_FIELDS - set(memory.keys())
        
        # Auto-populate defaults
        for field, default in AMX_DEFAULTS.items():
            if field in missing:
                memory[field] = default
                missing.discard(field)
        
        # Type validation
        if not isinstance(memory.get("provenance"), list):
            return {"valid": False, "message": "provenance must be a list"}
        
        if not isinstance(memory.get("causal_ancestry"), list):
            return {"valid": False, "message": "causal_ancestry must be a list"}
        
        return {
            "valid": len(missing) == 0,
            "missing_fields": sorted(list(missing)),
            "message": "AMX v1.1 compliant" if not missing else 
                       f"Missing fields: {', '.join(sorted(missing))}"
        }
    
    def _track_ecm_store(self, memory: Dict[str, Any]) -> str:
        """Create ECM records for store operation."""
        timestamp = datetime.now(timezone.utc).isoformat()
        task_id = f"memory:{memory['logical_identity']}:store"
        
        # Create task
        self.ecm_state["tasks"][task_id] = {
            "id": task_id,
            "description": f"Store AMX memory {memory['logical_identity']}",
            "type": "memory:store",
            "status": "completed",
            "created_at": timestamp,
            "completed_at": timestamp,
            "memory_logical_identity": memory["logical_identity"],
            "origin": memory.get("origin", "engram-mcp")
        }
        
        # Create attempt
        attempt_id = f"{task_id}:attempt:1"
        self.ecm_state["attempts"][attempt_id] = {
            "id": attempt_id,
            "task_id": task_id,
            "status": "completed",
            "started_at": timestamp,
            "completed_at": timestamp,
            "result": "stored"
        }
        
        # Create role
        role_id = "memory:amx"
        if role_id not in self.ecm_state["roles"]:
            self.ecm_state["roles"][role_id] = {
                "id": role_id,
                "name": "AMX Memory Manager",
                "capabilities": ["memory:store", "memory:retrieve", "memory:validate"],
                "assigned_at": timestamp
            }
        
        # Create artifact reference
        artifact_id = f"artifact:memory:{memory['logical_identity']}"
        self.ecm_state["artifact_references"][artifact_id] = {
            "id": artifact_id,
            "checksum": memory.get("canonical_semantic_digest", ""),
            "type": "memory",
            "logical_identity": memory["logical_identity"],
            "created_at": timestamp
        }
        
        # Create memory binding
        binding_id = f"binding:memory:{memory['logical_identity']}:ecm:task:{task_id}"
        self.ecm_state["memory_bindings"][binding_id] = {
            "id": binding_id,
            "memory_logical_identity": memory["logical_identity"],
            "ecm_task_id": task_id,
            "ecm_attempt_id": attempt_id,
            "artifact_id": artifact_id,
            "created_at": timestamp
        }
        
        # Create evidence reference
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
```

### Pattern 2: MCP Server with AMX/ECM

```python
class AMXECMMCPServer(MCPServer):
    """MCP Server with full AMX/ECM integration."""
    
    def __init__(self):
        super().__init__()
        self.memory_store = EngramMCPServer()
    
    def handle_resource_read(self, uri: str) -> Dict[str, Any]:
        """Handle MCP resource read requests."""
        if uri.startswith("memory://"):
            # Extract logical identity from URI
            logical_id = uri.replace("memory://", "")
            memory = self.memory_store.retrieve(logical_id)
            
            # Track ECM retrieve operation
            self.memory_store._track_ecm_retrieve(logical_id)
            
            return {"memory": memory}
        
        elif uri.startswith("ecm://"):
            # Handle ECM resource
            parts = uri.split("/")
            entity_type = parts[2]  # memory, task, attempt, etc.
            entity_id = parts[3]
            
            if entity_type == "memory":
                ecm_data = self._get_ecm_for_memory(entity_id)
                return {"ecm": ecm_data}
        
        raise ValueError(f"Unknown resource URI: {uri}")
    
    def handle_resource_list(self, uri_prefix: str) -> List[str]:
        """Handle MCP resource list requests."""
        if uri_prefix == "memory://":
            memories = self.memory_store.query(limit=100)
            return [f"memory://{m['logical_identity']}" for m in memories]
        
        elif uri_prefix == "ecm://":
            tasks = list(self.memory_store.ecm_state["tasks"].keys())
            return [f"ecm://task/{task_id}" for task_id in tasks]
        
        return []
    
    def handle_tool_call(self, name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Handle MCP tool calls."""
        if name == "memory_store":
            # Validate AMX
            validation = self.memory_store.validate_amx(arguments.get("memory", {}))
            if not validation["valid"]:
                return {"error": validation["message"]}
            
            # Store with ECM tracking
            logical_id = self.memory_store.store(arguments["memory"])
            return {"logical_identity": logical_id, "status": "stored"}
        
        elif name == "memory_retrieve":
            memory = self.memory_store.retrieve(arguments["logical_identity"])
            return {"memory": memory} if memory else {"error": "Not found"}
        
        elif name == "amx_validate":
            validation = self.memory_store.validate_amx(arguments.get("memory", {}))
            return validation
        
        raise ValueError(f"Unknown tool: {name}")
```

### Pattern 3: Vibe Skill with AMX/ECM

```python
class AMXECMSkill:
    """Vibe skill with AMX memory and ECM tracking."""
    
    def __init__(self):
        self.memory_client = AMXMemoryClient()
        self.ecm_client = ECMClient()
    
    def create_session_memory(self, session_data: Dict[str, Any]) -> str:
        """Create AMX-compliant session memory with ECM tracking."""
        # Build AMX memory
        memory = {
            "schema_version": "amx-memory-v1",
            "origin": "vibe-session",
            "logical_identity": f"session:{uuid.uuid4()}",
            "repository_scope": {
                "repository_path": "/data/data/com.termux/files/home/AutoDev",
                "worktree_id": "primary"
            },
            "provenance": [{
                "source": "vibe-skill",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "description": "Session memory creation"
            }],
            "causal_ancestry": [],
            "trust_validity_state": {
                "trust_level": "high",
                "validity_status": "valid",
                "last_validated": datetime.now(timezone.utc).isoformat()
            },
            "visibility": {
                "scope": "session",
                "access_level": "read_write"
            },
            "purpose": {
                "primary": "session_memory",
                "task_class": "memory:session"
            },
            "retraction_deletion_barriers": {
                "can_retract": True,
                "can_delete": False,
                "retention_period": "session"
            },
            "memory_type": "session",
            "content": session_data
        }
        
        # Store with AMX/ECM
        logical_id = self.memory_client.store(memory)
        
        # Track in ECM
        self.ecm_client.track_session_start(
            memory_logical_identity=logical_id,
            session_data=session_data
        )
        
        return logical_id
    
    def end_session(self, session_id: str) -> None:
        """End session and update ECM."""
        # Update memory
        memory = self.memory_client.retrieve(session_id)
        if memory:
            memory["purpose"]["session_status"] = "ended"
            memory["trust_validity_state"]["last_validated"] = datetime.now(timezone.utc).isoformat()
            self.memory_client.store(memory)
        
        # Update ECM
        self.ecm_client.track_session_end(
            memory_logical_identity=session_id,
            status="completed"
        )
```

## AutoDev Integration Points

### 1. ForgeCore Trusted Execution

**Constraint:** Only `crates/forge-core` can perform untrusted I/O.

**Integration Pattern:**
```python
# Vibe skill must delegate to ForgeCore for execution
class SafeExecutionSkill:
    """Skill that respects ForgeCore boundaries."""
    
    def __init__(self):
        self.forge_core = ForgeCoreAdapter()
    
    def execute_workflow(self, workflow: Dict[str, Any]) -> Dict[str, Any]:
        """Execute workflow through ForgeCore."""
        # Create authorization grant
        grant = AuthorizationGrant(
            workspace="/data/data/com.termux/files/home/AutoDev",
            permissions=["read", "write"],
            scope=["crates/forge-core"]
        )
        
        # Delegate to ForgeCore
        result = self.forge_core.execute(workflow, grant)
        
        # Store result as AMX memory
        memory = self._create_result_memory(workflow, result)
        self.memory_client.store(memory)
        
        return result
```

### 2. Harness Drift Checking

**Integration with `scripts/check_harness_drift.py`:**
```python
class HarnessDriftChecker:
    """Check AMX/ECM compliance in harness drift checks."""
    
    def check_amx_compliance(self, file_path: str) -> List[Dict[str, Any]]:
        """Check AMX compliance in a file."""
        issues = []
        
        # Read file
        content = read_file(file_path)
        
        # Check for AMX memory records
        memories = self._extract_amx_memories(content)
        
        for memory in memories:
            # Validate AMX
            validation = self.memory_client.validate_amx(memory)
            if not validation["valid"]:
                issues.append({
                    "file": file_path,
                    "type": "amx_compliance",
                    "severity": "error",
                    "message": validation["message"],
                    "missing_fields": validation.get("missing_fields", [])
                })
        
        return issues
    
    def check_ecm_tracking(self, directory: str) -> List[Dict[str, Any]]:
        """Check ECM tracking in a directory."""
        issues = []
        
        # Check for ECM state file
        ecm_file = os.path.join(directory, ".vibe/engram-memory/ecm-state.json")
        if os.path.exists(ecm_file):
            ecm_state = json.loads(read_file(ecm_file))
            
            # Check required ECM entity types
            required_types = ["tasks", "attempts", "memory_bindings"]
            for entity_type in required_types:
                if entity_type not in ecm_state:
                    issues.append({
                        "file": ecm_file,
                        "type": "ecm_missing_entity",
                        "severity": "warning",
                        "message": f"Missing ECM entity type: {entity_type}"
                    })
        
        return issues
```

### 3. CI Integration

**Add to `.github/workflows/ci.yml`:**
```yaml
- name: Check AMX/ECM Compliance
  run: |
    python -c "
    from skills.amx_ecm_integration import AMXECMChecker
    checker = AMXECMChecker()
    
    # Check all Python files for AMX compliance
    import os
    for root, dirs, files in os.walk('.'):
        for file in files:
            if file.endswith('.py'):
                path = os.path.join(root, file)
                issues = checker.check_amx_compliance(path)
                if issues:
                    print(f'::error file={path}::AMX compliance issues found')
                    for issue in issues:
                        print(f'  - {issue[\"message\"]}')
                    exit(1)
    
    # Check ECM tracking
    ecm_issues = checker.check_ecm_tracking('.')
    if ecm_issues:
        print('::warning::ECM tracking issues found')
        for issue in ecm_issues:
            print(f'  - {issue[\"message\"]}')
    "
```

## End-to-End Flow Examples

### Example 1: User Request to Skill Action

```
User: "Store this memory: {\"content\": \"Hello\", \"type\": \"note\"}"

Vibe Flow:
1. Route to meta-creator skill (user-invocable)
2. meta-creator recognizes this as AMX memory request
3. meta-creator delegates to amx-ecm-integration skill
4. amx-ecm-integration validates AMX fields
5. amx-ecm-integration auto-populates missing fields
6. amx-ecm-integration computes canonical digest
7. amx-ecm-integration tracks in ECM
8. amx-ecm-integration stores in SQLite
9. amx-ecm-integration returns logical_identity
10. meta-creator returns success to user

Result:
- AMX memory stored with all 11 required fields
- ECM task, attempt, artifact, binding, evidence all created
- SQLite database updated
- ECM state file updated
- User receives: {"logical_identity": "memory:note:001", "status": "stored"}
```

### Example 2: MCP Client Memory Query

```
MCP Client Request:
  Method: GET
  URI: memory://note-001

Vibe Flow:
1. MCP server (engram-mcp) receives request
2. MCP server delegates to EngramMCPServer
3. EngramMCPServer tracks ECM retrieve operation
4. EngramMCPServer queries SQLite database
5. EngramMCPServer converts row to AMX memory dict
6. EngramMCPServer returns memory to MCP server
7. MCP server returns to client

Response:
{
  "memory": {
    "schema_version": "amx-memory-v1",
    "origin": "engram-mcp",
    "logical_identity": "note-001",
    "repository_scope": {...},
    "provenance": [...],
    "causal_ancestry": [],
    "trust_validity_state": {...},
    "visibility": {...},
    "purpose": {...},
    "retraction_deletion_barriers": {...},
    "canonical_semantic_digest": "sha256:...",
    "memory_type": "note",
    "content": "Hello",
    "ecm_task_id": "memory:note-001:retrieve"
  }
}

ECM State Updated:
- New task: memory:note-001:retrieve
- New attempt: memory:note-001:retrieve:attempt:1
```

### Example 3: Skill Creation with AMX/ECM

```
User: "/skill-creator create amx-powered-tool"

Vibe Flow:
1. skill-creator skill loaded
2. skill-creator prompts for skill details
3. User provides: name, description, purpose
4. skill-creator generates skill directory
5. skill-creator uses ENGRAM_MCP_IMPL_TEMPLATE
6. Template includes AMX validation
7. Template includes ECM tracking
8. skill-creator writes SKILL.md, implementation.py, tests
9. skill-creator returns success

Generated Files:
.vibe/skills/amx-powered-tool/
├── SKILL.md              # With AMX/ECM documentation
├── __init__.py           # Package init
├── amx_tool.py           # With AMX_REQUIRED_FIELDS
└── tests/
    └── test_amx_tool.py   # With AMX validation tests

Implementation includes:
- AMX_MEMORY_REQUIRED_FIELDS constant
- validate_amx() method
- _track_ecm_store() method
- _track_ecm_retrieve() method
- SQLite storage with AMX schema
```

## Verification and Testing

### Unit Tests

```python
import unittest
from skills.engram_memory.engram_mcp import EngramMemory

class TestAMXECMIntegration(unittest.TestCase):
    """Test AMX/ECM integration."""
    
    def setUp(self):
        self.engram = EngramMemory(
            db_path=":memory:",
            ecm_state_path=":memory:"
        )
    
    def test_amx_validation(self):
        """Test AMX validation."""
        # Test with all fields
        memory = {
            "schema_version": "amx-memory-v1",
            "origin": "test",
            "logical_identity": "test-001",
            "repository_scope": {},
            "provenance": [],
            "causal_ancestry": [],
            "trust_validity_state": {},
            "visibility": {},
            "purpose": {},
            "retraction_deletion_barriers": {},
            "canonical_semantic_digest": ""
        }
        result = self.engram.validate_amx(memory)
        self.assertTrue(result["valid"])
        
        # Test with missing fields
        incomplete = {"logical_identity": "test-002"}
        result = self.engram.validate_amx(incomplete)
        self.assertFalse(result["valid"])
        self.assertIn("schema_version", result["missing_fields"])
    
    def test_ecm_tracking(self):
        """Test ECM tracking."""
        memory = {
            "logical_identity": "test-003",
            "content": "test"
        }
        # Auto-populate AMX fields
        self.engram.validate_amx(memory)
        
        # Store (triggers ECM tracking)
        self.engram.store(memory)
        
        # Check ECM state
        task_id = f"memory:test-003:store"
        self.assertIn(task_id, self.engram.ecm_state["tasks"])
        
        attempt_id = f"{task_id}:attempt:1"
        self.assertIn(attempt_id, self.engram.ecm_state["attempts"])
    
    def test_store_and_retrieve(self):
        """Test store and retrieve."""
        memory = {
            "logical_identity": "test-004",
            "content": "Hello World",
            "memory_type": "note"
        }
        
        # Store
        logical_id = self.engram.store(memory)
        self.assertEqual(logical_id, "test-004")
        
        # Retrieve
        retrieved = self.engram.retrieve(logical_id)
        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved["logical_identity"], logical_id)
        self.assertEqual(retrieved["content"], "Hello World")
        self.assertIn("canonical_semantic_digest", retrieved)
```

### Integration Tests

```python
class TestAMXECMIntegration(unittest.TestCase):
    """Integration tests for AMX/ECM/Vibe."""
    
    def test_end_to_end_flow(self):
        """Test complete flow from user request to stored memory."""
        # Simulate user request
        user_memory = {
            "content": "Test content",
            "memory_type": "user_note"
        }
        
        # Process through Vibe
        skill = AMXECMSkill()
        logical_id = skill.create_session_memory(user_memory)
        
        # Verify AMX compliance
        memory = skill.memory_client.retrieve(logical_id)
        self.assertTrue(skill.memory_client.validate_amx(memory)["valid"])
        
        # Verify ECM tracking
        ecm_task = skill.ecm_client.get_task(logical_id)
        self.assertIsNotNone(ecm_task)
        self.assertEqual(ecm_task["status"], "completed")
        
        # Verify binding
        binding = skill.ecm_client.get_memory_binding(logical_id)
        self.assertIsNotNone(binding)
        self.assertEqual(binding["memory_logical_identity"], logical_id)
```

## Performance Considerations

### Optimization Strategies

1. **Connection Pooling:**
```python
# Use connection pool for SQLite
from sqlite3 import Connection
from threading import local

connection_pool = local()

def get_connection(db_path: str) -> Connection:
    if not hasattr(connection_pool, 'connection'):
        connection_pool.connection = sqlite3.connect(db_path)
        connection_pool.connection.row_factory = sqlite3.Row
    return connection_pool.connection
```

2. **Batch Operations:**
```python
def batch_store(self, memories: List[Dict[str, Any]]) -> List[str]:
    """Store multiple memories in a batch."""
    logical_ids = []
    
    with sqlite3.connect(self.db_path) as conn:
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()
        
        for memory in memories:
            # Validate
            self.validate_amx(memory)
            
            # Track ECM
            ecm_task_id = self._track_ecm_store(memory)
            
            # Store
            cursor.execute(
                "INSERT OR REPLACE INTO memories VALUES (...)",
                (...)  # Parameters
            )
            logical_ids.append(memory["logical_identity"])
        
        conn.commit()
    
    return logical_ids
```

3. **Caching:**
```python
from functools import lru_cache

@lru_cache(maxsize=1000)
def retrieve_cached(self, logical_identity: str) -> Optional[Dict[str, Any]]:
    """Retrieve memory with caching."""
    return self._retrieve_from_db(logical_identity)
```

## Security Considerations

### Permission Boundaries

```python
class PermissionGuard:
    """Enforce permission boundaries."""
    
    ALLOWED_TOOLS = ["read_file", "write_file", "edit", "grep", "bash"]
    DENIED_PATHS = ["/etc", "/root", "/home"]
    
    @staticmethod
    def check_tool(tool_name: str) -> bool:
        """Check if tool is allowed."""
        return tool_name in PermissionGuard.ALLOWED_TOOLS
    
    @staticmethod
    def check_path(path: str) -> bool:
        """Check if path is allowed."""
        for denied in PermissionGuard.DENIED_PATHS:
            if path.startswith(denied):
                return False
        return True
    
    @staticmethod
    def guard_tool_call(tool_name: str, **kwargs) -> Any:
        """Guarded tool call."""
        if not PermissionGuard.check_tool(tool_name):
            raise PermissionError(f"Tool not allowed: {tool_name}")
        
        # Check path parameters
        for key, value in kwargs.items():
            if key.endswith("_path") and isinstance(value, str):
                if not PermissionGuard.check_path(value):
                    raise PermissionError(f"Path not allowed: {value}")
        
        # Call tool
        return getattr(tools, tool_name)(**kwargs)
```

### Input Validation

```python
def validate_memory_input(memory: Dict[str, Any]) -> None:
    """Validate memory input."""
    if not isinstance(memory, dict):
        raise ValueError("Memory must be a dictionary")
    
    if "logical_identity" in memory:
        if not isinstance(memory["logical_identity"], str):
            raise ValueError("logical_identity must be a string")
        if len(memory["logical_identity"]) > 256:
            raise ValueError("logical_identity too long (max 256 chars)")
    
    if "provenance" in memory:
        if not isinstance(memory["provenance"], list):
            raise ValueError("provenance must be a list")
        for entry in memory["provenance"]:
            if not isinstance(entry, dict):
                raise ValueError("provenance entries must be dictionaries")
    
    if "content" in memory:
        if not isinstance(memory["content"], (str, dict, list)):
            raise ValueError("content must be string, dict, or list")
```

## References

- [AMX Specification](AMX-v1.1.md) - Canonical portable memory format
- [ECM Specification](ECM-v1.1.md) - Collaboration history tracking
- [MCP Specification](https://github.com/modelcontextprotocol/spec) - Model Context Protocol
- [Vibe Documentation](https://github.com/mistralai/vibe) - Vibe agent runtime
- [AutoDev AGENTS.md](/data/data/com.termux/files/home/AutoDev/AGENTS.md) - Project constraints
- [Engram MCP Integration Plan](docs/superpowers/plans/2026-08-21-engram-mcp-integration.md) - Implementation details
