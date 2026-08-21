#!/usr/bin/env python3
"""Production Engram MCP Server for AI agent persistent memory

Engram MCP Server - Persistent memory with MCP interface.
Integrates with AMX memory patterns and AMCX-1 v1.1.
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


class EngramMemory:
    """Engram MCP Server with SQLite storage and AMX compliance."""
    
    def __init__(self, db_path: str = ".vibe/engram-memory/memory.db"):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_db()
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
            "updated_at TEXT NOT NULL)"
        )
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            conn.execute(create_table)
            conn.execute("CREATE INDEX IF NOT EXISTS idx_origin ON memories(origin)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_logical_identity ON memories(logical_identity)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_type ON memories(memory_type)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)")
            conn.commit()
    
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
            "updated_at": row["updated_at"]
        }
    
    def store(self, memory):
        """Store a memory with AMX compliance."""
        if "schema_version" not in memory:
            memory["schema_version"] = "amx-memory-v1"
        if "origin" not in memory:
            memory["origin"] = "vibe:skill-creator"
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
                "memory_type, content, metadata_json, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                    datetime.now(timezone.utc).isoformat()
                )
            )
            conn.commit()
        return memory["logical_identity"]
    
    def retrieve(self, logical_identity):
        """Retrieve a memory by logical identity."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            row = conn.execute(
                "SELECT * FROM memories WHERE logical_identity = ?",
                (logical_identity,)
            ).fetchone()
            if row:
                return self._memory_to_dict(row)
        return None
    
    def query(self, memory_type=None, limit=10):
        """Query memories with optional filters."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            if memory_type:
                rows = conn.execute(
                    "SELECT * FROM memories WHERE memory_type = ? "
                    "ORDER BY created_at DESC LIMIT ?",
                    (memory_type, limit)
                ).fetchall()
            else:
                rows = conn.execute(
                    "SELECT * FROM memories ORDER BY created_at DESC LIMIT ?",
                    (limit,)
                ).fetchall()
            return [self._memory_to_dict(row) for row in rows]
    
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
        """MCP HTTP request handler."""
        
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
            
            else:
                self.send_error(404, "Resource not found: " + parsed.path)
        
        def _send_json(self, data):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode('utf-8'))
        
        def log_message(self, format, *args):
            pass  # Suppress default logging
