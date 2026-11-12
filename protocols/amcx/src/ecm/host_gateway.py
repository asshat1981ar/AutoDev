"""In-process read-only AMCX host admission gateway.

The injected authenticate and authorize callbacks MUST be trusted host code:
this module does not implement network authentication or create ForgeCore grants.
The in-memory one-use ledger does not provide multi-process replay protection.
"""
from dataclasses import dataclass
import hmac
import threading
import time
from typing import Callable

from adapters.mcp_adapter import AutoDevMCPAdapter
from ecm.scope_binding import ScopeBindingAuthority, TrustedAdmissionRequest


@dataclass(frozen=True)
class AuthenticatedSession:
    principal_id: str
    tenant_id: str
    project_id: str
    session_id: str

    def is_well_formed(self):
        return all(isinstance(v, str) and bool(v.strip()) for v in (
            self.principal_id, self.tenant_id, self.project_id, self.session_id
        ))


class HostAdmissionGateway:
    """Issue and redeem one-use memory-read requests, never execution grants."""

    def __init__(self, *, tenant_id: str, project_id: str,
                 scope_authority: ScopeBindingAuthority, scope_bindings,
                 authenticate: Callable, authorize: Callable):
        if not isinstance(scope_authority, ScopeBindingAuthority):
            raise TypeError("trusted receiver authority required")
        if not callable(authenticate) or not callable(authorize):
            raise TypeError("trusted authentication and policy callbacks required")
        self._authority = scope_authority
        self._authenticate = authenticate
        self._authorize = authorize
        self._adapter = AutoDevMCPAdapter(
            tenant_id, project_id, scope_authority=scope_authority,
            scope_bindings=scope_bindings,
        )
        self.tenant_id = tenant_id
        self.project_id = project_id
        # Local-only ledger: protect check-and-consume with one lock.
        self._lock = threading.Lock()
        self._pending = {}
        self._revoked = set()

    def _identity(self, connection):
        try:
            identity = self._authenticate(connection)
        except Exception:
            return None
        if (not isinstance(identity, AuthenticatedSession)
                or not identity.is_well_formed()
                or identity.tenant_id != self.tenant_id
                or identity.project_id != self.project_id):
            return None
        return identity

    def _permitted(self, identity, task_id, purpose):
        try:
            return (isinstance(task_id, str) and bool(task_id.strip())
                    and isinstance(purpose, str) and bool(purpose.strip())
                    and self._authorize(identity, task_id, purpose) is True)
        except Exception:
            return False

    def issue(self, connection, task_id: str, purpose: str):
        """Only trusted host-resolved sessions may receive read-only proof."""
        identity = self._identity(connection)
        if identity is None or not self._permitted(identity, task_id, purpose):
            return None
        with self._lock:
            if identity.session_id in self._revoked:
                return None
            request = self._authority.issue_request(
                principal_id=identity.principal_id,
                tenant_id=identity.tenant_id,
                project_id=identity.project_id,
                task_id=task_id, purpose=purpose,
                expires_at=int(time.time()) + 60,
            )
            # Unique signed nonce prevents same-second issuance collisions.
            self._pending[request.nonce] = (request.signature, identity)
            return request

    def revoke_session(self, session_id: str):
        """Deny all future uses of an existing host session in this gateway."""
        if not isinstance(session_id, str) or not session_id:
            raise ValueError("session_id must be a non-empty string")
        with self._lock:
            self._revoked.add(session_id)
            for nonce, (_, identity) in tuple(self._pending.items()):
                if identity.session_id == session_id:
                    del self._pending[nonce]

    def read(self, connection, request: TrustedAdmissionRequest, dags):
        """Atomically consume once, revalidate policy, then read scoped DAGs."""
        result = {
            "admitted_records": [], "rejected_count": len(dags),
            "reason": "Host admission rejected (failed closed)",
        }
        identity = self._identity(connection)
        if (identity is None or not self._authority.verify_request(request)
                or request.tenant_id != self.tenant_id
                or request.project_id != self.project_id
                or not self._permitted(identity, request.task_id, request.purpose)):
            return result

        with self._lock:
            if identity.session_id in self._revoked:
                return result
            stored = self._pending.get(request.nonce)
            if (stored is None or stored[1] != identity
                    or not hmac.compare_digest(stored[0], request.signature)):
                return result
            # Invalidate before any processing, even when DAG reduction fails.
            del self._pending[request.nonce]

        return self._adapter.tool_get_context_view(
            request.task_id, dags, request=request,
        )
