---
name: meta-creator
description: |
  Load this skill when the user requests guidance on creating new skills, MCP servers, tools, or plugins for Vibe.
  
  This is a meta-skill that provides comprehensive guidance on the entire lifecycle:
  - Skill creation and structure
  - MCP server implementation
  - Tool and plugin development
  - Vibe system setup and configuration
  - AMX/ECM integration patterns
  
  Use this when the user needs architectural guidance rather than direct implementation.
model: mistral-medium-3.5
user-invocable: true
---

# Meta-Creator Skill

**Purpose:** Guide the creation of Vibe skills, MCP servers, tools, and plugins with AMX/ECM compliance.

## When to Load

Use this skill when the user asks for:
- "How do I create a new skill?"
- "What's the best way to implement an MCP server?"
- "How do I integrate AMX memory into my tool?"
- "What's the Vibe system architecture?"
- "How do I set up my development environment?"

## Architecture Overview

### Vibe Skill System

Vibe skills are organized as follows:

```
.vibe/skills/
├── skill-name/
│   ├── SKILL.md          # Required: Frontmatter + Markdown instructions
│   ├── __init__.py       # Optional: Python package
│   ├── implementation.py # Optional: Code implementation
│   └── tests/            # Optional: Unit tests
```

**Discovery Order (first match wins):**
1. `skill_paths` from `config.toml`
2. `.vibe/skills/` (project scope)
3. `.agents/skills/` (project scope)
4. `~/.vibe/skills/` (user global)
5. `~/.agents/skills/` (user global)

### AMX/ECM Compliance Requirements

All durable memory must retain:
- **origin** - Source of the memory
- **logical_identity** - Unique identifier
- **repository_scope** - Repository/worktree/project scope
- **provenance** - Causal history list
- **causal_ancestry** - Ancestry chain list
- **trust_validity_state** - Trust/validity metadata
- **visibility** - Access control scope
- **purpose** - Intended use
- **retraction_deletion_barriers** - Deletion controls
- **canonical_semantic_digest** - SHA-256 hash

## Skill Creation Workflow

### 1. Planning Phase

**REQUIRED:** Create or update a durable implementation plan (ExecPlan).

```markdown
# ExecPlan: [Skill Name]

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development

**Goal:** [Clear, specific goal]

**Architecture:** [Architecture description]

**Tech Stack:** [Technologies used]

**AMCX-1 Compliance:** [How AMX/ECM is handled]

---

## Global Constraints

- [Constraint 1]
- [Constraint 2]

---

## File map

- Create: [file path] — [purpose]
- Modify: [file path] — [changes]

---

## Progress

### Milestone 1: [Name]

**Status:** [COMPLETED/PENDING/IN_PROGRESS]

**Acceptance Criteria:**
- [Criterion 1]
- [Criterion 2]

**Evidence:**
- [Proof 1]
- [Proof 2]

**Completed:** [Date]
```

### 2. Design Phase

**For architectural decisions:**
1. Define exact files/interfaces expected to change
2. Divide work into independently verifiable vertical slices
3. Define acceptance evidence for each slice
4. Record dependencies between slices
5. Specify rollback path

**Recommended order:**
```
protocol/specification → schemas → reference implementation → host adapters → harness integration → CI → provider rendering
```

### 3. Implementation Phase (RED-GREEN-REFACTOR)

**For each behavioral change:**

**RED:**
- Add the smallest test expressing the desired behavior
- Run it
- Confirm it fails for the expected reason

**GREEN:**
- Implement the smallest change that satisfies the behavior
- Run the focused test
- Run affected tests

**REFACTOR:**
- Improve structure without changing behavior
- Rerun verification

**Critical Rule:** A test that was never observed failing before implementation does NOT establish regression protection.

### 4. Verification Phase

**Every change must:**
- Pass existing tests
- Pass new tests
- Maintain AMX/ECM compliance where applicable
- Respect Vibe permission boundaries

## MCP Server Implementation Guide

### Core Requirements

MCP servers must:
1. **Respect permission model:** Only use allowed tools (read_file, write_file, edit, grep, bash)
2. **No direct execution:** No subprocess, os.system, or direct command execution
3. **Stateless where possible:** Use HTTP server pattern for MCP interface
4. **Structured storage:** Use SQLite, JSON, or other structured formats

### Python MCP Server Template

```python
#!/usr/bin/env python3
"""MCP Server Implementation."""

import json
import sqlite3
from http.server import HTTPServer, BaseHTTPRequestHandler
from typing import Any, Dict, List, Optional

class MCPServer:
    """MCP Server with structured storage."""
    
    def __init__(self, db_path: str = ".vibe/mcp-state/memory.db"):
        self.db_path = db_path
        self._init_db()
    
    def _init_db(self):
        """Initialize database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS resources (
                    uri TEXT PRIMARY KEY,
                    content TEXT,
                    metadata_json TEXT
                )
            """)
            conn.commit()
    
    def start(self, host="127.0.0.1", port=8080):
        """Start MCP server."""
        server = HTTPServer((host, port), MCPHandler(self))
        server.serve_forever()

class MCPHandler(BaseHTTPRequestHandler):
    """Handle MCP requests."""
    
    def do_GET(self):
        """Handle resource reads."""
        # Implement MCP read operations
        pass
    
    def do_POST(self):
        """Handle resource writes."""
        # Implement MCP write operations
        pass
```

