"""AMCX MCP adapter: untrusted reads against host-bound admission policy."""
from typing import List

from adapters.base_adapter import BaseAdapter
from core.amx_dag import AMXEventDAG
from ecm.context_view import ContextViewService
from ecm.scope_binding import TrustedAdmissionRequest


class AutoDevMCPAdapter(BaseAdapter):
    """The trusted host supplies bindings and signed request context.

    This class does not authenticate callers and does not issue authorization.
    The host MUST authenticate and authorize before signing a request.
    """

    def __init__(self, tenant_id: str, project_id: str,
                 scope_authority=None, scope_bindings=()):
        self.context_service = ContextViewService(
            tenant_id, project_id, scope_authority=scope_authority,
            scope_bindings=scope_bindings,
        )

    def tool_get_context_view(self, task_id: str, dags: List[AMXEventDAG],
                              request: TrustedAdmissionRequest):
        return self.context_service.build_context_view(
            task_id=task_id, records=dags, request=request,
        )
