"""R-008 RED-only security oracles: replay rollback and SQLite path substitution.

Run against the untouched R-007 implementation. Every test isolates its
database in a temporary directory; no production repository files are opened.
These checks intentionally assert properties that R-007 might not implement.
"""
from contextlib import closing
from pathlib import Path
from unittest.mock import patch
import os
import sqlite3
import sys
import tempfile
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from ecm.host_gateway import AuthenticatedSession
from ecm.replay_store import SQLiteReplayLedger
from ecm.scope_binding import ScopeBindingAuthority


class RollbackAndPathSecurityTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.database = Path(self.directory.name) / "ledger.sqlite3"
        self.snapshot = Path(self.directory.name) / "before.sqlite3"
        self.authority = ScopeBindingAuthority(
            b"r008-test-only-receiver-key-not-for-production"
        )
        self.identity = AuthenticatedSession(
            "alice", "tenant-a", "project-a", "session-unique-008"
        )

    def ledger(self, path=None):
        return SQLiteReplayLedger(
            str(path or self.database),
            tenant_id="tenant-a", project_id="project-a",
        )

    def request(self):
        return self.authority.issue_request(
            principal_id="alice", tenant_id="tenant-a", project_id="project-a",
            task_id="task-1", purpose="DEV", expires_at=int(time.time()) + 600,
        )

    @staticmethod
    def snapshot_db(source, dest):
        # SQLite backup API produces a transactionally consistent snapshot,
        # including current WAL content. Avoid copying live SQLite files.
        with closing(sqlite3.connect(str(source))) as src:
            with closing(sqlite3.connect(str(dest))) as dst:
                src.backup(dst)

    def test_r8_01_consumed_request_cannot_resurrect_after_valid_backup_restore(self):
        ledger = self.ledger()
        req = self.request()
        self.assertTrue(ledger.register(req, self.identity))
        self.snapshot_db(self.database, self.snapshot)
        self.assertTrue(ledger.consume(req, self.identity))
        self.assertFalse(ledger.consume(req, self.identity))

        # Restore an older *valid* snapshot into the same path. Recovery
        # must not silently authorize a previously consumed request.
        self.snapshot_db(self.snapshot, self.database)
        reloaded = self.ledger()
        self.assertFalse(
            reloaded.consume(req, self.identity),
            "R8-01: restored older SQLite backup resurrected a consumed nonce",
        )

    def test_r8_02_revoked_session_cannot_resurrect_after_valid_backup_restore(self):
        ledger = self.ledger()
        req = self.request()
        self.assertTrue(ledger.register(req, self.identity))
        self.snapshot_db(self.database, self.snapshot)
        ledger.revoke_session(self.identity.session_id)
        self.assertFalse(ledger.consume(req, self.identity))
        self.assertFalse(ledger.register(self.request(), self.identity))

        self.snapshot_db(self.snapshot, self.database)
        reloaded = self.ledger()
        self.assertFalse(
            reloaded.consume(req, self.identity),
            "R8-02: restoring a pre-revocation backup restored revoked access",
        )
        self.assertFalse(
            reloaded.register(self.request(), self.identity),
            "R8-02: restored database accepted new requests for revoked session",
        )

    def test_r8_03_path_replacement_between_check_and_connect_rejected(self):
        self.ledger()
        replacement = Path(self.directory.name) / "replacement.sqlite3"
        replacement_ledger = self.ledger(replacement)
        replacement_ledger.revoke_session("replacement-marker")

        # Both paths are legitimate owner-only SQLite files with valid v1
        # schemas. Attack replaces checked path after validation, just before
        # _connect. This is a controlled TOCTOU injection, not a timing gamble.
        original_connect = SQLiteReplayLedger._connect
        switched = [False]
        target = self.database

        def swapped_connect(instance):
            if not switched[0] and Path(instance.path) == target:
                switched[0] = True
                os.replace(str(replacement), str(target))
            return original_connect(instance)

        with patch.object(SQLiteReplayLedger, "_connect", swapped_connect):
            with self.assertRaises(
                (OSError, ValueError, sqlite3.DatabaseError),
                msg="R8-03: path substitution after check was accepted",
            ):
                self.ledger()
        self.assertTrue(switched[0], "test did not reach controlled swap point")


if __name__ == "__main__":
    unittest.main()
