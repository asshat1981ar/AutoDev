#!/usr/bin/env python3
"""
AMCX-1 ECM MCP Server

This module implements the AMCX-1 v1.1 Entity-Collaboration Model (ECM) as an MCP server,
exposing collaboration history and coordination state as MCP resources.

ECM owns collaboration history: roles, attempts, messages, leases, task coordination,
promotion evidence and ContextView history.

MCP Server Contract:
- Exposes ECM state via MCP resource endpoints
- Read-only by default (GET resources)
- Write operations via MCP tool calls
- Health check endpoint for monitoring
- Schema validation against ecm-state-v1.json
"""

import argparse
import copy
import json
import os
import threading
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from typing import Any, Optional
from urllib.parse import urlparse, parse_qs, unquote


# ECM Schema version
ECM_SCHEMA_VERSION = "ecm-state-v1"

# Default state path
DEFAULT_STATE_PATH = Path(".vibe/amcx-ecm/state.json")


class ECMValidationError(ValueError):
    """Raised when ECM state validation fails."""
    pass


class ECMState:
    """
    ECM Collaboration State Manager
    
    Manages the canonical ECM state including tasks, attempts, roles, messages,
    role leases, artifact references, evidence references, memory bindings,
    context views, and promotion candidates.
    
    State is persisted to a JSON file and validated against the ECM schema.
    """
    
    DEFAULT_STATE_TEMPLATE = {
        "schema_version": ECM_SCHEMA_VERSION,
        "ecm_id": "default-ecm",
        "tasks": {},
        "attempts": {},
        "roles": {},
        "messages": [],
        "role_leases": {},
        "artifact_references": {},
        "evidence_references": {},
        "memory_bindings": {},
        "context_views": {},
        "promotion_candidates": [],
        "timestamp": None,
    }
    
    def __init__(self, state_path: Optional[Path] = None, ecm_id: str = "autodev-main-ecm"):
        """
        Initialize ECM State Manager.
        
        Args:
            state_path: Path to the JSON state file. If None, uses DEFAULT_STATE_PATH.
            ecm_id: Unique identifier for this ECM instance.
        """
        self.state_path = state_path or DEFAULT_STATE_PATH
        self.ecm_id = ecm_id
        self.state = self._load_state()
        self._lock = threading.Lock()
    
    def _load_state(self) -> dict[str, Any]:
        """Load state from file or return default template."""
        if self.state_path.exists():
            try:
                content = self.state_path.read_text(encoding="utf-8")
                state = json.loads(content)
                # Validate schema version
                if state.get("schema_version") != ECM_SCHEMA_VERSION:
                    raise ECMValidationError(
                        f"Invalid schema version: {state.get('schema_version')}. "
                        f"Expected: {ECM_SCHEMA_VERSION}"
                    )
                return state
            except (json.JSONDecodeError, OSError) as e:
                raise ECMValidationError(f"Failed to load state: {e}")
        
        # Return deep copy of template to avoid shared mutable state
        template = copy.deepcopy(self.DEFAULT_STATE_TEMPLATE)
        template["ecm_id"] = self.ecm_id
        template["repository_context"] = {
            "repository_revision": os.environ.get("AMCX_REVISION", "unknown"),
            "repository_path": os.environ.get("AMCX_REPO_PATH", str(Path.cwd())),
            "branch": os.environ.get("AMCX_BRANCH", "main"),
            "worktree_id": os.environ.get("AMCX_WORKTREE", "primary"),
        }
        return template
    
    def _save_state(self) -> None:
        """Save state to file with timestamp update."""
        try:
            # Ensure parent directory exists
            self.state_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Update timestamp
            self.state["timestamp"] = datetime.now(timezone.utc).isoformat()
            
            content = json.dumps(self.state, indent=2, ensure_ascii=False)
            self.state_path.write_text(content, encoding="utf-8")
        except OSError as e:
            raise ECMValidationError(f"Failed to save state: {e}")
    
    def _generate_id(self, prefix: str = "ecm") -> str:
        """Generate a unique ID for ECM entities."""
        import uuid
        # Use lowercase hex with prefix
        return f"{prefix}-{uuid.uuid4().hex[:12]}"
    
    def _validate_entity_id(self, entity_id: str, entity_type: str) -> None:
        """Validate entity ID format."""
        import re
        pattern = r"^[a-z0-9]+(?:-[a-z0-9]+)*$"
        if not re.match(pattern, entity_id):
            raise ECMValidationError(
                f"Invalid {entity_type}_id format: {entity_id}. "
                f"Must match pattern: {pattern}"
            )
    
    # Task operations
    def add_task(self, task: dict[str, Any]) -> str:
        """
        Add a task to ECM state.
        
        Args:
            task: Task object conforming to ECM Task schema
            
        Returns:
            The task_id of the added task
        """
        with self._lock:
            # Set defaults for optional fields first
            if "created_timestamp" not in task:
                task["created_timestamp"] = datetime.now(timezone.utc).isoformat()
            if "updated_timestamp" not in task:
                task["updated_timestamp"] = task["created_timestamp"]
            if "objective" not in task:
                task["objective"] = task.get("title", "")
            
            # Validate required fields
            required_fields = ["task_id", "title", "status", "created_timestamp", "objective"]
            for field in required_fields:
                if field not in task:
                    raise ECMValidationError(f"Task missing required field: '{field}'")
            
            task_id = task["task_id"]
            self._validate_entity_id(task_id, "task")
            
            # Validate status
            valid_statuses = ["pending", "in_progress", "completed", "failed", "cancelled", "blocked"]
            if task.get("status") not in valid_statuses:
                raise ECMValidationError(
                    f"Invalid task status: {task.get('status')}. "
                    f"Must be one of: {valid_statuses}"
                )
            
            # Add to state
            self.state.setdefault("tasks", {})[task_id] = task
            self._save_state()
            return task_id
    
    def get_task(self, task_id: str) -> Optional[dict[str, Any]]:
        """
        Get a task by its ID.
        
        Args:
            task_id: The task ID to retrieve
            
        Returns:
            The task object if found, None otherwise
        """
        self._validate_entity_id(task_id, "task")
        return self.state.get("tasks", {}).get(task_id)
    
    def list_tasks(self, status: Optional[str] = None) -> list[dict[str, Any]]:
        """
        List all tasks, optionally filtered by status.
        
        Args:
            status: Optional status filter
            
        Returns:
            List of task objects
        """
        tasks = list(self.state.get("tasks", {}).values())
        if status:
            tasks = [t for t in tasks if t.get("status") == status]
        return tasks
    
    def update_task_status(self, task_id: str, status: str) -> None:
        """
        Update a task's status.
        
        Args:
            task_id: The task ID to update
            status: The new status
        """
        with self._lock:
            self._validate_entity_id(task_id, "task")
            task = self.state.get("tasks", {}).get(task_id)
            if not task:
                raise ECMValidationError(f"Task not found: {task_id}")
            
            valid_statuses = ["pending", "in_progress", "completed", "failed", "cancelled", "blocked"]
            if status not in valid_statuses:
                raise ECMValidationError(
                    f"Invalid status: {status}. Must be one of: {valid_statuses}"
                )
            
            task["status"] = status
            task["updated_timestamp"] = datetime.now(timezone.utc).isoformat()
            
            if status == "completed":
                task["completed_timestamp"] = task["updated_timestamp"]
            
            self._save_state()
    
    # Attempt operations
    def add_attempt(self, attempt: dict[str, Any]) -> str:
        """
        Add an attempt to ECM state.
        
        Args:
            attempt: Attempt object conforming to ECM Attempt schema
            
        Returns:
            The attempt_id of the added attempt
        """
        with self._lock:
            # Set defaults for optional fields first
            if "start_timestamp" not in attempt:
                attempt["start_timestamp"] = datetime.now(timezone.utc).isoformat()
            if "status" not in attempt:
                attempt["status"] = "pending"
            if "result" not in attempt:
                attempt["result"] = None
            if "replan_count" not in attempt:
                attempt["replan_count"] = 0
            if "evidence_ids" not in attempt:
                attempt["evidence_ids"] = []
            if "message_ids" not in attempt:
                attempt["message_ids"] = []
            if "artifact_ids" not in attempt:
                attempt["artifact_ids"] = []
            if "metadata" not in attempt:
                attempt["metadata"] = {}
            
            # Validate required fields
            required_fields = ["attempt_id", "task_id", "status", "start_timestamp", "agent_id"]
            for field in required_fields:
                if field not in attempt:
                    raise ECMValidationError(f"Attempt missing required field: '{field}'")
            
            attempt_id = attempt["attempt_id"]
            self._validate_entity_id(attempt_id, "attempt")
            self._validate_entity_id(attempt["task_id"], "task")
            
            # Validate status
            valid_statuses = ["pending", "in_progress", "completed", "failed", "cancelled"]
            if attempt.get("status") not in valid_statuses:
                raise ECMValidationError(
                    f"Invalid attempt status: {attempt.get('status')}. "
                    f"Must be one of: {valid_statuses}"
                )
            
            # Link to task
            task = self.state.get("tasks", {}).get(attempt["task_id"])
            if task:
                task.setdefault("attempt_ids", []).append(attempt_id)
            
            # Add to state
            self.state.setdefault("attempts", {})[attempt_id] = attempt
            self._save_state()
            return attempt_id
    
    def get_attempt(self, attempt_id: str) -> Optional[dict[str, Any]]:
        """Get an attempt by its ID."""
        self._validate_entity_id(attempt_id, "attempt")
        return self.state.get("attempts", {}).get(attempt_id)
    
    def list_attempts(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """List all attempts, optionally filtered by task_id."""
        attempts = list(self.state.get("attempts", {}).values())
        if task_id:
            self._validate_entity_id(task_id, "task")
            attempts = [a for a in attempts if a.get("task_id") == task_id]
        return attempts
    
    # Role operations
    def add_role(self, role: dict[str, Any]) -> str:
        """
        Add a role to ECM state.
        
        Args:
            role: Role object conforming to ECM Role schema
            
        Returns:
            The role_id of the added role
        """
        with self._lock:
            required_fields = ["role_id", "name", "capabilities"]
            for field in required_fields:
                if field not in role:
                    raise ECMValidationError(f"Role missing required field: '{field}'")
            
            role_id = role["role_id"]
            self._validate_entity_id(role_id, "role")
            
            # Set defaults
            role.setdefault("authorized_tools", [])
            role.setdefault("constraints", [])
            role.setdefault("max_concurrent", 1)
            role.setdefault("metadata", {})
            
            # Add to state
            self.state.setdefault("roles", {})[role_id] = role
            self._save_state()
            return role_id
    
    def get_role(self, role_id: str) -> Optional[dict[str, Any]]:
        """Get a role by its ID."""
        self._validate_entity_id(role_id, "role")
        return self.state.get("roles", {}).get(role_id)
    
    def list_roles(self) -> list[dict[str, Any]]:
        """List all roles."""
        return list(self.state.get("roles", {}).values())
    
    # Message operations
    def add_message(self, message: dict[str, Any]) -> str:
        """
        Add a message to ECM state.
        
        Args:
            message: Message object conforming to ECM Message schema
            
        Returns:
            The message_id of the added message
        """
        with self._lock:
            required_fields = ["message_id", "sender", "timestamp", "content_type", "body"]
            for field in required_fields:
                if field not in message:
                    raise ECMValidationError(f"Message missing required field: '{field}'")
            
            message_id = message["message_id"]
            self._validate_entity_id(message_id, "message")
            
            # Validate sender
            sender = message.get("sender", {})
            if not isinstance(sender, dict):
                raise ECMValidationError("Message sender must be an object")
            if "agent_id" not in sender:
                raise ECMValidationError("Message sender missing required field: 'agent_id'")
            if "role_id" not in sender:
                raise ECMValidationError("Message sender missing required field: 'role_id'")
            
            # Validate content_type
            valid_content_types = ["text", "markdown", "json", "yaml", "diff", "log"]
            if message.get("content_type") not in valid_content_types:
                raise ECMValidationError(
                    f"Invalid content_type: {message.get('content_type')}. "
                    f"Must be one of: {valid_content_types}"
                )
            
            # Set default timestamp
            if "timestamp" not in message:
                message["timestamp"] = datetime.now(timezone.utc).isoformat()
            
            # Add to state
            self.state.setdefault("messages", []).append(message)
            self._save_state()
            return message_id
    
    def get_message(self, message_id: str) -> Optional[dict[str, Any]]:
        """Get a message by its ID."""
        self._validate_entity_id(message_id, "message")
        for msg in self.state.get("messages", []):
            if msg.get("message_id") == message_id:
                return msg
        return None
    
    def list_messages(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """List all messages, optionally filtered by task_id."""
        messages = list(self.state.get("messages", []))
        if task_id:
            self._validate_entity_id(task_id, "task")
            messages = [m for m in messages if m.get("task_id") == task_id]
        return messages
    
    # Role Lease operations
    def add_role_lease(self, lease: dict[str, Any]) -> str:
        """
        Add a role lease to ECM state.
        
        Args:
            lease: RoleLease object conforming to ECM RoleLease schema
            
        Returns:
            The lease_id of the added lease
        """
        with self._lock:
            required_fields = ["lease_id", "role_id", "agent_id", "status", "acquired_timestamp"]
            for field in required_fields:
                if field not in lease:
                    raise ECMValidationError(f"RoleLease missing required field: '{field}'")
            
            lease_id = lease["lease_id"]
            self._validate_entity_id(lease_id, "lease")
            self._validate_entity_id(lease["role_id"], "role")
            
            # Validate status
            valid_statuses = ["pending", "active", "expired", "revoked", "released"]
            if lease.get("status") not in valid_statuses:
                raise ECMValidationError(
                    f"Invalid lease status: {lease.get('status')}. "
                    f"Must be one of: {valid_statuses}"
                )
            
            # Set default timestamp
            if "acquired_timestamp" not in lease:
                lease["acquired_timestamp"] = datetime.now(timezone.utc).isoformat()
            
            # Add to state
            self.state.setdefault("role_leases", {})[lease_id] = lease
            self._save_state()
            return lease_id
    
    def get_role_lease(self, lease_id: str) -> Optional[dict[str, Any]]:
        """Get a role lease by its ID."""
        self._validate_entity_id(lease_id, "lease")
        return self.state.get("role_leases", {}).get(lease_id)
    
    def list_role_leases(self, role_id: Optional[str] = None) -> list[dict[str, Any]]:
        """List all role leases, optionally filtered by role_id."""
        leases = list(self.state.get("role_leases", {}).values())
        if role_id:
            self._validate_entity_id(role_id, "role")
            leases = [l for l in leases if l.get("role_id") == role_id]
        return leases
    
    # Artifact reference operations
    def add_artifact_reference(self, artifact: dict[str, Any]) -> str:
        """
        Add an artifact reference to ECM state.
        
        Args:
            artifact: ArtifactReference object
            
        Returns:
            The artifact_id of the added artifact reference
        """
        with self._lock:
            required_fields = ["artifact_id", "uri", "artifact_type", "created_timestamp"]
            for field in required_fields:
                if field not in artifact:
                    raise ECMValidationError(f"ArtifactReference missing required field: '{field}'")
            
            artifact_id = artifact["artifact_id"]
            self._validate_entity_id(artifact_id, "artifact")
            
            # Validate artifact_type
            valid_types = ["file", "directory", "image", "log", "report", "dataset", "model", "config"]
            if artifact.get("artifact_type") not in valid_types:
                raise ECMValidationError(
                    f"Invalid artifact_type: {artifact.get('artifact_type')}. "
                    f"Must be one of: {valid_types}"
                )
            
            # Add to state
            self.state.setdefault("artifact_references", {})[artifact_id] = artifact
            self._save_state()
            return artifact_id
    
    def get_artifact_reference(self, artifact_id: str) -> Optional[dict[str, Any]]:
        """Get an artifact reference by its ID."""
        self._validate_entity_id(artifact_id, "artifact")
        return self.state.get("artifact_references", {}).get(artifact_id)
    
    # Evidence reference operations
    def add_evidence_reference(self, evidence: dict[str, Any]) -> str:
        """
        Add an evidence reference to ECM state.
        
        Args:
            evidence: EvidenceReference object
            
        Returns:
            The evidence_id of the added evidence reference
        """
        with self._lock:
            required_fields = ["evidence_id", "evidence_type", "timestamp", "checksum"]
            for field in required_fields:
                if field not in evidence:
                    raise ECMValidationError(f"EvidenceReference missing required field: '{field}'")
            
            evidence_id = evidence["evidence_id"]
            self._validate_entity_id(evidence_id, "evidence")
            
            # Validate evidence_type
            valid_types = [
                "test:output", "build:log", "lint:result", "security:scan",
                "verification:check", "manual:review", "observation"
            ]
            if evidence.get("evidence_type") not in valid_types:
                raise ECMValidationError(
                    f"Invalid evidence_type: {evidence.get('evidence_type')}. "
                    f"Must be one of: {valid_types}"
                )
            
            # Validate checksum format
            checksum = evidence.get("checksum", "")
            if len(checksum) != 64:
                raise ECMValidationError(
                    f"Invalid checksum: {checksum}. Must be 64-character hex string."
                )
            
            # Add to state
            self.state.setdefault("evidence_references", {})[evidence_id] = evidence
            self._save_state()
            return evidence_id
    
    def get_evidence_reference(self, evidence_id: str) -> Optional[dict[str, Any]]:
        """Get an evidence reference by its ID."""
        self._validate_entity_id(evidence_id, "evidence")
        return self.state.get("evidence_references", {}).get(evidence_id)
    
    # Memory binding operations
    def add_memory_binding(self, binding: dict[str, Any]) -> str:
        """
        Add a memory binding to ECM state.
        
        Args:
            binding: MemoryBinding object
            
        Returns:
            The binding_id of the added binding
        """
        with self._lock:
            required_fields = ["binding_id", "memory_id", "bound_entity_type", "bound_entity_id"]
            for field in required_fields:
                if field not in binding:
                    raise ECMValidationError(f"MemoryBinding missing required field: '{field}'")
            
            binding_id = binding["binding_id"]
            self._validate_entity_id(binding_id, "binding")
            self._validate_entity_id(binding["memory_id"], "memory")
            self._validate_entity_id(binding["bound_entity_id"], binding["bound_entity_type"])
            
            # Validate bound_entity_type
            valid_types = ["task", "attempt", "role", "message", "artifact", "evidence"]
            if binding.get("bound_entity_type") not in valid_types:
                raise ECMValidationError(
                    f"Invalid bound_entity_type: {binding.get('bound_entity_type')}. "
                    f"Must be one of: {valid_types}"
                )
            
            # Validate binding_type
            valid_binding_types = ["context", "evidence", "input", "output", "reference", "dependency"]
            if binding.get("binding_type") not in valid_binding_types:
                raise ECMValidationError(
                    f"Invalid binding_type: {binding.get('binding_type')}. "
                    f"Must be one of: {valid_binding_types}"
                )
            
            # Set default timestamp
            if "created_timestamp" not in binding:
                binding["created_timestamp"] = datetime.now(timezone.utc).isoformat()
            
            # Add to state
            self.state.setdefault("memory_bindings", {})[binding_id] = binding
            self._save_state()
            return binding_id
    
    def get_memory_binding(self, binding_id: str) -> Optional[dict[str, Any]]:
        """Get a memory binding by its ID."""
        self._validate_entity_id(binding_id, "binding")
        return self.state.get("memory_bindings", {}).get(binding_id)
    
    # ContextView operations
    def add_context_view(self, context_view: dict[str, Any]) -> str:
        """
        Add a context view to ECM state.
        
        Args:
            context_view: ContextView object
            
        Returns:
            The context_view_id of the added context view
        """
        with self._lock:
            required_fields = ["context_view_id", "snapshot_timestamp", "context_type"]
            for field in required_fields:
                if field not in context_view:
                    raise ECMValidationError(f"ContextView missing required field: '{field}'")
            
            context_view_id = context_view["context_view_id"]
            self._validate_entity_id(context_view_id, "context_view")
            
            # Validate context_type
            valid_types = ["full", "partial", "focused", "minimal"]
            if context_view.get("context_type") not in valid_types:
                raise ECMValidationError(
                    f"Invalid context_type: {context_view.get('context_type')}. "
                    f"Must be one of: {valid_types}"
                )
            
            # Set default timestamp
            if "snapshot_timestamp" not in context_view:
                context_view["snapshot_timestamp"] = datetime.now(timezone.utc).isoformat()
            
            # Add to state
            self.state.setdefault("context_views", {})[context_view_id] = context_view
            self._save_state()
            return context_view_id
    
    def get_context_view(self, context_view_id: str) -> Optional[dict[str, Any]]:
        """Get a context view by its ID."""
        self._validate_entity_id(context_view_id, "context_view")
        return self.state.get("context_views", {}).get(context_view_id)
    
    # Promotion candidate operations
    def add_promotion_candidate(self, candidate: dict[str, Any]) -> str:
        """
        Add a promotion candidate to ECM state.
        
        Args:
            candidate: PromotionCandidate object
            
        Returns:
            The candidate_id of the added candidate
        """
        with self._lock:
            required_fields = ["candidate_id", "entity_type", "entity_id", "status"]
            for field in required_fields:
                if field not in candidate:
                    raise ECMValidationError(f"PromotionCandidate missing required field: '{field}'")
            
            candidate_id = candidate["candidate_id"]
            self._validate_entity_id(candidate_id, "candidate")
            self._validate_entity_id(candidate["entity_id"], candidate["entity_type"])
            
            # Validate entity_type
            valid_types = ["task", "attempt", "memory", "artifact", "evidence"]
            if candidate.get("entity_type") not in valid_types:
                raise ECMValidationError(
                    f"Invalid entity_type: {candidate.get('entity_type')}. "
                    f"Must be one of: {valid_types}"
                )
            
            # Validate status
            valid_statuses = ["proposed", "reviewed", "approved", "rejected", "promoted", "failed"]
            if candidate.get("status") not in valid_statuses:
                raise ECMValidationError(
                    f"Invalid status: {candidate.get('status')}. "
                    f"Must be one of: {valid_statuses}"
                )
            
            # Set default timestamp
            if "proposed_timestamp" not in candidate:
                candidate["proposed_timestamp"] = datetime.now(timezone.utc).isoformat()
            
            # Add to state
            self.state.setdefault("promotion_candidates", []).append(candidate)
            self._save_state()
            return candidate_id
    
    def get_promotion_candidate(self, candidate_id: str) -> Optional[dict[str, Any]]:
        """Get a promotion candidate by its ID."""
        self._validate_entity_id(candidate_id, "candidate")
        for c in self.state.get("promotion_candidates", []):
            if c.get("candidate_id") == candidate_id:
                return c
        return None
    
    def list_promotion_candidates(self, status: Optional[str] = None) -> list[dict[str, Any]]:
        """List all promotion candidates, optionally filtered by status."""
        candidates = list(self.state.get("promotion_candidates", []))
        if status:
            candidates = [c for c in candidates if c.get("status") == status]
        return candidates
    
    # Utility methods
    def get_full_state(self) -> dict[str, Any]:
        """Get the complete ECM state."""
        return self.state.copy()
    
    def get_statistics(self) -> dict[str, Any]:
        """Get statistics about the ECM state."""
        return {
            "task_count": len(self.state.get("tasks", {})),
            "attempt_count": len(self.state.get("attempts", {})),
            "role_count": len(self.state.get("roles", {})),
            "message_count": len(self.state.get("messages", [])),
            "lease_count": len(self.state.get("role_leases", {})),
            "artifact_count": len(self.state.get("artifact_references", {})),
            "evidence_count": len(self.state.get("evidence_references", {})),
            "binding_count": len(self.state.get("memory_bindings", {})),
            "context_view_count": len(self.state.get("context_views", {})),
            "promotion_candidate_count": len(self.state.get("promotion_candidates", [])),
            "ecm_id": self.state.get("ecm_id", "unknown"),
            "schema_version": self.state.get("schema_version", "unknown"),
            "last_updated": self.state.get("timestamp"),
        }


class ECMMCPServer:
    """
    MCP Server for ECM Collaboration State
    
    Exposes ECM state as MCP resources via HTTP/JSON API.
    Follows MCP (Model Context Protocol) conventions for resource exposure.
    """
    
    def __init__(self, host: str = "127.0.0.1", port: int = 8080, state_path: Optional[Path] = None, ecm_id: str = "autodev-main-ecm"):
        """
        Initialize ECM MCP Server.
        
        Args:
            host: Host to bind to
            port: Port to listen on
            state_path: Path to ECM state file
            ecm_id: ECM instance identifier
        """
        self.host = host
        self.port = port
        self.ecm_state = ECMState(state_path=state_path, ecm_id=ecm_id)
        self.server: Optional[HTTPServer] = None
        self._shutdown_flag = False
        
    def start(self) -> None:
        """Start the MCP server."""
        self.server = HTTPServer((self.host, self.port), ECMRequestHandler)
        self.server.ecm_state = self.ecm_state  # Pass state to handler
        
        print(f"ECM MCP Server started on http://{self.host}:{self.port}")
        print(f"ECM ID: {self.ecm_state.ecm_id}")
        print(f"State path: {self.ecm_state.state_path}")
        
        try:
            self.server.serve_forever()
        except KeyboardInterrupt:
            self.stop()
    
    def stop(self) -> None:
        """Stop the MCP server."""
        if self.server:
            self.server.shutdown()
            self.server = None
        self._shutdown_flag = True
    
    def health_check(self) -> dict[str, Any]:
        """Perform health check."""
        stats = self.ecm_state.get_statistics()
        return {
            "status": "healthy",
            "ecm_id": self.ecm_state.ecm_id,
            "schema_version": self.ecm_state.state.get("schema_version"),
            "statistics": stats,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }


class ECMRequestHandler(BaseHTTPRequestHandler):
    """HTTP request handler for ECM MCP Server."""
    
    # Disable logging by default (can be enabled via command line)
    def log_message(self, format, *args) -> None:
        pass
    
    def _send_json(self, status_code: int, data: Any) -> None:
        """Send JSON response."""
        content = json.dumps(data, indent=2, ensure_ascii=False, default=str)
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(content.encode("utf-8"))))
        self.end_headers()
        self.wfile.write(content.encode("utf-8"))
    
    def _send_error(self, status_code: int, message: str) -> None:
        """Send error response."""
        self._send_json(status_code, {
            "error": message,
            "status": "error",
            "status_code": status_code,
        })
    
    def _get_ecm_state(self) -> ECMState:
        """Get the ECMState instance from the server."""
        if not hasattr(self.server, 'ecm_state'):
            raise RuntimeError("ECMState not initialized on server")
        return self.server.ecm_state
    
    def do_GET(self) -> None:
        """Handle GET requests."""
        parsed = urlparse(self.path)
        path = unquote(parsed.path)
        query = parse_qs(parsed.query)
        
        try:
            # Health check
            if path == "/health" or path == "/healthz":
                ecm_state = self._get_ecm_state()
                server = self.server  # type: ignore
                if hasattr(server, 'mcp_server'):
                    health = server.mcp_server.health_check()  # type: ignore
                else:
                    health = ecm_state.get_statistics()
                    health["status"] = "healthy"
                self._send_json(200, health)
                return
            
            # Root endpoint - list available resources
            if path == "/":
                self._send_json(200, {
                    "message": "AMCX-1 ECM MCP Server",
                    "version": "1.0.0",
                    "endpoints": {
                        "GET /health": "Health check",
                        "GET /state": "Full ECM state",
                        "GET /state/stats": "ECM statistics",
                        "GET /tasks": "List all tasks",
                        "GET /tasks/{task_id}": "Get a task",
                        "GET /attempts": "List all attempts",
                        "GET /attempts/{attempt_id}": "Get an attempt",
                        "GET /roles": "List all roles",
                        "GET /roles/{role_id}": "Get a role",
                        "GET /messages": "List all messages",
                        "GET /messages/{message_id}": "Get a message",
                        "GET /leases": "List all role leases",
                        "GET /leases/{lease_id}": "Get a role lease",
                        "GET /artifacts": "List all artifact references",
                        "GET /artifacts/{artifact_id}": "Get an artifact reference",
                        "GET /evidence": "List all evidence references",
                        "GET /evidence/{evidence_id}": "Get an evidence reference",
                        "GET /bindings": "List all memory bindings",
                        "GET /bindings/{binding_id}": "Get a memory binding",
                        "GET /context-views": "List all context views",
                        "GET /context-views/{context_view_id}": "Get a context view",
                        "GET /promotion-candidates": "List all promotion candidates",
                        "GET /promotion-candidates/{candidate_id}": "Get a promotion candidate",
                    },
                    "schema_version": ECM_SCHEMA_VERSION,
                })
                return
            
            ecm_state = self._get_ecm_state()
            
            # Full state
            if path == "/state":
                self._send_json(200, ecm_state.get_full_state())
                return
            
            # Statistics
            if path == "/state/stats":
                self._send_json(200, ecm_state.get_statistics())
                return
            
            # Tasks
            if path == "/tasks":
                status_filter = query.get("status", [None])[0]
                tasks = ecm_state.list_tasks(status=status_filter)
                self._send_json(200, {"tasks": tasks, "count": len(tasks)})
                return
            
            if path.startswith("/tasks/"):
                task_id = path.split("/tasks/", 1)[1]
                task = ecm_state.get_task(task_id)
                if task:
                    self._send_json(200, task)
                else:
                    self._send_error(404, f"Task not found: {task_id}")
                return
            
            # Attempts
            if path == "/attempts":
                task_id = query.get("task_id", [None])[0]
                attempts = ecm_state.list_attempts(task_id=task_id)
                self._send_json(200, {"attempts": attempts, "count": len(attempts)})
                return
            
            if path.startswith("/attempts/"):
                attempt_id = path.split("/attempts/", 1)[1]
                attempt = ecm_state.get_attempt(attempt_id)
                if attempt:
                    self._send_json(200, attempt)
                else:
                    self._send_error(404, f"Attempt not found: {attempt_id}")
                return
            
            # Roles
            if path == "/roles":
                roles = ecm_state.list_roles()
                self._send_json(200, {"roles": roles, "count": len(roles)})
                return
            
            if path.startswith("/roles/"):
                role_id = path.split("/roles/", 1)[1]
                role = ecm_state.get_role(role_id)
                if role:
                    self._send_json(200, role)
                else:
                    self._send_error(404, f"Role not found: {role_id}")
                return
            
            # Messages
            if path == "/messages":
                task_id = query.get("task_id", [None])[0]
                messages = ecm_state.list_messages(task_id=task_id)
                self._send_json(200, {"messages": messages, "count": len(messages)})
                return
            
            if path.startswith("/messages/"):
                message_id = path.split("/messages/", 1)[1]
                message = ecm_state.get_message(message_id)
                if message:
                    self._send_json(200, message)
                else:
                    self._send_error(404, f"Message not found: {message_id}")
                return
            
            # Role Leases
            if path == "/leases":
                role_id = query.get("role_id", [None])[0]
                leases = ecm_state.list_role_leases(role_id=role_id)
                self._send_json(200, {"leases": leases, "count": len(leases)})
                return
            
            if path.startswith("/leases/"):
                lease_id = path.split("/leases/", 1)[1]
                lease = ecm_state.get_role_lease(lease_id)
                if lease:
                    self._send_json(200, lease)
                else:
                    self._send_error(404, f"Role lease not found: {lease_id}")
                return
            
            # Artifact References
            if path == "/artifacts":
                artifacts = list(ecm_state.state.get("artifact_references", {}).values())
                self._send_json(200, {"artifacts": artifacts, "count": len(artifacts)})
                return
            
            if path.startswith("/artifacts/"):
                artifact_id = path.split("/artifacts/", 1)[1]
                artifact = ecm_state.get_artifact_reference(artifact_id)
                if artifact:
                    self._send_json(200, artifact)
                else:
                    self._send_error(404, f"Artifact reference not found: {artifact_id}")
                return
            
            # Evidence References
            if path == "/evidence":
                evidence = list(ecm_state.state.get("evidence_references", {}).values())
                self._send_json(200, {"evidence": evidence, "count": len(evidence)})
                return
            
            if path.startswith("/evidence/"):
                evidence_id = path.split("/evidence/", 1)[1]
                evidence = ecm_state.get_evidence_reference(evidence_id)
                if evidence:
                    self._send_json(200, evidence)
                else:
                    self._send_error(404, f"Evidence reference not found: {evidence_id}")
                return
            
            # Memory Bindings
            if path == "/bindings":
                bindings = list(ecm_state.state.get("memory_bindings", {}).values())
                self._send_json(200, {"bindings": bindings, "count": len(bindings)})
                return
            
            if path.startswith("/bindings/"):
                binding_id = path.split("/bindings/", 1)[1]
                binding = ecm_state.get_memory_binding(binding_id)
                if binding:
                    self._send_json(200, binding)
                else:
                    self._send_error(404, f"Memory binding not found: {binding_id}")
                return
            
            # Context Views
            if path == "/context-views":
                views = list(ecm_state.state.get("context_views", {}).values())
                self._send_json(200, {"context_views": views, "count": len(views)})
                return
            
            if path.startswith("/context-views/"):
                view_id = path.split("/context-views/", 1)[1]
                view = ecm_state.get_context_view(view_id)
                if view:
                    self._send_json(200, view)
                else:
                    self._send_error(404, f"Context view not found: {view_id}")
                return
            
            # Promotion Candidates
            if path == "/promotion-candidates":
                status_filter = query.get("status", [None])[0]
                candidates = ecm_state.list_promotion_candidates(status=status_filter)
                self._send_json(200, {"candidates": candidates, "count": len(candidates)})
                return
            
            if path.startswith("/promotion-candidates/"):
                candidate_id = path.split("/promotion-candidates/", 1)[1]
                candidate = ecm_state.get_promotion_candidate(candidate_id)
                if candidate:
                    self._send_json(200, candidate)
                else:
                    self._send_error(404, f"Promotion candidate not found: {candidate_id}")
                return
            
            # Unknown endpoint
            self._send_error(404, f"Unknown endpoint: {path}")
            
        except Exception as e:
            self._send_error(500, f"Internal server error: {str(e)}")
    
    def do_POST(self) -> None:
        """Handle POST requests (create operations)."""
        parsed = urlparse(self.path)
        path = unquote(parsed.path)
        
        try:
            # Read request body
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)
            data = json.loads(body.decode("utf-8")) if body else {}
            
            ecm_state = self._get_ecm_state()
            
            # Add task
            if path == "/tasks":
                task_id = ecm_state.add_task(data)
                self._send_json(201, {"task_id": task_id, "status": "created"})
                return
            
            # Add attempt
            if path == "/attempts":
                attempt_id = ecm_state.add_attempt(data)
                self._send_json(201, {"attempt_id": attempt_id, "status": "created"})
                return
            
            # Add role
            if path == "/roles":
                role_id = ecm_state.add_role(data)
                self._send_json(201, {"role_id": role_id, "status": "created"})
                return
            
            # Add message
            if path == "/messages":
                message_id = ecm_state.add_message(data)
                self._send_json(201, {"message_id": message_id, "status": "created"})
                return
            
            # Add role lease
            if path == "/leases":
                lease_id = ecm_state.add_role_lease(data)
                self._send_json(201, {"lease_id": lease_id, "status": "created"})
                return
            
            # Add artifact reference
            if path == "/artifacts":
                artifact_id = ecm_state.add_artifact_reference(data)
                self._send_json(201, {"artifact_id": artifact_id, "status": "created"})
                return
            
            # Add evidence reference
            if path == "/evidence":
                evidence_id = ecm_state.add_evidence_reference(data)
                self._send_json(201, {"evidence_id": evidence_id, "status": "created"})
                return
            
            # Add memory binding
            if path == "/bindings":
                binding_id = ecm_state.add_memory_binding(data)
                self._send_json(201, {"binding_id": binding_id, "status": "created"})
                return
            
            # Add context view
            if path == "/context-views":
                view_id = ecm_state.add_context_view(data)
                self._send_json(201, {"context_view_id": view_id, "status": "created"})
                return
            
            # Add promotion candidate
            if path == "/promotion-candidates":
                candidate_id = ecm_state.add_promotion_candidate(data)
                self._send_json(201, {"candidate_id": candidate_id, "status": "created"})
                return
            
            # Unknown endpoint
            self._send_error(404, f"Unknown endpoint: {path}")
            
        except ECMValidationError as e:
            self._send_error(400, str(e))
        except Exception as e:
            self._send_error(500, f"Internal server error: {str(e)}")
    
    def do_PUT(self) -> None:
        """Handle PUT requests (update operations)."""
        parsed = urlparse(self.path)
        path = unquote(parsed.path)
        
        try:
            # Read request body
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)
            data = json.loads(body.decode("utf-8")) if body else {}
            
            ecm_state = self._get_ecm_state()
            
            # Update task status
            if path.startswith("/tasks/"):
                task_id = path.split("/tasks/", 1)[1]
                new_status = data.get("status")
                if new_status:
                    ecm_state.update_task_status(task_id, new_status)
                    self._send_json(200, {"task_id": task_id, "status": new_status, "message": "updated"})
                else:
                    self._send_error(400, "Missing 'status' field in request body")
                return
            
            # Unknown endpoint
            self._send_error(404, f"Unknown endpoint: {path}")
            
        except ECMValidationError as e:
            self._send_error(400, str(e))
        except Exception as e:
            self._send_error(500, f"Internal server error: {str(e)}")