### AMX-Compliant MCP Server

```python
class AMXMCPServer(MCPServer):
    """MCP Server with AMX memory compliance."""
    
    AMX_REQUIRED_FIELDS = {
        "schema_version", "origin", "logical_identity", "repository_scope",
        "provenance", "causal_ancestry", "trust_validity_state", "visibility",
        "purpose", "retraction_deletion_barriers", "canonical_semantic_digest"
    }
    
    def store_memory(self, memory: Dict[str, Any]):
        """Store AMX-compliant memory."""
        # Validate AMX compliance
        missing = self.AMX_REQUIRED_FIELDS - set(memory.keys())
        if missing:
            raise ValueError(f"Missing AMX fields: {missing}")
        
        # Auto-populate defaults
        memory.setdefault("schema_version", "amx-memory-v1")
        memory.setdefault("provenance", [])
        memory.setdefault("causal_ancestry", [])
        
        # Compute digest
        memory["canonical_semantic_digest"] = self._compute_digest(memory)
        
        # Store in database
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                "INSERT INTO memories VALUES (?, ?, ?)",
                (memory["logical_identity"], json.dumps(memory), 
                 memory["canonical_semantic_digest"])
            )
            conn.commit()
    
    def _compute_digest(self, data: Dict[str, Any]) -> str:
        """Compute SHA-256 semantic digest."""
        import hashlib
        data_str = json.dumps(data, sort_keys=True)
        return hashlib.sha256(data_str.encode('utf-8')).hexdigest()
```

## ECM Integration Guide

ECM (Entity-Collaboration Model) tracks:
- task
- attempt
- role
- role_lease
- message
- artifact_reference
- evidence_reference
- memory_binding
- ContextView

### ECM State Structure

```json
{
  "tasks": {
    "task-id": {
      "id": "task-id",
      "description": "Task description",
      "type": "memory:store",
      "status": "completed",
      "created_at": "2026-08-21T00:00:00Z",
      "completed_at": "2026-08-21T00:00:01Z"
    }
  },
  "attempts": {
    "attempt-id": {
      "id": "attempt-id",
      "task_id": "task-id",
      "status": "completed",
      "started_at": "2026-08-21T00:00:00Z",
      "completed_at": "2026-08-21T00:00:01Z",
      "result": "stored"
    }
  },
  "memory_bindings": {
    "binding-id": {
      "id": "binding-id",
      "memory_logical_identity": "memory-001",
      "ecm_task_id": "task-id",
      "ecm_attempt_id": "attempt-id"
    }
  }
}
```

### ECM Tracking Implementation

```python
def _track_ecm_operation(self, operation_type: str, memory: Dict[str, Any]):
    """Track operation in ECM state."""
    task_id = f"memory:{memory['logical_identity']}:{operation_type}"
    timestamp = datetime.now(timezone.utc).isoformat()
    
    # Create task
    self.ecm_state["tasks"][task_id] = {
        "id": task_id,
        "description": f"{operation_type} memory {memory['logical_identity']}",
        "type": f"memory:{operation_type}",
        "status": "completed",
        "created_at": timestamp,
        "completed_at": timestamp
    }
    
    # Create attempt
    attempt_id = f"{task_id}:attempt:1"
    self.ecm_state["attempts"][attempt_id] = {
        "id": attempt_id,
        "task_id": task_id,
        "status": "completed",
        "started_at": timestamp,
        "completed_at": timestamp,
        "result": operation_type
    }
    
    # Create memory binding
    binding_id = f"binding:memory:{memory['logical_identity']}:ecm:task:{task_id}"
    self.ecm_state["memory_bindings"][binding_id] = {
        "id": binding_id,
        "memory_logical_identity": memory["logical_identity"],
        "ecm_task_id": task_id,
        "ecm_attempt_id": attempt_id
    }
    
    self._save_ecm_state()
```

## Vibe System Setup Guide

### Prerequisites

**Required:**
- Python 3.10 or 3.11
- Rust stable toolchain (for Rust-based skills)
- Node 24 (for Node-based tools)

**Recommended:**
- Virtual environment for Python dependencies
- Trusted folder configuration

### Configuration Structure

```
.vibe/
├── config.toml          # Main configuration
├── skills/              # Project-scoped skills
│   └── skill-name/
│       └── SKILL.md
├── state.json           # Vibe state (optional)
└── logs/                # Log files
```

### config.toml Example

```toml
[general]
# Trusted folders for skill loading
trusted_folders = [
    "/data/data/com.termux/files/home/AutoDev"
]

[skills]
# Additional skill paths
skill_paths = [
    ".vibe/skills",
    ".agents/skills"
]

[permissions]
# Tool permissions
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash"]
```

