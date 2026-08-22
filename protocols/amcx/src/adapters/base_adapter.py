import re
from typing import Dict, Any, List

class SecretLeakDetectedError(Exception):
    pass

class BaseAdapter:
    """
    Neutral adapter interface enforcing input/output validation,
    fail-closed security, and zero raw secrets invariant (AMCX-R-0006).
    """
    SECRET_PATTERNS = [
        re.compile(r"(?i)(bearer\s+[a-z0-9_\-\.]{20,}|ghp_[a-zA-Z0-9]{36}|sk-[a-zA-Z0-9]{20,})"),
        re.compile(r"(?i)(api[_-]?key|password|secret)\s*[:=]\s*['\"][a-zA-Z0-9_\-\.]{8,}['\"]")
    ]

    @classmethod
    def sanitize_payload(cls, payload: Dict[str, Any]) -> Dict[str, Any]:
        serialized = str(payload)
        for pattern in cls.SECRET_PATTERNS:
            if pattern.search(serialized):
                raise SecretLeakDetectedError("Raw secret or bearer credential detected in adapter payload! Refusing operation.")
        return payload
