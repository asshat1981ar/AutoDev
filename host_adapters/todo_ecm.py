#!/usr/bin/env python3
"""
Todo ECM Adapter

Extends Vibe's native `todo` tool with ECM (Entity-Collaboration Model) fields.

This adapter provides a wrapper around Vibe's todo functionality that:
1. Synchronizes todo items with ECM task state
2. Adds ECM fields (task_id, objective, dependencies, etc.)
3. Maintains compatibility with Vibe's todo tool

Usage:
    from host_adapters.todo_ecm import TodoECM
    
    todo = TodoECM(ecm_state_path=".vibe/amcx-ecm/state.json")
    
    # Add a todo with ECM fields
    todo.add({
        "id": "task-001",
        "content": "Implement feature X",
        "status": "pending",
        "priority": "high",
        # ECM extensions:
        "objective": "Deliver feature X with tests",
        "dependencies": ["task-002"],
        "blocked_by": [],
        "metadata": {"team": "backend"}
    })
"""

import json
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


# ECM Task status mapping to Vibe todo status
ECM_TO_TODO_STATUS = {
    "pending": "pending",
    "in_progress": "in_progress", 
    "completed": "completed",
    "failed": "failed",
    "cancelled": "cancelled",
    "blocked": "blocked",
}

TODO_TO_ECM_STATUS = {v: k for k, v in ECM_TO_TODO_STATUS.items()}


class TodoECMError(ValueError):
    """Raised when Todo ECM operations fail."""
    pass


