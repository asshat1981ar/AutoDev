from typing import Dict, Any, List, Optional
from adapters.base_adapter import BaseAdapter
from core.amx_bundle import AMXMemoryBundle
from core.memory_binding import ECMMemoryBinding

class A2AMeshAdapter(BaseAdapter):
    """
    Agent-to-Agent (A2A) Mesh Protocol Adapter.
    Enables peer agents to exchange verified memory bundles and link noncanonical bindings.
    """
    def __init__(self, local_agent_id: str):
        self.local_agent_id = local_agent_id

    def transmit_bundle(self, bundle: AMXMemoryBundle, target_agent_id: str, task_id: str) -> Dict[str, Any]:
        bundle_dict = bundle.to_dict()
        self.sanitize_payload(bundle_dict)

        # Create noncanonical binding for the target task
        binding = ECMMemoryBinding(
            binding_id=f"bind-{bundle.bundle_id[:8]}",
            task_id=task_id,
            amx_record_id=bundle.records[0]["record_id"] if bundle.records else "none",
            amx_record_digest=bundle.bundle_digest,
            purpose="OUTPUT_EVIDENCE"
        )

        return {
            "sender": self.local_agent_id,
            "receiver": target_agent_id,
            "bundle": bundle_dict,
            "binding": binding.to_dict(),
            "status": "TRANSMITTED_VERIFIED"
        }
