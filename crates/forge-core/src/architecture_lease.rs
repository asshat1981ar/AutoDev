//! Current-eligibility policy primitives for architecture evidence.
//!
//! This module is deliberately declarative. Lease policies are normalized data
//! evaluated by ForgeCore; they cannot invoke connectors, tools, network calls,
//! or arbitrary code and they never grant execution authorization.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::sha256_hex;

/// Risk tier associated with evidence governed by a lease policy.
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

/// How a policy permits evidence to regain current eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevalidationMode {
    /// Automatic renewal is permitted only when later evaluation proves the
    /// authoritative source is materially unchanged.
    AutomaticIfUnchanged,
    /// A normalized explicit revalidation signal is required.
    Explicit,
}

impl RevalidationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::AutomaticIfUnchanged => "automatic_if_unchanged",
            Self::Explicit => "explicit",
        }
    }
}

/// Closed, connector-free algebra for evidence lease eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseRule {
    MaxAge { seconds: u64 },
    SourceVersionRequired,
    FingerprintStable,
    RiskAtMost(RiskTier),
    ExplicitRevalidation,
    ExplicitInvalidationAbsent,
    AllOf(Vec<LeaseRule>),
    AnyOf(Vec<LeaseRule>),
}

impl LeaseRule {
    fn validate(&self) -> Result<(), LeasePolicyError> {
        match self {
            Self::MaxAge { seconds: 0 } => Err(LeasePolicyError::InvalidMaxAge),
            Self::AllOf(rules) if rules.is_empty() => {
                Err(LeasePolicyError::EmptyCompositeRule("all_of"))
            }
            Self::AnyOf(rules) if rules.is_empty() => {
                Err(LeasePolicyError::EmptyCompositeRule("any_of"))
            }
            Self::AllOf(rules) | Self::AnyOf(rules) => {
                for rule in rules {
                    rule.validate()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn canonical(&self) -> String {
        match self {
            Self::MaxAge { seconds } => format!("max_age:{seconds}"),
            Self::SourceVersionRequired => "source_version_required".into(),
            Self::FingerprintStable => "fingerprint_stable".into(),
            Self::RiskAtMost(risk) => format!("risk_at_most:{}", risk.as_str()),
            Self::ExplicitRevalidation => "explicit_revalidation".into(),
            Self::ExplicitInvalidationAbsent => "explicit_invalidation_absent".into(),
            Self::AllOf(rules) => canonical_composite("all_of", rules),
            Self::AnyOf(rules) => canonical_composite("any_of", rules),
        }
    }
}

fn canonical_composite(kind: &str, rules: &[LeaseRule]) -> String {
    // AllOf and AnyOf are commutative. Sorting their normalized children makes
    // semantically equivalent rule order hash identically.
    let mut children: Vec<_> = rules.iter().map(LeaseRule::canonical).collect();
    children.sort();
    format!("{kind}:[{}]", children.join(","))
}

/// Named repository policy definition before compilation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeasePolicyDefinition {
    pub id: String,
    pub version: String,
    pub rules: LeaseRule,
    pub risk_tier: RiskTier,
    pub revalidation_mode: RevalidationMode,
}

impl LeasePolicyDefinition {
    /// Validate policy structure before registration or compilation.
    pub fn validate(&self) -> Result<(), LeasePolicyError> {
        required(&self.id, "id")?;
        required(&self.version, "version")?;
        self.rules.validate()
    }

    /// Deterministic SHA-256 fingerprint of canonical policy semantics.
    pub fn fingerprint(&self) -> Result<String, LeasePolicyError> {
        self.validate()?;
        Ok(sha256_hex(self.canonical().as_bytes()))
    }

    fn canonical(&self) -> String {
        format!(
            "id={};version={};risk={};revalidation={};rules={}",
            self.id,
            self.version,
            self.risk_tier.as_str(),
            self.revalidation_mode.as_str(),
            self.rules.canonical(),
        )
    }
}

/// Validated policy state used by later lease evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicy {
    pub id: String,
    pub version: String,
    pub rules: LeaseRule,
    pub risk_tier: RiskTier,
    pub revalidation_mode: RevalidationMode,
    pub fingerprint: String,
}

/// Repository-backed approval reference type accepted for policy relaxation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalReferenceKind {
    Commit,
    PullRequest,
    Adr,
}

/// Normalized repository-backed approval reference.
///
/// This is approval *evidence*, not an execution authorization grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalReference {
    pub kind: ApprovalReferenceKind,
    pub reference: String,
}

