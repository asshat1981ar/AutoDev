"""R-009A: controlled filesystem identity and SQLite sidecar regressions.

Only R8-03 is carried forward from R-008; no rollback tests or rollback
architecture are modified. Temporary private directories are used throughout.
"""
from concurrent.futures import ThreadPoolExecutor
from contextlib import closing
from pathlib import Path
from unittest.mock import patch
import os
import sqlite3
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from ecm.replay_store import SQLiteReplayLedger


class SQLiteIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.db = self.directory / "ledger.sqlite3"

    def ledger(self, path=None):
        return SQLiteReplayLedger(
            str(path or self.db), tenant_id="tenant-a", project_id="project-a")

    def test_r8_03_rejects_identity_swap_at_sqlite_open(self):
        self.ledger()
        replacement = self.directory / "replacement.sqlite3"
        other = self.ledger(replacement)
        other.revoke_session("replacement-marker")
        original_connect = SQLiteReplayLedger._connect
        changed = [False]
        target = self.db

        def swap_before_open(instance):
            if not changed[0] and Path(instance.path) == target:
                changed[0] = True
                os.replace(replacement, target)
            return original_connect(instance)

        with patch.object(SQLiteReplayLedger, "_connect", swap_before_open):
            with self.assertRaises(
                (OSError, ValueError, sqlite3.DatabaseError),
                msg="R8-03: same-scope replacement accepted after checking file"):
                self.ledger()
        self.assertTrue(changed[0], "the test did not reach its swap hook")

    def test_existing_symlink_is_denied(self):
        self.ledger()
        original = self.directory / "original.sqlite3"
        os.rename(self.db, original)
        self.db.symlink_to(original)
        with self.assertRaises((OSError, ValueError)):
            self.ledger()

    def test_existing_hardlink_is_denied_for_conservative_identity_policy(self):
        self.ledger()
        alias = self.directory / "alias.sqlite3"
        os.link(self.db, alias)
        with self.assertRaises((OSError, ValueError)):
            self.ledger()

    def test_wal_and_shm_are_private_when_present(self):
        self.ledger()
        with closing(sqlite3.connect(str(self.db))) as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("CREATE TABLE IF NOT EXISTS sidecar_probe (i INTEGER)")
            conn.commit()
            # A second connection forces shared-memory handling on platforms
            # that support it; do not assume both sidecars must always exist.
            with closing(sqlite3.connect(str(self.db))) as second:
                second.execute("SELECT count(*) FROM pending_read").fetchone()
                sidecars = (Path(str(self.db) + "-wal"), Path(str(self.db) + "-shm"))
                present = [p for p in sidecars if p.exists()]
                self.assertTrue(present, "no SQLite WAL/SHM sidecar created")
                for path in present:
                    info = path.lstat()
                    self.assertTrue(stat.S_ISREG(info.st_mode))
                    self.assertEqual(stat.S_IMODE(info.st_mode) & 0o077, 0,
                                     str(path) + " is not owner-only")

    def test_legitimate_concurrent_independent_ledger_instances(self):
        self.ledger()
        def reopen(_):
            store = self.ledger()
            return store.tenant_id, store.project_id
        with ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(reopen, range(8)))
        self.assertEqual(results, [("tenant-a", "project-a")] * 8)

    def test_checked_file_has_same_identity_on_clean_reopen(self):
        self.ledger()
        before = self.db.stat()
        self.ledger()
        after = self.db.stat()
        self.assertEqual((before.st_dev, before.st_ino),
                         (after.st_dev, after.st_ino))


if __name__ == "__main__":
    unittest.main()
