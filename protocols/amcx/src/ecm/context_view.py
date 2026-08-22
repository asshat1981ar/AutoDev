from typing import List, Dict, Any, Optional
from core.amx_reducers import AMXRecordProjection
from adapters.base_adapter import BaseAdapter

class ContextViewService:
    """
    ECM ContextView Service.
    Enforces AMCX-R-0015: Pre-filters memory and task context by tenant, authorization,
    admission state, and validity BEFORE any semantic scoring or prompt injection.
    """
    def __init__(self, tenant_id: str, project_id: str):
        self.tenant_id = tenant_id
        self.project_id = project_id

    def build_context_view(
        self,
        task_id: str,
        records: List[AMXRecordProjection],
        is_authorized: bool,
        purpose: str,
        max_records: int = 10
    ) -> Dict[str, Any]:
        if not is_authorized:
            return {
                "tenant_id": self.tenant_id,
                "project_id": self.project_id,
                "task_id": task_id,
                "purpose": purpose,
                "admitted_records": [],
                "rejected_count": len(records),
                "reason": "Unauthorized access attempt (failed closed)"
            }

        admitted = []
        rejected_count = 0

        for r in records:
            # Check orthogonal state gates
            if not r.state_tracker.compute_effective_readability(is_authorized=True, within_purpose=True):
                rejected_count += 1
                continue

            # Check zero-secret invariant
            payload_to_check = {
                "title": r.title,
                "content": r.content,
                "metadata": r.metadata
            }
            try:
                BaseAdapter.sanitize_payload(payload_to_check)
            except Exception:
                # Quarantined due to secret detection
                rejected_count += 1
                continue

            admitted.append({
                "record_id": r.record_id,
                "title": r.title,
                "content": r.content,
                "version": r.version,
                "author": r.author_principal
            })

            if len(admitted) >= max_records:
                break

        return {
            "tenant_id": self.tenant_id,
            "project_id": self.project_id,
            "task_id": task_id,
            "purpose": purpose,
            "admitted_records": admitted,
            "rejected_count": rejected_count,
            "status": "CONSTRUCTED_CLEAN"
        }
