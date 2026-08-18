//! Deterministic evaluation of current architecture-evidence leases.
//!
//! This module consumes normalized evidence, compiled policy, injected time,
//! optional prior lease state, and optional refresh proposals. It performs no
//! I/O and cannot authorize execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::architecture_evidence::EvidenceRecord;
use crate::architecture_lease::{
    ArchitectureLeaseError, EffectivePolicy, LeaseRule, RevalidationMode, RiskTier,
};

/// Immutable prior lease state consumed by the evaluator.
///
/// A4 intentionally defines only this data shape. Controlled issuance,
/// deterministic attestation fingerprinting, and self-validation are added in
/// A5; this structure alone is not execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseAttestation {
    pub evidence_id: String,
    pub objective_id: String,
    pub evidence_fingerprint: String,
    pub policy_id: String,
    pub policy_version: String,
    pub policy_fingerprint: String,
    pub source_version: String,
    pub evaluated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub risk_tier: RiskTier,
    pub attestation_fingerprint: String,
}

/// Normalized proposal containing refreshed source material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshProposal {
    pub previous_evidence_id: String,
    pub refreshed_evidence: EvidenceRecord,
    pub source_version: String,
    pub proposed_at: DateTime<Utc>,
}

/// Deterministic reason explaining a lease evaluation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEvaluationReason {
    FreshWithinPolicy,
    TtlExpired,
    SourceVersionChanged,
    FingerprintChanged,
    ExplicitlyInvalidated,
    PolicyChanged,
    MediumRiskReviewRequired,
    HighRiskReviewRequired,
    RelaxationMissingApproval,
}

/// Current eligibility state produced by the lease evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEvaluationStatus {
    Valid,
    Stale,
    RevalidationRequired,
    RelaxationApprovalRequired,
    Invalid,
}

/// Deterministic output of one lease evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEvaluation {
    pub status: LeaseEvaluationStatus,
    pub reason: LeaseEvaluationReason,
}

/// Evaluate whether architecture evidence remains currently eligible.
///
/// `evaluated_at` is injected explicitly. The evaluator never reads an ambient
/// clock or external source. Expiry uses the strict boundary
/// `evaluated_at < valid_until`; equality is stale.
pub fn evaluate_lease(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    risk_tier: RiskTier,
    evaluated_at: DateTime<Utc>,
    prior_attestation: Option<&LeaseAttestation>,
    refresh: Option<&RefreshProposal>,
    explicitly_invalidated: bool,
) -> Result<LeaseEvaluation, ArchitectureLeaseError> {
    validate_evidence(evidence)?;
    validate_policy(policy)?;

    if let Some(refresh) = refresh {
        validate_refresh(refresh, evidence, policy)?;
    }

    if policy
        .relaxation
        .as_ref()
        .is_some_and(|relaxation| !relaxation.allow_relaxation)
    {
        return Ok(LeaseEvaluation {
            status: LeaseEvaluationStatus::RelaxationApprovalRequired,
            reason: LeaseEvaluationReason::RelaxationMissingApproval,
        });
    }

    if explicitly_invalidated {
        return Ok(LeaseEvaluation {
            status: LeaseEvaluationStatus::Invalid,
            reason: LeaseEvaluationReason::ExplicitlyInvalidated,
        });
    }

    let Some(prior) = prior_attestation else {
        return Ok(LeaseEvaluation {
            status: LeaseEvaluationStatus::Valid,
            reason: LeaseEvaluationReason::FreshWithinPolicy,
        });
    };

    validate_prior_binding(prior, evidence, policy)?;

    if prior.policy_version != policy.version || prior.policy_fingerprint != policy.policy_fingerprint
    {
        return Ok(LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: LeaseEvaluationReason::PolicyChanged,
        });
    }

    if let Some(refresh) = refresh {
        if refresh.source_version != prior.source_version {
            return Ok(material_change_result(
                risk_tier,
                LeaseEvaluationReason::SourceVersionChanged,
            ));
        }
        if refresh.refreshed_evidence.content_fingerprint != prior.evidence_fingerprint {
            return Ok(material_change_result(
                risk_tier,
                LeaseEvaluationReason::FingerprintChanged,
            ));
        }
    }

    if evaluated_at >= prior.valid_until {
        if risk_tier == RiskTier::Low
            && policy.revalidation_mode == RevalidationMode::AutomaticLowRisk
            && refresh.is_some()
        {
            return Ok(LeaseEvaluation {
                status: LeaseEvaluationStatus::Valid,
                reason: LeaseEvaluationReason::FreshWithinPolicy,
            });
        }

        return Ok(expired_result(risk_tier));
    }

    Ok(LeaseEvaluation {
        status: LeaseEvaluationStatus::Valid,
        reason: LeaseEvaluationReason::FreshWithinPolicy,
    })
}

fn validate_evidence(evidence: &EvidenceRecord) -> Result<(), ArchitectureLeaseError> {
    evidence
        .validate()
        .map_err(|error| ArchitectureLeaseError::InvalidLeaseEvidence(error.to_string()))
}

