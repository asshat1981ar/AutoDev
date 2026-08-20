import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.amx_dag import AMXEventDAG, AMXEvent
from core.amx_bundle import AMXMemoryBundle, BundleError

class TestAMXBundle(unittest.TestCase):
    def setUp(self):
        self.record_id = "b" * 64
        self.dag = AMXEventDAG(self.record_id)
        e0 = AMXEvent("e0" + "0"*62, "RECORD_CREATED", self.record_id, [], "lead-coder", {
            "title": "Bundle Test Doc",
            "content": "Payload inside export bundle"
        })
        self.dag.append_event(e0)

    def test_export_and_import_integrity(self):
        bundle = AMXMemoryBundle.export_dag("bundle-001", self.dag)
        b_dict = bundle.to_dict()

        self.assertEqual(b_dict["bundle_id"], "bundle-001")
        self.assertEqual(len(b_dict["records"]), 1)
        self.assertEqual(len(b_dict["events"]), 1)

        imported = AMXMemoryBundle.import_and_verify(b_dict)
        self.assertEqual(imported.bundle_id, "bundle-001")
        self.assertEqual(imported.bundle_digest, bundle.bundle_digest)

    def test_tampered_bundle_fails_verification(self):
        bundle = AMXMemoryBundle.export_dag("bundle-002", self.dag)
        b_dict = bundle.to_dict()

        # Tamper payload
        b_dict["records"][0]["title"] = "HACKED TITLE"

        with self.assertRaises(BundleError):
            AMXMemoryBundle.import_and_verify(b_dict)

if __name__ == '__main__':
    unittest.main()
