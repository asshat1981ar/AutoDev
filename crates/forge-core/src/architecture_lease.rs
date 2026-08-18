//! Deterministic architecture-evidence lease policy definitions and evaluation.
//!
//! This module defines current-evidence eligibility policy only. It performs no
//! I/O, reads no ambient clock, grants no execution capability, and never turns
//! a lease policy into an [`crate::policy::AuthorizationGrant`].

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::architecture_evidence::EvidenceRecord;
use crate::evidence::sha256_hex;

/// Risk tier used by the closed evidence-lease policy algebra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Low,
    Medium,
    High,
}

impl RiskTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// How a policy permits evidence to be revalidated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevalidationMode {
    AutomaticLowRisk,
    ExplicitOnMaterialChange,
    Explicit,
}

impl RevalidationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::AutomaticLowRisk => "automatic_low_risk",
            Self::ExplicitOnMaterialChange => "explicit_on_material_change",
            Self::Explicit => "explicit",
        }
    }

    fn strictness(self) -> u8 {
        match self {
            Self::AutomaticLowRisk => 0,
            Self::ExplicitOnMaterialChange => 1,
            Self::Explicit => 2,
        }
    }
}

/// Closed, non-Turing-complete lease-policy rule algebra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseRule {
    MaxAgeSeconds(u64),
    SourceVersionRequired,
    FingerprintStable,
    RiskAtMost(RiskTier),
    ExplicitRevalidation,
    ExplicitInvalidationAbsent,
    AllOf(Vec<LeaseRule>),
    AnyOf(Vec<LeaseRule>),
}

impl LeaseRule {
    fn validate(&self) -> Result<(), ArchitectureLeaseError> {
        match self {
            Self::MaxAgeSeconds(0) => Err(ArchitectureLeaseError::InvalidMaxAge(0)),
            Self::AllOf(children) => validate_children("all_of", children),
            Self::AnyOf(children) => validate_children("any_of", children),
            _ => Ok(()),
        }
    }
}

/// One named, versioned lease-policy definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeasePolicyDefinition {
    pub id: String,
    pub version: String,
    pub rule: LeaseRule,
    pub revalidation_mode: RevalidationMode,
}

impl LeasePolicyDefinition {
    /// Validate the complete policy definition recursively.
    pub fn validate(&self) -> Result<(), ArchitectureLeaseError> {
        required(&self.id, "id")?;
        required(&self.version, "version")?;
        self.rule.validate()
    }

    /// Return the deterministic fingerprint used by approval evidence and attestations.
    pub fn fingerprint(&self) -> Result<String, ArchitectureLeaseError> {
        self.validate()?;
        Ok(policy_fingerprint(self))
    }
}

/// Repository-backed kinds of policy-relaxation approval references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalReferenceKind {
    Commit,
    PullRequest,
    Adr,
}

/// Immutable repository reference naming where a relaxation was approved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalReference {
    pub repository: String,
    pub kind: ApprovalReferenceKind,
    pub reference: String,
}

impl ApprovalReference {
    pub fn validate(&self) -> Result<(), ArchitectureLeaseError> {
        required(&self.repository, "approval_reference.repository")?;
        required(&self.reference, "approval_reference.reference")
    }
}

/// Normalized repository evidence proving approval of an exact candidate policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryApprovalEvidence {
    pub approval_reference: ApprovalReference,
    pub policy_id: String,
    pub approved_policy_version: String,
    pub approved_policy_fingerprint: String,
}

impl RepositoryApprovalEvidence {
    pub fn validate(&self) -> Result<(), ArchitectureLeaseError> {
        self.approval_reference.validate()?;
        required(&self.policy_id, "policy_id")?;
        required(&self.approved_policy_version, "approved_policy_version")?;
        if !is_sha256_hex(&self.approved_policy_fingerprint) {
            return Err(ArchitectureLeaseError::InvalidApprovalFingerprint(
                self.approved_policy_fingerprint.clone(),
            ));
        }
        Ok(())
    }
}

