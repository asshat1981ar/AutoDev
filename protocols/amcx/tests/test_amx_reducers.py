import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.amx_dag import AMXEventDAG, AMXEvent
from core.amx_reducers import AMXRecordReducer, ReducerError

class TestAMXReducers(unittest.TestCase):
    def setUp(self):
        self.record_id = "a" * 64
        self.dag = AMXEventDAG(self.record_id)

    def test_deterministic_projection(self):
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", self.record_id, [], "agent-1", {
            "title": "Architecture Baseline",
            "content": "Core invariant definitions",
            "metadata": {"tags": ["amcx", "v1"]}
        })
        self.dag.append_event(e0)

        e1 = AMXEvent("e1" + "0"*62, "RECORD_MUTATED", self.record_id, [e0.event_id], "agent-2", {
            "title": "Architecture Baseline v2",
            "metadata": {"reviewed": True}
        })
        self.dag.append_event(e1)

        proj = AMXRecordReducer.reduce(self.dag)
        self.assertEqual(proj.version, 2)
        self.assertEqual(proj.title, "Architecture Baseline v2")
        self.assertEqual(proj.content, "Core invariant definitions")
        self.assertTrue(proj.metadata.get("reviewed"))
        self.assertEqual(proj.state_tracker.get_state("content_lifecycle"), "CURRENT")

    def test_quarantine_transition(self):
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", self.record_id, [], "agent-1", {"title": "Suspicious input"})
        self.dag.append_event(e0)

        e1 = AMXEvent("e1" + "0"*62, "RECORD_QUARANTINED", self.record_id, [e0.event_id], "security-sensor", {})
        self.dag.append_event(e1)

        proj = AMXRecordReducer.reduce(self.dag)
        self.assertEqual(proj.state_tracker.get_state("admission"), "QUARANTINED")
        self.assertFalse(proj.state_tracker.compute_effective_readability(is_authorized=True, within_purpose=True))

    def test_retraction_barrier_blocks_further_mutation(self):
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", self.record_id, [], "agent-1", {"title": "Obsolete spec"})
        self.dag.append_event(e0)

        e1 = AMXEvent("e1" + "0"*62, "RECORD_RETRACTED", self.record_id, [e0.event_id], "governance-lead", {})
        self.dag.append_event(e1)

        e2 = AMXEvent("e2" + "0"*62, "RECORD_MUTATED", self.record_id, [e1.event_id], "rogue-agent", {"title": "Resurrected"})
        self.dag.append_event(e2)

        with self.assertRaises(ReducerError):
            AMXRecordReducer.reduce(self.dag)

if __name__ == '__main__':
    unittest.main()
