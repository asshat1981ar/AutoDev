"""Durable one-use AMCX read admission ledger (SQLite stdlib).

Only trusted host code may call register/consume/revoke_session. This store
does NOT authenticate identity or verify signed requests: the host gateway
must do both. SQLite locking works for cooperating local processes sharing
one database file on a filesystem with correct SQLite locking semantics.
"""
from contextlib import closing
import hashlib
import json
import sqlite3
import time


class SQLiteReplayLedger:
    """Transactional nonce consumption and session revocation, scoped to one tenant/project."""

    def __init__(self, path: str, *, tenant_id: str, project_id: str):
        if not isinstance(path, str) or not path:
            raise ValueError("SQLite path required")
        if not all(isinstance(v, str) and v.strip() for v in (tenant_id, project_id)):
            raise ValueError("tenant and project scope required")
        self.path = path
        self.tenant_id = tenant_id
        self.project_id = project_id
        with closing(self._connect()) as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("""CREATE TABLE IF NOT EXISTS pending_read (
                nonce_key TEXT PRIMARY KEY,
                signature_key TEXT NOT NULL,
                principal_key TEXT NOT NULL,
                session_key TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                expires_at INTEGER NOT NULL
            )""")
            conn.execute("""CREATE TABLE IF NOT EXISTS revoked_session (
                session_key TEXT PRIMARY KEY,
                revoked_at INTEGER NOT NULL
            )""")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_pending_expiry ON pending_read(expires_at)")

    def _connect(self):
        conn = sqlite3.connect(self.path, timeout=15, isolation_level=None)
        conn.execute("PRAGMA busy_timeout=15000")
        return conn

    @staticmethod
    def _hash(*parts):
        blob = json.dumps(list(parts), separators=(",", ":"), ensure_ascii=True).encode("utf-8")
        return hashlib.sha256(blob).hexdigest()

    def _session_key(self, session_id):
        return self._hash("AMCX-session-v1", self.tenant_id, self.project_id, session_id)

    def _principal_key(self, principal_id):
        return self._hash("AMCX-principal-v1", self.tenant_id, self.project_id, principal_id)

    @staticmethod
    def _nonce_key(nonce):
        return SQLiteReplayLedger._hash("AMCX-nonce-v1", nonce)

    @staticmethod
    def _signature_key(signature):
        return SQLiteReplayLedger._hash("AMCX-signature-v1", signature)

    def _matching(self, request, identity):
        return (request is not None and identity is not None
                and request.tenant_id == self.tenant_id
                and request.project_id == self.project_id
                and identity.tenant_id == self.tenant_id
                and identity.project_id == self.project_id
                and request.principal_id == identity.principal_id
                and isinstance(identity.session_id, str) and bool(identity.session_id)
                and isinstance(request.nonce, str) and len(request.nonce) == 32
                and isinstance(request.signature, str)
                and type(request.expires_at) is int)

    def register(self, request, identity):
        """Register a trusted-host-verified request; no signatures or content stored."""
        if not self._matching(request, identity) or request.expires_at <= int(time.time()):
            return False
        nonce = self._nonce_key(request.nonce)
        session = self._session_key(identity.session_id)
        with closing(self._connect()) as conn:
            try:
                conn.execute("BEGIN IMMEDIATE")
                conn.execute("DELETE FROM pending_read WHERE expires_at <= ?", (int(time.time()),))
                if conn.execute("SELECT 1 FROM revoked_session WHERE session_key=?", (session,)).fetchone():
                    conn.execute("ROLLBACK")
                    return False
                cursor = conn.execute(
                    """INSERT OR IGNORE INTO pending_read
                       (nonce_key, signature_key, principal_key, session_key, tenant_id, project_id, expires_at)
                       VALUES (?, ?, ?, ?, ?, ?, ?)""",
                    (nonce, self._signature_key(request.signature),
                     self._principal_key(identity.principal_id),
                     session, self.tenant_id, self.project_id, request.expires_at),
                )
                success = cursor.rowcount == 1
                conn.execute("COMMIT")
                return success
            except BaseException:
                # A failed BEGIN (or corrupted connection) has no active
                # transaction; attempting ROLLBACK would mask the real error.
                if conn.in_transaction:
                    conn.rollback()
                raise

    def consume(self, request, identity):
        """Atomically consume a nonce once, across gateways sharing this DB."""
        if not self._matching(request, identity) or request.expires_at <= int(time.time()):
            return False
        nonce = self._nonce_key(request.nonce)
        session = self._session_key(identity.session_id)
        with closing(self._connect()) as conn:
            try:
                conn.execute("BEGIN IMMEDIATE")
                row = conn.execute(
                    """SELECT signature_key, principal_key, session_key, tenant_id,
                              project_id, expires_at FROM pending_read WHERE nonce_key=?""",
                    (nonce,),
                ).fetchone()
                if row is None or conn.execute(
                    "SELECT 1 FROM revoked_session WHERE session_key=?", (session,)
                ).fetchone():
                    conn.execute("ROLLBACK")
                    return False
                expected = (self._signature_key(request.signature),
                            self._principal_key(identity.principal_id),
                            session, self.tenant_id, self.project_id, request.expires_at)
                if tuple(row) != expected or request.expires_at <= int(time.time()):
                    conn.execute("ROLLBACK")
                    return False
                count = conn.execute(
                    "DELETE FROM pending_read WHERE nonce_key=?", (nonce,)
                ).rowcount
                conn.execute("COMMIT")
                return count == 1
            except BaseException:
                # A failed BEGIN (or corrupted connection) has no active
                # transaction; attempting ROLLBACK would mask the real error.
                if conn.in_transaction:
                    conn.rollback()
                raise

    def revoke_session(self, session_id: str):
        """Durably revoke one session and delete its still-pending requests."""
        if not isinstance(session_id, str) or not session_id:
            raise ValueError("nonempty session ID required")
        session = self._session_key(session_id)
        with closing(self._connect()) as conn:
            try:
                conn.execute("BEGIN IMMEDIATE")
                conn.execute(
                    "INSERT OR IGNORE INTO revoked_session(session_key, revoked_at) VALUES (?, ?)",
                    (session, int(time.time())),
                )
                conn.execute("DELETE FROM pending_read WHERE session_key=?", (session,))
                conn.execute("COMMIT")
            except BaseException:
                # A failed BEGIN (or corrupted connection) has no active
                # transaction; attempting ROLLBACK would mask the real error.
                if conn.in_transaction:
                    conn.rollback()
                raise
