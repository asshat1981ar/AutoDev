"""R-001 record-limit behavior ported to the R-003 DAG/request contract."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.context_view import ContextViewService
from ecm.scope_binding import ScopeBindingAuthority


class ContextLimitCompatibilityTests(unittest.TestCase):
    def setUp(self):
        self.dag = AMXEventDAG("a" * 64)
        self.dag.append_event(AMXEvent(
            "1" * 64, "RECORD_CREATED", self.dag.record_id, [],
            "test-author", {"title": "allowed", "content": "safe",
                            "admission": "ADMITTED", "validity": "VALID"},
            timestamp="2026-11-11T00:00:00+00:00",
        ))
        authority = ScopeBindingAuthority(b"r004-context-limit-test-secret-long-enough")
        binding = authority.issue(
            record_id=self.dag.record_id, dag_digest=self.dag.compute_dag_digest(),
            tenant_id="tenant-a", project_id="project-a",
            allowed_purposes=("DEV",),
        )
        self.service = ContextViewService(
            "tenant-a", "project-a", scope_authority=authority,
            scope_bindings=(binding,),
        )
        self.request = authority.issue_request(
            principal_id="trusted", tenant_id="tenant-a",
            project_id="project-a", task_id="task-1", purpose="DEV",
        )

    def test_zero_limit_admits_nothing(self):
        result = self.service.build_context_view(
            "task-1", [self.dag], request=self.request, max_records=0,
        )
        self.assertEqual(result["admitted_records"], [])
        self.assertEqual(result["rejected_count"], 1)

    def test_negative_limit_rejected(self):
        with self.assertRaises(ValueError):
            self.service.build_context_view(
                "task-1", [self.dag], request=self.request, max_records=-1,
            )

    def test_positive_limit_preserves_authorized_read(self):
        result = self.service.build_context_view(
            "task-1", [self.dag], request=self.request, max_records=1,
        )
        self.assertEqual(len(result["admitted_records"]), 1)



    def test_non_integer_limits_rejected(self):
        for limit in (True, False, 1.0, "1", None):
            with self.subTest(limit=limit), self.assertRaises(ValueError):
                self.service.build_context_view(
                    "task-1", [self.dag], request=self.request, max_records=limit,
                )

    def test_zero_limit_still_requires_trusted_request(self):
        result = self.service.build_context_view(
            "task-1", [self.dag], request=None, max_records=0,
        )
        self.assertEqual(result["admitted_records"], [])
        self.assertEqual(result["rejected_count"], 1)
        self.assertIn("failed closed", result["reason"])

if __name__ == "__main__":
    unittest.main()
