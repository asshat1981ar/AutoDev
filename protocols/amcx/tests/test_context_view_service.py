import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from ecm.context_view import ContextViewService
from core.amx_reducers import AMXRecordProjection

class TestContextViewService(unittest.TestCase):
    def setUp(self):
        self.service = ContextViewService(tenant_id="tenant-alpha", project_id="autodev-core")

    def test_unauthorized_access_fails_closed(self):
        rec = AMXRecordProjection("r1" + "0"*62)
        rec.title = "Secret doc"
        rec.state_tracker.transition("admission", "ADMITTED")

        view = self.service.build_context_view(
            task_id="task-1",
            records=[rec],
            is_authorized=False,
            purpose="AUDIT"
        )
        self.assertEqual(len(view["admitted_records"]), 0)
        self.assertEqual(view["rejected_count"], 1)

    def test_quarantined_record_filtered_out(self):
        rec1 = AMXRecordProjection("r1" + "0"*62)
        rec1.title = "Valid record"
        rec1.content = "Safe content"
        rec1.state_tracker.transition("admission", "ADMITTED")

        rec2 = AMXRecordProjection("r2" + "0"*62)
        rec2.title = "Quarantined record"
        rec2.content = "Suspicious content"
        rec2.state_tracker.transition("admission", "QUARANTINED")

        view = self.service.build_context_view(
            task_id="task-1",
            records=[rec1, rec2],
            is_authorized=True,
            purpose="DEV"
        )
        self.assertEqual(len(view["admitted_records"]), 1)
        self.assertEqual(view["admitted_records"][0]["title"], "Valid record")
        self.assertEqual(view["rejected_count"], 1)

    def test_secret_containing_record_quarantined(self):
        rec = AMXRecordProjection("r3" + "0"*62)
        rec.title = "Leaky Record"
        rec.content = "Secret: sk-abcdef1234567890abcdef1234567890"
        rec.state_tracker.transition("admission", "ADMITTED")

        view = self.service.build_context_view(
            task_id="task-1",
            records=[rec],
            is_authorized=True,
            purpose="DEV"
        )
        self.assertEqual(len(view["admitted_records"]), 0)
        self.assertEqual(view["rejected_count"], 1)

if __name__ == '__main__':
    unittest.main()
