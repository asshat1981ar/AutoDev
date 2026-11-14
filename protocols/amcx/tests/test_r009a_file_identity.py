"""R-009A filesystem identity safety probes, isolated to temporary SQLite stores.

Baseline is R-007. R8-01/02 rollback tests remain ONLY in R-008 PR #64.
This test module adds no credentials, execution grants, or network adapters.
"""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unittest.mock import patch
import os
import sqlite3
import stat
import sys
import tempfile
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from ecm.host_gateway import AuthenticatedSession
from ecm.replay_store import SQLiteReplayLedger
from ecm.scope_binding import ScopeBindingAuthority


class SQLiteFilesystemIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.database = Path(self.temp.name) / "ledger.sqlite3"
        self.authority = ScopeBindingAuthority(b"r009a-test-secret-exclusively-for-ci-32bytes")
        self.identity = AuthenticatedSession("alice", "tenant-a", "project-a", "session-a")

    def store(self, path=None):
        return SQLiteReplayLedger(
            str(path if path is not None else self.database),
            tenant_id="tenant-a", project_id="project-a",
        )

    def request(self):
        return self.authority.issue_request(
            principal_id="alice", tenant_id="tenant-a", project_id="project-a",
            task_id="task-1", purpose="DEV", expires_at=int(time.time()) + 60,
        )

    def test_r8_03_checked_database_replaced_before_sqlite_open_fails_closed(self):
        self.store()
        replacement = Path(self.temp.name) / "replacement.sqlite3"
        self.store(replacement).revoke_session("marker")
        original_connect = SQLiteReplayLedger._connect
        swapped = [False]

        def swap_before_connect(instance):
            if not swapped[0] and Path(instance.path) == self.database:
                swapped[0] = True
                os.replace(str(replacement), str(self.database))
            return original_connect(instance)

        with patch.object(SQLiteReplayLedger, "_connect", swap_before_connect):
            with self.assertRaises((OSError, ValueError, sqlite3.DatabaseError)):
                self.store()
        self.assertTrue(swapped[0], "R8-03 swap injection never executed")

    def test_post_initialization_database_replacement_denies_redemption(self):
        ledger = self.store()
        request = self.request()
        self.assertTrue(ledger.register(request, self.identity))
        replacement = Path(self.temp.name) / "replacement.sqlite3"
        self.store(replacement)
        os.replace(str(replacement), str(self.database))
        with self.assertRaises((OSError, ValueError, sqlite3.DatabaseError)):
            ledger.consume(request, self.identity)

    def test_symlink_path_rejected_at_initialization(self):
        self.store()
        original = Path(self.temp.name) / "original.sqlite3"
        self.database.rename(original)
        self.database.symlink_to(original)
        with self.assertRaises((OSError, ValueError)):
            self.store()

    @unittest.skipUnless(hasattr(os, "link"), "hard links unavailable")
    def test_hardlinked_database_rejected_at_initialization(self):
        self.store()
        alias = Path(self.temp.name) / "alias.sqlite3"
        os.link(str(self.database), str(alias))
        with self.assertRaises((OSError, ValueError)):
            self.store()

    def test_wal_sidecar_files_are_not_world_readable(self):
        ledger = self.store()
        with sqlite3.connect(ledger.path) as connection:
            connection.execute("PRAGMA journal_mode=WAL")
            connection.execute("PRAGMA wal_autocheckpoint=0")
            connection.execute("BEGIN IMMEDIATE")
            connection.execute("INSERT INTO revoked_session VALUES (?, ?)",
                               ("sidecar-probe", int(time.time())))
            connection.commit()
            for suffix in ("-wal", "-shm"):
                sidecar = Path(ledger.path + suffix)
                self.assertTrue(sidecar.exists(), sidecar.name)
                self.assertEqual(stat.S_IMODE(sidecar.stat().st_mode) & 0o077, 0)

    def test_legitimate_concurrent_redemption_still_has_one_winner(self):
        ledger = self.store()
        request = self.request()
        self.assertTrue(ledger.register(request, self.identity))
        with ThreadPoolExecutor(max_workers=4) as pool:
            outcomes = list(pool.map(
                lambda _: ledger.consume(request, self.identity), range(4)))
        self.assertEqual(outcomes.count(True), 1)
        self.assertEqual(outcomes.count(False), 3)


if __name__ == "__main__":
    unittest.main()