impl ApprovalReference {
    fn validate(&self) -> Result<(), LeasePolicyError> {
        required(&self.reference, "approval_reference")
    }
}

/// Normalized observation that a repository approval reference was approved.
///
/// Connectors may produce this observation; ForgeCore alone evaluates whether
/// it satisfies a controlled policy-relaxation rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryApprovalEvidence {
    pub reference: ApprovalReference,
    pub approved: bool,
}

/// Metadata required when a repository policy intentionally relaxes a floor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRelaxation {
    pub rationale: String,
    pub approval_reference: ApprovalReference,
}

impl PolicyRelaxation {
    fn validate(&self) -> Result<(), LeasePolicyError> {
        required(&self.rationale, "relaxation_rationale")?;
        self.approval_reference.validate()
    }
}

/// Repository policy replacement applied against a registered safety floor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPolicyOverride {
    pub policy_id: String,
    pub replacement: LeasePolicyDefinition,
    pub allow_relaxation: bool,
    pub relaxation: Option<PolicyRelaxation>,
}

/// Registry of named lease policies. Unknown or malformed policies fail closed.
#[derive(Debug, Clone, Default)]
pub struct LeasePolicyRegistry {
    definitions: BTreeMap<String, LeasePolicyDefinition>,
}

impl LeasePolicyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one validated policy definition.
    pub fn register(&mut self, definition: LeasePolicyDefinition) -> Result<(), LeasePolicyError> {
        definition.validate()?;
        if self.definitions.contains_key(&definition.id) {
            return Err(LeasePolicyError::DuplicatePolicyId(definition.id));
        }
        self.definitions.insert(definition.id.clone(), definition);
        Ok(())
    }

    /// Compile a named definition into immutable effective policy state.
    pub fn compile(&self, id: &str) -> Result<EffectivePolicy, LeasePolicyError> {
        let definition = self.definition(id)?;
        compile_definition(definition)
    }

    /// Compile a repository override against a registered policy floor.
    ///
    /// Obvious tightening is accepted without approval. Obvious relaxation
    /// requires explicit opt-in, rationale, and matching repository-backed
    /// approval evidence. Comparisons that require general logical reasoning
    /// fail closed before approval metadata is considered.
    pub fn compile_with_override(
        &self,
        id: &str,
        override_policy: &RepositoryPolicyOverride,
        approvals: &[RepositoryApprovalEvidence],
    ) -> Result<EffectivePolicy, LeasePolicyError> {
        let base = self.definition(id)?;
        required(&override_policy.policy_id, "override_policy_id")?;
        if override_policy.policy_id != id {
            return Err(LeasePolicyError::OverridePolicyMismatch {
                expected: id.to_string(),
                actual: override_policy.policy_id.clone(),
            });
        }
        if override_policy.replacement.id != id {
            return Err(LeasePolicyError::OverridePolicyMismatch {
                expected: id.to_string(),
                actual: override_policy.replacement.id.clone(),
            });
        }
        override_policy.replacement.validate()?;

        match compare_policy_change(base, &override_policy.replacement)? {
            PolicyChange::Equivalent | PolicyChange::Tightening => {
                compile_definition(&override_policy.replacement)
            }
            PolicyChange::Relaxation => {
                if !override_policy.allow_relaxation {
                    return Err(LeasePolicyError::RelaxationNotAllowed);
                }
                let relaxation = override_policy
                    .relaxation
                    .as_ref()
                    .ok_or(LeasePolicyError::MissingRelaxationMetadata)?;
                relaxation.validate()?;
                let approval_reference = &relaxation.approval_reference;
                let approved = approvals
                    .iter()
                    .any(|evidence| evidence.approved && evidence.reference == *approval_reference);
                if !approved {
                    return Err(LeasePolicyError::MissingRepositoryApproval(
                        approval_reference.clone(),
                    ));
                }
                compile_definition(&override_policy.replacement)
            }
        }
    }

    fn definition(&self, id: &str) -> Result<&LeasePolicyDefinition, LeasePolicyError> {
        let definition = self
            .definitions
            .get(id)
            .ok_or_else(|| LeasePolicyError::UnknownPolicyId(id.to_string()))?;
        definition.validate()?;
        Ok(definition)
    }
}

