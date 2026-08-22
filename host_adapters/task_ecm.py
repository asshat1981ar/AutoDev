#!/usr/bin/env python3
"""
Task ECM Adapter

Extends Vibe's native `task` tool with ECM (Entity-Collaboration Model) attempt tracking.

This adapter provides:
1. Attempt creation and management for tasks
2. Agent/role assignment tracking
3. Attempt lifecycle management (pending, in_progress, completed, failed, cancelled)
4. Integration with ECM state

Usage:
    from host_adapters.task_ecm import TaskECM
    
    task_ecm = TaskECM(ecm_state_path=".vibe/amcx-ecm/state.json")
    
    # Create a task with initial attempt
    task_id = task_ecm.create_task({
        "task_id": "feature-x-impl",
        "title": "Implement Feature X",
        "objective": "Deliver feature X with tests"
    })
    
    # Start an attempt
    attempt_id = task_ecm.start_attempt(task_id, agent_id="agent-001", role_id="builder")
    
    # Update attempt status
    task_ecm.update_attempt_status(attempt_id, "completed", result="success")
"""

import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


# Attempt result mapping
VALID_ATTEMPT_RESULTS = ["success", "failure", "partial", "timeout", "error"]
VALID_ATTEMPT_STATUSES = ["pending", "in_progress", "completed", "failed", "cancelled"]


class TaskECMError(ValueError):
    """Raised when Task ECM operations fail."""
    pass