/// Explicit metadata required before a repository policy may relax a safety floor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRelaxation {
    pub allow_relaxation: bool,
    pub rationale: String,
    pub approval_reference: ApprovalReference,
}

impl PolicyRelaxation {
    fn validate(&self) -> Result<(), ArchitectureLeaseError> {
        if !self.allow_relaxation {
            return Err(ArchitectureLeaseError::UnsafePolicyRelaxation);
        }
        if self.rationale.trim().is_empty() {
            return Err(ArchitectureLeaseError::InvalidPolicyRelaxation);
        }
        self.approval_reference.validate()
    }
}

/// Optional repository-local candidate policy and its relaxation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPolicyOverride {
    pub definition: LeasePolicyDefinition,
    pub relaxation: Option<PolicyRelaxation>,
}

/// A validated policy resolved from the registry with a deterministic fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicy {
    pub id: String,
    pub version: String,
    pub rule: LeaseRule,
    pub revalidation_mode: RevalidationMode,
    pub policy_fingerprint: String,
    pub relaxation: Option<PolicyRelaxation>,
}

/// Immutable prior lease-state data consumed by the evaluator.
///
/// A4 intentionally exposes only the data shape. A5 owns trusted issuance,
/// deterministic attestation fingerprinting, expiry derivation, and validation.
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

/// Adapter-normalized candidate refresh input. It proposes new evidence but does
/// not mutate or replace the historical evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshProposal {
    pub previous_evidence_id: String,
    pub refreshed_evidence: EvidenceRecord,
    pub source_version: String,
    pub proposed_at: DateTime<Utc>,
}

impl RefreshProposal {
    fn validate_against(&self, previous: &EvidenceRecord) -> Result<(), ArchitectureLeaseError> {
        required(&self.previous_evidence_id, "previous_evidence_id")?;
        required(&self.source_version, "source_version")?;
        if self.previous_evidence_id != previous.id {
            return Err(ArchitectureLeaseError::RefreshPreviousEvidenceMismatch {
                expected: previous.id.clone(),
                actual: self.previous_evidence_id.clone(),
            });
        }
        if self.refreshed_evidence.id == previous.id {
            return Err(ArchitectureLeaseError::RefreshOverwritesPreviousEvidence(
                previous.id.clone(),
            ));
        }
        if self.refreshed_evidence.objective_id != previous.objective_id {
            return Err(ArchitectureLeaseError::RefreshObjectiveMismatch {
                expected: previous.objective_id.clone(),
                actual: self.refreshed_evidence.objective_id.clone(),
            });
        }
        Ok(())
    }
}

/// Current deterministic lease state produced by ForgeCore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEvaluationStatus {
    Valid,
    Stale,
    RevalidationRequired,
    Invalid,
}

/// Stable reason code explaining a lease-state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEvaluationReason {
    FreshWithinPolicy,
    TtlExpired,
    SourceVersionChanged,
    FingerprintChanged,
    PolicyChanged,
    MediumRiskReviewRequired,
    HighRiskReviewRequired,
    ExplicitlyInvalidated,
}

/// Deterministic result of evaluating one evidence record under one policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEvaluation {
    pub status: LeaseEvaluationStatus,
    pub reason: LeaseEvaluationReason,
    pub evaluated_at: DateTime<Utc>,
}

