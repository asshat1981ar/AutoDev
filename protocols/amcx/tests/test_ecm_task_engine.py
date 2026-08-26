import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from ecm.task_engine import ECMTask, ECMTaskError

class TestECMTaskEngine(unittest.TestCase):
    def setUp(self):
        self.task = ECMTask(
            task_id="task-100",
            title="Implement Core Router",
            acceptance_contract_id="contract-v1",
            budget_limit_tokens=5000,
            initial_state="PROPOSED",
            assigned_role="CODER"
        )

    def test_full_lifecycle_progression(self):
        self.assertEqual(self.task.task_state, "PROPOSED")
        self.task.transition("READY", "Spec approved")
        self.assertEqual(self.task.task_state, "READY")
        self.task.transition("CLAIMED", "Agent lease active")
        self.task.transition("RUNNING", "Execution started")
        self.task.transition("REVIEW_PENDING", "Code submitted for review")
        self.task.transition("ACCEPTANCE_PENDING", "Review passed")
        self.task.transition("COMPLETED", "Acceptance tests passed")
        self.assertEqual(self.task.task_state, "COMPLETED")
        self.assertEqual(len(self.task.history), 7)

    def test_illegal_transition_fails(self):
        # Cannot jump from PROPOSED directly to COMPLETED
        with self.assertRaises(ECMTaskError):
            self.task.transition("COMPLETED", "Bypassing checks")

    def test_token_budget_enforcement(self):
        self.task.consume_tokens(2000)
        self.assertEqual(self.task.tokens_used, 2000)
        self.task.consume_tokens(3000)
        self.assertEqual(self.task.tokens_used, 5000)
        # Exceeding budget should raise error
        with self.assertRaises(ECMTaskError):
            self.task.consume_tokens(1)

if __name__ == '__main__':
    unittest.main()
