import unittest
import sys

from dataclasses import replace
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

sys.path.insert(0, str(SRC_DIR))
from core.gate_profile import EvidenceReceiptVerdict, GateProfileEngine, GateProfileError


class TestGateProfileEngine(unittest.TestCase):
    def setUp(self):
        self.receipts = {
            "rcpt-001": EvidenceReceiptVerdict(
                receipt_id="rcpt-001",
                evidence_kind="UNIT_TEST_RECEIPT",
                candidate_id="candidate-001",
                profile_id="gate-skill-prompts-v1",
                verified=True,
                current=True,
            ),
            "rcpt-002": EvidenceReceiptVerdict(
                receipt_id="rcpt-002",
                evidence_kind="BENCHMARK_RESULT",
                candidate_id="candidate-001",
                profile_id="gate-skill-prompts-v1",
                verified=True,
                current=True,
            ),
            "rcpt-003": EvidenceReceiptVerdict(
                receipt_id="rcpt-003",
                evidence_kind="BENCHMARK_RESULT",
                candidate_id="candidate-001",
                profile_id="gate-skill-prompts-v1",
                verified=True,
                current=True,
            ),
        }

        def verifier(receipt_id, evidence_kind, candidate_id, profile_id):
            return self.receipts.get(receipt_id)

        self.gate = GateProfileEngine(
            profile_id="gate-skill-prompts-v1",
            subject_kind="SKILL",
            candidate_id="candidate-001",
            candidate_producer_id="producer-agent",
            required_evidence_kinds=["UNIT_TEST_RECEIPT", "BENCHMARK_RESULT"],
            receipt_verifier=verifier,
            min_reviewer_quorum=2
        )

    def test_gate_progression_lifecycle(self):
        self.assertEqual(self.gate.candidate_state, "DRAFT")
        self.gate.submit_for_evaluation()
        self.assertEqual(self.gate.candidate_state, "EVALUATING")

        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")
        self.gate.attach_evidence("BENCHMARK_RESULT", "rcpt-002")

        self.gate.add_reviewer_approval("reviewer-alice")
        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        self.gate.add_reviewer_approval("reviewer-bob")
        self.gate.promote_to_canary()
        self.assertEqual(self.gate.candidate_state, "CANARY")

        self.gate.promote_to_production()
        self.assertEqual(self.gate.candidate_state, "PROMOTED")

    def test_candidate_producer_cannot_approve_own_candidate(self):
        with self.assertRaises(GateProfileError):
            self.gate.add_reviewer_approval("producer-agent")
        self.assertEqual(self.gate.reviewer_approvals, set())

    def test_evidence_attachment_requires_evaluating_state(self):
        with self.assertRaises(GateProfileError):
            self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")

    def test_fabricated_receipt_is_rejected(self):
        self.gate.submit_for_evaluation()
        with self.assertRaises(GateProfileError):
            self.gate.attach_evidence("UNIT_TEST_RECEIPT", "fabricated-receipt")

    def test_receipt_bound_to_other_candidate_is_rejected(self):
        self.receipts["rcpt-other"] = EvidenceReceiptVerdict(
            receipt_id="rcpt-other",
            evidence_kind="UNIT_TEST_RECEIPT",
            candidate_id="candidate-other",
            profile_id="gate-skill-prompts-v1",
            verified=True,
            current=True,
        )
        self.gate.submit_for_evaluation()
        with self.assertRaises(GateProfileError):
            self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-other")

    def test_receipt_must_still_be_current_at_promotion(self):
        self.gate.submit_for_evaluation()
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")
        self.gate.attach_evidence("BENCHMARK_RESULT", "rcpt-002")
        self.gate.add_reviewer_approval("reviewer-alice")
        self.gate.add_reviewer_approval("reviewer-bob")
        self.receipts["rcpt-002"] = replace(self.receipts["rcpt-002"], current=False)

        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

    def test_duplicate_evidence_kind_does_not_satisfy_multi_kind_requirement(self):
        self.receipts["rcpt-004"] = EvidenceReceiptVerdict(
            receipt_id="rcpt-004",
            evidence_kind="UNIT_TEST_RECEIPT",
            candidate_id="candidate-001",
            profile_id="gate-skill-prompts-v1",
            verified=True,
            current=True,
        )
        self.gate.submit_for_evaluation()
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-001")
        self.gate.attach_evidence("UNIT_TEST_RECEIPT", "rcpt-004")
        self.gate.add_reviewer_approval("reviewer-alice")
        self.gate.add_reviewer_approval("reviewer-bob")

        with self.assertRaises(GateProfileError):
            self.gate.promote_to_canary()

        self.gate.attach_evidence("BENCHMARK_RESULT", "rcpt-003")
        self.gate.promote_to_canary()
        self.assertEqual(self.gate.candidate_state, "CANARY")

    def test_unrequired_evidence_kind_is_rejected(self):
        self.gate.submit_for_evaluation()
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
