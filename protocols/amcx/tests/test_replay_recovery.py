"""R-006 security regressions: independent processes, crash durability, storage errors.

Tests are isolated to a temporary SQLite database. They never exercise
ForgeCore execution or modify an existing repository memory store.
"""
import json
import os
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
from dataclasses import asdict
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
sys.path.insert(0, str(SRC))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.host_gateway import AuthenticatedSession, HostAdmissionGateway
from ecm.replay_store import SQLiteReplayLedger
from ecm.scope_binding import ScopeBindingAuthority, TrustedAdmissionRequest


# Run only as child OS processes. An independent interpreter and SQLite connection
# is required; a thread-level race cannot substitute for this.
WORKER = """
import json, os, sqlite3, sys, time
from pathlib import Path
from ecm.replay_store import SQLiteReplayLedger
from ecm.scope_binding import TrustedAdmissionRequest
from ecm.host_gateway import AuthenticatedSession
path, mode, request_data, identity_data, flag, ready = sys.argv[1:]
req = TrustedAdmissionRequest(**json.loads(request_data))
identity = AuthenticatedSession(**json.loads(identity_data))
store = SQLiteReplayLedger(path, tenant_id='tenant-a', project_id='project-a')
if ready != '-':
    Path(ready).write_text('ready')
    deadline = time.monotonic() + 10
    while not Path(flag).exists() and time.monotonic() < deadline:
        time.sleep(.01)
    if not Path(flag).exists():
        sys.exit(9)
if mode == 'consume':
    print(int(store.consume(req, identity)), flush=True)
elif mode == 'consume-crash':
    success = store.consume(req, identity)
    os._exit(0 if success else 8)
elif mode == 'revoke-crash':
    store.revoke_session(identity.session_id)
    os._exit(0)
elif mode == 'revoke':
    store.revoke_session(identity.session_id)
    print('revoked', flush=True)
elif mode == 'uncommitted-crash':
    connection = sqlite3.connect(path)
    connection.execute('BEGIN IMMEDIATE')
    connection.execute('DELETE FROM pending_read')
    os._exit(19)
else:
    sys.exit(3)
"""


class ReplayRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.db_path = str(Path(self.temp.name) / "replay.sqlite3")
        self.authority = ScopeBindingAuthority(b"r006-security-tests-use-only-host-secret-32bytes")
        self.identity = AuthenticatedSession("alice", "tenant-a", "project-a", "session-1")
        self.dag = AMXEventDAG("a" * 64)
        self.dag.append_event(AMXEvent(
            "1" * 64, "RECORD_CREATED", self.dag.record_id, [], "producer",
            {"title": "allowed", "content": "safe memory"},
            timestamp="2026-11-11T00:00:00+00:00"))
        binding = self.authority.issue(
            record_id=self.dag.record_id, dag_digest=self.dag.compute_dag_digest(),
            tenant_id="tenant-a", project_id="project-a", allowed_purposes=("DEV",))
        self.binding = binding
        self.ledger = SQLiteReplayLedger(
            self.db_path, tenant_id="tenant-a", project_id="project-a")

    def gateway(self):
        return HostAdmissionGateway(
            tenant_id="tenant-a", project_id="project-a",
            scope_authority=self.authority, scope_bindings=(self.binding,),
            authenticate=lambda c: self.identity if c == "trusted" else None,
            authorize=lambda i, task, purpose: task == "task-1" and purpose == "DEV",
            replay_store=self.ledger)

    def request(self):
        req = self.authority.issue_request(
            principal_id="alice", tenant_id="tenant-a", project_id="project-a",
            task_id="task-1", purpose="DEV", expires_at=int(time.time()) + 60)
        self.assertTrue(self.ledger.register(req, self.identity))
        return req

    def spawn(self, mode, req, flag="-", ready="-"):
        env = dict(os.environ, PYTHONPATH=str(SRC))
        return subprocess.Popen(
            [sys.executable, "-c", WORKER, self.db_path, mode,
             json.dumps(asdict(req)), json.dumps(asdict(self.identity)), flag, ready],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)

    def test_independent_os_processes_have_exactly_one_redeemer(self):
        req = self.request()
        flag = str(Path(self.temp.name) / "go")
        ready = [str(Path(self.temp.name) / ("ready" + str(i))) for i in range(2)]
        workers = [self.spawn("consume", req, flag, ready[i]) for i in range(2)]
        try:
            deadline = time.monotonic() + 10
            while not all(Path(p).exists() for p in ready) and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(all(Path(p).exists() for p in ready), "workers did not reach race barrier")
            Path(flag).write_text("go")
            results = [worker.communicate(timeout=15) for worker in workers]
            self.assertEqual([worker.returncode for worker in workers], [0, 0], results)
            self.assertEqual(sorted(int(stdout.strip()) for stdout, _ in results), [0, 1])
        finally:
            for worker in workers:
                if worker.poll() is None:
                    worker.kill()
                    worker.communicate()

    def test_crash_before_commit_rolls_back(self):
        req = self.request()
        child = self.spawn("uncommitted-crash", req)
        child.communicate(timeout=10)
        self.assertEqual(child.returncode, 19)
        self.assertTrue(self.ledger.consume(req, self.identity))
        self.assertFalse(self.ledger.consume(req, self.identity))

    def test_crash_after_commit_does_not_resurrect_request(self):
        req = self.request()
        child = self.spawn("consume-crash", req)
        out, err = child.communicate(timeout=10)
        self.assertEqual(child.returncode, 0, err)
        self.assertFalse(self.ledger.consume(req, self.identity))

    def test_revocation_persists_after_worker_crash(self):
        req = self.request()
        child = self.spawn("revoke-crash", req)
        out, err = child.communicate(timeout=10)
        self.assertEqual(child.returncode, 0, err)
        self.assertFalse(self.ledger.consume(req, self.identity))
        self.assertIsNone(self.gateway().issue("trusted", "task-1", "DEV"))

    def test_cross_process_revocation_race_is_serialized(self):
        req = self.request()
        flag = str(Path(self.temp.name) / "race-go")
        ready = [str(Path(self.temp.name) / ("race-ready-" + str(i))) for i in range(2)]
        consume = self.spawn("consume", req, flag, ready[0])
        revoke = self.spawn("revoke", req, flag, ready[1])
        workers = [consume, revoke]
        try:
            deadline = time.monotonic() + 10
            while not all(Path(p).exists() for p in ready) and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(all(Path(p).exists() for p in ready), "race barrier timed out")
            Path(flag).write_text("go")
            results = [worker.communicate(timeout=15) for worker in workers]
            self.assertEqual([worker.returncode for worker in workers], [0, 0], results)
            self.assertIn(results[0][0].strip(), ("0", "1"))
            self.assertEqual(results[1][0].strip(), "revoked")
            self.assertFalse(self.ledger.consume(req, self.identity))
            self.assertIsNone(self.gateway().issue("trusted", "task-1", "DEV"))
        finally:
            for worker in workers:
                if worker.poll() is None:
                    worker.kill()
                    worker.communicate()

    def test_corrupt_database_denies_gateway_issue_and_read(self):
        gateway = self.gateway()
        req = gateway.issue("trusted", "task-1", "DEV")
        self.assertIsNotNone(req)
        Path(self.db_path).write_bytes(b"corrupt SQLite header - R006")
        self.assertIsNone(gateway.issue("trusted", "task-1", "DEV"))
        result = gateway.read("trusted", req, [self.dag])
        self.assertEqual(result["admitted_records"], [])

    def test_incompatible_schema_never_silently_recreated(self):
        gateway = self.gateway()
        with sqlite3.connect(self.db_path) as connection:
            connection.execute("DROP TABLE pending_read")
        self.assertIsNone(gateway.issue("trusted", "task-1", "DEV"))
        with sqlite3.connect(self.db_path) as connection:
            tables = {name for (name,) in connection.execute(
                "SELECT name FROM sqlite_master WHERE type='table'")}
        self.assertNotIn("pending_read", tables)

    def test_failed_administrative_revocation_is_not_reported_successful(self):
        from unittest.mock import patch
        gateway = self.gateway()
        with patch.object(
            self.ledger, "revoke_session",
            side_effect=sqlite3.OperationalError("database unavailable"),
        ):
            with self.assertRaises(sqlite3.OperationalError):
                gateway.revoke_session(self.identity.session_id)

    def test_storage_error_never_falls_back_to_memory(self):
        from unittest.mock import patch
        gateway = self.gateway()
        with patch.object(self.ledger, "register", side_effect=sqlite3.OperationalError("locked")):
            self.assertIsNone(gateway.issue("trusted", "task-1", "DEV"))
        req = gateway.issue("trusted", "task-1", "DEV")
        self.assertIsNotNone(req)
        with patch.object(self.ledger, "consume", side_effect=sqlite3.OperationalError("locked")):
            self.assertEqual(gateway.read("trusted", req, [self.dag])["admitted_records"], [])
        # Failed consumption must not fall back into the in-memory store.
        self.assertEqual(gateway._pending, {})


if __name__ == "__main__":
    unittest.main()
