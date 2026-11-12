"""R-005 RED: durable replay and revocation across gateway instances."""
import concurrent.futures
import tempfile
import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.host_gateway import AuthenticatedSession, HostAdmissionGateway
from ecm.scope_binding import ScopeBindingAuthority
from ecm.replay_store import SQLiteReplayLedger


class DurableReplayTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = str(Path(self.directory.name) / "replay.sqlite3")
        self.authority = ScopeBindingAuthority(b"r005-trusted-host-replay-key-32-bytes")
        self.identity = AuthenticatedSession("alice", "tenant-a", "project-a", "session-1")
        self.dag = AMXEventDAG("a" * 64)
        self.dag.append_event(AMXEvent(
            "1" * 64, "RECORD_CREATED", self.dag.record_id, [], "producer",
            {"title": "safe", "content": "approved"},
            timestamp="2026-11-11T00:00:00+00:00",
        ))
        self.binding = self.authority.issue(
            record_id=self.dag.record_id, dag_digest=self.dag.compute_dag_digest(),
            tenant_id="tenant-a", project_id="project-a", allowed_purposes=("DEV",),
        )
        self.sessions = {"connection": self.identity}
        self.allowed = True

    def ledger(self):
        return SQLiteReplayLedger(self.path, tenant_id="tenant-a", project_id="project-a")

    def gateway(self):
        return HostAdmissionGateway(
            tenant_id="tenant-a", project_id="project-a",
            scope_authority=self.authority, scope_bindings=(self.binding,),
            authenticate=lambda connection: self.sessions.get(connection),
            authorize=lambda identity, task, purpose:
                self.allowed and identity.principal_id == "alice"
                and task == "task-1" and purpose == "DEV",
            replay_store=self.ledger(),
        )

    def test_redeem_on_different_gateway_and_block_replay(self):
        first, second = self.gateway(), self.gateway()
        req = first.issue("connection", "task-1", "DEV")
        self.assertIsNotNone(req)
        self.assertEqual(len(second.read("connection", req, [self.dag])["admitted_records"]), 1)
        self.assertEqual(first.read("connection", req, [self.dag])["admitted_records"], [])

    def test_revocation_survives_gateway_restart(self):
        first = self.gateway()
        req = first.issue("connection", "task-1", "DEV")
        first.revoke_session("session-1")
        restarted = self.gateway()
        self.assertEqual(restarted.read("connection", req, [self.dag])["admitted_records"], [])
        self.assertIsNone(restarted.issue("connection", "task-1", "DEV"))

    def test_concurrent_redemption_has_single_winner(self):
        first, second = self.gateway(), self.gateway()
        req = first.issue("connection", "task-1", "DEV")
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            outcomes = list(pool.map(
                lambda gateway: len(gateway.read("connection", req, [self.dag])["admitted_records"]),
                (first, second),
            ))
        self.assertEqual(sorted(outcomes), [0, 1])

    def test_revoke_and_issuance_are_atomic(self):
        first, second = self.gateway(), self.gateway()
        first.revoke_session("session-1")
        self.assertIsNone(second.issue("connection", "task-1", "DEV"))

    def test_namespace_prevents_cross_project_consume(self):
        first = self.gateway()
        request = first.issue("connection", "task-1", "DEV")
        foreign = SQLiteReplayLedger(self.path, tenant_id="tenant-a", project_id="project-b")
        self.assertFalse(foreign.consume(request, self.identity))
        self.assertEqual(len(first.read("connection", request, [self.dag])["admitted_records"]), 1)

    def test_persists_without_secret_or_payload(self):
        gateway = self.gateway()
        req = gateway.issue("connection", "task-1", "DEV")
        contents = Path(self.path).read_bytes()
        self.assertNotIn(b"r005-trusted-host-replay-key", contents)
        self.assertNotIn(b"approved", contents)
        self.assertNotIn(b"safe", contents)
        self.assertNotIn(req.signature.encode(), contents)

    def test_expired_request_cannot_register(self):
        expired = self.authority.issue_request(
            principal_id="alice", tenant_id="tenant-a", project_id="project-a",
            task_id="task-1", purpose="DEV", expires_at=1,
        )
        self.assertFalse(self.ledger().register(expired, self.identity))


if __name__ == "__main__":
    unittest.main()
