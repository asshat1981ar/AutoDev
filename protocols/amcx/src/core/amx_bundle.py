import json
import hashlib
from datetime import datetime, timezone
from typing import Dict, Any, List, Optional
from core.amx_dag import AMXEventDAG, AMXEvent
from core.amx_reducers import AMXRecordReducer

class BundleError(Exception):
    pass

class AMXMemoryBundle:
    """
    Portable Memory Bundle (amx.bundle.v1).
    Encapsulates memory records, causal DAG events, roots, heads, and cryptographic bundle digest.
    """
    def __init__(
        self,
        bundle_id: str,
        records: List[Dict[str, Any]],
        events: List[Dict[str, Any]],
        causal_roots: List[str],
        causal_heads: List[str],
        exported_at: Optional[str] = None,
        bundle_digest: Optional[str] = None
    ):
        self.bundle_id = bundle_id
        self.schema_version = "amx.bundle.v1"
        self.records = records
        self.events = events
        self.causal_roots = causal_roots
        self.causal_heads = causal_heads
        self.exported_at = exported_at or datetime.now(timezone.utc).isoformat()
        self.bundle_digest = bundle_digest or self.compute_digest()

    def compute_digest(self) -> str:
        serialized = json.dumps({
            "bundle_id": self.bundle_id,
            "schema_version": self.schema_version,
            "records": self.records,
            "events": self.events,
            "causal_roots": self.causal_roots,
            "causal_heads": self.causal_heads,
            "exported_at": self.exported_at
        }, sort_keys=True)
        return hashlib.sha256(serialized.encode("utf-8")).hexdigest()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "bundle_id": self.bundle_id,
            "schema_version": self.schema_version,
            "records": self.records,
            "events": self.events,
            "causal_roots": self.causal_roots,
            "causal_heads": self.causal_heads,
            "bundle_digest": self.bundle_digest,
            "exported_at": self.exported_at
        }

    @classmethod
    def export_dag(cls, bundle_id: str, dag: AMXEventDAG) -> "AMXMemoryBundle":
        proj = AMXRecordReducer.reduce(dag)
        events_dicts = [e.to_dict() for e in dag.list_events_topological()]
        return cls(
            bundle_id=bundle_id,
            records=[proj.to_dict()],
            events=events_dicts,
            causal_roots=dag.get_causal_roots(),
            causal_heads=dag.get_causal_heads()
        )

    @classmethod
    def import_and_verify(cls, bundle_dict: Dict[str, Any]) -> "AMXMemoryBundle":
        bundle = cls(
            bundle_id=bundle_dict["bundle_id"],
            records=bundle_dict["records"],
            events=bundle_dict["events"],
            causal_roots=bundle_dict["causal_roots"],
            causal_heads=bundle_dict["causal_heads"],
            exported_at=bundle_dict["exported_at"],
            bundle_digest=bundle_dict["bundle_digest"]
        )
        expected = bundle.compute_digest()
        if bundle.bundle_digest != expected:
            raise BundleError(f"Bundle digest verification failed! Expected {expected}, got {bundle.bundle_digest}")
        return bundle
