//! ForgeLoop skill definitions and deterministic routing.
//!
//! Skills are declarative capabilities, not processes. A [`SkillRegistry`]
//! stores reusable development behaviors and [`route_skills`] selects a small,
//! auditable set for a [`DevelopmentContract`]. The first implementation is
//! deliberately deterministic and model-free so learned routing can later be
//! evaluated against a reproducible baseline.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{AgentRole, Capability, RiskLevel};

/// A normalized development request used for skill routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevelopmentContract {
    /// Desired outcome in natural language.
    pub goal: String,
    /// Observable conditions that define success.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    /// Constraints that should influence planning/routing.
    #[serde(default)]
    pub constraints: Vec<String>,
    /// Capabilities known to be required by the requested work.
    #[serde(default)]
    pub required_capabilities: Vec<Capability>,
    /// Maximum risk that may be selected without escalation.
    pub risk_ceiling: RiskLevel,
}

impl DevelopmentContract {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            acceptance_criteria: Vec::new(),
            constraints: Vec::new(),
            required_capabilities: Vec::new(),
            risk_ceiling: RiskLevel::Medium,
        }
    }

    fn searchable_text(&self) -> String {
        let mut parts =
            Vec::with_capacity(1 + self.acceptance_criteria.len() + self.constraints.len());
        parts.push(self.goal.as_str());
        parts.extend(self.acceptance_criteria.iter().map(String::as_str));
        parts.extend(self.constraints.iter().map(String::as_str));
        parts.join(" ").to_ascii_lowercase()
    }
}

/// Declarative metadata for a reusable development skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Stable wire identifier, e.g. `debug-systematically`.
    pub id: String,
    pub description: String,
    /// Terms that make the skill relevant to a development contract.
    #[serde(default)]
    pub activation_terms: Vec<String>,
    /// Capabilities the skill may require when executed.
    #[serde(default)]
    pub required_capabilities: Vec<Capability>,
    /// Logical agent roles that can execute the skill.
    #[serde(default)]
    pub compatible_roles: Vec<AgentRole>,
    /// Maximum expected risk for normal execution of this skill.
    pub risk: RiskLevel,
    /// Human-readable checks expected after execution.
    #[serde(default)]
    pub verification: Vec<String>,
    /// Relative routing-cost hint. Lower is cheaper.
    pub cost_hint: u32,
}

/// Append-oriented registry of skill definitions.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: Vec<SkillDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SkillError {
    #[error("skill '{0}' is already registered")]
    Duplicate(String),
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, skill: SkillDefinition) -> Result<(), SkillError> {
        if self.skills.iter().any(|s| s.id == skill.id) {
            return Err(SkillError::Duplicate(skill.id));
        }
        self.skills.push(skill);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&SkillDefinition> {
        self.skills.iter().find(|s| s.id == id)
    }

    pub fn skills(&self) -> &[SkillDefinition] {
        &self.skills
    }
}

/// Evidence explaining why a skill was selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRoutingEvidence {
    pub skill_id: String,
    pub score: u32,
    pub reasons: Vec<String>,
}

/// Deterministic result of routing a development contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRoute {
    pub selected: Vec<SkillRoutingEvidence>,
}

/// Select at most `max_skills` relevant skills, best first.
///
/// Routing combines activation-term matches and required-capability overlap,
/// rejects skills above the contract's risk ceiling, then uses lower cost as a
/// deterministic tie-break before skill id.
pub fn route_skills(
    registry: &SkillRegistry,
    contract: &DevelopmentContract,
    max_skills: usize,
) -> SkillRoute {
    let text = contract.searchable_text();
    let contract_caps: BTreeSet<String> = contract
        .required_capabilities
        .iter()
        .map(|cap| cap.as_str().to_string())
        .collect();

    let mut scored: Vec<(SkillRoutingEvidence, u32)> = registry
        .skills()
        .iter()
        .filter(|skill| risk_rank(skill.risk) <= risk_rank(contract.risk_ceiling))
        .filter_map(|skill| {
            let mut score = 0u32;
            let mut reasons = Vec::new();

            for term in &skill.activation_terms {
                let term = term.to_ascii_lowercase();
                if !term.is_empty() && text.contains(&term) {
                    score += 10;
                    reasons.push(format!("term:{term}"));
                }
            }

            for cap in &skill.required_capabilities {
                if contract_caps.contains(cap.as_str()) {
                    score += 6;
                    reasons.push(format!("capability:{}", cap.as_str()));
                }
            }

            if score == 0 {
                None
            } else {
                Some((
                    SkillRoutingEvidence {
                        skill_id: skill.id.clone(),
                        score,
                        reasons,
                    },
                    skill.cost_hint,
                ))
            }
        })
        .collect();

    scored.sort_by(|(a, a_cost), (b, b_cost)| {
        b.score
            .cmp(&a.score)
            .then_with(|| a_cost.cmp(b_cost))
            .then_with(|| a.skill_id.cmp(&b.skill_id))
    });

    SkillRoute {
        selected: scored
            .into_iter()
            .take(max_skills)
            .map(|(evidence, _)| evidence)
            .collect(),
    }
}

fn risk_rank(risk: RiskLevel) -> u8 {
    match risk {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Critical => 3,
    }
}