fn compile_definition(
    definition: &LeasePolicyDefinition,
) -> Result<EffectivePolicy, LeasePolicyError> {
    definition.validate()?;
    Ok(EffectivePolicy {
        id: definition.id.clone(),
        version: definition.version.clone(),
        rules: definition.rules.clone(),
        risk_tier: definition.risk_tier,
        revalidation_mode: definition.revalidation_mode,
        fingerprint: definition.fingerprint()?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyChange {
    Equivalent,
    Tightening,
    Relaxation,
}

fn compare_policy_change(
    base: &LeasePolicyDefinition,
    replacement: &LeasePolicyDefinition,
) -> Result<PolicyChange, LeasePolicyError> {
    // Changing the assigned risk tier changes policy meaning but is not safely
    // comparable as a simple relaxation/tightening relation here.
    if base.risk_tier != replacement.risk_tier {
        return Err(LeasePolicyError::UnsupportedPolicyComparison);
    }

    let rule_change = compare_rule(&base.rules, &replacement.rules)?;
    let revalidation_change =
        compare_revalidation(base.revalidation_mode, replacement.revalidation_mode);
    combine_policy_changes(rule_change, revalidation_change)
}

fn compare_rule(
    base: &LeaseRule,
    replacement: &LeaseRule,
) -> Result<PolicyChange, LeasePolicyError> {
    if base == replacement {
        return Ok(PolicyChange::Equivalent);
    }

    match (base, replacement) {
        (
            LeaseRule::MaxAge {
                seconds: base_seconds,
            },
            LeaseRule::MaxAge {
                seconds: replacement_seconds,
            },
        ) => Ok(if replacement_seconds < base_seconds {
            PolicyChange::Tightening
        } else {
            PolicyChange::Relaxation
        }),
        (LeaseRule::RiskAtMost(base_risk), LeaseRule::RiskAtMost(replacement_risk)) => {
            Ok(if replacement_risk < base_risk {
                PolicyChange::Tightening
            } else {
                PolicyChange::Relaxation
            })
        }
        _ => Err(LeasePolicyError::UnsupportedPolicyComparison),
    }
}

fn compare_revalidation(base: RevalidationMode, replacement: RevalidationMode) -> PolicyChange {
    match (base, replacement) {
        (left, right) if left == right => PolicyChange::Equivalent,
        (RevalidationMode::AutomaticIfUnchanged, RevalidationMode::Explicit) => {
            PolicyChange::Tightening
        }
        (RevalidationMode::Explicit, RevalidationMode::AutomaticIfUnchanged) => {
            PolicyChange::Relaxation
        }
        _ => PolicyChange::Equivalent,
    }
}

fn combine_policy_changes(
    left: PolicyChange,
    right: PolicyChange,
) -> Result<PolicyChange, LeasePolicyError> {
    match (left, right) {
        (PolicyChange::Equivalent, change) | (change, PolicyChange::Equivalent) => Ok(change),
        (PolicyChange::Tightening, PolicyChange::Tightening) => Ok(PolicyChange::Tightening),
        (PolicyChange::Relaxation, PolicyChange::Relaxation) => Ok(PolicyChange::Relaxation),
        (PolicyChange::Tightening, PolicyChange::Relaxation)
        | (PolicyChange::Relaxation, PolicyChange::Tightening) => {
            Err(LeasePolicyError::UnsupportedPolicyComparison)
        }
    }
}

/// Fail-closed policy construction and lookup errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LeasePolicyError {
    #[error("lease policy field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("lease policy composite `{0}` must contain at least one rule")]
    EmptyCompositeRule(&'static str),
    #[error("max age must be greater than zero")]
    InvalidMaxAge,
    #[error("duplicate lease policy id `{0}`")]
    DuplicatePolicyId(String),
    #[error("unknown lease policy id `{0}`")]
    UnknownPolicyId(String),
    #[error("repository override policy mismatch: expected `{expected}`, got `{actual}`")]
    OverridePolicyMismatch { expected: String, actual: String },
    #[error("repository policy relaxation is not allowed")]
    RelaxationNotAllowed,
    #[error("repository policy relaxation metadata is required")]
    MissingRelaxationMetadata,
    #[error("missing approved repository evidence for {0:?}")]
    MissingRepositoryApproval(ApprovalReference),
    #[error("policy comparison is unsupported and therefore fails closed")]
    UnsupportedPolicyComparison,
}

fn required(value: &str, field: &'static str) -> Result<(), LeasePolicyError> {
    if value.trim().is_empty() {
        Err(LeasePolicyError::EmptyField(field))
    } else {
        Ok(())
    }
}
