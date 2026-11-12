"""R-004 adversarial tests for receiver-side authentication, replay, and revocation."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.scope_binding import ScopeBindingAuthority
from ecm.host_gateway import AuthenticatedSession, HostAdmissionGateway


def make_dag():
    dag = AMXEventDAG("a" * 64)
    dag.append_event(AMXEvent(
        "1" * 64, "RECORD_CREATED", dag.record_id, [], "producer",
        {"title": "approved", "content": "safe context",
         "admission": "ADMITTED", "validity": "VALID"},
        timestamp="2026-11-11T00:00:00+00:00",
    ))
    return dag


class HostAdmissionGatewayTests(unittest.TestCase):
    def setUp(self):
        self.dag = make_dag()
        self.authority = ScopeBindingAuthority(b"r004-test-host-only-secret-32-bytes-long")
        self.scope = self.authority.issue(
            record_id=self.dag.record_id,
            dag_digest=self.dag.compute_dag_digest(),
            tenant_id="tenant-a", project_id="project-a",
            allowed_purposes=("DEV",),
        )
        self.connections = {
            "trusted-connection": AuthenticatedSession(
                "alice", "tenant-a", "project-a", "session-alice"),
            "another-session": AuthenticatedSession(
                "alice", "tenant-a", "project-a", "session-other"),
            "foreign-connection": AuthenticatedSession(
                "bob", "tenant-b", "project-b", "session-bob"),
        }
        self.permissions = {"alice": True, "bob": False}
        self.gateway = HostAdmissionGateway(
            tenant_id="tenant-a", project_id="project-a",
            scope_authority=self.authority, scope_bindings=(self.scope,),
            authenticate=lambda connection: self.connections.get(connection),
            authorize=lambda identity, task, purpose:
                self.permissions.get(identity.principal_id, False)
                and task == "task-1" and purpose == "DEV",
        )

    def test_authenticated_authorized_session_reads_once(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.assertIsNotNone(req)
        result = self.gateway.read("trusted-connection", req, [self.dag])
        self.assertEqual(len(result["admitted_records"]), 1)

    def test_unauthenticated_connection_cannot_issue(self):
        self.assertIsNone(self.gateway.issue("attacker", "task-1", "DEV"))

    def test_policy_denial_cannot_issue(self):
        self.assertIsNone(self.gateway.issue("foreign-connection", "task-1", "DEV"))

    def test_foreign_session_cannot_redeem_request(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.assertEqual(
            self.gateway.read("another-session", req, [self.dag])["admitted_records"], []
        )

    def test_request_replay_fails_closed(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.assertEqual(len(self.gateway.read("trusted-connection", req, [self.dag])["admitted_records"]), 1)
        self.assertEqual(self.gateway.read("trusted-connection", req, [self.dag])["admitted_records"], [])

    def test_revocation_before_redeem_denied(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.gateway.revoke_session("session-alice")
        self.assertEqual(self.gateway.read("trusted-connection", req, [self.dag])["admitted_records"], [])

    def test_policy_rechecked_before_read(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.permissions["alice"] = False
        self.assertEqual(self.gateway.read("trusted-connection", req, [self.dag])["admitted_records"], [])

    def test_forged_authorization_labels_are_ignored(self):
        self.assertIsNone(self.gateway.issue(
            {"principal_id": "alice", "tenant_id": "tenant-a",
             "project_id": "project-a", "authorized": True}, "task-1", "DEV"))

    def test_gateway_is_not_an_execution_grant(self):
        req = self.gateway.issue("trusted-connection", "task-1", "DEV")
        self.assertFalse(hasattr(req, "approval_ref"))
        self.assertFalse(hasattr(self.gateway, "execute"))


if __name__ == "__main__":
    unittest.main()
