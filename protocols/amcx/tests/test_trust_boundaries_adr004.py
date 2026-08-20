import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.trust_boundaries import DomainOwnershipMap, TrustBoundaryViolation

class TestTrustBoundariesADR004(unittest.TestCase):
    def test_exact_18_domains_present(self):
        self.assertEqual(len(DomainOwnershipMap.DOMAINS), 18)
        for i in range(1, 19):
            domain = DomainOwnershipMap.get_domain(i)
            self.assertIn("name", domain)
            self.assertIn("canonical_history", domain)
            self.assertIn("decision_authority", domain)
            self.assertIn("execution_materialization", domain)

    def test_specific_domain_identities(self):
        # Domain 1: Plan and step lifecycle
        d1 = DomainOwnershipMap.get_domain(1)
        self.assertEqual(d1["name"], "Plan and step lifecycle")
        self.assertEqual(d1["canonical_history"], "ExecPlan")

        # Domain 3: Portable memory
        d3 = DomainOwnershipMap.get_domain(3)
        self.assertEqual(d3["name"], "Portable memory")
        self.assertIn("AMX", d3["canonical_history"])

        # Domain 16: Contract activation
        d16 = DomainOwnershipMap.get_domain(16)
        self.assertEqual(d16["name"], "Contract activation")
        self.assertEqual(d16["canonical_history"], "Neutral Contract Registry")

    def test_authority_validation(self):
        self.assertTrue(DomainOwnershipMap.validate_action_authority(16, "Repository review/ADR and authorized maintainers"))
        self.assertFalse(DomainOwnershipMap.validate_action_authority(16, "Untrusted LLM Output"))
        self.assertFalse(DomainOwnershipMap.validate_action_authority(11, "Model Prompt Token"))

if __name__ == '__main__':
    unittest.main()