/// Initial ForgeLoop skill catalog. These are intentionally broad primitives;
/// additional skills can be registered without changing the router.
pub fn default_skills() -> SkillRegistry {
    let mut registry = SkillRegistry::new();
    for skill in [
        skill(
            "map-repository",
            "Inspect repository structure, entrypoints, manifests, tests, CI, and architecture boundaries.",
            &[
                "repository",
                "architecture",
                "entrypoint",
                "codebase",
                "inspect",
            ],
            &[Capability::ReadFile, Capability::Git],
            &[AgentRole::Researcher, AgentRole::Architect],
            RiskLevel::Low,
            &["repository evidence captured"],
            1,
        ),
        skill(
            "design-context-fabric",
            "Design or improve bounded repository context selection and project memory.",
            &[
                "context",
                "context routing",
                "retrieval",
                "memory",
                "repository intelligence",
            ],
            &[Capability::ReadFile],
            &[AgentRole::Architect, AgentRole::Researcher],
            RiskLevel::Low,
            &["retrieval behavior is deterministic or benchmarked"],
            2,
        ),
        skill(
            "build-vertical-slice",
            "Implement the smallest end-to-end path that proves a design.",
            &[
                "implement",
                "build",
                "feature",
                "continue development",
                "vertical slice",
            ],
            &[
                Capability::ReadFile,
                Capability::WriteFile,
                Capability::PatchFile,
            ],
            &[AgentRole::Developer],
            RiskLevel::Medium,
            &["focused tests pass", "acceptance criteria are met"],
            3,
        ),
        skill(
            "debug-systematically",
            "Reproduce failures, rank hypotheses, repair the cause, and add regression evidence.",
            &["bug", "failure", "failed", "error", "debug", "regression"],
            &[Capability::ReadFile, Capability::RunTest],
            &[AgentRole::Developer, AgentRole::Tester],
            RiskLevel::Medium,
            &["failure reproduced", "regression test passes"],
            2,
        ),
        skill(
            "test-risk-first",
            "Prioritize verification around high-impact failure modes and trust boundaries.",
            &["test", "verify", "security", "risk", "boundary"],
            &[Capability::ReadFile, Capability::RunTest],
            &[AgentRole::Tester, AgentRole::SecurityReviewer],
            RiskLevel::Medium,
            &["risk-relevant checks pass"],
            2,
        ),
        skill(
            "review-change-gate",
            "Review a completed patch for correctness, security, maintainability, and architecture drift.",
            &[
                "review",
                "gate",
                "maintainability",
                "architecture drift",
            ],
            &[Capability::ReadFile, Capability::Git],
            &[
                AgentRole::Architect,
                AgentRole::SecurityReviewer,
                AgentRole::Tester,
            ],
            RiskLevel::Low,
            &["change-gate findings recorded"],
            1,
        ),
    ] {
        registry
            .register(skill)
            .expect("default skill ids are unique");
    }
    registry
}

// This private catalog constructor is intentionally declarative: keeping all
// skill metadata visible at each call site is clearer than hiding fields behind
// a builder, and it is not part of the runtime API.
#[allow(clippy::too_many_arguments)]
fn skill(
    id: &str,
    description: &str,
    activation_terms: &[&str],
    required_capabilities: &[Capability],
    compatible_roles: &[AgentRole],
    risk: RiskLevel,
    verification: &[&str],
    cost_hint: u32,
) -> SkillDefinition {
    SkillDefinition {
        id: id.to_string(),
        description: description.to_string(),
        activation_terms: activation_terms.iter().map(|s| s.to_string()).collect(),
        required_capabilities: required_capabilities.to_vec(),
        compatible_roles: compatible_roles.to_vec(),
        risk,
        verification: verification.iter().map(|s| s.to_string()).collect(),
        cost_hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_small_relevant_skill_set() {
        let registry = default_skills();
        let mut contract = DevelopmentContract::new(
            "Continue development: implement repository context routing and verify it with tests",
        );
        contract.required_capabilities = vec![Capability::ReadFile, Capability::RunTest];
        let route = route_skills(&registry, &contract, 3);
        assert!(route.selected.len() <= 3);
        assert!(route
            .selected
            .iter()
            .any(|item| item.skill_id == "design-context-fabric"));
        assert!(route
            .selected
            .iter()
            .any(|item| item.skill_id == "test-risk-first"));
    }

    #[test]
    fn risk_ceiling_filters_high_risk_skills() {
        let mut registry = SkillRegistry::new();
        registry
            .register(SkillDefinition {
                id: "danger".into(),
                description: "danger".into(),
                activation_terms: vec!["deploy".into()],
                required_capabilities: vec![],
                compatible_roles: vec![],
                risk: RiskLevel::Critical,
                verification: vec![],
                cost_hint: 1,
            })
            .unwrap();
        let contract = DevelopmentContract::new("deploy now");
        assert!(route_skills(&registry, &contract, 5).selected.is_empty());
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut registry = SkillRegistry::new();
        let s = SkillDefinition {
            id: "same".into(),
            description: String::new(),
            activation_terms: vec![],
            required_capabilities: vec![],
            compatible_roles: vec![],
            risk: RiskLevel::Low,
            verification: vec![],
            cost_hint: 1,
        };
        registry.register(s.clone()).unwrap();
        assert_eq!(
            registry.register(s),
            Err(SkillError::Duplicate("same".into()))
        );
    }
}
