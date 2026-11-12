"""AMCX context admission regression tests using canonical source DAGs."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.context_view import ContextViewService
from ecm.scope_binding import ScopeBindingAuthority


def record_dag(ch, title, content, admission="ADMITTED"):
    dag = AMXEventDAG(ch * 64)
    dag.append_event(AMXEvent(
        "1" * 64, "RECORD_CREATED", dag.record_id, [], "producer",
        {"title": title, "content": content, "admission": admission},
        timestamp="2026-11-11T00:00:00+00:00",
    ))
    return dag


class TestContextViewService(unittest.TestCase):
    def setUp(self):
        self.authority = ScopeBindingAuthority(
            b"amcx-test-receiver-secret-minimum-32-bytes"
        )

    def service(self, *dags):
        bindings = [
            self.authority.issue(
                record_id=dag.record_id, dag_digest=dag.compute_dag_digest(),
                tenant_id="tenant-alpha", project_id="autodev-core",
                allowed_purposes=("DEV",),
            ) for dag in dags
        ]
        return ContextViewService(
            "tenant-alpha", "autodev-core", scope_authority=self.authority,
            scope_bindings=bindings,
        )

    def request(self, purpose="DEV"):
        return self.authority.issue_request(
            principal_id="trusted-user", tenant_id="tenant-alpha",
            project_id="autodev-core", task_id="task-1", purpose=purpose,
        )

    def test_unauthorized_access_fails_closed(self):
        dag = record_dag("a", "Secret doc", "sensitive")
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=None,
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_quarantined_record_filtered_out(self):
        good = record_dag("a", "Valid record", "Safe content")
        bad = record_dag("b", "Quarantined record", "Suspicious", "QUARANTINED")
        view = self.service(good, bad).build_context_view(
            "task-1", [good, bad], request=self.request(),
        )
        self.assertEqual(len(view["admitted_records"]), 1)
        self.assertEqual(view["admitted_records"][0]["title"], "Valid record")
        self.assertEqual(view["rejected_count"], 1)

    def test_secret_containing_record_quarantined(self):
        dag = record_dag(
            "c", "Leaky Record", "Secret: sk-abcdef1234567890abcdef1234567890",
        )
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)


if __name__ == "__main__":
    unittest.main()
