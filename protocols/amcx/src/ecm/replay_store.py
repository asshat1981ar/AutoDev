"""Durable one-use AMCX read admission ledger (SQLite stdlib).

Only trusted host code may call register/consume/revoke_session. This store
does NOT authenticate identity or verify signed requests: the host gateway
must do both. SQLite locking works for cooperating local processes sharing
one database file on a filesystem with correct SQLite locking semantics.
"""
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import stat
import sqlite3
import time


class SQLiteReplayLedger:
    """Transactional nonce consumption and session revocation, scoped to one tenant/project."""

    # Schema is owned by AMCX read admission, not ForgeCore or the memory DAG.
    SCHEMA_VERSION = 1
    DEFAULT_MAX_PENDING = 4096
    _TABLE_COLUMNS = {
        "pending_read": (
            "nonce_key", "signature_key", "principal_key", "session_key",
            "tenant_id", "project_id", "expires_at",
        ),
        "revoked_session": ("session_key", "revoked_at"),
    }

    def __init__(self, path: str, *, tenant_id: str, project_id: str,
                 max_pending: int = DEFAULT_MAX_PENDING):
        if not isinstance(path, str) or not path or path == ":memory:":
            raise ValueError("persistent SQLite database path required")
        if not all(isinstance(v, str) and v.strip() for v in (tenant_id, project_id)):
            raise ValueError("tenant and project scope required")
        if type(max_pending) is not int or max_pending <= 0:
            raise ValueError("max_pending must be a positive integer")
        self.path = path
        self.tenant_id = tenant_id
        self.project_id = project_id
        self.max_pending = max_pending
        self._ensure_private_regular_file()
        with closing(self._connect()) as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("BEGIN IMMEDIATE")
            try:
                check = conn.execute("PRAGMA quick_check").fetchone()
                if not check or check[0] != "ok":
                    raise sqlite3.DatabaseError("SQLite integrity check failed")
                version = conn.execute("PRAGMA user_version").fetchone()[0]
                if version not in (0, self.SCHEMA_VERSION):
                    raise ValueError("unsupported SQLite replay schema version")
                existing = {row[0] for row in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type='table'")}
                named = {name for name in existing if not name.startswith("sqlite_")}
                if version == 0 and not named:
                    # First creation: both tables and schema version commit together.
                    conn.execute("""CREATE TABLE pending_read (
                        nonce_key TEXT PRIMARY KEY,
                        signature_key TEXT NOT NULL,
                        principal_key TEXT NOT NULL,
                        session_key TEXT NOT NULL,
                        tenant_id TEXT NOT NULL,
                        project_id TEXT NOT NULL,
                        expires_at INTEGER NOT NULL
                    )""")
                    conn.execute("""CREATE TABLE revoked_session (
                        session_key TEXT PRIMARY KEY,
                        revoked_at INTEGER NOT NULL
                    )""")
                else:
                    if named != set(self._TABLE_COLUMNS):
                        raise ValueError("incomplete or foreign replay schema")
                    for name, columns in self._TABLE_COLUMNS.items():
                        info = tuple(conn.execute("PRAGMA table_info(" + name + ")"))
                        found = tuple(row[1] for row in info)
                        if found != columns:
                            raise ValueError("replay schema column mismatch: " + name)
                        # The nonce/session identifiers MUST retain uniqueness.
                        # Matching column names alone cannot establish it.
                        if info[0][5] != 1:
                            raise ValueError("replay schema requires a primary key: " + name)
                        if any(row[5] != 0 for row in info[1:]):
                            raise ValueError(
                                "replay schema requires a single-column primary key: " + name
                            )
                        if any(row[3] != 1 for row in info[1:]):
                            raise ValueError("replay schema NOT NULL constraints missing: " + name)
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_pending_expiry "
                    "ON pending_read(expires_at)"
                )
                if version == 0:
                    # Upgrade exact R-005/R-006 legacy shape without deleting rows.
                    conn.execute("PRAGMA user_version=1")
                conn.execute("DELETE FROM pending_read WHERE expires_at <= ?",
                             (int(time.time()),))
                conn.execute("COMMIT")
            except BaseException:
                if conn.in_transaction:
                    conn.rollback()
                raise

    def _ensure_private_regular_file(self):
        """Reject existing unsafe files, create new database with mode 0600.

        The containing directory and all untrusted filesystem actors remain
        a deployment responsibility. The directory must be host-controlled.
        """
        target = Path(self.path)
        parent = target.absolute().parent
        directory = parent.lstat()
        if (not stat.S_ISDIR(directory.st_mode) or parent.is_symlink()
                or directory.st_mode & 0o022):
            raise PermissionError("SQLite replay parent directory must not be writable by others")
        if hasattr(os, "getuid") and directory.st_uid != os.getuid():
            raise PermissionError("SQLite replay parent directory owner mismatch")
        try:
            info = target.lstat()
        except FileNotFoundError:
            # Exclusive creation prevents clobbering a file or link already
            # placed at this path. SQLite then opens that private file.
            flags = os.O_CREAT | os.O_EXCL | os.O_RDWR
            flags |= getattr(os, "O_NOFOLLOW", 0)
            try:
                fd = os.open(self.path, flags, 0o600)
            except FileExistsError:
                info = target.lstat()
            else:
                os.close(fd)
                info = target.lstat()
        if not stat.S_ISREG(info.st_mode):
            raise ValueError("SQLite path must be a regular file, not a link")
        if info.st_mode & 0o077:
            raise PermissionError("SQLite replay database must be owner-only")
        if hasattr(os, "getuid") and info.st_uid != os.getuid():
            raise PermissionError("SQLite replay database owner mismatch")
        # Conservative local-file policy: hard-linked replay databases may
        # permit mutation through another pathname outside this boundary.
        if info.st_nlink != 1:
            raise PermissionError("SQLite replay database must have exactly one hard link")
        self._checked_file_identity = (info.st_dev, info.st_ino)

    def _identity_matches_checked_file(self):
        # Detect observable substitutions before and after sqlite3.connect.
        # This is not an atomic OS-level descriptor binding.
        info = os.lstat(self.path)
        if (not stat.S_ISREG(info.st_mode)
                or info.st_nlink != 1
                or info.st_mode & 0o077
                or (hasattr(os, "getuid") and info.st_uid != os.getuid())
                or (info.st_dev, info.st_ino) != self._checked_file_identity):
            raise PermissionError("SQLite replay database identity changed after validation")

    def _connect(self):
        self._identity_matches_checked_file()
        conn = sqlite3.connect(self.path, timeout=15, isolation_level=None)
        try:
            self._identity_matches_checked_file()
            conn.execute("PRAGMA busy_timeout=15000")
        except BaseException:
            conn.close()
            raise
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
                # Capacity is enforced after expired entries are removed,
                # inside the same write transaction as the pending insert.
                count = conn.execute("SELECT count(*) FROM pending_read").fetchone()[0]
                if count >= self.max_pending:
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
