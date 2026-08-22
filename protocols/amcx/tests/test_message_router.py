import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from ecm.task_engine import MessageRouter, RoleLeaseManager, ECMTaskError


class TestMessageRouter(unittest.TestCase):
    def setUp(self):
        self.leases = RoleLeaseManager()
        self.router = MessageRouter(self.leases)
        self.lease_id = self.leases.acquire_lease(
            task_id="task-01",
            role="SUPERVISOR",
            agent_id="agent-alice",
        )

    def test_valid_message_dispatch(self):
        msg = self.router.dispatch_message(
            task_id="task-01",
            sender_role="SUPERVISOR",
            message_kind="PROPOSAL",
            content="Task decomposed into 3 sub-units.",
            agent_id="agent-alice",
            lease_id=self.lease_id,
        )
        self.assertEqual(msg["sender_role"], "SUPERVISOR")
        self.assertEqual(msg["message_kind"], "PROPOSAL")
        self.assertNotIn("agent_id", msg)
        self.assertNotIn("lease_id", msg)

        messages = self.router.get_task_messages("task-01")
        self.assertEqual(len(messages), 1)

    def test_wrong_agent_cannot_use_role_lease(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message(
                "task-01",
                "SUPERVISOR",
                "PROPOSAL",
                "Spoofed sender",
                "agent-mallory",
                self.lease_id,
            )
        self.assertEqual(self.router.get_task_messages("task-01"), [])

    def test_wrong_role_cannot_use_role_lease(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message(
                "task-01",
                "CODER",
                "PROPOSAL",
                "Role escalation",
                "agent-alice",
                self.lease_id,
            )

    def test_wrong_task_cannot_use_role_lease(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message(
                "task-02",
                "SUPERVISOR",
                "PROPOSAL",
                "Cross-task replay",
                "agent-alice",
                self.lease_id,
            )

    def test_invalid_role_rejected(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message(
                "task-01",
                "UNAUTHORIZED_BOT",
                "PROPOSAL",
                "Hello",
                "agent-alice",
                self.lease_id,
            )

    def test_invalid_kind_rejected(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message(
                "task-01",
                "SUPERVISOR",
                "UNRECOGNIZED_KIND",
                "Hello",
                "agent-alice",
                self.lease_id,
            )


if __name__ == '__main__':
    unittest.main()
