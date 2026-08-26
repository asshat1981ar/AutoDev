#!/usr/bin/env python3
"""
Toolset Memory AMX Compliance Extension

Extends Vibe/AutoDev's Toolset Learning Memory to AMCX-1 v1.1 AMX compliance.

This module provides:
1. AMX field validation for toolset memory patterns
2. Automatic canonical semantic digest computation
3. Origin, scope, provenance, and ancestry tracking
4. Trust/validity state management
5. Visibility and retraction barrier enforcement

Background:
Vibe/AutoDev's Toolset Learning Memory (memory/toolsets/patterns.jsonl) stores
pattern-recognition memories. This extension adds AMX-compliant fields to ensure
these memories meet AMCX-1 v1.1 requirements.

AMX Required Fields:
- origin
- logical_identity
- repository/worktree/project scope
- provenance
- causal ancestry
- trust/validity state
- visibility
- purpose
- retraction/deletion barriers
- canonical semantic digest
"""

import hashlib
import json
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


# AMX Required Fields
AMX_REQUIRED_FIELDS = [
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
]

# Default values for AMX fields
DEFAULT_SCHEMA_VERSION = "amx-memory-v1"
DEFAULT_ORIGIN = "vibe:toolset-memory"
DEFAULT_VISIBILITY = {"scope": "workspace", "access_level": "read"}
DEFAULT_TRUST_STATE = {
    "trust_level": "medium",
    "validity_status": "valid",
    "last_validated": None,
}
DEFAULT_BARRIERS = {"retractable": True, "deletable": False}


class ToolsetAMXError(ValueError):
    """Raised when Toolset AMX operations fail."""
    pass