/// Evaluate current evidence eligibility using only explicit inputs.
///
/// `evaluated_at` is caller-supplied; this function reads no ambient clock and
/// performs no connector, network, filesystem, process, or authorization work.
pub fn evaluate_lease(
    evidence: &EvidenceRecord,
    policy: &EffectivePolicy,
    risk_tier: RiskTier,
    evaluated_at: DateTime<Utc>,
    prior_attestation: Option<&LeaseAttestation>,
    refresh_proposal: Option<&RefreshProposal>,
    explicitly_invalidated: bool,
) -> Result<LeaseEvaluation, ArchitectureLeaseError> {
    required(&policy.id, "policy.id")?;
    required(&policy.version, "policy.version")?;
    policy.rule.validate()?;

    if let Some(proposal) = refresh_proposal {
        proposal.validate_against(evidence)?;
    }

    if explicitly_invalidated {
        return Ok(evaluation(
            LeaseEvaluationStatus::Invalid,
            LeaseEvaluationReason::ExplicitlyInvalidated,
            evaluated_at,
        ));
    }

    let Some(prior) = prior_attestation else {
        return Ok(review_for_risk(risk_tier, evaluated_at));
    };

    if prior.policy_id != policy.id
        || prior.policy_version != policy.version
        || prior.policy_fingerprint != policy.policy_fingerprint
    {
        return Ok(evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::PolicyChanged,
            evaluated_at,
        ));
    }

    if prior.evidence_id != evidence.id
        || prior.objective_id != evidence.objective_id
        || prior.evidence_fingerprint != evidence.content_fingerprint
    {
        return Ok(evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::FingerprintChanged,
            evaluated_at,
        ));
    }

    if let Some(proposal) = refresh_proposal {
        let source_changed = proposal.source_version != prior.source_version;
        let fingerprint_changed =
            proposal.refreshed_evidence.content_fingerprint != evidence.content_fingerprint;

        if source_changed || fingerprint_changed {
            return Ok(match risk_tier {
                RiskTier::High => evaluation(
                    LeaseEvaluationStatus::RevalidationRequired,
                    LeaseEvaluationReason::HighRiskReviewRequired,
                    evaluated_at,
                ),
                RiskTier::Medium => evaluation(
                    LeaseEvaluationStatus::RevalidationRequired,
                    LeaseEvaluationReason::MediumRiskReviewRequired,
                    evaluated_at,
                ),
                RiskTier::Low if source_changed => evaluation(
                    LeaseEvaluationStatus::RevalidationRequired,
                    LeaseEvaluationReason::SourceVersionChanged,
                    evaluated_at,
                ),
                RiskTier::Low => evaluation(
                    LeaseEvaluationStatus::RevalidationRequired,
                    LeaseEvaluationReason::FingerprintChanged,
                    evaluated_at,
                ),
            });
        }

        if risk_tier == RiskTier::Low
            && policy.revalidation_mode == RevalidationMode::AutomaticLowRisk
        {
            return Ok(evaluation(
                LeaseEvaluationStatus::Valid,
                LeaseEvaluationReason::FreshWithinPolicy,
                evaluated_at,
            ));
        }
    }

    if evaluated_at < prior.valid_until {
        return Ok(evaluation(
            LeaseEvaluationStatus::Valid,
            LeaseEvaluationReason::FreshWithinPolicy,
            evaluated_at,
        ));
    }

    Ok(match risk_tier {
        RiskTier::High => evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::HighRiskReviewRequired,
            evaluated_at,
        ),
        RiskTier::Medium => evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::MediumRiskReviewRequired,
            evaluated_at,
        ),
        RiskTier::Low => evaluation(
            LeaseEvaluationStatus::Stale,
            LeaseEvaluationReason::TtlExpired,
            evaluated_at,
        ),
    })
}

/// Deterministic registry of named evidence-lease policies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeasePolicyRegistry {
    policies: BTreeMap<String, LeasePolicyDefinition>,
}

impl LeasePolicyRegistry {
    /// Construct the built-in safety-floor registry.
    ///
    /// Production TTL values are intentionally not invented here. The initial
    /// `repo_state` policy proves source stability and low-risk eligibility;
    /// later evaluation decides whether evidence is currently acceptable.
    pub fn built_ins() -> Self {
        let repo_state = LeasePolicyDefinition {
            id: "repo_state".to_string(),
            version: "1".to_string(),
            rule: LeaseRule::AllOf(vec![
                LeaseRule::SourceVersionRequired,
                LeaseRule::FingerprintStable,
                LeaseRule::RiskAtMost(RiskTier::Low),
                LeaseRule::ExplicitInvalidationAbsent,
            ]),
            revalidation_mode: RevalidationMode::AutomaticLowRisk,
        };

        let mut policies = BTreeMap::new();
        policies.insert(repo_state.id.clone(), repo_state);
        Self { policies }
    }

