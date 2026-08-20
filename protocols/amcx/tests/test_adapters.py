import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from adapters.chatgpt_adapter import ChatGPTCodexAdapter
from adapters.mcp_adapter import AutoDevMCPAdapter
from adapters.a2a_adapter import A2AMeshAdapter
from adapters.mistral_adapter import MistralVibeAdapter
from core.amx_dag import AMXEventDAG, AMXEvent
from core.amx_bundle import AMXMemoryBundle

class TestAdapters(unittest.TestCase):
    def test_chatgpt_adapter_untrusted_flag(self):
        adapter = ChatGPTCodexAdapter(agent_id="gpt-4o-01", role="CODER")
        msg = adapter.parse_completion_to_message("task-99", "I propose refactoring auth module")
        self.assertTrue(msg["untrusted_evidence"])
        self.assertEqual(msg["sender_role"], "CODER")
        self.assertEqual(len(msg["claimed_capabilities"]), 0)

    def test_mcp_adapter_context_view(self):
        adapter = AutoDevMCPAdapter(tenant_id="tenant-1", project_id="proj-1")
        dag = AMXEventDAG("r" * 64)
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", "r" * 64, [], "architect", {
            "title": "MCP Tool Test",
            "content": "Context for Cline/AutoDev"
        })
        dag.append_event(e0)

        view = adapter.tool_get_context_view("task-1", [dag], is_authorized=True, purpose="DEV")
        self.assertEqual(len(view["admitted_records"]), 1)
        self.assertEqual(view["admitted_records"][0]["title"], "MCP Tool Test")

    def test_a2a_mesh_adapter(self):
        a2a = A2AMeshAdapter(local_agent_id="agent-sender")
        dag = AMXEventDAG("r" * 64)
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", "r" * 64, [], "sender", {"title": "A2A Mesh", "content": "Data"})
        dag.append_event(e0)
        bundle = AMXMemoryBundle.export_dag("bundle-123", dag)

        tx = a2a.transmit_bundle(bundle, target_agent_id="agent-receiver", task_id="task-44")
        self.assertEqual(tx["status"], "TRANSMITTED_VERIFIED")
        self.assertFalse(tx["binding"]["is_canonical"])

    def test_mistral_vibe_adapter(self):
        mistral = MistralVibeAdapter(project_name="AutoDev-System")
        context = {
            "admitted_records": [
                {"title": "Core Invariant", "content": "confine execution"}
            ]
        }
        res = mistral.format_agent_prompt(context, "Implement step 4")
        self.assertIn("AutoDev-System", res["formatted_prompt"])
        self.assertIn("Core Invariant", res["formatted_prompt"])

if __name__ == '__main__':
    unittest.main()