## Tool/Plugin Development

### Tool Requirements

All tools must:
1. **Respect permission boundaries:** Only perform allowed operations
2. **No side effects:** Tools should not modify state without explicit permission
3. **Idempotent where possible:** Same input produces same output
4. **Error handling:** Graceful degradation on failures

### Python Tool Template

```python
"""Custom tool implementation."""

import json
from typing import Any, Dict

class CustomTool:
    """Custom tool with Vibe integration."""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
    
    def execute(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Execute the tool."""
        # Validate input
        required = ["param1", "param2"]
        for param in required:
            if param not in params:
                raise ValueError(f"Missing required parameter: {param}")
        
        # Execute logic
        result = self._internal_execute(params)
        
        # Return structured result
        return {
            "status": "success",
            "result": result,
            "metadata": {
                "tool": "custom_tool",
                "timestamp": datetime.now(timezone.utc).isoformat()
            }
        }
    
    def _internal_execute(self, params: Dict[str, Any]) -> Any:
        """Internal execution logic."""
        # Implement tool logic here
        pass
```

## Integration Patterns

### Pattern 1: AMX Memory in MCP Server

```python
class MemoryMCPServer(MCPServer):
    """MCP Server with AMX memory integration."""
    
    def __init__(self):
        super().__init__()
        self.memory_store = AMXMemoryStore()
        self.ecm_tracker = ECMStateTracker()
    
    def handle_memory_request(self, request):
        # Validate request
        # Track in ECM
        # Store with AMX compliance
        # Return result
        pass
```

### Pattern 2: ECM-Aware Tool

```python
class ECMAwareTool:
    """Tool with ECM tracking."""
    
    def __init__(self):
        self.ecm_state = ECMState()
    
    def execute(self, params):
        # Start ECM task
        task_id = self.ecm_state.start_task(
            description=params.get("description"),
            task_type=params.get("type")
        )
        
        try:
            # Execute tool logic
            result = self._execute_logic(params)
            
            # Complete ECM task
            self.ecm_state.complete_task(task_id, result=result)
            
            return result
        except Exception as e:
            # Fail ECM task
            self.ecm_state.fail_task(task_id, error=str(e))
            raise
```

## Best Practices

### 1. Skill Organization

- **Single responsibility:** One skill per capability
- **Clear naming:** Use hyphens, lowercase (e.g., `memory-manager`, not `MemoryManager`)
- **Documentation first:** SKILL.md must clearly state when to load
- **Minimal dependencies:** Prefer stdlib-only implementations

### 2. AMX/ECM Compliance

- **Always validate:** Check AMX fields on all memory operations
- **Auto-populate defaults:** Fill missing fields with sensible defaults
- **Track everything:** Every operation should have ECM tracking
- **Immutable digests:** Never modify canonical_semantic_digest after creation

### 3. Security

- **Never bypass boundaries:** Respect Vibe's permission model
- **No direct execution:** Use bash tool, not subprocess
- **Validate all inputs:** Don't trust user input
- **Sanitize outputs:** Remove sensitive data from responses

### 4. Testing

- **RED phase first:** Write failing test before implementation
- **Test edge cases:** Empty inputs, invalid data, error conditions
- **Verify AMX compliance:** Check all required fields in tests
- **Test ECM tracking:** Verify collaboration state is tracked correctly

## Checklist for New Skills

- [ ] SKILL.md created with proper frontmatter
- [ ] Description clearly states when to load
- [ ] Implementation follows AMX/ECM patterns if applicable
- [ ] Unit tests written (RED phase first)
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Integration tested

## Verification Commands

```bash
# Python syntax check
python -m py_compile skill_file.py

# Run unit tests
python -m unittest discover -s tests -v

# Check AMX compliance (if applicable)
python -c "from skill import Implementation; impl = Implementation(); print(impl.validate_amx())"

# Verify MCP server starts
python -c "from skill import MCPServer; s = MCPServer(); s.start()"
```

## Troubleshooting

### Common Issues

1. **Triple-quote conflicts:** Use single-quote (`'''`) for outer template strings
2. **SQLite row access:** Use `conn.row_factory = sqlite3.Row` for dict-like access
3. **Template substitution:** Use `{{` and `}}` to escape curly braces in templates
4. **Import errors:** Ensure all paths are relative and packages are properly structured

### Debugging Tips

1. **Check imports:** `python -c "import skill_module"`
2. **Verify paths:** Use absolute paths for file operations
3. **Test in isolation:** Run tests in clean environment
4. **Check permissions:** Ensure trusted folder configuration is correct

## References

- [Vibe Documentation](https://github.com/mistralai/vibe)
- [MCP Specification](https://github.com/modelcontextprotocol/python-sdk)
- [AMX Specification](AMX-v1.1.md)
- [ECM Specification](ECM-v1.1.md)
- [AutoDev AGENTS.md](/data/data/com.termux/files/home/AutoDev/AGENTS.md)