fn validate_policy(policy: &EffectivePolicy) -> Result<(), ArchitectureLeaseError> {
    if policy.id.trim().is_empty() {
        return Err(ArchitectureLeaseError::EmptyField("policy_id"));
    }
    if policy.version.trim().is_empty() {
        return Err(ArchitectureLeaseError::EmptyField("policy_version"));
    }
    if policy.policy_fingerprint.trim().is_empty() {
        return Err(ArchitectureLeaseError::EmptyField("policy_fingerprint"));
    }
    validate_rule(&policy.rule)
}

fn validate_rule(rule: &LeaseRule) -> Result<(), ArchitectureLeaseError> {
    match rule {
        LeaseRule::MaxAgeSeconds(0) => Err(ArchitectureLeaseError::InvalidMaxAge(0)),
        LeaseRule::AllOf(children) if children.is_empty() => {
            Err(ArchitectureLeaseError::EmptyRuleSet("all_of"))
        }
        LeaseRule::AnyOf(children) if children.is_empty() => {
            Err(ArchitectureLeaseError::EmptyRuleSet("any_of"))
        }
        LeaseRule::AllOf(children) | LeaseRule::AnyOf(children) => {
            for child in children {
                validate_rule(child)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_refresh(
    refresh: &RefreshProposal,
    previous: &EvidenceRecord,
    policy: &EffectivePolicy,
) -> Result<(), ArchitectureLeaseError> {
    if refresh.previous_evidence_id.trim().is_empty() {
        return Err(ArchitectureLeaseError::EmptyField("previous_evidence_id"));
    }
    if refresh.previous_evidence_id != previous.id {
        return Err(ArchitectureLeaseError::RefreshPreviousEvidenceMismatch {
            expected: previous.id.clone(),
            actual: refresh.previous_evidence_id.clone(),
        });
    }
    if requires_source_version(&policy.rule) && refresh.source_version.trim().is_empty() {
        return Err(ArchitectureLeaseError::EmptyField("source_version"));
    }
    refresh
        .refreshed_evidence
        .validate()
        .map_err(|error| ArchitectureLeaseError::InvalidRefreshEvidence(error.to_string()))?;
    if refresh.refreshed_evidence.objective_id != previous.objective_id {
        return Err(ArchitectureLeaseError::RefreshObjectiveMismatch {
            expected: previous.objective_id.clone(),
            actual: refresh.refreshed_evidence.objective_id.clone(),
        });
    }
    if refresh.refreshed_evidence.id == previous.id {
        return Err(ArchitectureLeaseError::RefreshOverwritesPreviousEvidence(
            previous.id.clone(),
        ));
    }
    Ok(())
}

fn validate_prior_binding(
    prior: &LeaseAttestation,
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
) -> Result<(), ArchitectureLeaseError> {
    if prior.evidence_id != evidence.id {
        return Err(ArchitectureLeaseError::PriorEvidenceMismatch {
            expected: evidence.id.clone(),
            actual: prior.evidence_id.clone(),
        });
    }
    if prior.objective_id != evidence.objective_id {
        return Err(ArchitectureLeaseError::PriorObjectiveMismatch {
            expected: evidence.objective_id.clone(),
            actual: prior.objective_id.clone(),
        });
    }
    if prior.evidence_fingerprint != evidence.content_fingerprint {
        return Err(ArchitectureLeaseError::PriorEvidenceFingerprintMismatch {
            expected: evidence.content_fingerprint.clone(),
            actual: prior.evidence_fingerprint.clone(),
        });
    }
    if prior.policy_id != policy.id {
        return Err(ArchitectureLeaseError::PriorPolicyMismatch {
            expected: policy.id.clone(),
            actual: prior.policy_id.clone(),
        });
    }
    Ok(())
}

fn requires_source_version(rule: &LeaseRule) -> bool {
    match rule {
        LeaseRule::SourceVersionRequired => true,
        LeaseRule::AllOf(children) | LeaseRule::AnyOf(children) => {
            children.iter().any(requires_source_version)
        }
        _ => false,
    }
}

fn material_change_result(
    risk_tier: RiskTier,
    low_risk_reason: LeaseEvaluationReason,
) -> LeaseEvaluation {
    match risk_tier {
        RiskTier::Low => LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: low_risk_reason,
        },
        RiskTier::Medium => LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: LeaseEvaluationReason::MediumRiskReviewRequired,
        },
        RiskTier::High => LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: LeaseEvaluationReason::HighRiskReviewRequired,
        },
    }
}

fn expired_result(risk_tier: RiskTier) -> LeaseEvaluation {
    match risk_tier {
        RiskTier::Low => LeaseEvaluation {
            status: LeaseEvaluationStatus::Stale,
            reason: LeaseEvaluationReason::TtlExpired,
        },
        RiskTier::Medium => LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: LeaseEvaluationReason::MediumRiskReviewRequired,
        },
        RiskTier::High => LeaseEvaluation {
            status: LeaseEvaluationStatus::RevalidationRequired,
            reason: LeaseEvaluationReason::HighRiskReviewRequired,
        },
    }
}
