from typing import Dict, Any, List, Optional
from adapters.base_adapter import BaseAdapter

class MistralVibeAdapter(BaseAdapter):
    """
    Mistral Vibe and Mistral Agent Adapter.
    Formats project instructions and ensures fail-closed prompt boundaries.
    """
    def __init__(self, project_name: str = "AutoDev"):
        self.project_name = project_name

    def format_agent_prompt(self, context_view: Dict[str, Any], task_instruction: str) -> Dict[str, Any]:
        self.sanitize_payload({"instruction": task_instruction})
        
        system_rules = [
            f"Project: {self.project_name}",
            "Core Invariant: untrusted intent -> trusted authorization -> confined execution",
            "Zero raw secrets in memory or output.",
            "Model confidence is never authority."
        ]

        records_text = []
        for rec in context_view.get("admitted_records", []):
            records_text.append(f"[{rec['title']}]: {rec['content']}")

        formatted_prompt = (
            "### System Rules\n" + "\n".join(system_rules) + "\n\n"
            "### Admitted Context\n" + ("\n".join(records_text) if records_text else "None") + "\n\n"
            "### Task\n" + task_instruction
        )

        return {
            "formatted_prompt": formatted_prompt,
            "context_records_count": len(context_view.get("admitted_records", []))
        }
