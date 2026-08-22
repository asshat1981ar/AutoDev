import copy
import hashlib
import hmac
import json
from datetime import datetime, timezone
from typing import Dict, Any, List, Optional

from core.amx_dag import AMXEventDAG, AMXEvent, AMXDAGError
from core.amx_reducers import AMXRecordReducer, ReducerError


class BundleError(Exception):
    pass


class AMXMemoryBundle:
    """
    Portable Memory Bundle (amx.bundle.v1).
    Encapsulates memory records, causal DAG events, roots, heads, and cryptographic bundle digest.
    """

    _EVENT_TYPES = {
        "RECORD_CREATED",
        "RECORD_MUTATED",
        "RECORD_SUPERSEDED",
        "RECORD_RETRACTED",
        "RECORD_QUARANTINED",
    }

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

    @staticmethod
    def _lower_hex_64(value: Any) -> bool:
        return (
            isinstance(value, str)
            and len(value) == 64
            and all(ch in "0123456789abcdef" for ch in value)
        )

    @classmethod
    def _event_from_dict(cls, raw: Dict[str, Any]) -> AMXEvent:
        required = {
            "event_id",
            "event_type",
            "record_id",
            "parent_event_ids",
            "event_digest",
            "timestamp",
            "actor",
            "payload_delta",
        }
        if not isinstance(raw, dict) or set(raw) != required:
            raise BundleError("AMX bundle contains malformed event fields")
        if not cls._lower_hex_64(raw["event_id"]):
            raise BundleError("AMX bundle contains invalid event_id")
        if not cls._lower_hex_64(raw["record_id"]):
            raise BundleError("AMX bundle contains invalid record_id")
        if not cls._lower_hex_64(raw["event_digest"]):
            raise BundleError("AMX bundle contains invalid event_digest")
        if raw["event_type"] not in cls._EVENT_TYPES:
            raise BundleError("AMX bundle contains invalid event_type")
        if not isinstance(raw["parent_event_ids"], list) or not all(
            cls._lower_hex_64(parent_id) for parent_id in raw["parent_event_ids"]
        ):
            raise BundleError("AMX bundle contains invalid parent_event_ids")
        if not isinstance(raw["actor"], str) or not raw["actor"].strip():
            raise BundleError("AMX bundle event provenance requires a non-blank actor")
        if not isinstance(raw["payload_delta"], dict):
            raise BundleError("AMX bundle contains invalid payload_delta")
        if not isinstance(raw["timestamp"], str):
            raise BundleError("AMX bundle contains invalid timestamp")
        try:
            datetime.fromisoformat(raw["timestamp"].replace("Z", "+00:00"))
        except ValueError as exc:
            raise BundleError("AMX bundle contains invalid timestamp") from exc

        return AMXEvent(
            event_id=raw["event_id"],
            event_type=raw["event_type"],
            record_id=raw["record_id"],
            parent_event_ids=raw["parent_event_ids"],
            actor=raw["actor"],
            payload_delta=copy.deepcopy(raw["payload_delta"]),
            timestamp=raw["timestamp"],
            event_digest=raw["event_digest"],
        )

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
    def import_and_verify(
        cls,
        bundle_dict: Dict[str, Any],
        trusted_bundle_digest: str,
    ) -> "AMXMemoryBundle":
        if not isinstance(bundle_dict, dict):
            raise BundleError("AMX bundle must be an object")
        if bundle_dict.get("schema_version") != "amx.bundle.v1":
            raise BundleError("Unsupported AMX bundle schema_version")
        if not cls._lower_hex_64(trusted_bundle_digest):
            raise BundleError("Trusted AMX bundle digest must be lowercase SHA-256")

        required = {
            "bundle_id",
            "schema_version",
            "records",
            "events",
            "causal_roots",
            "causal_heads",
            "bundle_digest",
            "exported_at",
        }
        if set(bundle_dict) != required:
            raise BundleError("AMX bundle fields do not match amx.bundle.v1")
        if not isinstance(bundle_dict["bundle_id"], str) or not bundle_dict["bundle_id"].strip():
            raise BundleError("AMX bundle_id must be non-blank")
        if not cls._lower_hex_64(bundle_dict["bundle_digest"]):
            raise BundleError("AMX bundle_digest must be lowercase SHA-256")
        if not isinstance(bundle_dict["records"], list) or len(bundle_dict["records"]) != 1:
            raise BundleError("AMX bundle must contain exactly one record projection")
        if not isinstance(bundle_dict["events"], list) or not bundle_dict["events"]:
            raise BundleError("AMX bundle must contain at least one event")
        if not isinstance(bundle_dict["causal_roots"], list) or not isinstance(
            bundle_dict["causal_heads"], list
        ):
            raise BundleError("AMX bundle causal roots and heads must be arrays")
        if not isinstance(bundle_dict["exported_at"], str):
            raise BundleError("AMX bundle exported_at must be a timestamp")
        try:
            datetime.fromisoformat(bundle_dict["exported_at"].replace("Z", "+00:00"))
        except ValueError as exc:
            raise BundleError("AMX bundle exported_at must be a valid timestamp") from exc

        bundle = cls(
            bundle_id=bundle_dict["bundle_id"],
            records=copy.deepcopy(bundle_dict["records"]),
            events=copy.deepcopy(bundle_dict["events"]),
            causal_roots=copy.deepcopy(bundle_dict["causal_roots"]),
            causal_heads=copy.deepcopy(bundle_dict["causal_heads"]),
            exported_at=bundle_dict["exported_at"],
            bundle_digest=bundle_dict["bundle_digest"],
        )
        expected = bundle.compute_digest()
        if not hmac.compare_digest(bundle.bundle_digest, expected):
            raise BundleError(
                f"Bundle digest verification failed! Expected {expected}, got {bundle.bundle_digest}"
            )
        if not hmac.compare_digest(bundle.bundle_digest, trusted_bundle_digest):
            raise BundleError("Bundle digest does not match trusted external digest")

        try:
            events = [cls._event_from_dict(raw) for raw in bundle.events]
            record_id = events[0].record_id
            dag = AMXEventDAG(record_id)
            for event in events:
                dag.append_event(event)
            projection = AMXRecordReducer.reduce(dag).to_dict()
        except (AMXDAGError, ReducerError) as exc:
            raise BundleError(f"AMX bundle event semantics are invalid: {exc}") from exc

        if bundle.causal_roots != dag.get_causal_roots():
            raise BundleError("AMX bundle causal_roots do not match reconstructed DAG")
        if bundle.causal_heads != dag.get_causal_heads():
            raise BundleError("AMX bundle causal_heads do not match reconstructed DAG")
        if bundle.records != [projection]:
            raise BundleError("AMX bundle records do not match reconstructed event projection")

        return bundle
