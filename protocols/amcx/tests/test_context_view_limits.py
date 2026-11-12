"""R-001: fail-closed ContextView record-limit regression tests."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from core.amx_reducers import AMXRecordProjection
from ecm.context_view import ContextViewService


class ContextViewLimitTests(unittest.TestCase):
    def setUp(self):
        self.service = ContextViewService("tenant-alpha", "autodev-core")
        self.record = AMXRecordProjection("r" * 64)
        self.record.title = "Safe record"
        self.record.content = "Ordinary documentation"
        self.record.state_tracker.transition("admission", "ADMITTED")

    def test_zero_limit_never_admits_records(self):
        result = self.service.build_context_view(
            task_id="r001",
            records=[self.record],
            is_authorized=True,
            purpose="DEV",
            max_records=0,
        )
        self.assertEqual(result["admitted_records"], [])
        self.assertEqual(result["rejected_count"], 1)

    def test_negative_limit_fails_closed(self):
        with self.assertRaises(ValueError):
            self.service.build_context_view(
                task_id="r001",
                records=[self.record],
                is_authorized=True,
                purpose="DEV",
                max_records=-1,
            )


if __name__ == "__main__":
    unittest.main()