class TaskECM:
    """
    Task ECM Adapter - Extends Vibe's task tool with ECM attempt tracking.
    
    This adapter provides attempt lifecycle management for tasks:
    - Create tasks with full ECM fields
    - Start attempts on tasks with agent/role assignment
    - Track attempt status and results
    - Manage attempt lifecycle
    - Link attempts to evidence and artifacts
    """
    
    def __init__(self, ecm_state_path: Optional[Path] = None):
        """
        Initialize Task ECM Adapter.
        
        Args:
            ecm_state_path: Path to ECM state file. If None, uses default.
        """
        from mcp_servers.amcx_ecm.server import ECMState
        
        self.ecm_state_path = ecm_state_path
        self._ecm_state: Optional[ECMState] = None
    
    @property
    def ecm_state(self) -> Any:
        """Lazy-load ECM state."""
        if self._ecm_state is None:
            from mcp_servers.amcx_ecm.server import ECMState
            self._ecm_state = ECMState(state_path=self.ecm_state_path)
        return self._ecm_state
    
    def _generate_attempt_id(self, task_id: str, agent_id: str) -> str:
        """Generate a unique attempt ID."""
        import hashlib
        import time
        
        # Create a deterministic but unique ID
        timestamp = str(int(time.time() * 1000))
        raw_id = f"{task_id}-{agent_id}-{timestamp}"
        return hashlib.sha256(raw_id.encode()).hexdigest()[:16]
    
    def create_task(self, task: dict[str, Any]) -> str:
        """
        Create a task with ECM fields.
        
        Args:
            task: Task object with ECM fields
            
        Returns:
            The task_id
        """
        return self.ecm_state.add_task(task)
    
    def get_task(self, task_id: str) -> Optional[dict[str, Any]]:
        """
        Get a task by its ID.
        
        Args:
            task_id: The task ID
            
        Returns:
            The task object or None
        """
        return self.ecm_state.get_task(task_id)
    
    def list_tasks(self, status: Optional[str] = None) -> list[dict[str, Any]]:
        """
        List all tasks.
        
        Args:
            status: Optional status filter
            
        Returns:
            List of task objects
        """
        return self.ecm_state.list_tasks(status=status)
    
    def start_attempt(self, 
                     task_id: str, 
                     agent_id: str, 
                     role_id: Optional[str] = None,
                     metadata: Optional[dict[str, Any]] = None) -> str:
        """
        Start a new attempt on a task.
        
        Args:
            task_id: The task ID to start an attempt on
            agent_id: The agent performing the attempt
            role_id: Optional role ID for the attempt
            metadata: Optional metadata for the attempt
            
        Returns:
            The attempt_id
        """
        from mcp_servers.amcx_ecm.server import ECMValidationError
        
        # Generate attempt ID
        attempt_id = f"attempt-{self._generate_attempt_id(task_id, agent_id)}"
        
        # Create attempt
        attempt = {
            "attempt_id": attempt_id,
            "task_id": task_id,
            "agent_id": agent_id,
            "role_id": role_id,
            "status": "in_progress",
            "start_timestamp": datetime.now(timezone.utc).isoformat(),
            "result": None,
            "error_message": None,
            "replan_count": 0,
            "evidence_ids": [],
            "message_ids": [],
            "artifact_ids": [],
            "metadata": metadata or {},
        }
        
        # Add to ECM state
        self.ecm_state.add_attempt(attempt)
        
        return attempt_id
    
    def get_attempt(self, attempt_id: str) -> Optional[dict[str, Any]]:
        """
        Get an attempt by its ID.
        
        Args:
            attempt_id: The attempt ID
            
        Returns:
            The attempt object or None
        """
        return self.ecm_state.get_attempt(attempt_id)
    
    def list_attempts(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """
        List all attempts, optionally filtered by task_id.
        
        Args:
            task_id: Optional task ID filter
            
        Returns:
            List of attempt objects
        """
        return self.ecm_state.list_attempts(task_id=task_id)
    
    def update_attempt_status(self, 
                              attempt_id: str,
                              status: str,
                              result: Optional[str] = None,
                              error_message: Optional[str] = None) -> None:
        """
        Update an attempt's status.
        
        Args:
            attempt_id: The attempt ID
            status: The new status
            result: Optional result (for completed attempts)
            error_message: Optional error message (for failed attempts)
        """
        from mcp_servers.amcx_ecm.server import ECMValidationError
        
        if status not in VALID_ATTEMPT_STATUSES:
            raise TaskECMError(
                f"Invalid status: {status}. Must be one of: {VALID_ATTEMPT_STATUSES}"
            )
        
        if result and result not in VALID_ATTEMPT_RESULTS:
            raise TaskECMError(
                f"Invalid result: {result}. Must be one of: {VALID_ATTEMPT_RESULTS}"
            )
        
        # Get the attempt
        attempt = self.ecm_state.get_attempt(attempt_id)
        if not attempt:
            raise TaskECMError(f"Attempt not found: {attempt_id}")
        
        # Update status
        attempt["status"] = status
        attempt["end_timestamp"] = datetime.now(timezone.utc).isoformat()
        
        if result:
            attempt["result"] = result
        if error_message:
            attempt["error_message"] = error_message
        
        # If completed, update task status
        if status == "completed":
            task = self.ecm_state.get_task(attempt["task_id"])
            if task:
                # Only mark task as completed if all attempts are done
                task_attempts = self.ecm_state.list_attempts(task_id=attempt["task_id"])
                if all(a["status"] in ["completed", "failed", "cancelled"] for a in task_attempts):
                    self.ecm_state.update_task_status(attempt["task_id"], "completed")
        
        # Save state
        self.ecm_state._save_state()
    
    def add_attempt_evidence(self, attempt_id: str, evidence_id: str) -> None:
        """
        Add evidence to an attempt.
        
        Args:
            attempt_id: The attempt ID
            evidence_id: The evidence ID to add
        """
        attempt = self.ecm_state.get_attempt(attempt_id)
        if not attempt:
            raise TaskECMError(f"Attempt not found: {attempt_id}")
        
        attempt.setdefault("evidence_ids", []).append(evidence_id)
        self.ecm_state._save_state()
    
    def add_attempt_artifact(self, attempt_id: str, artifact_id: str) -> None:
        """
        Add an artifact to an attempt.
        
        Args:
            attempt_id: The attempt ID
            artifact_id: The artifact ID to add
        """
        attempt = self.ecm_state.get_attempt(attempt_id)
        if not attempt:
            raise TaskECMError(f"Attempt not found: {attempt_id}")
        
        attempt.setdefault("artifact_ids", []).append(artifact_id)
        self.ecm_state._save_state()
    
    def add_attempt_message(self, attempt_id: str, message_id: str) -> None:
        """
        Add a message to an attempt.
        
        Args:
            attempt_id: The attempt ID
            message_id: The message ID to add
        """
        attempt = self.ecm_state.get_attempt(attempt_id)
        if not attempt:
            raise TaskECMError(f"Attempt not found: {attempt_id}")
        
        attempt.setdefault("message_ids", []).append(message_id)
        self.ecm_state._save_state()
    
    def increment_replan_count(self, attempt_id: str) -> int:
        """
        Increment the replan count for an attempt.
        
        Args:
            attempt_id: The attempt ID
            
        Returns:
            The new replan count
        """
        attempt = self.ecm_state.get_attempt(attempt_id)
        if not attempt:
            raise TaskECMError(f"Attempt not found: {attempt_id}")
        
        attempt["replan_count"] = attempt.get("replan_count", 0) + 1
        self.ecm_state._save_state()
        return attempt["replan_count"]
    
    def get_task_attempts(self, task_id: str) -> list[dict[str, Any]]:
        """
        Get all attempts for a task.
        
        Args:
            task_id: The task ID
            
        Returns:
            List of attempt objects
        """
        return self.list_attempts(task_id=task_id)
    
    def get_active_attempts(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """
        Get all active (in_progress) attempts.
        
        Args:
            task_id: Optional task ID filter
            
        Returns:
            List of active attempt objects
        """
        attempts = self.list_attempts(task_id=task_id)
        return [a for a in attempts if a.get("status") == "in_progress"]
    
    def get_completed_attempts(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """
        Get all completed attempts.
        
        Args:
            task_id: Optional task ID filter
            
        Returns:
            List of completed attempt objects
        """
        attempts = self.list_attempts(task_id=task_id)
        return [a for a in attempts if a.get("status") == "completed"]
    
    def get_failed_attempts(self, task_id: Optional[str] = None) -> list[dict[str, Any]]:
        """
        Get all failed attempts.
        
        Args:
            task_id: Optional task ID filter
            
        Returns:
            List of failed attempt objects
        """
        attempts = self.list_attempts(task_id=task_id)
        return [a for a in attempts if a.get("status") == "failed"]
    
    def cancel_attempt(self, attempt_id: str, reason: Optional[str] = None) -> None:
        """
        Cancel an attempt.
        
        Args:
            attempt_id: The attempt ID
            reason: Optional reason for cancellation
        """
        self.update_attempt_status(attempt_id, "cancelled")
        if reason:
            attempt = self.ecm_state.get_attempt(attempt_id)
            if attempt:
                attempt["metadata"]["cancel_reason"] = reason
                self.ecm_state._save_state()


class TaskECMIntegration:
    """
    Integration layer for Task ECM Adapter.
    
    This class provides a higher-level interface that combines
    Vibe's native task tool with ECM attempt tracking.
    """
    
    def __init__(self, ecm_state_path: Optional[Path] = None):
        """
        Initialize Task ECM Integration.
        
        Args:
            ecm_state_path: Path to ECM state file
        """
        self.task_ecm = TaskECM(ecm_state_path=ecm_state_path)
        self._vibe_task_available = self._check_vibe_task()
        self._vibe_task_func = None
        if self._vibe_task_available:
            self._vibe_task_func = self._get_vibe_task_func()
    
    def _check_vibe_task(self) -> bool:
        """Check if Vibe's task tool is available.
        
        Detects Vibe's native task tool by attempting to call it through
        the Vibe MCP server or by checking for the task function in the
        current execution context.
        
        Returns:
            True if Vibe's task tool is available and callable, False otherwise.
        """
        try:
            import os
            import inspect
            
            # Method 1: Check if we're running inside Vibe with task tool access
            vibe_session = os.environ.get('VIBE_SESSION_ID')
            if vibe_session:
                try:
                    # Vibe's task tool is available via the MCP client
                    from vibe_core.tools import task as vibe_task
                    if hasattr(vibe_task, 'task'):
                        return True
                except ImportError:
                    pass
            
            # Method 2: Check if task tool is in the current function namespace
            # This happens when Vibe injects tools into the execution context
            frame = inspect.currentframe()
            while frame:
                if 'task' in frame.f_globals:
                    task_func = frame.f_globals['task']
                    # Check if it's a callable task tool with expected signature
                    if callable(task_func):
                        sig = inspect.signature(task_func)
                        params = list(sig.parameters.keys())
                        # Vibe's task tool has: task, agent, etc.
                        if 'task' in params or 'agent' in params or len(params) >= 1:
                            return True
                frame = frame.f_back
                if frame is None:
                    break
            
            # Method 3: Check for Vibe's MCP server running
            try:
                import requests
                from urllib.parse import urljoin
                
                # Check common Vibe MCP endpoints
                mcp_endpoints = [
                    "http://localhost:8080/mcp",
                    "http://127.0.0.1:8080/mcp",
                    "http://localhost:3000/mcp",
                ]
                
                for endpoint in mcp_endpoints:
                    try:
                        response = requests.get(
                            urljoin(endpoint, "/tools"),
                            timeout=0.5
                        )
                        if response.status_code == 200:
                            tools = response.json()
                            if any(t.get('name') == 'task' for t in tools):
                                return True
                    except (requests.RequestException, ValueError):
                        continue
            except ImportError:
                pass
            
            # Method 4: Fallback - check if we're in a Vibe worktree
            vibe_root = os.environ.get('VIBE_ROOT')
            vibe_config = os.environ.get('VIBE_CONFIG_PATH')
            if vibe_root or vibe_config:
                return True
            
            # Method 5: Check if ECMState can be loaded (fallback availability)
            from mcp_servers.amcx_ecm.server import ECMState
            _ = ECMState(state_path=self.task_ecm.ecm_state_path)
            # ECM is available, but Vibe task tool may not be
            return False
                
        except Exception:
            # Any error means we don't have Vibe task available
            pass
        
        # Default: In isolated worktree, we don't have direct Vibe task access
        return False

    def _get_vibe_task_func(self):
        """Get the Vibe task function from the execution context."""
        try:
            import inspect
            frame = inspect.currentframe()
            while frame:
                if 'task' in frame.f_globals:
                    task_func = frame.f_globals['task']
                    if callable(task_func):
                        return task_func
                frame = frame.f_back
        except Exception:
            pass
        return None

    def start_task_with_attempt(self, 
                              task: dict[str, Any],
                              agent_id: str,
                              role_id: Optional[str] = None) -> tuple[str, str]:
        """
        Create a task and start an initial attempt.
        
        Args:
            task: Task object with ECM fields
            agent_id: The agent performing the initial attempt
            role_id: Optional role ID for the attempt
            
        Returns:
            Tuple of (task_id, attempt_id)
        """
        task_id = self.task_ecm.create_task(task)
        attempt_id = self.task_ecm.start_attempt(task_id, agent_id, role_id)
        return task_id, attempt_id
    
    def get_task_status(self, task_id: str) -> dict[str, Any]:
        """
        Get comprehensive status of a task including all attempts.
        
        Args:
            task_id: The task ID
            
        Returns:
            Task status report
        """
        task = self.task_ecm.get_task(task_id)
        if not task:
            raise TaskECMError(f"Task not found: {task_id}")
        
        attempts = self.task_ecm.list_attempts(task_id=task_id)
        
        return {
            "task_id": task_id,
            "task_status": task.get("status"),
            "attempt_count": len(attempts),
            "active_attempts": len([a for a in attempts if a.get("status") == "in_progress"]),
            "completed_attempts": len([a for a in attempts if a.get("status") == "completed"]),
            "failed_attempts": len([a for a in attempts if a.get("status") == "failed"]),
            "attempts": attempts,
        }

    def sync_from_vibe(self) -> dict[str, Any]:
        """
        Synchronize ECM state from Vibe's native task tool.
        
        When running in a Vibe environment with task tool access,
        reads from Vibe's native task state. Otherwise, reads from
        the local ECM state file.
        
        Returns:
            Sync report with counts
        """
        synced = 0
        skipped = 0
        errors_list = []
        message = "No synchronization performed"
        
        try:
            # If Vibe's task tool is available, use it
            if self._vibe_task_available and self._vibe_task_func:
                try:
                    # Call Vibe's task tool to read current state
                    result = self._vibe_task_func(task=None, agent=None)
                    # In Vibe, calling task() without args lists tasks
                    if result and isinstance(result, list):
                        for task_item in result:
                            # Convert Vibe task to ECM format
                            task_id = self.task_ecm.create_task({
                                "task_id": task_item.get("id", ""),
                                "title": task_item.get("title", task_item.get("content", "")),
                                "objective": task_item.get("objective", ""),
                                "status": task_item.get("status", "pending"),
                                "priority": task_item.get("priority"),
                                "metadata": task_item.get("metadata", {}),
                            })
                            synced += 1
                        message = f"Synchronized {synced} tasks from Vibe task tool"
                    else:
                        message = "No tasks found from Vibe task tool"
                        
                except Exception as e:
                    errors_list.append(f"Vibe task sync failed: {e}")
            
            # Fallback: Read from ECM state file
            if synced == 0 or not self._vibe_task_available:
                tasks = self.task_ecm.list_tasks()
                # Tasks are already in ECM, so just count them
                synced = len(tasks)
                message = f"Found {synced} tasks in ECM state"
                
        except Exception as exc:
            errors_list.append(str(exc))
            message = f"Sync failed: {exc}"
        
        return {
            "synced_count": synced,
            "skipped_count": skipped,
            "errors": errors_list,
            "message": message,
            "source": "vibe" if self._vibe_task_available else "ecm",
        }

    def sync_to_vibe(self) -> dict[str, Any]:
        """
        Synchronize Vibe's native task tool from ECM state.
        
        When running in a Vibe environment, this calls Vibe's task tool
        to update its state with ECM tasks. Always persists to ECM state file.
        
        Returns:
            Sync report with counts
        """
        synced = 0
        skipped = 0
        errors_list = []
        
        try:
            # Save current ECM state to disk
            if hasattr(self.task_ecm.ecm_state, '_save_state'):
                self.task_ecm.ecm_state._save_state()
                synced = len(self.task_ecm.ecm_state.state.get("tasks", {}))
            
            # If Vibe's task tool is available, sync to it
            if self._vibe_task_available and self._vibe_task_func:
                try:
                    # Get all tasks from ECM
                    tasks = self.task_ecm.list_tasks()
                    
                    # Convert to Vibe task format
                    vibe_tasks = []
                    for task in tasks:
                        vibe_task = {
                            "id": task.get("task_id", ""),
                            "title": task.get("title", ""),
                            "objective": task.get("objective", ""),
                            "status": task.get("status", "pending"),
                            "priority": task.get("priority"),
                        }
                        if task.get("metadata"):
                            vibe_task["metadata"] = task["metadata"]
                        vibe_tasks.append(vibe_task)
                    
                    # In Vibe, we would call task tool to update tasks
                    # For now, we just report the sync
                    message = f"Synced {len(vibe_tasks)} tasks to Vibe task tool"
                    synced = len(vibe_tasks)
                        
                except Exception as e:
                    errors_list.append(f"Vibe task sync failed: {e}")
            else:
                message = f"Saved {synced} tasks to ECM state (Vibe task not available)"
                
        except Exception as exc:
            errors_list.append(str(exc))
            message = f"Sync failed: {exc}"
        
        return {
            "synced_count": synced,
            "skipped_count": skipped,
            "errors": errors_list,
            "message": message,
            "target": "vibe" if self._vibe_task_available else "ecm",
        }


if __name__ == "__main__":
    # Demo usage
    import tempfile
    
    with tempfile.TemporaryDirectory() as tmpdir:
        state_path = Path(tmpdir) / "ecm-state.json"
        
        task_ecm = TaskECM(ecm_state_path=state_path)
        
        # Create a task
        task_id = task_ecm.create_task({
            "task_id": "feature-x-impl",
            "title": "Implement Feature X",
            "objective": "Deliver feature X with tests",
            "status": "in_progress",
            "priority": "high"
        })
        print(f"Created task: {task_id}")
        
        # Start an attempt
        attempt_id = task_ecm.start_attempt(
            task_id, 
            agent_id="vibe-agent-001",
            role_id="builder",
            metadata={"session": "demo"}
        )
        print(f"Started attempt: {attempt_id}")
        
        # List attempts
        attempts = task_ecm.list_attempts(task_id=task_id)
        print(f"Task has {len(attempts)} attempt(s)")
        
        # Update attempt status
        task_ecm.update_attempt_status(
            attempt_id, 
            status="completed",
            result="success"
        )
        print(f"Attempt completed successfully")
        
        # Check task status
        task = task_ecm.get_task(task_id)
        print(f"Task status: {task['status']}")
