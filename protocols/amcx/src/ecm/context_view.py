"""Memory ContextView: host-authenticated request and source DAG admission only."""
import copy
import hmac
from typing import Any, Dict, List

from core.amx_dag import AMXEventDAG
from core.amx_reducers import AMXRecordReducer
from adapters.base_adapter import BaseAdapter
from ecm.scope_binding import ScopeBindingAuthority, TrustedAdmissionRequest


class ContextViewService:
    """Read-only context admission; never grants ForgeCore execution authority."""

    def __init__(self, tenant_id: str, project_id: str,
                 scope_authority=None, scope_bindings=()):
        self.tenant_id = tenant_id
        self.project_id = project_id
        self._scope_authority = scope_authority
        self._scope_bindings = tuple(scope_bindings)

    def _request_allowed(self, request, task_id):
        return (
            isinstance(self._scope_authority, ScopeBindingAuthority)
            and self._scope_authority.verify_request(request)
            and request.tenant_id == self.tenant_id
            and request.project_id == self.project_id
            and request.task_id == task_id
        )

    def _bound_for(self, record, purpose):
        for binding in self._scope_bindings:
            if (self._scope_authority.verify(binding)
                    and binding.record_id == record.record_id
                    and hmac.compare_digest(binding.dag_digest, record.source_digest)
                    and binding.tenant_id == self.tenant_id
                    and binding.project_id == self.project_id
                    and purpose in binding.allowed_purposes):
                return True
        return False

    def build_context_view(
        self, task_id: str, records: List[AMXEventDAG],
        request: TrustedAdmissionRequest = None, max_records: int = 10,
    ) -> Dict[str, Any]:
        if type(max_records) is not int or max_records < 0:
            raise ValueError("max_records must be a non-negative integer")

        admitted = []
        rejected_count = 0
        purpose = request.purpose if isinstance(request, TrustedAdmissionRequest) else ""
        if not self._request_allowed(request, task_id):
            return {
                "tenant_id": self.tenant_id, "project_id": self.project_id,
                "task_id": task_id, "purpose": purpose,
                "admitted_records": [], "rejected_count": len(records),
                "reason": "Untrusted or out-of-scope admission request (failed closed)",
            }

        if max_records == 0:
            return {
                "tenant_id": self.tenant_id, "project_id": self.project_id,
                "task_id": task_id, "purpose": purpose,
                "admitted_records": [], "rejected_count": len(records),
                "status": "CONSTRUCTED_CLEAN",
            }

        for source in records:
            # Projections are intentionally not accepted: mutable source_digest
            # cannot attest to independently modified projection fields.
            if not isinstance(source, AMXEventDAG):
                rejected_count += 1
                continue
            try:
                snapshot = copy.deepcopy(source)
                events = snapshot.list_events_topological()
                if not events or any(
                    event.record_id != snapshot.record_id
                    or event.compute_digest() != event.event_digest
                    for event in events
                ):
                    rejected_count += 1
                    continue
                record = AMXRecordReducer.reduce(snapshot)
                if not self._bound_for(record, purpose):
                    rejected_count += 1
                    continue
                if not record.state_tracker.compute_effective_readability(
                        is_authorized=True, within_purpose=True):
                    rejected_count += 1
                    continue
                BaseAdapter.sanitize_payload({
                    "title": record.title, "content": record.content,
                    "metadata": record.metadata,
                    "author": record.author_principal,
                })
            except Exception:
                rejected_count += 1
                continue

            admitted.append({
                "record_id": record.record_id, "title": record.title,
                "content": record.content, "version": record.version,
                "author": record.author_principal,
            })
            if len(admitted) >= max_records:
                break

        return {
            "tenant_id": self.tenant_id, "project_id": self.project_id,
            "task_id": task_id, "purpose": purpose,
            "admitted_records": admitted, "rejected_count": rejected_count,
            "status": "CONSTRUCTED_CLEAN",
        }
