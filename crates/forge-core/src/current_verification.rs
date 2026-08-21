//! Lease-aware current verification for architecture decisions.
//!
//! Historical decision maturity is immutable. This gate answers only whether
//! referenced evidence is eligible *now* under its current lease attestation
//! and effective policy. It performs no I/O, reads no ambient clock, and grants
//! no execution authority.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::architecture_evidence::{
    ArchitectureDecision, ArchitectureEvidenceError, EvidenceRecord,
};
use crate::architecture_lease::{EffectivePolicy, LeaseAttestation};

/// Whether a decision has at least one evidence reference that can satisfy the
/// verified gate at the supplied evaluation time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentVerificationStatus {
    Eligible,
    Ineligible,
}

/// Deterministic partition of a decision's evidence references by current
/// eligibility. Reference order is preserved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentVerificationResult {
    pub status: CurrentVerificationStatus,
    pub eligible_evidence_ids: Vec<String>,
    pub ineligible_evidence_ids: Vec<String>,
}

/// Evaluate whether the evidence supporting an architecture decision remains
/// eligible at an explicit point in time.
///
/// Lease failures are represented as current ineligibility rather than as a
/// mutation of historical decision maturity. Structural evidence failures and
/// dangling decision references remain hard errors.
pub fn evaluate_current_verification(
    decision: &ArchitectureDecision,
    evidence: &BTreeMap<String, EvidenceRecord>,
    attestations: &BTreeMap<String, LeaseAttestation>,
    policies: &BTreeMap<String, EffectivePolicy>,
    evaluated_at: DateTime<Utc>,
) -> Result<CurrentVerificationResult, ArchitectureEvidenceError> {
    decision.validate(evidence)?;

    let mut eligible_evidence_ids = Vec::new();
    let mut ineligible_evidence_ids = Vec::new();

    for evidence_id in &decision.evidence_refs {
        let record = evidence.get(evidence_id).ok_or_else(|| {
            ArchitectureEvidenceError::UnknownEvidenceReference(
                decision.id.clone(),
                evidence_id.clone(),
            )
        })?;
        record.validate()?;

        if record.objective_id != decision.objective_id {
            return Err(ArchitectureEvidenceError::ObjectiveMismatch {
                item_kind: "evidence",
                item_id: record.id.clone(),
                expected: decision.objective_id.clone(),
                actual: record.objective_id.clone(),
            });
        }

        let eligible = record.evidence_class.can_satisfy_verified_gate()
            && attestations.get(evidence_id).is_some_and(|attestation| {
                attestation.validate().is_ok()
                    && attestation.evidence_id == record.id
                    && attestation.objective_id == record.objective_id
                    && attestation.evidence_fingerprint == record.content_fingerprint
                    && evaluated_at < attestation.valid_until
                    && policies.get(&attestation.policy_id).is_some_and(|policy| {
                        policy.id == attestation.policy_id
                            && policy.policy_fingerprint == attestation.policy_fingerprint
                    })
            });

        if eligible {
            eligible_evidence_ids.push(evidence_id.clone());
        } else {
            ineligible_evidence_ids.push(evidence_id.clone());
        }
    }

    let status = if eligible_evidence_ids.is_empty() {
        CurrentVerificationStatus::Ineligible
    } else {
        CurrentVerificationStatus::Eligible
    };

    Ok(CurrentVerificationResult {
        status,
        eligible_evidence_ids,
        ineligible_evidence_ids,
    })
}
