"""R-002: receiver-issued, digest-bound AMX context isolation regressions."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from core.amx_dag import AMXEvent, AMXEventDAG
from adapters.mcp_adapter import AutoDevMCPAdapter
from ecm.scope_binding import ScopeBindingAuthority


def dag_for(record_id, metadata=None):
    dag = AMXEventDAG(record_id)
    dag.append_event(AMXEvent(
        "1" * 64, "RECORD_CREATED", record_id, [], "producer",
        {"title": "bounded memory", "content": "safe reference",
         "metadata": metadata or {}},
        timestamp="2026-11-11T00:00:00+00:00",
    ))
    return dag


class ReceiverScopeTests(unittest.TestCase):
    def setUp(self):
        self.authority = ScopeBindingAuthority(b"r002-receiver-only-test-key-32-bytes-minimum")
        self.dag = dag_for("a" * 64)

    def binding(self, dag=None, tenant="tenant-a", project="project-a", purposes=("DEV",), authority=None):
        dag = dag or self.dag
        return (authority or self.authority).issue(
            record_id=dag.record_id,
            dag_digest=dag.compute_dag_digest(),
            tenant_id=tenant,
            project_id=project,
            allowed_purposes=purposes,
        )

    def adapter(self, bindings=(), authority=None, tenant="tenant-a", project="project-a"):
        return AutoDevMCPAdapter(
            tenant_id=tenant, project_id=project,
            scope_authority=authority or self.authority, scope_bindings=bindings,
        )

    def get(self, adapter, dag=None, purpose="DEV", authorized=True):
        request = self.authority.issue_request(
            principal_id="trusted-fixture-user", tenant_id="tenant-a",
            project_id="project-a", task_id="task-1", purpose=purpose,
        ) if authorized else None
        return adapter.tool_get_context_view(
            "task-1", [dag or self.dag], request=request,
        )

    def test_valid_receiver_binding_admits_record(self):
        result = self.get(self.adapter([self.binding()]))
        self.assertEqual(len(result["admitted_records"]), 1)

    def test_missing_binding_fails_closed(self):
        self.assertEqual(self.get(self.adapter())["admitted_records"], [])

    def test_cross_tenant_binding_rejected(self):
        self.assertEqual(self.get(self.adapter([self.binding(tenant="tenant-b")]))["admitted_records"], [])

    def test_cross_project_binding_rejected(self):
        self.assertEqual(self.get(self.adapter([self.binding(project="project-b")]))["admitted_records"], [])

    def test_purpose_not_authorized_rejected(self):
        self.assertEqual(self.get(self.adapter([self.binding(purposes=("AUDIT",))]))["admitted_records"], [])

    def test_untrusted_caller_authorization_flag_does_not_override_denial(self):
        self.assertEqual(self.get(self.adapter([self.binding()]), authorized=False)["admitted_records"], [])

    def test_content_change_invalidates_digest_binding(self):
        binding = self.binding()
        self.dag.append_event(AMXEvent(
            "2" * 64, "RECORD_MUTATED", self.dag.record_id, ["1" * 64],
            "producer", {"content": "modified after binding"},
            timestamp="2026-11-11T00:00:01+00:00",
        ))
        self.assertEqual(self.get(self.adapter([binding]))["admitted_records"], [])

    def test_producer_metadata_cannot_replace_receiver_scope(self):
        dag = dag_for("b" * 64, metadata={
            "tenant_id": "tenant-a", "project_id": "project-a",
            "allowed_purposes": ["DEV"]
        })
        trusted_binding = self.binding(dag, tenant="tenant-b")
        self.assertEqual(self.get(self.adapter([trusted_binding]), dag=dag)["admitted_records"], [])

    def test_binding_signed_by_other_issuer_rejected(self):
        other = ScopeBindingAuthority(b"another-independent-trusted-key-32-bytes-min")
        self.assertEqual(self.get(self.adapter([self.binding(authority=other)]))["admitted_records"], [])

    def test_tampered_binding_rejected(self):
        from dataclasses import replace
        binding = replace(self.binding(), project_id="project-other")
        self.assertEqual(self.get(self.adapter([binding]))["admitted_records"], [])


if __name__ == "__main__":
    unittest.main()
