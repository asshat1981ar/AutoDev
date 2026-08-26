from typing import Dict, Any, List, Optional
from adapters.base_adapter import BaseAdapter
from ecm.task_engine import ECMTask, MessageRouter

class ChatGPTCodexAdapter(BaseAdapter):
    """
    ChatGPT / Codex Adapter.
    Translates chat-based completions into structured ECM messages and task updates.
    Enforces that model outputs are treated strictly as untrusted evidence (AMCX-R-0125).
    """
    def __init__(self, agent_id: str, role: str = "CODER"):
        self.agent_id = agent_id
        self.role = role

    def parse_completion_to_message(self, task_id: str, completion_text: str, message_kind: str = "PROPOSAL") -> Dict[str, Any]:
        sanitized_content = self.sanitize_payload({"content": completion_text})["content"]
        # Model output CANNOT self-grant privileges or set authority flags
        return {
            "task_id": task_id,
            "sender_role": self.role,
            "message_kind": message_kind,
            "content": sanitized_content,
            "untrusted_evidence": True,
            "claimed_capabilities": [] # Ignored by execution kernel
        }
