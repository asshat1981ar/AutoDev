import hashlib
import json
from datetime import datetime, timezone
from typing import List, Dict, Any, Optional, Set

class AMXDAGError(Exception):
    pass

class AMXEvent:
    """
    Immutable Event in the AMX Causal Event DAG (amx.event.v1).
    """
    def __init__(
        self,
        event_id: str,
        event_type: str,
        record_id: str,
        parent_event_ids: List[str],
        actor: str,
        payload_delta: Dict[str, Any],
        timestamp: Optional[str] = None,
        event_digest: Optional[str] = None
    ):
        self.event_id = event_id
        self.event_type = event_type
        self.record_id = record_id
        self.parent_event_ids = sorted(parent_event_ids)
        self.actor = actor
        self.payload_delta = payload_delta
        self.timestamp = timestamp or datetime.now(timezone.utc).isoformat()
        self.event_digest = event_digest or self.compute_digest()

    def compute_digest(self) -> str:
        serialized = json.dumps({
            "event_id": self.event_id,
            "event_type": self.event_type,
            "record_id": self.record_id,
            "parent_event_ids": self.parent_event_ids,
            "actor": self.actor,
            "payload_delta": self.payload_delta,
            "timestamp": self.timestamp
        }, sort_keys=True)
        return hashlib.sha256(serialized.encode("utf-8")).hexdigest()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "event_id": self.event_id,
            "event_type": self.event_type,
            "record_id": self.record_id,
            "parent_event_ids": self.parent_event_ids,
            "event_digest": self.event_digest,
            "timestamp": self.timestamp,
            "actor": self.actor,
            "payload_delta": self.payload_delta
        }

class AMXEventDAG:
    """
    Append-only causal DAG engine for canonical AMX memory events.
    Maintains causal parent hash chaining and computes deterministic heads.
    """
    def __init__(self, record_id: str):
        self.record_id = record_id
        self._events: Dict[str, AMXEvent] = {}
        self._heads: Set[str] = set()
        self._roots: Set[str] = set()

    def append_event(self, event: AMXEvent) -> None:
        if event.record_id != self.record_id:
            raise AMXDAGError(f"Event record_id {event.record_id} does not match DAG record_id {self.record_id}")
        
        if event.event_id in self._events:
            raise AMXDAGError(f"Duplicate event_id: {event.event_id}")

        # Verify parent existence
        for pid in event.parent_event_ids:
            if pid not in self._events:
                raise AMXDAGError(f"Parent event {pid} not found in DAG. Causality broken.")

        # Verify digest
        expected_digest = event.compute_digest()
        if event.event_digest != expected_digest:
            raise AMXDAGError(f"Event digest mismatch: expected {expected_digest}, got {event.event_digest}")

        self._events[event.event_id] = event

        # Update roots and heads
        if not event.parent_event_ids:
            self._roots.add(event.event_id)
        
        # Remove parents from heads since this new event extends them
        for pid in event.parent_event_ids:
            if pid in self._heads:
                self._heads.remove(pid)
        self._heads.add(event.event_id)

    def get_event(self, event_id: str) -> Optional[AMXEvent]:
        return self._events.get(event_id)

    def list_events_topological(self) -> List[AMXEvent]:
        visited = set()
        result = []

        def visit(node_id: str):
            if node_id in visited:
                return
            event = self._events[node_id]
            for pid in event.parent_event_ids:
                visit(pid)
            visited.add(node_id)
            result.append(event)

        for head_id in sorted(self._heads):
            visit(head_id)

        return result

    def get_causal_heads(self) -> List[str]:
        return sorted(list(self._heads))

    def get_causal_roots(self) -> List[str]:
        return sorted(list(self._roots))

    def compute_dag_digest(self) -> str:
        events = self.list_events_topological()
        serialized = "".join([e.event_digest for e in events])
        return hashlib.sha256(serialized.encode("utf-8")).hexdigest()
