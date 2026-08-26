import unittest
import sys
import time

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from ecm.task_engine import RoleLeaseManager, ECMTaskError

class TestRoleLeaseManager(unittest.TestCase):
    def setUp(self):
        self.manager = RoleLeaseManager()

    def test_lease_acquisition_and_conflict(self):
        lease_1 = self.manager.acquire_lease("task-1", "CODER", "agent-alice", duration_seconds=60)
        self.assertTrue(self.manager.is_lease_valid(lease_1))

        # Another agent trying to acquire the same role for same task should be rejected
        with self.assertRaises(ECMTaskError):
            self.manager.acquire_lease("task-1", "CODER", "agent-bob", duration_seconds=60)

        # Same agent can renew/extend without error
        lease_1_renew = self.manager.acquire_lease("task-1", "CODER", "agent-alice", duration_seconds=120)
        self.assertTrue(self.manager.is_lease_valid(lease_1_renew))

    def test_lease_release(self):
        lease_id = self.manager.acquire_lease("task-2", "REVIEWER", "agent-reviewer", duration_seconds=60)
        self.manager.release_lease(lease_id)
        self.assertFalse(self.manager.is_lease_valid(lease_id))

if __name__ == '__main__':
    unittest.main()
