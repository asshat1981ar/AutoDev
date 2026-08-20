from typing import Dict, Any, List, Optional

class TrustBoundaryViolation(Exception):
    pass

class DomainOwnershipMap:
    """
    Enforces the 18 Canonical Domain Ownership Boundaries from AMCX-1 §5 (ADR-004).
    """
    DOMAINS = {
        1: {
            "name": "Plan and step lifecycle",
            "canonical_history": "ExecPlan",
            "decision_authority": "AutoDev plan reducer/policy",
            "execution_materialization": "AutoDev orchestrator"
        },
        2: {
            "name": "Collaboration",
            "canonical_history": "ECM event log",
            "decision_authority": "ECM reducer and role policy",
            "execution_materialization": "ECM orchestrator/adapters"
        },
        3: {
            "name": "Portable memory",
            "canonical_history": "AMX event DAG and bundles",
            "decision_authority": "AMX validates grammar only",
            "execution_materialization": "AMX store/projections"
        },
        4: {
            "name": "Origin/receiver identity",
            "canonical_history": "Attestation reference",
            "decision_authority": "Authenticated host/transport",
            "execution_materialization": "Identity/attestation store"
        },
        5: {
            "name": "Evidence verdict/freshness",
            "canonical_history": "EvidenceStore/VerificationFabric",
            "decision_authority": "Independent verifier",
            "execution_materialization": "Evidence store"
        },
        6: {
            "name": "Quarantine restriction",
            "canonical_history": "AMX event/state",
            "decision_authority": "Deterministic AMX restriction",
            "execution_materialization": "AMX reducer"
        },
        7: {
            "name": "Release/trust/visibility widening",
            "canonical_history": "AMX records result",
            "decision_authority": "External memory-governance policy",
            "execution_materialization": "AMX reducer/projections"
        },
        8: {
            "name": "Retraction suppression barriers",
            "canonical_history": "Memory Governance Ledger",
            "decision_authority": "External memory-governance policy",
            "execution_materialization": "Ledger plus AMX commit coordinator"
        },
        9: {
            "name": "Cross-project grant",
            "canonical_history": "Approval record",
            "decision_authority": "Scoped user/host approval",
            "execution_materialization": "Memory-governance service"
        },
        10: {
            "name": "Effective retrieval",
            "canonical_history": "Current decision",
            "decision_authority": "Host/ForgeCore policy intersected with AMX state",
            "execution_materialization": "Retrieval/context service"
        },
        11: {
            "name": "Effects and receipts",
            "canonical_history": "ForgeCore ledger",
            "decision_authority": "ForgeCore/host policy",
            "execution_materialization": "Trusted executor"
        },
        12: {
            "name": "ContextView history",
            "canonical_history": "ECM artifact/workflow",
            "decision_authority": "ECM admission plus current policy",
            "execution_materialization": "ECM context service/CAS"
        },
        13: {
            "name": "Hard purge",
            "canonical_history": "External deletion ledger",
            "decision_authority": "Authorized retention/privacy policy",
            "execution_materialization": "Deletion coordinator/adapters"
        },
        14: {
            "name": "Prompt/skill/router activation",
            "canonical_history": "ECM promotion log",
            "decision_authority": "Trusted deployment/approval authority structurally separate from content-producing agents",
            "execution_materialization": "Configuration deployment service"
        },
        15: {
            "name": "GateProfile publication/status",
            "canonical_history": "Reviewed Evaluation Policy Registry in Git",
            "decision_authority": "Authorized evaluation-policy maintainers, separate from candidate producers/evaluators",
            "execution_materialization": "Gate validators consume exact active digest"
        },
        16: {
            "name": "Contract activation",
            "canonical_history": "Neutral Contract Registry",
            "decision_authority": "Repository review/ADR and authorized maintainers",
            "execution_materialization": "Validators/adapters"
        },
        17: {
            "name": "Artifact bytes",
            "canonical_history": "CAS",
            "decision_authority": "Owning domain’s retention policy",
            "execution_materialization": "Artifact service"
        },
        18: {
            "name": "Aggregate budgets",
            "canonical_history": "ECM budget ledger",
            "decision_authority": "ECM orchestrator/policy",
            "execution_materialization": "Scheduler/adapters"
        }
    }

    @classmethod
    def get_domain(cls, domain_id: int) -> Dict[str, str]:
        if domain_id not in cls.DOMAINS:
            raise TrustBoundaryViolation(f"Invalid domain ID {domain_id}")
        return cls.DOMAINS[domain_id]

    @classmethod
    def get_domain_by_name(cls, name: str) -> Optional[Dict[str, Any]]:
        for d_id, d_info in cls.DOMAINS.items():
            if d_info["name"].lower() == name.lower():
                return {"domain_id": d_id, **d_info}
        return None

    @classmethod
    def validate_action_authority(cls, domain_id: int, claiming_authority: str) -> bool:
        domain = cls.get_domain(domain_id)
        # Verify claiming authority matches decision authority
        expected_authority = domain["decision_authority"].lower()
        if claiming_authority.lower() in expected_authority or expected_authority in claiming_authority.lower():
            return True
        return False
