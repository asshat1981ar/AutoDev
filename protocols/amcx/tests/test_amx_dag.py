import unittest
import sys
import hashlib

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.amx_dag import AMXEventDAG, AMXEvent, AMXDAGError

class TestAMXEventDAG(unittest.TestCase):
    def setUp(self):
        self.record_id = "r" * 64
        self.dag = AMXEventDAG(self.record_id)

    def test_root_event_creation(self):
        e0 = AMXEvent(
            event_id="e0" + "0" * 62,
            event_type="RECORD_CREATED",
            record_id=self.record_id,
            parent_event_ids=[],
            actor="agent-architect",
            payload_delta={"title": "Root Record", "content": "Initial context"}
        )
        self.dag.append_event(e0)
        self.assertEqual(self.dag.get_causal_roots(), [e0.event_id])
        self.assertEqual(self.dag.get_causal_heads(), [e0.event_id])
        self.assertEqual(len(self.dag.list_events_topological()), 1)

    def test_causal_chaining(self):
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", self.record_id, [], "agent-1", {"title": "V1", "content": "Initial"})
        self.dag.append_event(e0)

        e1 = AMXEvent("e1" + "0"*62, "RECORD_MUTATED", self.record_id, [e0.event_id], "agent-2", {"title": "V2"})
        self.dag.append_event(e1)

        self.assertEqual(self.dag.get_causal_heads(), [e1.event_id])
        self.assertEqual(self.dag.get_causal_roots(), [e0.event_id])

        events = self.dag.list_events_topological()
        self.assertEqual(len(events), 2)
        self.assertEqual(events[0].event_id, e0.event_id)
        self.assertEqual(events[1].event_id, e1.event_id)

    def test_broken_causality_rejected(self):
        e_orphan = AMXEvent("e9" + "0"*62, "RECORD_MUTATED", self.record_id, ["nonexistent_parent"], "agent-1", {"content": "Bad"})
        with self.assertRaises(AMXDAGError):
            self.dag.append_event(e_orphan)

    def test_mismatched_record_id_rejected(self):
        e_wrong = AMXEvent("e2" + "0"*62, "RECORD_CREATED", "other_record" + "0"*52, [], "agent-1", {"content": "Wrong"})
        with self.assertRaises(AMXDAGError):
            self.dag.append_event(e_wrong)

if __name__ == '__main__':
    unittest.main()