    /// Return a policy definition without compiling repository overrides.
    pub fn get(&self, id: &str) -> Option<&LeasePolicyDefinition> {
        self.policies.get(id)
    }

    /// Resolve and validate one built-in policy deterministically.
    pub fn resolve(&self, id: &str) -> Result<EffectivePolicy, ArchitectureLeaseError> {
        let definition = self
            .get(id)
            .ok_or_else(|| ArchitectureLeaseError::UnknownLeasePolicy(id.to_string()))?;
        effective_from(definition, None)
    }

    /// Compile a repository override against its built-in safety floor.
    ///
    /// Tightening is allowed without approval. Relaxation requires explicit
    /// relaxation metadata plus repository-observed approval evidence bound to
    /// the exact candidate policy fingerprint. Structurally incomparable
    /// policies fail closed.
    pub fn compile(
        &self,
        policy_id: &str,
        repository_override: Option<&RepositoryPolicyOverride>,
        approval_evidence: Option<&RepositoryApprovalEvidence>,
    ) -> Result<EffectivePolicy, ArchitectureLeaseError> {
        let base = self
            .get(policy_id)
            .ok_or_else(|| ArchitectureLeaseError::UnknownLeasePolicy(policy_id.to_string()))?;
        base.validate()?;

        let Some(repository_override) = repository_override else {
            return self.resolve(policy_id);
        };
        let candidate = &repository_override.definition;
        candidate.validate()?;
        if candidate.id != policy_id {
            return Err(ArchitectureLeaseError::PolicyIdMismatch {
                expected: policy_id.to_string(),
                actual: candidate.id.clone(),
            });
        }

        match compare_definition(base, candidate)? {
            PolicyRelation::Equal | PolicyRelation::Tightening => effective_from(candidate, None),
            PolicyRelation::Relaxation => {
                let relaxation = repository_override
                    .relaxation
                    .as_ref()
                    .ok_or(ArchitectureLeaseError::UnsafePolicyRelaxation)?;
                relaxation.validate()?;
                let approval =
                    approval_evidence.ok_or(ArchitectureLeaseError::RelaxationApprovalRequired)?;
                approval.validate()?;
                let candidate_fingerprint = candidate.fingerprint()?;

                if approval.approval_reference != relaxation.approval_reference
                    || approval.policy_id != candidate.id
                    || approval.approved_policy_version != candidate.version
                    || approval.approved_policy_fingerprint != candidate_fingerprint
                {
                    return Err(ArchitectureLeaseError::ApprovalEvidenceMismatch);
                }

                effective_from(candidate, Some(relaxation.clone()))
            }
        }
    }
}

