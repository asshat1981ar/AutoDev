"""Receiver-owned, digest-bound context-admission attestations.

These bindings describe *read eligibility*, never execution authority.
Only a trusted host should construct and inject a ScopeBindingAuthority.
Do not expose issuance or its key through MCP/model-facing tool methods.
"""
from dataclasses import dataclass
import hashlib
import hmac
import json
from typing import Iterable, Tuple


@dataclass(frozen=True)
class ScopeBinding:
    record_id: str
    dag_digest: str
    tenant_id: str
    project_id: str
    allowed_purposes: Tuple[str, ...]
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
