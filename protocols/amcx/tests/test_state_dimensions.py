import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.state_dimensions import OrthogonalStateTracker, StateDimensionError

class TestStateDimensions(unittest.TestCase):
    def setUp(self):
        self.tracker = OrthogonalStateTracker()

    def test_initial_defaults(self):
        self.assertEqual(self.tracker.get_state("content_lifecycle"), "PROPOSED")
        self.assertEqual(self.tracker.get_state("admission"), "UNASSESSED")
        self.assertEqual(self.tracker.get_state("validity"), "VALID")
        self.assertEqual(self.tracker.get_state("runtime_sharing"), "PRIVATE_ATTEMPT")

    def test_independent_transition(self):
        # Mutating admission does not touch content_lifecycle
        self.tracker.transition("admission", "QUARANTINED")
        self.assertEqual(self.tracker.get_state("admission"), "QUARANTINED")
        self.assertEqual(self.tracker.get_state("content_lifecycle"), "PROPOSED")

    def test_effective_readability_closed_gates(self):
        # Normal admitted item with authorization is readable
        self.tracker.transition("admission", "ADMITTED")
        self.assertTrue(self.tracker.compute_effective_readability(is_authorized=True, within_purpose=True))

        # Quarantined item fails closed
        self.tracker.transition("admission", "QUARANTINED")
        self.assertFalse(self.tracker.compute_effective_readability(is_authorized=True, within_purpose=True))

        # Retracted item fails closed
        self.tracker.transition("admission", "ADMITTED")
        self.tracker.transition("content_lifecycle", "RETRACTED")
        self.assertFalse(self.tracker.compute_effective_readability(is_authorized=True, within_purpose=True))

if __name__ == '__main__':
    unittest.main()