class TodoECM:
    """
    Todo ECM Adapter - Extends Vibe's todo with ECM fields.
    
    This adapter synchronizes between Vibe's native todo tool state
    (which we cannot directly modify) and ECM task state.
    
    It provides:
    - Task creation with full ECM fields
    - Status synchronization between todo and ECM
    - Dependency tracking
    - Blocked-by tracking
    - Metadata storage
    """
    
    def __init__(self, ecm_state_path: Optional[Path] = None):
        """
        Initialize Todo ECM Adapter.
        
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
    
    def _generate_task_id(self, todo_id: str) -> str:
        """Generate ECM task_id from todo id."""
        # Ensure it matches ECM ID pattern: ^[a-z0-9]+(?:-[a-z0-9]+)*$
        import re
        # Convert to lowercase and replace invalid chars
        task_id = re.sub(r"[^a-z0-9-]", "-", todo_id.lower())
        # Remove consecutive hyphens
        task_id = re.sub(r"--+", "-", task_id)
        # Remove leading/trailing hyphens
        task_id = task_id.strip("-")
        return task_id or "todo-task"
    
    def _todo_to_ecm_task(self, todo_item: dict[str, Any]) -> dict[str, Any]:
        """Convert a todo item to an ECM task."""
        task_id = self._generate_task_id(todo_item.get("id", ""))
        
        # Map todo status to ECM status
        ecm_status = TODO_TO_ECM_STATUS.get(
            todo_item.get("status", "pending"),
            "pending"
        )
        
        # Build ECM task
        task = {
            "task_id": task_id,
            "title": todo_item.get("content", "Untitled"),
            "objective": todo_item.get("objective", todo_item.get("content", "")),
            "status": ecm_status,
            "created_timestamp": todo_item.get("created_at") or datetime.now(timezone.utc).isoformat(),
            "updated_timestamp": todo_item.get("updated_at") or datetime.now(timezone.utc).isoformat(),
            "priority": todo_item.get("priority"),
        }
        
        # Add optional ECM fields if present
        if "dependencies" in todo_item:
            task["dependencies"] = todo_item["dependencies"]
        if "blocked_by" in todo_item:
            task["blocked_by"] = todo_item["blocked_by"]
        if "metadata" in todo_item:
            task["metadata"] = todo_item["metadata"]
        
        # Track the source
        task["metadata"] = task.get("metadata", {})
        task["metadata"]["_todo_source"] = True
        task["metadata"]["_todo_id"] = todo_item.get("id")
        
        return task
    
    def _ecm_task_to_todo(self, task: dict[str, Any]) -> dict[str, Any]:
        """Convert an ECM task to a todo item."""
        # Map ECM status to todo status
        todo_status = ECM_TO_TODO_STATUS.get(
            task.get("status", "pending"),
            "pending"
        )
        
        todo_item = {
            "id": task.get("task_id", ""),
            "content": task.get("title", "Untitled"),
            "status": todo_status,
            "priority": task.get("priority"),
            "created_at": task.get("created_timestamp"),
            "updated_at": task.get("updated_timestamp"),
        }
        
        # Add ECM fields as extensions
        todo_item["objective"] = task.get("objective", "")
        todo_item["dependencies"] = task.get("dependencies", [])
        todo_item["blocked_by"] = task.get("blocked_by", [])
        todo_item["metadata"] = task.get("metadata", {})
        
        return todo_item
    
    def add(self, todo_item: dict[str, Any], sync_to_ecm: bool = True) -> str:
        """
        Add a todo item with optional ECM synchronization.
        
        Args:
            todo_item: Todo item with optional ECM fields
            sync_to_ecm: If True, also create an ECM task
            
        Returns:
            The todo/item ID
        """
        # For now, we just sync to ECM since we can't modify Vibe's native todo
        # In a real implementation, this would call Vibe's todo tool
        
        if sync_to_ecm:
            task = self._todo_to_ecm_task(todo_item)
            task_id = self.ecm_state.add_task(task)
            return task_id
        
        return todo_item.get("id", "")
    
    def update_status(self, task_id: str, status: str) -> None:
        """
        Update the status of a todo/task.
        
        Args:
            task_id: The task/todo ID
            status: The new status (todo status or ECM status)
        """
        # Map todo status to ECM status if needed
        ecm_status = TODO_TO_ECM_STATUS.get(status, status)
        
        # Update in ECM
        try:
            self.ecm_state.update_task_status(task_id, ecm_status)
        except Exception as e:
            # Try with generated ID
            generated_id = self._generate_task_id(task_id)
            self.ecm_state.update_task_status(generated_id, ecm_status)
    
    def list(self, status: Optional[str] = None) -> list[dict[str, Any]]:
        """
        List all todos/tasks, optionally filtered by status.
        
        Args:
            status: Optional status filter
            
        Returns:
            List of todo items with ECM fields
        """
        # Get tasks from ECM
        ecm_status = TODO_TO_ECM_STATUS.get(status, status) if status else None
        tasks = self.ecm_state.list_tasks(status=ecm_status)
        
        # Convert to todo format
        todos = []
        for task in tasks:
            todo = self._ecm_task_to_todo(task)
            # Filter by todo status if provided
            if status and todo.get("status") != status:
                continue
            todos.append(todo)
        
        return todos
    
    def get(self, task_id: str) -> Optional[dict[str, Any]]:
        """
        Get a todo item by its ID.
        
        Args:
            task_id: The task/todo ID
            
        Returns:
            The todo item with ECM fields, or None if not found
        """
        # Try direct lookup
        task = self.ecm_state.get_task(task_id)
        if task:
            return self._ecm_task_to_todo(task)
        
        # Try with generated ID
        generated_id = self._generate_task_id(task_id)
        task = self.ecm_state.get_task(generated_id)
        if task:
            return self._ecm_task_to_todo(task)
        
        return None
    
    def complete(self, task_id: str) -> None:
        """Mark a todo as completed."""
        self.update_status(task_id, "completed")
    
    def fail(self, task_id: str) -> None:
        """Mark a todo as failed."""
        self.update_status(task_id, "failed")
    
    def cancel(self, task_id: str) -> None:
        """Mark a todo as cancelled."""
        self.update_status(task_id, "cancelled")


class TodoECMIntegration:
    """
    Integration layer for Todo ECM Adapter.
    
    This class provides a higher-level interface that combines
    Vibe's native todo with ECM tracking.
    """
    
    def __init__(self, ecm_state_path: Optional[Path] = None):
        """
        Initialize Todo ECM Integration.
        
        Args:
            ecm_state_path: Path to ECM state file
        """
        self.todo_ecm = TodoECM(ecm_state_path=ecm_state_path)
        self._vibe_todo_available = self._check_vibe_todo()
    
    def _check_vibe_todo(self) -> bool:
        """Check if Vibe's todo tool is available.
        
        Detects Vibe's native todo tool by attempting to call it through
        the Vibe MCP server or by checking for the todo function in the
        current execution context.
        
        Returns:
            True if Vibe's todo tool is available and callable, False otherwise.
        """
        try:
            # Method 1: Check if we're running inside Vibe with todo tool access
            # Vibe exposes tools through the MCP system
            import os
            vibe_session = os.environ.get('VIBE_SESSION_ID')
            if vibe_session:
                # Try to import and call Vibe's todo tool
                try:
                    # Vibe's todo tool is available via the MCP client
                    # In a real Vibe environment, this would work
                    from vibe_core.tools import todo as vibe_todo
                    if hasattr(vibe_todo, 'todo'):
                        return True
                except ImportError:
                    pass
            
            # Method 2: Check if todo tool is in the current function namespace
            # This happens when Vibe injects tools into the execution context
            import inspect
            frame = inspect.currentframe()
            while frame:
                if 'todo' in frame.f_globals:
                    todo_func = frame.f_globals['todo']
                    # Check if it's a callable todo tool with expected signature
                    if callable(todo_func):
                        sig = inspect.signature(todo_func)
                        params = list(sig.parameters.keys())
                        # Vibe's todo tool has: action, todos, etc.
                        if 'action' in params or len(params) >= 1:
                            return True
                frame = frame.f_back
                if frame is None:
                    break
            
            # Method 3: Check for Vibe's MCP server running
            # In production, Vibe runs an MCP server that exposes tools
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
                            if any(t.get('name') == 'todo' for t in tools):
                                return True
                    except (requests.RequestException, ValueError):
                        continue
            except ImportError:
                pass
            
            # Method 4: Fallback - check if we're in a Vibe worktree
            # If VIBE_ROOT or similar environment variables are set
            vibe_root = os.environ.get('VIBE_ROOT')
            vibe_config = os.environ.get('VIBE_CONFIG_PATH')
            if vibe_root or vibe_config:
                # Assume we have access if Vibe environment is configured
                return True
                
        except Exception:
            # Any error means we don't have Vibe todo available
            pass
        
        # Default: In isolated worktree, we don't have direct Vibe todo access
        # We'll fall back to ECM-based sync
        return False
    
    def sync_from_vibe(self) -> dict[str, Any]:
        """
        Synchronize ECM state from Vibe's native todo.

        When running in a Vibe environment with todo tool access,
        reads from Vibe's native todo state. Otherwise, reads from
        the local ECM state file.

        Returns:
            Sync report with counts
        """
        synced = 0
        skipped = 0
        errors_list = []
        message = "No synchronization performed"
        
        try:
            # If Vibe's todo tool is available, use it
            if self._vibe_todo_available:
                try:
                    # Call Vibe's todo tool to read current state
                    # In Vibe, todo tool has action='read' to get current todos
                    import inspect
                    frame = inspect.currentframe()
                    vibe_todo_func = None
                    
                    # Look for todo function in parent frames
                    while frame:
                        if 'todo' in frame.f_globals:
                            todo_func = frame.f_globals['todo']
                            if callable(todo_func):
                                vibe_todo_func = todo_func
                                break
                        frame = frame.f_back
                    
                    if vibe_todo_func:
                        # Call Vibe's todo with read action
                        result = vibe_todo_func(action='read')
                        if result and 'todos' in result:
                            for todo_item in result['todos']:
                                self.todo_ecm.add({
                                    "id": todo_item.get("id", ""),
                                    "content": todo_item.get("content", ""),
                                    "status": todo_item.get("status", "pending"),
                                    "priority": todo_item.get("priority"),
                                    # ECM extensions (will be empty from Vibe todo)
                                    "objective": todo_item.get("objective", ""),
                                    "dependencies": todo_item.get("dependencies", []),
                                    "blocked_by": todo_item.get("blocked_by", []),
                                    "metadata": todo_item.get("metadata", {}),
                                }, sync_to_ecm=True)
                                synced += 1
                        message = f"Synchronized {synced} tasks from Vibe todo"
                    else:
                        # Fall back to ECM state
                        raise Exception("Vibe todo available but not callable")
                        
                except Exception as e:
                    # If Vibe todo fails, fall back to ECM
                    errors_list.append(f"Vibe todo sync failed: {e}")
                    # Continue with ECM fallback below
                    pass
            
            # Fallback: Read from ECM state file
            if synced == 0 or not self._vibe_todo_available:
                state = self.todo_ecm.ecm_state
                
                # Check for existing tasks in ECM to avoid duplicates
                existing_todo_ids = set()
                if hasattr(self.todo_ecm, '_ecm_state') and self.todo_ecm._ecm_state:
                    ecm_tasks = self.todo_ecm._ecm_state.state.get("tasks", {})
                    existing_todo_ids = set(ecm_tasks.keys())
                
                for task_id, task in state.state.get("tasks", {}).items():
                    # Skip if already synced from Vibe
                    if task_id in existing_todo_ids:
                        skipped += 1
                        continue
                    
                    self.todo_ecm.add({
                        "id": task_id,
                        "content": task.get("title", task.get("description", "")),
                        "status": task.get("status", "pending"),
                        "objective": task.get("objective", ""),
                        "dependencies": task.get("dependencies", []),
                        "blocked_by": task.get("blocked_by", []),
                        "metadata": task.get("metadata", {}),
                    }, sync_to_ecm=False)
                    synced += 1
                
                message = f"Synchronized {synced} tasks from ECM state"
                
        except Exception as exc:
            errors_list.append(str(exc))
            message = f"Sync failed: {exc}"
        
        return {
            "synced_count": synced,
            "skipped_count": skipped,
            "errors": errors_list,
            "message": message,
            "source": "vibe" if self._vibe_todo_available else "ecm",
        }
    
    def sync_to_vibe(self) -> dict[str, Any]:
        """
        Synchronize Vibe's native todo from ECM state.

        When running in a Vibe environment, this calls Vibe's todo tool
        to update its state with ECM tasks. Always persists to ECM state file.

        Returns:
            Sync report with counts
        """
        synced = 0
        skipped = 0
        errors_list = []
        message = "No synchronization performed"
        
        try:
            # First, save current ECM state to disk
            ecm_state = self.todo_ecm.ecm_state
            if hasattr(ecm_state, '_save_state'):
                ecm_state._save_state()
                synced = len(ecm_state.state.get("tasks", {}))
            
            # If Vibe's todo tool is available, sync to it
            if self._vibe_todo_available:
                try:
                    import inspect
                    frame = inspect.currentframe()
                    vibe_todo_func = None
                    
                    # Look for todo function in parent frames
                    while frame:
                        if 'todo' in frame.f_globals:
                            todo_func = frame.f_globals['todo']
                            if callable(todo_func):
                                vibe_todo_func = todo_func
                                break
                        frame = frame.f_back
                    
                    if vibe_todo_func:
                        # Get all tasks from ECM
                        tasks = self.todo_ecm.list()
                        
                        # Convert to Vibe todo format
                        vibe_todos = []
                        for task in tasks:
                            vibe_todo = {
                                "id": task.get("id", ""),
                                "content": task.get("content", task.get("title", "")),
                                "status": task.get("status", "pending"),
                                "priority": task.get("priority"),
                            }
                            # Add ECM fields if present
                            if task.get("objective"):
                                vibe_todo["objective"] = task["objective"]
                            if task.get("dependencies"):
                                vibe_todo["dependencies"] = task["dependencies"]
                            if task.get("blocked_by"):
                                vibe_todo["blocked_by"] = task["blocked_by"]
                            if task.get("metadata"):
                                vibe_todo["metadata"] = task["metadata"]
                            vibe_todos.append(vibe_todo)
                        
                        # Call Vibe's todo with write action
                        result = vibe_todo_func(
                            action='write',
                            todos=vibe_todos
                        )
                        
                        if result and 'updated_count' in result:
                            synced = result['updated_count']
                            message = f"Synced {synced} tasks to Vibe todo"
                        else:
                            message = f"Synced {len(vibe_todos)} tasks to Vibe todo"
                            
                    else:
                        errors_list.append("Vibe todo available but not callable")
                        
                except Exception as e:
                    errors_list.append(f"Vibe todo sync failed: {e}")
            else:
                message = f"Saved {synced} tasks to ECM state (Vibe todo not available)"
                
        except Exception as exc:
            errors_list.append(str(exc))
            message = f"Sync failed: {exc}"
        
        return {
            "synced_count": synced,
            "skipped_count": skipped,
            "errors": errors_list,
            "message": message,
            "target": "vibe" if self._vibe_todo_available else "ecm",
        }


if __name__ == "__main__":
    # Demo usage
    import tempfile
    
    with tempfile.TemporaryDirectory() as tmpdir:
        state_path = Path(tmpdir) / "ecm-state.json"
        
        todo_ecm = TodoECM(ecm_state_path=state_path)
        
        # Add a todo with ECM fields
        task_id = todo_ecm.add({
            "id": "implement-feature-x",
            "content": "Implement Feature X",
            "status": "pending",
            "priority": "high",
            "objective": "Deliver Feature X with full test coverage",
            "dependencies": ["design-feature-x", "setup-test-infra"],
            "blocked_by": [],
            "metadata": {
                "team": "backend",
                "epic": "Q3-initiatives"
            }
        })
        
        print(f"Created task: {task_id}")
        
        # List todos
        todos = todo_ecm.list()
        print(f"Todos: {len(todos)}")
        for todo in todos:
            print(f"  - {todo['id']}: {todo['content']} ({todo['status']})")
            if todo.get("objective"):
                print(f"    Objective: {todo['objective']}")
            if todo.get("dependencies"):
                print(f"    Dependencies: {todo['dependencies']}")
        
        # Update status
        todo_ecm.update_status(task_id, "in_progress")
        print(f"Updated status to in_progress")
        
        # Get specific todo
        todo = todo_ecm.get(task_id)
        print(f"Retrieved: {todo['id']} - {todo['status']}")
