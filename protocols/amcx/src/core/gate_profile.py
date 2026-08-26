from dataclasses import dataclass
from typing import Callable, Dict, List, Optional, Set


class GateProfileError(Exception):
    pass


@dataclass(frozen=True)
class EvidenceReceiptVerdict:
    receipt_id: str
    evidence_kind: str
    candidate_id: str
    profile_id: str
    verified: bool
    current: bool


ReceiptVerifier = Callable[
    [str, str, str, str],
    Optional[EvidenceReceiptVerdict],
]


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
        candidate_id: str,
        candidate_producer_id: str,
        required_evidence_kinds: List[str],
        receipt_verifier: ReceiptVerifier,
        min_reviewer_quorum: int = 2
    ):
        if not isinstance(candidate_id, str) or not candidate_id.strip():
            raise GateProfileError("candidate_id must be a non-blank identity")
        if not isinstance(candidate_producer_id, str) or not candidate_producer_id.strip():
            raise GateProfileError("candidate_producer_id must be a non-blank identity")
        if not callable(receipt_verifier):
            raise GateProfileError("receipt_verifier must be an authoritative read-only verifier")

        self.profile_id = profile_id
        self.subject_kind = subject_kind
        self._candidate_id = candidate_id
        self._candidate_producer_id = candidate_producer_id
        self._receipt_verifier = receipt_verifier
        self.required_evidence_kinds = required_evidence_kinds
        self.min_reviewer_quorum = min_reviewer_quorum
        self.candidate_state = "DRAFT"
        self.evidence_receipts: Dict[str, List[str]] = {}
        self.reviewer_approvals: Set[str] = set()

    @property
    def candidate_id(self) -> str:
        return self._candidate_id

    @property
    def candidate_producer_id(self) -> str:
        return self._candidate_producer_id

    def _verify_receipt(self, evidence_kind: str, receipt_id: str) -> EvidenceReceiptVerdict:
        try:
            verdict = self._receipt_verifier(
                receipt_id,
                evidence_kind,
                self._candidate_id,
                self.profile_id,
            )
        except Exception as exc:
            raise GateProfileError("Authoritative evidence verifier failed closed") from exc

        if not isinstance(verdict, EvidenceReceiptVerdict):
            raise GateProfileError("Evidence receipt is not authoritatively verified")
        if (
            verdict.receipt_id != receipt_id
            or verdict.evidence_kind != evidence_kind
            or verdict.candidate_id != self._candidate_id
            or verdict.profile_id != self.profile_id
            or not verdict.verified
            or not verdict.current
        ):
            raise GateProfileError(
                "Evidence receipt is unverified, stale, or not bound to this candidate and profile"
            )
        return verdict

    def submit_for_evaluation(self) -> None:
        if self.candidate_state != "DRAFT":
            raise GateProfileError(f"Cannot evaluate candidate in state {self.candidate_state}")
        self.candidate_state = "EVALUATING"

    def attach_evidence(self, evidence_kind: str, receipt_id: str) -> None:
        if self.candidate_state != "EVALUATING":
            raise GateProfileError(
                f"Cannot attach evidence while candidate is in state {self.candidate_state}"
            )
        if evidence_kind not in self.required_evidence_kinds:
            raise GateProfileError(f"Evidence kind '{evidence_kind}' not required by profile.")
        if not isinstance(receipt_id, str) or not receipt_id.strip():
            raise GateProfileError("Evidence receipt_id must be non-blank")

        self._verify_receipt(evidence_kind, receipt_id)
        self.evidence_receipts.setdefault(evidence_kind, []).append(receipt_id)

    def add_reviewer_approval(self, reviewer_id: str) -> None:
        if reviewer_id == self._candidate_producer_id:
            raise GateProfileError("Candidate producer cannot approve their own candidate")
        self.reviewer_approvals.add(reviewer_id)

    def promote_to_canary(self) -> None:
        if self.candidate_state != "EVALUATING":
            raise GateProfileError(f"Cannot promote to canary from {self.candidate_state}")
        if len(self.reviewer_approvals) < self.min_reviewer_quorum:
            raise GateProfileError(
                f"Quorum failure: {len(self.reviewer_approvals)} approvals, minimum {self.min_reviewer_quorum} required."
            )

        missing_kinds = []
        for evidence_kind in self.required_evidence_kinds:
            receipt_ids = self.evidence_receipts.get(evidence_kind, [])
            has_current_verified_receipt = False
            for receipt_id in receipt_ids:
                try:
                    self._verify_receipt(evidence_kind, receipt_id)
                except GateProfileError:
                    continue
                has_current_verified_receipt = True
                break
            if not has_current_verified_receipt:
                missing_kinds.append(evidence_kind)

        if missing_kinds:
            raise GateProfileError(
                f"Missing current verified evidence receipts for: {', '.join(missing_kinds)}."
            )
        self.candidate_state = "CANARY"

    def promote_to_production(self) -> None:
        if self.candidate_state != "CANARY":
            raise GateProfileError("Cannot promote to production without passing CANARY stage.")
        self.candidate_state = "PROMOTED"

    def rollback(self, reason: str) -> None:
        self.candidate_state = "ROLLED_BACK"
