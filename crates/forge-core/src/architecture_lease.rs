//! Deterministic architecture-evidence lease policy definitions.
//!
//! This module defines current-evidence eligibility policy only. It performs no
//! I/O, reads no ambient clock, grants no execution capability, and never turns
//! a lease policy into an [`crate::policy::AuthorizationGrant`].

use std::collections::BTreeMap;
use std::fmt::Write;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
}

/// A validated policy resolved from the registry with a deterministic fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicy {
    pub id: String,
    pub version: String,
    pub rule: LeaseRule,
    pub revalidation_mode: RevalidationMode,
    pub policy_fingerprint: String,
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
        definition.validate()?;

        Ok(EffectivePolicy {
            id: definition.id.clone(),
            version: definition.version.clone(),
            rule: definition.rule.clone(),
            revalidation_mode: definition.revalidation_mode,
            policy_fingerprint: policy_fingerprint(definition),
        })
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
            write!(out, "risk_at_most({})", risk.as_str())
                .expect("writing to String cannot fail");
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