/// Typed failures for deterministic lease-policy construction and resolution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArchitectureLeaseError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("max age must be greater than zero seconds, got {0}")]
    InvalidMaxAge(u64),
    #[error("lease rule set `{0}` must not be empty")]
    EmptyRuleSet(&'static str),
    #[error("unknown lease policy `{0}`")]
    UnknownLeasePolicy(String),
    #[error("repository policy id `{actual}` does not match requested policy `{expected}`")]
    PolicyIdMismatch { expected: String, actual: String },
    #[error("invalid approved policy SHA-256 fingerprint `{0}`")]
    InvalidApprovalFingerprint(String),
    #[error("repository policy would relax the built-in safety floor without permission")]
    UnsafePolicyRelaxation,
    #[error("policy relaxation metadata is invalid")]
    InvalidPolicyRelaxation,
    #[error("policy relaxation requires repository-backed approval evidence")]
    RelaxationApprovalRequired,
    #[error("repository approval evidence does not match the exact candidate policy")]
    ApprovalEvidenceMismatch,
    #[error("policy structures cannot be compared safely")]
    UnsupportedPolicyComparison,
    #[error("refresh previous evidence id `{actual}` does not match `{expected}`")]
    RefreshPreviousEvidenceMismatch { expected: String, actual: String },
    #[error("refresh objective `{actual}` does not match `{expected}`")]
    RefreshObjectiveMismatch { expected: String, actual: String },
    #[error("refresh proposal cannot overwrite historical evidence `{0}`")]
    RefreshOverwritesPreviousEvidence(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyRelation {
    Equal,
    Tightening,
    Relaxation,
}

fn evaluation(
    status: LeaseEvaluationStatus,
    reason: LeaseEvaluationReason,
    evaluated_at: DateTime<Utc>,
) -> LeaseEvaluation {
    LeaseEvaluation {
        status,
        reason,
        evaluated_at,
    }
}

fn review_for_risk(risk_tier: RiskTier, evaluated_at: DateTime<Utc>) -> LeaseEvaluation {
    match risk_tier {
        RiskTier::High => evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::HighRiskReviewRequired,
            evaluated_at,
        ),
        RiskTier::Medium => evaluation(
            LeaseEvaluationStatus::RevalidationRequired,
            LeaseEvaluationReason::MediumRiskReviewRequired,
            evaluated_at,
        ),
        RiskTier::Low => evaluation(
            LeaseEvaluationStatus::Stale,
            LeaseEvaluationReason::TtlExpired,
            evaluated_at,
        ),
    }
}

fn validate_children(
    kind: &'static str,
    children: &[LeaseRule],
) -> Result<(), ArchitectureLeaseError> {
    if children.is_empty() {
        return Err(ArchitectureLeaseError::EmptyRuleSet(kind));
    }
    for child in children {
        child.validate()?;
    }
    Ok(())
}

fn required(value: &str, field: &'static str) -> Result<(), ArchitectureLeaseError> {
    if value.trim().is_empty() {
        Err(ArchitectureLeaseError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn effective_from(
    definition: &LeasePolicyDefinition,
    relaxation: Option<PolicyRelaxation>,
) -> Result<EffectivePolicy, ArchitectureLeaseError> {
    Ok(EffectivePolicy {
        id: definition.id.clone(),
        version: definition.version.clone(),
        rule: definition.rule.clone(),
        revalidation_mode: definition.revalidation_mode,
        policy_fingerprint: definition.fingerprint()?,
        relaxation,
    })
}

fn compare_definition(
    base: &LeasePolicyDefinition,
    candidate: &LeasePolicyDefinition,
) -> Result<PolicyRelation, ArchitectureLeaseError> {
    let rule_relation = compare_rule(&base.rule, &candidate.rule)?;
    let mode_relation = compare_revalidation(base.revalidation_mode, candidate.revalidation_mode);
    combine_relation(rule_relation, mode_relation)
}

fn compare_revalidation(base: RevalidationMode, candidate: RevalidationMode) -> PolicyRelation {
    match candidate.strictness().cmp(&base.strictness()) {
        Ordering::Less => PolicyRelation::Relaxation,
        Ordering::Equal => PolicyRelation::Equal,
        Ordering::Greater => PolicyRelation::Tightening,
    }
}

fn compare_rule(
    base: &LeaseRule,
    candidate: &LeaseRule,
) -> Result<PolicyRelation, ArchitectureLeaseError> {
    if base == candidate {
        return Ok(PolicyRelation::Equal);
    }

    match (base, candidate) {
        (LeaseRule::MaxAgeSeconds(base_age), LeaseRule::MaxAgeSeconds(candidate_age)) => {
            Ok(match candidate_age.cmp(base_age) {
                Ordering::Less => PolicyRelation::Tightening,
                Ordering::Equal => PolicyRelation::Equal,
                Ordering::Greater => PolicyRelation::Relaxation,
            })
        }
        (LeaseRule::RiskAtMost(base_risk), LeaseRule::RiskAtMost(candidate_risk)) => {
            Ok(match candidate_risk.cmp(base_risk) {
                Ordering::Less => PolicyRelation::Tightening,
                Ordering::Equal => PolicyRelation::Equal,
                Ordering::Greater => PolicyRelation::Relaxation,
            })
        }
        (LeaseRule::AllOf(base_children), LeaseRule::AllOf(candidate_children)) => {
            compare_children(base_children, candidate_children, true)
        }
        (LeaseRule::AnyOf(base_children), LeaseRule::AnyOf(candidate_children)) => {
            compare_children(base_children, candidate_children, false)
        }
        _ => Err(ArchitectureLeaseError::UnsupportedPolicyComparison),
    }
}

fn compare_children(
    base: &[LeaseRule],
    candidate: &[LeaseRule],
    all_of: bool,
) -> Result<PolicyRelation, ArchitectureLeaseError> {
    let mut relation = PolicyRelation::Equal;
    let shared_len = base.len().min(candidate.len());
    for index in 0..shared_len {
        relation = combine_relation(relation, compare_rule(&base[index], &candidate[index])?)?;
    }

    if base.len() != candidate.len() {
        let length_relation = if all_of {
            if candidate.len() > base.len() {
                PolicyRelation::Tightening
            } else {
                PolicyRelation::Relaxation
            }
        } else if candidate.len() > base.len() {
            PolicyRelation::Relaxation
        } else {
            PolicyRelation::Tightening
        };
        relation = combine_relation(relation, length_relation)?;
    }

    Ok(relation)
}

fn combine_relation(
    left: PolicyRelation,
    right: PolicyRelation,
) -> Result<PolicyRelation, ArchitectureLeaseError> {
    match (left, right) {
        (PolicyRelation::Equal, other) | (other, PolicyRelation::Equal) => Ok(other),
        (PolicyRelation::Tightening, PolicyRelation::Tightening) => Ok(PolicyRelation::Tightening),
        (PolicyRelation::Relaxation, PolicyRelation::Relaxation) => Ok(PolicyRelation::Relaxation),
        _ => Err(ArchitectureLeaseError::UnsupportedPolicyComparison),
    }
}

fn policy_fingerprint(definition: &LeasePolicyDefinition) -> String {
    let mut canonical = String::new();
    write!(
        canonical,
        "id:{}:{}|version:{}:{}|revalidation:{}|rule:",
        definition.id.len(),
        definition.id,
        definition.version.len(),
        definition.version,
        definition.revalidation_mode.as_str(),
    )
    .expect("writing to String cannot fail");
    canonical_rule(&definition.rule, &mut canonical);
    sha256_hex(canonical.as_bytes())
}

fn canonical_rule(rule: &LeaseRule, out: &mut String) {
    match rule {
        LeaseRule::MaxAgeSeconds(value) => {
            write!(out, "max_age({value})").expect("writing to String cannot fail");
        }
        LeaseRule::SourceVersionRequired => out.push_str("source_version_required"),
        LeaseRule::FingerprintStable => out.push_str("fingerprint_stable"),
        LeaseRule::RiskAtMost(risk) => {
            write!(out, "risk_at_most({})", risk.as_str()).expect("writing to String cannot fail");
        }
        LeaseRule::ExplicitRevalidation => out.push_str("explicit_revalidation"),
        LeaseRule::ExplicitInvalidationAbsent => out.push_str("explicit_invalidation_absent"),
        LeaseRule::AllOf(children) => canonical_children("all", children, out),
        LeaseRule::AnyOf(children) => canonical_children("any", children, out),
    }
}

fn canonical_children(kind: &str, children: &[LeaseRule], out: &mut String) {
    out.push_str(kind);
    out.push('(');
    for child in children {
        canonical_rule(child, out);
        out.push(';');
    }
    out.push(')');
}
