from typing import Dict, Any, List, Optional, Set

class GateProfileError(Exception):
    pass

class GateProfileEngine:
    """
    GateProfile & Configuration Promotion Engine (amcx.gate_profile.v1).
    Enforces independent reviewer quorum, required evidence receipts, and canary stages
    for prompts, skills, routers, and schema candidates (Domain #14 & #15).
    """
    def __init__(
        self,
        profile_id: str,
        subject_kind: str,
        required_evidence_kinds: List[str],
        min_reviewer_quorum: int = 2
    ):
        self.profile_id = profile_id
        self.subject_kind = subject_kind
        self.required_evidence_kinds = required_evidence_kinds
        self.min_reviewer_quorum = min_reviewer_quorum
        self.candidate_state = "DRAFT"
        self.evidence_receipts: Dict[str, List[str]] = {}
        self.reviewer_approvals: Set[str] = set()

    def submit_for_evaluation(self) -> None:
        if self.candidate_state != "DRAFT":
            raise GateProfileError(f"Cannot evaluate candidate in state {self.candidate_state}")
        self.candidate_state = "EVALUATING"

    def attach_evidence(self, evidence_kind: str, receipt_id: str) -> None:
        if evidence_kind not in self.required_evidence_kinds:
            raise GateProfileError(f"Evidence kind '{evidence_kind}' not required by profile.")
        self.evidence_receipts.setdefault(evidence_kind, []).append(receipt_id)

    def add_reviewer_approval(self, reviewer_id: str) -> None:
        self.reviewer_approvals.add(reviewer_id)

    def promote_to_canary(self) -> None:
        if self.candidate_state != "EVALUATING":
            raise GateProfileError(f"Cannot promote to canary from {self.candidate_state}")
        if len(self.reviewer_approvals) < self.min_reviewer_quorum:
            raise GateProfileError(
                f"Quorum failure: {len(self.reviewer_approvals)} approvals, minimum {self.min_reviewer_quorum} required."
            )
        missing_kinds = [
            evidence_kind
            for evidence_kind in self.required_evidence_kinds
            if not self.evidence_receipts.get(evidence_kind)
        ]
        if missing_kinds:
            raise GateProfileError(
                f"Missing required evidence receipts for: {', '.join(missing_kinds)}."
            )
        self.candidate_state = "CANARY"

    def promote_to_production(self) -> None:
        if self.candidate_state != "CANARY":
            raise GateProfileError(f"Cannot promote to production without passing CANARY stage.")
        self.candidate_state = "PROMOTED"

    def rollback(self, reason: str) -> None:
        self.candidate_state = "ROLLED_BACK"
