"""R-007 RED: versioned SQLite replay-ledger initialization and safe lifecycle."""
import os
import sqlite3
import stat
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from ecm.replay_store import SQLiteReplayLedger
from ecm.host_gateway import AuthenticatedSession
from ecm.scope_binding import ScopeBindingAuthority


class LedgerLifecycleTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / "admission.sqlite3"
        self.authority = ScopeBindingAuthority(b"r007-only-for-security-test-secret-length")
        self.identity = AuthenticatedSession("alice", "tenant-a", "project-a", "session-a")

    def store(self, **options):
        return SQLiteReplayLedger(
            str(self.path), tenant_id="tenant-a", project_id="project-a",
            **options,
        )

    def request(self, expiry=None):
        return self.authority.issue_request(
            principal_id="alice", tenant_id="tenant-a", project_id="project-a",
            task_id="task-1", purpose="DEV",
            expires_at=expiry or int(time.time()) + 600,
        )

    def test_fresh_db_has_versioned_complete_schema(self):
        self.store()
        with sqlite3.connect(self.path) as conn:
            self.assertEqual(conn.execute("PRAGMA user_version").fetchone()[0], 1)
            self.assertEqual(conn.execute("PRAGMA quick_check").fetchone()[0], "ok")
            tables = {r[0] for r in conn.execute(
                "SELECT name FROM sqlite_master WHERE type='table'")}
        self.assertTrue({"pending_read", "revoked_session"}.issubset(tables))

    def test_legacy_unversioned_complete_schema_upgrades_without_data_loss(self):
        store = self.store()
        pending = self.request()
        self.assertTrue(store.register(pending, self.identity))
        with sqlite3.connect(self.path) as conn:
            conn.execute("PRAGMA user_version=0")
        upgraded = self.store()
        with sqlite3.connect(self.path) as conn:
            self.assertEqual(conn.execute("PRAGMA user_version").fetchone()[0], 1)
        self.assertTrue(upgraded.consume(pending, self.identity))
        self.assertFalse(upgraded.consume(pending, self.identity))

    def test_unknown_future_schema_version_is_rejected_without_mutation(self):
        self.store()
        with sqlite3.connect(self.path) as conn:
            conn.execute("PRAGMA user_version=99")
        with self.assertRaises((ValueError, sqlite3.DatabaseError)):
            self.store()
        with sqlite3.connect(self.path) as conn:
            self.assertEqual(conn.execute("PRAGMA user_version").fetchone()[0], 99)

    def test_partial_schema_rejected_not_silently_recreated(self):
        self.store()
        with sqlite3.connect(self.path) as conn:
            conn.execute("DROP TABLE revoked_session")
        with self.assertRaises((ValueError, sqlite3.DatabaseError)):
            self.store()
        with sqlite3.connect(self.path) as conn:
            count = conn.execute(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='revoked_session'"
            ).fetchone()[0]
        self.assertEqual(count, 0)

    def test_world_writable_parent_directory_rejected(self):
        unsafe = Path(self.tmp.name) / "unsafe-parent"
        unsafe.mkdir()
        os.chmod(unsafe, 0o777)
        self.path = unsafe / "replay.sqlite3"
        with self.assertRaises((OSError, ValueError)):
            self.store()

    def test_nonce_key_must_retain_primary_key_constraint(self):
        self.store()
        with sqlite3.connect(self.path) as conn:
            conn.execute("DROP TABLE pending_read")
            conn.execute("""CREATE TABLE pending_read (
                nonce_key TEXT,
                signature_key TEXT NOT NULL,
                principal_key TEXT NOT NULL,
                session_key TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                expires_at INTEGER NOT NULL
            )""")
        with self.assertRaises((OSError, ValueError, sqlite3.DatabaseError)):
            self.store()

    def test_symlink_database_path_rejected(self):
        actual = Path(self.tmp.name) / "actual.sqlite3"
        self.store()
        os.rename(self.path, actual)
        self.path.symlink_to(actual)
        with self.assertRaises((OSError, ValueError)):
            self.store()

    def test_world_readable_existing_db_rejected(self):
        self.store()
        os.chmod(self.path, 0o644)
        with self.assertRaises((OSError, ValueError)):
            self.store()

    def test_new_db_is_private_to_owner(self):
        self.store()
        self.assertEqual(stat.S_IMODE(self.path.stat().st_mode) & 0o077, 0)

    def test_configured_pending_capacity_fails_closed_and_releases_on_use(self):
        store = self.store(max_pending=1)
        a, b = self.request(), self.request()
        self.assertTrue(store.register(a, self.identity))
        self.assertFalse(store.register(b, self.identity))
        self.assertTrue(store.consume(a, self.identity))
        self.assertTrue(store.register(b, self.identity))

    def test_expired_pending_purged_when_ledger_reopens(self):
        store = self.store()
        now = int(time.time())
        req = self.request(expiry=now + 10)
        self.assertTrue(store.register(req, self.identity))
        with patch("ecm.replay_store.time.time", return_value=now + 11):
            self.store()
        with sqlite3.connect(self.path) as conn:
            remaining = conn.execute("SELECT count(*) FROM pending_read").fetchone()[0]
        self.assertEqual(remaining, 0)

    def test_revocation_persists_across_reopen_and_expiry_cleanup(self):
        store = self.store()
        req = self.request()
        self.assertTrue(store.register(req, self.identity))
        store.revoke_session("session-a")
        with patch("ecm.replay_store.time.time", return_value=int(time.time()) + 3600):
            self.store()
        self.assertFalse(self.store().register(self.request(), self.identity))
        with sqlite3.connect(self.path) as conn:
            count = conn.execute("SELECT count(*) FROM revoked_session").fetchone()[0]
        self.assertEqual(count, 1)


    def test_duplicate_registration_does_not_replace_original_session(self):
        from dataclasses import replace
        store = self.store()
        request = self.request()
        self.assertTrue(store.register(request, self.identity))
        other = replace(self.identity, session_id="another-session")
        self.assertFalse(store.register(request, other))
        self.assertFalse(store.consume(request, other))
        self.assertTrue(store.consume(request, self.identity))
        self.assertFalse(store.consume(request, self.identity))

    def test_mismatched_signature_or_expiry_does_not_consume_original(self):
        from dataclasses import replace
        store = self.store()
        request = self.request()
        self.assertTrue(store.register(request, self.identity))
        for changed in (replace(request, signature="0" * 64),
                        replace(request, expires_at=request.expires_at + 1)):
            with self.subTest(request=changed):
                self.assertFalse(store.consume(changed, self.identity))
        self.assertTrue(store.consume(request, self.identity))

    def test_session_revocation_is_scoped_in_shared_database(self):
        store = self.store()
        other = SQLiteReplayLedger(
            str(self.path), tenant_id="tenant-a", project_id="project-b",
        )
        other.revoke_session(self.identity.session_id)
        request = self.request()
        self.assertTrue(store.register(request, self.identity))
        self.assertTrue(store.consume(request, self.identity))

    def test_expiry_boundary_reclaims_capacity_during_registration(self):
        with patch("ecm.replay_store.time.time", return_value=1000):
            store = self.store(max_pending=1)
            expired = self.request(expiry=1001)
            replacement = self.request(expiry=1002)
            self.assertTrue(store.register(expired, self.identity))
            self.assertFalse(store.register(replacement, self.identity))
        with patch("ecm.replay_store.time.time", return_value=1001):
            self.assertFalse(store.consume(expired, self.identity))
            self.assertTrue(store.register(replacement, self.identity))
            self.assertTrue(store.consume(replacement, self.identity))


if __name__ == "__main__":
    unittest.main()
