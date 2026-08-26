import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from adapters.base_adapter import BaseAdapter, SecretLeakDetectedError
from core.memory_binding import ECMMemoryBinding

class TestFailClosedSecurity(unittest.TestCase):
    def test_raw_secret_leak_blocked(self):
        dirty_payload = {
            "title": "API config",
            "content": "Using bearer token sk-1234567890abcdef1234567890abcdef for auth"
        }
        with self.assertRaises(SecretLeakDetectedError):
            BaseAdapter.sanitize_payload(dirty_payload)

    def test_clean_payload_passes(self):
        clean_payload = {
            "title": "Design Doc",
            "content": "Using brokered secret reference ref://vault/tenant-1/api-key"
        }
        sanitized = BaseAdapter.sanitize_payload(clean_payload)
        self.assertEqual(sanitized["title"], "Design Doc")

    def test_ecm_memory_binding_noncanonical_invariant(self):
        binding = ECMMemoryBinding(
            binding_id="bind-001",
            task_id="task-42",
            amx_record_id="a" * 64,
            amx_record_digest="b" * 64,
            purpose="INPUT_CONTEXT"
        )
        b_dict = binding.to_dict()
        # Invariant 3: Binding MUST be explicitly non-canonical
        self.assertFalse(b_dict["is_canonical"])
        self.assertEqual(b_dict["binding_purpose"], "INPUT_CONTEXT")

if __name__ == '__main__':
    unittest.main()
