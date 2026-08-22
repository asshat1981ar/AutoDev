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

    def _recompute_bundle_digest(self, bundle_dict):
        forged = AMXMemoryBundle(
            bundle_id=bundle_dict["bundle_id"],
            records=bundle_dict["records"],
            events=bundle_dict["events"],
            causal_roots=bundle_dict["causal_roots"],
            causal_heads=bundle_dict["causal_heads"],
            exported_at=bundle_dict["exported_at"],
        )
        bundle_dict["bundle_digest"] = forged.bundle_digest
        return forged.bundle_digest

    def test_export_and_import_integrity(self):
        bundle = AMXMemoryBundle.export_dag("bundle-001", self.dag)
        b_dict = bundle.to_dict()

        self.assertEqual(b_dict["bundle_id"], "bundle-001")
        self.assertEqual(len(b_dict["records"]), 1)
        self.assertEqual(len(b_dict["events"]), 1)

        imported = AMXMemoryBundle.import_and_verify(b_dict, bundle.bundle_digest)
        self.assertEqual(imported.bundle_id, "bundle-001")
        self.assertEqual(imported.bundle_digest, bundle.bundle_digest)

    def test_tampered_bundle_fails_verification(self):
        bundle = AMXMemoryBundle.export_dag("bundle-002", self.dag)
        b_dict = bundle.to_dict()

        b_dict["records"][0]["title"] = "HACKED TITLE"

        with self.assertRaises(BundleError):
            AMXMemoryBundle.import_and_verify(b_dict, bundle.bundle_digest)

    def test_wrong_schema_version_fails_closed(self):
        bundle = AMXMemoryBundle.export_dag("bundle-003", self.dag)
        b_dict = bundle.to_dict()
        b_dict["schema_version"] = "amx.bundle.v999"

        with self.assertRaises(BundleError):
            AMXMemoryBundle.import_and_verify(b_dict, bundle.bundle_digest)

    def test_self_consistent_forged_record_fails_semantic_verification(self):
        bundle = AMXMemoryBundle.export_dag("bundle-004", self.dag)
        b_dict = bundle.to_dict()
        b_dict["records"][0]["title"] = "FORGED BUT REHASHED"
        forged_digest = self._recompute_bundle_digest(b_dict)

        with self.assertRaises(BundleError):
            AMXMemoryBundle.import_and_verify(b_dict, forged_digest)

    def test_rehashed_causal_root_tampering_fails_semantic_verification(self):
        bundle = AMXMemoryBundle.export_dag("bundle-005", self.dag)
        b_dict = bundle.to_dict()
        b_dict["causal_roots"] = ["f" * 64]
        forged_digest = self._recompute_bundle_digest(b_dict)

        with self.assertRaises(BundleError):
            AMXMemoryBundle.import_and_verify(b_dict, forged_digest)


if __name__ == '__main__':
    unittest.main()
