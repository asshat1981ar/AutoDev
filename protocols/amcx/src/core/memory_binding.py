import hashlib
from typing import Dict, Any

class ECMMemoryBinding:
    """
    Noncanonical, rebuildable link between ECM collaboration tasks and AMX memory records.
    Guarantee: Does NOT own, mutate, or alter AMX causal heads (Invariant 2 & 3).
    """
    def __init__(self, binding_id: str, task_id: str, amx_record_id: str, amx_record_digest: str, purpose: str):
        self.binding_id = binding_id
        self.task_id = task_id
        self.amx_record_id = amx_record_id
        self.amx_record_digest = amx_record_digest
        self.purpose = purpose
        self.is_canonical = False

    def to_dict(self) -> Dict[str, Any]:
        return {
            "binding_id": self.binding_id,
            "task_id": self.task_id,
            "amx_record_id": self.amx_record_id,
            "amx_record_digest": self.amx_record_digest,
            "binding_purpose": self.purpose,
            "is_canonical": self.is_canonical
        }
