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
        let definition = self
            .definitions
            .get(id)
            .ok_or_else(|| LeasePolicyError::UnknownPolicyId(id.to_string()))?;
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
}

fn required(value: &str, field: &'static str) -> Result<(), LeasePolicyError> {
    if value.trim().is_empty() {
        Err(LeasePolicyError::EmptyField(field))
    } else {
        Ok(())
    }
}
