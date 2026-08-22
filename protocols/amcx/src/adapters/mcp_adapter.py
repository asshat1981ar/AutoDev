from typing import Dict, Any, List, Optional
from adapters.base_adapter import BaseAdapter
from core.amx_dag import AMXEventDAG
from core.amx_reducers import AMXRecordReducer
from ecm.context_view import ContextViewService

class AutoDevMCPAdapter(BaseAdapter):
    """
    FastMCP / Model Context Protocol Adapter for AutoDev / Cline.
    Exposes tool interfaces for safe memory reading and collaboration dispatch.
    """
    def __init__(self, tenant_id: str, project_id: str):
        self.context_service = ContextViewService(tenant_id, project_id)

    def tool_get_context_view(self, task_id: str, dags: List[AMXEventDAG], is_authorized: bool, purpose: str) -> Dict[str, Any]:
        projections = []
        for dag in dags:
            try:
                proj = AMXRecordReducer.reduce(dag)
                projections.append(proj)
            except Exception:
                continue
        
        return self.context_service.build_context_view(
            task_id=task_id,
            records=projections,
            is_authorized=is_authorized,
            purpose=purpose
        )
