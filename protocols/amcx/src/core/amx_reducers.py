from typing import Dict, Any, List, Optional
from core.amx_dag import AMXEventDAG, AMXEvent
from core.state_dimensions import OrthogonalStateTracker

class ReducerError(Exception):
    pass

class AMXRecordProjection:
    """
    Materialized projection of an AMX Memory Record computed deterministically from its DAG.
    """
    def __init__(self, record_id: str):
        self.record_id = record_id
        self.state_tracker = OrthogonalStateTracker()
        self.title: str = ""
        self.content: str = ""
        self.metadata: Dict[str, Any] = {}
        self.author_principal: str = ""
        self.version: int = 0
        self.last_event_id: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return {
            "record_id": self.record_id,
            "version": self.version,
            "title": self.title,
            "content": self.content,
            "metadata": self.metadata,
            "content_lifecycle": self.state_tracker.get_state("content_lifecycle"),
            "admission": self.state_tracker.get_state("admission"),
            "validity": self.state_tracker.get_state("validity"),
            "runtime_sharing": self.state_tracker.get_state("runtime_sharing"),
            "author_principal": self.author_principal,
            "last_event_id": self.last_event_id
        }

class AMXRecordReducer:
    """
    Deterministic state reducer.
    Applies AMX events in topological causal order to produce a verified projection.
    """
    @classmethod
    def reduce(cls, dag: AMXEventDAG) -> AMXRecordProjection:
        events = dag.list_events_topological()
        if not events:
            raise ReducerError(f"Cannot reduce empty DAG for record {dag.record_id}")

        proj = AMXRecordProjection(dag.record_id)

        for event in events:
            # Retraction Barrier Invariant: Once retracted, no mutations allowed except audit logging
            if proj.state_tracker.get_state("content_lifecycle") == "RETRACTED":
                if event.event_type not in ["RECORD_RETRACTED"]:
                    raise ReducerError("Retraction barrier violation: Cannot modify a retracted memory record.")

            delta = event.payload_delta

            if event.event_type == "RECORD_CREATED":
                proj.title = delta.get("title", "")
                proj.content = delta.get("content", "")
                proj.metadata = delta.get("metadata", {})
                proj.author_principal = event.actor
                proj.state_tracker.transition("content_lifecycle", delta.get("content_lifecycle", "CURRENT"))
                proj.state_tracker.transition("admission", delta.get("admission", "ADMITTED"))
                proj.state_tracker.transition("validity", delta.get("validity", "VALID"))

            elif event.event_type == "RECORD_MUTATED":
                if "title" in delta:
                    proj.title = delta["title"]
                if "content" in delta:
                    proj.content = delta["content"]
                if "metadata" in delta:
                    proj.metadata.update(delta["metadata"])

            elif event.event_type == "RECORD_SUPERSEDED":
                proj.state_tracker.transition("content_lifecycle", "SUPERSEDED")

            elif event.event_type == "RECORD_RETRACTED":
                proj.state_tracker.transition("content_lifecycle", "RETRACTED")

            elif event.event_type == "RECORD_QUARANTINED":
                proj.state_tracker.transition("admission", "QUARANTINED")

            proj.version += 1
            proj.last_event_id = event.event_id

        return proj
