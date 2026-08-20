import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from ecm.task_engine import MessageRouter, ECMTaskError

class TestMessageRouter(unittest.TestCase):
    def setUp(self):
        self.router = MessageRouter()

    def test_valid_message_dispatch(self):
        msg = self.router.dispatch_message(
            task_id="task-01",
            sender_role="SUPERVISOR",
            message_kind="PROPOSAL",
            content="Task decomposed into 3 sub-units."
        )
        self.assertEqual(msg["sender_role"], "SUPERVISOR")
        self.assertEqual(msg["message_kind"], "PROPOSAL")

        messages = self.router.get_task_messages("task-01")
        self.assertEqual(len(messages), 1)

    def test_invalid_role_rejected(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message("task-01", "UNAUTHORIZED_BOT", "PROPOSAL", "Hello")

    def test_invalid_kind_rejected(self):
        with self.assertRaises(ECMTaskError):
            self.router.dispatch_message("task-01", "CODER", "UNRECOGNIZED_KIND", "Hello")

if __name__ == '__main__':
    unittest.main()
