//! Pure AMCX projection bridge for existing ForgeCore state.
//!
//! This module is deliberately non-authorizing: it validates source identity
//! and emits immutable references derived from canonical ForgeCore objects. It
//! does not execute effects, mutate plans/evidence/context, or construct
//! authorization grants.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ContextPack, Evidence, ExecPlan, ExecPlanStatus, PlanCheckpoint, VerificationKind,
    VerificationReport, VerificationVerdict,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmcxSourceIdentity {
    pub repository: String,
    pub revision: String,
    pub worktree: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmcxPlanRef {
    pub source: AmcxSourceIdentity,
    pub plan_id: String,
    pub checkpoint_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmcxEvidenceRef {
    pub source: AmcxSourceIdentity,
    pub evidence_id: String,
    pub fingerprint_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmcxVerificationRef {
    pub source: AmcxSourceIdentity,
    pub report_ref: String,
    pub report_sha256: String,
    pub completed_at: String,
    pub verdict: String,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmcxRepositoryContextRef {
    pub source: AmcxSourceIdentity,
    pub artifact_ref: String,
    pub artifact_sha256: String,
    pub query: String,
    pub item_count: usize,
    pub total_bytes: usize,
}

/// Immutable artifact identity whose digest has already been verified against
/// the exact artifact bytes supplied by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedArtifactRef {
    reference: String,
    sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AmcxBridgeError {
    #[error("AMCX projection source identity must be complete and non-blank")]
    MissingIdentity,
    #[error("execution evidence fingerprint failed verification")]
    InvalidEvidenceFingerprint,
    #[error("projection requires a non-blank immutable artifact reference")]
    MissingArtifactReference,
    #[error("projection requires a SHA-256 digest bound to the referenced artifact bytes")]
    InvalidArtifactDigest,
}

impl VerifiedArtifactRef {
    /// Construct a verified immutable artifact reference by hashing the exact
    /// artifact bytes and comparing them with the claimed SHA-256 digest.
    pub fn from_bytes(
        reference: &str,
        artifact_bytes: &[u8],
        claimed_sha256: &str,
    ) -> Result<Self, AmcxBridgeError> {
        if reference.trim().is_empty() {
            return Err(AmcxBridgeError::MissingArtifactReference);
        }
        if !valid_sha256_hex(claimed_sha256) {
            return Err(AmcxBridgeError::InvalidArtifactDigest);
        }

        let actual_sha256 = format!("{:x}", Sha256::digest(artifact_bytes));
        if !actual_sha256.eq_ignore_ascii_case(claimed_sha256) {
            return Err(AmcxBridgeError::InvalidArtifactDigest);
        }

        Ok(Self {
            reference: reference.to_string(),
            sha256: actual_sha256,
        })
    }

    pub fn reference(&self) -> &str {
        &self.reference
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

fn validate_source(source: &AmcxSourceIdentity) -> Result<(), AmcxBridgeError> {
    if source.repository.trim().is_empty()
        || source.revision.trim().is_empty()
        || source.worktree.trim().is_empty()
    {
        return Err(AmcxBridgeError::MissingIdentity);
    }
    Ok(())
}

fn valid_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn plan_status(status: ExecPlanStatus) -> &'static str {
    match status {
        ExecPlanStatus::Planned => "planned",
        ExecPlanStatus::Running => "running",
        ExecPlanStatus::Interrupted => "interrupted",
        ExecPlanStatus::Blocked => "blocked",
        ExecPlanStatus::Completed => "completed",
        ExecPlanStatus::Cancelled => "cancelled",
        ExecPlanStatus::Failed => "failed",
    }
}

fn verification_verdict(verdict: VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Pass => "pass",
        VerificationVerdict::Fail => "fail",
        VerificationVerdict::Skipped => "skipped",
    }
}

fn verification_kind(kind: VerificationKind) -> &'static str {
    kind.as_str()
}

pub fn project_plan(
    source: AmcxSourceIdentity,
    plan: &ExecPlan,
    checkpoint: &PlanCheckpoint,
) -> Result<AmcxPlanRef, AmcxBridgeError> {
    validate_source(&source)?;
    if plan.id().trim().is_empty()
        || checkpoint.id().trim().is_empty()
        || checkpoint.plan_id() != plan.id()
    {
        return Err(AmcxBridgeError::MissingIdentity);
    }

    Ok(AmcxPlanRef {
        source,
        plan_id: plan.id().to_string(),
        checkpoint_id: checkpoint.id().to_string(),
        status: plan_status(plan.status()).to_string(),
    })
}

pub fn project_evidence(
    source: AmcxSourceIdentity,
    evidence: &Evidence,
) -> Result<AmcxEvidenceRef, AmcxBridgeError> {
    validate_source(&source)?;
    if evidence.record.id.trim().is_empty() {
        return Err(AmcxBridgeError::MissingIdentity);
    }
    if !evidence.verify() || !valid_sha256_hex(&evidence.fingerprint.digest) {
        return Err(AmcxBridgeError::InvalidEvidenceFingerprint);
    }

    Ok(AmcxEvidenceRef {
        source,
        evidence_id: evidence.record.id.clone(),
        fingerprint_sha256: evidence.fingerprint.digest.clone(),
    })
}

pub fn project_verification(
    source: AmcxSourceIdentity,
    report: &VerificationReport,
    artifact: &VerifiedArtifactRef,
) -> Result<AmcxVerificationRef, AmcxBridgeError> {
    validate_source(&source)?;

    Ok(AmcxVerificationRef {
        source,
        report_ref: artifact.reference().to_string(),
        report_sha256: artifact.sha256().to_string(),
        completed_at: report.completed_at.to_rfc3339(),
        verdict: verification_verdict(report.overall).to_string(),
        checks: report
            .results
            .iter()
            .map(|result| verification_kind(result.kind).to_string())
            .collect(),
    })
}

pub fn project_context(
    source: AmcxSourceIdentity,
    pack: &ContextPack,
    artifact: &VerifiedArtifactRef,
) -> Result<AmcxRepositoryContextRef, AmcxBridgeError> {
    validate_source(&source)?;

    Ok(AmcxRepositoryContextRef {
        source,
        artifact_ref: artifact.reference().to_string(),
        artifact_sha256: artifact.sha256().to_string(),
        query: pack.query.clone(),
        item_count: pack.items.len(),
        total_bytes: pack.total_bytes,
    })
}
