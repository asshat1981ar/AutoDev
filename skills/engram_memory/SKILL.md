---
name: engram-memory
version: "1.0.0"
description: |
  Production Engram MCP Server for AI agent persistent memory

  Engram MCP Server - Persistent memory for AI agents.

  Provides:
  - SQLite-backed persistent memory storage
  - MCP server interface for agent access
  - Semantic search capabilities
  - Knowledge graph support
  - Memory lifecycle management

  AMCX-1 Compliance: Implements AMX memory component with MCP interface.
model: "mistral-large-latest"
preferred_tools: ["read_file", "write_file", "edit", "grep", "bash"]
temperature: 0.0
---

## engram-memory - Engram MCP Server

**Purpose:** Production Engram MCP Server for AI agent persistent memory

**AMCX-1 Compliance:** This skill implements Engram MCP server.

### Capabilities

- Persistent Memory Storage: SQLite-based with AMX compliance
- MCP Server: HTTP-based MCP protocol interface
- Semantic Search: Ready for vector embeddings
- Knowledge Graph: Extensible schema for relationships
- Memory Lifecycle: Automatic cleanup and organization

### Memory Schema

Conforms to AMX v1.1 plus MCP extensions:
- logical_identity: Unique memory identifier
- origin: Memory source
- repository_scope: Git repository context
- provenance: Memory lineage tracking
- canonical_semantic_digest: SHA-256 hash
- memory_type: categorization (episode, semantic, declarative, procedural)

### Usage

```python
from skills.engram-memory.engram_mcp import EngramMemory
engram = EngramMemory(db_path=".vibe/engram-memory/memory.db")
engram.start_server(host="127.0.0.1", port=8080)
```
