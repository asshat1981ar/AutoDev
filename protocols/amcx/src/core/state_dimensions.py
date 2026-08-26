from typing import Dict, Any, Optional

class StateDimensionError(Exception):
    pass

class OrthogonalStateTracker:
    """
    Maintains and reduces independent orthogonal state dimensions for AMCX-1 entities.
    Enforces that transitions in one dimension do not implicitly cause transitions in another.
    """
    VALID_STATES = {
        "content_lifecycle": ["PROPOSED", "CURRENT", "SUPERSEDED", "RETRACTED"],
        "admission": ["UNASSESSED", "QUARANTINED", "ADMITTED", "REJECTED"],
        "validity": ["VALID", "STALE", "EXPIRED", "REVOKED"],
        "runtime_sharing": ["PRIVATE_ATTEMPT", "TASK", "PROJECT"],
        "repository_publication": ["NOT_PUBLISHED", "REVIEW_PENDING", "PUBLISHED", "PUBLICATION_RETRACTED"],
        "cross_project_export": ["NOT_APPROVED", "APPROVED", "EXPORTED", "REVOKED"],
        "collaboration_task": [
            "PROPOSED", "READY", "CLAIMED", "RUNNING", "BLOCKED",
            "REVIEW_PENDING", "ACCEPTANCE_PENDING", "RETRY_WAIT",
            "CANCEL_REQUESTED", "COMPLETED", "FAILED", "EXPIRED",
            "CANCELLED", "CANCELLED_WITH_EFFECT", "CANCELLED_EFFECT_UNKNOWN", "MANUAL_REQUIRED"
        ],
        "configuration_candidate": ["DRAFT", "EVALUATING", "CANARY", "PROMOTED", "ROLLED_BACK", "SUPERSEDED", "EXPIRED", "REJECTED"]
    }

    def __init__(self, initial_states: Optional[Dict[str, str]] = None):
        self._states: Dict[str, str] = {
            "content_lifecycle": "PROPOSED",
            "admission": "UNASSESSED",
            "validity": "VALID",
            "runtime_sharing": "PRIVATE_ATTEMPT",
            "repository_publication": "NOT_PUBLISHED",
            "cross_project_export": "NOT_APPROVED",
            "collaboration_task": "PROPOSED",
            "configuration_candidate": "DRAFT"
        }
        if initial_states:
            for k, v in initial_states.items():
                self.transition(k, v)

    def get_state(self, dimension: str) -> str:
        if dimension not in self._states:
            raise StateDimensionError(f"Unknown state dimension: {dimension}")
        return self._states[dimension]

    def transition(self, dimension: str, target_state: str) -> None:
        if dimension not in self.VALID_STATES:
            raise StateDimensionError(f"Invalid dimension {dimension}")
        if target_state not in self.VALID_STATES[dimension]:
            raise StateDimensionError(f"Invalid state {target_state} for dimension {dimension}")
        self._states[dimension] = target_state

    def compute_effective_readability(self, is_authorized: bool, within_purpose: bool) -> bool:
        # Effective readability = current intersection of declared visibility, validity, admission, authorization
        if not is_authorized or not within_purpose:
            return False
        if self._states["admission"] in ["QUARANTINED", "REJECTED"]:
            return False
        if self._states["validity"] in ["EXPIRED", "REVOKED"]:
            return False
        if self._states["content_lifecycle"] == "RETRACTED":
            return False
        return True
