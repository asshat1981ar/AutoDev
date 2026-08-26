#!/usr/bin/env python3
"""
AMX Memory Skill - Canonical Portable Memory Implementation

This module implements the AMCX-1 v1.1 Agent Memory (AMX) canonical portable
memory history for integration with Vibe.

AMX owns canonical portable memory history. Every durable memory must retain
sufficient information to determine:
- origin
- logical identity
- repository/worktree/project scope
- provenance
- causal ancestry where applicable
- trust/validity state
- visibility
- purpose
- retraction/deletion barriers
- canonical semantic digest

This implementation provides:
- Storage and retrieval of AMX-compliant memory records
- Validation against the AMX memory schema
- SHA-256 semantic digest computation
- Persistence to JSON state files
"""

import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


# AMX v1.1 Required Fields
AMX_REQUIRED_FIELDS = {
    "schema_version",
    "origin",
    "logical_identity",
    "repository_scope",
    "provenance",
    "causal_ancestry",
    "trust_validity_state",
    "visibility",
    "purpose",
    "retraction_deletion_barriers",
    "canonical_semantic_digest",
}

# Sub-fields that are required within their parent objects
REPOSITORY_SCOPE_REQUIRED = {"repository_revision", "repository_path", "worktree_id"}
TRUST_STATE_REQUIRED = {"trust_level", "validity_status", "last_validated"}
VISIBILITY_REQUIRED = {"scope", "access_level"}
PURPOSE_REQUIRED = {"primary", "task_class"}
BARRIERS_REQUIRED = {"retractable", "deletable"}
PROVENANCE_ENTRY_REQUIRED = {"source", "timestamp", "checksum"}


class AMXValidationError(ValueError):
    """Raised when AMX memory validation fails."""
    pass


