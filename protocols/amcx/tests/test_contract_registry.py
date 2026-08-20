import unittest
import os
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from registry.contract_registry import NeutralContractRegistry, ContractRegistryError

class TestNeutralContractRegistry(unittest.TestCase):
    def setUp(self):
        self.registry_dir = str(REPO_ROOT / 'registry')
        self.registry = NeutralContractRegistry(self.registry_dir)

    def test_all_12_schemas_loaded(self):
        active_schemas = self.registry.list_active_schemas()
        self.assertEqual(len(active_schemas), 12)
        expected_schemas = [
            "amx.record.v1.json", "amx.event.v1.json", "amx.bundle.v1.json", "amx.transition.v1.json",
            "ecm.task.v1.json", "ecm.attempt.v1.json", "ecm.message.v1.json", "ecm.lease.v1.json",
            "ecm.memory_binding.v1.json", "amcx.state_dimensions.v1.json", "amcx.trust_boundaries.v1.json",
            "amcx.gate_profile.v1.json"
        ]
        for s in expected_schemas:
            self.assertIn(s, active_schemas)
            schema = self.registry.get_schema(s)
            self.assertIsNotNone(schema)

    def test_unknown_schema_fails_closed(self):
        with self.assertRaises(ContractRegistryError):
            self.registry.get_schema("malicious.schema.v999.json")

    def test_schema_activation_authority_boundary(self):
        # Runtime agents CANNOT activate schemas
        self.assertFalse(self.registry.validate_schema_activation_authority("RUNTIME_AGENT"))
        self.assertFalse(self.registry.validate_schema_activation_authority("CODER_AGENT"))
        self.assertFalse(self.registry.validate_schema_activation_authority("ORCHESTRATOR"))
        # Only reviewed repository ADR / maintainer process can activate
        self.assertTrue(self.registry.validate_schema_activation_authority("MAINTAINER_ADR_REVIEW"))
        self.assertTrue(self.registry.validate_schema_activation_authority("REPOSITORY_STEWARD"))

if __name__ == '__main__':
    unittest.main()
