import uuid
import datetime
from datetime import datetime, timezone, timedelta
from typing import Dict, Any, List, Optional, Set

class ECMTaskError(Exception):
    pass

class ECMTask:
    """
    ECM Collaboration Task State Machine (ecm.task.v1).
    Supports all 16 canonical lifecycle states with strictly validated transitions.
    """
    ALL_STATES = [
        "PROPOSED", "READY", "CLAIMED", "RUNNING", "BLOCKED",
        "REVIEW_PENDING", "ACCEPTANCE_PENDING", "RETRY_WAIT",
        "CANCEL_REQUESTED", "COMPLETED", "FAILED", "EXPIRED",
        "CANCELLED", "CANCELLED_WITH_EFFECT", "CANCELLED_EFFECT_UNKNOWN", "MANUAL_REQUIRED"
    ]

    ALLOWED_TRANSITIONS = {
        "PROPOSED": ["READY", "CANCELLED"],
        "READY": ["CLAIMED", "EXPIRED", "CANCELLED"],
        "CLAIMED": ["RUNNING", "READY", "CANCELLED"],
        "RUNNING": ["BLOCKED", "REVIEW_PENDING", "ACCEPTANCE_PENDING", "FAILED", "CANCEL_REQUESTED"],
        "BLOCKED": ["RUNNING", "CANCEL_REQUESTED", "FAILED", "MANUAL_REQUIRED"],
        "REVIEW_PENDING": ["ACCEPTANCE_PENDING", "RETRY_WAIT", "FAILED"],
        "ACCEPTANCE_PENDING": ["COMPLETED", "RETRY_WAIT", "MANUAL_REQUIRED"],
        "RETRY_WAIT": ["READY", "FAILED", "MANUAL_REQUIRED"],
        "CANCEL_REQUESTED": ["CANCELLED", "CANCELLED_WITH_EFFECT", "CANCELLED_EFFECT_UNKNOWN"],
        "COMPLETED": [],
        "FAILED": ["RETRY_WAIT", "MANUAL_REQUIRED"],
        "EXPIRED": [],
        "CANCELLED": [],
        "CANCELLED_WITH_EFFECT": [],
        "CANCELLED_EFFECT_UNKNOWN": ["MANUAL_REQUIRED"],
        "MANUAL_REQUIRED": ["READY", "CANCELLED", "COMPLETED"]
    }

    def __init__(
        self,
        task_id: str,
        title: str,
        acceptance_contract_id: str,
        budget_limit_tokens: int,
        initial_state: str = "PROPOSED",
        assigned_role: Optional[str] = None
    ):
        self.task_id = task_id
        self.title = title
        self.acceptance_contract_id = acceptance_contract_id
        self.budget_limit_tokens = budget_limit_tokens
        self.tokens_used = 0
        if initial_state not in self.ALL_STATES:
            raise ECMTaskError(f"Invalid initial state: {initial_state}")
        self.task_state = initial_state
        self.assigned_role = assigned_role
        self.created_at = datetime.now(timezone.utc).isoformat()
        self.history: List[Dict[str, str]] = [{"from": "NONE", "to": self.task_state, "timestamp": self.created_at}]

    def transition(self, target_state: str, reason: str = "") -> None:
        if target_state not in self.ALL_STATES:
            raise ECMTaskError(f"Unknown target state: {target_state}")
        allowed = self.ALLOWED_TRANSITIONS.get(self.task_state, [])
        if target_state not in allowed:
            raise ECMTaskError(
                f"Illegal transition from {self.task_state} to {target_state}. Allowed: {allowed}"
            )
        old_state = self.task_state
        self.task_state = target_state
        self.history.append({
            "from": old_state,
            "to": target_state,
            "reason": reason,
            "timestamp": datetime.now(timezone.utc).isoformat()
        })

    def consume_tokens(self, count: int) -> None:
        if self.tokens_used + count > self.budget_limit_tokens:
            raise ECMTaskError(
                f"Token budget exceeded: limit is {self.budget_limit_tokens}, requested additional {count} with {self.tokens_used} already used."
            )
        self.tokens_used += count

    def to_dict(self) -> Dict[str, Any]:
        return {
            "task_id": self.task_id,
            "title": self.title,
            "task_state": self.task_state,
            "acceptance_contract_id": self.acceptance_contract_id,
            "budget_limit_tokens": self.budget_limit_tokens,
            "tokens_used": self.tokens_used,
            "assigned_role": self.assigned_role,
            "created_at": self.created_at,
            "history": self.history
        }

class RoleLeaseManager:
    """
    Manages active role leases for ECM tasks (ecm.lease.v1).
    Enforces time-bounded mutual exclusion per role/task pair.
    """
    def __init__(self):
        self._leases: Dict[str, Dict[str, Any]] = {} # lease_id -> lease_info

    def acquire_lease(self, task_id: str, role: str, agent_id: str, duration_seconds: int = 300) -> str:
        now = datetime.now(timezone.utc)
        # Check active leases for same task & role
        for l_id, info in self._leases.items():
            if info["task_id"] == task_id and info["role"] == role:
                expires_at = datetime.fromisoformat(info["expires_at"])
                if now < expires_at and info["holder_agent_id"] != agent_id:
                    raise ECMTaskError(
                        f"Role '{role}' for task '{task_id}' is already leased to agent '{info['holder_agent_id']}' until {info['expires_at']}"
                    )

        lease_id = str(uuid.uuid4())
        expires_at = (now + timedelta(seconds=duration_seconds)).isoformat()
        self._leases[lease_id] = {
            "lease_id": lease_id,
            "task_id": task_id,
            "role": role,
            "holder_agent_id": agent_id,
            "granted_at": now.isoformat(),
            "expires_at": expires_at
        }
        return lease_id

    def is_lease_valid(self, lease_id: str) -> bool:
        if lease_id not in self._leases:
            return False
        info = self._leases[lease_id]
        now = datetime.now(timezone.utc)
        expires_at = datetime.fromisoformat(info["expires_at"])
        return now < expires_at

    def release_lease(self, lease_id: str) -> None:
        if lease_id in self._leases:
            del self._leases[lease_id]

class MessageRouter:
    """
    Inter-Agent Typed Message Router (ecm.message.v1).
    Validates message kinds, sender roles, and recipient bindings.
    """
    VALID_ROLES = ["SUPERVISOR", "PLANNER", "ORCHESTRATOR", "CODER", "REVIEWER", "RESEARCHER", "PEER_REVIEWER", "META_AGENT"]
    VALID_KINDS = ["PROPOSAL", "CHALLENGE", "REVIEW_DECISION", "SPEC_CORRECTION", "HEARTBEAT"]

    def __init__(self):
        self._messages: List[Dict[str, Any]] = []

    def dispatch_message(
        self,
        task_id: str,
        sender_role: str,
        message_kind: str,
        content: str,
        references: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        if sender_role not in self.VALID_ROLES:
            raise ECMTaskError(f"Invalid sender role: {sender_role}")
        if message_kind not in self.VALID_KINDS:
            raise ECMTaskError(f"Invalid message kind: {message_kind}")

        msg = {
            "message_id": str(uuid.uuid4()),
            "task_id": task_id,
            "sender_role": sender_role,
            "message_kind": message_kind,
            "content": content,
            "references": references or [],
            "sent_at": datetime.now(timezone.utc).isoformat()
        }
        self._messages.append(msg)
        return msg

    def get_task_messages(self, task_id: str) -> List[Dict[str, Any]]:
        return [m for m in self._messages if m["task_id"] == task_id]
