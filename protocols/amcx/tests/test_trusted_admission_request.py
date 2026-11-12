"""R-003 RED: host-authorized request and fresh-DAG-only context admission."""
import sys
import unittest
from dataclasses import replace
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from adapters.mcp_adapter import AutoDevMCPAdapter
from core.amx_dag import AMXEvent, AMXEventDAG
from core.amx_reducers import AMXRecordReducer
from ecm.context_view import ContextViewService
from ecm.scope_binding import ScopeBindingAuthority


class TrustedAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.authority = ScopeBindingAuthority(b"r003-host-only-test-authority-key-long-enough")
        self.dag = AMXEventDAG("a" * 64)
        self.dag.append_event(AMXEvent(
            "1" * 64, "RECORD_CREATED", self.dag.record_id, [], "producer",
            {"title": "trusted", "content": "original", "metadata": {}},
            timestamp="2026-11-11T00:00:00+00:00",
        ))
        self.binding = self.authority.issue(
            record_id=self.dag.record_id, dag_digest=self.dag.compute_dag_digest(),
            tenant_id="tenant-a", project_id="project-a", allowed_purposes=("DEV",),
        )
        self.adapter = AutoDevMCPAdapter(
            tenant_id="tenant-a", project_id="project-a",
            scope_authority=self.authority, scope_bindings=(self.binding,),
        )

    def request(self, **overrides):
        opts = dict(principal_id="trusted-user", tenant_id="tenant-a",
                    project_id="project-a", task_id="task-1", purpose="DEV")
        opts.update(overrides)
        return self.authority.issue_request(**opts)

    def test_valid_signed_request_admits_matching_dag(self):
        view = self.adapter.tool_get_context_view("task-1", [self.dag], request=self.request())
        self.assertEqual([x["content"] for x in view["admitted_records"]], ["original"])

    def test_legacy_boolean_cannot_authorize(self):
        with self.assertRaises((TypeError, ValueError)):
            self.adapter.tool_get_context_view(
                "task-1", [self.dag], is_authorized=True, purpose="DEV",
            )

    def test_request_tampering_fails_closed(self):
        tampered = replace(self.request(), principal_id="attacker")
        view = self.adapter.tool_get_context_view("task-1", [self.dag], request=tampered)
        self.assertEqual(view["admitted_records"], [])

    def test_request_task_mismatch_fails_closed(self):
        view = self.adapter.tool_get_context_view(
            "task-1", [self.dag], request=self.request(task_id="other-task"),
        )
        self.assertEqual(view["admitted_records"], [])

    def test_request_other_tenant_fails_closed(self):
        view = self.adapter.tool_get_context_view(
            "task-1", [self.dag], request=self.request(tenant_id="other-tenant"),
        )
        self.assertEqual(view["admitted_records"], [])

    def test_untrusted_projection_mutation_is_not_admitted(self):
        projection = AMXRecordReducer.reduce(self.dag)
        projection.content = "injected after digest calculation"
        view = self.adapter.context_service.build_context_view(
            task_id="task-1", records=[projection], request=self.request(),
        )
        self.assertEqual(view["admitted_records"], [])

    def test_modified_dag_is_rejected_against_original_binding(self):
        self.dag.append_event(AMXEvent(
            "2" * 64, "RECORD_MUTATED", self.dag.record_id, ["1" * 64],
            "producer", {"content": "changed"}, timestamp="2026-11-11T00:00:01+00:00",
        ))
        view = self.adapter.tool_get_context_view("task-1", [self.dag], request=self.request())
        self.assertEqual(view["admitted_records"], [])


    def test_expired_signed_request_fails_closed(self):
        request = self.authority.issue_request(
            principal_id="trusted-user", tenant_id="tenant-a",
            project_id="project-a", task_id="task-1", purpose="DEV",
            expires_at=1,
        )
        view = self.adapter.tool_get_context_view("task-1", [self.dag], request=request)
        self.assertEqual(view["admitted_records"], [])

    def test_existing_event_mutation_after_binding_fails_closed(self):
        event = self.dag.get_event("1" * 64)
        event.payload_delta["content"] = "tampered without new event"
        view = self.adapter.tool_get_context_view(
            "task-1", [self.dag], request=self.request(),
        )
        self.assertEqual(view["admitted_records"], [])

    def test_signed_request_with_other_purpose_rejects_binding(self):
        view = self.adapter.tool_get_context_view(
            "task-1", [self.dag], request=self.request(purpose="AUDIT"),
        )
        self.assertEqual(view["admitted_records"], [])


if __name__ == "__main__":
    unittest.main()
