import unittest
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.gate_profile import GateProfileEngine, GateProfileError

class TestGateProfileEngine(unittest.TestCase):
    def setUp(self):
        self.gate = GateProfileEngine(
            profile_id="gate-skill-prompts-v1",
            subject_kind="SKILL",
            candidate_producer_id="producer-agent",
            required_evidence_kinds=["UNIT_TEST_RECEIPT", "BENCHMARK_RESULT"],
            min_reviewer_quorum=2
        )

    def test_gate_progression_lifecycle(self):
        self.assertEqual(self.gate.candidate_state, "DRAFT")
        self.gate.submit_for_evaluation()
        self.assertEqual(self.gate.candidate_state, "EVALUATING")

        # Premature promotion should fail (no evidence, no quorum)
        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        # Attach evidence
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")
        self.gate.attach_evidence("BENCHMARK_RESULT", "rcpt-002")

        # 1 reviewer approval (quorum requires 2)
        self.gate.add_reviewer_approval("reviewer-alice")
        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        # 2nd reviewer approval reaches quorum
        self.gate.add_reviewer_approval("reviewer-bob")
        self.gate.promote_to_canary()
        self.assertEqual(self.gate.candidate_state, "CANARY")

        # Promote to production
        self.gate.promote_to_production()
        self.assertEqual(self.gate.candidate_state, "PROMOTED")

    def test_candidate_producer_cannot_approve_own_candidate(self):
        with self.assertRaises(GateProfileError):
            self.gate.add_reviewer_approval("producer-agent")
        self.assertEqual(self.gate.reviewer_approvals, set())

    def test_duplicate_evidence_kind_does_not_satisfy_multi_kind_requirement(self):
        self.gate.submit_for_evaluation()
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-002")
        self.gate.add_reviewer_approval("reviewer-alice")
        self.gate.add_reviewer_approval("reviewer-bob")

        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        self.gate.attach_evidence("BENCHMARK_RESULT", "rcpt-003")
        self.gate.promote_to_canary()
        self.assertEqual(self.gate.candidate_state, "CANARY")

    def test_unrequired_evidence_kind_is_rejected(self):
        with self.assertRaises(GateProfileError):
            self.gate.attach_evidence("VIBE_CHECK", "rcpt-999")

    def test_cannot_skip_canary_stage(self):
        with self.assertRaises(GateProfileError):
            self.gate.promote_to_production()

        self.gate.submit_for_evaluation()
        with self.assertRaises(GateProfileError):
            self.gate.promote_to_production()

    def test_rollback(self):
        self.gate.submit_for_evaluation()
        self.gate.rollback("Degradation detected in benchmark")
        self.assertEqual(self.gate.candidate_state, "ROLLED_BACK")

if __name__ == '__main__':
    unittest.main()