class AMXMemory:
    """
    AMX Memory Store - Canonical Portable Memory Implementation
    
    Provides storage, retrieval, and validation of AMX-compliant memory records.
    """
    
    DEFAULT_STATE_TEMPLATE = {
        "schema_version": "amx-memory-store-v1",
        "memories": [],
        "created_timestamp": None,
        "updated_timestamp": None,
    }
    
    def __init__(self, state_path: Optional[Path] = None):
        """
        Initialize AMX Memory Store.
        
        Args:
            state_path: Path to the JSON state file. If None, uses in-memory state.
        """
        self.state_path = state_path
        self.state = self._load_state()
    
    def _load_state(self) -> dict[str, Any]:
        """Load state from file or return default."""
        if self.state_path is None:
            # Deep copy to avoid shared mutable state
            return {
                "schema_version": "amx-memory-store-v1",
                "memories": [],
                "created_timestamp": None,
                "updated_timestamp": None,
            }
        
        if self.state_path.exists():
            try:
                content = self.state_path.read_text(encoding="utf-8")
                return json.loads(content)
            except (json.JSONDecodeError, OSError) as e:
                raise AMXValidationError(f"Failed to load state: {e}")
        
        # Deep copy to avoid shared mutable state
        return {
            "schema_version": "amx-memory-store-v1",
            "memories": [],
            "created_timestamp": None,
            "updated_timestamp": None,
        }
    
    def _save_state(self) -> None:
        """Save state to file."""
        if self.state_path is None:
            return
        
        try:
            # Ensure parent directory exists
            self.state_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Update timestamp
            self.state["updated_timestamp"] = datetime.now(timezone.utc).isoformat()
            if self.state.get("created_timestamp") is None:
                self.state["created_timestamp"] = self.state["updated_timestamp"]
            
            content = json.dumps(self.state, indent=2, ensure_ascii=False)
            self.state_path.write_text(content, encoding="utf-8")
        except OSError as e:
            raise AMXValidationError(f"Failed to save state: {e}")
    
    def _validate_memory(self, memory: dict[str, Any]) -> None:
        """
        Validate that a memory record meets AMX requirements.
        
        Args:
            memory: The memory record to validate
            
        Raises:
            AMXValidationError: If the memory is invalid
        """
        # Check top-level required fields
        for field in AMX_REQUIRED_FIELDS:
            if field not in memory:
                raise AMXValidationError(f"Missing required field: '{field}'")
        
        # Validate schema_version
        if memory.get("schema_version") != "amx-memory-v1":
            raise AMXValidationError(
                f"Invalid schema_version: {memory.get('schema_version')}"
            )
        
        # Validate repository_scope
        repo_scope = memory.get("repository_scope", {})
        if not isinstance(repo_scope, dict):
            raise AMXValidationError("repository_scope must be an object")
        for field in REPOSITORY_SCOPE_REQUIRED:
            if field not in repo_scope:
                raise AMXValidationError(
                    f"repository_scope missing required field: '{field}'"
                )
        
        # Validate provenance (must be non-empty array)
        provenance = memory.get("provenance", [])
        if not isinstance(provenance, list) or len(provenance) == 0:
            raise AMXValidationError("provenance must be a non-empty array")
        for entry in provenance:
            if not isinstance(entry, dict):
                raise AMXValidationError("provenance entries must be objects")
            for field in PROVENANCE_ENTRY_REQUIRED:
                if field not in entry:
                    raise AMXValidationError(
                        f"provenance entry missing required field: '{field}'"
                    )
        
        # Validate causal_ancestry (must be array)
        ancestry = memory.get("causal_ancestry", [])
        if not isinstance(ancestry, list):
            raise AMXValidationError("causal_ancestry must be an array")
        for entry in ancestry:
            if not isinstance(entry, dict):
                raise AMXValidationError("causal_ancestry entries must be objects")
            if "logical_identity" not in entry:
                raise AMXValidationError(
                    "causal_ancestry entry missing 'logical_identity'"
                )
            if "relationship" not in entry:
                raise AMXValidationError(
                    "causal_ancestry entry missing 'relationship'"
                )
        
        # Validate trust_validity_state
        trust_state = memory.get("trust_validity_state", {})
        if not isinstance(trust_state, dict):
            raise AMXValidationError("trust_validity_state must be an object")
        for field in TRUST_STATE_REQUIRED:
            if field not in trust_state:
                raise AMXValidationError(
                    f"trust_validity_state missing required field: '{field}'"
                )
        
        # Validate visibility
        visibility = memory.get("visibility", {})
        if not isinstance(visibility, dict):
            raise AMXValidationError("visibility must be an object")
        for field in VISIBILITY_REQUIRED:
            if field not in visibility:
                raise AMXValidationError(
                    f"visibility missing required field: '{field}'"
                )
        
        # Validate purpose
        purpose = memory.get("purpose", {})
        if not isinstance(purpose, dict):
            raise AMXValidationError("purpose must be an object")
        for field in PURPOSE_REQUIRED:
            if field not in purpose:
                raise AMXValidationError(
                    f"purpose missing required field: '{field}'"
                )
        
        # Validate retraction_deletion_barriers
        barriers = memory.get("retraction_deletion_barriers", {})
        if not isinstance(barriers, dict):
            raise AMXValidationError("retraction_deletion_barriers must be an object")
        for field in BARRIERS_REQUIRED:
            if field not in barriers:
                raise AMXValidationError(
                    f"retraction_deletion_barriers missing required field: '{field}'"
                )
        
        # Validate canonical_semantic_digest format
        digest = memory.get("canonical_semantic_digest", "")
        if not isinstance(digest, str) or len(digest) != 64:
            raise AMXValidationError(
                "canonical_semantic_digest must be a 64-character hex string"
            )
    
    def _compute_semantic_digest(self, memory: dict[str, Any]) -> str:
        """
        Compute SHA-256 semantic digest for a memory record.
        
        The digest is computed over the canonical representation of the memory
        (excluding the digest field itself to avoid circular dependencies).
        
        Args:
            memory: The memory record to compute digest for
            
        Returns:
            SHA-256 hex digest
        """
        # Create a copy without the digest field
        memory_for_digest = memory.copy()
        memory_for_digest.pop("canonical_semantic_digest", None)
        
        # Convert to canonical JSON representation
        canonical_json = json.dumps(memory_for_digest, sort_keys=True, ensure_ascii=False)
        
        # Compute SHA-256
        return hashlib.sha256(canonical_json.encode("utf-8")).hexdigest()
    
    def store(self, memory: dict[str, Any]) -> None:
        """
        Store an AMX-compliant memory record.
        
        Validates the memory against AMX requirements before storing.
        Automatically computes and sets the canonical_semantic_digest if not provided.
        If a digest is provided, verifies it matches the computed value.
        
        Args:
            memory: The memory record to store
            
        Raises:
            AMXValidationError: If the memory is invalid
        """
        # Track if digest was provided
        digest_provided = "canonical_semantic_digest" in memory
        
        # Compute semantic digest if not provided
        if not digest_provided:
            memory["canonical_semantic_digest"] = self._compute_semantic_digest(memory)
        
        # Validate the memory (digest is now always present)
        self._validate_memory(memory)
        
        # If digest was provided, verify it matches
        if digest_provided:
            computed = self._compute_semantic_digest(memory)
            if memory["canonical_semantic_digest"] != computed:
                raise AMXValidationError(
                    f"Provided canonical_semantic_digest does not match computed value. "
                    f"Expected: {computed}, Got: {memory['canonical_semantic_digest']}"
                )
        
        # Add to state
        self.state["memories"].append(memory)
        
        # Save state
        self._save_state()
    
    def retrieve(self, logical_identity: str) -> Optional[dict[str, Any]]:
        """
        Retrieve a memory by its logical_identity.
        
        Args:
            logical_identity: The logical identity of the memory to retrieve
            
        Returns:
            The memory record if found, None otherwise
        """
        for memory in self.state.get("memories", []):
            if memory.get("logical_identity") == logical_identity:
                return memory
        return None
    
    def list_all(self) -> list[dict[str, Any]]:
        """
        List all stored memories.
        
        Returns:
            List of all memory records
        """
        return list(self.state.get("memories", []))
    
    def find_by_origin(self, origin: str) -> list[dict[str, Any]]:
        """
        Find all memories with a specific origin.
        
        Args:
            origin: The origin to filter by
            
        Returns:
            List of memory records matching the origin
        """
        return [
            m for m in self.state.get("memories", [])
            if m.get("origin") == origin
        ]
    
    def find_by_scope(self, repository_path: str, worktree_id: str) -> list[dict[str, Any]]:
        """
        Find all memories within a specific repository/worktree scope.
        
        Args:
            repository_path: The repository path to filter by
            worktree_id: The worktree ID to filter by
            
        Returns:
            List of memory records matching the scope
        """
        return [
            m for m in self.state.get("memories", [])
            if (m.get("repository_scope", {}).get("repository_path") == repository_path and
                m.get("repository_scope", {}).get("worktree_id") == worktree_id)
        ]
    
    def count(self) -> int:
        """
        Count the number of stored memories.
        
        Returns:
            Number of memory records
        """
        return len(self.state.get("memories", []))