# Global server instance for module-level access
_app: Optional[ECMMCPServer] = None


def create_app(state_path: Optional[Path] = None, ecm_id: str = "autodev-main-ecm") -> ECMState:
    """
    Create an ECMState instance for programmatic access.
    
    This allows the test suite to access ECM state directly without
    starting the HTTP server.
    
    Args:
        state_path: Path to the state file
        ecm_id: ECM instance identifier
        
    Returns:
        ECMState instance
    """
    return ECMState(state_path=state_path, ecm_id=ecm_id)


def get_app() -> Optional[ECMMCPServer]:
    """Get the global MCP server instance."""
    return _app


def serve(host: str = "127.0.0.1", port: int = 8080, state_path: Optional[Path] = None, ecm_id: str = "autodev-main-ecm") -> None:
    """
    Start the ECM MCP Server.
    
    Args:
        host: Host to bind to
        port: Port to listen on
        state_path: Path to ECM state file
        ecm_id: ECM instance identifier
    """
    global _app
    _app = ECMMCPServer(host=host, port=port, state_path=state_path, ecm_id=ecm_id)
    _app.start()


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(description="AMCX-1 ECM MCP Server")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8080, help="Port to listen on")
    parser.add_argument("--state-path", type=Path, default=None, help="Path to ECM state file")
    parser.add_argument("--ecm-id", default="autodev-main-ecm", help="ECM instance identifier")
    parser.add_argument("--check", action="store_true", help="Check server can be imported and run tests")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.check:
        # Test mode: just verify imports work
        state = create_app(state_path=args.state_path, ecm_id=args.ecm_id)
        print("ECM MCP Server: OK")
        print(f"  State path: {state.state_path}")
        print(f"  ECM ID: {state.ecm_id}")
        print(f"  Schema version: {state.state.get('schema_version')}")
        return
    
    # Normal mode: start server
    if args.verbose:
        # Enable HTTP server logging
        pass
    
    print(f"Starting ECM MCP Server on {args.host}:{args.port}")
    serve(host=args.host, port=args.port, state_path=args.state_path, ecm_id=args.ecm_id)


if __name__ == "__main__":
    main()