class ToolsetAMX:
    """
    Toolset Memory AMX Compliance Extension.
    
    This class extends Toolset Learning Memory patterns to AMX compliance
    by adding required AMX fields and computing semantic digests.
    """
    
    def __init__(self, memory_path: Optional[Path] = None):
        """
        Initialize Toolset AMX.
        
        Args:
            memory_path: Path to toolset memory file (patterns.jsonl).
                        If None, uses default path.
        """
        self.memory_path = memory_path or Path("memory/toolsets/patterns.jsonl")
        self._cache: dict[str, Any] = {}
    
    def _get_repository_context(self) -> dict[str, str]:
        """Get current repository context.

        Uses AMCX_* environment variables when available (set by
        CI or worktree harness), falling back to git CLI to
        discover revision, path, branch, and worktree id.
        """
        import subprocess

        revision = os.environ.get("AMCX_REVISION", "")
        repo_path = os.environ.get("AMCX_REPO_PATH", "")
        branch = os.environ.get("AMCX_BRANCH", "")
        worktree_id = os.environ.get("AMCX_WORKTREE", "")

        # Fall back to git CLI for missing values
        if not revision:
            try:
                result = subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0:
                    revision = result.stdout.strip()
            except (subprocess.SubprocessError, FileNotFoundError):
                revision = "unknown"

        if not repo_path:
            try:
                result = subprocess.run(
                    ["git", "rev-parse", "--show-toplevel"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0:
                    repo_path = result.stdout.strip()
            except (subprocess.SubprocessError, FileNotFoundError):
                repo_path = str(Path.cwd())

        if not branch:
            try:
                result = subprocess.run(
                    ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0:
                    branch = result.stdout.strip()
            except (subprocess.SubprocessError, FileNotFoundError):
                branch = "main"

        if not worktree_id:
            try:
                result = subprocess.run(
                    ["git", "rev-parse", "--git-path", ".git"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0:
                    git_path = result.stdout.strip()
                    if git_path and "/.worktrees/" in git_path:
                        worktree_id = git_path.split("/.worktrees/", 1)[1].split("/", 1)[0]
            except (subprocess.SubprocessError, FileNotFoundError):
                pass
            if not worktree_id:
                worktree_id = "primary"

        return {
            "repository_revision": revision or "unknown",
            "repository_path": repo_path or str(Path.cwd()),
            "branch": branch or "main",
            "worktree_id": worktree_id or "primary",
        }
    
    def _generate_logical_identity(self, pattern: dict[str, Any]) -> str:
        """Generate logical identity from pattern."""
        import re
        
        # Use pattern_id if available
        if "pattern_id" in pattern:
            lid = pattern["pattern_id"]
        elif "id" in pattern:
            lid = pattern["id"]
        else:
            # Generate from task_class and when_to_use
            parts = []
            if "task_class" in pattern:
                parts.append(str(pattern["task_class"]).lower().replace(" ", "-"))
            if "when_to_use" in pattern:
                parts.append(str(pattern["when_to_use"]).lower().replace(" ", "-")[:20])
            if parts:
                lid = "-".join(parts)
            else:
                # Fallback: hash-based
                raw = json.dumps(pattern, sort_keys=True)
                lid = hashlib.sha256(raw.encode()).hexdigest()[:16]
        
        # Ensure it matches AMX ID pattern: ^[a-z0-9]+(?:-[a-z0-9]+)*$
        lid = re.sub(r"[^a-z0-9-]", "-", lid.lower())
        lid = re.sub(r"--+", "-", lid)
        lid = lid.strip("-")
        return lid or "pattern-memory"
    
    def _compute_semantic_digest(self, memory: dict[str, Any]) -> str:
        """Compute SHA-256 semantic digest for a memory record."""
        # Create a copy without the digest field to avoid circularity
        memory_for_digest = memory.copy()
        memory_for_digest.pop("canonical_semantic_digest", None)
        
        # Convert to canonical JSON
        canonical_json = json.dumps(memory_for_digest, sort_keys=True, ensure_ascii=False)
        return hashlib.sha256(canonical_json.encode("utf-8")).hexdigest()
    
    def _build_provenance(self, pattern: dict[str, Any]) -> list[dict[str, Any]]:
        """Build provenance chain for a pattern."""
        provenance = []
        
        # Add creation provenance
        provenance.append({
            "source": pattern.get("source", "toolset:learning"),
            "timestamp": pattern.get("timestamp") or datetime.now(timezone.utc).isoformat(),
            "checksum": pattern.get("checksum", ""),
            "description": "Pattern learned from tool usage"
        })
        
        # Add validation provenance if available
        if "validation" in pattern:
            val = pattern["validation"]
            provenance.append({
                "source": "validation",
                "timestamp": val.get("timestamp") or datetime.now(timezone.utc).isoformat(),
                "checksum": val.get("checksum", ""),
                "description": "Pattern validated"
            })
        
        return provenance
    
    def _build_causal_ancestry(self, pattern: dict[str, Any]) -> list[dict[str, Any]]:
        """Build causal ancestry for a pattern."""
        ancestry = []
        
        # Add ancestor references if present
        if "ancestors" in pattern:
            for ancestor in pattern["ancestors"]:
                ancestry.append({
                    "logical_identity": str(ancestor.get("pattern_id", ancestor.get("id", ""))),
                    "relationship": "derived_from",
                    "description": "Pattern derived from ancestor"
                })
        
        return ancestry
    
    def _ensure_amx_fields(self, pattern: dict[str, Any]) -> dict[str, Any]:
        """Ensure a pattern has all required AMX fields."""
        amx_pattern = pattern.copy()
        
        # Add schema version
        amx_pattern.setdefault("schema_version", DEFAULT_SCHEMA_VERSION)
        
        # Add origin
        amx_pattern.setdefault("origin", DEFAULT_ORIGIN)
        
        # Generate logical identity
        amx_pattern.setdefault("logical_identity", self._generate_logical_identity(pattern))
        
        # Add repository scope
        if "repository_scope" not in amx_pattern:
            amx_pattern["repository_scope"] = self._get_repository_context()
        
        # Add provenance
        if "provenance" not in amx_pattern or not amx_pattern["provenance"]:
            amx_pattern["provenance"] = self._build_provenance(pattern)
        
        # Add causal ancestry
        if "causal_ancestry" not in amx_pattern:
            amx_pattern["causal_ancestry"] = self._build_causal_ancestry(pattern)
        
        # Add trust validity state
        amx_pattern.setdefault("trust_validity_state", DEFAULT_TRUST_STATE.copy())
        amx_pattern["trust_validity_state"]["last_validated"] = \
            amx_pattern["trust_validity_state"].get("last_validated") or \
            datetime.now(timezone.utc).isoformat()
        
        # Add visibility
        amx_pattern.setdefault("visibility", DEFAULT_VISIBILITY.copy())
        
        # Add purpose
        if "purpose" not in amx_pattern:
            amx_pattern["purpose"] = {
                "primary": pattern.get("task_class", "unknown"),
                "task_class": pattern.get("task_class", "pattern:recognition")
            }
        
        # Add retraction/deletion barriers
        amx_pattern.setdefault("retraction_deletion_barriers", DEFAULT_BARRIERS.copy())
        
        # Compute and set canonical semantic digest
        amx_pattern["canonical_semantic_digest"] = self._compute_semantic_digest(amx_pattern)
        
        return amx_pattern
    
    def validate_amx(self, pattern: dict[str, Any]) -> tuple[bool, list[str]]:
        """
        Validate that a pattern meets AMX requirements.
        
        Args:
            pattern: The pattern to validate
            
        Returns:
            Tuple of (is_valid, list of error messages)
        """
        errors = []
        
        # Check required fields
        for field in AMX_REQUIRED_FIELDS:
            if field not in pattern:
                errors.append(f"Missing required field: '{field}'")
        
        # Validate schema version
        if pattern.get("schema_version") != DEFAULT_SCHEMA_VERSION:
            errors.append(f"Invalid schema_version: {pattern.get('schema_version')}")
        
        # Validate repository scope
        repo_scope = pattern.get("repository_scope", {})
        if not isinstance(repo_scope, dict):
            errors.append("repository_scope must be an object")
        else:
            for field in ["repository_revision", "repository_path"]:
                if field not in repo_scope:
                    errors.append(f"repository_scope missing: '{field}'")
        
        # Validate provenance
        provenance = pattern.get("provenance", [])
        if not isinstance(provenance, list) or len(provenance) == 0:
            errors.append("provenance must be a non-empty array")
        
        # Validate causal ancestry
        ancestry = pattern.get("causal_ancestry", [])
        if not isinstance(ancestry, list):
            errors.append("causal_ancestry must be an array")
        
        # Validate trust_validity_state
        trust_state = pattern.get("trust_validity_state", {})
        if not isinstance(trust_state, dict):
            errors.append("trust_validity_state must be an object")
        else:
            for field in ["trust_level", "validity_status", "last_validated"]:
                if field not in trust_state:
                    errors.append(f"trust_validity_state missing: '{field}'")
        
        # Validate visibility
        visibility = pattern.get("visibility", {})
        if not isinstance(visibility, dict):
            errors.append("visibility must be an object")
        else:
            for field in ["scope", "access_level"]:
                if field not in visibility:
                    errors.append(f"visibility missing: '{field}'")
        
        # Validate purpose
        purpose = pattern.get("purpose", {})
        if not isinstance(purpose, dict):
            errors.append("purpose must be an object")
        else:
            for field in ["primary", "task_class"]:
                if field not in purpose:
                    errors.append(f"purpose missing: '{field}'")
        
        # Validate retraction_deletion_barriers
        barriers = pattern.get("retraction_deletion_barriers", {})
        if not isinstance(barriers, dict):
            errors.append("retraction_deletion_barriers must be an object")
        else:
            for field in ["retractable", "deletable"]:
                if field not in barriers:
                    errors.append(f"retraction_deletion_barriers missing: '{field}'")
        
        # Validate canonical_semantic_digest
        digest = pattern.get("canonical_semantic_digest", "")
        if not isinstance(digest, str) or len(digest) != 64:
            errors.append("canonical_semantic_digest must be a 64-character hex string")
        
        return len(errors) == 0, errors
    
    def upgrade_to_amx(self, pattern: dict[str, Any]) -> dict[str, Any]:
        """
        Upgrade a toolset pattern to AMX compliance.
        
        Args:
            pattern: The original pattern
            
        Returns:
            AMX-compliant pattern with all required fields
        """
        return self._ensure_amx_fields(pattern)
    
    def load_memory(self, path: Optional[Path] = None) -> list[dict[str, Any]]:
        """
        Load toolset memory and upgrade to AMX compliance.
        
        Args:
            path: Path to memory file. If None, uses memory_path.
            
        Returns:
            List of AMX-compliant patterns
        """
        load_path = path or self.memory_path
        
        if not load_path.exists():
            return []
        
        patterns = []
        try:
            with open(load_path, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    pattern = json.loads(line)
                    patterns.append(pattern)
        except (json.JSONDecodeError, OSError) as e:
            raise ToolsetAMXError(f"Failed to load memory: {e}")
        
        return patterns
    
    def save_memory(self, patterns: list[dict[str, Any]], path: Optional[Path] = None) -> None:
        """
        Save AMX-compliant patterns to memory file.
        
        Args:
            patterns: List of patterns to save
            path: Path to memory file. If None, uses memory_path.
        """
        save_path = path or self.memory_path
        
        try:
            save_path.parent.mkdir(parents=True, exist_ok=True)
            with open(save_path, 'w', encoding='utf-8') as f:
                for pattern in patterns:
                    f.write(json.dumps(pattern, ensure_ascii=False) + '\n')
        except OSError as e:
            raise ToolsetAMXError(f"Failed to save memory: {e}")
    
    def migrate_to_amx(self, path: Optional[Path] = None) -> dict[str, Any]:
        """
        Migrate existing toolset memory to AMX compliance.
        
        Args:
            path: Path to memory file. If None, uses memory_path.
            
        Returns:
            Migration report
        """
        load_path = path or self.memory_path
        report = {
            "total_patterns": 0,
            "already_compliant": 0,
            "upgraded": 0,
            "errors": 0,
            "error_details": [],
        }
        
        # Load existing patterns
        patterns = self.load_memory(load_path)
        report["total_patterns"] = len(patterns)
        
        upgraded_patterns = []
        
        for i, pattern in enumerate(patterns):
            try:
                # Check if already compliant
                is_valid, errors = self.validate_amx(pattern)
                if is_valid:
                    report["already_compliant"] += 1
                    upgraded_patterns.append(pattern)
                else:
                    # Upgrade to AMX compliance
                    upgraded = self.upgrade_to_amx(pattern)
                    upgraded_patterns.append(upgraded)
                    report["upgraded"] += 1
            except Exception as e:
                report["errors"] += 1
                # Preserve the original pattern: upgraded_patterns replaces the
                # source file below, so dropping a failed pattern here would
                # silently delete data.
                upgraded_patterns.append(pattern)
                report["error_details"].append({
                    "index": i,
                    "pattern_id": pattern.get("pattern_id", pattern.get("id", "unknown")),
                    "error": str(e)
                })
        
        # Save upgraded patterns
        if upgraded_patterns:
            self.save_memory(upgraded_patterns, load_path)
        
        return report
    
    def add_amx_pattern(self, pattern: dict[str, Any]) -> str:
        """
        Add a new AMX-compliant pattern to memory.
        
        Args:
            pattern: Pattern to add (will be upgraded to AMX if needed)
            
        Returns:
            The logical_identity of the added pattern
        """
        # Ensure AMX compliance
        amx_pattern = self.upgrade_to_amx(pattern)
        
        # Load existing patterns
        patterns = self.load_memory()
        
        # Add new pattern
        patterns.append(amx_pattern)
        
        # Save
        self.save_memory(patterns)
        
        return amx_pattern["logical_identity"]
    
    def get_amx_pattern(self, logical_identity: str) -> Optional[dict[str, Any]]:
        """
        Get a pattern by its logical identity.
        
        Args:
            logical_identity: The logical identity to retrieve
            
        Returns:
            The pattern if found, None otherwise
        """
        patterns = self.load_memory()
        for pattern in patterns:
            if pattern.get("logical_identity") == logical_identity:
                return pattern
        return None
    
    def list_amx_patterns(self, origin: Optional[str] = None) -> list[dict[str, Any]]:
        """
        List all AMX-compliant patterns.
        
        Args:
            origin: Optional origin filter
            
        Returns:
            List of patterns matching the filter
        """
        patterns = self.load_memory()
        if origin:
            patterns = [p for p in patterns if p.get("origin") == origin]
        return patterns
    
    def verify_amx_compliance(self, path: Optional[Path] = None) -> dict[str, Any]:
        """
        Verify that all patterns in memory are AMX-compliant.
        
        Args:
            path: Path to memory file. If None, uses memory_path.
            
        Returns:
            Compliance report
        """
        patterns = self.load_memory(path)
        report = {
            "total_patterns": len(patterns),
            "compliant": 0,
            "non_compliant": 0,
            "errors": [],
        }
        
        for i, pattern in enumerate(patterns):
            is_valid, errors = self.validate_amx(pattern)
            if is_valid:
                report["compliant"] += 1
            else:
                report["non_compliant"] += 1
                report["errors"].append({
                    "index": i,
                    "logical_identity": pattern.get("logical_identity", pattern.get("pattern_id", "unknown")),
                    "errors": errors
                })
        
        return report


class ToolsetAMXIntegration:
    """
    Integration layer for Toolset AMX Compliance.
    
    This class provides a higher-level interface for managing
    AMX-compliant toolset memory.
    """
    
    def __init__(self, memory_path: Optional[Path] = None):
        """
        Initialize Toolset AMX Integration.
        
        Args:
            memory_path: Path to toolset memory file
        """
        self.toolset_amx = ToolsetAMX(memory_path=memory_path)
    
    def migrate(self) -> dict[str, Any]:
        """Migrate existing memory to AMX compliance."""
        return self.toolset_amx.migrate_to_amx()
    
    def verify(self) -> dict[str, Any]:
        """Verify AMX compliance of current memory."""
        return self.toolset_amx.verify_amx_compliance()
    
    def add_pattern(self, pattern: dict[str, Any]) -> str:
        """Add a new AMX-compliant pattern."""
        return self.toolset_amx.add_amx_pattern(pattern)
    
    def get_pattern(self, logical_identity: str) -> Optional[dict[str, Any]]:
        """Get a pattern by logical identity."""
        return self.toolset_amx.get_amx_pattern(logical_identity)
    
    def list_patterns(self) -> list[dict[str, Any]]:
        """List all patterns."""
        return self.toolset_amx.list_amx_patterns()


if __name__ == "__main__":
    # Demo usage
    import tempfile
    
    with tempfile.TemporaryDirectory() as tmpdir:
        memory_path = Path(tmpdir) / "patterns.jsonl"
        
        toolset_amx = ToolsetAMX(memory_path=memory_path)
        
        # Create a sample pattern (non-AMX)
        sample_pattern = {
            "pattern_id": "test-pattern-001",
            "task_class": "file:edit",
            "when_to_use": "When editing Python files",
            "result": "high",
            "context": {"file_extension": ".py"},
            "timestamp": "2026-08-20T21:00:00Z",
            "checksum": "abc123",
        }
        
        # Upgrade to AMX
        amx_pattern = toolset_amx.upgrade_to_amx(sample_pattern)
        print(f"Upgraded pattern: {amx_pattern['logical_identity']}")
        print(f"  Schema version: {amx_pattern['schema_version']}")
        print(f"  Origin: {amx_pattern['origin']}")
        print(f"  Digest: {amx_pattern['canonical_semantic_digest'][:16]}...")
        
        # Validate
        is_valid, errors = toolset_amx.validate_amx(amx_pattern)
        print(f"  Valid: {is_valid}")
        if errors:
            print(f"  Errors: {errors}")
        
        # Add to memory
        lid = toolset_amx.add_amx_pattern(sample_pattern)
        print(f"  Added to memory with ID: {lid}")
        
        # Verify compliance
        report = toolset_amx.verify_amx_compliance(memory_path)
        print(f"  Compliance: {report['compliant']}/{report['total_patterns']} patterns compliant")
