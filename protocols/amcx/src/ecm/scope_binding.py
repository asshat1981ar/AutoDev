"""Receiver-owned, digest-bound context-admission attestations.

These bindings describe *read eligibility*, never execution authority.
Only a trusted host should construct and inject a ScopeBindingAuthority.
Do not expose issuance or its key through MCP/model-facing tool methods.
"""
from dataclasses import dataclass
import hashlib
import hmac
import json
import secrets
import time
from typing import Iterable, Tuple


@dataclass(frozen=True)
class ScopeBinding:
    record_id: str
    dag_digest: str
    tenant_id: str
    project_id: str
    allowed_purposes: Tuple[str, ...]
    signature: str


@dataclass(frozen=True)
class TrustedAdmissionRequest:
    principal_id: str
    tenant_id: str
    project_id: str
    task_id: str
    purpose: str
    expires_at: int
    nonce: str
    signature: str


class ScopeBindingAuthority:
    """Host-controlled HMAC issuer/verifier for one trusted receiver scope."""

    def __init__(self, secret: bytes):
        if not isinstance(secret, bytes) or len(secret) < 32:
            raise ValueError("scope authority needs at least 32 unpredictable secret bytes")
        self._secret = secret

    @staticmethod
    def _message(record_id, dag_digest, tenant_id, project_id, allowed_purposes):
        fields = (record_id, dag_digest, tenant_id, project_id)
        if any(not isinstance(v, str) or not v.strip() for v in fields):
            raise ValueError("binding identity fields must be non-blank strings")
        if any(len(v) != 64 or any(c not in "0123456789abcdef" for c in v)
               for v in (record_id, dag_digest)):
            raise ValueError("binding record ID and digest must be lowercase SHA-256 hex")
        if (not isinstance(allowed_purposes, tuple) or not allowed_purposes
                or any(not isinstance(v, str) or not v.strip() for v in allowed_purposes)
                or tuple(sorted(set(allowed_purposes))) != allowed_purposes):
            raise ValueError("allowed purposes must be a sorted non-empty tuple of unique strings")
        return json.dumps(
            [record_id, dag_digest, tenant_id, project_id, allowed_purposes],
            separators=(",", ":"), ensure_ascii=True,
        ).encode("utf-8")

    def issue(self, *, record_id: str, dag_digest: str, tenant_id: str,
              project_id: str, allowed_purposes: Iterable[str]) -> ScopeBinding:
        purposes = tuple(sorted(set(allowed_purposes)))
        message = self._message(record_id, dag_digest, tenant_id, project_id, purposes)
        signature = hmac.new(self._secret, message, hashlib.sha256).hexdigest()
        return ScopeBinding(record_id, dag_digest, tenant_id, project_id, purposes, signature)

    def verify(self, binding: ScopeBinding) -> bool:
        if not isinstance(binding, ScopeBinding):
            return False
        try:
            message = self._message(
                binding.record_id, binding.dag_digest, binding.tenant_id,
                binding.project_id, binding.allowed_purposes,
            )
            return (isinstance(binding.signature, str)
                    and hmac.compare_digest(
                        hmac.new(self._secret, message, hashlib.sha256).hexdigest(),
                        binding.signature,
                    ))
        except (ValueError, TypeError):
            return False

    @staticmethod
    def _request_message(principal_id, tenant_id, project_id, task_id, purpose, expires_at, nonce):
        if any(not isinstance(v, str) or not v.strip()
               for v in (principal_id, tenant_id, project_id, task_id, purpose)):
            raise ValueError("trusted request fields must be nonblank strings")
        if type(expires_at) is not int or expires_at <= 0:
            raise ValueError("trusted request expiry must be positive unix seconds")
        if not isinstance(nonce, str) or len(nonce) != 32 or any(c not in "0123456789abcdef" for c in nonce):
            raise ValueError("nonce must be 128-bit lowercase hexadecimal")
        return json.dumps(
            ["AMCX-TRUSTED-REQUEST-v2", principal_id, tenant_id, project_id,
             task_id, purpose, expires_at, nonce],
            separators=(",", ":"), ensure_ascii=True,
        ).encode("utf-8")

    def issue_request(self, *, principal_id: str, tenant_id: str,
                      project_id: str, task_id: str, purpose: str,
                      expires_at=None) -> TrustedAdmissionRequest:
        """Trusted host only: after authenticating principal and checking policy."""
        if expires_at is None:
            expires_at = int(time.time()) + 300
        nonce = secrets.token_hex(16)
        msg = self._request_message(
            principal_id, tenant_id, project_id, task_id, purpose, expires_at, nonce
        )
        sig = hmac.new(self._secret, msg, hashlib.sha256).hexdigest()
        return TrustedAdmissionRequest(principal_id, tenant_id, project_id,
                                       task_id, purpose, expires_at, nonce, sig)

    def verify_request(self, request: TrustedAdmissionRequest) -> bool:
        if not isinstance(request, TrustedAdmissionRequest):
            return False
        try:
            msg = self._request_message(
                request.principal_id, request.tenant_id, request.project_id,
                request.task_id, request.purpose, request.expires_at, request.nonce
            )
            return (request.expires_at > int(time.time())
                    and isinstance(request.signature, str)
                    and hmac.compare_digest(
                        hmac.new(self._secret, msg, hashlib.sha256).hexdigest(),
                        request.signature,
                    ))
        except (TypeError, ValueError):
            return False
