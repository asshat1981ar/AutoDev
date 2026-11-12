from typing import List, Dict, Any, Optional
from core.amx_reducers import AMXRecordProjection
from adapters.base_adapter import BaseAdapter
from ecm.scope_binding import ScopeBindingAuthority, ScopeBinding

class ContextViewService:
    """
    ECM ContextView Service.
    Enforces AMCX-R-0015: Pre-filters memory and task context by tenant, authorization,
    admission state, and validity BEFORE any semantic scoring or prompt injection.
    """
    def __init__(self, tenant_id: str, project_id: str, scope_authority=None, scope_bindings=()):
        self.tenant_id = tenant_id
        self.project_id = project_id
        # Injected by the trusted host. No untrusted caller may issue bindings.
        self._scope_authority = scope_authority
        self._scope_bindings = tuple(scope_bindings)

    def _bound_for(self, record: AMXRecordProjection, purpose: str) -> bool:
        if not isinstance(self._scope_authority, ScopeBindingAuthority):
            return False
        for binding in self._scope_bindings:
            if (self._scope_authority.verify(binding)
                    and binding.record_id == record.record_id
                    and binding.dag_digest == record.source_digest
                    and binding.tenant_id == self.tenant_id
                    and binding.project_id == self.project_id
                    and purpose in binding.allowed_purposes):
                return True
        return False

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
            # Receiver-owned scope must match before any content inspection.
            if not self._bound_for(r, purpose):
                rejected_count += 1
                continue

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
